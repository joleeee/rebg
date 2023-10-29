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

#[derive(Clone, Debug)]
pub struct Instruction {
    pub len: usize,
    pub address: u64,

    // registers used
    pub read: Box<[Reg]>,
    pub write: Box<[Reg]>,

    // string stuff
    pub mnemonic: Option<String>,
    pub op_str: Option<String>,

    // blah
    pub operands: Box<[capstone::arch::ArchOperand]>,
    pub groups: Box<[Group]>,
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

        let groups: Box<[Group]> = detail
            .groups()
            .iter()
            .map(|id| Group::from_num(self.arch, id.0).ok_or(DisError::NoGroup(id.0)))
            .collect::<Result<_, _>>()?;

        let (read_ids, write_ids) = self.cs.regs_access(insn).unwrap().unwrap();

        let read: Box<[Reg]> = read_ids
            .iter()
            .map(|id| Reg::from_num(self.arch, id.0).ok_or(DisError::NoReg(id.0)))
            .collect::<Result<_, _>>()?;

        let write: Box<[Reg]> = write_ids
            .iter()
            .map(|id| Reg::from_num(self.arch, id.0).ok_or(DisError::NoReg(id.0)))
            .collect::<Result<_, _>>()?;

        Ok(Instruction {
            address: insn.address(),
            len: insn.len(),
            read,
            write,
            mnemonic: insn.mnemonic().map(str::to_string),
            op_str: insn.op_str().map(str::to_string),
            operands: operands.into_boxed_slice(),
            groups,
        })
    }
}
