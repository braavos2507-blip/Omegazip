#![no_main]

// Декодер Huffman и analyze_bytes не должны паниковать на произвольном вводе.
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = omegazip::huffman::decode(data);
    let _ = omegazip::analyze_bytes(data, None);
});
