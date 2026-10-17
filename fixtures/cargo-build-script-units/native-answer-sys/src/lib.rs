unsafe extern "C" {
    fn native_answer() -> i32;
}

pub fn answer() -> i32 {
    unsafe { native_answer() }
}
