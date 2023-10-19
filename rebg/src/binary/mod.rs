use std::path::{Path, PathBuf};

use crate::{arch::Arch, host::Host};

use self::box_ptr::BoxData;

// A big big thanks to "alt f4" @__repr__ in the rust discord server for helping
// me with this :)
mod box_ptr {
    use std::ptr::NonNull;

    pub struct BoxData {
        inner: NonNull<[u8]>,
    }

    impl BoxData {
        pub(super) fn from_box(bytes: Box<[u8]>) -> Self {
            BoxData {
                inner: {
                    let ptr = Box::into_raw(bytes);
                    unsafe { NonNull::new_unchecked(ptr) }
                },
            }
        }

        pub(super) fn as_ptr(&self) -> *const [u8] {
            self.inner.as_ptr()
        }

        pub(super) fn as_slice(&self) -> &[u8] {
            unsafe { &*self.inner.as_ptr() }
        }
    }

    // SAFETY: [u8] is valid to send between threads, and inner is a uniqe pointer
    unsafe impl Send for BoxData {}
    unsafe impl Sync for BoxData {}

    impl Drop for BoxData {
        fn drop(&mut self) {
            // SAFETY: inner is a pointer made from a box
            unsafe { drop(Box::from_raw(self.inner.as_ptr())) }
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum BinaryError<LAUNCHERERR>
where
    LAUNCHERERR: std::fmt::Debug,
{
    #[error("launcher")]
    Launcher(LAUNCHERERR),
    #[error("goblin")]
    Goblin(#[from] goblin::error::Error),
}

pub struct Binary<'a> {
    #[allow(dead_code)]
    raw: box_ptr::BoxData,
    elf: goblin::elf::Elf<'a>,
}

impl<'a> Binary<'a> {
    /// Reads and parses the binary
    pub fn from_path<LAUNCHER>(
        launcher: &LAUNCHER,
        path: &Path,
    ) -> Result<Binary<'a>, BinaryError<LAUNCHER::Error>>
    where
        LAUNCHER: Host,
        <LAUNCHER as Host>::Error: std::fmt::Debug,
    {
        let raw = launcher.read_file(path).map_err(BinaryError::Launcher)?;

        let raw = raw.into_boxed_slice();
        let raw = BoxData::from_box(raw);

        let elf = goblin::elf::Elf::parse(unsafe { &*raw.as_ptr() })?;

        Ok(Binary { raw, elf })
    }

    /// Tries finding an elf with debug symbols for this buildid
    pub fn try_from_buildid<LAUNCHER>(
        launcher: &LAUNCHER,
        buildid: &str,
        arch: Arch,
    ) -> Option<Binary<'a>>
    where
        LAUNCHER: Host,
        <LAUNCHER as Host>::Error: std::fmt::Debug,
    {
        let prefix = &buildid[..2];
        let suffix = &buildid[2..];
        for platform in [
            "/usr/lib/debug/.build-id",
            "/usr/x86_64-linux-gnu/lib/debug/.build-id",
            "/usr/aarch64-linux-gnu/lib/debug/.build-id",
        ] {
            let debug_sym_path = format!("{platform}/{prefix}/{suffix}.debug",);

            println!("Trying {}", debug_sym_path);

            let bin = Self::from_path(launcher, &PathBuf::from(&debug_sym_path));

            if let Ok(bin) = bin {
                let bin_arch = Arch::from_elf(bin.elf().header.e_machine).ok();

                if bin_arch != Some(arch) {
                    println!("wrong arch {:?}", bin_arch);
                    continue;
                }

                return Some(bin);
            } else {
                continue;
            }
        }

        None
    }

    pub fn build_id(&self) -> Option<String> {
        let buildid = self
            .elf
            .section_headers
            .iter()
            .find(|s| self.elf.shdr_strtab.get_at(s.sh_name) == Some(".note.gnu.build-id"))?;

        let buildid = {
            let id = &self.raw.as_slice()[buildid.file_range()?];
            // only use the last 20 bytes!!
            let id = &id[id.len() - 20..];
            hex::encode(id)
        };

        Some(buildid)
    }

    pub fn elf(&self) -> &goblin::elf::Elf {
        &self.elf
    }
}
