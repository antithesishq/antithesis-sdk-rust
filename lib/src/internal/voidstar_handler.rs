use libc::{c_char, size_t};
use libloading::{Library, Symbol};
use serde_json::{Value};
use std::io::{Error};
use std::sync::{Once, Mutex, Arc};

use crate::internal::{LibHandler};

static LIB_NAME: &str = "/usr/lib/libmockstar.so";

pub fn has_voidstar() -> bool {
    load_voidstar();
    LIB_VOIDSTAR.lock().unwrap().is_some()
}

static LIB_VOIDSTAR: Mutex<Option<Arc<Library>>> = Mutex::new(None);

fn load_voidstar() {
    static LOAD_VOIDSTAR: Once = Once::new();
    LOAD_VOIDSTAR.call_once(|| {
        let result = unsafe {
            Library::new(LIB_NAME)
        };
        let mut lib_voidstar = LIB_VOIDSTAR.lock().unwrap();
        *lib_voidstar = result.ok().map(Arc::new);
    });
}

#[derive(Debug)]
pub struct VoidstarHandler {
    voidstar_lib: Arc<Library>,
}

impl VoidstarHandler {
    pub fn new() -> Self {
        load_voidstar();
        let lib = LIB_VOIDSTAR.lock().unwrap().as_ref().unwrap().clone();
        VoidstarHandler{
            voidstar_lib: lib,
        }
    }
}

impl LibHandler for VoidstarHandler {
    fn output(&mut self, value: &Value) -> Result<(), Error> {
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

