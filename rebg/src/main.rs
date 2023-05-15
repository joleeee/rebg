use std::{
    os::unix::process::CommandExt,
    path::PathBuf,
    process::{Command, Stdio},
};

fn run_qemu(id: &str, program: &str) {
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
    
    println!("{}", output);
    
    let lines = output.split("\n").into_iter().filter(|x| x.starts_with("0x")).collect::<Vec<&str>>();
    for line in lines {
        println!("{}", line);
    }
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
    run_qemu(&id, "linux-ls");
}
