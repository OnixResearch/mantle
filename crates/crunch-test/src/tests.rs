use super::*;

// Real libtest subprocess fixtures. They are never run as ordinary workspace tests.
#[test]
#[ignore]
fn owned_sleeper_fixture() {
    thread::sleep(Duration::from_secs(30));
}
#[test]
#[ignore]
fn owned_parent_fixture() {
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--ignored", "--exact", "tests::owned_sleeper_fixture"])
        .spawn()
        .unwrap();
    fs::write(std::env::var("TEST_CHILD_PATH").unwrap(), child.id().to_string()).unwrap();
    assert!(child.wait().unwrap().success());
}
#[test]
#[cfg(target_os = "linux")]
fn deadline_kills_and_reaps_the_owned_libtest_child_group() {
    assert_eq!(unsafe { libc::prctl(libc::PR_SET_CHILD_SUBREAPER, 1, 0, 0, 0) }, 0);
    let scratch = tempfile::tempdir().unwrap();
    let work = scratch.path().to_path_buf();
    for name in ["runfiles", "home", "tmp"] {
        fs::create_dir(work.join(name)).unwrap();
    }
    let child_path = work.join("child-pid");
    let tool = Tool {
        root: "/mantle/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-tool".into(),
        member: "fixture".into(),
        blake3: "a".repeat(64),
    };
    let target = Target {
        schema: crunch_test_core::TARGET_SCHEMA.into(),
        name: "deadline".into(),
        cohort: crunch_test_core::COHORT.into(),
        rustc_verbose_version: "rustc 1.90.0 (1159e78c4 2025-09-14)\nLLVM version: 20.1.8\n".into(),
        store_prefix: "/mantle/store".into(),
        platform: "x86_64-linux".into(),
        executable: tool.clone(),
        runner: tool,
        inputs: BTreeMap::new(),
        runfiles: BTreeMap::new(),
        environment: BTreeMap::from([("TEST_CHILD_PATH".into(), child_path.display().to_string())]),
        tests: vec!["tests::owned_parent_fixture".into()],
        resources: crunch_test_core::Resources {
            memory_bytes: 268_435_456,
            output_bytes: 65_536,
            timeout_ms: 500,
        },
        deterministic: true,
        external_services: false,
        coverage: None,
    };
    let mut runner = Runner {
        work,
        next: 0,
        harness: None,
        interruption: None,
    };
    let mut command = runner.command(&std::env::current_exe().unwrap(), &target, &Invocation::default());
    command.args(["--ignored", "--exact", "tests::owned_parent_fixture"]);
    let started = Instant::now();
    let capture = runner.process(command, 500, 65_536).unwrap();
    assert_eq!(capture.interrupted, Some(Outcome::Timeout));
    assert!(started.elapsed() < Duration::from_secs(6));
    let pid: i32 = fs::read_to_string(&child_path).expect("real helper must have started").parse().unwrap();
    assert_eq!(unsafe { libc::kill(pid, 0) }, -1, "owned descendant survived teardown");
    assert_eq!(std::io::Error::last_os_error().raw_os_error(), Some(libc::ESRCH));
    fs::remove_file(&child_path).unwrap();
    let signal_path = child_path.clone();
    CANCELLED.store(false, Ordering::Relaxed);
    let previous = unsafe { libc::signal(libc::SIGTERM, cancel as *const () as libc::sighandler_t) };
    let signaller = thread::spawn(move || {
        for _ in 0..100 {
            if signal_path.exists() {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        unsafe {
            libc::kill(libc::getpid(), libc::SIGTERM);
        }
    });
    let mut command = runner.command(&std::env::current_exe().unwrap(), &target, &Invocation::default());
    command.args(["--ignored", "--exact", "tests::owned_parent_fixture"]);
    let capture = runner.process(command, 2000, 65_536).unwrap();
    signaller.join().unwrap();
    unsafe {
        libc::signal(libc::SIGTERM, previous);
    }
    CANCELLED.store(false, Ordering::Relaxed);
    assert_eq!(capture.interrupted, Some(Outcome::Cancelled));
    let pid: i32 = fs::read_to_string(child_path).unwrap().parse().unwrap();
    assert_eq!(unsafe { libc::kill(pid, 0) }, -1, "cancelled descendant survived teardown");
    assert_eq!(std::io::Error::last_os_error().raw_os_error(), Some(libc::ESRCH));
}
