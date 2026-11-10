#![cfg(feature = "full")]

use antithesis_sdk::catalog::{self, Assertion, AssertionKind};
use antithesis_sdk::{
    antithesis_init, assert_always, assert_always_greater_than, assert_always_or_unreachable,
    assert_reachable, assert_sometimes, assert_unreachable, LOCAL_OUTPUT,
};
use serde_json::{json, Value};
use std::fs;

mod common;

// Never called: the catalog is a link-time property of the binary, which is
// exactly what these tests exercise.
#[allow(dead_code)]
fn declarations(x: u64) {
    let details = json!({});
    assert_always!(x > 0, "catalog-test: always holds", &details);
    assert_always_or_unreachable!(x > 1, "catalog-test: guard holds", &details);
    assert_sometimes!(x > 2, "catalog-test: sometimes fires", &details);
    assert_reachable!("catalog-test: reached", &details);
    assert_unreachable!("catalog-test: never reached", &details);
    assert_always_greater_than!(x, 10, "catalog-test: numeric always", &details);
}

fn assertion(message: &str) -> Assertion {
    match catalog::assertions().find(|e| e.message == message) {
        Some(e) => e,
        None => panic!("assertion {:?} missing from catalog", message),
    }
}

#[test]
fn assertions_enumerate_declarations_without_execution() {
    assert!(matches!(
        assertion("catalog-test: always holds").kind,
        AssertionKind::Always
    ));
    assert!(matches!(
        assertion("catalog-test: guard holds").kind,
        AssertionKind::AlwaysOrUnreachable
    ));
    assert!(matches!(
        assertion("catalog-test: sometimes fires").kind,
        AssertionKind::Sometimes
    ));
    assert!(matches!(
        assertion("catalog-test: reached").kind,
        AssertionKind::Reachable
    ));
    assert!(matches!(
        assertion("catalog-test: never reached").kind,
        AssertionKind::Unreachable
    ));
    // The numeric family reports as an ordinary Always assertion.
    assert!(matches!(
        assertion("catalog-test: numeric always").kind,
        AssertionKind::Always
    ));

    let e = assertion("catalog-test: sometimes fires");
    assert_eq!(e.id, e.message);
    assert!(
        e.location.file.ends_with("catalog.rs"),
        "unexpected file {:?}",
        e.location.file
    );
    assert!(e.location.begin_line > 0);
}

#[test]
fn write_matches_assertions() {
    let mut buf: Vec<u8> = Vec::new();
    catalog::write(&mut buf).unwrap();
    let text = String::from_utf8(buf.clone()).unwrap();
    let lines: Vec<Value> = text
        .lines()
        .map(|l| serde_json::from_str(l).expect("valid JSON line"))
        .collect();

    // Every line is an assertion catalog registration; guidance is init's
    // business and stays out of write().
    let assertions: Vec<&Value> = lines
        .iter()
        .filter_map(|v| v.get("antithesis_assert"))
        .collect();
    assert_eq!(assertions.len(), lines.len());
    assert_eq!(assertions.len(), catalog::assertions().count());
    for a in &assertions {
        assert_eq!(a["hit"], Value::Bool(false));
    }

    // Same order as assertions(): the linker's, whatever it is.
    let written_messages: Vec<&str> = assertions
        .iter()
        .map(|a| a["message"].as_str().unwrap())
        .collect();
    let listed_messages: Vec<&str> = catalog::assertions().map(|a| a.message).collect();
    assert_eq!(written_messages, listed_messages);

    // Byte-for-byte deterministic within one binary.
    let mut again: Vec<u8> = Vec::new();
    catalog::write(&mut again).unwrap();
    assert_eq!(buf, again);
}

// The accessor is a second copy of the strings the macros pass as
// `display_type`; the serialized catalog is the source of truth for both.
#[test]
fn kind_display_type_matches_serialized_events() {
    let mut buf: Vec<u8> = Vec::new();
    catalog::write(&mut buf).unwrap();
    let text = String::from_utf8(buf).unwrap();
    let serialized: Vec<Value> = text
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .filter_map(|v: Value| v.get("antithesis_assert").cloned())
        .collect();
    assert!(!serialized.is_empty());
    for a in &serialized {
        let e = assertion(a["message"].as_str().unwrap());
        assert_eq!(
            a["display_type"],
            e.kind.display_type(),
            "for {:?}",
            e.message
        );
    }
    // Every kind is exercised by the declarations above.
    let kinds: std::collections::HashSet<AssertionKind> =
        catalog::assertions().map(|e| e.kind).collect();
    assert_eq!(kinds.len(), 5);
}

// `write()` and `antithesis_init()` consume the same assertion sequence, so
// init's output is the sdk-info header, then exactly write()'s lines in
// order, then the guidance registrations.
#[test]
fn write_matches_init_emission() {
    let output_file = "/tmp/antithesis-sdk-catalog-parity.jsonl";
    let _ = fs::remove_file(output_file);
    let prev_v = common::env::set_var(LOCAL_OUTPUT, output_file);
    antithesis_init();
    let from_init = fs::read_to_string(output_file).expect("init wrote local output");
    common::env::restore_var(LOCAL_OUTPUT, prev_v);

    let mut init_lines = from_init.lines();
    assert!(init_lines.next().unwrap().contains("antithesis_sdk"));

    let mut buf: Vec<u8> = Vec::new();
    catalog::write(&mut buf).unwrap();
    let text = String::from_utf8(buf).unwrap();
    let written: Vec<&str> = text.lines().collect();

    let from_init_assertions: Vec<&str> = init_lines.by_ref().take(written.len()).collect();
    assert_eq!(from_init_assertions, written);

    // What remains is the guidance the numeric macro also registers.
    let guidance: Vec<Value> = init_lines
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert!(!guidance.is_empty());
    assert!(guidance
        .iter()
        .all(|g| g.get("antithesis_guidance").is_some()));
    assert!(guidance
        .iter()
        .any(|g| g["antithesis_guidance"]["message"] == "catalog-test: numeric always"));
}
