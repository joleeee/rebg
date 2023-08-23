use capstone::{prelude::BuildsCapstone, Capstone};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Arch {
    ARM64,
    X86_64,
}

impl argh::FromArgValue for Arch {
    fn from_arg_value(value: &str) -> Result<Self, String> {
        match value {
            "arm64" | "arm" | "aarch64" => Ok(Arch::ARM64),
            "x86_64" | "amd64" | "amd" | "x64" => Ok(Arch::X86_64),
            _ => Err(format!("Unknown arch: {}", value)),
        }
    }
}

impl Arch {
    pub fn from_elf(machine: u16) -> anyhow::Result<Self> {
        match machine {
            0xB7 => Ok(Arch::ARM64),
            0x3E => Ok(Arch::X86_64),
            _ => Err(anyhow::anyhow!("Unknown machine: {}", machine)),
        }
    }

    pub fn make_capstone(&self) -> Result<Capstone, capstone::Error> {
        let cs = Capstone::new();

        match self {
            Arch::ARM64 => cs
                .arm64()
                .mode(capstone::arch::arm64::ArchMode::Arm)
                .detail(true)
                .build(),
            Arch::X86_64 => cs
                .x86()
                .mode(capstone::arch::x86::ArchMode::Mode64)
                .detail(true)
                .build(),
        }
    }

    pub fn qemu_user_bin(&self) -> &str {
        match self {
            Arch::ARM64 => "qemu-aarch64",
            Arch::X86_64 => "qemu-x86_64",
        }
    }

    pub fn docker_platform(&self) -> &str {
        match self {
            Arch::ARM64 => "linux/arm64",
            Arch::X86_64 => "linux/amd64",
        }
    }

    pub fn architecture_str(&self) -> &str {
        match self {
            Arch::ARM64 => "arm64",
            Arch::X86_64 => "amd64",
        }
    }
}
