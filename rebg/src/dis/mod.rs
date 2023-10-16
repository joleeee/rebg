//! This contains everything for dissassembling\
//! currently we just have support for capstone, but this code lets us hide some
//! of the internals of capstone

// we also want to abstract away some things, like "pseudoregisters" like `eax`
// and `ah` which are really part of `rax`. in some situations like when
// highlighting what registers changed, we dont care about `eax` or `ah`, we
// want `rax` (or some other canonical ref)

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

    pub fn group_name(&self, group: capstone::InsnGroupId) -> Option<&'static str> {
        match self.arch {
            Arch::ARM64 => Self::group_name_aarch64(group.0),
            Arch::X86_64 => Self::group_name_x86(group.0),
        }
    }

    const fn group_name_aarch64(id: u8) -> Option<&'static str> {
        Some(match id {
            1 => "jump",
            2 => "call",
            3 => "return",
            4 => "int",
            6 => "privilege",
            7 => "branch_relative",
            8 => "pointer authentication",
            128 => "crypto",
            129 => "fparmv8",
            130 => "neon",
            131 => "crc",
            132 => "aes",
            133 => "dotprod",
            134 => "fullfp16",
            135 => "lse",
            136 => "rcpc",
            137 => "rdm",
            138 => "sha2",
            139 => "sha3",
            140 => "sm4",
            141 => "sve",
            142 => "sve2",
            143 => "sve2-aes",
            144 => "sve2-bitperm",
            145 => "sve2-sha3",
            146 => "sve2-sm4",
            147 => "sme",
            148 => "sme-f64",
            149 => "sme-i64",
            150 => "f32mm",
            151 => "f64mm",
            152 => "i8mm",
            153 => "v8_1a",
            154 => "v8_3a",
            155 => "v8_4a",
            _ => return None,
        })
    }

    const fn group_name_x86(id: u8) -> Option<&'static str> {
        Some(match id {
            1 => "jump",
            2 => "call",
            3 => "ret",
            4 => "int",
            5 => "iret",
            6 => "privilege",
            7 => "branch_relative",
            128 => "vm",
            129 => "3dnow",
            130 => "aes",
            131 => "adx",
            132 => "avx",
            133 => "avx2",
            134 => "avx512",
            135 => "bmi",
            136 => "bmi2",
            137 => "cmov",
            138 => "fc16",
            139 => "fma",
            140 => "fma4",
            141 => "fsgsbase",
            142 => "hle",
            143 => "mmx",
            144 => "mode32",
            145 => "mode64",
            146 => "rtm",
            147 => "sha",
            148 => "sse1",
            149 => "sse2",
            150 => "sse3",
            151 => "sse41",
            152 => "sse42",
            153 => "sse4a",
            154 => "ssse3",
            155 => "pclmul",
            156 => "xop",
            157 => "cdi",
            158 => "eri",
            159 => "tbm",
            160 => "16bitmode",
            161 => "not64bitmode",
            162 => "sgx",
            163 => "dqi",
            164 => "bwi",
            165 => "pfi",
            166 => "vlx",
            167 => "smap",
            168 => "novlx",
            169 => "fpu",
            _ => return None,
        })
    }
}

struct Group {
    inner: capstone::InsnGroupId,
}

// common groups (used in an example binary)
// aarch64: 7, 3, 2, 4, 129, 130, 1, 6
// x86: 132, 3, 4, 133, 149, 2, 168, 7, 161, 135, 148, 137, 145, 1

#[cfg(test)]
mod tests {
    use capstone::InsnGroupId;
    use std::rc::Rc;

    use super::Dis;
    use crate::arch::Arch;

    #[test]
    fn aarch64_group_names() {
        let arch = Arch::ARM64;
        let cs = arch.make_capstone().unwrap();
        let cs = Rc::new(cs);
        let dis = Dis {
            arch,
            cs: cs.clone(),
        };

        for i in 0..=u8::MAX {
            let i = InsnGroupId(i);
            let cs_name = cs.group_name(i);
            let dis_name = dis.group_name(i);
            if cs_name != dis_name.map(str::to_string) {
                panic!("{cs_name:?} != {dis_name:?}")
            }

            // use this to basically print the match table :)
            // if let Some(name) = cs_name {
            //     println!("{} => {:?},", i.0, name);
            // }
        }
    }

    #[test]
    fn x86_group_names() {
        let arch = Arch::X86_64;
        let cs = arch.make_capstone().unwrap();
        let cs = Rc::new(cs);
        let dis = Dis {
            arch,
            cs: cs.clone(),
        };

        for i in 0..=u8::MAX {
            let i = InsnGroupId(i);
            let cs_name = cs.group_name(i);
            let dis_name = dis.group_name(i);
            if cs_name != dis_name.map(str::to_string) {
                panic!("{cs_name:?} != {dis_name:?}")
            }

            // use this to basically print the match table :)
            // if let Some(name) = cs_name {
            //     println!("{} => {:?},", i.0, name);
            // }
        }
    }
}
