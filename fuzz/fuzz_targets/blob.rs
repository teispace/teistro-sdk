#![no_main]

libfuzzer_sys::fuzz_target!(|data: &[u8]| teistro_fuzz::blob(data));
