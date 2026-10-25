# Tasks: Standardize derivation finish gates

Source implementation and fixture delivery are distinct from producer acceptance.
Checked implementation tasks below do not accept the ADR, production adoption,
or runtime gates. Those require captured execution.

## Phase 1: Baseline and contract

- [ ] [serial] T1.1 Create an isolated worktree from current `origin/main`. Record the current per-recipe smoke inventory under `bootstrap/`, the builder-layer finish surface, and focused baseline test output. r[mantle.derivation_finish_gates.shared_contract]
- [x] [serial] T1.2 Define the versioned finish-gate contract in typed Nickel policy with the deterministic export, named bounds, defaults, and explicit opt-outs. r[mantle.derivation_finish_gates.shared_contract]
- [ ] [serial] T1.3 Review and accept the gate-set decision (version default-on, leak policy per build kind, relocation and dlopen opt-in) in an ADR. r[mantle.derivation_finish_gates.shared_contract]

## Phase 2: Core and shell

- [x] [serial] T2.1 Implement pure gate evaluation over declared gate records and observed facts with typed denials and named bounds. r[mantle.derivation_finish_gates.shared_contract]
- [x] [serial] T2.2 Implement the version gate in the finish shell: empty-environment run, declared command resolution, default `--version` command for bin-installing derivations. r[mantle.derivation_finish_gates.version_check]
- [x] [serial] T2.3 Implement the reference leak gate: prefix scan over outputs, cross-build deny list from build-only dependencies, bounded context excerpts. r[mantle.derivation_finish_gates.reference_leak_gate]
- [x] [serial] T2.4 Implement the relocation rerun gate as an opt-in finish step. r[mantle.derivation_finish_gates.relocated_rerun]
- [x] [serial] T2.5 Implement the dlopen audit gate with a bounded loader-audit module for Linux dynamic outputs. r[mantle.derivation_finish_gates.dlopen_audit]
- [x] [serial] T2.6 Surface gate results in the build report under a versioned block. r[mantle.derivation_finish_gates.shared_contract]

## Phase 3: Positive and negative verification

- [x] [parallel] T3.1 Add positive fixtures: passing version output, clean leak scan, passing relocation rerun, declared optional soname. r[mantle.derivation_finish_gates.version_check] r[mantle.derivation_finish_gates.reference_leak_gate] r[mantle.derivation_finish_gates.relocated_rerun]
- [ ] [parallel] T3.2 Add negative fixtures: wrong version, nonzero exit, missing binary without opt-out, cross leak, native report hit, failed rerun, undeclared missing dlopen, oversized or malformed gate record. r[mantle.derivation_finish_gates.version_check] r[mantle.derivation_finish_gates.reference_leak_gate] r[mantle.derivation_finish_gates.relocated_rerun] r[mantle.derivation_finish_gates.dlopen_audit]
- [ ] [serial] T3.3 Prove existing recipes evaluate and build unchanged before any adoption commit. r[mantle.derivation_finish_gates.shared_contract]

## Phase 4: Adoption

- [ ] [serial] T4.1 Adopt the gates in the bootstrap recipe family with per-family review of opt-outs, starting with the GCC wrapper outputs that previously leaked build-tree paths. r[mantle.derivation_finish_gates.reference_leak_gate]
- [ ] [serial] T4.2 Run focused core and shell tests before and after changes, strict Clippy, policy freshness, and relevant Nix checks. Preserve exact blockers. r[mantle.derivation_finish_gates.shared_contract]
- [ ] [serial] T4.3 Sync accepted specs and archive through the isolated branch workflow with retained completion evidence. r[mantle.derivation_finish_gates.shared_contract]

## Current source delivery and verification boundary (2026-10-23)

