use std::str::FromStr;

use super::{Branching, GenericState, GenericStep, Instrument, State, Step};
use crate::{
    arch::Arch,
    dis::{self, groups::Group, regs::Reg},
};
use bitflags::bitflags;

#[derive(Clone, Debug)]
pub struct Aarch64Step {
    state: Aarch64State,
    code: [u8; 4],
    address: u64,
    strace: Option<String>,
    memory_ops: Vec<super::MemoryOp>,
}

impl Step<32> for Aarch64Step {
    type STATE = Aarch64State;
    type INSTRUMENT = Aarch64Instrument;

    fn arch(&self) -> Arch {
        Arch::ARM64
    }

    fn code(&self) -> &[u8] {
        &self.code[..]
    }

    fn state(&self) -> &Aarch64State {
        &self.state
    }

    fn address(&self) -> u64 {
        self.address
    }

    fn strace(&self) -> Option<&String> {
        self.strace.as_ref()
    }

    fn memory_ops(&self) -> &[super::MemoryOp] {
        &self.memory_ops[..]
    }

    fn instrument(&self) -> Self::INSTRUMENT {
        Aarch64Instrument { step: self.clone() }
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

    fn reg_name_idx(i: usize) -> &'static str {
        [
            "x0", "x1", "x2", "x3", "x4", "x5", "x6", "x7", "x8", "x9", "x10", "x11", "x12", "x13",
            "x14", "x15", "x16", "x17", "x18", "x19", "x20", "x21", "x22", "x23", "x24", "x25",
            "x26", "x27", "x28", "x29", "x30", "sp", "xzr",
        ][i]
    }

    fn reg_idx(reg: Reg) -> Option<usize> {
        let reg = reg.canonical();
        Some(match reg {
            Reg::Aarch64Reg(v) => match v {
                dis::regs::Aarch64Reg::X0 => 0,
                dis::regs::Aarch64Reg::X1 => 1,
                dis::regs::Aarch64Reg::X2 => 2,
                dis::regs::Aarch64Reg::X3 => 3,
                dis::regs::Aarch64Reg::X4 => 4,
                dis::regs::Aarch64Reg::X5 => 5,
                dis::regs::Aarch64Reg::X6 => 6,
                dis::regs::Aarch64Reg::X7 => 7,
                dis::regs::Aarch64Reg::X8 => 8,
                dis::regs::Aarch64Reg::X9 => 9,
                dis::regs::Aarch64Reg::X10 => 10,
                dis::regs::Aarch64Reg::X11 => 11,
                dis::regs::Aarch64Reg::X12 => 12,
                dis::regs::Aarch64Reg::X13 => 13,
                dis::regs::Aarch64Reg::X14 => 14,
                dis::regs::Aarch64Reg::X15 => 15,
                dis::regs::Aarch64Reg::X16 => 16,
                dis::regs::Aarch64Reg::X17 => 17,
                dis::regs::Aarch64Reg::X18 => 18,
                dis::regs::Aarch64Reg::X19 => 19,
                dis::regs::Aarch64Reg::X20 => 20,
                dis::regs::Aarch64Reg::X21 => 21,
                dis::regs::Aarch64Reg::X22 => 22,
                dis::regs::Aarch64Reg::X23 => 23,
                dis::regs::Aarch64Reg::X24 => 24,
                dis::regs::Aarch64Reg::X25 => 25,
                dis::regs::Aarch64Reg::X26 => 26,
                dis::regs::Aarch64Reg::X27 => 27,
                dis::regs::Aarch64Reg::X28 => 28,
                dis::regs::Aarch64Reg::Fp => 29,
                dis::regs::Aarch64Reg::Lr => 30,
                _ => return None,
            },
            Reg::X64Reg(_) => return None,
        })
    }

    fn reg_name(reg: Reg) -> Option<&'static str> {
        let idx = Self::reg_idx(reg)?;
        Some(Self::reg_name_idx(idx))
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

impl TryFrom<&[String]> for Aarch64Step {
    type Error = anyhow::Error;

    fn try_from(input: &[String]) -> anyhow::Result<Self> {
        let generic: GenericStep<Aarch64State> = GenericStep::try_from(input)?;

        Ok(Self {
            state: generic.state,
            code: generic.code.try_into().unwrap(),
            address: generic.address,
            strace: generic.strace,
            memory_ops: generic.memory_ops,
        })
    }
}

pub struct Aarch64Instrument {
    step: Aarch64Step,
}
impl Instrument for Aarch64Instrument {
    fn recover_branch(
        &self,
        cs: &capstone::Capstone,
        insn: &dis::Instruction,
    ) -> Option<Branching> {
        let is_call_insn = insn.groups.iter().any(Group::is_call);
        let is_ret_insn = insn.groups.iter().any(Group::is_ret);

        assert!(!(is_call_insn && is_ret_insn));

        if is_call_insn {
            let mnem = insn.mnemonic.as_ref().unwrap();
            let return_address = insn.address + insn.len as u64;

            let operand = {
                assert_eq!(insn.operands.len(), 1);
                insn.operands[0].clone()
            };

            let operand = match operand {
                capstone::arch::ArchOperand::Arm64Operand(o) => o,
                _ => panic!("nah"),
            };

            match mnem.as_str() {
                "bl" => {
                    let operand_val = match operand.op_type {
                        capstone::arch::arm64::Arm64OperandType::Imm(val) => val,
                        _ => panic!("bl without imm argument {:?}", operand.op_type),
                    };

                    let to = operand_val as u64;

                    Some(Branching::Call(to, return_address))
                }
                "blr" => {
                    let operand_nr = match operand.op_type {
                        capstone::arch::arm64::Arm64OperandType::Reg(reg) => reg,
                        _ => panic!("blr without reg argument {:?}", operand.op_type),
                    };

                    let operand_str = cs.reg_name(operand_nr).unwrap();

                    // hacky af
                    let reg_nr = operand_str
                        .strip_prefix('x')
                        .unwrap()
                        .parse::<u32>()
                        .unwrap();

                    // TODO: think about if we should actually use previous state instead of this current/next state?
                    let target_address = self.step.state().regs()[reg_nr as usize];

                    Some(Branching::Call(target_address, return_address))
                }
                _x => {
                    eprintln!("Unknown Aarch64 call mnemonic: {}", _x);
                    None
                }
            }
        } else if is_ret_insn {
            Some(Branching::Return)
        } else {
            None
        }
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
