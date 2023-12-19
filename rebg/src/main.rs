use rebg::analyzer::dump::TraceDumper;
use rebg::binary::Binary;
use rebg::host::docker::{Docker, DockerArgs};
use rebg::host::native::{Native, NativeArgs};
use rebg::serve;
use rebg::state::{Aarch64Step, Step, X64Step};
use rebg::tracer::parser::{GenericParser, Message};
use rebg::tracer::qiling::Qiling;
use rebg::tracer::TracerCmd;
use rebg::{
    arch::Arch,
    host::Host,
    tracer::{qemu::QEMU, Tracer},
};
use std::fmt;
use std::path::Path;
use std::{fs, path::PathBuf};
use strum::EnumString;
use tracing_subscriber::{fmt as tracing_fmt, EnvFilter};

#[derive(EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum TraceTypes {
    Qemu,
    Qiling,
}

#[derive(argh::FromArgs)]
/// tracer
struct Arguments {
    /// the program to trace
    #[argh(positional)]
    program: PathBuf,

    #[argh(switch, short = 'q', long = "quit")]
    /// quit instead of opening a ws server
    quit: bool,

    #[argh(switch, short = 'p', long = "print")]
    /// print trace
    print: bool,

    #[argh(option, short = 'a')]
    /// override detected architecture (arm64, amd64, ...)
    target_arch: Option<Arch>,

    #[argh(option, short = 't', long = "tracer")]
    /// the tracer to use
    tracer: TraceTypes,

    #[argh(subcommand)]
    launcher: LauncherArgs,
}

#[derive(argh::FromArgs)]
#[argh(subcommand)]
enum LauncherArgs {
    Docker(DockerArgs),
    Native(NativeArgs),
}

impl LauncherArgs {
    fn start_tracer(self, program: PathBuf, arch: Arch) -> Launchers {
        match self {
            LauncherArgs::Docker(x) => Launchers::Docker(x.start(program, arch)),
            LauncherArgs::Native(x) => Launchers::Native(x.start()),
        }
    }
}

enum Launchers {
    Docker(Docker),
    Native(Native),
}

impl Host for Launchers {
    type Error = anyhow::Error;

    fn launch(&self, program: &str, args: Vec<String>) -> Result<std::process::Child, Self::Error> {
        match self {
            Launchers::Docker(d) => d.launch(program, args),
            Launchers::Native(n) => n.launch(program, args),
        }
    }

    fn read_file(&self, path: &std::path::Path) -> Result<Vec<u8>, Self::Error> {
        match self {
            Launchers::Docker(x) => x.read_file(path),
            Launchers::Native(x) => x.read_file(path),
        }
    }

    fn localhost(&self) -> &'static str {
        match self {
            Launchers::Docker(x) => x.localhost(),
            Launchers::Native(x) => x.localhost(),
        }
    }
}

fn main() {
    tracing_fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let Arguments {
        program,
        quit,
        target_arch,
        launcher,
        tracer,
        print,
    } = argh::from_env();

    let bin = {
        let buffer = fs::read(&program).unwrap();
        Binary::from_bytes(buffer.into_boxed_slice()).unwrap()
    };
    let target_arch =
        target_arch.unwrap_or_else(|| Arch::from_elf(bin.elf().header.e_machine).unwrap());

    let launcher = launcher.start_tracer(program.clone(), target_arch);

    let dumper = TraceDumper { print };

    match target_arch {
        Arch::ARM64 => match tracer {
            TraceTypes::Qemu => {
                let qemu = QEMU {};
                analyze_arch::<Aarch64Step, QEMU, 32>(
                    &dumper,
                    quit,
                    &launcher,
                    qemu,
                    target_arch,
                    &program,
                );
            }
            TraceTypes::Qiling => {
                let qiling = Qiling {};
                analyze_arch::<Aarch64Step, Qiling, 32>(
                    &dumper,
                    quit,
                    &launcher,
                    qiling,
                    target_arch,
                    &program,
                );
            }
        },
        Arch::X86_64 => match tracer {
            TraceTypes::Qemu => {
                let qemu = QEMU {};
                analyze_arch::<X64Step, QEMU, 16>(
                    &dumper,
                    quit,
                    &launcher,
                    qemu,
                    target_arch,
                    &program,
                );
            }
            TraceTypes::Qiling => {
                let qiling = Qiling {};
                analyze_arch::<X64Step, Qiling, 16>(
                    &dumper,
                    quit,
                    &launcher,
                    qiling,
                    target_arch,
                    &program,
                );
            }
        },
    }
}

fn analyze_arch<STEP, TRACER, const N: usize>(
    dumper: &TraceDumper,
    quit: bool,
    launcher: &Launchers,
    tracer: TRACER,
    target_arch: Arch,
    program: &Path,
) where
    STEP: Step<N> + Send + 'static + fmt::Debug + std::marker::Send + std::marker::Sync,
    STEP: for<'a> TryFrom<&'a [Message], Error = anyhow::Error>,
    TRACER: Tracer<STEP, N, ITER = GenericParser<STEP, N>>,
{
    let parser = launch_qemu::<_, _, STEP, N>(launcher, tracer, target_arch, program);
    let analysis = dumper.analyze::<_, _, QEMU, _, N>(launcher, parser, target_arch);
    if !quit {
        serve::ws(analysis, target_arch);
    }
}

fn launch_qemu<LAUNCHER, TRACER, STEP, const N: usize>(
    launcher: &LAUNCHER,
    tracer: TRACER,
    arch: Arch,
    program: &Path,
) -> GenericParser<STEP, N>
where
    LAUNCHER: Host<Error = anyhow::Error>,
    TRACER: Tracer<STEP, N, ITER = GenericParser<STEP, N>>,
    STEP: Step<N> + Send + 'static + fmt::Debug,
    STEP: for<'a> TryFrom<&'a [Message], Error = anyhow::Error>,
{
    let cmd: TracerCmd<STEP, N> = tracer.command(program, arch, launcher.localhost());

    let child = launcher
        .launch(&cmd.program, cmd.args)
        .unwrap_or_else(|err| panic!("Failed launching '{}': {:?}", cmd.program, err));

    tracer.parse(child)
}
