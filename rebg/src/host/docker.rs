use bollard::container::{Config, CreateContainerOptions, DownloadFromContainerOptions};
use futures::TryStreamExt;
use std::{
    io::Read,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
};
use tracing::info;

use super::Host;
use crate::arch::Arch;

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
    async fn spawn_container(&self) -> String {
        let docker = bollard::Docker::connect_with_local_defaults().unwrap();

        // stop any previous container
        docker
            .kill_container::<&str>("rebg-runner", None)
            .await
            .ok(); // ignoring result

        // delete previous instance
        docker.remove_container("rebg-runner", None).await.ok(); // ignoring result

        // set up bind mount (bleh)
        let path_mount = format!(
            "{}/container:/container",
            std::env::current_dir().unwrap().to_str().unwrap()
        );

        // spawn container (background)
        let container = docker
            .create_container(
                Some(CreateContainerOptions {
                    name: "rebg-runner",
                    platform: Some(self.arch.docker_platform()),
                }),
                Config {
                    host_config: Some(bollard::service::HostConfig {
                        binds: Some(vec![path_mount]),
                        ..Default::default()
                    }),
                    image: Some(self.image.clone()),
                    cmd: None,
                    tty: Some(true),
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        docker
            .start_container::<String>(&container.id, None)
            .await
            .unwrap();

        // copy program into container
        Command::new("cp")
            .arg(&self.program)
            .arg("container/")
            .spawn()
            .unwrap()
            .wait_with_output()
            .unwrap();

        container.id
    }

    pub fn spawn(self) -> Docker {
        let id = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(self.spawn_container());
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

impl Host for Docker {
    type Error = anyhow::Error;

    fn read_file(&self, path: &Path) -> Result<Vec<u8>, anyhow::Error> {
        let realpath = self.get_absolute_path(path)?;

        let contents: Vec<u8> = tokio::runtime::Runtime::new().unwrap().block_on(async {
            let docker = bollard::Docker::connect_with_local_defaults().unwrap();

            docker
                .download_from_container(
                    &self.id,
                    Some(DownloadFromContainerOptions { path: realpath }),
                )
                .try_collect::<Vec<_>>()
                .await
                .unwrap()
                .into_iter()
                .flatten()
                .collect()
        });

        let mut archive = tar::Archive::new(&contents[..]);

        let mut output = None;

        for file in archive.entries().unwrap() {
            let mut file = file.unwrap();

            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes).unwrap();

            if output.is_some() {
                panic!("multiple files in tar");
            }
            output = Some(bytes);
        }

        Ok(output.expect("no files in tar"))
    }

    fn launch(&self, program: String, mut args: Vec<String>) -> Result<Child, Self::Error> {
        // run qemu inside the container
        info!("Starting qemu");

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
            .arg("-i") // keep stdin open
            .arg(&self.id)
            .arg(program)
            .args(args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::piped()) // todo use different file descriptors for qemu output so they dont collide
            .spawn()?;

        Ok(child)
    }
}
