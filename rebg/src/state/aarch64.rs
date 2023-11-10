use super::{Branching, GenericState, GenericStep, Instrument, MemoryOp, State, Step};
use crate::{
    arch::Arch,
    dis::{self, groups::Group},
    tracer::parser::{Message, RegisterMessage},
};
use bitflags::bitflags;

#[derive(Clone, Debug)]
pub struct Aarch64Step {
    state: Aarch64State,
    code: [u8; 4],
    address: u64,
    strace: Option<Box<str>>,
    memory_ops: Box<[MemoryOp]>,
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

    fn strace(&self) -> Option<&str> {
        self.strace.as_deref()
    }

    fn memory_ops(&self) -> &[MemoryOp] {
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
}

impl TryFrom<RegisterMessage> for Aarch64State {
    type Error = anyhow::Error;

    fn try_from(value: RegisterMessage) -> Result<Self, Self::Error> {
        let generic: GenericState<u64, 32> = GenericState::try_from(value)?;

        Ok(Self {
            regs: generic.regs,
            pc: generic.pc,
            flags: Aarch64Flags::from_bits_retain(generic.flags as u32),
        })
    }
}

impl TryFrom<&[Message]> for Aarch64Step {
    type Error = anyhow::Error;

    fn try_from(input: &[Message]) -> anyhow::Result<Self> {
        let generic: GenericStep<Aarch64State> = GenericStep::try_from(input)?;

        Ok(Self {
            state: generic.state,
            code: generic.code.try_into().unwrap(),
            address: generic.address,
            strace: generic.strace.map(|x| x.into()),
            memory_ops: generic.memory_ops.into_boxed_slice(),
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
            let mnem = insn.mnemonic.as_ref().unwrap().as_ref();
            let return_address = insn.address + insn.len as u64;

            let operand = {
                assert_eq!(insn.operands.len(), 1);
                insn.operands[0].clone()
            };

            let operand = match operand {
                capstone::arch::ArchOperand::Arm64Operand(o) => o,
                _ => panic!("nah"),
            };

            match mnem {
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
    use crate::{state::Aarch64Flags, tracer::parser::RegisterMessage};

    #[test]
    fn aarch64_state_deser() {
        let input = RegisterMessage {
            pc: 0,
            flags: 0,
            regs: [0x1234; 32].into(),
        };

        let result = Aarch64State::try_from(input);

        assert!(result.is_ok());

        assert_eq!(
            result.unwrap(),
            Aarch64State {
                regs: [0x1234; 32],
                pc: 0,
                flags: Aarch64Flags::empty(),
            }
        );
    }
}
