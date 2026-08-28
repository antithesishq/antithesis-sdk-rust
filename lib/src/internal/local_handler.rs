use std::env;
use std::fs::{File, OpenOptions};
use std::io::{Error, Write};

use crate::internal::{LibHandler, LOCAL_OUTPUT};

pub struct LocalHandler {
    writer: File,
}

impl LocalHandler {
    pub fn new() -> Option<Self> {
        let filename = env::var(LOCAL_OUTPUT).ok()?;

        // Open in O_APPEND mode so that concurrent writers do not trample each other
        let create_result = OpenOptions::new().create(true).append(true).open(&filename);
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
        // Avoiding the `writeln!` macro because it issues two write syscalls (one for the provided content and a second for the newline)
        let mut line = String::with_capacity(value.len() + 1);
        line.push_str(value);
        line.push('\n');
        (&self.writer).write_all(line.as_bytes())?;
        Ok(())
    }

    fn random(&self) -> u64 {
        rand::random::<u64>()
    }
}
