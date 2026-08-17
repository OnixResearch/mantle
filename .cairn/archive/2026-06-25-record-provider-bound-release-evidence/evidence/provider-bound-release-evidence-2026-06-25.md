# Provider-bound release evidence transcript — 2026-06-25

Task-ID: I1, I2, I3, I4, V1
Covers: r[verification_evidence.provider_bound_release_evidence_transcripts]

## Generated release bundle

- Release id: `provider-bound-release-evidence-2026-06-25`
- Release bundle: `target/release-evidence/provider-bound-release-evidence-2026-06-25`
- Required verifier receipt: `target/release-evidence/verify-provider-bound-release-evidence-2026-06-25.json`
- Determinism summary JSON: `target/release-evidence/provider-bound-release-evidence-2026-06-25-determinism-summary.json`
- Determinism summary Markdown: `target/release-evidence/provider-bound-release-evidence-2026-06-25-determinism-summary.md`

The release manifest records two packaged binaries so both proof linkages are
checked:

- `binaries/01-mantle` — provider fixed-point stage binary, BLAKE3 `288023366ab86aafdc2384a113d208a295693b145567d757af56bfe6dafa9ec3`
- `binaries/02-stage2-mantle` — full self-hosting proof stage2 binary, BLAKE3 `84184eee1eeeeb10f357a097f25f984a0b753b711c01c2b0ca4655cd9bd8aacd`

## Deterministic release proof summary

Command evidence: pueue task 224 refreshed the summary after the verifier
receipt was regenerated.

```text
real release determinism summary written
  json: target/release-evidence/provider-bound-release-evidence-2026-06-25-determinism-summary.json
  markdown: target/release-evidence/provider-bound-release-evidence-2026-06-25-determinism-summary.md
  bounded claim: This artifact rebuilt twice from the recorded inputs under the recorded mantle-proof-sandbox-v1 profiles and the BLAKE3 digest sets matched. This is a bounded packaged-artifact proof, not a full-bootstrap reproducibility claim.
```

Summary values from
`target/release-evidence/provider-bound-release-evidence-2026-06-25-determinism-summary.md`:

- Proof verdict: `self-rebuild-match`
- Verify status: `eligible`
- Source BLAKE3: `56f50909309d8d16bb53eb57b25f8277677dc88f049393d329fb68df2e06549b`
- Provider/self-hosting proof bundle BLAKE3: `45901c49cd828f8bcd4d6d1681b630e5e97d1f2d4a0ac06bf36dd7d6c419fcfb`
- Deterministic proof BLAKE3: `cf90e08d6e6833e02357d8334c3f66f4bd16f55aa0c449dfd3174be5158e757b`
- Sandbox isolation evidence BLAKE3: `13c3525b25e58af6791ffc85a5f925f830249cf6bcce1c0041d74c1918132c23`
- Verify receipt BLAKE3: `20e32d1e395faede8bc78e9ace9574059d97d031a18205379053b961c9772682`

## Required verifier evidence

Command evidence: pueue task 220 reran the required verifier on the original
bundle with both gates enabled.

```sh
/home/brittonr/.cargo-target/debug/mantle --json release verify \
  target/release-evidence/provider-bound-release-evidence-2026-06-25 \
  --require-deterministic-release \
  --require-provider-fixed-point-proof \
  > target/release-evidence/verify-provider-bound-release-evidence-2026-06-25.json
```

Verifier output slice:

```text
"eligible":true
"release_artifact_digest_blake3":"288023366ab86aafdc2384a113d208a295693b145567d757af56bfe6dafa9ec3"
"release_artifact_relative_path":"binaries/01-mantle"
"status":"eligible"
"status":"valid"
"valid":true
```

Provider fixed-point binding recorded by the verifier:

- Provider proof status: `valid`
- Provider proof digest: `fe2cf79289372b1231ab95cf0d7161218ce6740f80e864b137858d20c679df59`
- Closure policy digest: `c242c98d018670073a465568a5a1957e48dda771e8c793cc9b0ebc203eb01093`
- Matched release artifact path: `binaries/01-mantle`
- Matched release artifact BLAKE3: `288023366ab86aafdc2384a113d208a295693b145567d757af56bfe6dafa9ec3`

## Receipt checker evidence

Command evidence: pueue task 222 reran the receipt checker after the verifier
receipt was regenerated.

```sh
cargo -Zscript scripts/check-real-release-determinism-receipt.rs \
  target/release-evidence/provider-bound-release-evidence-2026-06-25 \
  --verify-receipt target/release-evidence/verify-provider-bound-release-evidence-2026-06-25.json
```

Output:

