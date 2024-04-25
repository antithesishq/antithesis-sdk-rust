use serde_json::{Value};
use std::env;
use std::fs::File;
use std::io::{Write, Error};

use crate::internal::{LibHandler};

const LOCAL_OUTPUT: &str = "ANTITHESIS_SDK_LOCAL_OUTPUT";

pub struct LocalHandler {
    maybe_writer: Option<File>
}

impl LocalHandler {
    pub fn new() -> Self {
        let filename = match env::var(LOCAL_OUTPUT) {
            Err(_) => return LocalHandler{ maybe_writer: None },
            Ok(s) => s
        };

        let create_result = File::create(&filename);
        if let Ok(f) = create_result {
            // Disabling buffering by setting capacity to 0 for now
            // Inefficient, but ensures that no buffered bytes are abandoned
            // for a LocalHandler instance that does not get Drop'ed
            // Seems like LocalHandler gets bound to a reference with 
            // a 'static lifetime.
            LocalHandler{
                maybe_writer: Some(f)
            }
        } else {
                eprintln!("Unable to write to '{}' - {}", filename.as_str(), create_result.unwrap_err());
                LocalHandler {
                    maybe_writer: None
                }
        }
    }
}

impl LibHandler for LocalHandler {
    fn output(&self, value: &Value) -> Result<(), Error> {
        match &self.maybe_writer {
            Some(b2w) => {
                let mut b2w = b2w;
                // The compact Display impl (selected using `{}`) of `serde_json::Value` contains no newlines,
                // hence we are outputing valid JSONL format here.
                // Using the `{:#}` format specifier may results in extra newlines and indentation.
                // See https://docs.rs/serde_json/latest/serde_json/enum.Value.html#impl-Display-for-Value.
                writeln!(b2w, "{}", value)?;
                b2w.flush()?;
                Ok(())
            },
            None => Ok(())
        }
    }

    fn random(&self) -> u64 {
        rand::random::<u64>()
    }
}
