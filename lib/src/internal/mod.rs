use once_cell::sync::Lazy;
use rustc_version_runtime::version;
use serde_json::{Value, json};
use std::io::{Error};
use local_handler::LocalHandler;
use voidstar_handler::{VoidstarHandler};

mod local_handler;
mod voidstar_handler;

// Hardly ever changes, refers to the underlying JSON representation
const PROTOCOL_VERSION: &str = "1.0.0";

// Tracks SDK releases
const SDK_VERSION: &str = "0.1.1";

pub(crate) static LIB_HANDLER: Lazy<Box<dyn LibHandler + Sync + Send>> = Lazy::new(|| {
    let handler: Box<dyn LibHandler + Sync + Send> = match VoidstarHandler::try_load() {
        Ok(handler) => Box::new(handler),
        Err(_) => Box::new(LocalHandler::new()),
    };
    let _ = handler.output(&sdk_info());
    handler
});

pub(crate) trait LibHandler {
    fn output(&self, value: &Value) -> Result<(), Error>;
    fn random(&self) -> u64;
}

// Made public so it can be invoked from the antithesis_sdk_rust::random module
pub fn dispatch_random() -> u64 {
    LIB_HANDLER.random()
}

// Ignore any and all errors - either the output is completed,
// or it fails silently.
//
// For a Voidstar handler, there is no indication that something failed
//
// For a Local handler, either:
// - Output was not requested (so not really an error)
// - Output was requested and attempted, but an io::Error was detected
// in this case the io::Error is silently ignored.
//
// It would be possible to distinguish between these two cases
// and report detected io:Error's but there is no requirement 
// to implement this. 
//
// Made public so it can be invoked from the antithesis_sdk_rust::lifecycle 
// and antithesis_sdk_rust::assert module
pub fn dispatch_output(json_data: &Value) {
    let _ = LIB_HANDLER.output(json_data);
}

fn sdk_info() -> Value {
    let language_info: Value = json!({
        "name": "Rust",
        "version": version().to_string()
    });
    
    let version_info: Value = json!({
        "language": language_info,
        "sdk_version": SDK_VERSION,
        "protocol_version": PROTOCOL_VERSION
    });
    
    json!({
        "antithesis_sdk": version_info
    })
}
