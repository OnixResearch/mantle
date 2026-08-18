# Portable release verification replay

Date: 2026-06-25
Change: `portable-release-verification-replay`
Requirement: r[verification_evidence.portable_release_verification_replay]

## Scope and bounded claim

This transcript records a portable replay of release verification for release id
`provider-fixed-point-release-evidence-2026-06-25` from a copied artifact set,
not from the original release bundle path.

Bounded claim: the copied release bundle, copied deterministic build proof
receipt, copied deterministic sandbox isolation evidence, and bundle-local
provider fixed-point proof are sufficient for `mantle release verify` to reach
`deterministic_release.status = eligible`, `reproducibility_status = matched`,
and `provider_fixed_point_proof.status = valid`.

Non-claims:

- No source checkout cleanliness claim.
- No full bootstrap reproducibility claim.
- No compiler correctness claim.
- No full Cargo compatibility claim.
- No deploy success claim.

## Minimal portable artifact set

Scratch/export root prepared in pueue task 151:

- Scratch root: `/home/brittonr/git/mantle/target/portable-release-verification-replay-2026-06-25`
- Runtime-resolved positive replay cwd: `/home/brittonr/.cargo-target/repo-targets/mantle/portable-release-verification-replay-2026-06-25/positive`
- Copied release bundle: `positive/release-bundle`
- Copied deterministic proof receipt: `positive/deterministic-build-proof.json`
- Copied deterministic sandbox isolation evidence: `positive/deterministic-sandbox-isolation-evidence.json`

The release bundle was copied with `cp -al` into the scratch root to avoid
duplicating multi-GiB generated payloads while proving the verifier reads the
scratch/export paths. The deterministic proof sidecars were copied with `cp -a`.
All generated replay payloads remain under ignored `target/` paths; only this
transcript is tracked.

Preparation command (pueue task 151):

```sh
set -euo pipefail
SCRATCH="$PWD/target/portable-release-verification-replay-2026-06-25"
POS="$SCRATCH/positive"
mkdir -p "$POS"
cp -al "$PWD/target/release-evidence/provider-fixed-point-release-evidence-2026-06-25" "$POS/release-bundle"
cp -a "$PWD/target/release-rebuild/record-deterministic-release-evidence-rerun-2026-06-25-abs-proof/deterministic-build-proof.json" "$POS/deterministic-build-proof.json"
cp -a "$PWD/target/release-rebuild/record-deterministic-release-evidence-rerun-2026-06-25-abs-proof/deterministic-sandbox-isolation-evidence.json" "$POS/deterministic-sandbox-isolation-evidence.json"
printf 'scratch=%s\n' "$SCRATCH"
printf 'bundle=%s\n' "$POS/release-bundle"
printf 'proof=%s\n' "$POS/deterministic-build-proof.json"
printf 'sandbox=%s\n' "$POS/deterministic-sandbox-isolation-evidence.json"
```

Output:

```text
scratch=/home/brittonr/git/mantle/target/portable-release-verification-replay-2026-06-25
bundle=/home/brittonr/git/mantle/target/portable-release-verification-replay-2026-06-25/positive/release-bundle
proof=/home/brittonr/git/mantle/target/portable-release-verification-replay-2026-06-25/positive/deterministic-build-proof.json
sandbox=/home/brittonr/git/mantle/target/portable-release-verification-replay-2026-06-25/positive/deterministic-sandbox-isolation-evidence.json
```

## Positive replay

Command (pueue task 152, cwd `target/portable-release-verification-replay-2026-06-25/positive`):

```sh
./release-bundle/binaries/01-stage2-mantle --json release verify ./release-bundle --require-deterministic-release --deterministic-proof ./deterministic-build-proof.json --deterministic-sandbox-isolation-evidence ./deterministic-sandbox-isolation-evidence.json --require-provider-fixed-point-proof | tee ./positive-verify.json
```

Output:

