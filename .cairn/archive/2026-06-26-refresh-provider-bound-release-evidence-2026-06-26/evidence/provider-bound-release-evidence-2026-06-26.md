# Provider-bound release evidence transcript — 2026-06-26

Task-ID: I1, I2, I3, I4, V1
Covers: r[verification_evidence.provider_bound_release_evidence_refresh_transcripts]

## Generated release bundle

Command evidence: pueue task 63 created the bundle.

```sh
/home/brittonr/.cargo-target/debug/mantle --json release create \
  --release-id provider-bound-release-evidence-2026-06-26 \
  --bundle-dir target/release-evidence/provider-bound-release-evidence-2026-06-26 \
  --binary /home/brittonr/git/mantle-source-built-rust-provider-fixed-point-native-static-pie-crt-2026-06-26-rerun4/stage2/mantle \
  --binary target/release-evidence/provider-bound-release-evidence-2026-06-25/proof/self-hosting/binaries/stage2-mantle \
  --proof-bundle target/release-evidence/provider-bound-release-evidence-2026-06-25/proof/self-hosting \
  --provider-fixed-point-proof /home/brittonr/git/mantle-source-built-rust-provider-fixed-point-native-static-pie-crt-2026-06-26-rerun4
```

Generated paths:

- Release id: `provider-bound-release-evidence-2026-06-26`
- Release bundle: `target/release-evidence/provider-bound-release-evidence-2026-06-26`
- Create receipt: `target/release-evidence/create-provider-bound-release-evidence-2026-06-26.json`
- Required verifier receipt: `target/release-evidence/verify-provider-bound-release-evidence-2026-06-26.json`
- Reproducibility report: `target/release-evidence/provider-bound-release-evidence-2026-06-26-rebuild/reproducibility-report.json`
- Reproduce receipt: `target/release-evidence/reproduce-provider-bound-release-evidence-2026-06-26.json`
- Portable replay: `target/portable-provider-bound-release-replay-2026-06-26/`

The release manifest records two packaged binaries so both proof linkages are checked:

- `binaries/01-mantle` — current-code provider fixed-point stage binary, BLAKE3 `b4fdeca80db000a4e003513417b665ae2a1014f6752ed9ff65bb3b6b57ce48f3`
- `binaries/02-stage2-mantle` — full self-hosting proof stage2 binary, BLAKE3 `84184eee1eeeeb10f357a097f25f984a0b753b711c01c2b0ca4655cd9bd8aacd`

Other manifest digests:

- Source archive BLAKE3: `3d77f8ce752879a0ab0f44e2a3918482f9080d500dc6b19f8ee760ef4fd5b5de`
- Full self-hosting proof bundle BLAKE3: `45901c49cd828f8bcd4d6d1681b630e5e97d1f2d4a0ac06bf36dd7d6c419fcfb`
- Provider fixed-point proof BLAKE3: `0c6208800b8f48317b3f10e78dcd0255bc84f95e656abb8740de93f3f63d6543`

## Deterministic release proof

Command evidence: pueue task 92 reran release reproducibility with two deterministic proof runs and attached deterministic proof artifacts to the bundle.

```sh
MANTLE_DETERMINISTIC_PROOF_BWRAP=/nix/store/gr9l6ql3wg70idpqlqhnfdx81hak22c8-bubblewrap-0.11.0/bin/bwrap \
/home/brittonr/.cargo-target/debug/mantle --json release reproduce \
  /home/brittonr/git/mantle/target/release-evidence/provider-bound-release-evidence-2026-06-26 \
  --rebuild-output-dir /home/brittonr/git/mantle/target/release-evidence/provider-bound-release-evidence-2026-06-26-rebuild \
  --rebuild-command /nix/store/4mdqc2snfiihr6r61ln1rqs4fis6br9b-busybox-static-x86_64-unknown-linux-musl-1.36.1/bin/busybox \
  --rebuild-arg sh \
  --rebuild-arg /home/brittonr/git/mantle/target/release-evidence/provider-bound-release-evidence-2026-06-26/rebuild-both-binaries.sh \
  --report-path /home/brittonr/git/mantle/target/release-evidence/provider-bound-release-evidence-2026-06-26-rebuild/reproducibility-report.json \
  --deterministic-proof-runs 2 \
  --deterministic-proof-dir /home/brittonr/git/mantle/target/release-evidence/provider-bound-release-evidence-2026-06-26-deterministic-proof
```

Reproduce output values:

```text
matched_count: 2
mismatched_count: 0
missing_count: 0
deterministic_proof_verdict: self-rebuild-match
deterministic_proof_digest_blake3: f5dcdb324bb053469ed0a4f23e36c14520d09c2c4508943151780436d1effd34
deterministic_sandbox_isolation_evidence_digest_blake3: 60df953db77b6cb984ccf1040dd17cd1f85b34ccd8a4c0daee8d887b25cd84be
reproducibility_report_digest_blake3: eea9f95f2bbfa4a3755b6fb6dbf78bfa59c37136af3dc406a820dd591f767d48
deterministic_proof_sandbox_profiles:
  - mantle-proof-sandbox-v1:00afb65ad321e12c12a9aacdbe0f97fd6233281d6928d5d7c32161d619cef9ae
  - mantle-proof-sandbox-v1:946105023a671f417fd7bf177c44bfe8219dfea4d6b5bbfb67749c25bbe7686d
```

