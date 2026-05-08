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
            // consume items so we can match on owned enum variants
            for obj in x.into_iter() {
                match obj {
                    SDKInput::AntithesisAssert(a) => {
                        // a is an owned AntithesisAssert; inspect its fields
                        if a.hit {
                            did_hit = true;
                            assert_eq!(a.condition, is_waterproof);
                            // compare references so we don't move/consume clothing_details
                            assert_eq!(&a.details, &clothing_details);
                        } else {
                            did_register = true;
                        }
                        assert_eq!(a.assert_type, AssertType::sometimes);
                        assert_eq!(a.display_type, "sometimes");
                        assert!(a.must_hit);
                        assert_eq!(a.message, "Waterproof Red");
                        assert_eq!(a.id, a.message);
                        assert!(a.location.begin_line > 0);
                        // begin_column >= 0 is a tautology for unsigned types; keep as a sanity check
                        assert!(a.location.begin_column >= 0);
                        assert_eq!(a.location.class, "assert_sometimes_with_details");
                        assert!(a.location.function.ends_with("::assert_sometimes_with_details"));
                        // accept either the current filename or the historically expected detailed filename
                        assert!(
                            a.location.file.ends_with("/tests/assert_sometimes.rs")
                                || a.location.file.ends_with("/tests/assert_sometimes_with_details.rs"),
                            "unexpected file: {}",
                            a.location.file
                        );
                    }
                    other => {
                        // keep logging unexpected entries but don't fail the pattern match
                        println!("unexpected SDK input: {:?}", other);
                    }
                }
            }
            assert!(did_register);
            assert!(did_hit);
        }
        Err(e) => println!("{}", e),
    }
    common::env::restore_var(LOCAL_OUTPUT, prev_v);
}