`crates/crunch-build/src/finish_gates.rs` consumes the typed policy and existing
CAS reference scanner; the builder shell runs the gates after the child builder
exits. `builders/mk_derivation.ncl` now retains version-command/version defaults
under partial gate adoption, while explicit overrides win. Loader-audit parsing
does not merge provider success across original/relocated runs, rejects malformed
initialization/name records, and limits unresolved checks to sonames rather than
absolute dependency paths. Focused boundary fixtures accompany these changes.
Pure policy/declaration admission, reference bounds, execution-evidence evaluation
and loader-audit parsing now return `GateDenial` variants, with distinct schema,
command, environment, dependency, capacity, evidence and audit failure categories.
There is no generic string-error variant. Existing diagnostic framing is retained
by `Display`; `build_request.rs` and `orchestrate.rs` format failures at their
actual build-shell boundaries, and audit errors are formatted at report creation.
I/O errors remain separate. No report schema, enabled-input set, policy source,
or generated-export bytes changed. Boundary fixtures assert denial variants and
the existing candidate/finding admission edges rather than diagnostic wording.
`bootstrap/gcc-10-final.ncl` already declares version and build-tree leak gates;
that declaration is not evidence that the family built or passed.

The retained 2026-10-22 baseline at
`../../archive/2026-10-22-schedule-tigerstyle-hardening-backlog/evidence/v1-gates-2026-10-22/head-remaining-suites/mantle-finish_gates.log`
has five finish-gate fixture failures at the policy-freshness blocker. The parent
subsequently published the contract digest repair at `81a74ed7d`; this work does
not change policy sources, regenerate exports, or re-pin that digest. No build or
test was run for that source-only delivery, as requested. Its historical outcome
is unchanged; the completed later bounded runtime observation is recorded below.

Original parent integration commands (not executed by the source-only worker):

```sh
nix develop -c cargo test -p crunch-build --lib finish_gates -- --nocapture
nix develop -c cargo test -p mantle --test finish_gates -- --nocapture
```
For an actual CLI smoke after building the integrated binary, create an isolated
physical store/state directory and run from the repository root with a static
BusyBox available to the ordinary sandbox configuration:

```sh
CRUNCH_NO_FUSE=1 target/debug/mantle --json --store <empty-physical-store> --state-dir <empty-state-dir> build --no-substitute -I . -I lib tests/fixtures/finish_gates.ncl
```

Retain JSON and logs; require version/relocation/reference gate passes. This is
the invocation used by the sandbox fixture helper, not a production GCC proof.
Typed-denial source delivery and bounded runtime observation are distinct from
acceptance. Full recipe baseline/adoption, complete negative/core/shell matrix,
strict Clippy/Nix gates, policy approval and archive/sync remain open.

## Completed bounded runtime observation (2026-10-23)

`evidence/actual-finish-runtime-2026-10-23.json` links the single retained driver/result/raw-stream bundle under the fault change, with BLAKE3 and immutable wait-request-repaired executable pins. An actual CLI build of `tests/fixtures/finish_gates.ncl` exited 0 with `succeeded=1`, `built=1`, `cached=0`, `failed=0`; version expected `1.0`, reference-leak and relocation reports all passed. A separate wrong-version fixture expected `2.0`, exited 1, reported version denied, `built=0`, `failed=1`, no outcomes, and reference/relocation not-run. Both reports explicitly opt out of dlopen: no dynamic loader claim follows.

These reachable positive and wrong-version paths are exercised on the existing immutable wait-request-repaired CLI cohort, not a newly pinned exact-exec/final integrated subject. The receipt retains selected unchanged source bindings, command/environment and repeatability limits; it does not prove whole-tree source/binary identity. The original five policy-freshness failures remain historical evidence rather than being rewritten as passes.

Build finish-shell/policy maintainers own bounded interpretation/repetition; Main owns final gate integration. Complete negative/core/shell matrix, isolated-origin inventory/baseline, unchanged recipe/family builds and bootstrap adoption are local actions, not external blockers. Gate-set ADR and family opt-out approval require the responsible human/policy owner; no signature or authority is invented. No checkbox, production adoption, provider authority, lifecycle acceptance, spec sync, archive or release claim changes.
