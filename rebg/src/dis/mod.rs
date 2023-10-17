//! This contains everything for dissassembling\
//! currently we just have support for capstone, but this code lets us hide some
//! of the internals of capstone

// we also want to abstract away some things, like "pseudoregisters" like `eax`
// and `ah` which are really part of `rax`. in some situations like when
// highlighting what registers changed, we dont care about `eax` or `ah`, we
// want `rax` (or some other canonical ref)

pub mod groups;
pub mod regs;

use crate::arch::Arch;
use std::rc::Rc;
use thiserror::Error;

use self::{groups::Group, regs::Reg};

pub struct Dis {
    pub arch: Arch,
    pub cs: Rc<capstone::Capstone>,
}

pub struct Instruction {
    pub len: usize,
    pub address: u64,

    // registers used
    pub read: Vec<Reg>,
    pub write: Vec<Reg>,

    // string stuff
    pub mnemonic: Option<String>,
    pub op_str: Option<String>,

    // blah
    pub operands: Vec<capstone::arch::ArchOperand>,
    pub groups: Vec<Group>,
}

#[derive(Error, Debug)]
pub enum DisError {
    #[error("capstone")]
    Capstone(#[from] capstone::Error),
    #[error("invalid groupid")]
    NoGroup(u8),
    #[error("invalid regid")]
    NoReg(u16),
}

impl Dis {
    pub fn disassemble_one(&self, code: &[u8], pc: u64) -> Result<Instruction, DisError> {
        let instructions = self.cs.disasm_count(code, pc, 1)?;

        let insn = instructions
            .iter()
            .next()
            .expect("wrong amount of instructions");

        let detail = self.cs.insn_detail(insn)?;

        let operands = detail.arch_detail().operands();
        let groups_ids: Vec<_> = detail.groups().into();

        let mut groups = Vec::new();
        for id in groups_ids {
            groups.push(Group::from_num(self.arch, id.0).ok_or(DisError::NoGroup(id.0))?);
        }

        let (read_ids, write_ids) = self.cs.regs_access(insn).unwrap().unwrap();
        let mut read = Vec::new();
        for id in read_ids {
            read.push(Reg::from_num(self.arch, id.0).ok_or(DisError::NoReg(id.0))?);
        }
        let mut write = Vec::new();
        for id in write_ids {
            write.push(Reg::from_num(self.arch, id.0).ok_or(DisError::NoReg(id.0))?);
        }

        Ok(Instruction {
            address: insn.address(),
            len: insn.len(),
            read,
            write,
            mnemonic: insn.mnemonic().map(str::to_string),
            op_str: insn.op_str().map(str::to_string),
            operands,
            groups,
        })
    }
}
