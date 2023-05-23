use anyhow::Context;
use capstone::{prelude::BuildsCapstone, Capstone};
use state::{Aarch64Step, State, Step, X64Step};
use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    process::{exit, Command, Stdio},
    str::FromStr,
};
use syms::SymbolTable;

mod rstate;
mod state;
mod syms;

fn run_qemu(id: &str, program: &str, arch: &Arch) -> anyhow::Result<String> {
    // copy program into container
    let cp = Command::new("cp")
        .arg(program)
        .arg("container/")
        .spawn()
        .unwrap();
    cp.wait_with_output().unwrap();

    // run qemu inside the container
    let guest_path = format!(
        "/container/{}",
        PathBuf::from(program)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
    );
    let run = Command::new("docker")
        .arg("exec")
        .arg(id)
        .arg(arch.qemu_user_bin())
        .arg("-one-insn-per-tb")
        .args(["-d", "in_asm"])
        .arg(guest_path)
        .stdin(Stdio::null()) // todo pass through from nc
        .stdout(Stdio::piped()) // same here
        .stderr(Stdio::piped()) // also use different file descriptors for qemu output so they dont collide
        .spawn()
        .unwrap();

    println!("Starting qemu");
    // todo spawn new thread
    let result = run.wait_with_output().unwrap();

    if !result.status.success() {
        println!(
            "QEMU Failed with code {} and err \"{}\"",
            result.status.code().unwrap(),
            String::from_utf8(result.stderr).unwrap().trim()
        );
        exit(1);
    }

    Ok(String::from_utf8(result.stderr).unwrap())
}

struct InitialParseResult {
    remaining: Vec<String>,
    elfs: HashMap<String, (u64, u64)>,
}

fn parse_elflibload(output: &str) -> anyhow::Result<InitialParseResult> {
    let mut chunks = output
        .split("----------------")
        .into_iter()
        .map(|x| x.trim())
        .map(|chunk| {
            chunk
                .split('\n')
                .into_iter()
                .map(|x| x.trim())
                .filter(|x| !matches!(*x, "" | "IN:"))
        });

    let header_chunk: Vec<_> = chunks
        .next()
        .context("missing header")?
        .map(|x| x.split_once('|'))
        .collect::<Option<Vec<_>>>()
        .context("invalid header, should only be | separated key|values")?;

    let mut elfs = HashMap::new();
    for (key, value) in header_chunk {
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

    let remaining = chunks
        .map(|list| list.collect::<Vec<_>>().join("\n"))
        .collect();

    Ok(InitialParseResult { remaining, elfs })
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

    #[argh(positional)]
    /// architecture: arm, x64, ...
    arch: Arch,

    #[argh(option, short = 'i', default = r#"String::from("rebg")"#)]
    /// docker image to use
    image: String,

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
}

fn print_trace<STATE, STEP, const N: usize, FLAGS>(
    trace: &[STEP],
    cs: Capstone,
    syms: Option<&SymbolTable>,
) where
    STATE: State<N>,
    STEP: Step<STATE, N, FLAGS>,
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

        println!("{}: {} {} {}", location, hex::encode(code), dis_mn, dis_op);
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

    let id = container.unwrap_or_else(|| spawn_runner(&image, &arch));

    let cs = arch.make_capstone().unwrap();

    let raw_output = run_qemu(&id, program.to_str().unwrap(), &arch).unwrap();

    let buffer = fs::read(&program).unwrap();

    let p = goblin::Object::parse(&buffer).unwrap();
    let p = match p {
        goblin::Object::Elf(e) => e,
        _ => todo!("only elf supported."),
    };

    let symbol_table = SymbolTable::from_elf(p);

    let program_path_inside = format!(
        "/container/{}",
        program.file_name().unwrap().to_str().unwrap()
    );

    let InitialParseResult { remaining, elfs } = parse_elflibload(&raw_output).unwrap();
    let main_binary = elfs.get(&program_path_inside).unwrap();
    let symbol_table = symbol_table.pie(main_binary.0);

    match arch {
        Arch::ARM64 => {
            let trace: Vec<Aarch64Step> = remaining
                .iter()
                .map(|chunk| Aarch64Step::from_str(chunk))
                .collect::<anyhow::Result<Vec<_>>>()
                .unwrap();

            print_trace(&trace, cs, Some(&symbol_table));
        }
        Arch::X86_64 => {
            let trace: Vec<_> = remaining
                .iter()
                .map(|chunk| X64Step::from_str(chunk))
                .collect::<anyhow::Result<Vec<_>>>()
                .unwrap();

            print_trace(&trace, cs, Some(&symbol_table));
        }
    }
}
