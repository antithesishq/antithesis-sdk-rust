use rand::{Error, RngCore};
use crate::internal;

/// Returns a u64 value chosen by Antithesis. You should not
/// store this value or use it to seed a PRNG, but should use it
/// immediately.
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

/// Returns a randomly chosen item from a list of options. You
/// should not store this value, but should use it immediately.
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

pub struct AntithesisRng;

impl RngCore for AntithesisRng {
    fn next_u32(&mut self) -> u32 {
        get_random() as u32
    }

    fn next_u64(&mut self) -> u64 {
        get_random()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        let mut chunks = dest.chunks_exact_mut(8);

        for chunk in chunks.by_ref() {
            let random_bytes = self.next_u64().to_le_bytes();
            chunk.copy_from_slice(&random_bytes);
        }

        let remainder = chunks.into_remainder();

        if !remainder.is_empty() {
            let random_bytes = self.next_u64().to_le_bytes();
            remainder.copy_from_slice(&random_bytes[..remainder.len()]);
        }
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
}
