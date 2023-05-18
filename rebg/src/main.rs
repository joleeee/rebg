use std::{
    path::PathBuf,
    process::{exit, Command, Stdio},
};

use capstone::{prelude::BuildsCapstone, Capstone};

struct Step<A, C, M> {
    address: A,
    code: C,
    mnemonic: M,
}

// define ARM64Step as a Step
type ARM64Step = Step<u64, u32, String>;

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

    let lines = output
        .split('\n')
        .into_iter()
        .filter(|x| x.starts_with("0x"));

    let mut steps = Vec::new();

    for line in lines {
        // a, _, c, _, m -> a, c, m
        let mut parts = line
            .splitn(5, char::is_whitespace)
            .filter(|x| !x.is_empty());

        // text
        let address = parts
            .next()
            .unwrap()
            .strip_suffix(':')
            .unwrap()
            .strip_prefix("0x")
            .unwrap();
        let inst_data = parts.next().unwrap();
        let inst_mnem = parts.next().unwrap().trim(); // why do i only have to trim this

        assert_eq!(parts.next(), None); // only 3

        // binary
        let address = u64::from_str_radix(address, 16).unwrap();
        // x86 can have 15 byte long instructions
        // lets just do arm64 for now...
        let inst_data = u32::from_str_radix(inst_data, 16).unwrap();

        let step = ARM64Step {
            address,
            code: inst_data,
            mnemonic: inst_mnem.to_string(),
        };

        steps.push(step);
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
        mnemonic: _,
    } in trace
    {
        let disasm = cs.disasm_all(&code.to_le_bytes(), address).unwrap();
        assert_eq!(disasm.len(), 1);
        let disasm = disasm.first().unwrap();
        let dis_mn = disasm.mnemonic().unwrap();
        let dis_op = disasm.op_str().unwrap();

        println!("0x{:016x}: {:08x} {} {}", address, code, dis_mn, dis_op);
    }
}
