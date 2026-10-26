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
- [x] [parallel] T3.2 Add negative fixtures: wrong version, nonzero exit, missing binary without opt-out, cross leak, native report hit, failed rerun, undeclared missing dlopen, oversized or malformed gate record. r[mantle.derivation_finish_gates.version_check] r[mantle.derivation_finish_gates.reference_leak_gate] r[mantle.derivation_finish_gates.relocated_rerun] r[mantle.derivation_finish_gates.dlopen_audit]
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

## T3.2 fixture inventory and verification (2026-10-26)

The matching-version/nonzero-exit regression is now present as
`real_finish_gate_denies_matching_version_with_nonzero_exit_without_publication`
in `tests/finish_gates.rs`. It derives the negative recipe from the existing
positive fixture by printing `demo-1.0` and then exiting 7, matching the recipe
mutation in `target/cairn-finish-20261023/finish-nonzero-smoke-v13.py`. It requires
CLI exit 1, expected version `1.0`, version denied, reference-leak/relocation
not-run, built/cached counts zero, failed count one, empty successful outcomes,
and no action-result publication phase. No source behavior or base fixture
changes are needed. The existing positive runtime test supplies the zero-exit
control; the standalone smoke runs both variants against the pinned CLI.

Existing named coverage for the other T3.2 cases (inventory, not new execution
claims):

| T3.2 case | Existing named test | Boundary |
| --- | --- | --- |
| Wrong version, missing binary without opt-out, failed relocation rerun | `real_finish_gate_denies_wrong_version_missing_binary_and_relocation_failure` | Real sandboxed CLI; asserts version/relocation denial as appropriate. |
| Builder default version mismatch | `real_builder_default_requires_matching_executable_version` | Real sandboxed builder-default path. |
| Cross leak policy | `finish_gates::tests::reference_policy_reports_native_but_denies_cross_and_build_only_hits` | Pure policy with `cross_build=true`; also exercises native report and explicit native deny. |
| Build-only wrapper leak and native report hit | `real_finish_gate_denies_build_tree_wrapper_leak_and_reports_native_prefix` | Real sandboxed CLI, explicit build-platform deny path, and a native finding reported without denial. |
| Undeclared missing dlopen | `finish_gates::tests::real_glibc_audit_distinguishes_missing_and_opened_sonames` | `mantle_native_dlopen`-gated host glibc probe and actual audit events passed to report evaluation; asserts denied before optional-soname adoption and passed after. |
| Audit evidence cannot merge success across runs | `finish_gates::tests::audit_missing_soname_cannot_be_satisfied_by_another_run` | Pure audit parser; also checks malformed audit records. |
| Oversized or malformed gate declaration | `finish_gates::tests::rejects_unbounded_gate_input` | Pure admission returns `DeclarationTooLarge` and `InvalidDeclaration`. |
| Unsafe/missing command and invalid gate dependencies | `finish_gates::tests::declaration_denials_distinguish_unsafe_commands_and_gate_dependencies` | Pure typed admission denials. |
| Reference bounds and absent execution evidence | `finish_gates::tests::reference_bounds_and_required_execution_evidence_fail_closed` | Pure boundary/evidence checks. |

Scope gaps remain explicit: the cross-build flag is exercised by pure policy,
not a real cross-toolchain CLI fixture; missing dlopen is exercised by a
host-loader/report test, not a sandboxed CLI failure/publication test; malformed
and oversized declarations are checked at pure admission, not through CLI
reporting. Do not infer those broader end-to-end claims from the inventory.
`real_builder_exit_and_shell_syntax_cannot_skip_required_version` additionally
guards gate bypass and publication on child-builder failure, but does not
replace the new matching-text/nonzero-exit case.

The source-only worker handed these commands to Main without running checks.
The post-integration execution and retained CLI reuse are recorded below:

```sh
nix develop -c cargo test -p mantle --test finish_gates real_finish_gate_denies_matching_version_with_nonzero_exit_without_publication -- --exact --nocapture
nix develop -c cargo test -p mantle --test finish_gates -- --nocapture
nix develop -c cargo test -p crunch-build --lib finish_gates -- --nocapture
python3 target/cairn-finish-20261023/finish-nonzero-smoke-v13.py
```

Require actual sandbox execution, not capability-skipped tests; retain the
native-dlopen configuration/availability for the conditional host-loader case.
The pinned CLI smoke must retain its zero-exit passing control and nonzero
denied result with zero built, one failed, no outcomes/publication, and unchanged
executable binding. The permanent regression additionally asserts zero cached.
`evidence/nonzero-exit-fixture-2026-10-26.json` records six core fixtures and
eight real sandbox fixtures passing, including the new nonzero case. The actual
glibc audit test was compiled and executed; no fixture skipped. Format, strict
Clippy and both adoption-receipt typechecks/boundary fixtures also exited 0.
The existing completed v8 CLI smoke retains a passing zero-exit control and
matching-text/nonzero denial with no outcomes/publication. Its output and pin
digests were checked; it was not overwritten or re-executed. Pueue task 28's
attempted duplicate smoke launch refused the existing directory before running
the CLI and remains exit 1, not a newly passed smoke.

T3.2 is checked for the named fixture delivery only. Historical failures, the
pure-policy/host-loader scope limits above and all ADR, bootstrap adoption,
full-gate, spec-sync, archive and external-authority boundaries remain unchanged.
