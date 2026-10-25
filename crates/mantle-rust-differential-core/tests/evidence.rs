use mantle_rust_differential_core::*;
use serde_json::Value;
use serde_json::json;

fn diagnostic(code: &str, message: &str) -> Value {
    json!({"message":message,"code":{"code":code,"explanation":null},"level":"error",
        "spans":[],"children":[],"rendered":null})
}

fn rejection(records: Vec<Value>) -> Observation {
    Observation {
        status: Status::Rejected,
        units: vec![UnitRecords {
            unit: "target-lib".into(),
            records,
        }],
        program_stdout: None,
        program_stderr: None,
        accepted_outputs: vec![],
        complete: true,
        teardown_complete: true,
        ice: false,
    }
}

fn expected(codes: &[&str]) -> Expected {
    Expected {
        status: Status::Rejected,
        diagnostic_codes: codes.iter().map(|s| (*s).into()).collect(),
        diagnostic_messages: codes
            .iter()
            .map(|code| {
                match *code {
                    "E0308" => "mismatched types",
                    _ => "cannot find value",
                }
                .into()
            })
            .collect(),
        program_stdout: None,
    }
}

#[test]
fn repeated_and_ordered_diagnostics_remain_consumer_visible() {
    let first = diagnostic("E0308", "mismatched types");
    let second = diagnostic("E0425", "cannot find value");
    let left = rejection(vec![first.clone(), second.clone(), first.clone()]);
    assert_eq!(compare(&left, &left, &expected(&["E0308", "E0425", "E0308"])), Verdict::Equivalent);
    let reordered = rejection(vec![second, first.clone(), first]);
    assert_eq!(compare(&left, &reordered, &expected(&["E0308", "E0425", "E0308"])), Verdict::Mismatch);
    let lost = rejection(vec![diagnostic("E0308", "mismatched types")]);
    assert_eq!(compare(&left, &lost, &expected(&["E0308", "E0425", "E0308"])), Verdict::Mismatch);
}

#[test]
fn unknown_fields_truncation_ice_and_bad_cleanup_are_not_equivalence() {
    let good = rejection(vec![diagnostic("E0308", "mismatched types")]);
    let oracle = expected(&["E0308"]);
    let mut unknown = good.clone();
    unknown.units[0].records[0]["new_semantic_fact"] = json!(true);
    assert_eq!(validate_observation(&unknown), Err(Error::UnknownSemanticField));
    for mutate in [0, 1, 2] {
        let mut broken = good.clone();
        match mutate {
            0 => broken.complete = false,
            1 => broken.teardown_complete = false,
            _ => broken.ice = true,
        }
        assert_eq!(compare(&broken, &broken, &oracle), Verdict::HarnessError);
    }
}

#[test]
fn rustc_diagnostic_discriminator_is_validated_without_accepting_other_records() {
    let mut observed = rejection(vec![diagnostic("E0425", "cannot find value")]);
    observed.units[0].records[0]["$message_type"] = json!("diagnostic");
    assert_eq!(compare(&observed, &observed, &expected(&["E0425"])), Verdict::Equivalent);
    for message_type in [json!("artifact"), Value::Null, json!(true)] {
        let mut invalid = observed.clone();
        invalid.units[0].records[0]["$message_type"] = message_type;
        assert_eq!(validate_observation(&invalid), Err(Error::Malformed));
        assert_eq!(compare(&invalid, &invalid, &expected(&["E0425"])), Verdict::HarnessError);
    }
    observed.units[0].records[0]["new_semantic_fact"] = json!(true);
    assert_eq!(validate_observation(&observed), Err(Error::UnknownSemanticField));
}

#[test]
fn expected_rejection_does_not_accept_success_crash_or_wrong_error() {
    let oracle = expected(&["E0308"]);
    let mut wrong = rejection(vec![diagnostic("E0425", "cannot find value")]);
    assert_eq!(compare(&wrong, &wrong, &oracle), Verdict::Mismatch);
    wrong.status = Status::Crash;
    assert_eq!(compare(&wrong, &wrong, &oracle), Verdict::HarnessError);
}

