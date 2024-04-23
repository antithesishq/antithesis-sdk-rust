use crate::internal;

pub fn get_random() -> u64 {
    internal::dispatch_random()
}

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


#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};
    use super::*;

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
        // randiom_choice responses received matching that key
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
