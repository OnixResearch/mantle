use std::path::Path;

fn main() {
    let root = std::env::var("NATIVE_ANSWER_ROOT").expect("declared native input root");
    let library = Path::new(&root).join("lib/libnative_answer.a");
    assert!(library.is_file(), "native library absent from declared input: {}", library.display());
    println!("cargo:rustc-link-search=native={}/lib", root);
    println!("cargo:rustc-link-lib=static=native_answer");
    println!("cargo:rerun-if-env-changed=NATIVE_ANSWER_ROOT");
}
