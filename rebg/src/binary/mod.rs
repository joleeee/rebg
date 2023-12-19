use object::{elf, read::elf::FileHeader, Endianness, Object, ObjectSection};
use std::path::{Path, PathBuf};
use tracing::debug;

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
    #[error("object")]
    Object(#[from] object::Error),
}

pub struct Binary<'a> {
    #[allow(dead_code)]
    raw: box_ptr::BoxData,
    obj: object::File<'a>,
    header: elf::FileHeader64<Endianness>,
}

impl<'a> Binary<'a> {
    pub fn from_bytes(bytes: Box<[u8]>) -> Result<Binary<'a>, object::Error> {
        let raw = BoxData::from_box(bytes);
        let obj = object::File::parse(unsafe { &*raw.as_ptr() })?;
        let header = elf::FileHeader64::<Endianness>::parse(unsafe { &*raw.as_ptr() })?.to_owned();
        Ok(Binary { raw, obj, header })
    }

    /// Reads and parses the binary
    pub fn from_path<LAUNCHER>(
        launcher: &LAUNCHER,
        path: &Path,
    ) -> Result<Binary<'a>, BinaryError<LAUNCHER::Error>>
    where
        LAUNCHER: Host,
        <LAUNCHER as Host>::Error: std::fmt::Debug,
    {
        let raw = launcher
            .read_file(path)
            .map_err(BinaryError::Launcher)?
            .into_boxed_slice();

        Ok(Self::from_bytes(raw)?)
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

            debug!("Trying {}", debug_sym_path);

            let bin = Self::from_path(launcher, &PathBuf::from(&debug_sym_path));

            if let Ok(bin) = bin {
                let bin_arch = Arch::from_object(bin.obj.architecture()).ok();

                if bin_arch != Some(arch) {
                    debug!("wrong arch {:?}", bin_arch);
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
        let id = self
            .obj
            .section_by_name(".note.gnu.build-id")?
            .data()
            .ok()?;

        // only use the last 20 bytes!!
        let id = &id[id.len() - 20..];
        let id = hex::encode(id);

        Some(id)
    }

    pub fn obj(&self) -> &object::File {
        &self.obj
    }

    pub fn header(&self) -> &elf::FileHeader64<Endianness> {
        &self.header
    }

    pub fn raw(&self) -> &[u8] {
        self.raw.as_slice()
    }

    pub fn bin(&self) -> &[u8] {
        self.raw.as_slice()
    }
}
