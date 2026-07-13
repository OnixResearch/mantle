# Local implementation evidence — 2026-07-12

## Goal and claim boundary

Drain `add-wasm-component-build-pipeline` through the largest locally verifiable
slice without claiming unavailable tool execution. Completion evidence for this
slice is:

- a typed Nickel manifest and deterministic generated-input export;
- source/dependency ownership receipts and typed output classes;
- a compiler-enforced no-std functional core for every pure operation named by
  tasks 1 and 2;
- positive and negative tests, host and wasm compilation, strict Clippy, Tiger
  Style, stdlib embedding, and Cairn lifecycle validation.

False completion is explicitly excluded. This evidence does **not** claim that
`wkg`, Rust `wasm32-wasip2`, WAC, WASI-Virt, wasm-tools, Octet, Wizer, or
Wasmtime executed; that a compatible component tool cohort exists; that a
portable component was produced; or that a component has runtime authority,
behavioral correctness, or release eligibility.

## Proven task slice

### Task 1 — typed configuration and build-only boundary

Implemented by:

- `lib/wasm_component.ncl` — typed package/WIT/world/source/cohort/composition/
  virtualization/validation/Wizer/AOT/output/non-claim configuration;
- deterministic exports for tool-native `wkg.toml`, explicit registry/protocol
  config, WAC source, and Mantle command/runtime DTOs;
- generated input ownership records binding source and dependency BLAKE3
  identities, followed by core-computed content and receipt BLAKE3 identities;
- typed output classes and stale content/owner-receipt denial;
- embedded stdlib parity and Nickel-to-Rust generated-input deserialization;
- ADR 0016, which preserves ADR 0010's build-only boundary.

### Task 2 — pure functional core

Implemented in `crates/crunch-wasm-component-core/`:

- strong custom-serde `Blake3Identity` and protocol-only
  `OciSha256Digest` roles;
- canonical BLAKE3 manifest, cohort, plan, receipt, and report identities;
- bounded manifest and parsed `wkg.lock` validation, exact `=version`
  requirements, registry/config identity checks, checked lock-byte identity,
  and duplicate/unlocked/missing immutable materialization denials;
- deterministic source acquisition plans carrying credential handles rather
  than credential values;
- exact local composition graphs with duplicate, missing dependency/import,
  wrong-world, self-edge, cycle, and non-local fallback denials;
- explicit deny-all WASI subsystem planning, identified fixed values and
  virtual mounts, reviewed grants/passthrough, and post-composition remaining-
  import equality;
- exact build-local/Octet report-to-artifact binding without interpreting Octet
  findings;
- Wizer ambient-input and repeated-output admission;
- target/CPU/config/cohort/source-bound AOT admission labeled
  target-specific trusted native code;
- bounded typed stage-report DTOs with BLAKE3 parent links, required non-claims,
  and no runtime-authority claim variant.

## Baseline

The first isolated baseline attempt failed while compiling
`nickel-lang-parser` with:

```text
Disk quota exceeded (os error 122)
```

This was an environmental failure, not a test failure. Only
`/tmp/mantle-wasm-target/baseline*` was removed. The filesystem then reported
176 GiB available. Before source edits, focused baselines succeeded:

```text
test result: ok. 161 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

for `cargo test -p crunch-project-core --lib`, and:

```text
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

for `cargo test -p mantle --test stdlib_tests`.

## Final deterministic checks

Pueue task `1393` ran the final focused command set in group
`mantle-wasm-component-20260712` with
`CARGO_TARGET_DIR=/tmp/mantle-wasm-target/work-cargo`. Full outputs are under
`/tmp/mantle-wasm-target/final/`.

### Pure core tests

Command:

```text
cargo test -p crunch-wasm-component-core
```

Result:

```text
running 28 tests
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The suite contains happy paths and denials for digest-role confusion, manifest
and lock drift, registry/materialization drift, missing/wrong-world/cyclic/
non-local composition, pass-through defaults, undeclared remaining imports,
Octet report byte mismatch, ambient/drifting Wizer output, cross-target/tampered
AOT receipts, stale generated inputs/receipts, mixed ownership, circular report
parents, missing non-claims, and runtime-authority overclaims.

### no-std wasm compilation

The host toolchain did not have a preinstalled `wasm32-unknown-unknown` target,
so the direct target check failed with `can't find crate for core`. The
nightly/rust-src fallback was then used explicitly:

```text
cargo check -Zbuild-std=core,alloc \
  -p crunch-wasm-component-core \
  --target wasm32-unknown-unknown
```

Result:

```text
Checking crunch-wasm-component-core v0.1.0
Finished `dev` profile
```

### Nickel and embedded stdlib

Commands and results:

