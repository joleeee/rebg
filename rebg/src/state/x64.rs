use std::str::FromStr;

use crate::{
    arch::Arch,
    dis::{self, groups::Group},
};

use super::{Branching, GenericState, GenericStep, Instrument, MemoryOp, State, Step};
use bitflags::bitflags;
use capstone::{
    arch::{
        self,
        x86::X86Reg::{
            X86_REG_R10, X86_REG_R11, X86_REG_R12, X86_REG_R13, X86_REG_R14, X86_REG_R15,
            X86_REG_R8, X86_REG_R9, X86_REG_RAX, X86_REG_RBP, X86_REG_RBX, X86_REG_RCX,
            X86_REG_RDI, X86_REG_RDX, X86_REG_RIP, X86_REG_RSI, X86_REG_RSP,
        },
    },
    RegId,
};

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

    fn arch(&self) -> Arch {
        Arch::X86_64
    }

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

    type INSTRUMENT = X64Instrument;

    fn instrument(&self) -> Self::INSTRUMENT {
        X64Instrument { step: self.clone() }
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

impl X64State {
    // Currently a best effort, only the normal register, only r prefix (ex. no eax)
    fn read_reg(&self, reg_id: RegId) -> Option<u64> {
        let idx = match reg_id.0 as u32 {
            X86_REG_RAX => 0,
            X86_REG_RBX => 1,
            X86_REG_RCX => 2,
            X86_REG_RDX => 3,
            X86_REG_RBP => 4,
            X86_REG_RSP => 5,
            X86_REG_RSI => 6,
            X86_REG_RDI => 7,
            X86_REG_R8 => 8,
            X86_REG_R9 => 9,
            X86_REG_R10 => 10,
            X86_REG_R11 => 11,
            X86_REG_R12 => 12,
            X86_REG_R13 => 13,
            X86_REG_R14 => 14,
            X86_REG_R15 => 15,
            X86_REG_RIP => return Some(self.pc),
            _ => return None,
        };

        Some(self.regs[idx])
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
pub struct X64Instrument {
    step: X64Step,
}
impl Instrument for X64Instrument {
    fn recover_branch(
        &self,
        _cs: &capstone::Capstone,
        insn: &dis::Instruction,
    ) -> Option<Branching> {
        let is_call_insn = insn.groups.iter().any(Group::is_call);
        let is_ret_insn = insn.groups.iter().any(Group::is_ret);

        assert!(!(is_call_insn && is_ret_insn));

        if is_call_insn {
            let return_address = insn.address + insn.len as u64;

            let operand = {
                assert_eq!(insn.operands.len(), 1);
                insn.operands[0].clone()
            };

            let operand = match operand {
                capstone::arch::ArchOperand::X86Operand(o) => o,
                _ => panic!("nah"),
            };

            match operand.op_type {
                arch::x86::X86OperandType::Reg(reg) => {
                    let address = match self.step.state().read_reg(reg) {
                        Some(v) => v,
                        None => {
                            eprintln!("Unknown register {:?}", reg);
                            return None;
                        }
                    };

                    Some(Branching::Call(address, return_address))
                }
                arch::x86::X86OperandType::Imm(imm) => {
                    Some(Branching::Call(imm as u64, return_address))
                }
                arch::x86::X86OperandType::Mem(mem) => {
                    assert_eq!(mem.segment().0, 0);

                    let base = match self.step.state().read_reg(mem.base()) {
                        Some(v) => v,
                        None => {
                            if mem.base().0 == 0 {
                                0
                            } else {
                                eprintln!("Unknown base register {:?}", mem.base());
                                return None;
                            }
                        }
                    } as i128;

                    let index = match self.step.state().read_reg(mem.index()) {
                        Some(v) => v,
                        None => {
                            if mem.index().0 == 0 {
                                0
                            } else {
                                eprintln!("Unknown index register {:?}", mem.index());
                                return None;
                            }
                        }
                    } as i128;

                    let scale = mem.scale() as i128;

                    let disp = mem.disp() as i128;

                    let target_address = base + index * scale + disp;
                    let target_address = target_address as u64;

                    Some(Branching::Call(target_address, return_address))
                }
                arch::x86::X86OperandType::Invalid => None,
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