#[test]
fn successful_compilation_requires_current_accepted_output_identity() {
    let mut observed = Observation {
        status: Status::Success,
        units: vec![],
        program_stdout: None,
        program_stderr: None,
        accepted_outputs: vec![Identity {
            role: "current-check-output".into(),
            digest: Digest::from_slice(b"current"),
        }],
        complete: true,
        teardown_complete: true,
        ice: false,
    };
    let oracle = Expected {
        status: Status::Success,
        diagnostic_codes: vec![],
        diagnostic_messages: vec![],
        program_stdout: None,
    };
    assert_eq!(compare(&observed, &observed, &oracle), Verdict::Equivalent);
    observed.accepted_outputs.clear();
    assert_eq!(compare(&observed, &observed, &oracle), Verdict::HarnessError);
}

fn manifest() -> Manifest {
    let roles = [
        "source",
        "config",
        "lock",
        "dependencies",
        "rustc",
        "cargo",
        "sysroot",
        "tools",
        "argv",
        "environment",
        "profile",
        "features",
        "cache",
        "mode",
        "comparison",
        "edits",
        "harness",
    ];
    Manifest {
        schema: SCHEMA.into(),
        policy: POLICY.into(),
        bounds: Bounds::default(),
        identities: roles
            .iter()
            .map(|role| Identity {
                role: (*role).into(),
                digest: Digest::from_slice(role.as_bytes()),
            })
            .collect(),
        population: vec![
            Member {
                key: "chain-build".into(),
                role: "cell".into(),
                required: true,
                selected: true,
                eligible: true,
                reason: None,
            },
            Member {
                key: "test-sentinel".into(),
                role: "test".into(),
                required: true,
                selected: true,
                eligible: true,
                reason: None,
            },
            Member {
                key: "future".into(),
                role: "cell".into(),
                required: false,
                selected: true,
                eligible: false,
                reason: Some("no-protocol".into()),
            },
        ],
        denominators: vec![
            Denominator {
                role: "cell".into(),
                members: vec!["chain-build".into()],
            },
            Denominator {
                role: "test".into(),
                members: vec!["test-sentinel".into()],
            },
        ],
        cells: vec![Cell {
            id: "chain-build".into(),
            fixture: "chain".into(),
            operation: Operation::Build,
            profile: "dev".into(),
            lto: None,
            features: vec![],
            required: true,
            mode: "baseline-replay".into(),
        }],
    }
}

fn coverage() -> Vec<Coverage> {
    ["chain-build", "test-sentinel", "future"]
        .into_iter()
        .map(|key| Coverage {
            key: key.into(),
            attempted: key != "future",
            executed: key != "future",
            reused: false,
            fallback: None,
            unsupported: if key == "future" {
                Some("no-protocol".into())
            } else {
                None
            },
            failed: false,
            unrun: key == "future",
        })
        .collect()
}

#[test]
fn lost_test_or_replaced_denominator_cannot_hide_behind_equal_counts() {
    let frozen = manifest();
    assert!(reconcile(&frozen, &frozen, &coverage()).unwrap().complete);
    let mut revised = frozen.clone();
    revised.population[1].key = "replacement-test".into();
    revised.denominators[1].members[0] = "replacement-test".into();
    assert_eq!(reconcile(&frozen, &revised, &coverage()), Err(Error::BaselineRewrite));
    let mut lost = coverage();
    lost.remove(1);
    assert_eq!(reconcile(&frozen, &frozen, &lost), Err(Error::Missing));
}

#[test]
fn fallback_and_required_unrun_do_not_cover_original_route() {
    let frozen = manifest();
    let mut rows = coverage();
    rows[0].fallback = Some("fresh-target-restart".into());
    let report = reconcile(&frozen, &frozen, &rows).unwrap();
    assert!(!report.complete);
    assert_eq!(report.deficits, vec!["chain-build"]);
    rows[0].fallback = None;
    rows[0].attempted = false;
    rows[0].executed = false;
    rows[0].unrun = true;
    assert!(!reconcile(&frozen, &frozen, &rows).unwrap().complete);
}

