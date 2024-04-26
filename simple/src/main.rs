use once_cell::sync::Lazy;
use serde_json::{json, Value};

use antithesis_sdk_rust::prelude::*;

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

fn assert_demo() {

    // always
    let details = json!({"things": 13});
    assert_always!(true, "Things 777 look good", &details);

    // alwaysOrUnreachable
    let details = json!({"more things": "red and blue"});
    assert_always_or_unreachable!(true, "A few colors", &details);

    // sometimes
    let details = json!({"notes": [1,2,3,4,5]});
    assert_sometimes!(false, "Notes have small values", &details);


    // reachable
    for i in 0..4 {
        let details = json!({"got here": {"name": "somewhere", "scores": [i*10,(i+1)*10,(i+2)*10]}});
        assert_reachable!("Someplace we need to be", &details);
    }

    // ant_unreachable
    let details = json!({"impossible!": {"name": "trouble", "weights": [100,200,300]}});
    assert_unreachable!("Impossible to get here", &details);
}

pub fn main() {

    
    // antithesis_init();
    // antithesis_init();
    // antithesis_init();

    random_demo();

    lifecycle_demo();

    assert_demo();
}
