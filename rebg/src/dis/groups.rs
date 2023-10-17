//! The point of this is to wrap capstone's groups so we can easily check if a
//! an instruction is in a group, in a cross-architecture way. We also provide a
//! non-cross-platform name by effectively caching capstone's cs_group_name.
//! This is unit tested to make sure there are not any regressions.

#![allow(non_camel_case_types, dead_code)]

use crate::arch::Arch;

macro_rules! enum_from_pairs {
    ($name:ident, $(($num:expr, $s:ident, $str:expr)),*) => {
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub enum $name {
            $( $s = $num, )*
        }

        impl $name {
            fn from_num(num: u8) -> Option<Self> {
                match num {
                    $( $num => Some($name::$s), )*
                    _ => None
                }
            }

            fn as_str(&self) -> &'static str {
                match self {
                    $( $name::$s => $str, )*
                }
            }
        }
    };
}

enum_from_pairs!(
    Aarch64Group,
    (1, Jump, "jump"),
    (2, Call, "call"),
    (3, Return, "return"),
    (4, Int, "int"),
    (6, Privilege, "privilege"),
    (7, BranchRelative, "branch_relative"),
    (8, PointerAuthentication, "pointer authentication"),
    (128, Crypto, "crypto"),
    (129, Fparmv8, "fparmv8"),
    (130, Neon, "neon"),
    (131, Crc, "crc"),
    (132, Aes, "aes"),
    (133, Dotprod, "dotprod"),
    (134, Fullfp16, "fullfp16"),
    (135, Lse, "lse"),
    (136, Rcpc, "rcpc"),
    (137, Rdm, "rdm"),
    (138, Sha2, "sha2"),
    (139, Sha3, "sha3"),
    (140, Sm4, "sm4"),
    (141, Sve, "sve"),
    (142, Sve2, "sve2"),
    (143, Sve2Aes, "sve2-aes"),
    (144, Sve2Bitperm, "sve2-bitperm"),
    (145, Sve2Sha3, "sve2-sha3"),
    (146, Sve2Sm4, "sve2-sm4"),
    (147, Sme, "sme"),
    (148, SmeF64, "sme-f64"),
    (149, SmeI64, "sme-i64"),
    (150, F32Mm, "f32mm"),
    (151, F64Mm, "f64mm"),
    (152, I8Mm, "i8mm"),
    (153, V81A, "v8_1a"),
    (154, V83A, "v8_3a"),
    (155, V84A, "v8_4a")
);

enum_from_pairs!(
    X64Group,
    (1, Jump, "jump"),
    (2, Call, "call"),
    (3, Ret, "ret"),
    (4, Int, "int"),
    (5, Iret, "iret"),
    (6, Privilege, "privilege"),
    (7, BranchRelative, "branch_relative"),
    (128, Vm, "vm"),
    (129, Group3DNow, "3dnow"), // had to fix this
    (130, Aes, "aes"),
    (131, Adx, "adx"),
    (132, Avx, "avx"),
    (133, Avx2, "avx2"),
    (134, Avx512, "avx512"),
    (135, Bmi, "bmi"),
    (136, Bmi2, "bmi2"),
    (137, Cmov, "cmov"),
    (138, Fc16, "fc16"),
    (139, Fma, "fma"),
    (140, Fma4, "fma4"),
    (141, Fsgsbase, "fsgsbase"),
    (142, Hle, "hle"),
    (143, Mmx, "mmx"),
    (144, Mode32, "mode32"),
    (145, Mode64, "mode64"),
    (146, Rtm, "rtm"),
    (147, Sha, "sha"),
    (148, Sse1, "sse1"),
    (149, Sse2, "sse2"),
    (150, Sse3, "sse3"),
    (151, Sse41, "sse41"),
    (152, Sse42, "sse42"),
    (153, Sse4A, "sse4a"),
    (154, Ssse3, "ssse3"),
    (155, Pclmul, "pclmul"),
    (156, Xop, "xop"),
    (157, Cdi, "cdi"),
    (158, Eri, "eri"),
    (159, Tbm, "tbm"),
    (160, Group16Bitmode, "16bitmode"), // and this
    (161, Not64Bitmode, "not64bitmode"),
    (162, Sgx, "sgx"),
    (163, Dqi, "dqi"),
    (164, Bwi, "bwi"),
    (165, Pfi, "pfi"),
    (166, Vlx, "vlx"),
    (167, Smap, "smap"),
    (168, Novlx, "novlx"),
    (169, Fpu, "fpu")
);

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Group {
    Aarch64Group(Aarch64Group),
    X64Group(X64Group),
}

impl Group {
    pub fn from_num(arch: Arch, num: u8) -> Option<Self> {
        Some(match arch {
            Arch::ARM64 => Group::Aarch64Group(Aarch64Group::from_num(num)?),
            Arch::X86_64 => Group::X64Group(X64Group::from_num(num)?),
        })
    }

    pub fn is_call(&self) -> bool {
        matches!(
            self,
            Group::Aarch64Group(Aarch64Group::Call) | Group::X64Group(X64Group::Call)
        )
    }

    pub fn is_ret(&self) -> bool {
        matches!(
            self,
            Group::Aarch64Group(Aarch64Group::Return) | Group::X64Group(X64Group::Ret)
        )
    }
}

// common groups (used in an example binary)
// aarch64: 7, 3, 2, 4, 129, 130, 1, 6
// x86: 132, 3, 4, 133, 149, 2, 168, 7, 161, 135, 148, 137, 145, 1

#[cfg(test)]
mod tests {
    use super::{Aarch64Group, X64Group};
    use crate::arch::Arch;
    use capstone::InsnGroupId;
    use std::rc::Rc;

    #[test]
    fn aarch64_group_names() {
        let arch = Arch::ARM64;
        let cs = arch.make_capstone().unwrap();
        let cs = Rc::new(cs);

        for i in 0..=u8::MAX {
            let i = InsnGroupId(i);
            let cs_name = cs.group_name(i);
            let dis_name = Aarch64Group::from_num(i.0)
                .map(|x| Aarch64Group::as_str(&x))
                .map(str::to_string);
            if cs_name != dis_name {
                panic!("{cs_name:?} != {dis_name:?}")
            }

            // cargo test dis::tests::aarch64_group_names -- --nocapture
            // if let Some(name) = cs_name {
            //     println!("({}, {}, \"{}\"),", i.0, name.to_case(convert_case::Case::Pascal), name);
            // }
        }
    }

    #[test]
    fn x64_group_names() {
        let arch = Arch::X86_64;
        let cs = arch.make_capstone().unwrap();
        let cs = Rc::new(cs);

        for i in 0..=u8::MAX {
            let i = InsnGroupId(i);
            let cs_name = cs.group_name(i);
            let dis_name = X64Group::from_num(i.0)
                .map(|x| X64Group::as_str(&x))
                .map(str::to_string);
            if cs_name != dis_name {
                panic!("{cs_name:?} != {dis_name:?}")
            }

            // if let Some(name) = cs_name {
            //     println!("({}, {}, \"{}\"),", i.0, name.to_case(convert_case::Case::Pascal), name);
            // }
        }
    }
}
