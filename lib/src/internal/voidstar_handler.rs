use libc::{c_char, size_t};
use libloading::{Library, Symbol};
use serde_json::{Value};
use std::io::{Error};

use crate::internal::{LibHandler};

const LIB_NAME: &str = "/usr/lib/libmockstar.so";

#[derive(Debug)]
pub struct VoidstarHandler {
    voidstar_lib: Library,
}

impl VoidstarHandler {
    pub fn try_load() -> Result<Self, libloading::Error> {
        let lib = unsafe { Library::new(LIB_NAME) }?;
        Ok(VoidstarHandler { voidstar_lib: lib })
    }
}

impl LibHandler for VoidstarHandler {
    fn output(&self, value: &Value) -> Result<(), Error> {
        let payload = value.to_string();
        unsafe {
            let json_data_func: Symbol<unsafe extern fn(s: *const c_char, l: size_t)> = self.voidstar_lib.get(b"fuzz_json_data").unwrap();
            json_data_func(payload.as_bytes().as_ptr() as *const c_char, payload.len());
        }
        Ok(())
    }

    fn random(&self) -> u64 {
        unsafe {
            let get_random_func: Symbol<unsafe extern fn() -> u64> = self.voidstar_lib.get(b"fuzz_get_random").unwrap();
            get_random_func()
        }
    }
}

