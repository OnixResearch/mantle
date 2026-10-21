//! Compile with the pinned rustc 1.90.0 cohort using compile.sh; not a mock harness.
#[test]
fn pass_alpha() { assert_eq!(6 * 7, 42); }
#[test]
fn pass_beta() { assert!("mantle".starts_with("man")); }
#[test]
fn assertion_failure() { assert_eq!(std::env::var("TEST_EXPECT").unwrap_or_default(), "accepted"); }
#[test]
fn timeout_owned_child() {
    let child = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "timeout_child", "--ignored"])
        .spawn().unwrap();
    if let Ok(path) = std::env::var("TEST_CHILD_PID_FILE") { std::fs::write(path, child.id().to_string()).unwrap(); }
    std::thread::sleep(std::time::Duration::from_secs(30));
    panic!("deadline did not stop the test");
}
#[test]
#[ignore]
fn timeout_child() { std::thread::sleep(std::time::Duration::from_secs(30)); }
#[test]
fn undeclared_input_denied() {
    let path = std::env::var("TEST_UNDECLARED_PATH").expect("declare a host sentinel path, not a runfile");
    assert!(std::fs::read(path).is_err(), "undeclared host input was visible");
}
#[test]
fn crash() { std::process::abort(); }
#[test]
fn partial_output() { std::process::exit(0); }
