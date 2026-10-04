# Tasks: Select the store backend explicitly

T1.1 is checked from the isolated pre-selection `7ec51777` capture and the
combined same-root strict Snix comparison in
`evidence/prechange-snix-golden-2026-10-04.md` and
`evidence/finish-conformance-2026-10-04.md`; the baseline construction and
launcher inventory is in `evidence/finish-inventory-2026-10-04.md`.
T1.2–T1.4 are checked from the backend grammar/profile and pure identity
decision in `crates/crunch-store/src/backend.rs` and `overlay.rs`, the declared
blockers and profile in `docs/store-backends.md`, and the still-Proposed ADR
0082 with its `adr/README.md` index row. T4.1 remains checked from Run 5 in
`evidence/test-runs-2026-09-30.md`; T4.2 is checked from the operator
documentation and `README.md` index. The **first preserved** supplemental
same-key 7ec three-path rail capture supplies signed core paths, NARs,
reopen/closure/reuse, archive and two-signer facts. The parameterized
Snix/Casita core rail compares historical GC consumer facts after sorting
only historical blob-index/blob-chunk observation paths within each
category, as the selected source does; it passes at the historical fixed
root and portable roots. All historical reruns and differing old
`read_dir` orders are retained as evidence, not alternate golden inputs.
Exact numerical prechange GC plan-ID parity is claimed only for the
**original two-path** fixed-root baseline-keep/candidate T1.1 golden, not
the supplemental three-path rail. Spec lines 181–189 bind `snix` parity
to goldens recorded **before** this change: that is the original T1.1
two-path capture. The later separately keyed three-path capture supplies
additional signed/NAR and canonicalized consumer-fact evidence, but its
prechange `read_dir`-dependent numeric ID was not a T1.1 golden and is
not asserted equal to the selected canonical ID. This bounded non-claim
alone does **not** prevent T3.1 from passing once its optional and bound
fixtures actually run on the backend-parameterized rail. T3.1 remains
open because those branches are not yet proven on that rail, not because
a favorable old three-path order has yet to be found. T2.1/T2.2 and
T2.4–T2.7 have combined source, CLI, and library proof, including a
production-default constructor without the synthetic injected-service seam;
T3.4 has the original two-path exact golden, real legacy and identity-less
Snix signed reopen, file invariants, and all-Snix overlay composition.
The five launcher routes are wired but local-worker and fixed-point
children have not all run, so T2.3 stays open. T3.2 retains the literal
runtime blocker/unchanged-state request for a `StoreConfig` without a
backend: this type cannot be constructed without one. T3.3's disabled
test-only Rust-cache profile and action-result reuse have not been proven
in one fixture; separate Casita reuse is not that fixture. T4.3 stays
open until the final post-source full scoped suites and strict Clippy
finish; its original checker failure and subsequent zero-escape passes
are recorded. T4.4 archive/sync waits for every prerequisite, and ADR
0082 remains Proposed. A passing structural Cairn gate is not archive
or implementation acceptance.

## Phase 1: Baseline and contract

- [x] [serial] T1.1 Record the baseline in an isolated worktree from current `origin/main` with an explicitly provisioned fixture signing key and a fixed environment, both recorded in `evidence/`: store paths, NAR SHA-256, signed PathInfo bytes, GC plan identities, and `store info`, `store roots`, and `store gc --dry-run` JSON for the Snix fixtures; the `store-identity.json` bytes of a fresh state directory; every `StoreConfig` and `StoreHandle::open` construction site; and every launcher that forwards `--state-dir` or `--store-prefix`. Preserve exact output in `evidence/`. r[mantle.store_backends.admission_invariants]
- [x] [serial] T1.2 Define the backend identifier grammar, the versioned identity record, the per-backend persistent-state markers, the pure open decision, and the blocker catalog `store-backend-unknown` and `store-backend-mismatch`. r[mantle.store_backends.state_identity] r[mantle.store_backends.mixed_open_rejection]
- [x] [serial] T1.3 Define the capability profile: the core capability list (including `store sign` and PathInfo-backed `ActionResultPort` output storage and reuse), the per-backend `store-repair-final-nar` entry and its fail-closed rule, the optional capabilities `overlay-composition`, `atomic-batch-import` with a declared maximum batch size, `unsigned-admission`, and `rust-unit-cache`, and the rule for backend-specific blockers. r[mantle.store_backends.capability_profile]
- [x] [serial] T1.4 Record the explicit-selection, recorded-identity, capability-profile, and no-local-fallback decisions in ADR 0082 with an index row in `adr/README.md`. r[mantle.store_backends.no_silent_fallback]

## Phase 2: Selection, identity, and profiles

