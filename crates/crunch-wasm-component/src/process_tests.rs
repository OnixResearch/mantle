use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use crunch_wasm_component_core::Blake3Identity;

use crate::ToolInvocation;
use crate::ToolLimits;
use crate::ToolRecord;
use crate::ToolchainManifest;
use crate::VerifiedToolchain;
use crate::run_offline_tool;

const TEST_TIMEOUT_MS: u64 = 250;
const TEST_OUTPUT_BYTES: u64 = 1024;
const OVERSIZED_ARGUMENT_BYTES: usize = 16 * 1024 + 1;
const OVERSIZED_ENV_VALUE_BYTES: usize = 64 * 1024 + 1;

#[test]
fn offline_tool_emits_bound_success_receipt() {
    let fixture = Fixture::new(&find_program("echo"));
    let run = run_offline_tool(&fixture.toolchain, fixture.invocation(vec!["receipt-ok".to_string()]))
        .expect("bounded echo invocation succeeds");

    assert!(run.success, "{run:#?}");
    assert_eq!(run.stdout, b"receipt-ok\n");
    assert_eq!(run.receipt.status, "succeeded");
    assert_eq!(run.receipt.stdout_blake3, Blake3Identity::from_slice(b"receipt-ok\n"));
    assert!(!fixture.root.path().join(".mantle-tool-logs").exists());
}

#[test]
fn offline_tool_terminates_on_output_limit_and_keeps_bounded_receipt() {
    let fixture = Fixture::new(&find_program("yes"));
    let mut invocation = fixture.invocation(Vec::new());
    invocation.limits.output_bytes = TEST_OUTPUT_BYTES;
    let run = run_offline_tool(&fixture.toolchain, invocation).expect("output limit produces a failed receipt");

    assert!(!run.success);
    assert_eq!(run.receipt.status, "output-limit-exceeded", "{run:#?}");
    assert!(run.stdout.len() <= usize::try_from(TEST_OUTPUT_BYTES).unwrap());
    assert!(String::from_utf8_lossy(&run.stderr).contains("output-limit-exceeded"));
}

#[test]
fn offline_tool_rejects_fast_oversized_output_after_child_exit() {
    let fixture = Fixture::new(&find_program("seq"));
    let mut invocation = fixture.invocation(vec!["1".to_string(), "10000".to_string()]);
    invocation.limits.output_bytes = TEST_OUTPUT_BYTES;
    let run = run_offline_tool(&fixture.toolchain, invocation).expect("fast output overflow produces a receipt");

    assert!(!run.success);
    assert_eq!(run.receipt.status, "output-limit-exceeded");
    assert!(run.stdout.len() <= usize::try_from(TEST_OUTPUT_BYTES).unwrap());
}

#[test]
fn offline_tool_timeout_kills_descendant_process_group() {
    let fixture = Fixture::new(&find_program("sh"));
    let pid_file = fixture.work.join("descendant.pid");
    let sleep = copy_executable(&find_program("sleep"), &fixture.root.path().join("bin/sleep"));
    let script = format!("{} 30 & echo $! > {}; wait", sleep.display(), pid_file.display());
    let mut invocation = fixture.invocation(vec!["-c".to_string(), script]);
    invocation.limits.timeout_ms = TEST_TIMEOUT_MS;
    let started = std::time::Instant::now();
    let run = run_offline_tool(&fixture.toolchain, invocation).expect("timeout produces a failed receipt");

    assert!(!run.success);
    assert_eq!(run.receipt.status, "timeout-exceeded");
    assert!(started.elapsed() < Duration::from_secs(5));
    let pid = wait_for_pid(&pid_file);
    assert!(!process_is_running(pid), "descendant process {pid} survived timeout");
}

#[test]
fn offline_tool_rejects_symlink_output_and_parent_escape() {
    let fixture = Fixture::new(&find_program("sh"));
    let output = fixture.work.join("artifact.wasm");
    let link = find_program("ln");
    let mut invocation = fixture.invocation(vec![
        "-c".to_string(),
        format!("{} -s /etc/passwd {}", link.display(), output.display()),
    ]);
    invocation.output_path = Some(output);
    let error =
        run_offline_tool(&fixture.toolchain, invocation).expect_err("symlink output must fail no-follow hashing");
    assert!(error.to_string().contains("symlink file component"), "{error}");

    let outside = fixture.root.path().join("outside");
    fs::create_dir(&outside).unwrap();
    let linked_parent = fixture.work.join("linked");
    symlink(&outside, &linked_parent).unwrap();
    let mut escaped = fixture.invocation(Vec::new());
    escaped.output_path = Some(linked_parent.join("out.wasm"));
    let error = run_offline_tool(&fixture.toolchain, escaped).expect_err("symlink parent must fail confinement");
    assert!(error.to_string().contains("symlink path component"));
}

