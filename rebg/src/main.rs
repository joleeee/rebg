use anyhow::Context;
use capstone::{prelude::BuildsCapstone, Capstone};
use flume;
use lazy_static::lazy_static;
use regex::Regex;
use state::{Aarch64Step, State, Step, X64Step};
use std::{
    collections::HashMap,
    fmt, fs,
    io::{BufRead, BufReader},
    path::PathBuf,
    process::{Child, Command, Stdio},
    thread,
};
use syms::SymbolTable;

use crate::state::{MemoryOp, MemoryOpKind};

mod rstate;
mod state;
mod syms;

#[derive(Debug)]
enum QemuHeader<STEP, const N: usize>
where
    STEP: Step<N>,
{
    ElfLoad(HashMap<String, (u64, u64)>),
    Upgrade(flume::Receiver<QemuMessage<STEP, N>>),
}

#[derive(Debug)]
enum QemuMessage<STEP, const N: usize>
where
    STEP: Step<N>,
{
    Step(STEP),
    Final(std::process::Output),
}

fn parse_qemu<STEP, const N: usize>(
    mut child: Child,
) -> anyhow::Result<flume::Receiver<QemuHeader<STEP, N>>>
where
    STEP: Step<N> + Send + 'static + fmt::Debug,
    STEP: for<'a> TryFrom<&'a [String], Error = anyhow::Error>,
{
    let (header_tx, header_rx) = flume::unbounded();

    enum CurrentTx<STEP, const N: usize>
    where
        STEP: Step<N>,
    {
        Header(flume::Sender<QemuHeader<STEP, N>>),
        Body(flume::Sender<QemuMessage<STEP, N>>),
    }

    let mut current_tx = CurrentTx::Header(header_tx);

    thread::spawn(move || {
        let stderr = child.stderr.take().unwrap();
        let mut stderr = BufReader::new(stderr);

        let mut lines: Vec<String> = vec![];

        loop {
            let done = lines.last().map(|x| x.as_str()) == Some(&"----------------");

            if done {
                lines.pop();

                match current_tx {
                    CurrentTx::Header(ref htx) => {
                        let e = parse_elflibload(&lines).unwrap();
                        let e = QemuHeader::ElfLoad(e);
                        htx.send(e).unwrap();

                        // switch to body
                        let (btx, brx) = flume::unbounded();
                        htx.send(QemuHeader::Upgrade(brx)).unwrap();
                        current_tx = CurrentTx::Body(btx);
                    }
                    CurrentTx::Body(ref btx) => {
                        let s = STEP::try_from(&lines).unwrap();
                        let s = QemuMessage::Step(s);
                        btx.send(s).unwrap();
                    }
                }

                lines.clear();
            }

            let mut stderr_buf = String::new();
            let result = stderr.read_line(&mut stderr_buf).unwrap();
            if result == 0 {
                // EOF

                // now make sure it closed gracefully
                let result = child.wait_with_output().unwrap();

                match current_tx {
                    CurrentTx::Header(_) => {
                        panic!("program quit before sending header {:?}", result)
                    }
                    CurrentTx::Body(ref btx) => btx.send(QemuMessage::Final(result)).unwrap(),
                };

                return;
            }
            lines.push(stderr_buf.strip_suffix('\n').unwrap().to_string());
        }
    });

    Ok(header_rx)
}

fn run_qemu(id: &str, program: &str, arch: &Arch) -> anyhow::Result<Child> {
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

    println!("Starting qemu");

    let child = Command::new("docker")
        .arg("exec")
        .arg(id)
        .arg(arch.qemu_user_bin())
        .arg("-one-insn-per-tb")
        .args(["-d", "in_asm,strace"])
        .arg(guest_path)
        .stdin(Stdio::null()) // todo pass through from nc
        .stdout(Stdio::piped()) // same here
        .stderr(Stdio::piped()) // also use different file descriptors for qemu output so they dont collide
        .spawn()?;

    Ok(child)
}

fn parse_elflibload(output: &[String]) -> anyhow::Result<HashMap<String, (u64, u64)>> {
    let parts: Vec<_> = output
        .iter()
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
    if !output.status.success() {
        panic!("Failed to start the container: {:#?}", output);
    }
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

fn inst_to_str(inst: &capstone::Insn, table: Option<&SymbolTable>) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r#"(.*)0x([0-9a-fA-F]*)(.*)"#).unwrap();
    }

    let mn = inst.mnemonic().unwrap();
    let op = inst.op_str().unwrap();

    let op = match RE.captures(op).zip(table) {
        Some((caps, table)) => {
            let mut caps = caps.iter();

            let _whole = caps.next().unwrap().unwrap().as_str();

            let parts: Vec<_> = caps.map(|x| x.unwrap()).map(|x| x.as_str()).collect();

            let (first, rest) = parts.split_first().unwrap();
            let (last, middle) = rest.split_last().unwrap();

            let mut middle: Vec<_> = middle
                .iter()
                .map(|x| u64::from_str_radix(x, 16).unwrap())
                .map(|x| match table.lookup(x) {
                    Some(sym) => format!("<{}>", sym),
                    None => format!("0x{:x}", x),
                })
                .collect();

            let mut strs = vec![];
            strs.push(first.to_string());
            strs.append(&mut middle);
            strs.push(last.to_string());

            strs.join("")
        }
        None => op.to_string(),
    };

    format!("{} {}", mn, op)
}

