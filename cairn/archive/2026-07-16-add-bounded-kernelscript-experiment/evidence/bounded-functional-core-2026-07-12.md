# Bounded KernelScript functional-core evidence — 2026-07-12

## Status

This is partial implementation evidence for the safely feasible slice of
`add-bounded-kernelscript-experiment`. The change is intentionally active and
unarchived. Compiler materialization, code-generation execution, artifact
builds, real target inspection, and runtime handoff are not claimed.

## Authority checkpoint

**Question:** Which KernelScript source and compiler closure can Mantle admit as
authoritative for the bounded experiment?

**Inspected evidence:** KernelScript tag/release `v0.1.2`, Git revision
`0c80d4e4ac0029d34cbc9d65e76d78c075b64555`, release archive
`kernelscript-0.1.2-source.tar.gz`, release checksums, tagged
`kernelscript.opam`, `.github/workflows/ci.yml`, and
`.github/workflows/release.yml`.

**Decision:** Pin the official source archive by upstream SHA-256
`9a00b96e1f127d4806c28b076f270acdc4bf4a8c558ca636bfd9f49268b479c1`
and measured archive BLAKE3
`439431f81df45b043c218f4f5a41917ddd616e0defa35ff134c1cf5273124a57`.
Do not admit an executable compiler cohort: the tag/release has no immutable
`opam.locked`, while CI/release workflows resolve mutable opam repositories.
Do not use an upstream release binary, host `kernelscript`, or ambient opam
switch as a closure substitute.

The fetched official release archive was independently remeasured in this
session:

```text
$ sha256sum kernelscript-0.1.2-source.tar.gz
9a00b96e1f127d4806c28b076f270acdc4bf4a8c558ca636bfd9f49268b479c1
$ b3sum kernelscript-0.1.2-source.tar.gz
439431f81df45b043c218f4f5a41917ddd616e0defa35ff134c1cf5273124a57
```

**Owner:** A future Mantle KernelScript cohort update owns producing and
reviewing a complete immutable opam dependency lock and its materialized
compiler-closure BLAKE3 identity.

**Next action:** Materialize only after that lock exists; then bind the exact
compiler executable and full closure before enabling code-generation execution.

## Target checkpoint

**Question:** Is there an authoritative exact Onix kernel cohort available for
actual eBPF/module compilation and inspection?

**Inspected evidence:** Current worktree bootstrap/kernel assets and the
accepted frontend-neutral kernel-bundle OCI projection contract.

**Decision:** No exact KernelScript experiment cohort binds one authoritative
Onix kernel build, architecture, release, BTF, headers, config, compiler flags,
and compilation toolchain. Synthetic identities are permitted only in pure
positive/negative planning fixtures and are labeled fixture-only.

**Owner:** A future Onix/Mantle integration update owns publishing and admitting
that exact cohort. Onix semantics remain external to Mantle.

**Next action:** Keep all real build/inspection/candidate-production tasks
unchecked until the cohort is available and exact admission passes without
ambient running-host fallback.

## Implemented bounded slice

- Closed typed Nickel `mantle-kernelscript-experiment-v1` profile with mandatory
  beta/default-off posture, explicit source/compiler/toolchain/target/output
  facts, named bounds, denied network policy, and non-claims.
- Official flat fixed-output source derivation and explicit unavailable-lock /
  rejected-release-binary facts.
- `no_std` functional core for profile/compiler/target admission, pure offline
  code-generation planning, exact generated-project classification, Mantle-owned
  compilation plans, execution-request rejection, bounded ELF/BTF/module shape
  inspection, experimental-unverified ModulePack/BPF Pack candidate projection,
  and canonical BLAKE3 receipts. Compiler admission compares the exact declared
  package/version/artifact cohort, not only a dependency count or lock digest.
- Command admission rejects generated `Makefile`/`Kbuild`, `make`, shells,
  unknown tools, shell operators/interpolation, response files, and path escapes.
- Synthetic positive fixture shapes plus comprehensive negative tests for
  profile, compiler, generated-project, target, planner, checked ELF range
  arithmetic, malformed BTF headers, mismatched module `vermagic`, inspection,
  handoff, receipt leakage, and overclaim failures.
- Operator documentation with source pins, blockers, review boundary, upgrade
  procedure, lifecycle commands, ChaosControl dependency, and non-claims.

A local VibeThinker adversarial review was treated as non-authoritative test
input. Its actionable findings were validated against the code and closed by
binding exact dependency members, checking ELF range overflow and BTF/module
metadata bytes, binding exact generated source members, and adding recomputable
BLAKE3 identities for plans, inspections, candidates, and receipt relations.
Its requests for real build/runtime evidence remain blocked rather than being
converted into synthetic success claims. Its assertion that the new crate
contains the workspace `winx`/`crossbeam-epoch` failures was rejected by the
locked package-scoped dependency tree above.

## Focused validation transcript

Pueue task `787` completed successfully in this worktree after the final
identity and inspection hardening. Commands ran under `nix develop`;
`CARGO_TARGET_DIR` values were isolated under `/tmp`.

