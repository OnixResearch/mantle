# Two-output witness rebuild validation

Date: 2026-06-26
Change: `witness-rebuild-two-output-release`

## Claim boundary

This evidence proves Mantle can parse multi-output proof bundle manifests, bind rebuilt artifacts to published release outputs by BLAKE3 digest, reject path-escaping proof artifacts, preserve legacy one-output proof manifests, and fail closed before signing when the real provider-bound request lacks a rebuilt proof artifact for the provider binary digest.

This evidence does **not** claim independent witness agreement for `provider-bound-release-evidence-2026-06-26`. The full checked-in witness helper rerun did not complete a proof; it failed during stage0 source fetching before rebuilt-output binding.

## Focused bin tests

Command:

```text
cargo test -p mantle --bin mantle witness_rebuild -- --nocapture
```

Output excerpt:

```text
test witness_rebuild::tests::validate_supported_workflow_identity_rejects_unknown_pair ... ok
test witness_rebuild::tests::workflow_args_preserve_non_nix_proof_mode_and_reject_unknown_modes ... ok
test witness_rebuild::tests::validate_existing_scratch_root_rejects_file_helper_owned_cargo_target_entry ... ok
test witness_rebuild::tests::collect_rebuilt_output_paths_rejects_escaping_proof_binary_path ... ok
test witness_rebuild::tests::collect_rebuilt_output_paths_rejects_absolute_proof_binary_path ... ok
test witness_rebuild::tests::validate_existing_scratch_root_allows_helper_owned_dirs_only ... ok
test witness_rebuild::tests::validate_existing_scratch_root_rejects_symlink_root ... ok
test witness_rebuild::tests::collect_rebuilt_output_paths_rejects_missing_expected_digest ... ok
test witness_rebuild::tests::validate_existing_scratch_root_rejects_symlinked_helper_owned_tmp_entry ... ok
test witness_rebuild::tests::validate_existing_scratch_root_rejects_unexpected_entries ... ok
test witness_rebuild::tests::collect_rebuilt_output_paths_preserves_one_output_legacy_manifest ... ok
test witness_rebuild::tests::collect_rebuilt_output_paths_maps_two_expected_outputs_by_digest ... ok

test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 859 filtered out; finished in 0.01s
```

## Release CLI witness tests

Command:

```text
cargo test -p mantle --test release_cli witness_rebuild -- --nocapture
```

Output excerpt:

```text
running 10 tests
test witness_rebuild_cli_rejects_request_paths_that_escape_request_dir ... ok
test witness_rebuild_cli_rejects_symlinked_helper_owned_scratch_entry_before_launching_driver ... ok
test witness_rebuild_cli_rejects_published_output_name_mismatch_before_running_driver ... ok
test witness_rebuild_cli_rejects_unsupported_workflow_before_running_driver ... ok
test witness_rebuild_cli_rejects_symlinked_scratch_root_before_launching_driver ... ok
test witness_rebuild_cli_check_is_preflight_only ... ok
test witness_rebuild_cli_rejects_unmatched_rebuilt_output_digest ... ok
test witness_rebuild_helper_check_is_preflight_only ... ok
test witness_rebuild_cli_happy_path_writes_sidecars_and_audit ... ok
test witness_rebuild_helper_happy_path_writes_sidecars_and_audit ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 89 filtered out; finished in 0.39s
```

## Real provider-bound request replay

The replay used the prior successful proof bundle only as a deterministic workflow output fixture. It did not perform a fresh independent rebuild.

Command summary:

```text
CRUNCH_WITNESS_REBUILD_DRIVER=<replay-proof-bundle-driver.sh> \
MANTLE_REPLAY_PROOF_BUNDLE_SOURCE=target/release-witness-requests/provider-bound-release-evidence-2026-06-26.work/proof-bundle \
  /home/brittonr/.cargo-target/debug/mantle release witness-rebuild \
  target/release-witness-requests/provider-bound-release-evidence-2026-06-26 \
  --scratch-dir target/release-witness-requests/provider-bound-release-evidence-2026-06-26.two-output-replay.work \
  --identity provider-bound-witness-2026-06-26 \
  --system x86_64-linux \
  --toolchain rust-1.91.1 \
  --host-class nixos-25.05
```

