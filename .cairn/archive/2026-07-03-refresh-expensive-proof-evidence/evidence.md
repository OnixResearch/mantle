# Evidence: refresh-expensive-proof-evidence

## Scope

Task-IDs: I1, I2, I3, I4, V1, V2, V3, V4
Covers: r[verification_evidence.expensive_proof_evidence_refresh]

## Refreshed proof command

### I1/I2/I4/V1/V3 — current expensive proof refresh

Command (pueue task 48):

```text
set -euo pipefail
export PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:/run/wrappers/bin:/run/current-system/sw/bin:$PATH
export PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig
export SNIX_BUILD_SANDBOX_SHELL=/bin/sh
OUT=/tmp/mantle-cargo-free-fixed-point-classifier2-20260703T203015Z
echo "OUT=$OUT"
/home/brittonr/.cargo-target/debug/mantle self-build --cargo-free --fixed-point --out "$OUT" --rustc /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustc
```

Output:

```text
OUT=/tmp/mantle-cargo-free-fixed-point-classifier2-20260703T203015Z
Cargo-free fixed-point: blocked
bundle: /tmp/mantle-cargo-free-fixed-point-classifier2-20260703T203015Z
error: build failed
stage1 blocked: topology execution status was blocked; classification=topology-level-blocker; topology-level blocker class native-host-unit-graph-blocked; nested /rust_plan/native_registry_source_planning/blockers/0 class vendor-checksum-mismatch: vendor package checksum does not match Cargo.lock checksum material; package=registry+https://github.com/rust-lang/crates.io-index#astral-tokio-tar@0.6.3; blocker_class=vendor-checksum-mismatch
```

Verdict: blocked. This is current frontier evidence, not a successful Cargo-free or Nix-free fixed-point claim.

Bundle path: `/tmp/mantle-cargo-free-fixed-point-classifier2-20260703T203015Z`.

Relevant digest extraction (pueue task 49):

```json
{"topology_receipt_hash":"11d5c390fe79fbc1227079c4275baefeceb2d941cc0d0985c2f6c973fc8a2660","rust_plan_receipt_hash":"1bdc412bce65980d883057db2b67a2a947d08cb1ed1f21591260998b9f0e3768","registry_digest":"ba666c4b9c177ab3e64fe2b8818ed56e5fe0ed1b5577b3375fb6019ff6e76214","source_closure_digest":"54592ec9e2451950ef9f2be10dac168d2c9821ccc8db38006548ba2ddcd40b37","stage_binary_digests":"none; stage1 blocked before binary"}
```

The proof produced no stage binary BLAKE3 digests because stage1 blocked before binary materialization.

Current blocker and next action:

- Blocker class: `vendor-checksum-mismatch`.
- Blocked package: `registry+https://github.com/rust-lang/crates.io-index#astral-tokio-tar@0.6.3`.
- Receipt context: nested `/rust_plan/native_registry_source_planning/blockers/0`; native registry source planning remains blocked before topology units run.
- Next action: refresh or repair checked-in `vendor-deps/` / declared source material against `Cargo.lock`, then rerun `mantle self-build --cargo-free --fixed-point`.
- Follow-on status: no new implementation-owned Cairn change was created because the current blocker is repository input/materialization drift, not a native topology executor defect.

## Docs/status updates

### I3/V2/V3 — current evidence and stale-claim guard

Updated files:

- `docs/operator-proof-guide.md` now has `Current refreshed evidence (2026-07-03)` with the command, output bundle, blocked verdict, digest summary, blocker class, package, and next action.
- `README.md` now narrows Cargo-free topology status to the current `vendor-checksum-mismatch` blocker and points operators to the guide before reporting status.
- `scripts/check-operator-proof-guide.rs` now requires the current evidence fragments and self-tests stale current blocker wording plus missing cargo-free bundle paths.

Guide guard and negative stale/overbroad self-test (pueue task 50):

```text
cargo -Zscript scripts/check-operator-proof-guide.rs
cargo -Zscript scripts/check-operator-proof-guide.rs --self-test
```

Output:

```text
operator proof guide drift check passed
operator proof guide checker self-test passed
```

The self-test rejects stale cargo-free command wording, missing cargo-free bundle paths, stale current blocker text, missing demo fields, and overbroad proof claims. The guide explicitly says the refreshed blocked evidence is not a Nix-free fixed-point success claim.

### V4 — formatting and Cairn gates

Formatting and whitespace checks (pueue task 51):

```text
cargo fmt -p mantle --check
git diff --check
```

Output: task completed successfully.

Cairn validation and gates (pueue task 52):

```text
COMMAND=validate
{"stage":"validate","valid":true,"verdict":null,"issue_count":0,"receipt_hash":null}
COMMAND=proposal
{"stage":"proposal","valid":true,"verdict":"PASS","issue_count":0,"receipt_hash":"a7bba9e176490b844f85c26bd856f3099215b8790446051710b3d8285f082f90"}
COMMAND=design
{"stage":"design","valid":true,"verdict":"PASS","issue_count":0,"receipt_hash":"74964cb86c8dc0482161ca168c6c15b5231d855dffba786a7bf4cf7cb717e95a"}
COMMAND=tasks
{"stage":"tasks","valid":true,"verdict":"PASS","issue_count":0,"receipt_hash":"72ad685365178cc21a68a09f857988d3cbae97cb58d22ae4d2cff6899f3b5609"}
```

## Lifecycle evidence

Sync/archive/post-archive validation (pueue tasks 53 and 54):

```text
nix run path:/home/brittonr/git/cairn#cairn -- sync refresh-expensive-proof-evidence --root . --execute
nix run path:/home/brittonr/git/cairn#cairn -- validate --root . | jq -c '{valid, issue_count:(.issues|length)}'
CAIRN_ARCHIVE_DATE=2026-07-03 nix run path:/home/brittonr/git/cairn#cairn -- archive refresh-expensive-proof-evidence --root . --execute
nix run path:/home/brittonr/git/cairn#cairn -- validate --root . | jq -c '{valid, issue_count:(.issues|length)}'
...
"receipt_hash": "ec96e45b1a63eb46b3cb00ad7b22dd6ff1c558fb75a6d85a58dd3a3513f9ff7f"
{"valid":true,"issue_count":0}

nix run path:/home/brittonr/git/cairn#cairn -- validate --root . | jq -c '{valid, issue_count:(.issues|length)}'
{"valid":true,"issue_count":0}
```
