use std::collections::BTreeMap;

use crunch_test_core::*;
fn target() -> Target {
    let root = "/mantle/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-tool".to_string();
    let hash = "a".repeat(64);
    let tool = Tool {
        root: root.clone(),
        member: "bin/fixture".into(),
        blake3: hash.clone(),
    };
    Target {
        schema: TARGET_SCHEMA.into(),
        name: "fixture".into(),
        cohort: COHORT.into(),
        rustc_verbose_version: "rustc 1.90.0 (1159e78c4 2025-09-14)\nLLVM version: 20.1.8\n".into(),
        store_prefix: "/mantle/store".into(),
        platform: "x86_64-linux".into(),
        executable: tool.clone(),
        runner: tool,
        inputs: BTreeMap::from([(root, hash)]),
        runfiles: BTreeMap::new(),
        environment: BTreeMap::new(),
        tests: vec!["pass_beta".into(), "pass_alpha".into(), "assertion_failure".into()],
        resources: Resources {
            memory_bytes: 268_435_456,
            output_bytes: 65_536,
            timeout_ms: 1000,
        },
        deterministic: true,
        external_services: false,
        coverage: None,
    }
}
fn invocation() -> Invocation {
    Invocation {
        shard_count: 1,
        repeat: 1,
        ..Invocation::default()
    }
}
fn capture(name: &str, failed: bool) -> String {
    let (event, passed, failure) = if failed { ("failed", 0, 1) } else { ("ok", 1, 0) };
    format!(
        "{{\"type\":\"suite\",\"event\":\"started\",\"test_count\":1}}\n{{\"type\":\"test\",\"event\":\"started\",\"name\":\"{name}\"}}\n{{\"type\":\"test\",\"event\":\"{event}\",\"name\":\"{name}\"}}\n{{\"type\":\"suite\",\"event\":\"{event}\",\"passed\":{passed},\"failed\":{failure},\"ignored\":0,\"measured\":0,\"filtered_out\":2,\"exec_time\":0.001}}\n"
    )
}
#[test]
fn selection_is_ordered_filtered_then_disjointly_sharded() {
    let mut i = invocation();
    i.filter = Some("pass_".into());
    let request = select(target(), i).unwrap();
    assert_eq!(request.selected, ["pass_alpha", "pass_beta"]);
    let mut i = request.invocation;
    i.shard_count = 2;
    let request = select(request.target, i).unwrap();
    assert_eq!(request.selected, ["pass_alpha"]);
    let mut i = request.invocation;
    i.shard_index = 1;
    let request = select(request.target, i).unwrap();
    assert_eq!(request.selected, ["pass_beta"]);
    let mut i = request.invocation;
    i.exact = true;
    assert_eq!(select(request.target, i).unwrap_err(), "test-selection-empty");
}
#[test]
fn crash_incomplete_wrong_test_and_contradictory_exit_never_pass() {
    let good = capture("pass_alpha", false);
    assert_eq!(classify_capture("pass_alpha".into(), good.clone(), Some(0)).outcome, Outcome::Pass);
    assert_eq!(
        classify_capture("pass_alpha".into(), capture("pass_alpha", true), Some(101)).outcome,
        Outcome::AssertionFailure
    );
    assert_eq!(classify_capture("pass_alpha".into(), good.clone(), None).outcome, Outcome::InfrastructureFailure);
    assert_eq!(classify_capture("pass_beta".into(), good.clone(), Some(0)).outcome, Outcome::InfrastructureFailure);
    assert_eq!(
        classify_capture("pass_alpha".into(), good.clone(), Some(101)).outcome,
        Outcome::InfrastructureFailure
    );
    assert_eq!(
        classify_capture("pass_alpha".into(), good.trim_end().into(), Some(0)).outcome,
        Outcome::InfrastructureFailure
    );
    assert_eq!(
        classify_capture("pass_alpha".into(), format!("{good}{good}"), Some(0)).outcome,
        Outcome::InfrastructureFailure
    );
    assert_eq!(
        classify_capture("pass_alpha".into(), good.replace("\"passed\":1", "\"passed\":0"), Some(0)).outcome,
        Outcome::InfrastructureFailure
    );
}
#[test]
fn changed_inputs_seed_repeat_selection_deadline_and_nonce_invalidate_identity() {
    let original = select(target(), invocation()).unwrap().identity;
    let mut changed_target = target();
    changed_target.executable.blake3 = "b".repeat(64);
    assert_ne!(original, select(changed_target, invocation()).unwrap().identity);
    let mut changed_target = target();
    *changed_target.inputs.values_mut().next().unwrap() = "c".repeat(64);
    assert_ne!(original, select(changed_target, invocation()).unwrap().identity);
    for changed in [
        Invocation {
            seed: 7,
            ..invocation()
        },
        Invocation {
            repeat: 2,
            ..invocation()
        },
        Invocation {
            timeout_ms: Some(100),
            ..invocation()
        },
        Invocation {
            force_rerun: Some("unique-run-2".into()),
            ..invocation()
        },
        Invocation {
            filter: Some("pass_alpha".into()),
            exact: true,
            ..invocation()
        },
    ] {
        assert_ne!(original, select(target(), changed).unwrap().identity);
    }
    assert_eq!(original, select(target(), invocation()).unwrap().identity);
}
#[test]
fn ambient_inputs_unknown_options_and_services_fail_closed() {
    let mut t = target();
    t.external_services = true;
    assert!(select(t, invocation()).is_err());
    let mut t = target();
    t.runfiles.insert("escape".into(), "/etc".into());
    assert!(select(t, invocation()).is_err());
    let mut t = target();
    t.environment.insert("LD_PRELOAD".into(), "/ambient.so".into());
    assert!(select(t, invocation()).is_err());
    assert!(serde_json::from_str::<Invocation>(r#"{"filter":null,"exact":false,"shard_index":0,"shard_count":1,"timeout_ms":null,"seed":0,"repeat":1,"force_rerun":null,"coverage":false,"retries":3}"#).is_err());
    let mut t = target();
    t.tests[0] = "--ignored".into();
    assert_eq!(select(t, invocation()).unwrap_err(), "test-inventory");
}
