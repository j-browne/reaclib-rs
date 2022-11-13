#![no_main]
use libfuzzer_sys::fuzz_target;
use reaclib::{to_hash_map, Format, Iter};

fuzz_target!(|data: &[u8]| {
    let iter_v1 = Iter::new(data, Format::Reaclib1);
    let iter_v2 = Iter::new(data, Format::Reaclib2);

    for set in Iterator::chain(iter_v1, iter_v2) {
        if let Ok(set) = set {
            let _rate = set.rate(1.0);
        }
    }

    let _hash_map_1 = to_hash_map(data, Format::Reaclib1);
    let _hash_map_1 = to_hash_map(data, Format::Reaclib2);
});
