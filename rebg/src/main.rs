use std::process::{Command, Stdio};

struct Step<A, C, M> {
    address: A,
    code: C,
    mnemonic: M,
}

// define ARM64Step as a Step
type ARM64Step = Step<u64, u32, String>;

fn run_qemu(id: &str, program: &str) -> Vec<ARM64Step> {
    // copy program into container
    //let mut copy = Command::new("docker").arg("cp").arg(program).arg(format!("{}:/{}", id, program)).spawn().unwrap();

    // just copy it into the `container` folder
    let cp = Command::new("cp")
        .arg(program)
        .arg("container/")
        .spawn()
        .unwrap();
    cp.wait_with_output().unwrap();

    // run qemu inside the container
    let guest_path = format!("/container/{}", program);
    let run = Command::new("docker")
        .arg("exec")
        .arg(id)
        .arg("qemu-aarch64")
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
        let inst_mnem = parts.next().unwrap();

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

fn spawn_runner() -> String {
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
        .arg("rebg")
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

fn main() {
    let id = spawn_runner();
    let trace = run_qemu(&id, "linux-ls");
    for Step {
        address,
        code,
        mnemonic,
    } in trace
    {
        println!("0x{:016x}: {:08x} {}", address, code, mnemonic);
    }
}
