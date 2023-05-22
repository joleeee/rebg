//! Register state

use crate::CpuState;
use core::fmt;
use num_traits::Num;

pub trait Register: Num + Copy + PartialEq + fmt::Debug {}
impl<T: Num + Copy + PartialEq + fmt::Debug> Register for T {}

#[derive(Clone, Copy, PartialEq, Debug)]
#[allow(dead_code)]
enum RDiff<B> {
    Changed { from: B, to: B },
    Unchanged { value: B },
}

#[allow(dead_code)]
impl<X: PartialEq> RDiff<X> {
    pub fn make(before: X, after: X) -> Self {
        if before != after {
            Self::Changed {
                from: before,
                to: after,
            }
        } else {
            Self::Unchanged { value: before }
        }
    }
}

#[allow(dead_code)]
impl<B: Register, const N: usize> CpuState<B, N> {
    fn diff(&self, other: &Self) -> CpuState<RDiff<B>, N> {
        let regs: [RDiff<B>; N] = self
            .regs
            .into_iter()
            .zip(other.regs.into_iter())
            .map(|(a, b)| RDiff::make(a, b))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        let pc = RDiff::make(self.pc, other.pc);
        let flags = RDiff::make(self.flags, other.flags);

        CpuState { regs, pc, flags }
    }
}

#[cfg(test)]
mod tests {
    use super::RDiff;
    use crate::CpuState;

    #[test]
    fn single_diff() {
        let i0 = 0;
        let i1 = 5;
        let i2 = 5;

        let d1 = RDiff::make(i0, i1);
        let d2 = RDiff::make(i1, i2);

        assert_eq!(d1, RDiff::Changed { from: 0, to: 5 });
        assert_eq!(d2, RDiff::Unchanged { value: 5 });
    }

    #[test]
    fn state_diff() {
        let original = crate::CpuState {
            regs: [0, 2, 3, 0],
            pc: 0x40000,
            flags: 0x10,
        };

        let modified = crate::CpuState {
            regs: [0, 3, 3, -1],
            pc: 0x40004,
            flags: 0x08,
        };

        let diff = original.diff(&modified);

        assert_eq!(
            diff,
            CpuState {
                regs: [
                    RDiff::Unchanged { value: 0 },
                    RDiff::Changed { from: 2, to: 3 },
                    RDiff::Unchanged { value: 3 },
                    RDiff::Changed { from: 0, to: -1 },
                ],
                pc: RDiff::Changed {
                    from: 0x40000,
                    to: 0x40004
                },
                flags: RDiff::Changed {
                    from: 0x10,
                    to: 0x8
                },
            }
        );
    }
}
