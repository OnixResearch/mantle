## Phase 1: Proof integration

- [x] [depends:bind-full-source-rust-provider] I1 Define a pure full-bootstrap proof plan over authenticated source authority, StageX lineage, native/Rust provider construction, immutable closure policy, Cargo-free stage execution, strict hermeticity, protected execution, bounded resources, and v2 receipt fields. r[bootstrap_inventory.source_built_mantle_fixed_point]
  - Evidence: `src/source_built_fixed_point.rs` defines the domain-separated six-stage plan, exact source roles, empty-output precondition, strict policy, resource bounds, stage1-owned stage2 orchestration, v2 receipt-field contract, and bounded non-claims. Focused validation passed 9 positive and negative tests plus strict first-party Clippy in pueue task `4428`. ADR 0050 records the composition boundary.
- [x] [depends:materialize-stagex-lineage-provider] I2 Extend source-bundle export/hydration so a fresh proof root receives every declared native and Rust source record plus vendor inputs, starts with empty provider/Mantle output authorities, and cannot admit provider outputs as proof inputs. r[bootstrap_inventory.source_built_mantle_fixed_point]
  - Evidence: `source-built-fixed-point` source-bundle mode labels the audited hex0 seed and validated StageX lineage as source authority, requires materialized native fetch records, Mantle source, vendor inputs, and Rust proof inputs, and rejects a provider manifest substitution. Hydration imports and pins every record, publishes only `vendor-deps/`, and leaves transition/provider/output directories absent. Four focused positive and negative tests passed; the full 70-test source-bundle suite and strict first-party Clippy passed in pueue tasks `4476`, `4470`, and `4473`.
- [ ] [serial] I3 Implement the shell that constructs the StageX native provider and full-source-bound Rust provider from those sources, then builds stage1 Mantle through the receipt-bound Cargo-free topology under strict no-fetch/no-fallback policy. Before execution, emit a complete root-scoped action trust plan with fixed or producer-linked executable authority, input authority, producer edges, local-only execution, event-count bounds, and resource limits. Reject incomplete action adapters and path-only generated authority. r[bootstrap_inventory.source_built_mantle_fixed_point]
- [ ] [serial] I4 Invoke the produced stage1 Mantle to build stage2 through the same plan and closure policy, compare BLAKE3 binary digests, and reconcile every protected-exec and build execution record with the planned action, executable, producer, locality, and count bounds. Fail closed on unknown or missing events, authority, closure, effect, smoke, or digest drift. r[bootstrap_inventory.source_built_mantle_fixed_point]
- [ ] [serial] I5 Emit and verify `mantle-deterministic-proof-receipt-v2`, durable failed-attempt evidence, provider-kind linkage, action-trust plan and observation digests, planned-versus-observed reconciliation, release-evidence inputs, and atomic successful-proof alias updates. Expose the bound result through `mantle --json bootstrap trust-report --proof-root <path>`. r[bootstrap_inventory.source_built_mantle_fixed_point]

## Phase 2: Verification

- [ ] [serial] V1 Add positive pure-plan, action-trust, reconciliation, and receipt tests. Add negative tests for missing source records, nonempty initial provider/output authority, imported or cache-hit provider completion, incomplete action adapters, path-only generated authority, producerless generated executables, unknown or missing execution events, executable digest drift, producer drift, event-count overflow, remote or cache-only execution, undeclared input authority, v1 receipt, wrong provider kind, closure drift, Cargo invocation, live fetch, ambient tool/read, protected-exec violation, practical fallback, stage2 host orchestration, output mismatch, timeout, disk preflight failure, and partial alias publication. Run focused `cargo_free_self_build`, `source_toolchain_closure`, `self_build`, release determinism, and bootstrap parity tests. r[bootstrap_inventory.source_built_mantle_fixed_point]
- [ ] [serial] V2 Run the full proof on Leviathan (`leviathan.cymric-daggertooth.ts.net`) as the sole execution host. Use a fresh hydrated source root and set `CRUNCH_NO_FUSE=1`. Before launch, retain exact source and profile transfer parity. Also retain observed host, system, architecture, kernel, resource, sandbox-tool, and pueue-task facts. Use SSH, rsync, and pueue only for outer transfer, launch, observation, and evidence retrieval. Do not use Mantle `--builder`, Nix remote-action dispatch, split-host stages, or route-name trust. Preserve exact provider, closure, stage, smoke, action-trust, reconciliation, audit, v2 receipt, and BLAKE3 evidence. Require stage1/stage2 binary equality and complete planned-versus-observed action coverage. Require all proof actions to execute locally on Leviathan. Require zero unknown events, missing events, live fetches, Cargo invocations, or fallback events. Require strict hermeticity. r[bootstrap_inventory.source_built_mantle_fixed_point]

