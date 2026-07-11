use antithesis_sdk::{antithesis_init, ghost::GhostState, observe, LOCAL_OUTPUT};
use serde_json::json;

mod common;
use common::{AntithesisAssert, AssertType, SDKInput};

// Exercises the `ghost` layer end-to-end: build ghost state, mutate it, and
// observe it — asserting from inside the `observe!` block. Then verify the
// assertion was actually emitted to the local output with the expected message
// and details.
#[test]
fn ghost_mutate_and_observe() {
    let output_file = "/tmp/antithesis-ghost-observe.json";
    let prev_v = common::env::set_var(LOCAL_OUTPUT, output_file);
    antithesis_init();

    // A tiny reference model: how many events we have seen.
    let mut seen = GhostState::new(|| 0i64);
    for _ in 0..3 {
        seen.mutate(|n: &mut i64| {
            *n += 1;
        });
    }

    // Read the ghost state back through the sole read path and assert a property.
    observe!(seen, |n: &i64| {
        assert!(*n == 3, "ghost state should have counted 3 events");
        let details = json!({ "seen": *n });
        antithesis_sdk::assert_always!(*n >= 0, "Seen count is never negative", &details);
    });

    // The `observe!` block above should have emitted the assertion.
    match common::read_jsonl_tags(output_file) {
        Ok(entries) => {
            let mut did_register = false;
            let mut did_hit = false;
            for obj in entries.iter() {
                if let SDKInput::AntithesisAssert(AntithesisAssert {
                    assert_type,
                    condition,
                    hit,
                    must_hit,
                    id,
                    message,
                    details,
                    ..
                }) = obj
                {
                    if message != "Seen count is never negative" {
                        continue;
                    }
                    if *hit {
                        did_hit = true;
                        assert!(*condition);
                        assert_eq!(details, &json!({ "seen": 3 }));
                    } else {
                        did_register = true;
                    }
                    assert_eq!(*assert_type, AssertType::Always);
                    assert!(*must_hit);
                    assert_eq!(id, message);
                }
            }
            assert!(did_register, "assertion catalog entry was not registered");
            assert!(did_hit, "assertion was not emitted from inside observe!");
        }
        Err(e) => panic!("failed to read local output: {}", e),
    }

    common::env::restore_var(LOCAL_OUTPUT, prev_v);
}

// Ghost state with several inner types can be observed together, and derived
// traits pass through when the inner type supports them.
#[test]
fn ghost_multi_and_derives() {
    let a = GhostState::new(|| 10u64);
    let b = GhostState::new(|| String::from("hello"));

    observe!(a, b, |x: &u64, y: &String| {
        assert!(*x == 10);
        assert!(y == "hello");
    });

    // derived traits available because the inner types implement them
    let b2 = b.clone(); // Clone (String inner is not Copy)
    assert_eq!(b, b2);
    let a2 = a; // Copy (u64 inner)
    assert_eq!(a, a2);
    let d: GhostState<u64> = Default::default();
    assert_eq!(format!("{:?}", d), "GhostState(0)");
}
