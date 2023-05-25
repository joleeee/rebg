use anyhow::Context;
use capstone::{prelude::BuildsCapstone, Capstone};
use state::{Aarch64Step, State, Step, X64Step};
use std::{
    collections::HashMap,
    fs,
    io::{BufRead, BufReader, Read},
    path::PathBuf,
    process::{exit, Child, ChildStderr, ChildStdout, Command, Stdio},
    str::FromStr,
};
use syms::SymbolTable;

mod rstate;
mod state;
mod syms;

struct RunningQemu {
    #[allow(dead_code)]
    stdout: BufReader<ChildStdout>,
    stderr: BufReader<ChildStderr>,
    child: Child,

    stderr_bfr: String,
}

impl RunningQemu {
    fn check_crash(&mut self) {
        if let Some(result) = self.child.try_wait().unwrap() {
            self.stderr.read_to_string(&mut self.stderr_bfr).unwrap();

            if !result.success() {
                println!(
                    "QEMU Failed with code {} and err \"{}\"",
                    result.code().unwrap(),
                    self.stderr_bfr.trim(),
                );
                exit(1);
            }
        }
    }
}

impl Iterator for RunningQemu {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        // read until found a chunk

        loop {
            let split = self
                .stderr_bfr
                .split_once("----------------")
                .map(|(a, b)| (a.to_string(), b.to_string()));

            if let Some((before, after)) = split {
                self.stderr_bfr = after.to_string();
                return Some(before.to_string());
            }

            let result = self.stderr.read_line(&mut self.stderr_bfr).unwrap();
            match result {
                0 => {
                    self.check_crash(); // make sure the reason we're done is not because of a crash
                    return None; // EOF
                }
                _ => {}
            }
        }
    }
}

fn run_qemu(id: &str, program: &str, arch: &Arch) -> anyhow::Result<RunningQemu> {
    // copy program into container
    let cp = Command::new("cp").arg(program).arg("container/").spawn()?;
    cp.wait_with_output()?;

    // run qemu inside the container
    let guest_path = format!(
        "/container/{}",
        PathBuf::from(program)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
    );

    let mut child = Command::new("docker")
        .arg("exec")
        .arg(id)
        .arg(arch.qemu_user_bin())
        .arg("-one-insn-per-tb")
        .args(["-d", "in_asm"])
        .arg(guest_path)
        .stdin(Stdio::null()) // todo pass through from nc
        .stdout(Stdio::piped()) // same here
        .stderr(Stdio::piped()) // also use different file descriptors for qemu output so they dont collide
        .spawn()?;

    println!("Starting qemu");
    // spawn new thread

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let stdout = BufReader::new(stdout);
    let stderr = BufReader::new(stderr);

    Ok(RunningQemu {
        stdout,
        stderr,
        stderr_bfr: String::new(),
        child,
    })
}

fn parse_elflibload(output: &str) -> anyhow::Result<HashMap<String, (u64, u64)>> {
    let chunk = output
        .split('\n')
        .into_iter()
        .map(|x| x.trim())
        .filter(|x| !matches!(*x, "" | "IN:"));

    let parts: Vec<_> = chunk
        .map(|x| x.split_once('|'))
        .collect::<Option<Vec<_>>>()
        .context("invalid header, should only be | separated key|values")?;

    let mut elfs = HashMap::new();
    for (key, value) in parts {
        match key {
            "elflibload" => {
                let (path, other) = value.split_once('|').unwrap();
                let (from, to) = other.split_once('|').unwrap();

                let from = u64::from_str_radix(from, 16).unwrap();
                let to = u64::from_str_radix(to, 16).unwrap();

                elfs.insert(path.to_string(), (from, to));
            }
            _ => {
                return Err(anyhow::anyhow!("unknown header key: {}", key));
            }
        }
    }

    Ok(elfs)
}

fn spawn_runner(image_name: &str, arch: &Arch) -> String {
    // stop any previous container
    let mut stop = Command::new("docker")
        .arg("kill")
        .arg("rebg-runner")
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .spawn()
        .unwrap();
    stop.wait().unwrap();

    // delete previous instance
    let mut rm = Command::new("docker")
        .arg("rm")
        .arg("rebg-runner")
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .spawn()
        .unwrap();
    rm.wait().unwrap();

    // spawn container (background)
    let run = Command::new("docker")
        .arg("run")
        .arg("-d")
        .args(["--platform", arch.docker_platform()])
        .arg("--name=rebg-runner")
        .arg(format!(
            "-v={}/container:/container",
            std::env::current_dir().unwrap().to_str().unwrap()
        ))
        .arg(image_name)
        .stdin(Stdio::null())
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let output = run.wait_with_output().unwrap();
    let id = String::from_utf8(output.stdout).unwrap().trim().to_string();

    println!("Spawned new runner with id: {}", id);

    id
}

#[derive(argh::FromArgs)]
/// tracer
struct Arguments {
    /// the program to trace
    #[argh(positional)]
    program: PathBuf,

    #[argh(option, short = 'a')]
    /// architecture: arm, x64, ...
    arch: Option<Arch>,

