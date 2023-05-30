use std::str::FromStr;

use super::{GenericState, GenericStep, MemoryOp, State, Step};
use bitflags::bitflags;

#[derive(Clone, Debug)]
pub struct X64Step {
    state: X64State,
    code: Vec<u8>,
    address: u64,
    strace: Option<String>,
    memory_ops: Vec<MemoryOp>,
}

impl Step<16> for X64Step {
    type STATE = X64State;

    fn code(&self) -> &[u8] {
        &self.code[..]
    }

    fn state(&self) -> &X64State {
        &self.state
    }

    fn address(&self) -> u64 {
        self.address
    }

    fn strace(&self) -> Option<&String> {
        self.strace.as_ref()
    }

    fn memory_ops(&self) -> &[super::MemoryOp] {
        &self.memory_ops
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct X64Flags: u32 {
        const CARRY = 1 << 0;
        const PARITY = 1 << 2;
        const AUXILIARY_CARRY = 1 << 4;
        const ZERO = 1 << 6;
        const SIGN = 1 << 7;
        const TRAP = 1 << 8;
        const INTERRUPT_EN = 1 << 9;
        const DIRECTION = 1 << 10;
        const OVERFLOW = 1 << 11;
        const IOPL_0 = 1 << 12;
        const IOPL_1 = 1 << 13;
        const NESTED_TASK = 1 << 14;
        const RESUME = 1 << 16;
        const VIRTUAL_8086 = 1 << 17;
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct X64State {
    regs: [u64; 16],
    pc: u64,
    flags: X64Flags,
}

impl State<16> for X64State {
    type FLAGS = X64Flags;

    fn pc(&self) -> u64 {
        self.pc
    }

    fn regs(&self) -> &[u64; 16] {
        &self.regs
    }

    fn flags(&self) -> &X64Flags {
        &self.flags
    }

    fn reg_name(i: usize) -> &'static str {
        [
            "rax", "rbx", "rcx", "rdx", "rbp", "rsp", "rsi", "rdi", "r8", "r9", "r10", "r11",
            "r12", "r13", "r14", "r15",
        ][i]
    }
}

impl FromStr for X64State {
    type Err = anyhow::Error;

    fn from_str(input: &str) -> anyhow::Result<Self> {
        let generic: GenericState<u64, 16> = GenericState::from_str(input)?;

        Ok(Self {
            regs: generic.regs,
            pc: generic.pc,
            flags: X64Flags::from_bits_retain(generic.flags as u32),
        })
    }
}

impl TryFrom<&[String]> for X64Step {
    type Error = anyhow::Error;

    fn try_from(input: &[String]) -> anyhow::Result<Self> {
        let generic: GenericStep<X64State> = GenericStep::try_from(input)?;

        Ok(Self {
            state: generic.state,
            code: generic.code,
            address: generic.address,
            strace: generic.strace,
            memory_ops: generic.memory_ops,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::X64State;
    use crate::state::X64Flags;
    use std::str::FromStr;

    #[test]
    fn aarch64_state_from_string() {
        let input = "r0=0|r1=0|r2=0|r3=0|r4=0|r5=0|r6=0|r7=0|r8=0|r9=0|r10=0|r11=0|r12=0|r13=0|r14=0|r15=0|pc=0|flags=0";

        let result = X64State::from_str(input);

        assert!(result.is_ok());

        assert_eq!(
            result.unwrap(),
            X64State {
                regs: [0; 16],
                pc: 0,
                flags: X64Flags::empty(),
            }
        );
    }
}
