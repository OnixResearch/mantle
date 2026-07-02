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

The aspen1 attempts all failed closed before writing witness sidecars. The useful split is:

- Non-unshare FUSE runs (pueue task 117) can see `/dev/fuse` and a Nix-provided `fusermount3`, but still fail during stage0 at `musl-seed-toolchain.drv` with `descriptor I/O error: No such file or directory (os error 2)`.
- A fresh full run under `unshare --user --map-root-user --mount --fork` got through provider proof and self-hosting with direct FUSE, but the self-hosting proof converged to a different digest than the published release binary.
- No-FUSE runs (`CRUNCH_NO_FUSE=1`, pueue tasks 177 and 52) get through the full provider proof and self-hosting proof, but the self-hosting proof also converges to a digest that does not match the published release binary.

Best no-FUSE failure, using the exact release proof helper Rust toolchain (`nightly-2025-11-01`) and then rerun with that toolchain installed at the default `/home/brittonr/.rustup` path:

```text
provider proof status: valid:aa55e64630390fbb1f1f2ab2102005b1e05ef07c8ed19325a00566631c626a20
expected binaries/02-stage2-mantle: 70f02150224073af2dfdabad5b697072a6399c361c8b448f0e17ee01431fc703
aspen1 rebuilt binaries.stage2: db24660988110725b52e25ff1c0fb4eb456c22209912ab81d3959ad7c6d77d57
aspen1 rebuilt binaries.stage1: db24660988110725b52e25ff1c0fb4eb456c22209912ab81d3959ad7c6d77d57
result: no witness attestation written; no signature sidecar written; nothing importable
```

Cross-checks from the successful provider-bound local witness and the aspen1 no-FUSE proof show both staged the same packaged source (`sk8fl4g2rk62qyq8g0z1sbmvr8lrzy1c-mantle-src`) and built the same bootstrap bwrap/busybox digests. Comparing the matching local stage2 binary to the aspen1 no-FUSE stage2 binary found the visible embedded string difference in Cargo's generated Nickel parser output path:

```text
local:  /tmp/cargo-target/x86_64-unknown-linux-musl/release/build/nickel-lang-parser-30857036f500e0c6/out/grammar.rs
aspen1: /tmp/cargo-target/x86_64-unknown-linux-musl/release/build/nickel-lang-parser-3f9e8346f23f6f7d/out/grammar.rs
```

This is useful diagnostic evidence, but not a release witness: the published release attestation requires digest `70f02150...`, and aspen1 only produced `db246609...` under its no-FUSE fallback.

Copied aspen1 public audit artifacts are stored outside the repo at:

```text
/home/brittonr/releases/mantle/provider-bound-release-evidence-2026-06-28-provider-remap-fixed/external-witness-aspen1-2026-07-02/
```

Artifact BLAKE3 digests (pueue tasks 209 and 172):

```text
788583a1e8b712f001e6dc428f6389b3694b98c3b73772b030a2ab3d197116f3  aspen1-trusted-key.txt
23e20c78040f78eaebe68414bed5b088184acfb29069632f336a8df1f7b97213  provider-fixed-point-meta.json
27f0435cfa508366121c81e9d9f1f046332d65e37ea1e8b871b51daf5c28deb0  self-hosting-summary.txt
1ae1def2fbae1ccef37449e791f3a6764fe5bc0a2cc2009548b05a24a6368955  witness-check.json
3410ce30cfbe63548b70d53f30ad8ddc6953d5ae349f107cd7f3847429032157  witness-rebuild-20251101-nofuse.stderr
9d8d6159c27da3a427b0805074437647e3b25ecda1ecfcbed9babdcca1ee22ea  witness-rebuild-audit-meta.json
60a4ed779f7705ff92e4e473a2f4f2b878bf342edd5a9e408b9e05ac693137a2  default-rustup-fuse/witness-rebuild-audit-meta.json
3f420a9cf0d3ab37d571f3fef8aa50f54053ee525d3766405a17431fa8c2203b  default-rustup-fuse/witness-rebuild-default-rustup-fuse.stderr
21fb7eae3c46eb5e1faed74ddeab42c93d37a6096259795c4285b2a80183e580  default-rustup-fuse/witness-rebuild-stderr.txt
715840521b75a58c4b5f6375bb4d7474ed42658a98f627f6cd5fde29d2881265  default-rustup-fuse/witness-rebuild-stdout.txt
86b81a7a90330bb8e8646f73783888beadb878970b2b8c02f0d0cebedd24b582  default-rustup-nofuse/self-hosting-manifest.json
fb25f4aea66fec65ab1003d7c303108562d0b34d35e4d871fd329199901f3ba0  default-rustup-nofuse/self-hosting-summary.txt
9e46d0f4d8c459089bf3fa6f7168f325dd912ee3b1a88adddf6cea279e527fb0  default-rustup-nofuse/witness-rebuild-audit-meta.json
3410ce30cfbe63548b70d53f30ad8ddc6953d5ae349f107cd7f3847429032157  default-rustup-nofuse/witness-rebuild-default-rustup-nofuse.stderr
```

