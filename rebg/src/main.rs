use anyhow::Context;
use capstone::{prelude::BuildsCapstone, Capstone};
use flume;
use state::{Aarch64Step, State, Step, X64Step};
use std::{
    collections::HashMap,
    fmt, fs,
    io::{BufRead, BufReader},
    path::PathBuf,
    process::{Child, Command, Stdio},
    str::FromStr,
    thread,
};
use syms::SymbolTable;

mod rstate;
mod state;
mod syms;

#[derive(Debug)]
enum QemuMessage<STEP, const N: usize>
where
    STEP: Step<N>,
{
    ElfLoad(HashMap<String, (u64, u64)>),
    // todo, could send a Update(Rx<Step>) here...
    Step(STEP),
}

fn parse_qemu<STEP, const N: usize>(
    mut child: Child,
) -> anyhow::Result<flume::Receiver<QemuMessage<STEP, N>>>
where
    STEP: Step<N> + Send + 'static + FromStr + fmt::Debug,
    STEP::Err: fmt::Debug,
{
    let (tx, rx) = flume::unbounded();

    thread::spawn(move || {
        let stderr = child.stderr.take().unwrap();
        let mut stderr = BufReader::new(stderr);

        let mut stderr_buf = String::new();

        loop {
            let split = stderr_buf
                .split_once("----------------")
                .map(|(a, b)| (a.trim().to_string(), b.to_string()));

            if let Some((before, after)) = split {
                stderr_buf = after;

                if before.starts_with("elflibload") {
                    let e = parse_elflibload(&before).unwrap();
                    let e = QemuMessage::ElfLoad(e);
                    tx.send(e).unwrap();
                } else {
                    let s = STEP::from_str(&before).unwrap();
                    let s = QemuMessage::Step(s);
                    tx.send(s).unwrap();
                }
            }

            let result = stderr.read_line(&mut stderr_buf).unwrap();
            if result == 0 {
                // EOF
                return;
            }
        }
    });

    Ok(rx)
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
        // EDIT: it seems like it happens when branching to somewhere doing a syscall. it results in two regs| messages, and the last one is the one that "counts"..., i guess where it jump to after the syscall is done or something...?
        assert_eq!(address, step.state().pc());

        if let Some(strace) = step.strace() {
            println!("syscall: {}", strace);
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
    rx: flume::Receiver<QemuMessage<STEP, N>>,
    arch: &Arch,
) {
    let cs = arch.make_capstone().unwrap();

    let offsets = match rx.recv().unwrap() {
        QemuMessage::ElfLoad(elfmsg) => elfmsg,
        _ => panic!("Expected elfmsg"),
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
    loop {
        let v = match rx.recv() {
            Ok(v) => v,
            Err(flume::RecvError::Disconnected) => break,
        };

        match v {
            QemuMessage::Step(step) => {
                trace.push(step);
            }
            QemuMessage::ElfLoad(_) => {
                panic!("Unexpected elf load");
            }
        }
    }

    print_trace(&trace, &cs, Some(&table));
}
