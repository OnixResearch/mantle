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

## Closeout checkpoint — 2026-07-14

This section supersedes the 2026-07-12 blocker table for the implementation
state at closeout. The exact Octet rail is now available from immutable Octet
revision `86ee46b3b9257b145d2dbeb6ce9d9897607db99c`, package
`cargo-octet` `0.1.0`, profile `portable-component-baseline`, and wasm-tools
cohort `c50e2d7f0e8c49de4a1d44afae196bdf96bb14e67e7de0a153de146a6207449a`.
Mantle retains Octet's receipt and verification report and binds them to the
exact final portable bytes without reinterpreting the independent decision.

The production fixture used
`/nix/store/ysg1d1202xq9hjar9lvzwp3qiyjlqgh8-mantle-wasm-component-toolchain-v1`.
It built a no-std Rust component, validated every identity-changing portable
stage with the pinned `wasm-tools`, composed and virtualized with explicit
plans, stripped producer metadata before final validation, invoked the exact
Octet rail, smoke-ran `run()` with the pinned Wasmtime, published a canonical
materialization bundle plus report/attestation/release bindings, and then
consumer-rehashed every referenced file and directory. A separate production
test emitted Wasmtime AOT bytes and retained target/CPU/configuration/cohort/
WIT/build-input identity while preserving the non-portable trusted-native
label and all release/runtime non-claims.

Current command evidence:

```text
$ cargo fmt --check -p crunch-wasm-component-core -p crunch-wasm-component
(exit 0)

$ cargo test -p crunch-wasm-component-core --offline -- --nocapture
running 35 tests
test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

$ cargo check -p crunch-wasm-component-core --target wasm32-unknown-unknown --offline
(exit 0)

$ cargo test -p crunch-wasm-component --lib --offline -- --nocapture
running 14 tests
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

$ cargo clippy -p crunch-wasm-component-core -p crunch-wasm-component --all-targets --offline -- -D warnings
(exit 0)

$ cargo test -p mantle --test wasm_component_cli --offline -- --nocapture
running 3 tests
test production_cli_executes_pinned_pipeline_and_publishes_rehashable_component_evidence ... ok
test production_cli_binds_target_specific_wasmtime_aot_without_promoting_portability ... ok
test production_cli_fails_closed_on_identity_interface_composition_and_runtime_drift ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

$ cargo test -p mantle --test stdlib_tests wasm_component --offline -- --nocapture
running 5 tests
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 22 filtered out

$ nix --option builders '' --option secret-key-files '' build '.#checks.x86_64-linux.wasm-component-toolchain-identity' --no-link -L
(exit 0)

$ nix --option builders '' --option secret-key-files '' build '.#checks.x86_64-linux.wasm-component-toolchain-compatibility' --no-link -L
(exit 0)

$ nix --option builders '' --option secret-key-files '' run .#tigerstyle -- check -p crunch-wasm-component-core
(exit 0)

$ /nix/store/jyg3svhxsnmwr98282ar4c93j625c2hh-cairn-0.1.0/bin/cairn validate --root . --policy cairn-policy/generated/cairn-policy.json
{"valid":true,"issues":[],"change_issues":[],"spec_issues":[]}

$ cairn gate proposal|design|tasks add-wasm-component-build-pipeline --root .
proposal: {"valid":true,"verdict":"PASS","issues":[]}
design: {"valid":true,"verdict":"PASS","issues":[]}
tasks: {"valid":true,"verdict":"PASS","issues":[]}
```

### Remaining bounded blockers

- **Wizer shell execution remains unimplemented and task 8 remains unchecked.**
  The pinned Wizer accepts core Wasm modules, not Component Model bytes. The
  pure core validates deterministic imports, repeated clean digest agreement,
  and drift denial, but the current producer has no declared pre-component
  core-module/componentization handoff. Enabling Wizer therefore fails closed
  with `wizer-pre-component-core-module-required`; Mantle makes no claim that
  any Component Model bytes were Wizer-transformed.
- **The final quality/closeout task remains unchecked.** Scoped Tiger Style for
  `crunch-wasm-component-core` passes, but the full
  `crunch-wasm-component` rail still reports 89 shell-quality violations in
  `files.rs`, `process.rs`, and neighboring imperative adapters. Strict Clippy,
  focused tests, and Nix checks pass, but this broader Tiger Style debt is not
  hidden or waived.

No spec was synced and the change was not archived at this checkpoint.

## Bounded Tiger Style and Wizer seam repair — 2026-07-14

