use rebg::analyzer::dump::TraceDumper;
use rebg::binary::Binary;
use rebg::host::docker::{Docker, DockerArgs};
use rebg::host::native::{Native, NativeArgs};
use rebg::serve;
use rebg::state::{Aarch64Step, Step, X64Step};
use rebg::tracer::qemu::QEMUParser;
use rebg::tracer::TracerCmd;
use rebg::{
    arch::Arch,
    host::Host,
    tracer::{qemu::QEMU, Tracer},
};
use std::fmt;
use std::path::Path;
use std::{fs, path::PathBuf};

#[derive(argh::FromArgs)]
/// tracer
struct Arguments {
    /// the program to trace
    #[argh(positional)]
    program: PathBuf,

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

    fn launch(
        &self,
        program: String,
        args: Vec<String>,
    ) -> Result<std::process::Child, Self::Error> {
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
}

fn main() {
    let Arguments {
        program,
        target_arch,
        launcher,
    } = argh::from_env();

    let bin = {
        let buffer = fs::read(&program).unwrap();
        Binary::from_bytes(buffer.into_boxed_slice()).unwrap()
    };
    let target_arch =
        target_arch.unwrap_or_else(|| Arch::from_elf(bin.elf().header.e_machine).unwrap());

    let qemu = QEMU {};

    let launcher = launcher.start_tracer(program.clone(), target_arch);

    match target_arch {
        Arch::ARM64 => {
            let parser =
                launch_qemu::<_, _, Aarch64Step, 32>(&launcher, qemu, target_arch, &program);
            let analysis =
                TraceDumper::analyze::<_, _, QEMU, _, 32>(&launcher, parser, target_arch);
            serve::ws(analysis, target_arch);
        }
        Arch::X86_64 => {
            let parser = launch_qemu::<_, _, X64Step, 16>(&launcher, qemu, target_arch, &program);
            let analysis =
                TraceDumper::analyze::<_, _, QEMU, _, 16>(&launcher, parser, target_arch);
            serve::ws(analysis, target_arch);
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
    STEP: for<'a> TryFrom<&'a [String], Error = anyhow::Error>,
{
    let cmd: TracerCmd<STEP, N> = tracer.command(program, arch);

    let child = launcher.launch(cmd.program, cmd.args).unwrap();

    tracer.parse(child)
}
