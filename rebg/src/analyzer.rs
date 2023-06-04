pub mod dump;
use crate::{arch::Arch, backend::ParsedStep, launcher::Launcher, state::Step};
use std::fmt;

pub trait Analyzer {
    fn analyze<STEP, LAUNCHER, const N: usize>(
        launcher: &LAUNCHER,
        rx: flume::Receiver<ParsedStep<STEP, N>>,
        arch: &Arch,
    ) where
        STEP: Step<N> + fmt::Debug,
        LAUNCHER: Launcher,
        <LAUNCHER as Launcher>::Error: fmt::Debug;
}