```json
{"deterministic_release":{"blockers":[],"eligible":true,"proof_digest_blake3":"c2a131abe400a9e56e5e6507c6cc6a304a420f98ce7d4d9981f174f0c21c8773","proof_path":"/home/brittonr/.cargo-target/repo-targets/mantle/portable-release-verification-replay-2026-06-25/positive/./deterministic-build-proof.json","sandbox_isolation_evidence_digest_blake3":"7f1eb23561dcf7dc68ca7b9d2c268326cdfef068a1cbcf5034d4ff2ae655ba31","sandbox_isolation_evidence_path":"/home/brittonr/.cargo-target/repo-targets/mantle/portable-release-verification-replay-2026-06-25/positive/./deterministic-sandbox-isolation-evidence.json","status":"eligible"},"kind":"mantle-release-verify-v1","manifest":{"binaries":[{"digest_blake3":"84184eee1eeeeb10f357a097f25f984a0b753b711c01c2b0ca4655cd9bd8aacd","kind":"file","relative_path":"binaries/01-stage2-mantle","size_bytes":63289920}],"claim_scope":"packaged-integrity-evidence","prerequisite_inventory":{"digest_blake3":"3d6cd61c3bfd893ec271556fec7974cacae70a6d989cab21f9213f1dbd5ffadb","kind":"file","relative_path":"proof/inventory.md","size_bytes":14244},"proof_bundle":{"digest_blake3":"45901c49cd828f8bcd4d6d1681b630e5e97d1f2d4a0ac06bf36dd7d6c419fcfb","kind":"directory","relative_path":"proof/self-hosting","size_bytes":127242957},"proof_linkage":{"prerequisite_inventory_digest_blake3":"3d6cd61c3bfd893ec271556fec7974cacae70a6d989cab21f9213f1dbd5ffadb","proof_bundle_schema":"mantle-self-hosting-proof-v2","proof_manifest_digest_blake3":"721443ada020a5845d12883f0655e4b42cd574362620e17e262d0f1949a0c499","proof_mode":"fixed-point","release_id":"provider-fixed-point-release-evidence-2026-06-25","selected_provider_kind":"legacy-fetch","source_archive_digest_blake3":"e831802092c14e9d5cc3c53d5f40accc3ba0edff977b63e5d05678bbdae72481","stage2_binary_digest_blake3":"84184eee1eeeeb10f357a097f25f984a0b753b711c01c2b0ca4655cd9bd8aacd","staged_source":"/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmprM89kH/store/brnjbzjaazy2j8kk5czadfs6yv1gnlhn-mantle-src"},"provider_fixed_point_proof":{"digest_blake3":"fe2cf79289372b1231ab95cf0d7161218ce6740f80e864b137858d20c679df59","evidence_role":"cargo-free-source-built-handoff-evidence","kind":"directory","relative_path":"proof/provider-fixed-point","size_bytes":2790890830},"release_id":"provider-fixed-point-release-evidence-2026-06-25","schema":"mantle-release-evidence-v1","source_archive":{"digest_blake3":"e831802092c14e9d5cc3c53d5f40accc3ba0edff977b63e5d05678bbdae72481","kind":"file","relative_path":"source/.tmp4us1I2","size_bytes":935829504},"workflow":{"command":"./scripts/prove-self-hosting.sh","version":"mantle-self-hosting-proof-v2"}},"provider_fixed_point_proof":{"blockers":[],"bounded_evidence_role":"cargo-free-source-built-handoff-evidence","closure_policy_digest_blake3":"c242c98d018670073a465568a5a1957e48dda771e8c793cc9b0ebc203eb01093","meta_digest_blake3":"6c575595b3bfe58fb464c99f21621b6ff2fff208283f1d314e9bda61b2c0025d","non_claims":["not-crunch-bootstrap","not-release-reproducibility","not-full-cargo-compatibility"],"proof_artifact_digest_blake3":"fe2cf79289372b1231ab95cf0d7161218ce6740f80e864b137858d20c679df59","proof_dir":"/home/brittonr/.cargo-target/repo-targets/mantle/portable-release-verification-replay-2026-06-25/positive/./release-bundle/proof/provider-fixed-point","proof_source":"bundled","stage1_unit_count":686,"stage2_unit_count":686,"stage_binary_digest_blake3":"288023366ab86aafdc2384a113d208a295693b145567d757af56bfe6dafa9ec3","status":"valid","valid":true},"release_id":"provider-fixed-point-release-evidence-2026-06-25","reproducibility_report":{"digest_blake3":"54aa592211670ee2ffae57122112925248fafa8333862ba1eeb93d97765b0bb5","path":"/home/brittonr/.cargo-target/repo-targets/mantle/portable-release-verification-replay-2026-06-25/positive/./release-bundle/reproducibility/reproducibility-report.json"},"reproducibility_status":"matched"}
```

