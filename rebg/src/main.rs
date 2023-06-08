use rebg::analyzer::dump::TraceDumper;
use rebg::analyzer::Analyzer;
use rebg::backend::qemu::QEMUParser;
use rebg::backend::BackendCmd;
use rebg::launcher::docker::{Docker, DockerArgs};
use rebg::launcher::native::{Native, NativeArgs};
use rebg::state::{Aarch64Step, Step, X64Step};
use rebg::{
    arch::Arch,
    backend::{qemu::QEMU, Backend},
    launcher::Launcher,
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
    /// override architecure: arm, x64, ...
    arch: Option<Arch>,

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
    fn start_backend(self, program: PathBuf, arch: Arch) -> Launchers {
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

impl Launcher for Launchers {
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
        arch,
        launcher,
    } = argh::from_env();

    let buffer = fs::read(&program).unwrap();
    let elf = goblin::elf::Elf::parse(&buffer).unwrap();
    let arch = arch.unwrap_or_else(|| Arch::from_elf(elf.header.e_machine).unwrap());

    let qemu = QEMU {};

    let launcher = launcher.start_backend(program.clone(), arch);

    match arch {
        Arch::ARM64 => {
            let parser = launch_qemu::<_, _, Aarch64Step, 32>(&launcher, qemu, arch, &program);
            TraceDumper::analyze::<_, _, QEMU, _, 32>(&launcher, parser, &arch);
        }
        Arch::X86_64 => {
            let parser = launch_qemu::<_, _, X64Step, 16>(&launcher, qemu, arch, &program);
            TraceDumper::analyze::<_, _, QEMU, _, 16>(&launcher, parser, &arch);
        }
    }
}

fn launch_qemu<LAUNCHER, BACKEND, STEP, const N: usize>(
    launcher: &LAUNCHER,
    backend: BACKEND,
    arch: Arch,
    program: &Path,
) -> QEMUParser<STEP, N>
where
    LAUNCHER: Launcher<Error = anyhow::Error>,
    BACKEND: Backend<STEP, N, ITER = QEMUParser<STEP, N>>,
    STEP: Step<N> + Send + 'static + fmt::Debug,
    STEP: for<'a> TryFrom<&'a [String], Error = anyhow::Error>,
{
    let cmd: BackendCmd<STEP, N> = backend.command(program, arch);

    let child = launcher.launch(cmd.program, cmd.args).unwrap();

    backend.parse(child)
}
