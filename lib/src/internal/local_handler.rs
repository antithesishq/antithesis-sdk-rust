use serde_json::{Value};
use std::env;
use std::fs::File;
use std::io::{Write, BufWriter, Error};

use crate::internal::{LibHandler};

static LOCAL_OUTPUT: &str = "ANTITHESIS_SDK_LOCAL_OUTPUT";

// #[allow(dead_code)]
pub struct LocalHandler {
    maybe_writer: Option<BufWriter<Box<dyn Write + Send>>>
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
                maybe_writer: Some(BufWriter::with_capacity(0, Box::new(f)))
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
    fn output(&mut self, value: &Value) -> Result<(), Error> {
        let maybe_writer = self.maybe_writer.as_mut();
        match maybe_writer {
            Some(b2w) => {
                let mut text_line = value.to_string();
                text_line.push('\n');
                b2w.write_all(text_line.as_bytes())?;
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
