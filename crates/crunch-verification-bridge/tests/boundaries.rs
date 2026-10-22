use std::fs;
use std::path::Path;
use std::path::PathBuf;

use crunch_verification_bridge::Producer;
use crunch_verification_bridge::consume;
use crunch_verification_bridge::consume_selected;
use crunch_verification_bridge::export;
use crunch_verification_bridge::packaging_inputs;
use crunch_verification_bridge_core::identity;
use serde_json::Value;

const OCTET_REV: &str = "4a540dff99aa2040e267e42bbafe09144f276572";
const TRELLIS_REV: &str = "22d51fb8c1f57155f6c0c4acc4e14902ebb6b4f3";

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/verification-bridge").join(name)
}

fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

#[test]
fn real_owner_handoffs_replay_with_exact_receipts_without_promoting_authority() {
    for (name, producer, revision) in [
        ("octet", Producer::Octet, OCTET_REV),
        ("trellis", Producer::Trellis, TRELLIS_REV),
    ] {
        let handoff = consume_selected(producer, &fixture(name), "profile.json", "receipt.json", revision).unwrap();
        let scratch = tempfile::tempdir().unwrap();
        let output = scratch.path().join("export");
        let handoff = export(handoff, &output).unwrap();
        let replay =
            consume_selected(producer, &output, "external-profile.json", "external-receipt.json", revision).unwrap();
        assert_eq!(replay.observation.association_identity, handoff.observation.association_identity);
        assert_eq!(replay.receipt, fs::read(fixture(name).join("receipt.json")).unwrap());
        assert_eq!(replay.observation.proof_acceptance, "not-evaluated-by-mantle");
        assert_eq!(replay.observation.receipt_identity, identity(handoff.receipt).identity);
        assert!(replay.observation.checked_materials.iter().any(|path| path.ends_with("llbc.llbc")));
    }
}

#[test]
fn stale_source_profile_tool_and_options_cannot_reuse_association() {
    let root = fixture("trellis");
    let profile = fs::read(root.join("profile.json")).unwrap();
    let expected = packaging_inputs(Producer::Trellis, profile, TRELLIS_REV).unwrap().inputs;
    for field in ["source", "profile", "tool", "options"] {
        let mut changed = expected.clone();
        match field {
            "source" => changed.source_identity = identity(b"changed source".to_vec()).identity,
            "profile" => changed.profile_identity = identity(b"changed profile".to_vec()).identity,
            "tool" => changed.tool_contract_identity = identity(b"changed tool contract".to_vec()).identity,
            "options" => changed.options_identity = identity(b"changed options".to_vec()).identity,
            _ => unreachable!(),
        }
        assert!(consume(Producer::Trellis, &root, "profile.json", "receipt.json", changed).is_err(), "{field}");
    }
}

#[test]
fn missing_tampered_and_escaped_payloads_fail_without_publication() {
    for mode in ["missing", "tampered", "escaped"] {
        let scratch = tempfile::tempdir().unwrap();
        copy_tree(&fixture("trellis"), scratch.path());
        let payload = scratch.path().join("aeneas-pilot/generated/Llbc.lean");
        match mode {
            "missing" => fs::remove_file(payload).unwrap(),
            "tampered" => fs::write(payload, b"changed generated model").unwrap(),
            "escaped" => {
                let receipt_path = scratch.path().join("receipt.json");
                let mut receipt: Value = serde_json::from_slice(&fs::read(&receipt_path).unwrap()).unwrap();
                receipt["artifacts"][0]["path"] = Value::String("../outside".into());
                fs::write(receipt_path, serde_json::to_vec(&receipt).unwrap()).unwrap();
            }
            _ => unreachable!(),
        }
        assert!(
            consume_selected(Producer::Trellis, scratch.path(), "profile.json", "receipt.json", TRELLIS_REV).is_err(),
            "{mode}"
        );
    }
}

#[test]
fn external_failure_and_unknown_versions_do_not_become_proof_acceptance() {
    for (pointer, replacement) in [
        ("/outcome", "failed"),
        ("/outcome", "blocked"),
        ("/schema_version", "trellis-aeneas-proof-receipt/v999"),
    ] {
        let scratch = tempfile::tempdir().unwrap();
        copy_tree(&fixture("trellis"), scratch.path());
        let path = scratch.path().join("receipt.json");
        let mut receipt: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        *receipt.pointer_mut(pointer).unwrap() = Value::String(replacement.into());
        fs::write(path, serde_json::to_vec(&receipt).unwrap()).unwrap();
        let error =
            match consume_selected(Producer::Trellis, scratch.path(), "profile.json", "receipt.json", TRELLIS_REV) {
                Ok(_) => panic!("external failure/version drift admitted"),
                Err(error) => error,
            };
        assert!(
            error.contains(if pointer == "/outcome" {
                "outcome mismatch"
            } else {
                "receipt schema mismatch"
            }),
            "{error}"
        );
    }
}

#[cfg(unix)]
#[test]
fn symbolic_link_substitution_is_not_a_payload() {
    let scratch = tempfile::tempdir().unwrap();
    copy_tree(&fixture("octet"), scratch.path());
    let path = scratch.path().join("proofs/aeneas/llbc.llbc");
    fs::remove_file(&path).unwrap();
    std::os::unix::fs::symlink(fixture("octet").join("proofs/aeneas/llbc.llbc"), &path).unwrap();
    let error = match consume_selected(Producer::Octet, scratch.path(), "profile.json", "receipt.json", OCTET_REV) {
        Ok(_) => panic!("symlink substitution admitted"),
        Err(error) => error,
    };
    assert!(error.contains("symlink artifact rejected"), "{error}");
}

#[test]
fn selected_fixture_cannot_stand_in_for_a_complete_owner_verification_bundle() {
    let scratch = tempfile::tempdir().unwrap();
    copy_tree(&fixture("octet"), scratch.path());
    assert!(
        consume_selected(Producer::OctetVerified, scratch.path(), "profile.json", "receipt.json", OCTET_REV).is_err(),
        "a selected-stage fixture without the owner's report was accepted"
    );
    for report in [
        serde_json::json!({"schema_version": 1, "status": "invalid", "diagnostics": ["external rejection"], "artifacts": []}),
        serde_json::json!({"schema_version": 999, "status": "valid", "diagnostics": [], "artifacts": []}),
        serde_json::json!({"schema_version": 1, "status": "valid", "diagnostics": [], "artifacts": []}),
    ] {
        fs::write(scratch.path().join("producer-artifact-verification.json"), serde_json::to_vec(&report).unwrap())
            .unwrap();
        assert!(
            consume_selected(Producer::OctetVerified, scratch.path(), "profile.json", "receipt.json", OCTET_REV)
                .is_err(),
            "an externally rejected, incompatible or unlinked report granted acceptance"
        );
    }
}
