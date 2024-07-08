use rustc_version_runtime::version;
use serde::Serialize;
use std::io::Error;

use noop_handler::NoOpHandler;
#[cfg(feature = "full")]
use voidstar_handler::VoidstarHandler;
#[cfg(feature = "full")]
use local_handler::LocalHandler;

#[cfg(feature = "full")]
use once_cell::sync::Lazy;


mod noop_handler;
#[cfg(feature = "full")]
mod voidstar_handler;

#[cfg(feature = "full")]
mod local_handler;


#[derive(Serialize, Debug)]
struct AntithesisLanguageInfo {
    name: &'static str,
    version: String,
}

#[derive(Serialize, Debug)]
struct AntithesisVersionInfo {
    language: AntithesisLanguageInfo,
    sdk_version: &'static str,
    protocol_version: &'static str,
}

#[derive(Serialize, Debug)]
struct AntithesisSDKInfo {
    antithesis_sdk: AntithesisVersionInfo,
}

// Hardly ever changes, refers to the underlying JSON representation
const PROTOCOL_VERSION: &str = "1.0.0";

// Tracks SDK releases
const SDK_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const LOCAL_OUTPUT: &str = "ANTITHESIS_SDK_LOCAL_OUTPUT";

#[cfg(feature = "full")]
fn get_handler() -> Box<dyn LibHandler + Sync + Send> {
    match VoidstarHandler::try_load() {
        Ok(handler) => Box::new(handler),
        Err(_) => match LocalHandler::new() {
            Some(h) => Box::new(h),
            None => Box::new(NoOpHandler::new()),
        },
    }
}

#[cfg(not(feature = "full"))]
fn get_handler() -> Box<dyn LibHandler + Sync + Send> {
    Box::new(NoOpHandler::new())
}

#[cfg(feature = "full")]
pub(crate) static LIB_HANDLER: Lazy<Box<dyn LibHandler + Sync + Send>> = Lazy::new(|| {
    let handler = get_handler();
    let s = serde_json::to_string(&sdk_info()).unwrap_or("{}".to_owned());
    let _ = handler.output(s.as_str());
    handler
});


#[cfg(not(feature = "full"))]
pub(crate) static LIB_HANDLER: NoOpHandler = NoOpHandler{};

pub(crate) trait LibHandler {
    fn output(&self, value: &str) -> Result<(), Error>;
    fn random(&self) -> u64;
}

// Made public so it can be invoked from the antithesis_sdk::random module
pub(crate) fn dispatch_random() -> u64 {
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
// Made public so it can be invoked from the antithesis_sdk::lifecycle
// and antithesis_sdk::assert module
pub fn dispatch_output<T: Serialize + ?Sized>(json_data: &T) {
    let s = serde_json::to_string(json_data).unwrap_or("{}".to_owned());
    let _ = LIB_HANDLER.output(s.as_str());
}

fn sdk_info() -> AntithesisSDKInfo {
    let language_data = AntithesisLanguageInfo {
        name: "Rust",
        version: version().to_string(),
    };

    let version_data = AntithesisVersionInfo {
        language: language_data,
        sdk_version: SDK_VERSION,
        protocol_version: PROTOCOL_VERSION,
    };

    AntithesisSDKInfo {
        antithesis_sdk: version_data,
    }
}
