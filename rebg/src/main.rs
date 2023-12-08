use object::Object;
use rebg::analyzer::dump::TraceDumper;
use rebg::binary::Binary;
use rebg::host::docker::{Docker, DockerArgs};
use rebg::host::native::{Native, NativeArgs};
use rebg::serve;
use rebg::state::{Aarch64Step, Step, X64Step};
use rebg::tracer::qemu::{Message, QEMUParser};
use rebg::tracer::TracerCmd;
use rebg::{
    arch::Arch,
    host::Host,
    tracer::{qemu::QEMU, Tracer},
};
use std::fmt;
use std::path::Path;
use std::{fs, path::PathBuf};
use tracing_subscriber::{fmt as tracing_fmt, EnvFilter};

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
        print,
    } = argh::from_env();

    let bin = {
        let buffer = fs::read(&program).unwrap();
        Binary::from_bytes(buffer.into_boxed_slice()).unwrap()
    };
    let target_arch =
        target_arch.unwrap_or_else(|| Arch::from_object(bin.obj().architecture()).unwrap());

    let qemu = QEMU {};

    let launcher = launcher.start_tracer(program.clone(), target_arch);

    let dumper = TraceDumper { print };

    match target_arch {
        Arch::ARM64 => {
            let parser =
                launch_qemu::<_, _, Aarch64Step, 32>(&launcher, qemu, target_arch, &program);
            let analysis = dumper.analyze::<_, _, QEMU, _, 32>(&launcher, parser, target_arch);
            if !quit {
                serve::ws(analysis, target_arch);
            }
        }
        Arch::X86_64 => {
            let parser = launch_qemu::<_, _, X64Step, 16>(&launcher, qemu, target_arch, &program);
            let analysis = dumper.analyze::<_, _, QEMU, _, 16>(&launcher, parser, target_arch);
            if !quit {
                serve::ws(analysis, target_arch);
            }
        }
    }
}

fn launch_qemu<LAUNCHER, TRACER, STEP, const N: usize>(
    launcher: &LAUNCHER,
    tracer: TRACER,
    arch: Arch,
    program: &Path,
) -> QEMUParser<STEP, N>
where
    LAUNCHER: Host<Error = anyhow::Error>,
    TRACER: Tracer<STEP, N, ITER = QEMUParser<STEP, N>>,
    STEP: Step<N> + Send + 'static + fmt::Debug,
    STEP: for<'a> TryFrom<&'a [Message], Error = anyhow::Error>,
{
    let cmd: TracerCmd<STEP, N> = tracer.command(program, arch, launcher.localhost());

    let child = launcher
        .launch(&cmd.program, cmd.args)
        .expect(&format!("Failed launching '{}'", cmd.program));

    tracer.parse(child)
}
