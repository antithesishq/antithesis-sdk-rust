use serde_json::json;
use antithesis_sdk::{antithesis_init, assert_always, assert_always_greater_than, assert_always_or_unreachable, assert_reachable, assert_sometimes, assert_sometimes_all, assert_unreachable};

/// Demonstrates the various assertion macros available in the Antithesis SDK
fn assert_demo() {
    // todo: this is confusing for me at first assert_always!(true, ...) did not really make sense to me why we were asserting true. also things 777 is confusing to me.
    let details = json!({"things": 13});
    assert_always!(true, "Things 777 look good", &details);

    let details = json!({"more things": "red and blue"});
    assert_always_or_unreachable!(true, "A few colors", &details);

    let details = json!({"notes": [1,2,3,4,5]});
    assert_sometimes!(false, "Notes have small values", &details);

    for i in 0..4 {
        let details =
            json!({"got here": {"name": "somewhere", "scores": [i*10,(i+1)*10,(i+2)*10]}});
        assert_reachable!("Someplace we need to be", &details);
    }

    let details = json!({"impossible!": {"name": "trouble", "weights": [100,200,300]}});
    assert_unreachable!("Impossible to get here", &details);

    assert_always_greater_than!(3, 100, "not right");

    assert_sometimes_all!({a: true, b: false}, "not all right");
}

/// Entry point that initializes the Antithesis SDK and runs the assertion demo
fn main() {
    antithesis_init();
    assert_demo();
}