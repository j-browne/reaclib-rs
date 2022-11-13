#![no_main]
use libfuzzer_sys::fuzz_target;
use reaclib::Set;

fuzz_target!(|input: Set| {
    let _s = serde_json::to_string(&input).unwrap();
    let _s = serde_json::to_string_pretty(&input).unwrap();
});
