pub mod docker;
pub mod native;

use std::{path::Path, process::Child};

/// - Runs the binary
/// - Exposes file read (for used libraries)
pub trait Host {
    type Error;
    fn launch(&self, program: &str, args: Vec<String>) -> Result<Child, Self::Error>;
    fn read_file(&self, path: &Path) -> Result<Vec<u8>, Self::Error>;
    /// where to send data (e.g. docker should sent to host machine)
    fn localhost(&self) -> &'static str;
}
