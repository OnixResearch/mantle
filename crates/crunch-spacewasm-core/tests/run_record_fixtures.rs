//! Fixtures for run-record admission and identity-cycle classification.
//!
//! Positive fixtures admit complete records. Negative fixtures cover missing
//! stable identity, unknown statuses, contradictory exit/signal facts,
//! duplicate captures, over-limit captures, cycle membership, and raw-capture
//! membership in the stable bundle.

use crunch_spacewasm_core::Blake3Digest;
use crunch_spacewasm_core::RawCapture;
use crunch_spacewasm_core::RunRecordRequest;
use crunch_spacewasm_core::admit_run_record;
use crunch_spacewasm_core::classify_identity_cycle;

fn digest(seed: u8) -> Blake3Digest {
    Blake3Digest::from_slice(&[seed; 32])
}

fn capture(role: &str, seed: u8) -> RawCapture {
    RawCapture {
        role: String::from(role),
        blake3: digest(seed),
        size_bytes: 512,
    }
}

fn request() -> RunRecordRequest {
    RunRecordRequest {
        stable_identity_blake3: digest(1),
        suite: String::from("upstream-unit-tests"),
        command: String::from(
            "cargo test --locked --offline --no-default-features --lib -- --format json -Z unstable-options",
        ),
        process_status: String::from("passed"),
        exit_code: Some(0),
        signal: None,
        captures: vec![capture("stdout", 2), capture("stderr", 3)],
    }
}

#[test]
fn complete_run_record_admits_with_canonical_capture_order() {
    let mut request = request();
    request.captures = vec![capture("stderr", 3), capture("stdout", 2)];
    let record = admit_run_record(&request).record.expect("admits");
    assert_eq!(record.captures[0].role, "stderr");
    assert_eq!(record.captures[1].role, "stdout");
    assert_eq!(record.schema, "mantle-spacewasm-run-record-v1");
    assert_eq!(record.encoding_version, 1);
}

#[test]
fn run_record_identity_is_order_and_presentation_invariant() {
    let first = admit_run_record(&request()).record.expect("admits");
    let mut reordered = request();
    reordered.captures = vec![capture("stderr", 3), capture("stdout", 2)];
    let second = admit_run_record(&reordered).record.expect("admits");
    assert_eq!(first.run_record_identity_blake3, second.run_record_identity_blake3);
}

#[test]
fn stable_identity_is_total_and_always_bound() {
    // A missing stable identity is unrepresentable: `Blake3Digest` only
    // constructs from hashed bytes or parsed lowercase hex. Every admitted
    // record therefore binds a stable identity.
    let record = admit_run_record(&request()).record.expect("admits");
    assert_eq!(record.stable_identity_blake3.to_string().len(), 64);
    assert!(record.stable_identity_blake3.to_string().bytes().all(|byte| byte.is_ascii_hexdigit()));
}

#[test]
fn unknown_status_rejects() {
    let mut request = request();
    request.process_status = String::from("probably-fine");
    let result = admit_run_record(&request);
    assert!(result.record.is_none());
    assert!(result.diagnostics.iter().any(|d| d.code == "run-record-unknown-status"));
}

#[test]
fn contradictory_exit_and_signal_facts_reject() {
    let mut signaled = request();
    signaled.process_status = String::from("signal");
    signaled.exit_code = None;
    let result = admit_run_record(&signaled);
    assert!(result.record.is_none());
    assert!(result.diagnostics.iter().any(|d| d.code == "run-record-status-contradiction"));

    let mut passing_with_failure = request();
    passing_with_failure.exit_code = Some(1);
    let result = admit_run_record(&passing_with_failure);
    assert!(result.record.is_none());
    assert!(result.diagnostics.iter().any(|d| d.code == "run-record-status-contradiction"));

    let mut signaled_with_code = request();
    signaled_with_code.signal = Some(9);
    let result = admit_run_record(&signaled_with_code);
    assert!(result.record.is_none());
    assert!(result.diagnostics.iter().any(|d| d.code == "run-record-status-contradiction"));
}

#[test]
fn duplicate_and_over_limit_captures_reject() {
    let mut duplicate = request();
    duplicate.captures = vec![capture("stdout", 2), capture("stdout", 9)];
    let result = admit_run_record(&duplicate);
    assert!(result.record.is_none());
    assert!(result.diagnostics.iter().any(|d| d.code == "run-record-duplicate-capture"));

    let mut over_limit = request();
    over_limit.captures = (0..9).map(|index| capture(&format!("capture-{index}"), index as u8)).collect();
    let result = admit_run_record(&over_limit);
    assert!(result.record.is_none());
    assert!(result.diagnostics.iter().any(|d| d.code == "run-record-capture-bound"));
}

#[test]
fn empty_capture_role_rejects() {
    let mut request = request();
    request.captures = vec![capture("", 2)];
    let result = admit_run_record(&request);
    assert!(result.record.is_none());
    assert!(result.diagnostics.iter().any(|d| d.code == "run-record-capture-role"));
}

#[test]
fn identity_cycles_are_rejected() {
    let stable_members = vec![
        String::from("reports/checks/upstream-unit-tests.json"),
        String::from("reports/checks/upstream-unit-tests.stable-report.json"),
    ];
    let outcome = classify_identity_cycle(&stable_members, "runs/upstream-unit-tests/run-record.json", &[
        String::from("stdout.txt"),
        String::from("stderr.txt"),
    ]);
    assert!(outcome.is_ok());

    let outcome = classify_identity_cycle(&stable_members, "reports/checks/upstream-unit-tests.json", &[]);
    let diagnostic = outcome.expect_err("run record as member is a cycle");
    assert_eq!(diagnostic.code, "identity-cycle-run-record-member");

    let outcome = classify_identity_cycle(&stable_members, "runs/upstream-unit-tests/run-record.json", &[
        String::from("upstream-unit-tests.stable-report.json"),
        String::from("stdout.txt"),
    ]);
    let diagnostic = outcome.expect_err("raw capture role must not appear in stable members");
    assert_eq!(diagnostic.code, "identity-cycle-raw-capture-member");
}
