use std::path::Path;

use crate::host::Host;

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
    /// Reads and parses the binary, adds external debug symbols if needed
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

        // Ok(Binary { raw, elf })
        Ok(Binary { raw, elf })
    }

    pub fn elf(&self) -> &goblin::elf::Elf {
        &self.elf
    }
}
