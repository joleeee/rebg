use crate::{arch::Arch, state::Step};
use std::{collections::HashMap, fmt, io::Read, marker::PhantomData, path::Path, process::Child};

use self::parser::Message;

pub mod parser;
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
    fn command(&self, executable: &Path, arch: Arch, localhost: &str) -> TracerCmd<STEP, N>;
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

pub fn get_next_step<R: Read, STEP, const N: usize>(
    reader: &mut R,
    proc: &mut Option<Child>,
) -> Option<ParsedStep<STEP, N>>
where
    STEP: Step<N> + Send + 'static + fmt::Debug,
    STEP: for<'a> TryFrom<&'a [Message], Error = anyhow::Error>,
{
    #[allow(clippy::question_mark)]
    if proc.is_none() {
        return None;
    }

    let mut msgs = vec![];

    while let Some(m) = parser::get_next_message(reader) {
        if matches!(m, Message::Separator) {
            break;
        }

        msgs.push(m);
    }

    // if there are no msgs, we're done!
    if msgs.is_empty() {
        let mut my_proc = None;
        std::mem::swap(proc, &mut my_proc);
        let my_proc = my_proc.unwrap();

        // make sure it closed gracefully
        let result = my_proc.wait_with_output().unwrap();

        return Some(ParsedStep::Final(result));
    }

    if matches!(msgs[0], Message::LibLoad(_, _, _)) {
        let map = msgs
            .into_iter()
            .map(|m| match m {
                Message::LibLoad(name, from, to) => (name.to_string(), (from, to)),
                _ => panic!("Got libload and some other junk!"),
            })
            .collect();

        return Some(ParsedStep::LibLoad(map));
    }

    // otherwise, it's just a step :)

    let s = STEP::try_from(&msgs).unwrap();
    Some(ParsedStep::TraceStep(s))
}
