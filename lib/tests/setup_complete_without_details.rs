#[cfg(test)]
use serde_json::{json};
use antithesis_sdk_rust::{lifecycle};
mod common;

use common::{AntithesisSetup, SDKInput};

const LOCAL_OUTPUT: &str = "ANTITHESIS_SDK_LOCAL_OUTPUT";

// ───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
// File: /tmp/antithesis-lifecycle.json
// ───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
// {"antithesis_sdk":{"language":{"name":"Rust","version":"1.77.1"},"protocol_version":"1.0.0","sdk_version":"0.1.1"}}
// {"antithesis_setup":{"details":{},"status":"complete"}}
// ───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
#[test]
fn setup_complete_without_details() {

    let output_file = "/tmp/antithesis-lifecycle.json";
    let prev_v = common::env::set_var(LOCAL_OUTPUT, output_file);
    let no_details = json!({});

    lifecycle::setup_complete(&no_details);

    // verify the output has landed in the expected file
    match common::read_jsonl_tags(output_file) {
        Ok(x) => {
            for obj in x.iter() {
                match obj {
                    SDKInput::AntithesisSetup(AntithesisSetup{status, details}) => {
                        assert_eq!(status, "complete");
                        assert_eq!(details, &no_details)
                    },
                    _ => ()
                }
            }
        },
        Err(e) => println!("{}", e)
    }
    common::env::restore_var(LOCAL_OUTPUT, prev_v);
}
