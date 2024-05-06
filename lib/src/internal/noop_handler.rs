use crate::internal::LibHandler;
use std::io::Error;

pub struct NoOpHandler {}

impl NoOpHandler {
    pub fn new() -> Self {
        NoOpHandler {}
    }
}

impl LibHandler for NoOpHandler {
    fn output(&self, _value: &str) -> Result<(), Error> {
        Ok(())
    }

    fn random(&self) -> u64 {
        rand::random::<u64>()
    }
}
