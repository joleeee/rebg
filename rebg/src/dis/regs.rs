#![allow(non_camel_case_types, dead_code)]

macro_rules! enum_from_pairs {
    ($name:ident, $(($num:expr, $s:ident, $str:expr, $parent:ident)),*) => {
        #[derive(PartialEq, Debug)]
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

            fn canonical(&self) -> Self {
                match self {
                    $( $name::$s => $name::$parent, )*
                }
            }
        }
    };
}

enum_from_pairs!(
    Aarch64Reg,
    (0, X0, "x0", X0),
    (1, X5, "x5", X5),
    (2, W0, "w0", X0),
    (3, W5, "w5", X5),
    (4, SP, "sp", SP),
    (5, XZR, "xzr", XZR),
    (6, W30, "w30", X30),
    (7, X30, "x30", X30)
);


pub enum Reg {
    Aarch64Reg(Aarch64Reg),
    // X64Reg(X64Reg),
}

// impl Reg {
//     pub fn from_num(arch: Arch, num: u8) -> Option<Self> {
//         Some(match arch {
//             Arch::ARM64 => Reg::Aarch64Reg(Aarch64Reg::from_num(num)?),
//             Arch::X86_64 => Reg::X64Reg(X64Reg::from_num(num)?),
//         })
//     }
// }

#[cfg(test)]
mod tests {
    use super::Aarch64Reg;
    use crate::{arch::Arch, dis::regs::Reg};

    #[test]
    fn aarch64_canon() {
        assert_eq!(Aarch64Reg::W0.canonical(), Aarch64Reg::X0);
        assert_eq!(Aarch64Reg::X5.canonical(), Aarch64Reg::X5);
        assert_eq!(Aarch64Reg::SP.canonical(), Aarch64Reg::SP);
        assert_eq!(Aarch64Reg::XZR.canonical(), Aarch64Reg::XZR);
        assert_eq!(Aarch64Reg::X30.canonical(), Aarch64Reg::X30);
        assert_eq!(Aarch64Reg::W30.canonical(), Aarch64Reg::X30);
    }
}
