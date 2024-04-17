use serde_json::{json, Value};
use antithesis_sdk_rust::{lifecycle, random};
use antithesis_sdk_rust::{always, always_or_unreachable, sometimes, reachable, unreachable};
use antithesis_sdk_rust::assert::{CatalogInfo};
use linkme::distributed_slice;

use antithesis_sdk_rust::assert_impl;

#[distributed_slice]
pub static ANTITHESIS_CATALOG: [CatalogInfo];

pub fn register_catalog() {
    let no_details: Value = json!({});
    for info in ANTITHESIS_CATALOG.iter() {
        println!("{} {} {} {} {}", info.display_type, info.message, info.class, info.file, info.begin_line);
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
fn random_demo() {
    // random::get_random()
    println!("fuzz_get_random() => {}", random::get_random());

    // random::random_choices()
    let choices: Vec<&str> = vec!("abc", "def", "xyz", "qrs");
    let nchoices = 10;
    print!("{nchoices} Choices: ");
    for n in 0..nchoices {
        let z = random::random_choice(choices.as_slice());
        if n > 0 {
            print!(" ,");
        }
        match z {
            Some(s) => print!("'{s}'"),
            None => print!("()")
        };
    }
    println!();
}

#[allow(dead_code)]
fn lifecycle_demo() {
    // lifecycle::setup_complete
    let bird_value: Value = json!({
        "name": "Tweety Bird",
        "age": 4,
        "phones": [
            "+1 9734970340"
        ]
    });
    let cat_value: Value = json!({
        "name": "Meow Cat",
        "age": 11,
        "phones": [
            "+1 2126581356",
            "+1 2126581384"
        ]
    });

    lifecycle::setup_complete(&bird_value);
    lifecycle::setup_complete(&cat_value);

    // lifecycle::send_event
    let info_value: Value = json!({
        "month": "January",
        "day": 32
    });
    lifecycle::send_event("user_info", &info_value);
}

#[allow(dead_code)]
fn slice_demo() {
    #[distributed_slice(ANTITHESIS_CATALOG)]
    static ALWAYS_001: antithesis_sdk_rust::assert::CatalogInfo = antithesis_sdk_rust::assert::CatalogInfo{
        assert_type: concat!("always"),
        display_type: concat!("Always"),
        condition: false,
        message: concat!("Things look good"),
        class: concat!(module_path!()),
        function: concat!("yes"),
        file: concat!(file!()),
        begin_line: line!(),
        begin_column: column!(), /* column */
        must_hit: true, /* must-hit */ 
        id: concat!("Things look good"), /* id */ 
    };

    #[distributed_slice(ANTITHESIS_CATALOG)]
    static SOMETIMES_001: antithesis_sdk_rust::assert::CatalogInfo = antithesis_sdk_rust::assert::CatalogInfo{
        assert_type: concat!("sometimes"),
        display_type: concat!("Sometimes"),
        condition: false,
        message: concat!("Notes have small values"),
        class: concat!(module_path!()),
        function: concat!("maybe"),
        file: concat!(file!()),
        begin_line: line!(),
        begin_column: column!(), /* column */
        must_hit: true, /* must-hit */ 
        id: concat!("Things look good"), /* id */ 
    };
}

fn assert_demo() {

    // catalog_entry!(
    //     assert_type = "always",
    //     display_type = "EachAndEvery"
    // );

    // always
    let details = json!({"things": 13});
    always!(true, "Things look good", &details, ALWAYS_23);

    // alwaysOrUnreachable
    let details = json!({"more things": "red and blue"});
    always_or_unreachable!(true, "A few colors", &details);

    // sometimes
    let details = json!({"notes": [1,2,3,4,5]});
    sometimes!(false, "Notes have small values", &details);


    // reachable
    for i in 0..4 {
        let details = json!({"got here": {"name": "somewhere", "scores": [i*10,(i+1)*10,(i+2)*10]}});
        reachable!("Someplace we need to be", &details);
    }

    // unreachable
    let details = json!({"impossible!": {"name": "trouble", "weights": [100,200,300]}});
    unreachable!("Impossible to get here", &details);
}

pub fn main() {

    register_catalog();

    // random_demo();

    // lifecycle_demo();

    // slice_demo();

    assert_demo();
}
