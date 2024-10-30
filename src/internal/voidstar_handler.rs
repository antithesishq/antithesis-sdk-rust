use libc::{c_char, size_t};
use libloading::Library;
use std::io::Error;

use crate::internal::LibHandler;

const LIB_NAME: &str = "/usr/lib/libvoidstar.so";


pub struct VoidstarHandler {
    // Not used directly but exists to ensure the library is loaded
    // and all the following function pointers points to valid memory.
    _lib: Library,
    // SAFETY: The memory pointed by `s` must be valid up to `l` bytes.
    fuzz_json_data: unsafe fn(s: *const c_char, l: size_t),
    fuzz_get_random: fn() -> u64,
    fuzz_flush: fn(),
}

impl VoidstarHandler {
    pub fn try_load() -> Result<Self, libloading::Error> {
        // SAFETY:
        // - The `libvoidstar`/`libmockstar `libraries that we intended to load
        //   should not have initalization procedures that requires special arrangments at loading time.
        //   Otherwise, loading an arbitrary library that happens to be at `LIB_NAME` is an unsupported case.
        // - Similarly, we load symbols by names and assume they have the expected signatures,
        //   and loading arbitrary symbols that happen to take those names are unsupported.
        // - `fuzz_json_data` and `fuzz_get_random` copy the function pointers,
        //   but they would be valid as we bind their lifetime to the library they are from
        //   by storing all of them in the `VoidstarHandler` struct.
        unsafe {
            let lib = Library::new(LIB_NAME)?;
            let fuzz_json_data = *lib.get(b"fuzz_json_data\0")?;
            let fuzz_get_random = *lib.get(b"fuzz_get_random\0")?;
            let fuzz_flush = *lib.get(b"fuzz_flush\0")?;
            Ok(VoidstarHandler {
                _lib: lib,
                fuzz_json_data,
                fuzz_get_random,
                fuzz_flush,
            })
        }
    }
}

impl LibHandler for VoidstarHandler {
    fn output(&self, value: &str) -> Result<(), Error> {
        // SAFETY: The data pointer and length passed into `fuzz_json_data` points to valid memory
        // that we just initialized above.
        unsafe {
            (self.fuzz_json_data)(value.as_bytes().as_ptr() as *const c_char, value.len());
            (self.fuzz_flush)();
        }
        Ok(())
    }

    fn random(&self) -> u64 {
        (self.fuzz_get_random)()
    }
}
