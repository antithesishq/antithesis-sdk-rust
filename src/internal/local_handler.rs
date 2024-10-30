use std::env;
use std::fs::File;
use std::io::{Error, Write};

use crate::internal::{LibHandler, LOCAL_OUTPUT};

pub struct LocalHandler {
    writer: File,
}

impl LocalHandler {
    pub fn new() -> Option<Self> {
        let filename = env::var(LOCAL_OUTPUT).ok()?;

        let create_result = File::create(&filename);
        if let Ok(writer) = create_result {
            Some(LocalHandler { writer })
        } else {
            eprintln!(
                "Unable to write to '{}' - {}",
                filename.as_str(),
                create_result.unwrap_err()
            );
            None
        }
    }
}

impl LibHandler for LocalHandler {
    fn output(&self, value: &str) -> Result<(), Error> {
        // The compact Display impl (selected using `{}`) of `serde_json::Value` contains no newlines,
        // hence we are outputing valid JSONL format here.
        // Using the `{:#}` format specifier may results in extra newlines and indentation.
        // See https://docs.rs/serde_json/latest/serde_json/enum.Value.html#impl-Display-for-Value.
        let mut writer_mut = &self.writer;
        writeln!(writer_mut, "{}", value)?;
        writer_mut.flush()?;
        Ok(())
    }

    fn random(&self) -> u64 {
        rand::random::<u64>()
    }
}
