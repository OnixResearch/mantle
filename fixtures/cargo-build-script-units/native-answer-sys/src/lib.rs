include!(concat!(env!("OUT_DIR"), "/generated.rs"));

unsafe extern "C" {
    fn native_answer() -> i32;
    fn fixture_zero() -> i32;
}

pub fn answer() -> i32 {
    GENERATED_ZERO + unsafe { native_answer() + fixture_zero() }
}
