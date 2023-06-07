use std::{
    fs,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
};

use crate::arch::Arch;

use super::Launcher;

/// Cli arguments for creation
#[derive(argh::FromArgs)]
#[argh(subcommand, name = "native")]
pub struct NativeArgs {
    #[argh(option, short = 'p')]
    /// override path to qemu
    pub path: Option<String>,
}

impl NativeArgs {
    pub fn start(self, _program: PathBuf, arch: Arch) -> Native {
        let path = self
            .path
            .unwrap_or_else(|| arch.qemu_user_bin().to_string());

        Native { path }
    }
}

/// This has the setup image
pub struct Native {
    /// Path to qemu
    pub path: String,
}

impl Launcher for Native {
    type Error = anyhow::Error;

    fn read_file(&self, path: &Path) -> Result<Vec<u8>, anyhow::Error> {
        Ok(fs::read(path)?)
    }

    fn launch(&self, program: String, args: Vec<String>) -> Result<Child, Self::Error> {
        let child = dbg!(Command::new(&self.path).args(args))
            .stdin(Stdio::null()) // todo pass through from nc
            .stdout(Stdio::piped()) // same here
            .stderr(Stdio::piped()) // also use different file descriptors for qemu output so they dont collide
            .spawn()?;

        Ok(child)
    }
}
