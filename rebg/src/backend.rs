use crate::{arch::Arch, state::Step};
use std::{collections::HashMap, path::Path, process::Child};

pub mod qemu;

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
pub trait Backend<STEP, const N: usize>
where
    STEP: Step<N>,
{
    fn command(&self, executable: &Path, arch: Arch) -> (String, Vec<String>);
    fn parse(&self, proc: Child) -> flume::Receiver<ParsedStep<STEP, N>>;
}