## Required verifier evidence

Command evidence: pueue task 93 reran the required verifier on the bundle with both gates enabled.

```sh
/home/brittonr/.cargo-target/debug/mantle --json release verify \
  target/release-evidence/provider-bound-release-evidence-2026-06-26 \
  --require-deterministic-release \
  --require-provider-fixed-point-proof \
  > target/release-evidence/verify-provider-bound-release-evidence-2026-06-26.json
```

Verifier output slice:

```text
deterministic_release.status: eligible
deterministic_release.eligible: true
deterministic_release.proof_digest_blake3: f5dcdb324bb053469ed0a4f23e36c14520d09c2c4508943151780436d1effd34
deterministic_release.sandbox_isolation_evidence_digest_blake3: 60df953db77b6cb984ccf1040dd17cd1f85b34ccd8a4c0daee8d887b25cd84be
provider_fixed_point_proof.status: valid
provider_fixed_point_proof.valid: true
provider_fixed_point_proof.proof_artifact_digest_blake3: 0c6208800b8f48317b3f10e78dcd0255bc84f95e656abb8740de93f3f63d6543
provider_fixed_point_proof.closure_policy_digest_blake3: c242c98d018670073a465568a5a1957e48dda771e8c793cc9b0ebc203eb01093
provider_fixed_point_proof.release_artifact_relative_path: binaries/01-mantle
provider_fixed_point_proof.release_artifact_digest_blake3: b4fdeca80db000a4e003513417b665ae2a1014f6752ed9ff65bb3b6b57ce48f3
```

## Receipt checker evidence

Command evidence: pueue task 99 reran the receipt checker after the verifier receipt was regenerated.

```sh
cargo -Zscript scripts/check-real-release-determinism-receipt.rs \
  target/release-evidence/provider-bound-release-evidence-2026-06-26 \
  --verify-receipt target/release-evidence/verify-provider-bound-release-evidence-2026-06-26.json
```

Output:

```text
real release determinism proof receipt valid
  release: provider-bound-release-evidence-2026-06-26
  provider: legacy-fetch
  source BLAKE3: 3d77f8ce752879a0ab0f44e2a3918482f9080d500dc6b19f8ee760ef4fd5b5de
  proof bundle BLAKE3: 45901c49cd828f8bcd4d6d1681b630e5e97d1f2d4a0ac06bf36dd7d6c419fcfb
  artifact digests: [("binaries/01-mantle", "b4fdeca80db000a4e003513417b665ae2a1014f6752ed9ff65bb3b6b57ce48f3"), ("binaries/02-stage2-mantle", "84184eee1eeeeb10f357a097f25f984a0b753b711c01c2b0ca4655cd9bd8aacd")]
  proof: target/release-evidence/provider-bound-release-evidence-2026-06-26/deterministic-release/deterministic-build-proof.json
  sandbox evidence: target/release-evidence/provider-bound-release-evidence-2026-06-26/deterministic-release/deterministic-sandbox-isolation-evidence.json
  verify: target/release-evidence/verify-provider-bound-release-evidence-2026-06-26.json
  bounded claim: packaged release artifacts rebuilt twice from recorded inputs under recorded mantle-proof-sandbox-v1: profiles with matching BLAKE3 digest sets; this is not a full-bootstrap reproducibility claim
```

## Portable replay from copied artifacts

Command evidence: pueue task 104 copied the release bundle into `target/portable-provider-bound-release-replay-2026-06-26/positive/release-bundle`, copied the current verifier binary to `target/portable-provider-bound-release-replay-2026-06-26/positive/verifier-mantle`, and verified the copied bundle with both gates enabled.

Positive replay command:

```sh
./verifier-mantle --json release verify ./release-bundle \
  --require-deterministic-release \
  --require-provider-fixed-point-proof \
  > ./positive-verify.json
```

Positive replay output slice:

```text
deterministic_release.status: eligible
deterministic_release.eligible: true
provider_fixed_point_proof.status: valid
provider_fixed_point_proof.valid: true
provider_fixed_point_proof.release_artifact_relative_path: binaries/01-mantle
provider_fixed_point_proof.release_artifact_digest_blake3: b4fdeca80db000a4e003513417b665ae2a1014f6752ed9ff65bb3b6b57ce48f3
```

Negative replay removed `target/portable-provider-bound-release-replay-2026-06-26/negative/release-bundle/deterministic-release/deterministic-build-proof.json` and reran the same command from the copied negative bundle. It failed closed:

```text
negative_exit_status=3
{"error":"verifying deterministic_build_proof: error: metadata /home/brittonr/.cargo-target/repo-targets/mantle/portable-provider-bound-release-replay-2026-06-26/negative/./release-bundle/deterministic-release/deterministic-build-proof.json: No such file or directory (os error 2)","code":3,"kind":"internal"}
```

## Bounded non-claims

This evidence only proves the recorded packaged artifact set and recorded proof inputs for `provider-bound-release-evidence-2026-06-26`:

- It does not prove full bootstrap reproducibility.
- It does not prove compiler correctness.
- It does not prove deploy success.
- It does not prove full Cargo compatibility.
