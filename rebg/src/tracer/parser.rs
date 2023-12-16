use std::{
    fmt,
    io::{BufReader, Read},
    marker::PhantomData,
    net::{TcpListener, TcpStream},
    process::Child,
};

use tracing::{info, trace};

use crate::state::Step;

use super::ParsedStep;

#[derive(Debug)]
enum Header {
    Separator = 0x55,
    Libload = 0xee,
    Address = 0xaa,
    Code = 0xff,
    Load = 0x33,
    Store = 0x44,
    Registers = 0x77,
    Syscall = 0x99,
    SyscallResult = 0x9a,
    Debug = 0xdd,
}

impl TryFrom<u8> for Header {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x55 => Ok(Self::Separator),
            0xee => Ok(Self::Libload),
            0xaa => Ok(Self::Address),
            0xff => Ok(Self::Code),
            0x33 => Ok(Self::Load),
            0x44 => Ok(Self::Store),
            0x77 => Ok(Self::Registers),
            0x99 => Ok(Self::Syscall),
            0x9a => Ok(Self::SyscallResult),
            0xdd => Ok(Self::Debug),
            _ => Err(()),
        }
    }
}

impl Header {
    fn deserialize<R: Read>(&self, reader: &mut R) -> Message {
        let mut buf_8 = [0; 8];
        let mut next_u64 = |reader: &mut R| {
            reader.read_exact(&mut buf_8).unwrap();
            u64::from_le_bytes(buf_8)
        };

        match self {
            Header::Libload => {
                let len = next_u64(reader);

                let name = {
                    let mut strbuf = vec![0; len as usize];
                    reader.read_exact(&mut strbuf).unwrap();
                    String::from_utf8(strbuf).unwrap().into_boxed_str()
                };

                let from = next_u64(reader);
                let to = next_u64(reader);

                Message::LibLoad(name, from, to)
            }
            Header::Separator => Message::Separator,
            Header::Address => Message::Address(next_u64(reader)),
            Header::Code => {
                let len = next_u64(reader);

                let mut code = vec![0; len as usize];
                reader.read_exact(&mut code).unwrap();

                Message::Code(code.into_boxed_slice())
            }
            Header::Load => {
                let size = {
                    let mut bytebuf = [0; 1];
                    reader.read_exact(&mut bytebuf).unwrap();
                    u8::from_le_bytes(bytebuf)
                };

                let adr = next_u64(reader);
                let value = next_u64(reader);

                Message::Load(adr, value, size)
            }
            Header::Store => {
                let size = {
                    let mut bytebuf = [0; 1];
                    reader.read_exact(&mut bytebuf).unwrap();
                    u8::from_le_bytes(bytebuf)
                };

                let adr = next_u64(reader);
                let value = next_u64(reader);

                Message::Store(adr, value, size)
            }
            Header::Registers => {
                let count = {
                    let mut bytebuf = [0; 1];
                    reader.read_exact(&mut bytebuf).unwrap();
                    u8::from_le_bytes(bytebuf) as usize
                };

                let flags = next_u64(reader);
                let pc = next_u64(reader);

                let mut regs = vec![0; count];

                for reg in regs.iter_mut() {
                    *reg = next_u64(reader);
                }

                Message::Registers(RegisterMessage {
                    pc,
                    flags,
                    regs: regs.into_boxed_slice(),
                })
            }
            Header::Syscall => {
                let len = next_u64(reader);

                let string = {
                    let mut strbuf = vec![0; len as usize];
                    reader.read_exact(&mut strbuf).unwrap();
                    String::from_utf8(strbuf).unwrap().into_boxed_str()
                };

                Message::Syscall(string)
            }
            Header::SyscallResult => {
                let len = next_u64(reader);

                let string = {
                    let mut strbuf = vec![0; len as usize];
                    reader.read_exact(&mut strbuf).unwrap();
                    String::from_utf8(strbuf).unwrap().into_boxed_str()
                };

                Message::SyscallResult(string)
            }
            Header::Debug => {
                let len = next_u64(reader);

                let string = {
                    let mut strbuf = vec![0; len as usize];
                    reader.read_exact(&mut strbuf).unwrap();
                    String::from_utf8(strbuf).unwrap().into_boxed_str()
                };

                Message::Debug(string)
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    LibLoad(Box<str>, u64, u64),
    Separator,
    Address(u64),
    Code(Box<[u8]>),
    Registers(RegisterMessage),
    Flags(u64),
    Load(u64, u64, u8),
    Store(u64, u64, u8),
    Syscall(Box<str>),
    SyscallResult(Box<str>),
    Debug(Box<str>)
}

#[derive(Clone, Debug)]
pub struct RegisterMessage {
    pub pc: u64,
    pub flags: u64,
    pub regs: Box<[u64]>,
}

pub fn get_next_message<R: Read>(reader: &mut R) -> Option<Message> {
    let mut header = [0; 1];

    reader.read_exact(&mut header).ok()?;

    let header: Header = header[0].try_into().unwrap_or_else(|_| {
        let mut data = [0; 16];
        reader.read_exact(&mut data).unwrap();
        println!("header: {:02x}. following: {:02x?}", header[0], data);
        panic!();
    });

    let msg = header.deserialize(reader);

    trace!("{:x?}", msg);

    Some(msg)
}

pub fn get_next_step<R: Read, STEP, const N: usize>(
    reader: &mut R,
    proc: &mut Option<Child>,
) -> Option<ParsedStep<STEP, N>>
where
    STEP: Step<N> + Send + 'static + fmt::Debug,
    STEP: for<'a> TryFrom<&'a [Message], Error = anyhow::Error>,
{
    #[allow(clippy::question_mark)]
    if proc.is_none() {
        return None;
    }

    let mut msgs = vec![];

    while let Some(m) = get_next_message(reader) {
        if matches!(m, Message::Separator) {
            break;
        }

        msgs.push(m);
    }

    // if there are no msgs, we're done!
    if msgs.is_empty() {
        let mut my_proc = None;
        std::mem::swap(proc, &mut my_proc);
        let my_proc = my_proc.unwrap();

        // make sure it closed gracefully
        let result = my_proc.wait_with_output().unwrap();

        return Some(ParsedStep::Final(result));
    }

    if matches!(msgs[0], Message::LibLoad(_, _, _)) {
        let map = msgs
            .into_iter()
            .flat_map(|m| match m {
                Message::LibLoad(name, from, to) => Some((name.to_string(), (from, to))),
                // _ => panic!("Got libload and some other junk!"),
                _ => None,
            })
            .collect();

        return Some(ParsedStep::LibLoad(map));
    }

    // otherwise, it's just a step :)

    let s = STEP::try_from(&msgs).unwrap();
    Some(ParsedStep::TraceStep(s))
}

#[derive(Debug)]
pub struct GenericParser<STEP, const N: usize> {
    /// None when done
    proc: Option<Child>,

    reader: BufReader<TcpStream>,
    _phantom: PhantomData<STEP>,
}

impl<STEP, const N: usize> Iterator for GenericParser<STEP, N>
where
    STEP: Step<N> + Send + 'static + fmt::Debug,
    STEP: for<'a> TryFrom<&'a [Message], Error = anyhow::Error>,
{
    type Item = ParsedStep<STEP, N>;

    fn next(&mut self) -> Option<Self::Item> {
        get_next_step(&mut self.reader, &mut self.proc)
    }
}

impl<STEP, const N: usize> GenericParser<STEP, N> {
    pub fn new(proc: Child) -> Self {
        let listener = TcpListener::bind("[::]:1337").unwrap();

        info!("Waiting for connection...");
        let con = listener.incoming().next().unwrap().unwrap();
        info!("Connected! {:?}", con);
        drop(listener); // close the socket, keep the connection, me THINKS

        let reader = BufReader::new(con);

        Self {
            proc: Some(proc),
            reader,
            _phantom: PhantomData,
        }
    }
}