- [x] [serial] T2.1 Add the Mantle-owned backend identifier, require it in `StoreConfig`, remove every constructor default, and migrate every construction site in `src/`, `crates/`, and `tests/`. r[mantle.store_backends.explicit_selection]
- [x] [serial] T2.2 Add the global `--store-backend` option with default `snix` at the CLI composition root only, and pass it through `RunContext` to every store-opening command. r[mantle.store_backends.explicit_selection]
- [ ] [serial] T2.3 Forward the selected identifier from the local remote worker launcher, bootstrap validation, the source-built fixed-point shell, the transcript command, and the Rust cache daemon. r[mantle.store_backends.explicit_selection]
- [x] [serial] T2.4 Write the versioned identity record for new state directories, read legacy records as `snix` without rewriting them, under `snix`, add only a `snix` identity record to a populated directory without an identity record or `casita` marker, changing no existing file before the Snix services open, reject an identity-less `casita` marker under either backend and identity-less content outside the allowlist under `casita`, and apply the pure open decision before any directory creation, lock acquisition, identity write, or service open. r[mantle.store_backends.state_identity] r[mantle.store_backends.mixed_open_rejection]
- [x] [serial] T2.5 Decide every overlay layer's recorded backend before any base read. r[mantle.store_backends.mixed_open_rejection]
- [x] [serial] T2.6 Add the `snix` profile with `store-repair-final-nar` in its core list and all four optional capabilities, including `rust-unit-cache`, and no backend bound on atomic batch import (the Nario v2 reader's 100,000-record limit applies to every backend and is not a profile bound), check requested optional capabilities before any state access and declared bounds before any state mutation, and report the profile and bounds in `store info` JSON and human output. r[mantle.store_backends.capability_profile]
- [x] [serial] T2.7 Confirm that no lookup, closure, export, GC, repair, or inspection path reads another local backend's state, and keep remote substitution on its existing policy. Prove it with a behavioral fixture: a `snix` state directory seeded with leftover files in another backend's layout reports a miss or rebuilds through `snix` and never reads the leftover files. r[mantle.store_backends.no_silent_fallback]

## Phase 3: Conformance and negative controls

- [ ] [serial] T3.1 Build the backend-parameterized conformance rail for the core capabilities, stale-plan rejection, identity checks, and profile-driven optional capability and bound fixtures. Run it with `snix` using the T1.1 fixture signing key and environment, and compare every result with the T1.1 goldens, including signed PathInfo. Add the cross-directory comparison: two `snix` state directories with one provisioned signing key produce equal signatures, and with different keys produce equal unsigned fields and signatures that verify under their own keys. r[mantle.store_backends.conformance_rail] r[mantle.store_backends.admission_invariants]
- [ ] [parallel] T3.2 Add negative fixtures: unknown identifier, recorded-backend mismatch, an identity-less `casita` marker under `snix` and under `casita` (alone and beside Snix files), identity-less content outside the allowlist under `casita`, mixed overlay layers, a launcher that drops the identifier, a `StoreConfig` without a backend, and an environment variable that tries to select a backend. Each asserts its stable blocker and a byte-identical state directory. r[mantle.store_backends.mixed_open_rejection] r[mantle.store_backends.explicit_selection]
- [ ] [parallel] T3.3 Add profile fixtures: `snix` overlay composition, atomic batch import, unsigned admission, and the Rust unit cache pass; a test-only profile without overlay composition fails closed before any layer is opened; a test-only profile with batch bound N accepts N paths and rejects N + 1 paths before any state mutation; a test-only profile without `rust-unit-cache` rejects Rust unit cache use before any effect while PathInfo-backed action-result outputs still reuse; and a profile without `store-repair-final-nar` rejects `store repair-final-nar`, as a dry run and with `--execute`, before any state access and the library repair calls before any effect, while `snix` repair is unchanged (the `casita` profile is such a profile; its fixture is T2.10 of `adopt-casita-store-backend`). r[mantle.store_backends.capability_profile]
- [x] [parallel] T3.4 Add positive fixtures: default and explicit `snix` equal the baseline under the T1.1 fixture signing key, a new state directory records `snix`, a legacy identity opens unchanged, a populated identity-less Snix directory without a `casita` marker gains only a `snix` identity with every existing file unchanged before the Snix services open, and after the open its files other than the Snix databases stay byte-identical and its PathInfo still resolves, and all-`snix` overlay layers compose. r[mantle.store_backends.state_identity]

## Phase 4: Surfaces, documentation, and verification

- [x] [serial] T4.1 Regenerate the operator command contract artifacts with the repository generators and run `scripts/check-operator-command-contract.sh`. Do not hand-edit generated JSON. r[mantle.store_backends.explicit_selection]
- [x] [serial] T4.2 Document selection, the identity record, the asymmetric rule for directories without an identity record and its provenance risk, capability profiles and bounds, mismatch remediation, the local no-fallback rule, the signing-key rule for signed comparisons, and non-claims in the store documentation and the README index. r[mantle.store_backends.claim_boundary]
- [ ] [serial] T4.3 Run the focused `crunch-store` and `mantle` store suites, `tools/check_store_capability_boundary.rs`, strict Clippy and rustfmt for touched first-party packages, `git diff --check`, Cairn validation, and the proposal, design, and tasks gates. Preserve exact output in `evidence/`. r[mantle.store_backends.conformance_rail]
- [ ] [serial] T4.4 Sync accepted specs and archive through the isolated branch workflow only after every task above is complete, with retained completion evidence. r[mantle.store_backends.claim_boundary]