This pass repaired every full-crate Tiger Style diagnostic whose source path was
modified by this change. The repair introduced explicit stage/invocation option
records, decomposed the pipeline into bounded phases, removed ambiguous boolean
and string interfaces, made bounded file reads visible to the checker, and
preserved the existing functional-core/imperative-shell boundary.

```text
$ cargo fmt --check -p crunch-wasm-component
(exit 0)

$ cargo check -p crunch-wasm-component --tests --offline
(exit 0)

$ cargo test -p crunch-wasm-component --lib --offline -- --nocapture
running 14 tests
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

$ cargo clippy -p crunch-wasm-component --all-targets --offline -- -D warnings
(exit 0)

$ cargo test -p mantle --test wasm_component_cli --offline -- --nocapture
running 3 tests
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

$ nix --option builders '' --option secret-key-files '' run .#tigerstyle -- check -p crunch-wasm-component
(exit 1; 43 diagnostics)
remaining source paths: crates/crunch-wasm-component/src/files.rs
                        crates/crunch-wasm-component/src/process.rs
changed-source-path diagnostics: 0
```

The full rail therefore remains red, but its remaining 43 diagnostics are
confined to `files.rs` and `process.rs`. Neither file was modified by commits
`18bec579` or `ccd9b9cf`, and this bounded pass did not suppress, waive, or
silently broaden into those pre-existing adapters. The final quality task stays
unchecked.

### Wizer seam portfolio assessment

| Family | Mechanism | Checked evidence | State | Exact blocker |
|---|---|---|---|---|
| linker split | Build with `-C link-arg=--skip-wit-component`, run Wizer twice on the emitted core module, then `wasm-tools component new` | The pinned cohort produced `(module ...)`; repeated Wizer files were byte-identical; the re-componentized artifact validated, retained `run: func() -> u32`, and Wasmtime returned `42` | technically viable, not admitted | The pipeline request/report contracts do not identify the linker split, core object, componentization stage/configuration, or compile environment |
| nested-module extraction | Unbundle a completed component and select one nested core module | `wasm-tools component unbundle` exists, but a component may contain multiple application/adapter modules | rejected | Selection and reassembly would not prove which module corresponds to the declared source or preserve the original componentization contract |
| alternate Rust target | Compile a separately declared core-Wasm target, embed component metadata, then componentize | Requires another target/profile, adapter/configuration identity, and fixture matrix | out of bounded pass | This is a new build profile and evidence contract, not a closeout repair |

The linker probe establishes capability, not pipeline admission. In particular,
`ToolExecutionReceipt` currently omits the invocation environment, so injecting
`RUSTFLAGS=-C link-arg=--skip-wit-component` would alter compilation without
binding that choice into the receipt. The current stage graph also has no
componentization node or rehashable componentization configuration. Implementing
Wizer here would therefore create an unbound intermediate and overstate the
existing evidence contract. The shell keeps the exact
`wizer-pre-component-core-module-required` blocker, now explaining that the
capability exists but is not yet modeled or attested.

After these task and evidence updates, `cairn validate` and the proposal, design,
and tasks gates each returned `valid: true` with no issues; all three gate
verdicts were `PASS`.

No spec was synced and the change was not archived during this repair pass.

## Receipt-bound Wizer and final-quality completion — 2026-07-14

This section supersedes the earlier Wizer and Tiger Style blockers. Mantle now
models the complete pre-component transformation graph rather than inferring a
nested module from already-componentized bytes:

1. Cargo runs with a cleared, receipt-bound environment containing exact
   `RUSTFLAGS=-C link-arg=--skip-wit-component` and the pinned
   `CARGO_TARGET_WASM32_WASIP2_LINKER`.
2. The compilation receipt binds and rehashes subordinate `rustc` and
   `wasm-component-ld` executables, and the declared core-module output is
   validated and published as `compiled-core.wasm`.
3. Pinned Wizer runs twice from clean output paths with inherited environment
   and stdio disabled. Admission requires equal BLAKE3 identities and sizes.
4. Pinned `wasm-tools component new` creates separately classified Component
   Model bytes. Mantle validates that artifact before composition and never
   labels it as direct Wizer output.
5. The final normalized component still passes exact Octet collection and
   verification, Wasmtime smoke execution, bundle/report/attestation/release
   binding, and consumer-side remeasurement. Release eligibility and runtime
   authority remain explicit non-claims.

The production fixture exports a no-op `wizer.initialize`; the successful smoke
invocation still returns `42`. Negative coverage rejects Wizer output drift,
ambient observations, compile-environment drift, unexpected compile-state keys,
and componentization argument drift. Tool receipts now use schema
`mantle-wasm-component-tool-execution-receipt-v2` and bind the exact cleared
environment plus subordinate tool identities.

