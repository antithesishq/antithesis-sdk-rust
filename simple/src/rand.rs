use antithesis_sdk::random::AntithesisRng;

mod rng_quality_tests {
    use std::collections::{HashMap, HashSet};

    use super::*;
    use rand_v0_8::seq::SliceRandom;
    use rand_v0_8::Rng;

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
        for _i in 0..30 {
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
            let rn: u64 = rng.gen();
            assert!(!random_numbers.contains(&rn));
            random_numbers.insert(rn);
        }
    }
}

mod rand_v0_8_tests {
    use super::*;
    use rand_v0_8::seq::{IteratorRandom, SliceRandom};
    use rand_v0_8::{Rng, RngCore};

    #[test]
    fn next_u64() {
        let mut rng = AntithesisRng;
        let a = rng.next_u64();
        let b = rng.next_u64();
        assert_ne!(a, b);
    }

    #[test]
    fn gen_f64() {
        let mut rng = AntithesisRng;
        let v: f64 = rng.gen();
        assert!((0.0..1.0).contains(&v));
    }

    #[test]
    fn gen_range() {
        let mut rng = AntithesisRng;
        let v: u32 = rng.gen_range(0..100);
        assert!(v < 100);
    }

    #[test]
    fn fill_bytes() {
        let mut rng = AntithesisRng;
        let mut buf = [0u8; 11];
        rng.fill_bytes(&mut buf);
        assert!(buf.iter().any(|&b| b != 0));
    }

    #[test]
    fn shuffle() {
        let mut rng = AntithesisRng;
        let mut arr = [1, 2, 3, 4, 5];
        arr.shuffle(&mut rng);
        arr.sort();
        assert_eq!(arr, [1, 2, 3, 4, 5]);
    }

    #[test]
    fn iterator_choose() {
        let mut rng = AntithesisRng;
        let val = (0..10).choose(&mut rng);
        assert!(val.is_some());
        assert!((0..10).contains(&val.unwrap()));
    }
}

mod rand_v0_9_tests {
    use super::*;
    use rand_v0_9::seq::{IteratorRandom, SliceRandom};
    use rand_v0_9::{Rng, RngCore};

    #[test]
    fn next_u64() {
        let mut rng = AntithesisRng;
        let a = rng.next_u64();
        let b = rng.next_u64();
        assert_ne!(a, b);
    }

    #[test]
    fn random_f64() {
        let mut rng = AntithesisRng;
        let v: f64 = rng.random();
        assert!((0.0..1.0).contains(&v));
    }

    #[test]
    fn random_range() {
        let mut rng = AntithesisRng;
        let v: u32 = rng.random_range(0..100);
        assert!(v < 100);
    }

    #[test]
    fn fill_bytes() {
        let mut rng = AntithesisRng;
        let mut buf = [0u8; 11];
        rng.fill_bytes(&mut buf);
        assert!(buf.iter().any(|&b| b != 0));
    }

    #[test]
    fn shuffle() {
        let mut rng = AntithesisRng;
        let mut arr = [1, 2, 3, 4, 5];
        arr.shuffle(&mut rng);
        arr.sort();
        assert_eq!(arr, [1, 2, 3, 4, 5]);
    }

    #[test]
    fn iterator_choose() {
        let mut rng = AntithesisRng;
        let val = (0..10).choose(&mut rng);
        assert!(val.is_some());
        assert!((0..10).contains(&val.unwrap()));
    }
}

mod rand_v0_10_tests {
    use super::*;
    use rand_v0_10::seq::{IteratorRandom, SliceRandom};
    use rand_v0_10::{Rng, RngExt};

    #[test]
    fn next_u64() {
        let mut rng = AntithesisRng;
        let a = rng.next_u64();
        let b = rng.next_u64();
        assert_ne!(a, b);
    }

    #[test]
    fn random_f64() {
        let mut rng = AntithesisRng;
        let v: f64 = rng.random();
        assert!((0.0..1.0).contains(&v));
    }

    #[test]
    fn random_range() {
        let mut rng = AntithesisRng;
        let v: u32 = rng.random_range(0..100);
        assert!(v < 100);
    }

    #[test]
    fn fill_bytes() {
        let mut rng = AntithesisRng;
        let mut buf = [0u8; 11];
        rng.fill_bytes(&mut buf);
        assert!(buf.iter().any(|&b| b != 0));
    }

    #[test]
    fn shuffle() {
        let mut rng = AntithesisRng;
        let mut arr = [1, 2, 3, 4, 5];
        arr.shuffle(&mut rng);
        arr.sort();
        assert_eq!(arr, [1, 2, 3, 4, 5]);
    }

    #[test]
    fn iterator_choose() {
        let mut rng = AntithesisRng;
        let val = (0..10).choose(&mut rng);
        assert!(val.is_some());
        assert!((0..10).contains(&val.unwrap()));
    }
}
