use antithesis_sdk::{lifecycle, LOCAL_OUTPUT};
use serde_json::json;

mod common;
use common::{AntithesisSetup, SDKInput};

// Expected output in /tmp/antithesis-setup-complete-without-details.json
// Note: Actual version info in antithesis_sdk can vary
//
// {
//   "antithesis_sdk": {
//     "language": {
//       "name": "Rust",
//       "version": "1.69.0"
//     },
//     "sdk_version": "0.1.2",
//     "protocol_version": "1.0.0"
//   }
// }
// {
//   "antithesis_setup": {
//     "status": "complete",
//     "details": {}
//   }
// }

#[test]
fn setup_complete_without_details() {
    let output_file = "/tmp/antithesis-setup-complete-without-details.json";
    let prev_v = common::env::set_var(LOCAL_OUTPUT, output_file);
    let no_details = json!({});

    lifecycle::setup_complete(&no_details);

    // verify the output has landed in the expected file
    match common::read_jsonl_tags(output_file) {
        Ok(x) => {
            for obj in x.iter() {
                if let SDKInput::AntithesisSetup(AntithesisSetup { status, details }) = obj {
                    assert_eq!(status, "complete");
                    assert_eq!(details, &no_details)
                }
            }
        }
        Err(e) => println!("{}", e),
    }
    common::env::restore_var(LOCAL_OUTPUT, prev_v);
}
