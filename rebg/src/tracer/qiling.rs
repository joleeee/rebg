use super::{
    parser::{GenericParser, Message},
    Tracer, TracerCmd,
};
use crate::{arch::Arch, state::Step};
use std::{fmt, marker::PhantomData, path::Path};

pub struct Qiling {}

impl<STEP, const N: usize> Tracer<STEP, N> for Qiling
where
    STEP: Step<N> + Send + 'static + fmt::Debug,
    STEP: for<'a> TryFrom<&'a [Message], Error = anyhow::Error>,
{
    type ITER = GenericParser<STEP, N>;

    fn command(&self, executable: &Path, arch: Arch, _localhost: &str) -> TracerCmd<STEP, N> {
        let python = "python3".to_string();

        let options = vec![
            String::from("../tools/ql/run.py"),
            format!("../tools/ql/{}", arch.qiling_rootfs()),
            executable.to_str().unwrap().to_string(),
        ];

        TracerCmd {
            program: python,
            args: options,
            _step: PhantomData,
        }
    }

    /// Takes output from the process and parses it to steps
    fn parse(&self, proc: std::process::Child) -> Self::ITER {
        GenericParser::new(proc)
    }
}
