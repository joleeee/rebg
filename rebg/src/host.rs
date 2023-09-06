pub mod docker;
pub mod native;

use std::{path::Path, process::Child};

/// - Runs the binary
/// - Exposes file read (for used libraries)
pub trait Host {
    type Error;
    fn launch(&self, program: String, args: Vec<String>) -> Result<Child, Self::Error>;
    fn read_file(&self, path: &Path) -> Result<Vec<u8>, Self::Error>;
}
