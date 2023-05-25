use std::str::FromStr;

use super::{GenericState, GenericStep, State, Step};
use bitflags::bitflags;

#[derive(Clone, Copy, Debug)]
pub struct Aarch64Step {
    state: Aarch64State,
    code: [u8; 4],
    address: u64,
}

impl Step<32> for Aarch64Step {
    type STATE = Aarch64State;

    fn code(&self) -> &[u8] {
        &self.code[..]
    }

    fn state(&self) -> &Aarch64State {
        &self.state
    }

    fn address(&self) -> u64 {
        self.address
    }
}

bitflags! {
    // this should be the same as aarch32?
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct Aarch64Flags: u32 {
        const NEGATIVE = 1 << 31;
        const ZERO = 1 << 30;
        const CARRY = 1 << 29;
        const OVERFLOW = 1 << 28;
        const CUMULATIVE_SATURATION = 1 << 27;
        // 26:24 reserved
        // these only apply if implemented, otherwise reserved:
        const SSBS = 1 << 23;
        const PAN = 1 << 22;
        const DIT = 1 << 21;
        // 20 reserved
        // GE [19:16]
        const GE_0 = 1 << 19;
        const GE_1 = 1 << 18;
        const GE_2 = 1 << 17;
        const GE_3 = 1 << 16;
        // 15:10 reserved
        const ENDIANNESS = 1 << 9; // (0 = little, 1 = big)
        const A = 1 << 8; // SError interrupt mask
        const IRQ = 1 << 7; // (0 = exception not masked, 1 = exception masked)
        const FIQ = 1 << 6; // (0 = exception not masked, 1 = exception masked)
        // 5:4 reserved
        const PE_0 = 1 << 3; // see the manual
        const PE_1 = 1 << 2;
        const PE_2 = 1 << 1;
        const PE_3 = 1 << 0;
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Aarch64State {
    regs: [u64; 32],
    pc: u64,
    flags: Aarch64Flags,
}

impl State<32> for Aarch64State {
    type FLAGS = Aarch64Flags;

    fn pc(&self) -> u64 {
        self.pc
    }

    fn regs(&self) -> &[u64; 32] {
        &self.regs
    }

    fn flags(&self) -> &Aarch64Flags {
        &self.flags
    }

    fn reg_name(i: usize) -> &'static str {
        [
            "x0", "x1", "x2", "x3", "x4", "x5", "x6", "x7", "x8", "x9", "x10", "x11", "x12", "x13",
            "x14", "x15", "x16", "x17", "x18", "x19", "x20", "x21", "x22", "x23", "x24", "x25",
            "x26", "x27", "x28", "x29", "x30", "sp", "xzr",
        ][i]
    }
}

impl FromStr for Aarch64State {
    type Err = anyhow::Error;

    fn from_str(input: &str) -> anyhow::Result<Self> {
        let generic: GenericState<u64, 32> = GenericState::from_str(input)?;

        Ok(Self {
            regs: generic.regs,
            pc: generic.pc,
            flags: Aarch64Flags::from_bits_retain(generic.flags as u32),
        })
    }
}

impl FromStr for Aarch64Step {
    type Err = anyhow::Error;

    fn from_str(input: &str) -> anyhow::Result<Self> {
        let generic: GenericStep<Aarch64State> = GenericStep::from_str(input)?;

        Ok(Self {
            state: generic.state,
            code: generic.code.try_into().unwrap(),
            address: generic.address,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Aarch64State;
    use crate::state::Aarch64Flags;
    use std::str::FromStr;

    #[test]
    fn aarch64_state_from_string() {
        let input = "r0=0|r1=0|r2=0|r3=0|r4=0|r5=0|r6=0|r7=0|r8=0|r9=0|r10=0|r11=0|r12=0|r13=0|r14=0|r15=0|r16=0|r17=0|r18=0|r19=0|r20=0|r21=0|r22=0|r23=0|r24=0|r25=0|r26=0|r27=0|r28=0|r29=0|r30=0|r31=0|pc=0|flags=0";

        let result = Aarch64State::from_str(input);

        assert!(result.is_ok());

        assert_eq!(
            result.unwrap(),
            Aarch64State {
                regs: [0; 32],
                pc: 0,
                flags: Aarch64Flags::empty(),
            }
        );
    }
}
