pub mod dump;
use crate::{
    arch::Arch,
    dis::Instruction,
    host::Host,
    state::{Instrumentation, Step},
    syms::SymbolTable,
    tracer::{ParsedStep, Tracer},
};
use std::{cell::RefCell, fmt};

#[derive(Clone, Debug)]
pub struct Analysis<STEP, const N: usize>
where
    STEP: Step<N> + fmt::Debug,
{
    pub trace: Vec<STEP>,
    pub insns: Vec<Instruction>,
    pub instrumentations: Vec<Instrumentation>,
    pub bt_lens: Vec<usize>,
    pub table: RefCell<SymbolTable>,
}

pub trait Analyzer {
    fn analyze<STEP, LAUNCHER, TRACER, ITER, const N: usize>(
        // to read files
        launcher: &LAUNCHER,
        // inferred from TRACER
        iter: ITER,
        arch: Arch,
    ) -> Analysis<STEP, N>
    where
        STEP: Step<N> + fmt::Debug,
        LAUNCHER: Host,
        <LAUNCHER as Host>::Error: fmt::Debug,
        TRACER: Tracer<STEP, N, ITER = ITER>,
        ITER: Iterator<Item = ParsedStep<STEP, N>>;
}
