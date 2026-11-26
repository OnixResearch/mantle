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

fn hex_digest(ch: char) -> String {
    std::iter::repeat_n(ch, 64).collect()
}

fn sample_tool(root: &str, member: &str, digest_ch: char) -> Tool {
    Tool {
        root: root.into(),
        member: member.into(),
        blake3: hex_digest(digest_ch),
    }
}

fn sample_target(with_coverage: bool) -> Target {
    let bin_root = "/mantle/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-bin";
    let src_root = "/mantle/store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-src";
    let mut inputs = BTreeMap::from([(bin_root.into(), hex_digest('1'))]);
    let coverage = if with_coverage {
        inputs.insert(src_root.into(), hex_digest('2'));
        Some(crunch_test_core::Coverage {
            profdata: sample_tool(bin_root, "bin/llvm-profdata", '3'),
            cov: sample_tool(bin_root, "bin/llvm-cov", '4'),
            source_root: src_root.into(),
            source_blake3: hex_digest('2'),
            max_profile_bytes: 1_048_576,
        })
    } else {
        None
    };
    Target {
        schema: crunch_test_core::TARGET_SCHEMA.into(),
        name: "unit-target".into(),
        cohort: crunch_test_core::COHORT.into(),
        rustc_verbose_version: "rustc 1.90.0 (1159e78c4 2025-09-14)\nLLVM version: 20.1.8\n".into(),
        store_prefix: "/mantle/store".into(),
        platform: "x86_64-linux".into(),
        executable: sample_tool(bin_root, "bin/test-bin", '5'),
        runner: sample_tool(bin_root, "bin/test-runner", '6'),
        inputs,
        runfiles: BTreeMap::new(),
        environment: BTreeMap::from([("TEST_FLAG".into(), "ok".into())]),
        tests: vec!["tests::alpha".into(), "tests::beta".into()],
        resources: crunch_test_core::Resources {
            memory_bytes: 67_108_864,
            output_bytes: 65_536,
            timeout_ms: 5_000,
        },
        deterministic: true,
        external_services: false,
        coverage,
    }
}

fn write_report_json(dir: &Path, report: &Report) {
    fs::write(dir.join("report.json"), serde_json::to_vec(report).unwrap()).unwrap();
}

fn sample_pass_report(request: &SelectedRequest, repeat: u32) -> Report {
    let mut attempts = Vec::new();
    for round in 0..repeat {
        for test in &request.selected {
            attempts.push(crunch_test_core::Attempt {
                repeat: round,
                test: test.clone(),
                outcome: Outcome::Pass,
                reason: "complete".into(),
            });
        }
    }
    Report {
        schema: crunch_test_core::REPORT_SCHEMA.into(),
        identity: request.identity.clone(),
        target: request.target.name.clone(),
        selected: request.selected.clone(),
        outcome: Outcome::Pass,
        attempts,
        coverage: None,
        collection_failure: None,
    }
}

fn sample_coverage_payload() -> Vec<u8> {
    br#"{"type":"llvm.coverage.json.export","version":"2.0.1","data":[{"totals":{"lines":{"count":20,"covered":15}}}]}"#
        .to_vec()
}

fn sample_coverage_report(export_bytes: &[u8]) -> CoverageReport {
    CoverageReport {
        cohort: crunch_test_core::COHORT.into(),
        source_root: "/mantle/store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-src".into(),
        source_blake3: hex_digest('2'),
        profdata_blake3: hex_digest('3'),
        cov_blake3: hex_digest('4'),
        export_blake3: blake3::hash(export_bytes).to_hex().to_string(),
        covered_lines: 15,
        total_lines: 20,
    }
}

