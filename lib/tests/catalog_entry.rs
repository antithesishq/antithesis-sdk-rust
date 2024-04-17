#[cfg(test)]
use serde_json::{json, Value};
// use antithesis_sdk_rust::{always, always_or_unreachable, sometimes, reachable, unreachable};
use antithesis_sdk_rust::assert_impl;
use assertion_catalog::{another_entry};

use linkme::distributed_slice;
use antithesis_sdk_rust::assert::CatalogInfo;

#[distributed_slice]
pub static ANTITHESIS_CATALOG: [CatalogInfo];

// TODO: This work to be done before main()
pub fn register_catalog() {
    let no_details: Value = json!({});
    for info in ANTITHESIS_CATALOG.iter() {
        assert_impl(
            info.assert_type,
            info.display_type,
            info.condition,
            info.message,
            info.class,
            info.function,
            info.file,
            info.begin_line,
            info.begin_column,
            false, /* hit */
            info.must_hit,
            info.id,
            &no_details
        );
    }
}

#[allow(dead_code)]
struct CatEntry {
    assert_type: String,
    display_type: String,
    condition: bool,
    message: String,
    class: String,
    function: String,
    file: String,
    begin_line: u32,
    begin_column: u32,
    must_hit: bool,
    id: String,
}

#[test]
fn another_catalog() {

    // TODO: This should be run automatic before main()
    register_catalog();

    another_entry!({
        assert_type: "sometimes",
        display_type: "AtLeastOnce",
        condition: false,
        message: "Listen for incoming client requests",
        class: module_path!(),
        function: "yes",
        file: file!(),
        begin_line: line!(),
        begin_column: column!(),
        must_hit: true,
        id: "Listen for incoming client requests",
    });
    assert!(true);
}