Output excerpt:

```text
replay exit status: 1
stdout=/home/brittonr/git/mantle/target/release-verification/provider-bound-release-evidence-2026-06-26/witness-replay-two-output-fix/replay.stdout
stderr=/home/brittonr/git/mantle/target/release-verification/provider-bound-release-evidence-2026-06-26/witness-replay-two-output-fix/replay.stderr
audit=/home/brittonr/git/mantle/target/release-witness-requests/provider-bound-release-evidence-2026-06-26.two-output-replay.work/witness-rebuild-audit/meta.json
error: build failed
witness rebuild workflow did not produce proof artifact matching binaries/01-mantle digest b4fdeca80db000a4e003513417b665ae2a1014f6752ed9ff65bb3b6b57ce48f3; available proof digests: binaries.stage2=cadfc8d78320bba63ff47a511df86b76966b3eb6f878a1d384c6201184d0b54b, binaries.stage1=cadfc8d78320bba63ff47a511df86b76966b3eb6f878a1d384c6201184d0b54b
replay failed closed on the provider binary digest gap
```

Audit excerpt:

```text
"status": "failed",
"rebuilt_outputs": [],
"witness_attestation_path": null,
"witness_signature_path": null,
"failure_message": "witness rebuild workflow did not produce proof artifact matching binaries/01-mantle digest b4fdeca80db000a4e003513417b665ae2a1014f6752ed9ff65bb3b6b57ce48f3; available proof digests: binaries.stage2=cadfc8d78320bba63ff47a511df86b76966b3eb6f878a1d384c6201184d0b54b, binaries.stage1=cadfc8d78320bba63ff47a511df86b76966b3eb6f878a1d384c6201184d0b54b"
```

## Full helper rerun blocker

Command summary:

```text
CRUNCH_CONFIG_DIR=target/release-signing/provider-bound-release-evidence-2026-06-26/witness-two-output-fix \
CRUNCH_WITNESS_REBUILD_CLI_BIN=/home/brittonr/.cargo-target/debug/mantle \
  ./scripts/rebuild-witness-request.sh \
  --scratch-dir target/release-witness-requests/provider-bound-release-evidence-2026-06-26.two-output-fix.work \
  target/release-witness-requests/provider-bound-release-evidence-2026-06-26 \
  --identity provider-bound-witness-2026-06-26 \
  --system x86_64-linux \
  --toolchain rust-1.91.1 \
  --host-class nixos-25.05
```

Output/blocker excerpt:

```text
status: failed
message: dependency gcc.drv failed
stage0 failed (exit 1)
store error: build: HTTP fetch failed for https://ftpmirror.gnu.org/gmp/gmp-6.2.1.tar.bz2: io: Network is unreachable (os error 101)
```

Audit path:

```text
target/release-witness-requests/provider-bound-release-evidence-2026-06-26.two-output-fix.work/witness-rebuild-audit/meta.json
```

## Build and lifecycle validation

Command:

```text
cargo build -p mantle --bin mantle
git diff --check
nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
```

Output excerpt:

```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 16.30s
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 7,
  "valid": true
}
```

## Post-archive validation

Command:

```text
nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
```

Output:

```text
{
  "change_issues": [],
  "changes": 0,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 6,
  "valid": true
}
```

## Post-archive traceability bridge check

Tracey coverage still has broader pre-existing missing groups, so this is not a global Tracey-green claim. The check below verifies the new accepted requirement is no longer present in Tracey's missing output after adding the bridge reference.

Command/output excerpt:

```text
tracey exit status: 1
new requirement not present in tracey missing output

$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 0,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 6,
  "valid": true
}
```
