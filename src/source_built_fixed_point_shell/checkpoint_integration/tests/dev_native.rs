use super::*;
use crate::source_built_derivation_action_plan::eager_action_test_plan;
use crate::source_built_derivation_action_plan::reconcile_eager_derivation_actions;
use crate::source_built_fixed_point_checkpoint::ProofCheckpointOrigin;

#[test]
fn incomplete_dev_observations_remain_incomplete_and_cannot_be_promoted() {
    let plan = eager_action_test_plan();
    let observed = reconcile_eager_derivation_actions(&plan, &[]).unwrap();
    let before = observed.clone();
    validate_checkpoint_native_reconciliation(&plan, &observed, ProofCheckpointOrigin::DevExecution).unwrap();
    assert!(!observed.is_complete());
    assert!(
        validate_checkpoint_native_reconciliation(&plan, &observed, ProofCheckpointOrigin::PromotedExecution).is_err()
    );
    assert_eq!(observed, before);
}

#[test]
fn complete_native_observations_keep_the_promoted_gate() {
    let plan = eager_action_test_plan();
    let events = plan.actions.iter().map(|action| action.observed_goal_key_blake3.clone()).collect::<Vec<_>>();
    let observed = reconcile_eager_derivation_actions(&plan, &events).unwrap();
    assert!(observed.is_complete());
    validate_checkpoint_native_reconciliation(&plan, &observed, ProofCheckpointOrigin::PromotedExecution).unwrap();
    validate_checkpoint_native_reconciliation(&plan, &observed, ProofCheckpointOrigin::DevExecution).unwrap();
}

#[test]
fn native_prefix_support_bindings_use_restored_paths_not_origin_paths() {
    let temp = tempfile::tempdir().unwrap();
    let prepared = crate::source_built_fixed_point_shell::tests::prepared_fixture(&temp, temp.path().join("fresh"));
    let paths = RestoredProviderPaths::new(&prepared, STAGEX_PROVIDER_STORE_BASENAME, "native").unwrap();
    std::fs::create_dir_all(paths.rust_host_tool("linux-headers")).unwrap();
    std::fs::create_dir_all(&paths.rust_host_evidence).unwrap();
    let attestation = paths.rust_host_evidence.join("headers.json");
    std::fs::write(&attestation, b"{}").unwrap();
    const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let mut inputs = [crate::full_source_rust_binding::FullSourceRustHostSupportInputBinding {
        id: "linux-headers".to_string(),
        path: "/absent-origin/headers".to_string(),
        content_digest_blake3: DIGEST.to_string(),
        source_id: "headers".to_string(),
        attestation_path: "/absent-origin/headers.json".to_string(),
        attestation_digest_blake3: DIGEST.to_string(),
    }];
    relocate_host_bindings(&paths, &mut [], &mut inputs).unwrap();
    assert_eq!(Path::new(&inputs[0].path), paths.rust_host_tool("linux-headers"));
    assert_eq!(Path::new(&inputs[0].attestation_path), attestation);
    assert_eq!(inputs[0].content_digest_blake3, DIGEST);
    std::fs::remove_file(&attestation).unwrap();
    assert!(relocate_host_bindings(&paths, &mut [], &mut inputs).is_err());
}

#[test]
fn native_prefix_observations_keep_roots_separate_from_executables() {
    let temp = tempfile::tempdir().unwrap();
    let prepared = crate::source_built_fixed_point_shell::tests::prepared_fixture(&temp, temp.path().join("fresh"));
    let paths = RestoredProviderPaths::new(&prepared, STAGEX_PROVIDER_STORE_BASENAME, "native").unwrap();
    let mut manifest = crate::full_source_rust_binding::host_tool_test_manifest();
    let receipt = temp.path().join("receipt.json");
    std::fs::write(&receipt, b"{}").unwrap();
    for tool in &mut manifest.tools {
        let name = rust_host_tool_name(tool.role);
        let root = paths.rust_host_tool(name);
        let relative =
            crate::full_source_rust_binding_shell::full_source_rust_host_tool_executable_relative_path(tool.role)
                .unwrap();
        let executable = root.join(relative);
        std::fs::create_dir_all(executable.parent().unwrap()).unwrap();
        std::fs::write(&executable, b"fixture").unwrap();
        tool.path = executable.to_str().unwrap().to_string();
        tool.construction_receipt_path = receipt.to_str().unwrap().to_string();
    }
    for input in &mut manifest.support_inputs {
        let root = paths.rust_host_tool("linux-headers");
        std::fs::create_dir_all(root).unwrap();
        input.path = root.to_str().unwrap().to_string();
        input.attestation_path = receipt.to_str().unwrap().to_string();
    }
    crate::full_source_rust_binding::validate_full_source_rust_host_tool_manifest(&manifest).unwrap();
    let observations = resumed_host_tool_observations(&manifest, &paths).unwrap();
    assert_eq!(observations["make"].output.path, paths.rust_host_tool("make"));
    assert!(observations.values().all(|observation| observation.output.path.is_dir()));
    std::fs::remove_dir_all(paths.rust_host_tool("make")).unwrap();
    assert!(resumed_host_tool_observations(&manifest, &paths).is_err());
}

#[test]
fn dev_native_observations_reject_forged_counts_and_unknown_events() {
    let plan = eager_action_test_plan();
    let mut observed = reconcile_eager_derivation_actions(&plan, &[]).unwrap();
    observed.matched_action_count = plan.action_count;
    assert!(validate_checkpoint_native_reconciliation(&plan, &observed, ProofCheckpointOrigin::DevExecution).is_err());
    const UNKNOWN_EVENT: &str = "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
    let unknown = reconcile_eager_derivation_actions(&plan, &[UNKNOWN_EVENT.to_string()]).unwrap();
    assert!(validate_checkpoint_native_reconciliation(&plan, &unknown, ProofCheckpointOrigin::DevExecution).is_err());
}
