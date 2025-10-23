// macro_rules! assert_sometimes {
//    ($condition:expr, $message:literal$(, $details:expr)?) => { ... };
//    ($($rest:tt)*) => { ... };
// }
// Gary M. CX technical screen

use antithesis_sdk::{antithesis_init, assert_sometimes, LOCAL_OUTPUT};
use serde_json::{json, Value};

mod common;
use common::{AntithesisAssert, AssertType, SDKInput};

// Expected Output in /tmp/antithesis-assert-sometimes-with-details.json
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
//   "antithesis_assert": {
//     "assert_type": "sometimes",
//     "condition": false,
//     "details": {},
//     "display_type": "sometimes",
//     "hit": false,
//     "id": "Waterproof Red",
//     "location": {
//       "begin_column": 5,
//       "begin_line": 23,
//       "class": "assert_sometimes_with_details",
//       "file": "lib/tests/assert_sometimes_with_details.rs",
//       "function": "assert_sometimes_with_details::assert_sometimes_with_details"
//     },
//     "message": "Waterproof Red",
//     "must_hit": true
//   }
// }
// {
//   "antithesis_assert": {
//     "assert_type": "sometimes",
//     "condition": true,
//     "details": {
//       "color": "red",
//       "labels": [
//         "outdoor",
//         "waterproof"
//       ],
//       "width": 4
//     },
//     "display_type": "sometimes",
//     "hit": true,
//     "id": "Waterproof Red",
//     "location": {
//       "begin_column": 5,
//       "begin_line": 23,
//       "class": "assert_sometimes_with_details",
//       "file": "lib/tests/assert_sometimes_with_details.rs",
//       "function": "assert_sometimes_with_details::assert_sometimes_with_details"
//     },
//     "message": "Waterproof Red",
//     "must_hit": true
//   }
// }

#[test]
fn assert_sometimes_with_details() {
    let output_file = "/tmp/antithesis-assert-sometimes-with-details.json";
    let prev_v = common::env::set_var(LOCAL_OUTPUT, output_file);
    antithesis_init();
    let clothing_details: Value = json!({
        "color": "red",
        "width": 4,
        "labels": [
            "outdoor",
            "waterproof"
        ]
    });
    let is_waterproof = true;
    assert_sometimes!(is_waterproof, "Waterproof Red", &clothing_details);

    // verify the output has landed in the expected file
    match common::read_jsonl_tags(output_file) {
        Ok(x) => {
            let mut did_register = false;
            let mut did_hit = false;
            for obj in x.iter() {
                if let SDKInput::AntithesisAssert(AntithesisAssert {
                    assert_type,
                    condition,
                    display_type,
                    hit,
                    must_hit,
                    id,
                    message,
                    location,
                    details,
                }) = obj
                {
                    if *hit {
                        did_hit = true;
                        assert_eq!(*condition, is_waterproof);
                        assert_eq!(details, &clothing_details);
                    } else {
                        did_register = true;
                    };
                    assert_eq!(*assert_type, AssertType::sometimes);
                    assert_eq!(*display_type, "sometimes");
                    assert!(*must_hit);
                    assert_eq!(message, "Waterproof Red");
                    assert_eq!(id, message);
                    assert!(location.begin_line > 0);
                    assert!(location.begin_column >= 0);
                    assert_eq!(location.class, "assert_sometimes_with_details");
                    assert!(location.function.ends_with("::assert_sometimes_with_details"));
                    assert!(location
                        .file
                        .ends_with("/tests/assert_sometimes_with_details.rs"));
                }
                println!("{:?}", obj);
            }
            assert!(did_register);
            assert!(did_hit);
        }
        Err(e) => println!("{}", e),
    }
    common::env::restore_var(LOCAL_OUTPUT, prev_v);
}