#[test]
fn read_report_accepts_complete_pass_assertion_failure_and_valid_coverage() {
    let dir = tempfile::tempdir().unwrap();
    let request = select(sample_target(true), Invocation {
        coverage: true,
        ..Invocation::default()
    })
    .unwrap();
    let cov_bytes = sample_coverage_payload();
    fs::write(dir.path().join("coverage.json"), &cov_bytes).unwrap();
    let mut report = sample_pass_report(&request, 1);
    report.coverage = Some(sample_coverage_report(&cov_bytes));
    write_report_json(dir.path(), &report);

    let loaded = read_report(dir.path(), request).unwrap();
    assert_eq!(loaded, report);

    let af_request = select(sample_target(false), Invocation::default()).unwrap();
    let mut af_report = sample_pass_report(&af_request, 1);
    af_report.attempts[1].outcome = Outcome::AssertionFailure;
    af_report.attempts[1].reason = "assertion".into();
    af_report.outcome = Outcome::AssertionFailure;
    write_report_json(dir.path(), &af_report);

    let loaded_af = read_report(dir.path(), af_request).unwrap();
    assert_eq!(loaded_af.outcome, Outcome::AssertionFailure);
}

#[test]
fn read_report_rejects_incomplete_non_cancelled_reports_without_collection_failure() {
    let dir = tempfile::tempdir().unwrap();
    let request = select(sample_target(false), Invocation {
        repeat: 2,
        ..Invocation::default()
    })
    .unwrap();
    let mut incomplete_pass = sample_pass_report(&request, 1);
    incomplete_pass.attempts.truncate(1);
    write_report_json(dir.path(), &incomplete_pass);
    let err_msg = read_report(
        dir.path(),
        select(sample_target(false), Invocation {
            repeat: 2,
            ..Invocation::default()
        })
        .unwrap(),
    )
    .unwrap_err();
    assert_eq!(err_msg, "test-result-incomplete");

    let mut empty_infra = sample_pass_report(&request, 1);
    empty_infra.attempts.clear();
    empty_infra.outcome = Outcome::InfrastructureFailure;
    write_report_json(dir.path(), &empty_infra);
    let err_msg = read_report(
        dir.path(),
        select(sample_target(false), Invocation {
            repeat: 2,
            ..Invocation::default()
        })
        .unwrap(),
    )
    .unwrap_err();
    assert_eq!(err_msg, "test-result-incomplete");

    let mut extra_attempts = sample_pass_report(&request, 2);
    extra_attempts.attempts.push(extra_attempts.attempts[0].clone());
    write_report_json(dir.path(), &extra_attempts);
    let err_msg = read_report(dir.path(), request).unwrap_err();
    assert_eq!(err_msg, "test-result-incomplete");
}

#[test]
fn read_report_handles_collection_failure_completeness_and_contradictions() {
    let dir = tempfile::tempdir().unwrap();
    for failure_outcome in [Outcome::InfrastructureFailure, Outcome::Timeout, Outcome::Cancelled] {
        let request = select(sample_target(false), Invocation::default()).unwrap();
        let mut report = sample_pass_report(&request, 1);
        report.attempts.clear();
        report.outcome = failure_outcome;
        report.collection_failure = Some(crunch_test_core::CollectionFailure {
            outcome: failure_outcome,
            reason: "prepare-failed".into(),
        });
        write_report_json(dir.path(), &report);
        let loaded = read_report(dir.path(), request).unwrap();
        assert_eq!(loaded.outcome, failure_outcome);
    }

    // Partial non-empty attempts with non-Cancelled outcome must be rejected even if collection_failure
    // is present.
    let partial_req = select(sample_target(false), Invocation::default()).unwrap();
    let mut partial = sample_pass_report(&partial_req, 1);
    partial.attempts.truncate(1);
    partial.outcome = Outcome::InfrastructureFailure;
    partial.collection_failure = Some(crunch_test_core::CollectionFailure {
        outcome: Outcome::InfrastructureFailure,
        reason: "partial".into(),
    });
    write_report_json(dir.path(), &partial);
    assert_eq!(read_report(dir.path(), partial_req).unwrap_err(), "test-result-incomplete");

    // Full attempts with collection_failure (coverage failure after tests ran) must succeed.
    let full_req = select(sample_target(false), Invocation::default()).unwrap();
    let mut full = sample_pass_report(&full_req, 1);
    full.outcome = Outcome::InfrastructureFailure;
    full.collection_failure = Some(crunch_test_core::CollectionFailure {
        outcome: Outcome::InfrastructureFailure,
        reason: "test-coverage-merge-failed".into(),
    });
    write_report_json(dir.path(), &full);
    assert_eq!(read_report(dir.path(), full_req).unwrap().attempts.len(), 2);

    // Invalid collection_failure outcome (Pass / AssertionFailure) must be rejected.
    for invalid_outcome in [Outcome::Pass, Outcome::AssertionFailure] {
        let req = select(sample_target(false), Invocation::default()).unwrap();
        let mut report = sample_pass_report(&req, 1);
        report.attempts.clear();
        report.outcome = invalid_outcome;
        report.collection_failure = Some(crunch_test_core::CollectionFailure {
            outcome: invalid_outcome,
            reason: "bogus".into(),
        });
        write_report_json(dir.path(), &report);
        assert_eq!(read_report(dir.path(), req).unwrap_err(), "test-result-collection-contradiction");
    }
}