Fresh full FUSE run under user/mount unshare (pueue task 244) used `CRUNCH_PROOF_RUSTUP_TOOLCHAIN=nightly-2025-11-01`, provider-bound inputs, and no `CRUNCH_NO_FUSE`. It failed closed after the workflow because the rebuilt stage2 binary still did not match the published release attestation digest:

```text
mode: unshare-user-map-root-mount-fork
provider proof status: valid:aa55e64630390fbb1f1f2ab2102005b1e05ef07c8ed19325a00566631c626a20
expected binaries/02-stage2-mantle: 70f02150224073af2dfdabad5b697072a6399c361c8b448f0e17ee01431fc703
aspen1 unshare-FUSE binaries.stage2: bbaf4413111a5e0d9b9a751ee662ace330313cd1a4e96fc6d9bcd12cf353ca47
aspen1 unshare-FUSE binaries.stage1: bbaf4413111a5e0d9b9a751ee662ace330313cd1a4e96fc6d9bcd12cf353ca47
self-hosting fixed-point: stage1_equals_stage2=true
stage2 hermeticity: strict
stage2 fallback events: []
release-verification witnesses: missing witnesses dir
result: no witness attestation written; no signature sidecar written; nothing importable
```

The unshare run proves the FUSE setup progressed farther than the earlier non-unshare failure: stage0 and stage2 logs contain `FUSE INIT major 7 minor 45`. It still is not release witness evidence because the proof artifact for `binaries/02-stage2-mantle` has digest `bbaf4413...`, not the required `70f02150...`.

Copied public audit artifacts for this fresh unshare run are stored outside the repo at:

```text
/home/brittonr/releases/mantle/provider-bound-release-evidence-2026-06-28-provider-remap-fixed/external-witness-aspen1-2026-07-02/unshare-fuse-full2/
```

BLAKE3 digests for the copied unshare-run evidence (pueue tasks 255 and 259):

```text
a3992ef18defd9de1370a9a05d20dcd5c7cc0de31f570427c5edbae13b2f25f4  proof-bundle/manifest.json
c2e142dd1777494d8ac54e475926fbae2fa88a7762778a95355ca2b29eb618cd  proof-bundle/summary.txt
23e0dfa96ba97d30ef3d82454ec73fc3eca922b73e288d90550cd8d84c09da40  provider-fixed-point-proof/meta.json
4f1050ebd87a6d9d3d682e0773184664f7cc912db794b7677993517a544d9edc  witness-rebuild-audit/meta.json
6c61b9973c80052ee41e30a6eca958c486f4910595beea89d8257267dc065cff  stderr.txt
8703411acc8de21a941655e56a806d7c6fc7ab0a92c4c8f018fefb30afbe2cf5  status.txt
6236f8b44a6317b7f5da4a430951a702e63a340deaa4d8177bb8110144b479b2  BLAKE3SUMS
```

Earlier aspen1 attempts also failed closed: the first full run used a stable Nix rustc and the proof helper rejected it (`nightly rustc required`); a second latest-nightly run required `CRUNCH_NO_FUSE=1` and then produced the same non-matching self-hosting digest. These are blockers, not witness evidence.

The existing release note already keeps the claim bounded: it says not to describe the result as external independent rebuild agreement unless a separate operator supplies/imports returned sidecars. No archive is valid until an external operator returns sidecars and the imported witness verifies as signature-valid, release-digest-matched, rebuilt-digest-matched, and policy-counted.

### Root cause for the Aspen digest mismatch

