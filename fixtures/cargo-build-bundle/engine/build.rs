fn main() {
    let file = std::env::var("BUNDLE_NATIVE_VERSION_FILE").expect("declare the native configuration role");
    let version = std::fs::read_to_string(file).expect("read the declared native input");
    let version = version.trim();
    assert!(!version.is_empty() && !version.contains(['\r', '\n']), "invalid native version");
    println!("cargo:rustc-env=BUNDLE_NATIVE_VERSION={version}");
}