#[test]
fn exact_edits_cover_binary_inputs_absence_and_wrong_preimages() {
    let original = SourceSet {
        files: vec![
            SourceFile {
                path: "bytes.bin".into(),
                bytes: vec![0, 1],
            },
            SourceFile {
                path: "src/lib.rs".into(),
                bytes: b"const N:u8=1;".to_vec(),
            },
        ],
    };
    let final_set = SourceSet {
        files: vec![
            SourceFile {
                path: "new.bin".into(),
                bytes: vec![255, 0],
            },
            SourceFile {
                path: "src/lib.rs".into(),
                bytes: b"const N:u8=2;".to_vec(),
            },
        ],
    };
    let edits = vec![
        ExactEdit {
            path: "bytes.bin".into(),
            before: Some(vec![0, 1]),
            after: None,
            needle: None,
            replacement: None,
            matches: 0,
        },
        ExactEdit {
            path: "new.bin".into(),
            before: None,
            after: Some(vec![255, 0]),
            needle: None,
            replacement: None,
            matches: 0,
        },
        ExactEdit {
            path: "src/lib.rs".into(),
            before: Some(b"const N:u8=1;".to_vec()),
            after: Some(b"const N:u8=2;".to_vec()),
            needle: Some(b"=1".to_vec()),
            replacement: Some(b"=2".to_vec()),
            matches: 1,
        },
    ];
    let before = original.identity().unwrap();
    let after = final_set.identity().unwrap();
    assert_eq!(apply_edits(&original, &edits, &before, &after).unwrap(), final_set);
    let mut missed = edits.clone();
    missed[2].matches = 2;
    assert_eq!(apply_edits(&original, &missed, &before, &after), Err(Error::Oracle));
    let mut traversal = original.clone();
    traversal.files[0].path = "../outside".into();
    assert_eq!(traversal.validate(), Err(Error::Identity));
}

#[test]
fn same_binary_future_registration_cannot_exchange_compilers() {
    let common = vec![
        Identity {
            role: "cargo".into(),
            digest: Digest::from_slice(b"cargo"),
        },
        Identity {
            role: "rustc".into(),
            digest: Digest::from_slice(b"rustc"),
        },
        Identity {
            role: "sysroot".into(),
            digest: Digest::from_slice(b"sysroot"),
        },
        Identity {
            role: "tools".into(),
            digest: Digest::from_slice(b"tools"),
        },
        Identity {
            role: "toggle".into(),
            digest: Digest::from_slice(b"off"),
        },
    ];
    let mut registration = ModeRegistration {
        id: "future-example".into(),
        version: 1,
        off: common.clone(),
        on: common,
        toggles: vec!["toggle".into()],
        capability: Some(Digest::from_slice(b"observed-protocol")),
        proof_obligations: [
            "after-publication-body-error",
            "swap-success",
            "crash",
            "cancel",
            "stale-output",
            "incremental-clean",
        ]
        .iter()
        .map(|s| (*s).into())
        .collect(),
    };
    registration.on[4].digest = Digest::from_slice(b"on");
    assert!(registration.validate().is_ok());
    registration.on[1].digest = Digest::from_slice(b"other compiler");
    assert_eq!(registration.validate(), Err(Error::Incomparable));
}

#[test]
fn recovery_preserves_the_interruption_instead_of_greenwashing_continuity() {
    let mut recovery = Recovery {
        predecessor: Digest::from_slice(b"interrupted-attempt"),
        successor: Digest::from_slice(b"fresh-successor"),
        cause: "producer-cancelled".into(),
        original_status: Status::Cancelled,
        original_observation: Digest::from_slice(b"retained-failed-observation"),
        discarded: vec![Identity {
            role: "cargo-target-not-reused".into(),
            digest: Digest::from_slice(b"partial-target"),
        }],
        preserved: vec![Identity {
            role: "source".into(),
            digest: Digest::from_slice(b"same-source"),
        }],
        teardown_complete: true,
        continuity_satisfied: false,
        remaining_obligations: vec!["uninterrupted-continuation-not-demonstrated".into()],
    };
    assert_eq!(recovery.validate(), Ok(()));
    recovery.continuity_satisfied = true;
    assert_eq!(recovery.validate(), Err(Error::Oracle));
    recovery.continuity_satisfied = false;
    recovery.teardown_complete = false;
    assert_eq!(recovery.validate(), Err(Error::IncompleteTeardown));
    recovery.teardown_complete = true;
    recovery.preserved.push(recovery.discarded[0].clone());
    assert_eq!(recovery.validate(), Err(Error::Duplicate));
}