Follow-up comparison of the published proof's staged source tree against the release source archive explains why Aspen cannot reproduce the published `70f02150...` digest from the public request alone. The original proof staged source path was:

```text
/home/brittonr/.cargo-target/repo-targets/mantle/self-hosting-provider-remap-order-fix-work/tmp/.tmphrmiye/store/bya49al2cav0cddpayw7xvfqlhl32q0m-mantle-src
```

The release archive extracts to the same allowlisted source set except for one file under `vendor/.pi/`, which `release_source_path_is_releasable()` excludes but `self_build::copy_selected_source_tree()` had copied by recursively staging the whole `vendor` directory:

```text
orig staged source files: 40776
archive allowlist files: 40775

files in orig staged source but not release archive allowlist (first 200):
vendor/.pi/prompt-history.jsonl
```

Stray-file digest evidence from pueue task 356:

```text
/home/brittonr/.cargo-target/repo-targets/mantle/self-hosting-provider-remap-order-fix-work/tmp/.tmphrmiye/store/bya49al2cav0cddpayw7xvfqlhl32q0m-mantle-src/vendor/.pi/prompt-history.jsonl present size=143 bytes
f1a3fc65440b6be92677fecdfe7ed069c52b8851981f0b8c07e6fb1d1d30a860  /home/brittonr/.cargo-target/repo-targets/mantle/self-hosting-provider-remap-order-fix-work/tmp/.tmphrmiye/store/bya49al2cav0cddpayw7xvfqlhl32q0m-mantle-src/vendor/.pi/prompt-history.jsonl
/tmp/mantle-release-source-compare/src/vendor/.pi/prompt-history.jsonl absent
/home/brittonr/git/mantle/vendor/.pi/prompt-history.jsonl present size=143 bytes
f1a3fc65440b6be92677fecdfe7ed069c52b8851981f0b8c07e6fb1d1d30a860  /home/brittonr/git/mantle/vendor/.pi/prompt-history.jsonl
```

This makes the current provider-bound release bundle internally insufficient for external witness reproduction: the signed release digest was produced from a staged source tree that contains a private `.pi` file not present in the public release source archive or exported witness request. The honest next action is to fix the staging/archive policy mismatch, create fresh release evidence from a clean source tree, and export a new external witness request. The Aspen attempts remain valid blocker evidence but still do not produce importable witness sidecars for this release.

## Active-change validation

Cairn validation, gates, and whitespace check after recording the aspen1 FUSE/no-FUSE blockers (pueue task 183):

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

Cairn validation, gates, and whitespace check after recording the fresh unshare-FUSE blocker (pueue task 265):

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
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "43e7764744b9c289b831a7e68c34b4e5922f0fa153e2628a485882299b31177a",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "918aad018cb0262116e438054e008fcbd968debe3398b374caea57cd8fdc7c51",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
## gate design
{
  "change": "external-release-witness-handoff",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "00d51e9c60e0ee6c6c96535e5646b2de2dd1bcc58ffe8378b174711f00d1de0e",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "baeeadfa3b928492e8688460759bf963d6e1e21ad1fc81846c63e52adff2c680",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
## gate tasks
{
  "change": "external-release-witness-handoff",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "1c64ab0b0ab2907c523d9d64a9a9e4a1a11a4201fe1607a042970dced3455123",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "00c4ccc9a724435fd7a88ec56869e47b6da44208f8ef3c05de38e589c28f7ff1",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
## git diff --check
```

Cairn validation, gates, and whitespace check after recording the source-tree root cause (pueue task 364):

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
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "43e7764744b9c289b831a7e68c34b4e5922f0fa153e2628a485882299b31177a",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "918aad018cb0262116e438054e008fcbd968debe3398b374caea57cd8fdc7c51",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
## gate design
{
  "change": "external-release-witness-handoff",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "00d51e9c60e0ee6c6c96535e5646b2de2dd1bcc58ffe8378b174711f00d1de0e",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "baeeadfa3b928492e8688460759bf963d6e1e21ad1fc81846c63e52adff2c680",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
## gate tasks
{
  "change": "external-release-witness-handoff",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "1c64ab0b0ab2907c523d9d64a9a9e4a1a11a4201fe1607a042970dced3455123",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "00c4ccc9a724435fd7a88ec56869e47b6da44208f8ef3c05de38e589c28f7ff1",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
## git diff --check
```
