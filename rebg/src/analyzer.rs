pub mod dump;
use crate::{
    arch::Arch,
    backend::{Backend, ParsedStep},
    host::Host,
    state::Step,
};
use std::fmt;

pub trait Analyzer {
    fn analyze<STEP, LAUNCHER, BACKEND, ITER, const N: usize>(
        // to read files
        launcher: &LAUNCHER,
        // inferred from BACKEND
        iter: ITER,
        arch: Arch,
    ) where
        STEP: Step<N> + fmt::Debug,
        LAUNCHER: Host,
        <LAUNCHER as Host>::Error: fmt::Debug,
        BACKEND: Backend<STEP, N, ITER = ITER>,
        ITER: Iterator<Item = ParsedStep<STEP, N>>;
}
