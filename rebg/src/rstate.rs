//! Register state

use bitflags::Flags;

use crate::state::State;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(dead_code)]
pub enum RDiff<B> {
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

pub struct StateDiff<const N: usize> {
    pub regs: [RDiff<u64>; N],
    pub pc: RDiff<u64>,
    pub flags: Vec<(&'static str, RDiff<bool>)>,
}

impl<const N: usize> StateDiff<N> {
    pub fn print<S: State<N>>(&self) -> bool {
        let diff_regs = self
            .regs
            .iter()
            .enumerate()
            .filter_map(|(i, s)| {
                if let RDiff::Changed { from, to } = s {
                    Some((i, from, to))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        for (i, a, b) in &diff_regs {
            println!("{} <- {:x} (prev {:x})", S::reg_name_idx(*i), b, a);
        }
        if let RDiff::Changed { from: _, to } = self.pc {
            println!("pc <- {:x}", to);
        }

        // ignore pc

        // pretty print flags
        let flags = self
            .flags
            .iter()
            .filter_map(|(name, flag)| {
                if let RDiff::Changed { from, to } = flag {
                    debug_assert_ne!(from, to);
                    Some((*name, *from))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        for (name, previous) in &flags {
            if *previous {
                println!("flags -{}", name);
            } else {
                println!("flags +{}", name);
            }
        }

        !diff_regs.is_empty() || !flags.is_empty()
    }
}

pub fn diff<S: State<N>, const N: usize>(current: &S, future: &S) -> StateDiff<N> {
    let regs: [RDiff<u64>; N] = current
        .regs()
        .iter()
        .zip(future.regs().iter())
        .map(|(a, b)| RDiff::make(*a, *b))
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();

    let pc = RDiff::make(current.pc(), future.pc());

    let flags: Vec<_> = S::FLAGS::all()
        .iter_names()
        .map(|(name, flag)| {
            let cur = current.flags().contains(flag);
            let fut = future.flags().contains(flag);

            let diff = if cur != fut {
                RDiff::Changed { from: cur, to: fut }
            } else {
                RDiff::Unchanged { value: cur }
            };

            (name, diff)
        })
        .collect();

    StateDiff { regs, pc, flags }
}

#[cfg(test)]
mod tests {
    use super::RDiff;
    //use crate::CpuState;

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

    //#[test]
    //fn state_diff() {
    //    let original = crate::CpuState {
    //        regs: [0, 2, 3, 0],
    //        pc: 0x40000,
    //        flags: 0x10,
    //    };

    //    let modified = crate::CpuState {
    //        regs: [0, 3, 3, -1],
    //        pc: 0x40004,
    //        flags: 0x08,
    //    };

    //    let diff = original.diff(&modified);

    //    assert_eq!(
    //        diff,
    //        CpuState {
    //            regs: [
    //                RDiff::Unchanged { value: 0 },
    //                RDiff::Changed { from: 2, to: 3 },
    //                RDiff::Unchanged { value: 3 },
    //                RDiff::Changed { from: 0, to: -1 },
    //            ],
    //            pc: RDiff::Changed {
    //                from: 0x40000,
    //                to: 0x40004
    //            },
    //            flags: RDiff::Changed {
    //                from: 0x10,
    //                to: 0x8
    //            },
    //        }
    //    );
    //}
}
