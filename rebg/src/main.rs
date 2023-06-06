use rebg::analyzer::dump::TraceDumper;
use rebg::analyzer::Analyzer;
use rebg::backend::qemu::QEMUParser;
use rebg::backend::BackendCmd;
use rebg::launcher::docker::DockerArgs;
use rebg::state::{Aarch64Step, X64Step};
use rebg::{
    arch::Arch,
    backend::{qemu::QEMU, Backend},
    launcher::Launcher,
};
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

    #[allow(clippy::infallible_destructuring_match)]
    let docker = match backend {
        Backends::Docker(docker) => docker,
    };
    let docker = docker.start(program.clone(), arch);

    let qemu = QEMU {};

    match arch {
        Arch::ARM64 => {
            let cmd: BackendCmd<Aarch64Step, 32> = qemu.command(&program, arch);

            let child = docker.launch(cmd.program, cmd.args).unwrap();
            let rx: QEMUParser<Aarch64Step, 32> = qemu.parse(child);

            TraceDumper::analyze::<_, _, QEMU, _, 32>(&docker, rx, &arch);
        }
        Arch::X86_64 => {
            let cmd: BackendCmd<X64Step, 16> = qemu.command(&program, arch);

            let child = docker.launch(cmd.program, cmd.args).unwrap();
            let rx: QEMUParser<X64Step, 16> = qemu.parse(child);

            TraceDumper::analyze::<_, _, QEMU, _, 16>(&docker, rx, &arch);
        }
    }
}
