use super::{
    parser::{GenericParser, Message},
    Tracer, TracerCmd,
};
use crate::{arch::Arch, state::Step};
use std::{fmt, marker::PhantomData, path::Path};

pub struct QEMU {}

impl<STEP, const N: usize> Tracer<STEP, N> for QEMU
where
    STEP: Step<N> + Send + 'static + fmt::Debug,
    STEP: for<'a> TryFrom<&'a [Message], Error = anyhow::Error>,
{
    type ITER = GenericParser<STEP, N>;

    fn command(&self, executable: &Path, arch: Arch, localhost: &str) -> TracerCmd<STEP, N> {
        let qemu = arch.qemu_user_bin().to_string();

        let options = vec![
            String::from("-rebglog"),
            String::from("/dev/null"),
            String::from("-rebgtcp"),
            format!("{localhost}:1337"),
            String::from("-one-insn-per-tb"),
            String::from("-d"),
            String::from("in_asm,strace"),
            executable.to_str().unwrap().to_string(),
        ];

        TracerCmd {
            program: qemu,
            args: options,
            _step: PhantomData,
        }
    }

    /// Takes output from the process and parses it to steps
    fn parse(&self, proc: std::process::Child) -> Self::ITER {
        GenericParser::new(proc)
    }
}
