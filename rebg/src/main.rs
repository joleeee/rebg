use rebg::analyzer::dump::TraceDumper;
use rebg::analyzer::Analyzer;
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

    let docker = match backend {
        Backends::Docker(docker) => docker,
    };
    let docker = docker.start(program.clone(), arch);

    let qemu = QEMU {};

    match arch {
        Arch::ARM64 => {
            let cmd = Backend::<Aarch64Step, 32>::command(&qemu, &program, arch);

            let child = docker.launch(cmd.0, cmd.1).unwrap();
            let rx = qemu.parse(child);
            TraceDumper::analyze::<Aarch64Step, _, 32>(&docker, rx, &arch);
        }
        Arch::X86_64 => {
            let cmd = Backend::<X64Step, 16>::command(&qemu, &program, arch);

            let child = docker.launch(cmd.0, cmd.1).unwrap();
            let rx = qemu.parse(child);
            TraceDumper::analyze::<X64Step, _, 16>(&docker, rx, &arch);
        }
    }
}
