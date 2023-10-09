use core::fmt;
use std::str::FromStr;

use anyhow::Context;
use bitflags::Flags;
use capstone::{Insn, InsnDetail};
use hex::FromHex;
use num_traits::Num;

pub mod aarch64;
pub use aarch64::{Aarch64Flags, Aarch64State, Aarch64Step};
pub mod x64;
pub use x64::{X64Flags, X64State, X64Step};

use crate::arch::Arch;

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
    fn strace(&self) -> Option<&String>;
    fn memory_ops(&self) -> &[MemoryOp];

    fn instrument(&self) -> Self::INSTRUMENT;
}

/// Register values and flags
pub trait State<const N: usize>: Clone {
    type FLAGS: Flags + Clone + Copy + fmt::Debug;
    fn pc(&self) -> u64;
    fn regs(&self) -> &[u64; N];
    fn reg_name(i: usize) -> &'static str;
    fn flags(&self) -> &Self::FLAGS;
}

pub trait Instrument {
    fn recover_branch(
        &self,
        cs: &capstone::Capstone,
        insn: &Insn,
        detail: &InsnDetail,
    ) -> Option<Branching>;
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
    #[allow(dead_code)]
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
            MemoryValue::Qword(q) => *q as u64,
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

impl<TYPE, const N: usize> FromStr for GenericState<TYPE, N>
where
    TYPE: Num + Copy,
    <TYPE as Num>::FromStrRadixErr: fmt::Debug,
{
    type Err = anyhow::Error;

    fn from_str(input: &str) -> anyhow::Result<Self> {
        let regs = input
            .split('|')
            .map(|data| data.split_once('='))
            .map(Option::unwrap)
            .map(|(name, value)| (name.trim(), TYPE::from_str_radix(value, 16).unwrap()));

        let mut registers: [Option<TYPE>; N] = [None; N];
        let mut pc = None;
        let mut flags = None;

        for (name, value) in regs {
            match name {
                "pc" => {
                    pc = Some(value);
                }
                "flags" => {
                    flags = Some(value);
                }
                _ => {
                    let index = name.strip_prefix('r').context("missing register prefix")?;
                    let index = usize::from_str_radix(index, 10)?;
                    registers[index] = Some(value);
                }
            }
        }

        let pc = pc.unwrap();
        let flags = flags.unwrap();

        if registers.contains(&None) {
            return Err(anyhow::anyhow!("register not set"));
        }
        let registers = registers.map(Option::unwrap);

        Ok(Self {
            regs: registers,
            pc,
            flags,
        })
    }
}

pub struct GenericStep<STATE: FromStr> {
    state: STATE,
    code: Vec<u8>,
    address: u64,
    strace: Option<String>,
    memory_ops: Vec<MemoryOp>,
}

impl<STATE> TryFrom<&[String]> for GenericStep<STATE>
where
    STATE: FromStr<Err = anyhow::Error>,
{
    type Error = anyhow::Error;

    fn try_from(input: &[String]) -> anyhow::Result<Self> {
        let mut s_state = None;
        let mut s_address = None;
        let mut s_code = None;

        let mut partial_strace = None;
        let mut strace = None;

        let mut memory_ops = vec![];

        for line in input {
            let (what, content) = match line.split_once('|') {
                Some(x) => x,
                None => continue,
            };

            match what {
                "regs" => {
                    s_state = if let Some(prev) = s_state {
                        Some(prev)
                    } else {
                        Some(STATE::from_str(content)?)
                    };
                }
                "address" => {
                    s_address = Some(
                        u64::from_str_radix(content, 16).map_err(Into::<anyhow::Error>::into)?,
                    );
                }
                "code" => {
                    s_code = Some(Vec::from_hex(content).unwrap());
                }
                "strace" => {
                    let content = {
                        let (pid, data) = content.split_once('|').expect("missing pid");
                        assert!(pid.starts_with("pid="));
                        data
                    };

                    let content = content
                        .strip_prefix("contents=")
                        .expect("missing content= prefix");

                    if let Some(data) = content.strip_suffix("|sdone") {
                        strace = Some(data.to_string())
                    } else {
                        partial_strace = Some(content)
                    }
                }
                "st" => {
                    let (bits, rest) = content.split_once('|').unwrap();

                    let bits = i32::from_str_radix(bits, 10).unwrap();

                    let (ptr, val) = rest.split_once('|').unwrap();
                    let ptr = ptr.strip_prefix("0x").unwrap();
                    let ptr = u64::from_str_radix(ptr, 16).unwrap();

                    let value = match bits {
                        8 => MemoryValue::Byte(u8::from_str_radix(val, 16)?),
                        16 => MemoryValue::Word(u16::from_str_radix(val, 16)?),
                        32 => MemoryValue::Dword(u32::from_str_radix(val, 16)?),
                        64 => MemoryValue::Qword(u64::from_str_radix(val, 16)?),
                        _ => return Err(anyhow::anyhow!("unknown value size: {} bits", bits)),
                    };

                    memory_ops.push(MemoryOp {
                        address: ptr,
                        kind: MemoryOpKind::Write,
                        value,
                    });
                }
                _ => {
                    // might be the end of an strace
                    if let Some(data) = line.strip_suffix("|sdone") {
                        strace = Some(
                            partial_strace
                                .expect("extending strace without a start")
                                .to_string()
                                + data,
                        );
                        partial_strace = None;
                    } else {
                        panic!("unknown data '{}'", line)
                    }
                }
            }
        }

        let address = s_address.unwrap();
        let code = s_code.unwrap();
        let state = s_state.unwrap();

        assert!(partial_strace.is_none());

        Ok(Self {
            state,
            code,
            address,
            strace,
            memory_ops,
        })
    }
}