The broader shell-quality debt was repaired rather than waived. `files.rs` now
uses bounded no-follow reads, copies, and hashing; `process.rs` uses bounded
supervision and stream draining without treating fragmented reads as fixed-size
buffers. Failed child executions are classified before output hashing, so an
empty partial file cannot mask the authoritative process diagnostic.

Final current evidence:

```text
$ cargo test -q -p crunch-wasm-component-core --lib --offline
running 36 tests
test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

$ cargo test -q -p crunch-wasm-component --lib --offline
running 15 tests
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

$ cargo test -p mantle --test wasm_component_cli --offline -- --nocapture
running 3 tests
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

$ cargo test -q -p mantle --test stdlib_tests wasm_component --offline -- --nocapture
running 5 tests
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 22 filtered out

$ cargo test -q -p crunch-eval stdlib::tests --offline -- --nocapture
running 10 tests
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 68 filtered out

$ cargo fmt --check -p crunch-wasm-component-core -p crunch-wasm-component -p mantle
(exit 0)

$ cargo clippy -p crunch-wasm-component-core -p crunch-wasm-component --all-targets --offline -- -D warnings
(exit 0)

$ cargo check -p crunch-wasm-component-core --target wasm32-unknown-unknown --offline
(exit 0)

$ nix --option builders '' --option secret-key-files '' run .#tigerstyle -- check -p crunch-wasm-component-core
(exit 0)

$ nix --option builders '' --option secret-key-files '' run .#tigerstyle -- check -p crunch-wasm-component
(exit 0)

$ nix --option builders '' --option secret-key-files '' build '.#checks.x86_64-linux.wasm-component-toolchain-identity' --no-link -L
(exit 0)

$ nix --option builders '' --option secret-key-files '' build '.#checks.x86_64-linux.wasm-component-toolchain-compatibility' --no-link -L
(exit 0)

$ cargo -Zscript scripts/check-machine-schema-contracts.rs
machine schema contract check: PASS (16 contracted, 45 classified)

$ cairn validate --root . --policy cairn-policy/generated/cairn-policy.json
{"valid":true,"issues":[],"change_issues":[],"spec_issues":[]}

$ cairn gate proposal|design|tasks add-wasm-component-build-pipeline --root . --policy cairn-policy/generated/cairn-policy.json
proposal: {"valid":true,"verdict":"PASS","issues":[]}
design: {"valid":true,"verdict":"PASS","issues":[]}
tasks: {"valid":true,"verdict":"PASS","issues":[]}
```

The machine-schema check initially exposed one pre-existing stale producer
freshness binding for `release.function-address-binding`. The checked generator
updated only that expected BLAKE3 line in `schemas/machine-contracts/inventory.ncl`;
no schema or contract bytes changed, and the immediate check passed.

No spec was synced and the change was not archived.

## Post-archive integrated validation

The exact post-archive validation command and output were:

