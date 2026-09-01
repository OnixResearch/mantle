#![allow(non_camel_case_types)]

type c_int = i32;
type c_long = i64;
type c_uint = u32;

unsafe extern "C" {
    fn ptrace(request: c_int, ...) -> c_long;
}

const PTRACE_SEIZE: c_uint = 0x4206;

pub fn seize_typecheck() -> c_long {
    unsafe { ptrace(PTRACE_SEIZE, 1_i32, core::ptr::null_mut::<u8>(), core::ptr::null_mut::<u8>()) }
}