Positive status summary:

- `deterministic_release.status`: `eligible`
- `deterministic_release.proof_digest_blake3`: `c2a131abe400a9e56e5e6507c6cc6a304a420f98ce7d4d9981f174f0c21c8773`
- `deterministic_release.sandbox_isolation_evidence_digest_blake3`: `7f1eb23561dcf7dc68ca7b9d2c268326cdfef068a1cbcf5034d4ff2ae655ba31`
- `provider_fixed_point_proof.status`: `valid`
- `provider_fixed_point_proof.proof_source`: `bundled`
- `reproducibility_status`: `matched`

## Negative replay: missing deterministic proof receipt

Command (pueue task 153, cwd `target/portable-release-verification-replay-2026-06-25/positive`):

```sh
set +e
./release-bundle/binaries/01-stage2-mantle --json release verify ./release-bundle --require-deterministic-release --deterministic-proof ./missing-deterministic-build-proof.json --deterministic-sandbox-isolation-evidence ./deterministic-sandbox-isolation-evidence.json --require-provider-fixed-point-proof > ./negative-missing-proof.txt 2>&1
status=$?
cat ./negative-missing-proof.txt
echo "negative_exit_status=$status"
if [ "$status" -eq 0 ]; then
  echo "negative replay unexpectedly succeeded" >&2
  exit 1
fi
exit 0
```

Output:

```text
{"error":"reading /home/brittonr/.cargo-target/repo-targets/mantle/portable-release-verification-replay-2026-06-25/positive/./missing-deterministic-build-proof.json: No such file or directory (os error 2)","code":3,"kind":"internal"}
negative_exit_status=3
```

Negative status summary: the required deterministic proof receipt was withheld,
`mantle release verify --require-deterministic-release` exited with status 3,
and no deterministic eligibility result was emitted or downgraded to a weaker
claim.

## Final validation

### Cairn validation

Command (pueue task 155):

```sh
nix run path:/home/brittonr/git/cairn#cairn -- validate --root $PWD
```

Output:

```json
{
  "change_issues": [],
  "changes": 2,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 8,
  "valid": true
}
```

### Task gate

Command (pueue task 156):

```sh
nix run path:/home/brittonr/git/cairn#cairn -- gate tasks portable-release-verification-replay --root $PWD
```

Output excerpt:

```json
{
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

### Tracked status

Command (pueue task 157):

```sh
git status --short --branch
```

Output:

```text
## main...origin/main [ahead 1]
 M cairn/changes/portable-release-verification-replay/tasks.md
?? cairn/changes/portable-release-verification-replay/evidence/
```

## Post-archive validation

### Cairn validation after archive execution

Command (pueue task 164):

```sh
nix run path:/home/brittonr/git/cairn#cairn -- validate --root $PWD
```

Output:

```json
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

### Tracked status after archive execution

Command (pueue task 165):

```sh
git status --short --branch
```

Output:

```text
## main...origin/main [ahead 1]
 D cairn/changes/portable-release-verification-replay/design.md
 D cairn/changes/portable-release-verification-replay/proposal.md
 D cairn/changes/portable-release-verification-replay/specs/verification-evidence/spec.md
 D cairn/changes/portable-release-verification-replay/tasks.md
 M cairn/specs/verification-evidence/spec.md
?? cairn/archive/2026-06-25-portable-release-verification-replay/
```