- [ ] [serial] V3 Run `nix develop -c cargo test -p mantle --test release_cli`, `nix develop -c cargo test -p mantle --test bootstrap_parity_cli`, focused action-trust core and CLI tests, the machine-contract checker, `nix develop -c cargo fmt --check -p mantle -v`, focused first-party Clippy, `git diff --check`, release-evidence verification, Cairn validation, and all three gates; do not mark the proof complete from preflight, a partial trust report, provider smoke, one-shot build, or a mismatched fixed point. r[bootstrap_inventory.source_built_mantle_fixed_point]

## Current source handoff (2026-10-23)

No build, test, formatter, or proof was run by this source/provider slice.
Earlier receipts and checked historical tasks do not accept the current tree.
`src/source_built_fixed_point_shell.rs` now validates all declared resource
bounds before hydration, and the current cold route rejects inherited compiler
cache/scheduling authority. See the current source-built section of
`docs/operator-proof-guide.md` for the two distinct preparation/proof commands,
profile identity handoff, 500 GiB example budget, and independent receipt and
parity gates. Preparation must start from unexpanded authenticated source and
publishes `source-profile.json` plus `source-program-preparation.json`; its
output profile identity replaces the input profile identity at proof launch.

Current host access is not the old blocker: Leviathan is reachable through the
configured SSH route. Its retained `proof-inputs/` contains StageX, native, and
Rust host-tool source manifests and Rust archive inputs. Those retained inputs
are not a qualified current cohort: the final frozen Mantle/lock/vendor source
must be rebound, every current source declaration covered, and inert programs
prepared and reviewed before the cold proof. The historical native output pin
in `bootstrap/evidence/full-source-provider-admission.json` remains an explicit
expected value, not an observation of this tree. Do not replace it with an
unreviewed output observed from the same run merely to obtain success.

The integration owner must run the source-core/source-bundle/source-observation
and source-built shell checks, then execute the real Leviathan cold proof and
validate its v2 receipt, complete action reconciliation, stage1/stage2 binary
equality, provider/closure links, and release evidence. I3–I5 and V1–V3 remain
unaccepted until that current evidence exists; old wrappers, provider-only
smokes, and successful source preparation do not substitute for it.

## Source-authoring handoff (2026-11-06)

The resolver and successor cc overlay repairs have focused test and real helper
execution evidence in [bounded minicargo resolution](evidence/2026-11-06-bounded-minicargo-resolution.md).
Real preparation now stops at the macro-continuation capacity boundary, not the
earlier version search or deleted-transformer panic. I3–I5 and V1–V3 remain
unchecked: this handoff does not accept preparation, a provider, or a cold proof.

## Compact macro handoff (2026-11-06)

The compact controller and explicit deferred-command namespace handoff have
author/protocol tests, real protected execution and focused Clippy/build evidence
in [compact macro continuations](evidence/2026-11-06-compact-macro-continuations.md).
Pueue 64 reran real preparation with unchanged profile pins and resource bounds;
it passed the prior macro-capacity boundary and now rejects ambiguous original
vendor identity `memchr 2.7.1` in successor source topology. No prepared profile,
provider or fixed-point proof is accepted; I3–I5 and V1–V3 remain unchecked.

## Successor resolution handoff (2026-11-06)

The consuming-workspace lock/patch repair has failing-before/passing-after
behavioral regressions and focused Clippy/build evidence in
[successor resolution](evidence/2026-11-06-successor-resolution.md). Real
preparation in Pueue 31 passed the stale `memchr 2.7.1` selection and reached
the fixed target's unclassified `target_has_atomic = "64"` predicate. Source
pins and resource bounds were unchanged. No prepared profile or cold proof is
accepted; I3–I5 and V1–V3 remain unchecked.

## Successor admission checkpoint (2026-11-06)

Consuming lock/patch scope, fixed native cfg selection, Cargo-feature namespace
separation and lossless producer labels now have final focused execution evidence
in [successor resolution](evidence/2026-11-06-successor-resolution.md).
Pueue 86 passed 2 label-admission and 15 successor-resolution tests, focused
release Clippy and CLI build. Real preparation in Pueue 88 passed the preceding
admission boundaries and now stops at `source recipe capacity exhausted before
original LLVM graph`. Existing bounds, reservations and profile pins were not
weakened. No prepared profile or provider/fixed-point acceptance is implied;
I3–I5 and V1–V3 remain unchecked.
