use core::fmt;

use bitflags::Flags;
use num_traits::Num;

pub mod aarch64;
pub use aarch64::{Aarch64Flags, Aarch64State, Aarch64Step};
pub mod x64;
pub use x64::{X64Flags, X64State, X64Step};

use crate::{
    arch::Arch,
    dis::{self},
    tracer::parser::{Message, RegisterMessage},
};

/// A single step in the trace.
pub trait Step<const N: usize>: Clone + std::marker::Send + 'static {
    type STATE: State<N>;
    type INSTRUMENT: Instrument;
    // static architecture
    fn arch(&self) -> Arch;

    fn code(&self) -> &[u8];
    // this also contains the pc
    fn state(&self) -> &Self::STATE;
    // sometimes they differ, though, so also keep address
    fn address(&self) -> u64;
    fn strace(&self) -> Option<&str>;
    fn memory_ops(&self) -> &[MemoryOp];

    fn instrument(&self) -> Self::INSTRUMENT;
}

/// Register values and flags
pub trait State<const N: usize>: Clone {
    type FLAGS: Flags + Clone + Copy + fmt::Debug;
    fn pc(&self) -> u64;
    fn regs(&self) -> &[u64; N];
    fn flags(&self) -> &Self::FLAGS;
}

pub trait Instrument {
    fn recover_branch(&self, cs: &capstone::Capstone, insn: &dis::Instruction)
        -> Option<Branching>;
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Branching {
    // (to, return to)
    Call(u64, u64),
    Return,
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct Instrumentation {
    pub branch: Option<Branching>,
    pub disassembly: String,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MemoryOpKind {
    Read,
    Write,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MemoryValue {
    Byte(u8),
    Word(u16),
    Dword(u32),
    Qword(u64),
}

impl MemoryValue {
    pub fn as_u64(&self) -> u64 {
        match &self {
            MemoryValue::Byte(b) => *b as u64,
            MemoryValue::Word(w) => *w as u64,
            MemoryValue::Dword(d) => *d as u64,
            MemoryValue::Qword(q) => *q,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MemoryOp {
    pub address: u64,
    pub kind: MemoryOpKind,
    pub value: MemoryValue,
}

// nasty shit
// ==========
struct GenericState<TYPE, const N: usize> {
    regs: [TYPE; N],
    pc: TYPE,
    flags: TYPE,
}

impl<TYPE, const N: usize> TryFrom<RegisterMessage> for GenericState<TYPE, N>
where
    TYPE: Num + Copy,
    TYPE: TryFrom<u64>,
    TYPE: std::fmt::Debug,
    <TYPE as Num>::FromStrRadixErr: fmt::Debug,
    <TYPE as TryFrom<u64>>::Error: fmt::Debug,
{
    type Error = anyhow::Error;

    fn try_from(input: RegisterMessage) -> anyhow::Result<Self> {
        let RegisterMessage { pc, flags, regs } = input;

        let pc = pc.try_into().unwrap();
        let flags = flags.try_into().unwrap();

        let regs = regs
            .iter()
            .map(|&v| v.try_into().unwrap())
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        Ok(Self { regs, pc, flags })
    }
}

pub struct GenericStep<STATE: TryFrom<RegisterMessage>> {
    state: STATE,
    code: Vec<u8>,
    address: u64,
    strace: Option<String>,
    memory_ops: Vec<MemoryOp>,
}

impl<STATE> TryFrom<&[Message]> for GenericStep<STATE>
where
    STATE: TryFrom<RegisterMessage, Error = anyhow::Error>,
{
    type Error = anyhow::Error;

    fn try_from(value: &[Message]) -> Result<Self, Self::Error> {
        let mut s_state = None;
        let mut s_address = None;
        let mut s_code = None;

        let mut strace = None;
        let mut strace_result = None;

        let mut memory_ops = vec![];

        for m in value {
            match m {
                Message::Address(a) => s_address = Some(*a),
                Message::Code(c) => s_code = Some(c.to_vec()),
                Message::Registers(regs) => {
                    s_state = if let Some(existing) = s_state {
                        Some(existing)
                    } else {
                        Some(STATE::try_from(regs.clone()).unwrap())
                    }
                }
                Message::Flags(_) => todo!(),
                Message::Load(adr, value, size) | Message::Store(adr, value, size) => {
                    let value = match size {
                        1 => MemoryValue::Byte(*value as u8),
                        2 => MemoryValue::Word(*value as u16),
                        4 => MemoryValue::Dword(*value as u32),
                        8 => MemoryValue::Qword(*value),
                        _ => return Err(anyhow::anyhow!("unknown value size: {} bytes", size)),
                    };

                    let kind = match m {
                        Message::Load(_, _, _) => MemoryOpKind::Read,
                        Message::Store(_, _, _) => MemoryOpKind::Write,
                        _ => unreachable!(),
                    };

                    memory_ops.push(MemoryOp {
                        address: *adr,
                        kind,
                        value,
                    });
                }
                Message::Syscall(s) => strace = Some(s.to_string()),
                Message::SyscallResult(s) => strace_result = Some(s.to_string()),
                Message::Debug(_) => {}
                Message::LibLoad(_, _, _) | Message::Separator => {
                    panic!("really shouldnt happen: {:x?}", m)
                }
            }
        }

        let strace = strace.map(|mut strace| {
            strace.push_str(&strace_result.unwrap_or_default());
            strace
        });

        let address = s_address.unwrap();
        let code = s_code.unwrap();
        let state = s_state.unwrap();

        Ok(Self {
            state,
            code,
            address,
            strace,
            memory_ops,
        })
    }
}
