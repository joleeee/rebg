pub mod dump;
use crate::{
    arch::Arch,
    host::Host,
    state::Step,
    tracer::{ParsedStep, Tracer},
};
use std::fmt;

pub trait Analyzer {
    fn analyze<STEP, LAUNCHER, TRACER, ITER, const N: usize>(
        // to read files
        launcher: &LAUNCHER,
        // inferred from TRACER
        iter: ITER,
        arch: Arch,
    ) where
        STEP: Step<N> + fmt::Debug,
        LAUNCHER: Host,
        <LAUNCHER as Host>::Error: fmt::Debug,
        TRACER: Tracer<STEP, N, ITER = ITER>,
        ITER: Iterator<Item = ParsedStep<STEP, N>>;
}
