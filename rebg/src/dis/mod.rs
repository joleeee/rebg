//! This contains everything for dissassembling\
//! currently we just have support for capstone, but this code lets us hide some
//! of the internals of capstone

// we also want to abstract away some things, like "pseudoregisters" like `eax`
// and `ah` which are really part of `rax`. in some situations like when
// highlighting what registers changed, we dont care about `eax` or `ah`, we
// want `rax` (or some other canonical ref)

pub mod groups;

use crate::arch::Arch;
use std::rc::Rc;
use thiserror::Error;

pub struct Dis {
    pub arch: Arch,
    pub cs: Rc<capstone::Capstone>,
}

pub struct Instruction {
    pub len: usize,
    pub address: u64,

    // registers used
    pub read: Vec<capstone::RegId>,
    pub write: Vec<capstone::RegId>,

    // string stuff
    pub mnemonic: Option<String>,
    pub op_str: Option<String>,

    // blah
    pub operands: Vec<capstone::arch::ArchOperand>,
    pub groups: Vec<capstone::InsnGroupId>,
    pub group_names: Vec<String>,
}

#[derive(Error, Debug)]
pub enum DisError {
    #[error("capstone")]
    Capstone(#[from] capstone::Error),
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
        let groups: Vec<_> = detail.groups().into();
        let group_names = groups
            .iter()
            .map(|g| self.cs.group_name(*g).unwrap())
            .collect();

        let (read, write) = self.cs.regs_access(insn).unwrap().unwrap();

        Ok(Instruction {
            address: insn.address(),
            len: insn.len(),
            read,
            write,
            mnemonic: insn.mnemonic().map(str::to_string),
            op_str: insn.op_str().map(str::to_string),
            operands,
            groups,
            group_names,
        })
    }
}
