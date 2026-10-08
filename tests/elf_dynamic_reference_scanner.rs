#[path = "../src/elf_dynamic_fixup_core.rs"]
mod elf_dynamic_fixup_core;

use std::fs;
use std::process::Command;

use elf_dynamic_fixup_core::DependencyRewrite;
use elf_dynamic_fixup_core::EntryKind;
use elf_dynamic_fixup_core::Refusal;
use elf_dynamic_fixup_core::apply_plan;
use elf_dynamic_fixup_core::plan_dynamic_relocation;
use snix_castore::refscan::ReferenceScanner;

const HASH: &str = "0123456789abcdfghijklmnpqrsvwxyz";
const WRONG: &str = "1123456789abcdfghijklmnpqrsvwxyz";
const MEMBER: &str = "0123456789abcdfghijklmnpqrsvwxyz-native-dependency";

fn run(command: &mut Command) {
    let result = command.output().expect("native compiler/readelf availability");
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
}

fn scanner_matches(bytes: &[u8]) -> Vec<String> {
    let scanner = ReferenceScanner::new(vec![HASH.to_owned(), WRONG.to_owned()]);
    scanner.scan(bytes);
    scanner.finalise().into_iter().collect()
}

#[test]
fn actual_snix_scanner_keeps_exact_references_after_native_elf_fixup() {
    let tmp = tempfile::tempdir().unwrap();
    let library = tmp.path().join("store").join(MEMBER).join("lib");
    fs::create_dir_all(&library).unwrap();
    let source = tmp.path().join("lib.c");
    let program = tmp.path().join("main.c");
    let binary = tmp.path().join("store/output/bin/native");
    fs::create_dir_all(binary.parent().unwrap()).unwrap();
    fs::write(&source, "int native_value(void) { return 42; }\n").unwrap();
    fs::write(&program, "extern int native_value(void); int main(void) { return native_value() != 42; }\n").unwrap();
    let needed = format!("{}/libnative.so{}", library.display(), "x".repeat(128));
    let padded = format!("{}:{}", library.display(), "x".repeat(128));
    run(Command::new("gcc")
        .arg("-fPIC")
        .arg("-shared")
        .arg(format!("-Wl,-soname,{needed}"))
        .arg(&source)
        .arg("-o")
        .arg(library.join("libnative.so")));
    run(Command::new("gcc")
        .arg(&program)
        .arg("-Wl,--enable-new-dtags")
        .arg(format!("-Wl,-rpath,{padded}"))
        .arg("-L")
        .arg(&library)
        .arg("-lnative")
        .arg("-o")
        .arg(&binary));
    let dynamic = Command::new("readelf").arg("-d").arg(&binary).output().unwrap();
    assert!(dynamic.status.success());
    let report = String::from_utf8(dynamic.stdout).unwrap();
    let runpath = report
        .lines()
        .find_map(|line| line.split_once("Library runpath: ["))
        .and_then(|(_, value)| value.strip_suffix(']'))
        .unwrap()
        .to_owned();
    let original = fs::read(&binary).unwrap();
    let mapping = [
        DependencyRewrite {
            kind: EntryKind::Needed,
            current: &needed,
            dependency: MEMBER,
            library_path: "lib/libnative.so",
        },
        DependencyRewrite {
            kind: EntryKind::Runpath,
            current: &runpath,
            dependency: MEMBER,
            library_path: "lib",
        },
    ];
    let plan = plan_dynamic_relocation(&original, 1, &mapping).unwrap();
    let transformed = apply_plan(&original, &plan).unwrap();
    assert_eq!(scanner_matches(&original), vec![HASH.to_owned()]);
    assert_eq!(scanner_matches(&transformed), scanner_matches(&original));
    assert_eq!(transformed.len(), original.len());
    assert_eq!(
        plan_dynamic_relocation(&original, 1, &[DependencyRewrite {
            kind: EntryKind::Needed,
            current: &needed,
            dependency: "1123456789abcdfghijklmnpqrsvwxyz-native-dependency",
            library_path: "lib/libnative.so"
        },])
        .unwrap_err(),
        Refusal::DependencyMismatch
    );
}
