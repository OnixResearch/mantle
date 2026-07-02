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

- FUSE-enabled runs (pueue task 117) can see `/dev/fuse` and a Nix-provided `fusermount3`, but still fail during stage0 at `musl-seed-toolchain.drv` with `descriptor I/O error: No such file or directory (os error 2)`.
- No-FUSE runs (`CRUNCH_NO_FUSE=1`, pueue tasks 177 and 52) get through the full provider proof and self-hosting proof, but the self-hosting proof converges to a digest that does not match the published release binary.

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

Earlier aspen1 attempts also failed closed: the first full run used a stable Nix rustc and the proof helper rejected it (`nightly rustc required`); a second latest-nightly run required `CRUNCH_NO_FUSE=1` and then produced the same non-matching self-hosting digest. These are blockers, not witness evidence.

The existing release note already keeps the claim bounded: it says not to describe the result as external independent rebuild agreement unless a separate operator supplies/imports returned sidecars. No archive is valid until an external operator returns sidecars and the imported witness verifies as signature-valid, release-digest-matched, rebuilt-digest-matched, and policy-counted.

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
