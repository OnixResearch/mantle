# External release witness handoff blocker

Task-ID: external-release-witness-handoff
Covers: r[release_witness.external_handoff]
Date: 2026-07-02

## Completed partial step

I1 is complete: a public-only witness request was exported for the current provider-bound release evidence bundle.

Command (pueue task 203):

```text
ROOT=/home/brittonr/releases/mantle/provider-bound-release-evidence-2026-06-28-provider-remap-fixed
BUNDLE="$ROOT/release-evidence"
VERIFY="$ROOT/release-verification"
REQUEST="$ROOT/external-witness-request-2026-07-02"
target/debug/mantle release witness-export "$BUNDLE" --verification-dir "$VERIFY" --request-dir "$REQUEST"
```

Output:

```text
witness request: /home/brittonr/releases/mantle/provider-bound-release-evidence-2026-06-28-provider-remap-fixed/external-witness-request-2026-07-02
request metadata: /home/brittonr/releases/mantle/provider-bound-release-evidence-2026-06-28-provider-remap-fixed/external-witness-request-2026-07-02/request.json
release id: provider-bound-release-evidence-2026-06-28-provider-remap-fixed
layout version: 1
release bundle copy: /home/brittonr/releases/mantle/provider-bound-release-evidence-2026-06-28-provider-remap-fixed/external-witness-request-2026-07-02/release-evidence/provider-bound-release-evidence-2026-06-28-provider-remap-fixed
verification seed: /home/brittonr/releases/mantle/provider-bound-release-evidence-2026-06-28-provider-remap-fixed/external-witness-request-2026-07-02/release-verification/provider-bound-release-evidence-2026-06-28-provider-remap-fixed
```

Request metadata:

```json
{"schema":"mantle-witness-request-v1","request_layout_version":1,"release_id":"provider-bound-release-evidence-2026-06-28-provider-remap-fixed","release_bundle_relative_path":"release-evidence/provider-bound-release-evidence-2026-06-28-provider-remap-fixed","verification_seed_relative_path":"release-verification/provider-bound-release-evidence-2026-06-28-provider-remap-fixed"}
```

Deterministic request-directory digest (pueue task 217):

```text
# Canonical tar stream with sorted paths, owner/group 0, and mtime epoch 0, hashed with BLAKE3.
6397a738c4047ee421e3e7baa856019fe0b47bb71c01a055cfe04a6fb5bd726b  -
```

Private-state check (pueue task 222):

```text
public request private-state check: no policy.json, revocations.json, *.key, signing-key*, or *secret* files present
```

Release attestation digest from the provider-bound release note: `c16aad67e2981a178e739c52dc07617c90976e2c8d2a566659e008bdf9542880`.

## Blocker

I2, I3, V1, V2, and V3 remain blocked because no independently operated external witness has returned signed sidecars for this request. The durable release-verification directory currently contains only the provider-bound/local witness identity:

```text
/home/brittonr/releases/mantle/provider-bound-release-evidence-2026-06-28-provider-remap-fixed/release-verification/witnesses/britton-desktop-provider-witness.json
/home/brittonr/releases/mantle/provider-bound-release-evidence-2026-06-28-provider-remap-fixed/release-verification/witnesses/britton-desktop-provider-witness.json.sig
```

### Aspen1 attempt

The request was sent to `aspen1` and verified there with the same canonical request-directory digest:

```text
6397a738c4047ee421e3e7baa856019fe0b47bb71c01a055cfe04a6fb5bd726b  -
```

Remote witness metadata used for the attempt:

```text
host: aspen1
system: x86_64-linux
host class: nixos-26.11-aspen1-nofuse
witness identity: aspen1-external-witness
trusted public key: crunch-aspen1-1:E7hNinFk5f1UKce5eEe8YjTuJmWnlH5mkuav5XpIvjU=
```

The final aspen1 attempt used the exact release proof helper Rust toolchain (`nightly-2025-11-01`) and `CRUNCH_NO_FUSE=1` to avoid the earlier aspen1 FUSE descriptor failure. The provider fixed-point proof completed and matched the published provider binary digest, but the self-hosting proof converged to a different stage1/stage2 digest and `mantle release witness-rebuild` failed closed before writing witness sidecars.

Final failure (pueue task 177, summarized from copied audit artifacts):

```text
provider proof status: valid:aa55e64630390fbb1f1f2ab2102005b1e05ef07c8ed19325a00566631c626a20
expected binaries/02-stage2-mantle: 70f02150224073af2dfdabad5b697072a6399c361c8b448f0e17ee01431fc703
aspen1 rebuilt binaries.stage2: db24660988110725b52e25ff1c0fb4eb456c22209912ab81d3959ad7c6d77d57
aspen1 rebuilt binaries.stage1: db24660988110725b52e25ff1c0fb4eb456c22209912ab81d3959ad7c6d77d57
result: no witness attestation written; no signature sidecar written; nothing importable
```

Copied aspen1 public audit artifacts are stored outside the repo at:

```text
/home/brittonr/releases/mantle/provider-bound-release-evidence-2026-06-28-provider-remap-fixed/external-witness-aspen1-2026-07-02/
```

Artifact BLAKE3 digests (pueue task 209):

```text
788583a1e8b712f001e6dc428f6389b3694b98c3b73772b030a2ab3d197116f3  aspen1-trusted-key.txt
23e20c78040f78eaebe68414bed5b088184acfb29069632f336a8df1f7b97213  provider-fixed-point-meta.json
27f0435cfa508366121c81e9d9f1f046332d65e37ea1e8b871b51daf5c28deb0  self-hosting-summary.txt
1ae1def2fbae1ccef37449e791f3a6764fe5bc0a2cc2009548b05a24a6368955  witness-check.json
3410ce30cfbe63548b70d53f30ad8ddc6953d5ae349f107cd7f3847429032157  witness-rebuild-20251101-nofuse.stderr
9d8d6159c27da3a427b0805074437647e3b25ecda1ecfcbed9babdcca1ee22ea  witness-rebuild-audit-meta.json
```

Earlier aspen1 attempts also failed closed: the first full run used a stable Nix rustc and the proof helper rejected it (`nightly rustc required`); a second latest-nightly run required `CRUNCH_NO_FUSE=1` and then produced the same non-matching self-hosting digest. These are blockers, not witness evidence.

The existing release note already keeps the claim bounded: it says not to describe the result as external independent rebuild agreement unless a separate operator supplies/imports returned sidecars. No archive is valid until an external operator returns sidecars and the imported witness verifies as signature-valid, release-digest-matched, rebuilt-digest-matched, and policy-counted.

## Active-change validation

Cairn validation and gates after recording the aspen1 blocker (pueue task 222):

```text
## validate
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 16,
  "valid": true
}
## gate proposal
{
  "change": "external-release-witness-handoff",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
## gate design
{
  "change": "external-release-witness-handoff",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
## gate tasks
{
  "change": "external-release-witness-handoff",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```
