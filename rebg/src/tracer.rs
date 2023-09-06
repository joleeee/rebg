use crate::{arch::Arch, state::Step};
use std::{collections::HashMap, marker::PhantomData, path::Path, process::Child};

pub mod qemu;

#[derive(Debug)]
pub enum ParsedStep<STEP, const N: usize>
where
    STEP: Step<N>,
{
    LibLoad(HashMap<String, (u64, u64)>),
    TraceStep(STEP),
    // TODO could handle this ourselves? esp when we have iterator?
    Final(std::process::Output),
}

/// - Gives the specific tracer to be ran, with options
/// - Parses output
pub trait Tracer<STEP, const N: usize>
where
    STEP: Step<N>,
{
    type ITER: Iterator<Item = ParsedStep<STEP, N>>;
    fn command(&self, executable: &Path, arch: Arch) -> TracerCmd<STEP, N>;
    fn parse(&self, proc: Child) -> Self::ITER;
}

pub struct TracerCmd<STEP, const N: usize>
where
    STEP: Step<N>,
{
    pub program: String,
    pub args: Vec<String>,
    // for trait inferance
    _step: PhantomData<STEP>,
}
