use core::ffi::c_void;

/// JNI primitive-only signature deliberately needs no downloaded JNI crate.
/// Device execution is a separate proof from building or inspecting this symbol.
#[unsafe(no_mangle)]
pub extern "system" fn Java_org_example_nativeapp_MainActivity_value(
    _env: *mut c_void,
    _object: *mut c_void,
) -> i32 {
    native_value::answer!()
}