#[test]
fn read_report_rejects_stale_identity_bad_order_outcome_contradiction_and_missing_coverage() {
    let dir = tempfile::tempdir().unwrap();
    let base_req = select(sample_target(false), Invocation::default()).unwrap();

    let mut wrong_schema = sample_pass_report(&base_req, 1);
    wrong_schema.schema = "stale-schema".into();
    write_report_json(dir.path(), &wrong_schema);
    assert_eq!(
        read_report(dir.path(), select(sample_target(false), Invocation::default()).unwrap()).unwrap_err(),
        "test-result-stale-or-mismatched"
    );

    let mut wrong_identity = sample_pass_report(&base_req, 1);
    wrong_identity.identity = hex_digest('f');
    write_report_json(dir.path(), &wrong_identity);
    assert_eq!(
        read_report(dir.path(), select(sample_target(false), Invocation::default()).unwrap()).unwrap_err(),
        "test-result-stale-or-mismatched"
    );

    let mut wrong_target = sample_pass_report(&base_req, 1);
    wrong_target.target = "other".into();
    write_report_json(dir.path(), &wrong_target);
    assert_eq!(
        read_report(dir.path(), select(sample_target(false), Invocation::default()).unwrap()).unwrap_err(),
        "test-result-stale-or-mismatched"
    );

    let mut wrong_selected = sample_pass_report(&base_req, 1);
    wrong_selected.selected.reverse();
    write_report_json(dir.path(), &wrong_selected);
    assert_eq!(
        read_report(dir.path(), select(sample_target(false), Invocation::default()).unwrap()).unwrap_err(),
        "test-result-stale-or-mismatched"
    );

    let mut wrong_order = sample_pass_report(&base_req, 1);
    wrong_order.attempts.swap(0, 1);
    write_report_json(dir.path(), &wrong_order);
    assert_eq!(
        read_report(dir.path(), select(sample_target(false), Invocation::default()).unwrap()).unwrap_err(),
        "test-result-attempt-order"
    );

    let mut outcome_mismatch = sample_pass_report(&base_req, 1);
    outcome_mismatch.outcome = Outcome::AssertionFailure;
    write_report_json(dir.path(), &outcome_mismatch);
    assert_eq!(
        read_report(dir.path(), select(sample_target(false), Invocation::default()).unwrap()).unwrap_err(),
        "test-result-outcome-contradiction"
    );

    let cov_req = select(sample_target(true), Invocation {
        coverage: true,
        ..Invocation::default()
    })
    .unwrap();
    let missing_cov = sample_pass_report(&cov_req, 1);
    write_report_json(dir.path(), &missing_cov);
    assert_eq!(read_report(dir.path(), cov_req).unwrap_err(), "test-result-coverage-missing");
}

