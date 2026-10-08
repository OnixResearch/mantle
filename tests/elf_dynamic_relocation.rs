#[path = "../src/elf_dynamic_fixup_core.rs"]
mod elf_dynamic_fixup_core;
#[path = "../src/elf_dynamic_fixup_shell.rs"]
mod elf_dynamic_fixup_shell;

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use elf_dynamic_fixup_core::DependencyRewrite;
use elf_dynamic_fixup_core::EntryKind;
use elf_dynamic_fixup_core::Refusal;
use elf_dynamic_fixup_core::apply_plan;
use elf_dynamic_fixup_core::plan_dynamic_relocation;

const HASH: &str = "0123456789abcdfghijklmnpqrsvwxyz";
const WRONG_HASH: &str = "1123456789abcdfghijklmnpqrsvwxyz";
const MEMBER: &str = "0123456789abcdfghijklmnpqrsvwxyz-native-dependency";

struct Fixture {
    root: PathBuf,
    first: PathBuf,
    binary: PathBuf,
    needed: String,
    runpath: String,
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn checked(command: &mut Command) {
    let output = command.output().expect("native fixture command");
    assert!(output.status.success(), "command failed: {}", String::from_utf8_lossy(&output.stderr));
}

fn fixture() -> Fixture {
    let root = std::env::temp_dir().join(format!(
        "mantle-elf-relocate-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    fs::create_dir(&root).unwrap();
    let first = root.join("prefix-one");
    let lib = first.join(MEMBER).join("lib");
    let bin = first.join("output").join("bin");
    fs::create_dir_all(&lib).unwrap();
    fs::create_dir_all(&bin).unwrap();
    let needed = format!("{}/libnative.so{}", lib.display(), "x".repeat(128));
    let padded_runpath = format!("{}:{}", lib.display(), "x".repeat(128));
    let libsource = root.join("lib.c");
    let mainsource = root.join("main.c");
    fs::write(&libsource, "int native_value(void) { return 42; }\n").unwrap();
    fs::write(&mainsource, "extern int native_value(void); int main(void) { return native_value() != 42; }\n").unwrap();
    checked(
        Command::new("gcc")
            .arg("-fPIC")
            .arg("-shared")
            .arg(format!("-Wl,-soname,{needed}"))
            .arg(&libsource)
            .arg("-o")
            .arg(lib.join("libnative.so").as_os_str()),
    );
    let binary = bin.join("native");
    checked(
        Command::new("gcc")
            .arg(&mainsource)
            .arg("-Wl,--enable-new-dtags")
            .arg(format!("-Wl,-rpath,{padded_runpath}"))
            .arg("-L")
            .arg(&lib)
            .arg("-lnative")
            .arg("-o")
            .arg(&binary),
    );
    let dynamic = Command::new("readelf").arg("-d").arg(&binary).output().unwrap();
    let report = String::from_utf8(dynamic.stdout).unwrap();
    let runpath = report
        .lines()
        .find_map(|line| line.split_once("Library runpath: ["))
        .and_then(|(_, value)| value.strip_suffix(']'))
        .unwrap()
        .to_owned();
    Fixture {
        root,
        first,
        binary,
        needed,
        runpath,
    }
}

fn rewrites<'a>(fixture: &'a Fixture, hash: &'a str) -> [DependencyRewrite<'a>; 2] {
    let member = if hash == HASH {
        MEMBER
    } else {
        "1123456789abcdfghijklmnpqrsvwxyz-native-dependency"
    };
    [
        DependencyRewrite {
            kind: EntryKind::Needed,
            current: &fixture.needed,
            dependency: member,
            library_path: "lib/libnative.so",
        },
        DependencyRewrite {
            kind: EntryKind::Runpath,
            current: &fixture.runpath,
            dependency: member,
            library_path: "lib",
        },
    ]
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|window| window == needle)
}

#[test]
fn actual_native_shared_binary_relocates_across_two_prefixes() {
    let fixture = fixture();
    let original = fs::read(&fixture.binary).unwrap();
    let needed = vec![fixture.needed.clone(), MEMBER.to_owned(), "lib/libnative.so".to_owned()];
    let runpath = vec![fixture.runpath.clone(), MEMBER.to_owned(), "lib".to_owned()];
    assert_eq!(elf_dynamic_fixup_shell::relocate_declared_file(&fixture.binary, 1, &needed, &runpath).unwrap(), 2);
    let rewritten = fs::read(&fixture.binary).unwrap();
    assert_eq!(original.len(), rewritten.len());
    assert!(contains(&original, HASH.as_bytes()) && contains(&rewritten, HASH.as_bytes()));
    assert!(!contains(&rewritten, fixture.first.to_string_lossy().as_bytes()));
    checked(&mut Command::new(&fixture.binary));
    let second = fixture.root.join("prefix-two");
    fs::create_dir(&second).unwrap();
    checked(Command::new("cp").arg("-a").arg(fixture.first.join(MEMBER)).arg(&second));
    checked(Command::new("cp").arg("-a").arg(fixture.first.join("output")).arg(&second));
    checked(&mut Command::new(second.join("output/bin/native")));
    let dynamic = Command::new("readelf").arg("-d").arg(second.join("output/bin/native")).output().unwrap();
    let output = String::from_utf8_lossy(&dynamic.stdout);
    assert!(output.contains("$ORIGIN/../../"));
    assert!(output.contains(MEMBER));
}

#[test]
fn native_no_soname_alias_reserves_needed_capacity_and_runs_after_copy() {
    use std::os::unix::fs::symlink;

    let fixture = fixture();
    let library = fixture.first.join(MEMBER).join("lib");
    let source = fixture.root.join("alias-lib.c");
    let program = fixture.root.join("alias-main.c");
    let real = library.join("libalias.so");
    let alias_name = format!("libalias.so.{}", "x".repeat(128));
    let alias = library.join(&alias_name);
    fs::write(&source, "int alias_value(void) { return 17; }\n").unwrap();
    fs::write(&program, "extern int alias_value(void); int main(void) { return alias_value() != 17; }\n").unwrap();
    checked(Command::new("gcc").arg("-shared").arg("-fPIC").arg(&source).arg("-o").arg(&real));
    symlink("libalias.so", &alias).unwrap();
    let binary = fixture.first.join("output/bin/alias-native");
    checked(
        Command::new("gcc")
            .arg(&program)
            .arg(&alias)
            .arg(format!("-Wl,-rpath,{}:{}", library.display(), "x".repeat(128)))
            .arg("-o")
            .arg(&binary),
    );
    let report = String::from_utf8(Command::new("readelf").arg("-d").arg(&binary).output().unwrap().stdout).unwrap();
    let needed = report
        .lines()
        .find_map(|line| line.split_once("Shared library: ["))
        .and_then(|(_, value)| value.strip_suffix(']'))
        .unwrap()
        .to_owned();
    assert_eq!(needed, alias.display().to_string());
    let runpath = report
        .lines()
        .find_map(|line| line.split_once("Library runpath: ["))
        .and_then(|(_, value)| value.strip_suffix(']'))
        .unwrap()
        .to_owned();
    let alias_member_path = format!("lib/{alias_name}");
    let declarations = [
        DependencyRewrite {
            kind: EntryKind::Needed,
            current: &needed,
            dependency: MEMBER,
            library_path: &alias_member_path,
        },
        DependencyRewrite {
            kind: EntryKind::Runpath,
            current: &runpath,
            dependency: MEMBER,
            library_path: "lib",
        },
    ];
    let old = fs::read(&binary).unwrap();
    assert_eq!(elf_dynamic_fixup_shell::relocate_dynamic_file(&binary, 1, &declarations).unwrap(), 2);
    let new = fs::read(&binary).unwrap();
    assert_eq!(new.len(), old.len());
    assert!(!contains(&new, fixture.first.to_string_lossy().as_bytes()));
    checked(&mut Command::new(&binary));
    let second = fixture.root.join("second-real-prefix");
    fs::create_dir(&second).unwrap();
    checked(Command::new("cp").arg("-a").arg(fixture.first.join(MEMBER)).arg(&second));
    checked(Command::new("cp").arg("-a").arg(fixture.first.join("output")).arg(&second));
    checked(&mut Command::new(second.join("output/bin/alias-native")));
}

#[test]
fn native_binary_refuses_capacity_mismatch_and_wrong_dependency_without_publication() {
    let fixture = fixture();
    let original = fs::read(&fixture.binary).unwrap();
    let mut wrong = rewrites(&fixture, HASH);
    wrong[0].current = "not-a-declared-needed";
    assert_eq!(plan_dynamic_relocation(&original, 1, &wrong).unwrap_err(), Refusal::DependencyMismatch);
    let mut overlong = rewrites(&fixture, HASH);
    let huge = "lib/".to_owned() + &"a".repeat(1024);
    overlong[0].library_path = &huge;
    assert_eq!(plan_dynamic_relocation(&original, 1, &overlong).unwrap_err(), Refusal::MissingCapacity);
    assert!(elf_dynamic_fixup_shell::relocate_dynamic_file(&fixture.binary, 1, &wrong).is_err());
    assert_eq!(fs::read(&fixture.binary).unwrap(), original);
    assert_ne!(WRONG_HASH, HASH);
    assert!(
        elf_dynamic_fixup_shell::relocate_declared_file(&fixture.binary, 1, std::slice::from_ref(&fixture.needed), &[])
            .is_err()
    );
    assert_eq!(fs::read(&fixture.binary).unwrap(), original);
    assert_eq!(
        plan_dynamic_relocation(&original, 1, &rewrites(&fixture, WRONG_HASH)).unwrap_err(),
        Refusal::DependencyMismatch
    );
}

#[test]
fn native_binary_refuses_unsupported_and_truncated_images() {
    let fixture = fixture();
    let original = fs::read(&fixture.binary).unwrap();
    let mapping = rewrites(&fixture, HASH);
    let mut class = original.clone();
    class[4] = 1;
    assert_eq!(plan_dynamic_relocation(&class, 1, &mapping).unwrap_err(), Refusal::UnsupportedClass);
    let mut endianness = original.clone();
    endianness[5] = 2;
    assert_eq!(plan_dynamic_relocation(&endianness, 1, &mapping).unwrap_err(), Refusal::UnsupportedEndianness);
    assert_eq!(plan_dynamic_relocation(&original[..40], 1, &mapping).unwrap_err(), Refusal::Truncated);
    let mut truncated = original.clone();
    truncated.truncate(original.len() / 2);
    assert!(matches!(plan_dynamic_relocation(&truncated, 1, &mapping), Err(Refusal::Truncated)));
    let plan = plan_dynamic_relocation(&original, 1, &mapping).unwrap();
    assert_eq!(apply_plan(&original, &plan).unwrap().len(), original.len());
    truncated = original.clone();
    truncated[plan.patches[0].offset] ^= 1;
    assert_eq!(apply_plan(&truncated, &plan).unwrap_err(), Refusal::InvalidLayout);
}

fn le(bytes: &[u8], at: usize, width: usize) -> usize {
    (0..width).fold(0, |n, i| n | (usize::from(bytes[at + i]) << (i * 8)))
}

fn symbol_tail_mutation(bytes: &mut [u8], target: usize) {
    let section_table = le(bytes, 40, 8);
    let section_count = le(bytes, 60, 2);
    let dynstr = (0..section_count)
        .find(|section| {
            let at = section_table + section * 64;
            le(bytes, at + 4, 4) == 6
        })
        .map(|section| le(bytes, section_table + section * 64 + 40, 4))
        .unwrap();
    let dynstr_start = le(bytes, section_table + dynstr * 64 + 24, 8);
    let dynsym = (0..section_count)
        .find(|section| {
            let at = section_table + section * 64;
            le(bytes, at + 4, 4) == 11 && le(bytes, at + 40, 4) == dynstr
        })
        .unwrap();
    let symtab = le(bytes, section_table + dynsym * 64 + 24, 8);
    let symtab_len = le(bytes, section_table + dynsym * 64 + 32, 8);
    assert!(symtab_len >= 48);
    bytes[symtab + 24..symtab + 28].copy_from_slice(&u32::try_from(target - dynstr_start + 2).unwrap().to_le_bytes());
}

#[test]
fn native_binary_refuses_symbol_name_inside_needed_string() {
    let fixture = fixture();
    let mut bytes = fs::read(&fixture.binary).unwrap();
    let mapping = rewrites(&fixture, HASH);
    let plan = plan_dynamic_relocation(&bytes, 1, &mapping).unwrap();
    symbol_tail_mutation(&mut bytes, plan.patches[0].offset);
    assert_eq!(plan_dynamic_relocation(&bytes, 1, &mapping).unwrap_err(), Refusal::SymbolTailOverlap);
}
