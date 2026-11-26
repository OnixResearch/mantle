use std::path::Path;
use std::process::Command;
use std::process::Stdio;
use std::time::Duration;

use crunch_store::StoreMutationGuard;

const CHILD_STATE_ENV: &str = "MANTLE_MUTATION_LOCK_TEST_STATE_DIR";
const CHILD_MODE_ENV: &str = "MANTLE_MUTATION_LOCK_TEST_MODE";
const CHILD_ROLE_ENV: &str = "MANTLE_MUTATION_LOCK_TEST_ROLE";
const CHILD_READY_ENV: &str = "MANTLE_MUTATION_LOCK_TEST_READY_FILE";
const CHILD_RELEASE_ENV: &str = "MANTLE_MUTATION_LOCK_TEST_RELEASE_FILE";

const MODE_HOLD_UNTIL_RELEASE: &str = "hold-until-release";
const MODE_HOLD_AND_ABORT: &str = "hold-and-abort";
const MODE_TRY_ACQUIRE_EXPECT_BUSY: &str = "try-acquire-expect-busy";
const MODE_WAIT_AND_ACQUIRE: &str = "wait-and-acquire";

fn wait_for_file(path: &Path, max_polls: u32) -> bool {
    for _ in 0..max_polls {
        if path.exists() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    false
}

#[test]
fn mutation_lock_child_worker() {
    let Ok(state_dir) = std::env::var(CHILD_STATE_ENV) else {
        return;
    };
    let mode = std::env::var(CHILD_MODE_ENV).expect("child mode env");
    let role = std::env::var(CHILD_ROLE_ENV).unwrap_or_else(|_| "child-worker".to_string());
    let state_path = Path::new(&state_dir);

    match mode.as_str() {
        MODE_HOLD_UNTIL_RELEASE => {
            let ready = std::env::var(CHILD_READY_ENV).expect("ready file env");
            let release = std::env::var(CHILD_RELEASE_ENV).expect("release file env");
            let guard = StoreMutationGuard::try_acquire_for(state_path, &role).expect("child acquires lock");
            std::fs::write(&ready, format!("{}", std::process::id())).expect("write ready marker");
            assert!(wait_for_file(Path::new(&release), 500), "parent never released child");
            drop(guard);
        }
        MODE_HOLD_AND_ABORT => {
            let ready = std::env::var(CHILD_READY_ENV).expect("ready file env");
            let guard = StoreMutationGuard::try_acquire_for(state_path, &role).expect("child acquires lock");
            std::fs::write(&ready, format!("{}", std::process::id())).expect("write ready marker");
            // Leak guard intentionally across process::abort so Drop never runs
            // and the OS kernel releases flock on abrupt termination.
            std::mem::forget(guard);
            std::process::abort();
        }
        MODE_TRY_ACQUIRE_EXPECT_BUSY => {
            let err = StoreMutationGuard::try_acquire_for(state_path, &role).expect_err("lock must be busy");
            println!("{err}");
        }
        MODE_WAIT_AND_ACQUIRE => {
            let guard = StoreMutationGuard::acquire_wait_bounded(state_path, &role, 5_000)
                .expect("child acquires after parent drops");
            drop(guard);
        }
        other => panic!("unexpected child mode: {other}"),
    }
}

#[test]
fn two_process_try_acquire_reports_remote_pid_and_role() {
    let dir = tempfile::tempdir().expect("tempdir");
    let state_dir = dir.path().join("state");
    let ready_file = dir.path().join("ready");
    let release_file = dir.path().join("release");

    let exe = std::env::current_exe().expect("current_exe");
    let mut holder = Command::new(&exe)
        .arg("mutation_lock_child_worker")
        .arg("--exact")
        .arg("--nocapture")
        .env(CHILD_STATE_ENV, &state_dir)
        .env(CHILD_MODE_ENV, MODE_HOLD_UNTIL_RELEASE)
        .env(CHILD_ROLE_ENV, "child-watch")
        .env(CHILD_READY_ENV, &ready_file)
        .env(CHILD_RELEASE_ENV, &release_file)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn holder");

    assert!(wait_for_file(&ready_file, 500), "holder did not acquire lock in time");
    let holder_pid: u32 = std::fs::read_to_string(&ready_file).expect("read ready").trim().parse().expect("pid");

    let err = StoreMutationGuard::try_acquire_for(&state_dir, "parent-build")
        .expect_err("parent must see lock held by child");
    let msg = err.to_string();
    assert!(msg.contains("active"), "{msg}");
    assert!(msg.contains(&format!("pid={holder_pid}")), "{msg}");
    assert!(msg.contains("role=child-watch"), "{msg}");

    let bounded_err = StoreMutationGuard::acquire_wait_bounded(&state_dir, "parent-cmd", 60)
        .expect_err("bounded wait must time out while child holds lock");
    let bounded_msg = bounded_err.to_string();
    assert!(bounded_msg.contains("timed out"), "{bounded_msg}");
    assert!(bounded_msg.contains(&format!("pid={holder_pid}")), "{bounded_msg}");

    std::fs::write(&release_file, b"go").expect("release holder");
    let status = holder.wait().expect("wait holder");
    assert!(status.success(), "holder exited with {status}");

    let guard =
        StoreMutationGuard::try_acquire_for(&state_dir, "parent-build").expect("parent acquires after holder exits");
    drop(guard);
}

#[test]
fn two_process_abrupt_abort_releases_kernel_flock_for_next_owner() {
    let dir = tempfile::tempdir().expect("tempdir");
    let state_dir = dir.path().join("state");
    let ready_file = dir.path().join("ready");

    let exe = std::env::current_exe().expect("current_exe");
    let status = Command::new(&exe)
        .arg("mutation_lock_child_worker")
        .arg("--exact")
        .arg("--nocapture")
        .env(CHILD_STATE_ENV, &state_dir)
        .env(CHILD_MODE_ENV, MODE_HOLD_AND_ABORT)
        .env(CHILD_ROLE_ENV, "aborted-builder")
        .env(CHILD_READY_ENV, &ready_file)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .status()
        .expect("run aborting child");

    assert!(!status.success(), "aborted child must exit unsuccessfully");
    assert!(ready_file.exists(), "child must have acquired lock before aborting");

    // Stale owner metadata remains in the file until the next acquirer takes
    // kernel flock and overwrites it cleanly.
    let guard = StoreMutationGuard::try_acquire_for(&state_dir, "recovery-gc")
        .expect("kernel flock must be released despite skipped Drop");
    let second_err = StoreMutationGuard::try_acquire_for(&state_dir, "probe")
        .expect_err("new guard must own lock and overwrite stale metadata");
    let msg = second_err.to_string();
    assert!(msg.contains(&format!("pid={}", std::process::id())), "{msg}");
    assert!(msg.contains("role=recovery-gc"), "{msg}");
    drop(guard);
}

#[test]
fn two_process_wait_hands_off_after_parent_releases_guard() {
    let dir = tempfile::tempdir().expect("tempdir");
    let state_dir = dir.path().join("state");

    let guard = StoreMutationGuard::try_acquire_for(&state_dir, "parent-build").expect("parent acquires");
    let exe = std::env::current_exe().expect("current_exe");

    let busy_probe = Command::new(&exe)
        .arg("mutation_lock_child_worker")
        .arg("--exact")
        .arg("--nocapture")
        .env(CHILD_STATE_ENV, &state_dir)
        .env(CHILD_MODE_ENV, MODE_TRY_ACQUIRE_EXPECT_BUSY)
        .env(CHILD_ROLE_ENV, "probe")
        .output()
        .expect("run busy probe");
    assert!(busy_probe.status.success());
    let stdout = String::from_utf8_lossy(&busy_probe.stdout);
    assert!(stdout.contains(&format!("pid={}", std::process::id())), "{stdout}");
    assert!(stdout.contains("role=parent-build"), "{stdout}");

    let mut waiter = Command::new(&exe)
        .arg("mutation_lock_child_worker")
        .arg("--exact")
        .arg("--nocapture")
        .env(CHILD_STATE_ENV, &state_dir)
        .env(CHILD_MODE_ENV, MODE_WAIT_AND_ACQUIRE)
        .env(CHILD_ROLE_ENV, "queued-child")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn waiter");

    std::thread::sleep(Duration::from_millis(80));
    assert!(waiter.try_wait().expect("try_wait").is_none(), "waiter must block until parent drops");

    drop(guard);
    let waiter_status = waiter.wait().expect("waiter finishes");
    assert!(waiter_status.success(), "waiter failed: {waiter_status}");
}