#[test]
fn check_coverage_match_validates_contract_digests_and_bounds() {
    let dir = tempfile::tempdir().unwrap();
    let cov_bytes = sample_coverage_payload();
    fs::write(dir.path().join("coverage.json"), &cov_bytes).unwrap();
    let target_with_cov = sample_target(true);
    let target_without_cov = sample_target(false);
    let cov_invocation = Invocation {
        coverage: true,
        ..Invocation::default()
    };
    let valid_cov = sample_coverage_report(&cov_bytes);

    assert!(check_coverage_match(dir.path(), &target_with_cov, &cov_invocation, &valid_cov).is_ok());
    assert_eq!(
        check_coverage_match(dir.path(), &target_without_cov, &cov_invocation, &valid_cov).unwrap_err(),
        "test-result-coverage-undeclared"
    );
    assert_eq!(
        check_coverage_match(dir.path(), &target_with_cov, &Invocation::default(), &valid_cov).unwrap_err(),
        "test-result-coverage-mismatch"
    );

    let mut bad = valid_cov.clone();
    bad.export_blake3 = hex_digest('0');
    assert_eq!(
        check_coverage_match(dir.path(), &target_with_cov, &cov_invocation, &bad).unwrap_err(),
        "test-result-coverage-mismatch"
    );

    bad = valid_cov.clone();
    bad.total_lines = 0;
    bad.covered_lines = 0;
    assert_eq!(
        check_coverage_match(dir.path(), &target_with_cov, &cov_invocation, &bad).unwrap_err(),
        "test-result-coverage-mismatch"
    );

    bad = valid_cov.clone();
    bad.covered_lines = bad.total_lines + 1;
    assert_eq!(
        check_coverage_match(dir.path(), &target_with_cov, &cov_invocation, &bad).unwrap_err(),
        "test-result-coverage-mismatch"
    );

    bad = valid_cov.clone();
    bad.cohort = "other-cohort".into();
    assert_eq!(
        check_coverage_match(dir.path(), &target_with_cov, &cov_invocation, &bad).unwrap_err(),
        "test-result-coverage-mismatch"
    );

    bad = valid_cov.clone();
    bad.source_blake3 = hex_digest('9');
    assert_eq!(
        check_coverage_match(dir.path(), &target_with_cov, &cov_invocation, &bad).unwrap_err(),
        "test-result-coverage-mismatch"
    );

    bad = valid_cov.clone();
    bad.profdata_blake3 = hex_digest('9');
    assert_eq!(
        check_coverage_match(dir.path(), &target_with_cov, &cov_invocation, &bad).unwrap_err(),
        "test-result-coverage-mismatch"
    );

    bad = valid_cov;
    bad.cov_blake3 = hex_digest('9');
    assert_eq!(
        check_coverage_match(dir.path(), &target_with_cov, &cov_invocation, &bad).unwrap_err(),
        "test-result-coverage-mismatch"
    );
}

#[test]
fn lower_and_write_recipe_render_nickel_escape_and_forbid_overwrite() {
    let dir = tempfile::tempdir().unwrap();
    let mut target = sample_target(false);
    target.environment.insert("TEST_INTERP".into(), "literal %{escape}".into());
    let request = select(target, Invocation::default()).unwrap();
    let expected_name = format!("test-unit-target-{}", &request.identity[..16]);

    let lowered = lower(request).unwrap();
    assert_eq!(lowered.derivation.name, expected_name);
    assert_eq!(lowered.derivation.builder, "/mantle/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-bin/bin/test-runner");
    assert_eq!(lowered.derivation.args, vec!["run"]);
    assert_eq!(lowered.derivation.system, "x86_64-linux");
    assert_eq!(lowered.derivation.outputs, vec!["out"]);
    assert_eq!(lowered.derivation.addressing_mode, "input-addressed");
    assert_eq!(lowered.derivation.inputs, vec!["/mantle/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-bin"]);
    assert!(lowered.derivation.env.contains_key("MANTLE_TEST_TARGET"));
    assert!(lowered.derivation.env.contains_key("MANTLE_TEST_INVOCATION"));

    let recipe_path = dir.path().join("recipe.ncl");
    write_recipe(lowered.request, &recipe_path).unwrap();
    let content = fs::read_to_string(&recipe_path).unwrap();
    assert!(content.contains("\\%{escape}"), "unexpected recipe content: {content}");

    let dup_request = select(sample_target(false), Invocation::default()).unwrap();
    assert!(write_recipe(dup_request, &recipe_path).is_err());
}

