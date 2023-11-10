use super::{get_next_step, parser::Message, ParsedStep, Tracer, TracerCmd};
use crate::{arch::Arch, state::Step};
use std::{
    fmt,
    io::BufReader,
    marker::PhantomData,
    net::{TcpListener, TcpStream},
    path::Path,
    process::Child,
};
use tracing::info;

pub struct QEMU {}

impl<STEP, const N: usize> Tracer<STEP, N> for QEMU
where
    STEP: Step<N> + Send + 'static + fmt::Debug,
    STEP: for<'a> TryFrom<&'a [Message], Error = anyhow::Error>,
{
    type ITER = QEMUParser<STEP, N>;

    fn command(&self, executable: &Path, arch: Arch, localhost: &str) -> TracerCmd<STEP, N> {
        let qemu = arch.qemu_user_bin().to_string();

        let options = vec![
            String::from("-rebglog"),
            String::from("/dev/null"),
            String::from("-rebgtcp"),
            format!("{localhost}:1337"),
            String::from("-one-insn-per-tb"),
            String::from("-d"),
            String::from("in_asm,strace"),
            executable.to_str().unwrap().to_string(),
        ];

        TracerCmd {
            program: qemu,
            args: options,
            _step: PhantomData,
        }
    }

    /// Takes output from the process and parses it to steps
    fn parse(&self, proc: std::process::Child) -> Self::ITER {
        QEMUParser::new(proc)
    }
}

// having the bounds here mean the STATE has to be the same type as the STATE type in QEMU, which
// means less room for error and automatic inference of this type

#[derive(Debug)]
pub struct QEMUParser<STEP, const N: usize> {
    /// None when done
    proc: Option<Child>,

    reader: BufReader<TcpStream>,
    _phantom: PhantomData<STEP>,
}

impl<STEP, const N: usize> Iterator for QEMUParser<STEP, N>
where
    STEP: Step<N> + Send + 'static + fmt::Debug,
    STEP: for<'a> TryFrom<&'a [Message], Error = anyhow::Error>,
{
    type Item = ParsedStep<STEP, N>;

    fn next(&mut self) -> Option<Self::Item> {
        get_next_step(&mut self.reader, &mut self.proc)
    }
}

impl<STEP, const N: usize> QEMUParser<STEP, N> {
    fn new(proc: Child) -> Self {
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