```text
$ /home/brittonr/git/OnixResearch/cairn/target/debug/cairn validate --root . --policy /home/brittonr/git/OnixResearch/cairn/cairn-policy/generated/cairn-policy.json
{
  "change_issues": [],
  "changes": 4,
  "findings": [],
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_findings": [],
  "spec_issues": [],
  "spec_substance": [
    {
      "path": "./cairn/changes/adapt-external-batch-dispatchers/specs/external-batch-dispatchers/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 18,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/changes/add-bounded-kernelscript-experiment/specs/kernelscript-experiment/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 14,
      "substantive_requirement_blocks": 8
    },
    {
      "path": "./cairn/changes/enforce-hermetic-release-handoff/specs/build-correctness/spec.md",
      "requirement_blocks": 4,
      "scenario_blocks": 7,
      "substantive_requirement_blocks": 4
    },
    {
      "path": "./cairn/changes/enforce-hermetic-release-handoff/specs/release-provenance/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 9,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "./cairn/changes/prove-hardware-simulation-build-flow/specs/hardware-simulation-builds/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 17,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/bootstrap-inventory/spec.md",
      "requirement_blocks": 3,
      "scenario_blocks": 10,
      "substantive_requirement_blocks": 3
    },
    {
      "path": "./cairn/specs/build-correctness/spec.md",
      "requirement_blocks": 21,
      "scenario_blocks": 52,
      "substantive_requirement_blocks": 21
    },
    {
      "path": "./cairn/specs/build-scheduling/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 16,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "./cairn/specs/build-tool-boundary/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 23,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/cache-substitution/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 16,
      "substantive_requirement_blocks": 5
    },
    {
      "path": "./cairn/specs/examples/spec.md",
      "requirement_blocks": 10,
      "scenario_blocks": 21,
      "substantive_requirement_blocks": 10
    },
    {
      "path": "./cairn/specs/foreign-derivation-import/spec.md",
      "requirement_blocks": 22,
      "scenario_blocks": 52,
      "substantive_requirement_blocks": 22
    },
    {
      "path": "./cairn/specs/gcc40-bridge/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 3,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "./cairn/specs/i386-tinycc27/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 4,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "./cairn/specs/kani-toolchain-evidence/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 11,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "./cairn/specs/kernel-bundle-oci/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 15,
      "substantive_requirement_blocks": 8
    },
    {
      "path": "./cairn/specs/machine-artifact-contracts/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 9,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/nickel-export-infrastructure/spec.md",
      "requirement_blocks": 6,
      "scenario_blocks": 12,
      "substantive_requirement_blocks": 6
    },
    {
      "path": "./cairn/specs/operator-diagnostics/spec.md",
      "requirement_blocks": 13,
      "scenario_blocks": 32,
      "substantive_requirement_blocks": 13
    },
    {
      "path": "./cairn/specs/portable-build-receipts/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 14,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "./cairn/specs/project-workflows/spec.md",
      "requirement_blocks": 24,
      "scenario_blocks": 80,
      "substantive_requirement_blocks": 24
    },
    {
      "path": "./cairn/specs/realization-routing/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 21,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/release-provenance/spec.md",
      "requirement_blocks": 60,
      "scenario_blocks": 85,
      "substantive_requirement_blocks": 60
    },
    {
      "path": "./cairn/specs/remote-builds/spec.md",
      "requirement_blocks": 35,
      "scenario_blocks": 109,
      "substantive_requirement_blocks": 35
    },
    {
      "path": "./cairn/specs/rust-package-planning/spec.md",
      "requirement_blocks": 129,
      "scenario_blocks": 447,
      "substantive_requirement_blocks": 129
    },
    {
      "path": "./cairn/specs/source-transports/spec.md",
      "requirement_blocks": 12,
      "scenario_blocks": 25,
      "substantive_requirement_blocks": 12
    },
    {
      "path": "./cairn/specs/spacewasm-reference-materialization/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 16,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/store-transports/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 19,
      "substantive_requirement_blocks": 8
    },
    {
      "path": "./cairn/specs/verification-evidence/spec.md",
      "requirement_blocks": 57,
      "scenario_blocks": 160,
      "substantive_requirement_blocks": 57
    },
    {
      "path": "./cairn/specs/wasm-component-builds/spec.md",
      "requirement_blocks": 14,
      "scenario_blocks": 25,
      "substantive_requirement_blocks": 14
    }
  ],
  "specs_validated": 30,
  "substance": [
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/adapt-external-batch-dispatchers/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 30,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/adapt-external-batch-dispatchers/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 15,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/adapt-external-batch-dispatchers/specs/external-batch-dispatchers/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 18,
      "substantive_lines": 82,
      "substantive_requirement_blocks": 9,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 17,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/adapt-external-batch-dispatchers/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 17,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 17,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 17
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-bounded-kernelscript-experiment/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 29,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-bounded-kernelscript-experiment/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 13,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-bounded-kernelscript-experiment/specs/kernelscript-experiment/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 14,
      "substantive_lines": 57,
      "substantive_requirement_blocks": 8,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 14,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-bounded-kernelscript-experiment/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 20,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 14,
      "task_done": 10,
      "task_in_progress": 0,
      "task_todo": 4
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/enforce-hermetic-release-handoff/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 12,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/enforce-hermetic-release-handoff/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 11,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/enforce-hermetic-release-handoff/specs/build-correctness/spec.md",
      "requirement_blocks": 4,
      "scenario_blocks": 7,
      "substantive_lines": 26,
      "substantive_requirement_blocks": 4,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/enforce-hermetic-release-handoff/specs/release-provenance/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 9,
      "substantive_lines": 35,
      "substantive_requirement_blocks": 7,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 13,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/enforce-hermetic-release-handoff/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 16,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 13,
      "task_done": 12,
      "task_in_progress": 0,
      "task_todo": 1
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/prove-hardware-simulation-build-flow/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 28,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/prove-hardware-simulation-build-flow/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 14,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/prove-hardware-simulation-build-flow/specs/hardware-simulation-builds/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 17,
      "substantive_lines": 78,
      "substantive_requirement_blocks": 9,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 16,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/prove-hardware-simulation-build-flow/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 16,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 16,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 16
    }
  ],
  "substance_findings": [],
  "substance_issues": [],
  "valid": true
}
```