```text
cargo test -p mantle --test stdlib_tests
running 27 tests
test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

cargo test -p crunch-eval stdlib::tests
running 10 tests
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 68 filtered out
```

### Quality and purity

Commands:

```text
cargo fmt --check -p crunch-wasm-component-core
cargo clippy -p crunch-wasm-component-core --all-targets -- -D warnings
nix run path:/tmp/mantle-wasm#tigerstyle -- \
  check -p crunch-wasm-component-core
```

All exited successfully. Strict Clippy and Tiger Style each reported
`Finished dev profile`. The production-core purity scan reported:

```text
PASS: no filesystem, environment, process, clock, async, or output effects in production core sources
```

The duplicate `src/main.rs` target warning and unrelated first-party/vendor
warnings emitted while compiling the root stdlib test predate this slice and did
not fail any focused check.

### Cairn validation and gates

Pueue task `1409` invoked the canonical, resolved Cairn checkout at
`/home/brittonr/git/OnixResearch/cairn` after task/evidence updates. Results:

```text
cairn validate: "valid": true, "issues": []
cairn gate proposal: "valid": true, "verdict": "PASS"
cairn gate design: "valid": true, "verdict": "PASS"
cairn gate tasks: "valid": true, "verdict": "PASS"
```

The exact JSON receipts are captured under `/tmp/mantle-wasm-target/final/`.
No sync or archive command was run.

## Mainline integration and adversarial hardening

The two implementation commits were integrated on main as `0e1c60dc` and
`3b3946d9`. The ADR was renumbered to 0016 because main already owns ADRs 0014
and 0015.

Adversarial review found a concrete false-acceptance path in generated-input
freshness validation: a caller could supply a mutated plan without revalidating
its plan/receipt identities, and extra or duplicate observed generated files
were ignored. The repaired core now remeasures receipt content, recomputes each
receipt and plan identity, validates schema/order/ownership, requires the
observed target set to match exactly, rejects duplicate/unowned targets, and
checks observed byte bounds before hashing. Two negative tests cover forged
plans and unexpected/duplicate observed files.

Current mainline evidence:

```text
$ nix develop -c env CARGO_TARGET_DIR=/tmp/mantle-main-wasm-target cargo test -q -p crunch-wasm-component-core --lib
running 30 tests
..............................
test result: ok. 30 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

$ nix develop -c env CARGO_TARGET_DIR=/tmp/mantle-main-wasm-target cargo clippy -q -p crunch-wasm-component-core --all-targets -- -D warnings
(exit 0)

$ nix run path:$PWD#tigerstyle -- check -p crunch-wasm-component-core
(exit 0)

$ nix develop -c env CARGO_TARGET_DIR=/tmp/mantle-main-wasm-target cargo check -Zbuild-std=core,alloc -p crunch-wasm-component-core --target wasm32-unknown-unknown
Checking crunch-wasm-component-core v0.1.0
Finished `dev` profile

$ nix develop -c env CARGO_TARGET_DIR=/tmp/mantle-main-wasm-target cargo test -q -p mantle --test stdlib_tests wasm_component -- --nocapture
running 5 tests
.....
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 22 filtered out; finished in 0.16s

$ nix develop -c env CARGO_TARGET_DIR=/tmp/mantle-main-wasm-target cargo test -q -p crunch-eval stdlib::tests -- --nocapture
running 10 tests
..........
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 68 filtered out; finished in 0.07s

$ nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs
machine schema contract check: PASS (15 contracted, 44 classified)

$ nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root . --policy cairn-policy/generated/cairn-policy.json
valid=true changes=13 specs_validated=39 issues=[]

$ nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate proposal add-wasm-component-build-pipeline --root . --policy cairn-policy/generated/cairn-policy.json
input_hash=318d309fe3a8c92b44f349cd09ca3f796f5ecb4cce21d1a52c8e6c837f6d7b60
receipt_hash=a92ca4def6f0d0722c197ee2dd0d1e3ce3c0a78837baf6a5c1914f25c9ed5177
issues=[] valid=true verdict=PASS

$ nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate design add-wasm-component-build-pipeline --root . --policy cairn-policy/generated/cairn-policy.json
input_hash=545d4ce8bd93734d23b709d84487a60f3569d3989c41996ef2ffc9da8d19f447
receipt_hash=0f8aab17aeb1a7164aca069a2c40db2a2294d002493e712682d407bfaf79b81c
issues=[] valid=true verdict=PASS

$ nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate tasks add-wasm-component-build-pipeline --root . --policy cairn-policy/generated/cairn-policy.json
input_hash=c5489886e0c405c1575866e6b9b234c1751d5ae76a5489950cf13033bedab235
receipt_hash=3a0704340c1372945afe3a2277bc3702ee2ae404422c4ba62693a7a5e942ad22
issues=[] valid=true verdict=PASS
```

