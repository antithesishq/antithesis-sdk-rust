#[cfg(test)]
use serde_json::{json};
use antithesis_sdk_rust::{lifecycle};
mod common;

use common::{SDKInput};

const LOCAL_OUTPUT: &str = "ANTITHESIS_SDK_LOCAL_OUTPUT";

// ───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
// File: /tmp/antithesis-lifecycle.json
// ───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
// {"antithesis_sdk":{"language":{"name":"Rust","version":"1.77.1"},"protocol_version":"1.0.0","sdk_version":"0.1.1"}}
// {"antithesis_setup":{"details":{"age":4,"name":"Tweety Bird","phones":["+1 9734970340"]},"status":"complete"}}
// ───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
#[test]
fn send_event() {

    let output_file = "/tmp/antithesis-send-event.json";
    let prev_v = common::env::set_var(LOCAL_OUTPUT, output_file);
    let details = json!({
        "x": 100,
        "tag": "last value"
    });

    // only added to force the antithesis_sdk info to be generated
    lifecycle::send_event("logging", &details);

    // verify the output has landed in the expected file
    match common::read_jsonl_tags(output_file) {
        Ok(x) => {
            for obj in x.iter() {
                match obj {
                    SDKInput::SendEvent{event_name, details} => {
                        assert_eq!(event_name, "logging");
                        assert_eq!(&details["x"], 100);
                        assert_eq!(&details["tag"], "last value");
                    },
                    _ => ()
                }
            }
        },
        Err(e) => println!("{}", e)
    }
    common::env::restore_var(LOCAL_OUTPUT, prev_v);
}


