use std::env;
use std::ffi::OsString;
use std::fs;
use std::os::unix::ffi::OsStringExt;
use std::path::PathBuf;
use std::process::Command;

fn compiler() -> PathBuf {
    let path = env::var_os("PATH").expect("PATH is required for the compiler smoke");
    env::split_paths(&path)
        .map(|directory| directory.join("cc"))
        .find(|path| path.is_file())
        .expect("real C compiler must be installed")
        .canonicalize()
        .unwrap()
}

fn remove_ambient_inputs(command: &mut Command) -> &mut Command {
    for name in [
        "CPATH",
        "C_INCLUDE_PATH",
        "CPLUS_INCLUDE_PATH",
        "OBJC_INCLUDE_PATH",
        "GCC_EXEC_PREFIX",
        "COMPILER_PATH",
        "SOURCE_DATE_EPOCH",
    ] {
        command.env_remove(name);
    }
    command
}

#[test]
fn binary_preserves_non_utf8_filename_for_real_file_read() {
    let temp = tempfile::tempdir().unwrap();
    let filename = OsString::from_vec(vec![b'i', b'n', b'p', b'u', b't', 0xff]);
    let input = temp.path().join(filename);
    let content = b"raw filename reached the external reader\n";
    fs::write(&input, content).unwrap();
    let receipt_path = temp.path().join("receipt.json");
    let output = Command::new(env!("CARGO_BIN_EXE_mantle-cc-cache-driver"))
        .arg("--compiler")
        .arg("/bin/cat")
        .arg("--socket")
        .arg(temp.path().join("missing.sock"))
        .arg("--receipt")
        .arg(&receipt_path)
        .arg("--platform-digest")
        .arg("a".repeat(64))
        .arg("--")
        .arg(input)
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(output.stdout, content);
    let receipt: serde_json::Value = serde_json::from_slice(&fs::read(receipt_path).unwrap()).unwrap();
    assert_eq!(receipt["disposition"], "fallback");
    assert_eq!(receipt["reason"], "non-utf8-compiler-argument");
    assert_eq!(receipt["compiler_exit_code"], 0);
}

#[test]
fn unavailable_socket_still_builds_identical_real_c_object_and_records_fallback() {
    let temp = tempfile::tempdir().unwrap();
    let source_root = temp.path().join("source");
    let artifacts = temp.path().join("artifacts");
    fs::create_dir(&source_root).unwrap();
    fs::create_dir(&artifacts).unwrap();
    let source = source_root.join("example.c");
    let header = source_root.join("number.h");
    fs::write(&source, "#include \"number.h\"\nint answer(void) { return NUMBER; }\n").unwrap();
    fs::write(&header, "#define NUMBER 42\n").unwrap();
    let mapped_output = artifacts.join("mapped.o");
    let direct_output = artifacts.join("direct.o");
    let mapped_depfile = artifacts.join("mapped.d");
    let direct_depfile = artifacts.join("direct.d");
    let mapping = format!("-ffile-prefix-map={}=/cc-root-0", source_root.display());
    let cc = compiler();
    let receipt_path = artifacts.join("receipt.json");
    let driver = remove_ambient_inputs(
        Command::new(env!("CARGO_BIN_EXE_mantle-cc-cache-driver"))
            .arg("--compiler")
            .arg(&cc)
            .arg("--socket")
            .arg(temp.path().join("missing.sock"))
            .arg("--receipt")
            .arg(&receipt_path)
            .arg("--platform-digest")
            .arg("a".repeat(64))
            .arg("--")
            .arg("-c")
            .arg(&source)
            .arg("-o")
            .arg(&mapped_output)
            .arg("-MMD")
            .arg("-MF")
            .arg(&mapped_depfile)
            .arg("-nostdinc")
            .arg("-w")
            .arg(&mapping),
    )
    .output()
    .unwrap();
    assert!(driver.status.success(), "{}", String::from_utf8_lossy(&driver.stderr));
    let direct = remove_ambient_inputs(
        Command::new(&cc)
            .arg("-c")
            .arg(&source)
            .arg("-o")
            .arg(&direct_output)
            .arg("-MMD")
            .arg("-MF")
            .arg(&direct_depfile)
            .arg("-nostdinc")
            .arg("-w")
            .arg(&mapping),
    )
    .output()
    .unwrap();
    assert!(direct.status.success(), "{}", String::from_utf8_lossy(&direct.stderr));
    assert_eq!(fs::read(&mapped_output).unwrap(), fs::read(&direct_output).unwrap());
    let dependencies = fs::read_to_string(&mapped_depfile).unwrap();
    assert!(dependencies.contains(source.to_str().unwrap()));
    assert!(dependencies.contains(header.to_str().unwrap()));
    let receipt: serde_json::Value = serde_json::from_slice(&fs::read(receipt_path).unwrap()).unwrap();
    assert_eq!(receipt["disposition"], "fallback");
    assert_eq!(receipt["reason"], "cache-unavailable");
    assert_eq!(receipt["compiler_exit_code"], 0);
}

#[test]
fn relative_artifacts_run_real_gcc_without_cache_admission() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source.c");
    fs::write(&source, "int answer(void) { return 42; }\n").unwrap();
    let cc = compiler();
    let map = format!("-ffile-prefix-map={}=/cc-root-0", temp.path().display());
    for (object, depfile) in [
        (PathBuf::from("relative.o"), temp.path().join("absolute.d")),
        (temp.path().join("absolute.o"), PathBuf::from("relative.d")),
    ] {
        let receipt_path = temp.path().join("receipt.json");
        let output = remove_ambient_inputs(
            Command::new(env!("CARGO_BIN_EXE_mantle-cc-cache-driver"))
                .current_dir(temp.path())
                .arg("--compiler")
                .arg(&cc)
                .arg("--socket")
                .arg(temp.path().join("missing.sock"))
                .arg("--receipt")
                .arg(&receipt_path)
                .arg("--platform-digest")
                .arg("a".repeat(64))
                .arg("--")
                .arg("-c")
                .arg(&source)
                .arg("-o")
                .arg(&object)
                .arg("-MMD")
                .arg("-MF")
                .arg(&depfile)
                .arg("-nostdinc")
                .arg("-w")
                .arg(&map),
        )
        .output()
        .unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        let actual_object = if object.is_absolute() {
            object
        } else {
            temp.path().join(object)
        };
        let actual_depfile = if depfile.is_absolute() {
            depfile
        } else {
            temp.path().join(depfile)
        };
        assert!(actual_object.is_file());
        assert!(actual_depfile.is_file());
        let receipt: serde_json::Value = serde_json::from_slice(&fs::read(receipt_path).unwrap()).unwrap();
        assert_eq!(receipt["disposition"], "fallback");
        assert_eq!(receipt["reason"], "unclassified-invocation");
    }
}