## Production execution checkpoint — 2026-07-12

All Cargo commands below used `TMPDIR=/tmp/mantle-wasm-production-tmp`,
`CARGO_TARGET_DIR=/tmp/mantle-wasm-production-target`, the pinned Rust/clang/mold
and OpenSSL development environment, and `SNIX_BUILD_SANDBOX_SHELL=/bin/sh`.
The CLI fixture additionally used
`MANTLE_WASM_COMPONENT_TOOLCHAIN=/nix/store/dyp444vl42jc33wh2sl9qbhc39051gy8-mantle-wasm-component-toolchain-v1`.

```text
$ cargo test -p crunch-wasm-component --offline -- --nocapture
running 11 tests
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

$ cargo clippy -p crunch-wasm-component --all-targets --offline -- -D warnings
(exit 0)

$ cargo test -p crunch-wasm-component-core --offline -- --nocapture
running 33 tests
test result: ok. 33 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

$ cargo test -p mantle --test stdlib_tests wasm_component --offline -- --nocapture
running 5 tests
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 22 filtered out

$ cargo test -p mantle --test wasm_component_cli --offline -- --nocapture
running 2 tests
test production_cli_executes_pinned_pipeline_and_persists_octet_blocker ... ok
test production_cli_fails_closed_on_identity_interface_composition_and_runtime_drift ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

$ nix build .#checks.x86_64-linux.wasm-component-toolchain-identity --no-link -L
(exit 0; no stdout/stderr)

$ nix build .#checks.x86_64-linux.wasm-component-toolchain-compatibility --no-link -L
(exit 0; no stdout/stderr)

$ nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root . --policy cairn-policy/generated/cairn-policy.json
{
  "change_issues": [],
  "changes": 6,
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "spec_issues": [],
  "specs_validated": 32,
  "valid": true
}
```

The positive CLI test exercised the checked local `wkg.lock`, pinned `wkg`,
`wit-bindgen`, offline Cargo `wasm32-wasip2`, build-local wasm-tools validation,
WAC composition, WASI-Virt, remaining-import inspection, and Wasmtime smoke
execution. It persisted only the bounded report and validated portable artifacts,
then stopped nonzero at `octet-wasm-artifact-rail-unavailable`; it emitted no
materialization bundle. The negative matrix rejected tool/package identity drift,
incomplete composition bindings, remaining-import drift, and runtime-output
drift. The post-review adapter rerun also covered no-follow checked-lock reads,
bounded process output/time/descendant termination, and no-replace publication.

## Portfolio-search registry

| Family | Mechanism | Evidence | State | Exact blocker / next check |
|---|---|---|---|---|
| typed-config-core | Nickel contracts plus no-std deterministic Rust core | 33 core tests and five focused stdlib tests | validated for tasks 1-2 | None inside the declared pure/config slice |
| packaged-tool-cohort | One Nix-owned physical executable cohort with manifest/tool BLAKE3 identities | Identity and Rust 1.90.0/WASI 0.2.3 compatibility checks plus CLI preflight | validated for task 4 | Re-run compatibility fixtures on any cohort member/configuration change |
| registry-and-build-shell | Checked local wkg resolution followed by network-denied Cargo/WAC/WASI-Virt execution | Positive CLI artifacts/receipts and negative tool/package/composition/import cases | validated for tasks 3, 5, and 6 | Live OCI resolution remains deliberately unadmitted in this local-only shell |
| independent-artifact-rail | Invoke Octet over exact portable bytes | The pipeline fails closed with the authoritative Octet blocker | blocked | Canonical Octet has no implemented Wasm artifact rail CLI/package |
| component-transform | Apply Wizer to Component Model bytes | Wizer 10.0.0 accepts core modules, not components | blocked | No unsafe core-module fallback is admitted |
| native-precompile-and-bundle | Wasmtime AOT plus consumer materialization handoff | Pure admission contracts exist; no production AOT or complete bundle was emitted | blocked | Implement exact precompile receipts only after Octet admission, then emit/reverify the complete bundle |

## Remaining blocker and non-claims

All tasks from the combined validation/Octet task onward remain unchecked. Build-local wasm-tools validation executed, but the
combined validation task cannot complete without Octet over the same bytes.
Wizer component transformation, Wasmtime AOT/precompile, final materialization
bundle creation, release/attestation integration, the full negative fixture
matrix, and final lifecycle gates remain unproven. Wasmtime smoke execution is
not AOT evidence, runtime authority, behavioral correctness, or release
eligibility.

No accepted spec was synced and the change was not archived.
