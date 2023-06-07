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
use std::{fs, path::PathBuf};

#[derive(argh::FromArgs)]
/// tracer
struct Arguments {
    /// the program to trace
    #[argh(positional)]
    program: PathBuf,

    #[argh(option, short = 'a')]
    /// architecture: arm, x64, ...
    arch: Option<Arch>,

    #[argh(subcommand)]
    backend: Backends,
}

#[derive(argh::FromArgs)]
#[argh(subcommand)]
enum Backends {
    Docker(DockerArgs),
    Native(NativeArgs),
}

fn main() {
    let Arguments {
        program,
        arch,
        backend,
    } = argh::from_env();

    let buffer = fs::read(&program).unwrap();
    let elf = goblin::elf::Elf::parse(&buffer).unwrap();
    let arch = arch.unwrap_or_else(|| Arch::from_elf(elf.header.e_machine).unwrap());

    let qemu = QEMU {};

    match backend {
        Backends::Docker(docker) => {
            let docker = docker.start(program.clone(), arch);

            match arch {
                Arch::ARM64 => {
                    launch_docker::<_, Aarch64Step, 32>(docker, qemu, arch, &program);
                }
                Arch::X86_64 => {
                    launch_docker::<_, X64Step, 16>(docker, qemu, arch, &program);
                }
            }
        }
        Backends::Native(native) => {
            let native = native.start(program.clone(), arch);

            match arch {
                Arch::ARM64 => {
                    launch_native::<_, Aarch64Step, 32>(native, qemu, arch, &program);
                }
                Arch::X86_64 => {
                    launch_native::<_, X64Step, 16>(native, qemu, arch, &program);
                }
            }
        }
    }
}

fn launch_docker<BACKEND, STEP, const N: usize>(
    docker: Docker,
    backend: BACKEND,
    arch: Arch,
    program: &PathBuf,
) where
    BACKEND: Backend<STEP, N, ITER = QEMUParser<STEP, N>>,
    STEP: Step<N> + Send + 'static + fmt::Debug,
    STEP: for<'a> TryFrom<&'a [String], Error = anyhow::Error>,
{
    let cmd: BackendCmd<STEP, N> = backend.command(program, arch);

    let child = docker.launch(cmd.program, cmd.args).unwrap();
    let rx: QEMUParser<STEP, N> = backend.parse(child);

    TraceDumper::analyze::<_, _, BACKEND, _, N>(&docker, rx, &arch);
}

fn launch_native<BACKEND, STEP, const N: usize>(
    native: Native,
    backend: BACKEND,
    arch: Arch,
    program: &PathBuf,
) where
    BACKEND: Backend<STEP, N, ITER = QEMUParser<STEP, N>>,
    STEP: Step<N> + Send + 'static + fmt::Debug,
    STEP: for<'a> TryFrom<&'a [String], Error = anyhow::Error>,
{
    let cmd: BackendCmd<STEP, N> = backend.command(program, arch);

    let child = native.launch(cmd.program, cmd.args).unwrap();
    let rx: QEMUParser<STEP, N> = backend.parse(child);

    TraceDumper::analyze::<_, _, BACKEND, _, N>(&native, rx, &arch);
}