#[test]
fn load_target_validates_file_size_json_and_presence() {
    let dir = tempfile::tempdir().unwrap();
    let valid_path = dir.path().join("target.json");
    let target = sample_target(false);
    fs::write(&valid_path, serde_json::to_vec(&target).unwrap()).unwrap();
    let selected = load_target(&valid_path, Invocation::default()).unwrap();
    assert_eq!(selected.target, target);
    assert_eq!(selected.selected, vec!["tests::alpha", "tests::beta"]);

    let bad_json = dir.path().join("bad.json");
    fs::write(&bad_json, b"{not-json").unwrap();
    assert!(load_target(&bad_json, Invocation::default()).is_err());

    let oversized = dir.path().join("oversized.json");
    fs::write(&oversized, vec![b' '; usize::try_from(MAX_TARGET_BYTES).unwrap() + 1]).unwrap();
    assert_eq!(load_target(&oversized, Invocation::default()).unwrap_err(), "test-file-limit");

    assert!(load_target(&dir.path().join("missing.json"), Invocation::default()).is_err());
}

#[test]
fn executable_enforces_declared_roots_symlinks_and_blake3_digest() {
    let dir = tempfile::tempdir().unwrap();
    let tool_root = dir.path().join("tool-root");
    let dep_root = dir.path().join("dep-root");
    let outside_root = dir.path().join("outside-root");
    fs::create_dir_all(tool_root.join("bin")).unwrap();
    fs::create_dir_all(dep_root.join("bin")).unwrap();
    fs::create_dir_all(outside_root.join("bin")).unwrap();

    let payload = b"#!/bin/sh\nexit 0\n";
    let digest = blake3::hash(payload).to_hex().to_string();
    let real_bin = tool_root.join("bin/real");
    fs::write(&real_bin, payload).unwrap();

    let mut target = sample_target(false);
    target.inputs.insert(dep_root.display().to_string(), hex_digest('a'));

    let direct_tool = Tool {
        root: tool_root.display().to_string(),
        member: "bin/real".into(),
        blake3: digest.clone(),
    };
    assert_eq!(executable(&target, &direct_tool).unwrap(), fs::canonicalize(&real_bin).unwrap());

    let dep_target_bin = dep_root.join("bin/from-dep");
    fs::write(&dep_target_bin, payload).unwrap();
    let symlink_in_dep = tool_root.join("bin/link-dep");
    symlink(&dep_target_bin, &symlink_in_dep).unwrap();
    let dep_symlink_tool = Tool {
        root: tool_root.display().to_string(),
        member: "bin/link-dep".into(),
        blake3: digest.clone(),
    };
    assert_eq!(executable(&target, &dep_symlink_tool).unwrap(), fs::canonicalize(&dep_target_bin).unwrap());

    let outside_bin = outside_root.join("bin/escape");
    fs::write(&outside_bin, payload).unwrap();
    let symlink_outside = tool_root.join("bin/link-outside");
    symlink(&outside_bin, &symlink_outside).unwrap();
    let escaped_tool = Tool {
        root: tool_root.display().to_string(),
        member: "bin/link-outside".into(),
        blake3: digest,
    };
    assert_eq!(executable(&target, &escaped_tool).unwrap_err(), "test-tool-symlink-undeclared");

    let wrong_digest_tool = Tool {
        root: tool_root.display().to_string(),
        member: "bin/real".into(),
        blake3: hex_digest('0'),
    };
    assert_eq!(executable(&target, &wrong_digest_tool).unwrap_err(), "test-tool-digest-mismatch");

    let missing_tool = Tool {
        root: tool_root.display().to_string(),
        member: "bin/missing".into(),
        blake3: hex_digest('0'),
    };
    assert!(executable(&target, &missing_tool).is_err());
}