fn print_trace<STEP, const N: usize>(trace: &[STEP], cs: &Capstone, syms: Option<&SymbolTable>)
where
    <STEP as Step<N>>::STATE: State<N>,
    STEP: Step<N>,
{
    let mut previous_state: Option<STEP::STATE> = None;

    for step in trace {
        if let Some(previous) = previous_state {
            let current = step.state();

            let diff = rstate::diff(&previous, current);
            diff.print::<STEP::STATE>();
            println!();
        }

        let address = step.address();
        let code = step.code();

        let disasm = cs.disasm_all(code, address).unwrap();
        assert_eq!(disasm.len(), 1);
        let op = inst_to_str(disasm.first().unwrap(), syms);

        let symbol = syms.and_then(|s| s.lookup(address));

        let location = if let Some(ref symbol) = symbol {
            let symbol = format!("<{}>", symbol);
            format!("{:>18}", symbol)
        } else {
            format!("0x{:016x}", address)
        };

        println!("{}: {}", location, op);
        // TODO: for some reason the pc is not always the same as the address, especially after cbnz, bl, etc, but also str...
        // EDIT: it seems like it happens when branching to somewhere doing a syscall. it results in two regs| messages, and the last one is the one that "counts"..., i guess where it jump to after the syscall is done or something...?
        assert_eq!(address, step.state().pc());

        if let Some(strace) = step.strace() {
            println!("syscall: {}", strace);
        }

        // only print memory changes if we're in the user binary
        for MemoryOp {
            address,
            kind,
            value,
        } in step.memory_ops()
        {
            let arrow = match kind {
                MemoryOpKind::Read => "->",
                MemoryOpKind::Write => "<-",
            };

            println!("0x{:016x} {} 0x{:x}", address, arrow, value.as_u64());
        }

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
    let p = goblin::elf::Elf::parse(&buffer).unwrap();

    let arch = arch.unwrap_or_else(|| Arch::from_elf(p.header.e_machine).unwrap());
    let image = image.unwrap_or_else(|| format!("rebg:{}", arch.architecture_str()));
    let id = container.unwrap_or_else(|| spawn_runner(&image, &arch));

    let child = run_qemu(&id, program.to_str().unwrap(), &arch).unwrap();

    match arch {
        Arch::ARM64 => {
            let rx = parse_qemu(child).unwrap();
            do_the_stuff::<Aarch64Step, 32>(&id, rx, &arch);
        }
        Arch::X86_64 => {
            let rx = parse_qemu(child).unwrap();
            do_the_stuff::<X64Step, 16>(&id, rx, &arch);
        }
    }
}

fn read_file_from_docker(id: &str, path: PathBuf) -> anyhow::Result<Vec<u8>> {
    // get the real path (annoying)
    let output = Command::new("docker")
        .arg("exec")
        .arg(id)
        .args(["realpath", path.to_str().unwrap()])
        .output()?;

    assert!(output.status.success());
    let realpath = String::from_utf8(output.stdout).unwrap();
    let realpath = realpath.trim();

    // copy it out
    Command::new("docker")
        .arg("cp")
        .arg(format!("{}:{}", id, realpath))
        .arg("/tmp/rebg-tmp")
        .output()?;

    // read it
    let bytes = fs::read("/tmp/rebg-tmp")?;

    // delete /tmp/rebg.tmp from the local machine
    fs::remove_file("/tmp/rebg-tmp")?;

    Ok(bytes)
}

fn do_the_stuff<STEP: Step<N> + fmt::Debug, const N: usize>(
    id: &str,
    rx: flume::Receiver<QemuHeader<STEP, N>>,
    arch: &Arch,
) {
    let cs = arch.make_capstone().unwrap();

    let offsets = match rx.recv().unwrap() {
        QemuHeader::ElfLoad(elfmsg) => elfmsg,
        _ => panic!("Expected elfmsg"),
    };

    let rx = match rx.recv().unwrap() {
        QemuHeader::Upgrade(brx) => brx,
        _ => panic!("Expected upgrade"),
    };

    // get symbol table from all binaries
    let mut symbol_tables = Vec::new();
    for path in offsets.keys() {
        let contents = read_file_from_docker(id, path.into()).unwrap();
        let elf = goblin::elf::Elf::parse(&contents).unwrap();

        let pie = offsets.get(path).unwrap();
        let table = SymbolTable::from_elf(elf).pie(pie.0);

        symbol_tables.push(table);
    }
    // merge into a single table
    let table = symbol_tables
        .into_iter()
        .reduce(|accum, item| accum.merge(item))
        .unwrap();

    let mut trace = Vec::new();
    let result = loop {
        let v = match rx.recv() {
            Ok(v) => v,
            Err(flume::RecvError::Disconnected) => panic!("premature disconnect"),
        };

        match v {
            QemuMessage::Step(step) => {
                trace.push(step);
            }
            QemuMessage::Final(f) => {
                // make sure it's done
                match rx.recv() {
                    Err(flume::RecvError::Disconnected) => (),
                    Ok(_) => panic!("Got message after final"),
                }
                break f;
            }
        }
    };

    print_trace(&trace, &cs, Some(&table));

    if !result.status.success() {
        println!("Failed with code: {}", result.status);
    }
    if !result.stdout.is_empty() {
        println!("stdout:\n{}", String::from_utf8(result.stdout).unwrap());
    }
    if !result.stderr.is_empty() {
        println!("stderr:\n{}", String::from_utf8(result.stderr).unwrap());
    }
}
