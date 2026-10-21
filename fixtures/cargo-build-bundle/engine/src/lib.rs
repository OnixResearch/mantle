pub fn message() -> &'static str {
    if cfg!(feature = "fancy") {
        concat!("immutable engine: fancy:", env!("BUNDLE_NATIVE_VERSION"))
    } else {
        concat!("immutable engine: baseline:", env!("BUNDLE_NATIVE_VERSION"))
    }
}