    #[argh(option, short = 'i')]
    /// docker image to use
    image: Option<String>,

    #[argh(option, short = 'e')]
    /// existing container to use
    container: Option<String>,
}

enum Arch {
    ARM64,
    X86_64,
}

impl argh::FromArgValue for Arch {
    fn from_arg_value(value: &str) -> Result<Self, String> {
        match value {
            "arm64" | "arm" | "aarch64" => Ok(Arch::ARM64),
            "x86_64" | "amd64" | "amd" | "x64" => Ok(Arch::X86_64),
            _ => Err(format!("Unknown arch: {}", value)),
        }
    }
}

impl Arch {
    fn from_elf(machine: u16) -> anyhow::Result<Self> {
        match machine {
            0xB7 => Ok(Arch::ARM64),
            0x3E => Ok(Arch::X86_64),
            _ => Err(anyhow::anyhow!("Unknown machine: {}", machine)),
        }
    }
}

impl Arch {
    fn make_capstone(&self) -> Result<Capstone, capstone::Error> {
        let cs = Capstone::new();

        match self {
            Arch::ARM64 => cs
                .arm64()
                .mode(capstone::arch::arm64::ArchMode::Arm)
                .detail(true)
                .build(),
            Arch::X86_64 => cs
                .x86()
                .mode(capstone::arch::x86::ArchMode::Mode64)
                .detail(true)
                .build(),
        }
    }

    fn qemu_user_bin(&self) -> &str {
        match self {
            Arch::ARM64 => "qemu-aarch64",
            Arch::X86_64 => "qemu-x86_64",
        }
    }

    fn docker_platform(&self) -> &str {
        match self {
            Arch::ARM64 => "linux/arm64",
            Arch::X86_64 => "linux/amd64",
        }
    }

    fn architecture_str(&self) -> &str {
        match self {
            Arch::ARM64 => "arm64",
            Arch::X86_64 => "amd64",
        }
    }
}

fn print_trace<STATE, STEP, const N: usize>(
    trace: &[STEP],
    cs: Capstone,
    syms: Option<&SymbolTable>,
) where
    STATE: State<N>,
    STEP: Step<STATE, N>,
{
    let mut previous_state: Option<STATE> = None;

    for step in trace {
        if let Some(previous) = previous_state {
            let current = step.state();

            let diff = rstate::diff(&previous, current);
            if diff.print::<STATE>() {
                println!();
            }
        }

        let address = step.address();
        let code = step.code();

        let disasm = cs.disasm_all(code, address).unwrap();
        assert_eq!(disasm.len(), 1);
        let disasm = disasm.first().unwrap();
        let dis_mn = disasm.mnemonic().unwrap();
        let dis_op = disasm.op_str().unwrap();

        let symbol = syms.and_then(|s| s.lookup(address));

        let location = if let Some(symbol) = symbol {
            let symbol = format!("<{}>", symbol);
            format!("{:>18}", symbol)
        } else {
            format!("0x{:016x}", address)
        };

        println!("{}: {} {}", location, dis_mn, dis_op);
        // TODO: for some reason the pc is not always the same as the address, especially after cbnz, bl, etc, but also str...

        previous_state = Some(step.state().clone());
    }

    let bytes = std::mem::size_of_val(&trace[0]) * trace.len();
    eprintln!(
        "Used {}kB of memory for {} steps",
        bytes / 1024,
        trace.len()
    );
}

fn main() {
    let Arguments {
        program,
        arch,
        image,
        container,
    } = argh::from_env();

    let buffer = fs::read(&program).unwrap();
    let p = match goblin::Object::parse(&buffer).unwrap() {
        goblin::Object::Elf(e) => e,
        _ => todo!("only elf supported."),
    };

    let arch = arch.unwrap_or_else(|| Arch::from_elf(p.header.e_machine).unwrap());
    let image = image.unwrap_or_else(|| format!("rebg:{}", arch.architecture_str()));
    let id = container.unwrap_or_else(|| spawn_runner(&image, &arch));

    let cs = arch.make_capstone().unwrap();

    let mut output = run_qemu(&id, program.to_str().unwrap(), &arch).unwrap();

    let symbol_table = SymbolTable::from_elf(p);

    let program_path_inside = format!(
        "/container/{}",
        program.file_name().unwrap().to_str().unwrap()
    );

    let elfs = parse_elflibload(&output.next().unwrap()).unwrap();
    let main_binary = elfs.get(&program_path_inside).unwrap();
    let symbol_table = symbol_table.pie(main_binary.0);

    match arch {
        Arch::ARM64 => {
            let trace: Vec<_> = output
                .map(|chunk| Aarch64Step::from_str(&chunk))
                .collect::<anyhow::Result<Vec<_>>>()
                .unwrap();

            print_trace(&trace, cs, Some(&symbol_table));
        }
        Arch::X86_64 => {
            let trace: Vec<_> = output
                .map(|chunk| X64Step::from_str(&chunk))
                .collect::<anyhow::Result<Vec<_>>>()
                .unwrap();

            print_trace(&trace, cs, Some(&symbol_table));
        }
    }
}
