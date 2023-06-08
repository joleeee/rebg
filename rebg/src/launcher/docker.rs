use std::{
    fs,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
};

use crate::arch::Arch;

use super::Launcher;

#[derive(argh::FromArgs)]
#[argh(subcommand, name = "docker")]
/// run inside docker
pub struct DockerArgs {
    #[argh(option, short = 'i')]
    /// optional image, ignored if `container` is set
    pub image: Option<String>,

    #[argh(option, short = 'e')]
    /// option existing container
    pub container: Option<String>,
}

/// Filled out fields, used for starting
pub struct DockerSpawner {
    pub program: PathBuf,
    pub arch: Arch,
    pub image: String,
}

impl DockerSpawner {
    fn spawn_container(&self) -> String {
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
            .args(["--platform", self.arch.docker_platform()])
            .arg("--name=rebg-runner")
            .arg(format!(
                "-v={}/container:/container",
                std::env::current_dir().unwrap().to_str().unwrap()
            ))
            .arg(&self.image)
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

        // copy program into container
        let cp = Command::new("cp")
            .arg(&self.program)
            .arg("container/")
            .spawn()
            .unwrap();
        cp.wait_with_output().unwrap();

        id
    }

    pub fn spawn(self) -> Docker {
        let id = self.spawn_container();
        Docker {
            target_arch: self.arch,
            id,
        }
    }
}

impl DockerArgs {
    /// Starts the container (but not the program yet!)
    /// If a id was given, just uses that
    pub fn start(self, program: PathBuf, arch: Arch) -> Docker {
        let image = self
            .image
            .unwrap_or_else(|| format!("rebg:{}", arch.architecture_str()));

        match self.container {
            Some(id) => Docker {
                target_arch: arch,
                id,
            },
            None => DockerSpawner {
                program,
                arch,
                image,
            }
            .spawn(),
        }
    }
}

/// This has the setup image
pub struct Docker {
    // docker specfic. native can't really do otherwise (except, maybe? like, rosetta stuff???)
    pub target_arch: Arch,

    /// The running container
    pub id: String,
}

impl Docker {
    /// Get the real path (annoying)
    fn get_absolute_path(&self, path: &Path) -> anyhow::Result<String> {
        let output = Command::new("docker")
            .arg("exec")
            .arg(&self.id)
            .args(["realpath", path.to_str().unwrap()])
            .output()?;

        assert!(output.status.success());
        let realpath = String::from_utf8(output.stdout).unwrap();
        let realpath = realpath.trim();
        Ok(realpath.to_string())
    }
}

impl Launcher for Docker {
    type Error = anyhow::Error;

    fn read_file(&self, path: &Path) -> Result<Vec<u8>, anyhow::Error> {
        let realpath = self.get_absolute_path(path)?;

        // copy it out
        Command::new("docker")
            .arg("cp")
            .arg(format!("{}:{}", self.id, realpath))
            .arg("/tmp/rebg-tmp")
            .output()?;

        // read it
        let bytes = fs::read("/tmp/rebg-tmp")?;

        // delete /tmp/rebg.tmp from the local machine
        fs::remove_file("/tmp/rebg-tmp")?;

        Ok(bytes)
    }

    fn launch(&self, program: String, mut args: Vec<String>) -> Result<Child, Self::Error> {
        // run qemu inside the container
        println!("Starting qemu");

        let actual_program = args.last_mut().unwrap();
        *actual_program = format!(
            "/container/{}",
            PathBuf::from(&actual_program)
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
        );

        let child = Command::new("docker")
            .arg("exec")
            .arg(&self.id)
            .arg(program)
            .args(args)
            .stdin(Stdio::null()) // todo pass through from nc
            .stdout(Stdio::piped()) // same here
            .stderr(Stdio::piped()) // also use different file descriptors for qemu output so they dont collide
            .spawn()?;

        Ok(child)
    }
}
