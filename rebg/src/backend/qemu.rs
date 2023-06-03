use super::Backend;
use crate::arch::Arch;
use std::path::Path;

struct QEMU {}

impl Backend for QEMU {
    fn command(&self, executable: &Path, arch: Arch) -> (String, Vec<String>) {
        let binary = arch.qemu_user_bin().to_string();
        let options = vec![
            String::from("-one-insn-per-tb"),
            String::from("-d"),
            String::from("in_asm,strace"),
            executable.to_str().unwrap().to_string(),
        ];

        (binary, options)
    }
}
