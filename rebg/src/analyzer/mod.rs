pub mod dump;
use crate::{
    dis::Instruction,
    state::{Instrumentation, Step},
    syms::SymbolTable,
};
use std::fmt;

#[derive(Clone, Debug)]
pub struct Analysis<STEP, const N: usize>
where
    STEP: Step<N> + fmt::Debug,
{
    pub trace: Vec<STEP>,
    pub insns: Vec<Instruction>,
    pub instrumentations: Vec<Instrumentation>,
    pub bt_lens: Vec<usize>,
    pub table: SymbolTable,
}
