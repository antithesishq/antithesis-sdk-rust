use antithesis_sdk::{antithesis_init, assert_always, LOCAL_OUTPUT};
use serde_json::json;

mod common;
use common::{AntithesisAssert, AssertType, SDKInput};

// A non-literal but const-evaluable `&'static str`
const MESSAGE: &str = concat!("Const ", "message");

#[test]
fn assert_const_message() {
    let output_file = "/tmp/antithesis-assert-const-message.json";
    let prev_v = common::env::set_var(LOCAL_OUTPUT, output_file);
    antithesis_init();

    assert_always!(true, MESSAGE, &json!({}));

    let mut did_register = false; // the catalog registration entry (hit == false)
    let mut did_hit = false; // the runtime emission (hit == true)
    match common::read_jsonl_tags(output_file) {
        Ok(entries) => {
            for obj in entries.iter() {
                if let SDKInput::AntithesisAssert(AntithesisAssert {
                    assert_type,
                    condition,
                    display_type,
                    hit,
                    must_hit,
                    id,
                    message,
                    location,
                    details: _,
                }) = obj
                {
                    assert_eq!(message, MESSAGE);
                    assert_eq!(id, MESSAGE);
                    assert_eq!(*assert_type, AssertType::Always);
                    assert_eq!(*display_type, "Always");
                    assert!(*must_hit);
                    assert_eq!(location.class, "assert_const_message");
                    assert!(location.function.ends_with("::assert_const_message"));

                    if *hit {
                        did_hit = true;
                        assert!(*condition);
                    } else {
                        did_register = true;
                    }
                }
            }
        }
        Err(e) => panic!("could not read SDK output: {e}"),
    }

    assert!(did_register, "catalog registration entry (hit=false) missing");
    assert!(did_hit, "runtime assertion emission (hit=true) missing");

    common::env::restore_var(LOCAL_OUTPUT, prev_v);
}
