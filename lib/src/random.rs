use crate::internal;
use rand::RngCore;

/// Returns a u64 value chosen by Antithesis.
///
/// You should use this value immediately rather than using it
/// later. If you delay, then it is possible for the simulation
/// to branch in between receiving the random data and using it.
/// These branches will have the same random value, which
/// defeats the purpose of branching.
///
/// Similarly, do not use the value to seed a pseudo-random
/// number generator. The PRNG will produce a deterministic
/// sequence of pseudo-random values based on the seed, so if the
/// simulation branches, the PRNG will use the same sequence of
/// values in all branches.
///
/// # Example
///
/// ```
/// use antithesis_sdk::random;
///
/// let value = random::get_random();
/// println!("Random value(u64): {value}");
/// ```
pub fn get_random() -> u64 {
    internal::dispatch_random()
}

/// Returns a randomly chosen item from a list of options.
///
/// You should use this value immediately rather than using it
/// later. If you delay, then it is possible for the simulation
/// to branch in between receiving the random data and using it.
/// These branches will have the same random value, which
/// defeats the purpose of branching.
///
/// Similarly, do not use the value to seed a pseudo-random
/// number generator. The PRNG will produce a deterministic
/// sequence of pseudo-random values based on the seed, so if the
/// simulation branches, the PRNG will use the same sequence of
/// values in all branches.
///
/// This function is not purely for convenience. Signaling to
/// the Antithesis platform that you intend to use a random value
/// in a structured way enables it to provide more interesting
/// choices over time.
///
/// # Example
///
/// ```
/// use antithesis_sdk::random;
///
/// let choices: Vec<&str> = vec!["abc", "def", "xyz", "qrs"];
/// if let Some(s) = random::random_choice(choices.as_slice()) {
///     println!("Choice: '{s}'");
/// };
/// ```
pub fn random_choice<T>(slice: &[T]) -> Option<&T> {
    match slice {
        [] => None,
        [x] => Some(x),
        _ => {
            let idx: usize = (get_random() as usize) % slice.len();
            Some(&slice[idx])
        }
    }
}

/// A random number generator that uses Antithesis's random number generation.
///
/// This implements the `RngCore` trait from the `rand` crate, allowing it to be used
/// with any code that expects a random number generator from that ecosystem.
///
/// # Example
///
/// ```
/// use antithesis_sdk::random::AntithesisRng;
/// use rand::{Rng, RngCore};
///
/// let mut rng = AntithesisRng;
/// let random_u32: u32 = rng.gen();
/// let random_u64: u64 = rng.gen();
/// let random_char: char = rng.gen();
///
/// let mut bytes = [0u8; 16];
/// rng.fill_bytes(&mut bytes);
/// ```
pub struct AntithesisRng;

impl RngCore for AntithesisRng {
    fn next_u32(&mut self) -> u32 {
        get_random() as u32
    }

    fn next_u64(&mut self) -> u64 {
        get_random()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        // Split the destination buffer into chunks of 8 bytes each
        // (since we'll fill each chunk with a u64/8 bytes of random data)
        let mut chunks = dest.chunks_exact_mut(8);

        // Fill each complete 8-byte chunk with random bytes
        for chunk in chunks.by_ref() {
            // Generate 8 random bytes from a u64 in native endian order
            let random_bytes = self.next_u64().to_ne_bytes();
            // Copy those random bytes into this chunk
            chunk.copy_from_slice(&random_bytes);
        }

        // Get any remaining bytes that didn't fit in a complete 8-byte chunk
        let remainder = chunks.into_remainder();

        if !remainder.is_empty() {
            // Generate 8 more random bytes
            let random_bytes = self.next_u64().to_ne_bytes();
            // Copy just enough random bytes to fill the remainder
            remainder.copy_from_slice(&random_bytes[..remainder.len()]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::prelude::IndexedRandom;
    use rand::Rng;
    use std::collections::{HashMap, HashSet};

    #[test]
    fn random_choice_no_choices() {
        let array = [""; 0];
        assert_eq!(0, array.len());
        assert_eq!(None, random_choice(&array))
    }

    #[test]
    fn random_choice_one_choice() {
        let array = ["ABc"; 1];
        assert_eq!(1, array.len());
        assert_eq!(Some(&"ABc"), random_choice(&array))
    }

    #[test]
    fn random_choice_few_choices() {
        // For each map key, the value is the count of the number of
        // random_choice responses received matching that key
        let mut counted_items: HashMap<&str, i64> = HashMap::new();
        counted_items.insert("a", 0);
        counted_items.insert("b", 0);
        counted_items.insert("c", 0);

        let all_keys: Vec<&str> = counted_items.keys().cloned().collect();
        assert_eq!(counted_items.len(), all_keys.len());
        for _i in 0..15 {
            let rc = random_choice(all_keys.as_slice());
            if let Some(choice) = rc {
                if let Some(x) = counted_items.get_mut(choice) {
                    *x += 1;
                }
            }
        }
        for (key, val) in counted_items.iter() {
            assert_ne!(*val, 0, "Did not produce the choice: {}", key);
        }
    }

    #[test]
    fn get_random_100k() {
        let mut random_numbers: HashSet<u64> = HashSet::new();
        for _i in 0..100000 {
            let rn = get_random();
            assert!(!random_numbers.contains(&rn));
            random_numbers.insert(rn);
        }
    }

    #[test]
    fn rng_no_choices() {
        let mut rng = AntithesisRng;
        let array = [""; 0];
        assert_eq!(0, array.len());
        assert_eq!(None, array.choose(&mut rng));
    }

    #[test]
    fn rng_one_choice() {
        let mut rng = AntithesisRng;
        let array = ["ABc"; 1];
        assert_eq!(1, array.len());
        assert_eq!(Some(&"ABc"), array.choose(&mut rng));
    }

    #[test]
    fn rng_few_choices() {
        let mut rng = AntithesisRng;
        // For each map key, the value is the count of the number of
        // random_choice responses received matching that key
        let mut counted_items: HashMap<&str, i64> = HashMap::new();
        counted_items.insert("a", 0);
        counted_items.insert("b", 0);
        counted_items.insert("c", 0);

        let all_keys: Vec<&str> = counted_items.keys().cloned().collect();
        assert_eq!(counted_items.len(), all_keys.len());
        for _i in 0..15 {
            let rc = all_keys.choose(&mut rng);
            if let Some(choice) = rc {
                if let Some(x) = counted_items.get_mut(choice) {
                    *x += 1;
                }
            }
        }
        for (key, val) in counted_items.iter() {
            assert_ne!(*val, 0, "Did not produce the choice: {}", key);
        }
    }

    #[test]
    fn rng_100k() {
        let mut rng = AntithesisRng;
        let mut random_numbers: HashSet<u64> = HashSet::new();
        for _i in 0..100000 {
            let rn: u64 = rng.random();
            assert!(!random_numbers.contains(&rn));
            random_numbers.insert(rn);
        }
    }
}