```text
real release determinism proof receipt valid
  release: provider-bound-release-evidence-2026-06-25
  provider: legacy-fetch
  source BLAKE3: 56f50909309d8d16bb53eb57b25f8277677dc88f049393d329fb68df2e06549b
  proof bundle BLAKE3: 45901c49cd828f8bcd4d6d1681b630e5e97d1f2d4a0ac06bf36dd7d6c419fcfb
  artifact digests: [("binaries/01-mantle", "288023366ab86aafdc2384a113d208a295693b145567d757af56bfe6dafa9ec3"), ("binaries/02-stage2-mantle", "84184eee1eeeeb10f357a097f25f984a0b753b711c01c2b0ca4655cd9bd8aacd")]
  proof: target/release-evidence/provider-bound-release-evidence-2026-06-25/deterministic-release/deterministic-build-proof.json
  sandbox evidence: target/release-evidence/provider-bound-release-evidence-2026-06-25/deterministic-release/deterministic-sandbox-isolation-evidence.json
  verify: target/release-evidence/verify-provider-bound-release-evidence-2026-06-25.json
  bounded claim: packaged release artifacts rebuilt twice from recorded inputs under recorded mantle-proof-sandbox-v1: profiles with matching BLAKE3 digest sets; this is not a full-bootstrap reproducibility claim
```

The checker was updated in this change to accept multi-binary bundles while
still requiring the full self-hosting `proof_linkage.stage2_binary_digest_blake3`
to appear in `manifest.binaries`, and to require the provider fixed-point
verifier receipt to bind `stage_binary_digest_blake3` to the reported release
artifact path/digest.

## Portable replay from copied artifacts

Command evidence: pueue task 213 copied the release bundle into
`target/portable-provider-bound-release-replay-2026-06-25/positive/release-bundle`,
copied the current verifier binary to
`target/portable-provider-bound-release-replay-2026-06-25/positive/verifier-mantle`,
and verified the copied bundle with both gates enabled. The copied verifier is
used because the packaged provider fixed-point binary predates the verifier flag
added by the artifact-binding change; the next milestone below records the
current-code provider proof blocker.

Positive replay command:

```sh
./verifier-mantle --json release verify ./release-bundle \
  --require-deterministic-release \
  --require-provider-fixed-point-proof \
  | tee ./positive-verify.json
```

Positive replay output slice:

```text
"deterministic_release":{"blockers":[],"eligible":true,..."status":"eligible"}
"provider_fixed_point_proof":{"blockers":[],..."release_artifact_digest_blake3":"288023366ab86aafdc2384a113d208a295693b145567d757af56bfe6dafa9ec3","release_artifact_relative_path":"binaries/01-mantle",..."status":"valid","valid":true}
```

Negative replay removed
`target/portable-provider-bound-release-replay-2026-06-25/negative/release-bundle/deterministic-release/deterministic-build-proof.json`
and reran the same command from the copied negative bundle. It failed closed:

```text
{"error":"verifying deterministic_build_proof: error: metadata /home/brittonr/.cargo-target/repo-targets/mantle/portable-provider-bound-release-replay-2026-06-25/negative/./release-bundle/deterministic-release/deterministic-build-proof.json: No such file or directory (os error 2)","code":3,"kind":"internal"}
negative_exit_status=3
```

## Current-code provider fixed-point rerun blocker

To test whether the packaged provider binary could be regenerated from current
code and serve as its own verifier, pueue task 201 attempted a fresh
provider-backed fixed-point proof:

```sh
/home/brittonr/.cargo-target/debug/mantle --json self-build --cargo-free --fixed-point \
  --out /home/brittonr/git/mantle-source-built-rust-provider-fixed-point-release-bound-2026-06-25 \
  --rust-source-provider target/rust-source-provider-musl-host-route-patch-plan-rerun38-2026-06-25/provider-out \
  --toolchain-closure target/source-built-rust-provider-fixed-point-2026-06-25/native-toolchain-closure.json
```

It blocked in stage1 before producing a current-code provider binary:

```text
{"error":"stage1 blocked: topology execution status was blocked","code":1,"kind":"build"}
"class": "build-script-run-failed"
"message": "... aws-lc-sys ... ### COMPILER BUG DETECTED ### ... memcmp related bug reported in https://gcc.gnu.org/bugzilla/show_bug.cgi?id=95189 ..."
```

Next concrete milestone: teach the source-built provider/native topology path to
handle `aws-lc-sys`'s GCC PR95189 memcmp guard without hiding the compiler-risk
evidence, then rerun the provider-backed fixed point from current code and
package that current provider binary as the self-verifying release artifact.

## Bounded non-claims

This evidence only proves the recorded packaged artifact set and recorded proof
inputs for `provider-bound-release-evidence-2026-06-25`:

- It does not prove full bootstrap reproducibility.
- It does not prove compiler correctness.
- It does not prove deploy success.
- It does not prove full Cargo compatibility.
