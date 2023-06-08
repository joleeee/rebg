use std::{
    fs,
    path::Path,
    process::{Child, Command, Stdio},
};

use super::Launcher;

/// run directly
#[derive(argh::FromArgs)]
#[argh(subcommand, name = "native")]
pub struct NativeArgs {}

impl NativeArgs {
    pub fn start(self) -> Native {
        // nothing to setup or copy files, they're already there
        Native {}
    }
}

pub struct Native {}

impl Launcher for Native {
    type Error = anyhow::Error;

    fn read_file(&self, path: &Path) -> Result<Vec<u8>, anyhow::Error> {
        Ok(fs::read(path)?)
    }

    fn launch(&self, program: String, args: Vec<String>) -> Result<Child, Self::Error> {
        let child = Command::new(&program)
            .args(args)
            .stdin(Stdio::null()) // todo pass through from nc
            .stdout(Stdio::piped()) // same here
            .stderr(Stdio::piped()) // also use different file descriptors for qemu output so they dont collide
            .spawn()?;

        Ok(child)
    }
}
