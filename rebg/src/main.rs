use std::{
    path::PathBuf,
    process::{exit, Command, Stdio},
};

use capstone::{prelude::BuildsCapstone, Capstone};

struct Step<A, C, R, const N: usize> {
    address: A,
    code: C,
    pc: R,
    registers: [R; N],
}

// define ARM64Step as a Step
type ARM64Step = Step<u64, u32, u64, 32>;

fn run_qemu(id: &str, program: &str, arch: &Arch) -> Vec<ARM64Step> {
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

    let chunks = output
        .split("\n----------------")
        .into_iter()
        .map(|x| x.trim())
        .map(|chunk| {
            chunk
                .split("\n")
                .into_iter()
                .map(|x| x.trim())
                .filter(|x| !matches!(*x, "" | "IN:"))
                .filter_map(|x| x.split_once('|'))
        });

    let mut steps = Vec::new();

    for chunk in chunks {
        let mut s_registers = None;
        let mut s_address = None;
        let mut s_code = None;
        let mut s_pc = None;

        for (what, content) in chunk {
            match what {
                "regs" => {
                    let regs = content
                        .split("|")
                        .map(|data| data.split_once('='))
                        .map(Option::unwrap)
                        .map(|(name, value)| {
                            (name.trim(), u64::from_str_radix(value, 16).unwrap())
                        });

                    let mut registers = [0u64; 32];

                    for (name, value) in regs {
                        match name {
                            "pc" => {
                                s_pc = Some(value);
                            }
                            _ => {
                                let index = name.strip_prefix('x').unwrap();
                                let index = usize::from_str_radix(index, 10).unwrap();
                                registers[index] = value;
                            }
                        }
                    }

                    s_registers = Some(registers);
                }
                "flags" => {
                    let flag = u64::from_str_radix(content, 16).unwrap();
                }
                "header" => {
                    let (address, code) = content.split_once("|").unwrap();

                    let address = u64::from_str_radix(address, 16).unwrap();
                    let code = u32::from_str_radix(code, 16).unwrap();

                    s_address = Some(address);
                    s_code = Some(code);
                }
                _ => panic!("unknown data"),
            }
        }

        let address = s_address.unwrap();
        let code = s_code.unwrap();
        let registers = s_registers.unwrap();
        let pc = s_pc.unwrap();

        steps.push(ARM64Step {
            address,
            code,
            pc,
            registers,
        });
    }

    steps
}

fn spawn_runner(image_name: &str) -> String {
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
            "arm64" | "arm" => Ok(Arch::ARM64),
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
        spawn_runner(&image)
    };

    let trace = run_qemu(&id, &program, &arch);

    let cs = arch.make_capstone().unwrap();

    for Step {
        address,
        code,
        pc: _,
        registers: _,
    } in trace
    {
        let disasm = cs.disasm_all(&code.to_be_bytes(), address).unwrap();
        assert_eq!(disasm.len(), 1);
        let disasm = disasm.first().unwrap();
        let dis_mn = disasm.mnemonic().unwrap();
        let dis_op = disasm.op_str().unwrap();

        println!("0x{:016x}: {:08x} {} {}", address, code, dis_mn, dis_op);
    }
}
