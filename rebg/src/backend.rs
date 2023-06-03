use crate::arch::Arch;
use std::path::Path;

pub mod qemu;

/// - Gives the specific tracer to be ran, with options
/// - Parses output
pub trait Backend {
    fn command(&self, executable: &Path, arch: Arch) -> (String, Vec<String>);
}