```text
$ cargo fmt -p crunch-kernelscript-core -- --check
PASS: core rustfmt
$ rustfmt --edition 2024 --check crates/crunch-eval/src/stdlib.rs tests/kernelscript_experiment.rs
PASS: shell/integration rustfmt
$ cargo test -p crunch-kernelscript-core
running 19 tests
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
$ cargo test -p mantle --test kernelscript_experiment
running 5 tests
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
$ cargo test -p crunch-eval stdlib::tests::
running 10 tests
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 68 filtered out; finished in 0.08s
$ cargo -Zbuild-std=core,alloc check -p crunch-kernelscript-core --target wasm32-unknown-unknown
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.92s
$ cargo clippy -p crunch-kernelscript-core --all-targets -- -D warnings
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.10s
PASS: focused KernelScript functional-core validation
```

A plain wasm check first reported that the Nix-provided toolchain lacked a
preinstalled `wasm32-unknown-unknown` standard library. The successful command
above used the pinned nightly toolchain's Rust sources via
`-Zbuild-std=core,alloc`; it does not weaken the no-std boundary.

Dependency inventory is bounded to already-locked workspace dependencies:
`blake3 1.8.2`, `serde 1.0.228`, and `serde_json 1.0.149` plus their existing
transitive closure. The exact inventory command was:

```text
$ cargo tree -p crunch-kernelscript-core --edges normal --locked
```

The direct `cargo-deny` executable is present in the Nix dev shell. Pueue task
`587` ran the full workspace policy and exited `5`:

```text
$ cargo-deny check
error[rejected]: failed to satisfy license requirements
  winx v0.36.4: Apache-2.0 WITH LLVM-exception is not explicitly allowed
error[vulnerability]: Invalid pointer dereference in fmt::Pointer impl for Atomic and Shared
  crossbeam-epoch v0.9.18
  ID: RUSTSEC-2026-0204
  Solution: upgrade to >=0.9.20
advisories FAILED, bans ok, licenses FAILED, sources ok
```

Neither `winx` nor `crossbeam-epoch` appears in the locked dependency tree for
`crunch-kernelscript-core`; the new crate added no third-party package beyond
already-present `blake3`, `serde`, and `serde_json`. The full-workspace failures
are recorded as baseline repository blockers rather than hidden or attributed
to this bounded slice. The broader validation task remains unchecked.

## Cairn lifecycle transcript

Pueue task `807` refreshed the active package after the implementation commit and final task-note update.
The gates are advisory package-shape reviews; they do not convert unchecked
execution tasks into success evidence.

```text
$ nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .
changes: 8
specs_validated: 35
change_issues: []
spec_issues: []
issues: []
valid: true
$ nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate proposal add-bounded-kernelscript-experiment --root .
input_hash: 3864f641099553847521483138b77c052ea74f415a097a0fa9593ad7c8d563cf
receipt_hash: c944a7f2e2c73e720c13deaa774171ab3d94c998499875e79d14085e0a697c8c
issues: []
verdict: PASS
$ nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate design add-bounded-kernelscript-experiment --root .
input_hash: 69ca1570938d3477625e96076fa351cdd9424baee24674ef1069b87bf77cbf19
receipt_hash: cf6907cb49026c45fcbc97bef877f810e061b559d3c3c0509186e95a4567687c
issues: []
verdict: PASS
$ nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate tasks add-bounded-kernelscript-experiment --root .
input_hash: 0d2fb2c6c41e74ebe513bb7947e5a15ce5faf19d5ae7d1f04c7b13a9e68ae35b
receipt_hash: 876f4857cdad460fb6417c9174215c87f680663876ef8eefea214d39bb4745fa
issues: []
verdict: PASS
```

No spec sync, archive, pull request, or push was performed.

## Main-branch integration checkpoint

After integration and ADR renumbering to 0022, pueue task `59` successfully reran the focused core tests, Mantle integration test, strict core Clippy, and `git diff --check` with an isolated `/tmp` Cargo target. Pueue task `88` then reran Cairn validation plus proposal/design/tasks gates against the integrated main tree; the final tasks gate remained `PASS` with no issues. These checks do not remove the authority blockers below.

## Exact blockers

1. **`compiler-dependency-lock-unavailable`** — no authoritative immutable opam
   dependency lock exists in KernelScript `v0.1.2`; compiler derivation,
   compiler success, code-generation execution, and downstream builds remain
   unchecked.
2. **`authoritative-onix-target-cohort-unavailable`** — no exact authoritative
   Onix kernel/BTF/header/config/architecture/toolchain cohort is available;
   real output inspection, candidate production, and runtime gates remain
   unchecked.

## Non-claims

This evidence does not establish KernelScript language soundness, BPF verifier
acceptance, kernel safety, runtime correctness, module load success, BPF
load/attach success, deployability, Onix semantics, ChaosControl runtime
evidence, production readiness, or release-binary trust. Synthetic ELF and
kernel identities prove deterministic pure checks only.
