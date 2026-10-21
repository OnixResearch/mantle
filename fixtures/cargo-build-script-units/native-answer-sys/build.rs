use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

fn run(tool: &str, arguments: &[&std::ffi::OsStr]) {
    let status = Command::new(tool).args(arguments).status().expect("declared native tool executes");
    assert!(status.success(), "declared native tool failed: {tool}: {status}");
}

fn main() {
    let root = std::env::var("NATIVE_ANSWER_ROOT").expect("declared native input root");
    let cc = std::env::var("CC").expect("declared C compiler role");
    let ar = std::env::var("AR").expect("declared archiver role");
    let out = PathBuf::from(std::env::var_os("OUT_DIR").expect("declared OUT_DIR"));
    let library = Path::new(&root).join("lib/libnative_answer.a");
    assert!(library.is_file(), "native library absent from declared input: {}", library.display());
    assert!(!std::env::var("HOST").unwrap().is_empty());
    assert!(!std::env::var("TARGET").unwrap().is_empty());
    assert!(!std::env::var("CARGO_CFG_TARGET_ARCH").unwrap().is_empty());
    let object = out.join("fixture.o");
    let archive = out.join("libfixture_zero.a");
    run(&cc, &[
        "-c".as_ref(),
        "native-zero.c".as_ref(),
        "-o".as_ref(),
        object.as_os_str(),
    ]);
    run(&ar, &["rcs".as_ref(), archive.as_os_str(), object.as_os_str()]);
    std::fs::write(out.join("generated.rs"), "const GENERATED_ZERO: i32 = 0;\n").unwrap();
    println!("cargo::rustc-link-search=native={}", out.display());
    println!("cargo::rustc-link-lib=static=fixture_zero");
    println!("cargo::rustc-link-search=native={root}/lib");
    println!("cargo::rustc-link-lib=static=native_answer");
    println!("cargo::metadata=generated=42");
    println!("cargo::rerun-if-changed=native-zero.c");
    println!("cargo::rerun-if-env-changed=NATIVE_ANSWER_ROOT");
}
