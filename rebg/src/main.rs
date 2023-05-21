use anyhow::Context;
use capstone::{prelude::BuildsCapstone, Capstone};
use hex::FromHex;
use num_traits::Num;
use std::{
    fmt::{Debug, LowerHex},
    path::PathBuf,
    process::{exit, Command, Stdio},
};

mod arch;
use arch::{ARM64Step, Code, X64Step};

trait Step {
    type Code: Code;
    type State;
    fn address(&self) -> u64;
    fn code(&self) -> &Self::Code;
    fn state(&self) -> &Self::State;
}

pub struct StepStruct<A, C, R> {
    address: A,
    code: C,
    state: R,
}

pub struct CpuState<B, const N: usize> {
    regs: [B; N],
    pc: B,
    flags: B,
}

struct QemuParser;

impl QemuParser {
    fn parse_regs<B, const N: usize>(input: &str) -> anyhow::Result<CpuState<B, N>>
    where
        B: Num + Copy,
        <B as Num>::FromStrRadixErr: Debug,
    {
        let regs = input
            .split('|')
            .map(|data| data.split_once('='))
            .map(Option::unwrap)
            .map(|(name, value)| (name.trim(), B::from_str_radix(value, 16).unwrap()));

        let mut registers: [Option<B>; N] = [None; N];
        let mut pc = None;
        let mut flags = None;

        for (name, value) in regs {
            match name {
                "pc" => {
                    pc = Some(value);
                }
                "flags" => {
                    flags = Some(value);
                }
                _ => {
                    let index = name.strip_prefix('r').context("missing register prefix")?;
                    let index = usize::from_str_radix(index, 10)?;
                    registers[index] = Some(value);
                }
            }
        }

        let pc = pc.unwrap();
        let flags = flags.unwrap();

        if registers.contains(&None) {
            return Err(anyhow::anyhow!("register not set"));
        }
        let registers = registers.map(Option::unwrap);

        Ok(CpuState {
            regs: registers,
            pc,
            flags,
        })
    }

    fn parse<'a, I, B, C, const N: usize>(
        input: I,
    ) -> anyhow::Result<StepStruct<B, C, CpuState<B, N>>>
    where
        I: Iterator<Item = &'a str>,
        B: Num + Copy,
        <B as Num>::FromStrRadixErr: Debug,
        C: Code,
        <C as FromHex>::Error: Debug,
    {
        let lines = input.filter_map(|x| x.split_once('|'));

        let mut s_state = None;
        let mut s_address = None;
        let mut s_code = None;

        for (what, content) in lines {
            match what {
                "regs" => {
                    s_state = Some(Self::parse_regs(content)?);
                }
                "header" => {
                    let (address, code) = content.split_once('|').context("missing |")?;

                    let address = B::from_str_radix(address, 16).unwrap();
                    let code = C::from_hex(code).unwrap();

                    s_address = Some(address);
                    s_code = Some(code);
                }
                _ => panic!("unknown data"),
            }
        }

        let address = s_address.unwrap();
        let code = s_code.unwrap();
        let state = s_state.unwrap();

        Ok(StepStruct {
            address,
            code,
            state,
        })
    }
}

fn run_qemu<C, B, const N: usize>(
    id: &str,
    program: &str,
    arch: &Arch,
) -> anyhow::Result<Vec<StepStruct<B, C, CpuState<B, N>>>>
where
    C: Code,
    B: Num + Copy + Debug + LowerHex,
    <B as Num>::FromStrRadixErr: Debug,
    C: Code,
    <C as FromHex>::Error: Debug,
{
    // copy program into container

    // just copy it into the `container` folder
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

    let output = String::from_utf8(result.stderr).unwrap();

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

    for (key, value) in header_chunk {
        match key {
            "elflibload" => {
                let (path, other) = value.split_once('|').unwrap();
                let (from, to) = other.split_once('|').unwrap();

                let from = B::from_str_radix(from, 16).unwrap();
                let to = B::from_str_radix(to, 16).unwrap();

                println!("{}, {:x}:{:x}", path, from, to);
            }
            _ => {
                return Err(anyhow::anyhow!("unknown header key: {}", key));
            }
        }
    }

    // this implicitly converts Vec<Result<T>> to Result<Vec<T>>
    chunks.map(QemuParser::parse).collect()
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
    program: String,

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

fn print_trace<S: Step>(trace: &[S], cs: Capstone) {
    for step in trace {
        let address = step.address();
        let code = step.code();

        let disasm = cs.disasm_all(code.be_bytes(), address).unwrap();
        assert_eq!(disasm.len(), 1);
        let disasm = disasm.first().unwrap();
        let dis_mn = disasm.mnemonic().unwrap();
        let dis_op = disasm.op_str().unwrap();

        println!("0x{:016x}: {:08x} {} {}", address, code, dis_mn, dis_op);
        // TODO: for some reason the pc is not always the same as the address, especially after cbnz, bl, etc, but also str...
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

    let id = if let Some(container) = container {
        container
    } else {
        spawn_runner(&image, &arch)
    };

    let cs = arch.make_capstone().unwrap();

    match arch {
        Arch::ARM64 => {
            let trace: Vec<ARM64Step> = run_qemu(&id, &program, &arch).unwrap();
            print_trace(&trace, cs);
        }
        Arch::X86_64 => {
            let trace: Vec<X64Step> = run_qemu(&id, &program, &arch).unwrap();
            print_trace(&trace, cs);
        }
    }
}
