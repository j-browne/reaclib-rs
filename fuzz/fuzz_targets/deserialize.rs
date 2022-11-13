#![no_main]
use libfuzzer_sys::fuzz_target;
use reaclib::Set;

fuzz_target!(|data: &[u8]| {
    let _sets: Result<Vec<Set>, _> = serde_json::from_reader(data);
});