#[test]
fn offline_tool_rejects_symlink_cwd_and_argument_environment_bounds() {
    let fixture = Fixture::new(&find_program("echo"));
    let linked_cwd = fixture.root.path().join("linked-cwd");
    symlink(&fixture.work, &linked_cwd).unwrap();
    let mut symlinked = fixture.invocation(Vec::new());
    symlinked.cwd = linked_cwd;
    let error = run_offline_tool(&fixture.toolchain, symlinked).expect_err("symlink cwd must fail confinement");
    assert!(error.to_string().contains("symlink path component"));

    let oversized_arg = fixture.invocation(vec!["a".repeat(OVERSIZED_ARGUMENT_BYTES)]);
    let error = run_offline_tool(&fixture.toolchain, oversized_arg).expect_err("oversized arg must fail");
    assert!(error.to_string().contains("argument exceeds"));

    let mut oversized_env = fixture.invocation(Vec::new());
    oversized_env.env.insert("OVERSIZED".to_string(), "v".repeat(OVERSIZED_ENV_VALUE_BYTES));
    let error = run_offline_tool(&fixture.toolchain, oversized_env).expect_err("oversized env must fail");
    assert!(error.to_string().contains("environment key/value exceeds"));
}

struct Fixture {
    root: tempfile::TempDir,
    work: PathBuf,
    toolchain: VerifiedToolchain,
}

impl Fixture {
    fn new(program: &Path) -> Self {
        let root = tempfile::tempdir().unwrap();
        let bin = root.path().join("bin");
        let work = root.path().join("work");
        fs::create_dir(&bin).unwrap();
        fs::create_dir(&work).unwrap();
        let runner = bin.join("runner");
        let shell = find_program("sh");
        let runner_script = format!("#!{}\nexec \"{}\" \"$@\"\n", shell.display(), program.display());
        fs::write(&runner, runner_script).unwrap();
        make_executable(&runner);
        let bwrap = bin.join("bwrap");
        let fake_bwrap = format!(
            "#!{}\nwhile [ \"$#\" -gt 0 ]; do\n  if [ \"$1\" = -- ]; then shift; exec \"$@\"; fi\n  shift\ndone\nexit 125\n",
            shell.display()
        );
        fs::write(&bwrap, fake_bwrap).unwrap();
        make_executable(&bwrap);
        let tools = vec![record("runner", &runner), record("bwrap", &bwrap)];
        let toolchain = VerifiedToolchain {
            root: root.path().to_path_buf(),
            manifest: ToolchainManifest {
                schema: "test-toolchain".to_string(),
                rust_target: "test-target".to_string(),
                tools,
                cohort_identity_blake3: Blake3Identity::from_slice(b"test-cohort"),
            },
        };
        Self { root, work, toolchain }
    }

    fn invocation(&self, args: Vec<String>) -> ToolInvocation {
        ToolInvocation {
            stage_key: "test-stage".to_string(),
            tool_name: "runner".to_string(),
            args,
            cwd: self.work.clone(),
            work_root: self.root.path().to_path_buf(),
            env: BTreeMap::new(),
            read_only_inputs: Vec::new(),
            output_path: None,
            limits: ToolLimits::default(),
        }
    }
}

fn record(name: &str, path: &Path) -> ToolRecord {
    let bytes = fs::read(path).unwrap();
    ToolRecord {
        name: name.to_string(),
        version: "test".to_string(),
        version_output: "test".to_string(),
        path: format!("bin/{name}"),
        binary_digest_blake3: Blake3Identity::from_slice(&bytes),
    }
}

fn copy_executable(source: &Path, destination: &Path) -> PathBuf {
    fs::copy(source, destination).unwrap();
    make_executable(destination);
    destination.to_path_buf()
}

fn make_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(path, permissions).unwrap();
}

fn find_program(name: &str) -> PathBuf {
    std::env::split_paths(&std::env::var_os("PATH").expect("test PATH"))
        .map(|directory| directory.join(name))
        .find(|candidate| candidate.is_file())
        .unwrap_or_else(|| panic!("test program `{name}` is unavailable"))
}

fn process_is_running(pid: u32) -> bool {
    match fs::read_to_string(format!("/proc/{pid}/stat")) {
        Ok(text) => text.split_whitespace().nth(2).is_some_and(|state| state != "Z"),
        Err(_) => false,
    }
}

fn wait_for_pid(path: &Path) -> u32 {
    for _ in 0..100 {
        if let Ok(text) = fs::read_to_string(path) {
            return text.trim().parse().unwrap();
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!("timed-out tool never wrote descendant pid")
}
