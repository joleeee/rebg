use crate::{
    arch::Arch,
    dis::{
        self,
        groups::Group,
        regs::{Reg, X64Reg},
    },
    tracer::parser::{Message, RegisterMessage},
};

use super::{Branching, GenericState, GenericStep, Instrument, MemoryOp, State, Step};
use bitflags::bitflags;
use capstone::{
    arch::{self},
    RegId,
};

#[derive(Clone, Debug)]
pub struct X64Step {
    state: X64State,
    code: Box<[u8]>,
    address: u64,
    strace: Option<Box<str>>,
    memory_ops: Box<[MemoryOp]>,
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

    fn strace(&self) -> Option<&str> {
        self.strace.as_deref()
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
}

impl X64State {
    fn read_reg(&self, reg_id: RegId) -> Option<u64> {
        let reg = Reg::from_num(Arch::X86_64, reg_id.0)?;

        // eax -> rax, w0 -> x0
        let reg = reg.canonical();

        if matches!(reg, Reg::X64Reg(X64Reg::Rip)) {
            return Some(self.pc);
        }

        Some(self.regs[reg.idx()?])
    }
}

impl TryFrom<RegisterMessage> for X64State {
    type Error = anyhow::Error;

    fn try_from(value: RegisterMessage) -> anyhow::Result<Self> {
        let generic: GenericState<u64, 16> = GenericState::try_from(value)?;

        Ok(Self {
            regs: generic.regs,
            pc: generic.pc,
            flags: X64Flags::from_bits_retain(generic.flags as u32),
        })
    }
}

impl TryFrom<&[Message]> for X64Step {
    type Error = anyhow::Error;

    fn try_from(input: &[Message]) -> anyhow::Result<Self> {
        let generic: GenericStep<X64State> = GenericStep::try_from(input)?;

        Ok(Self {
            state: generic.state,
            code: generic.code.into_boxed_slice(),
            address: generic.address,
            strace: generic.strace.map(|x| x.into_boxed_str()),
            memory_ops: generic.memory_ops.into_boxed_slice(),
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
    use crate::{state::X64Flags, tracer::parser::RegisterMessage};

    #[test]
    fn aarch64_state_deser() {
        let input = RegisterMessage {
            pc: 0,
            flags: 0,
            regs: [0x4321; 16].into(),
        };

        let result = X64State::try_from(input);

        assert!(result.is_ok());

        assert_eq!(
            result.unwrap(),
            X64State {
                regs: [0x4321; 16],
                pc: 0,
                flags: X64Flags::empty(),
            }
        );
    }
}
