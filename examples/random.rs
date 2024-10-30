use rand::Rng;
use rand::seq::SliceRandom;
use antithesis_sdk::prelude::*;

fn random_demo() {
    const N_CHOICES: usize = 10;
    let mut rng = AntithesisRng;

    let fuzz_get_random: u64 = rng.gen();
    println!("fuzz_get_random() => {}", fuzz_get_random);

    let choices: Vec<_> = vec!["abc", "def", "xyz", "qrs"];
    print!("{N_CHOICES} Choices: ");


    let choices: Vec<_> = core::iter::repeat_with(|| *choices.choose(&mut rng).unwrap())
        .take(N_CHOICES)
        .collect();

    let choices = choices.join(", ");
    println!("{choices}");
}

fn main() {
    antithesis_init();
    random_demo();
}
