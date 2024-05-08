use antithesis_sdk::lifecycle;
use serde_json::json;

mod common;
use common::SDKInput;

const LOCAL_OUTPUT: &str = "ANTITHESIS_SDK_LOCAL_OUTPUT";

// ───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
// File: /tmp/antithesis-lifecycle.json
// ───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
// {"antithesis_sdk":{"language":{"name":"Rust","version":"1.77.1"},"protocol_version":"1.0.0","sdk_version":"0.1.1"}}
// {"antithesis_setup":{"details":{"age":4,"name":"Tweety Bird","phones":["+1 9734970340"]},"status":"complete"}}
// ───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
#[test]
fn sdk_info() {
    let output_file = "/tmp/antithesis-sdk.json";
    let prev_v = common::env::set_var(LOCAL_OUTPUT, output_file);
    let no_details = json!({});

    // only added to force the antithesis_sdk info to be generated
    lifecycle::setup_complete(&no_details);

    // verify the output has landed in the expected file
    match common::read_jsonl_tags(output_file) {
        Ok(x) => {
            for obj in x.iter() {
                if let SDKInput::AntithesisSdk(sdk) = obj {
                    assert_eq!(sdk.protocol_version, "1.0.0");
                    assert_eq!(sdk.language.name, "Rust")
                }
            }
        }
        Err(e) => println!("{}", e),
    }
    common::env::restore_var(LOCAL_OUTPUT, prev_v);
}
