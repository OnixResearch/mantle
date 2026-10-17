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
Exact numerical prechange GC plan-ID parity is required and demonstrated
for the **original two-path** fixed-root baseline-keep/candidate T1.1
golden, not the supplemental three-path rail. The explicitly approved
spec amendment compares only the first preserved 7ec three-path
capture's canonical GC consumer facts, after sorting historical
blob-index/blob-chunk paths within each category, while all signed
PathInfo, NAR, store paths, candidates, retention, output behavior,
and negative blocker checks remain exact. Its first `fresh` raw plan
ID `b3:36eb6141fa768f053c08258dc684365f28e908643a2e51bffcb63a0b7cf31491`
is **not equal** to selected canonical
`b3:c73dcda6e8949135b7d49298cd219c3845e8eb6e18653d60b8cd5bb9c8b90e95`.
All 7ec recaptures remain historical counterexamples, never alternate
golden inputs. T3.1 is checked from the published-source real
signed/optional/bounded Snix/Casita rail at the first historical
physical root (**1/1**) and default/explicit Snix at the original
two-path fixed root (**1/1**), with exact non-equal first-7ec
supplemental raw IDs retained in the receipt.

T2.1/T2.2 and T2.4–T2.7 have combined source, CLI, and library proof, including a
production-default constructor without the synthetic injected-service seam;
T3.4 has the original two-path exact golden, real legacy and identity-less
Snix signed reopen, file invariants, and all-Snix overlay composition.
All five launcher routes forward the selected identifier in their source
contracts; bounded bootstrap, transcript, and Rust-cache daemon children
were exercised. T2.3 is checked for forwarding, **not** for executing a
local-remote-worker build or the source-built fixed-point proof; neither
unbounded route was launched in this finish pass. A real seeded `snix`
state remains byte-identical when a separate `StoreConfig` caller
without `backend` fails compilation with `E0063`; this is a compiler
diagnostic, never a runtime backend blocker. T3.2 is checked from the
full byte-preserving negative CLI matrix, real dropped-identifier child,
and the separate observed compiler rejection. T3.3's one test-only
profile with Rust-cache disabled and *real* signed PathInfo action-result
reuse passed on the combined checkout. The corrected fixed-root
Snix/Casita rail also passed **1/1** with the baseline signer,
locale/time, and a quota-safe pinned `TMPDIR`: real Snix overlay,
atomic batch, unsigned import and Rust-cache operations and actual
Casita 1,024-accept/1,025-before-mutation bounds and fail-closed
checks. The earlier unpinned `/tmp` quota failure is retained as an
environment-only red run, not a source defect. T3.3 is checked. T4.3 is
checked from the post-cherry store-core 405/405, archive CLI 14/14, GC
CLI 17/17, integration 77/77, focused Mantle bin selectors
`store_backend` 1/1, `store_cmd` 6/6, and `rust_cache` 4/4, original
two-path fixed-root golden, first-party strict Clippy and rustfmt on
published source `61bd4465`, the 558-file zero-escape checker, diff
check and pinned-policy Cairn gates. The broad Mantle bin run's two
unrelated Slurm/seccomp failures, vendored-path lint failures, and
sandbox-body skips remain nonclaims, not a green repository-wide suite.
T4.4's **pre-archive** milestone was checked only after the revised
implementation/change commit `c74edfec`, the pinned-policy preview
and actual accepted-spec sync into `.cairn/specs/store-backends/`,
post-sync validation/tasks gates, and the archive preview showing
only T4.4 itself as the remaining prerequisite. This checkbox never
claimed archive execution. Separately, **after** all 19
pre-archive tasks and the synced accepted spec were committed at
`d1b9dcf4`, the named isolated-branch Cairn archive was actually
executed; its mutation receipt and passing post-archive validation
are recorded in retained archived
`evidence/finish-conformance-2026-10-04.md`. ADR 0082 remains
Proposed while the separate Casita adoption has 28 open tasks.

## Phase 1: Baseline and contract

- [x] [serial] T1.1 Record the baseline in an isolated worktree from current `origin/main` with an explicitly provisioned fixture signing key and a fixed environment, both recorded in `evidence/`: store paths, NAR SHA-256, signed PathInfo bytes, GC plan identities, and `store info`, `store roots`, and `store gc --dry-run` JSON for the Snix fixtures; the `store-identity.json` bytes of a fresh state directory; every `StoreConfig` and `StoreHandle::open` construction site; and every launcher that forwards `--state-dir` or `--store-prefix`. Preserve exact output in `evidence/`. r[mantle.store_backends.admission_invariants]
- [x] [serial] T1.2 Define the backend identifier grammar, the versioned identity record, the per-backend persistent-state markers, the pure open decision, and the blocker catalog `store-backend-unknown` and `store-backend-mismatch`. r[mantle.store_backends.state_identity] r[mantle.store_backends.mixed_open_rejection]
- [x] [serial] T1.3 Define the capability profile: the core capability list (including `store sign` and PathInfo-backed `ActionResultPort` output storage and reuse), the per-backend `store-repair-final-nar` entry and its fail-closed rule, the optional capabilities `overlay-composition`, `atomic-batch-import` with a declared maximum batch size, `unsigned-admission`, and `rust-unit-cache`, and the rule for backend-specific blockers. r[mantle.store_backends.capability_profile]
- [x] [serial] T1.4 Record the explicit-selection, recorded-identity, capability-profile, and no-local-fallback decisions in ADR 0082 with an index row in `adr/README.md`. r[mantle.store_backends.no_silent_fallback]

## Phase 2: Selection, identity, and profiles

- [x] [serial] T2.1 Add the Mantle-owned backend identifier, require it in `StoreConfig`, remove every constructor default, and migrate every construction site in `src/`, `crates/`, and `tests/`. r[mantle.store_backends.explicit_selection]
- [x] [serial] T2.2 Add the global `--store-backend` option with default `snix` at the CLI composition root only, and pass it through `RunContext` to every store-opening command. r[mantle.store_backends.explicit_selection]
- [x] [serial] T2.3 Forward the selected identifier from the local remote worker launcher, bootstrap validation, the source-built fixed-point shell, the transcript command, and the Rust cache daemon. r[mantle.store_backends.explicit_selection]
- [x] [serial] T2.4 Write the versioned identity record for new state directories, read legacy records as `snix` without rewriting them, under `snix`, add only a `snix` identity record to a populated directory without an identity record or `casita` marker, changing no existing file before the Snix services open, reject an identity-less `casita` marker under either backend and identity-less content outside the allowlist under `casita`, and apply the pure open decision before any directory creation, lock acquisition, identity write, or service open. r[mantle.store_backends.state_identity] r[mantle.store_backends.mixed_open_rejection]
- [x] [serial] T2.5 Decide every overlay layer's recorded backend before any base read. r[mantle.store_backends.mixed_open_rejection]
- [x] [serial] T2.6 Add the `snix` profile with `store-repair-final-nar` in its core list and all four optional capabilities, including `rust-unit-cache`, and no backend bound on atomic batch import (the Nario v2 reader's 100,000-record limit applies to every backend and is not a profile bound), check requested optional capabilities before any state access and declared bounds before any state mutation, and report the profile and bounds in `store info` JSON and human output. r[mantle.store_backends.capability_profile]
- [x] [serial] T2.7 Confirm that no lookup, closure, export, GC, repair, or inspection path reads another local backend's state, and keep remote substitution on its existing policy. Prove it with a behavioral fixture: a `snix` state directory seeded with leftover files in another backend's layout reports a miss or rebuilds through `snix` and never reads the leftover files. r[mantle.store_backends.no_silent_fallback]

## Phase 3: Conformance and negative controls

- [x] [serial] T3.1 Build the backend-parameterized conformance rail for the core capabilities, stale-plan rejection, identity checks, and profile-driven optional capability and bound fixtures. Run it with `snix` using the recorded fixture signing key and environment; require exact signed PathInfo, NAR and deterministic facts, including the original T1.1 two-path same-root numerical GC plan-ID golden. Compare only the first preserved 7ec supplemental three-path capture's canonical GC consumer facts; preserve unequal raw plan IDs and never substitute a favorable recapture. Add the cross-directory comparison: two `snix` state directories with one provisioned signing key produce equal signatures, and with different keys produce equal unsigned fields and signatures that verify under their own keys. r[mantle.store_backends.conformance_rail] r[mantle.store_backends.admission_invariants]
- [x] [parallel] T3.2 Add negative fixtures: unknown identifier, recorded-backend mismatch, an identity-less `casita` marker under `snix` and under `casita` (alone and beside Snix files), identity-less content outside the allowlist under `casita`, mixed overlay layers, a launcher that drops the identifier, an environment variable that tries to select a backend, and a compile-fail `StoreConfig` literal without `backend`. Runtime negatives assert their stable blocker and byte-identical state; the omitted-field fixture asserts Rust `E0063` (`missing field backend`) and byte-identical real seeded state, not a fabricated runtime blocker. r[mantle.store_backends.mixed_open_rejection] r[mantle.store_backends.explicit_selection]
- [x] [parallel] T3.3 Add profile fixtures: `snix` overlay composition, atomic batch import, unsigned admission, and the Rust unit cache pass; a test-only profile without overlay composition fails closed before any layer is opened; a test-only profile with batch bound N accepts N paths and rejects N + 1 paths before any state mutation; a test-only profile without `rust-unit-cache` rejects Rust unit cache use before any effect while PathInfo-backed action-result outputs still reuse; and a profile without `store-repair-final-nar` rejects `store repair-final-nar`, as a dry run and with `--execute`, before any state access and the library repair calls before any effect, while `snix` repair is unchanged (the `casita` profile is such a profile; its fixture is T2.10 of `adopt-casita-store-backend`). r[mantle.store_backends.capability_profile]
- [x] [parallel] T3.4 Add positive fixtures: default and explicit `snix` equal the baseline under the T1.1 fixture signing key, a new state directory records `snix`, a legacy identity opens unchanged, a populated identity-less Snix directory without a `casita` marker gains only a `snix` identity with every existing file unchanged before the Snix services open, and after the open its files other than the Snix databases stay byte-identical and its PathInfo still resolves, and all-`snix` overlay layers compose. r[mantle.store_backends.state_identity]

## Phase 4: Surfaces, documentation, and verification

- [x] [serial] T4.1 Regenerate the operator command contract artifacts with the repository generators and run `scripts/check-operator-command-contract.sh`. Do not hand-edit generated JSON. r[mantle.store_backends.explicit_selection]
- [x] [serial] T4.2 Document selection, the identity record, the asymmetric rule for directories without an identity record and its provenance risk, capability profiles and bounds, mismatch remediation, the local no-fallback rule, the signing-key rule for signed comparisons, and non-claims in the store documentation and the README index. r[mantle.store_backends.claim_boundary]
- [x] [serial] T4.3 Run the focused `crunch-store` and `mantle` store suites, `tools/check_store_capability_boundary.rs`, strict Clippy and rustfmt for touched first-party packages, `git diff --check`, Cairn validation, and the proposal, design, and tasks gates. Preserve exact output in `evidence/`. r[mantle.store_backends.conformance_rail]
- [x] [serial] T4.4 Complete pre-archive readiness only after T3.1 and every preceding task: commit the implementation and revised change, preview and sync accepted specs under the explicitly pinned Cairn policy, retain the sync mutation receipt and validation/gate results, and confirm that all implementation tasks and dependencies are ready for the actual archive. This checkbox never claims archive execution; actual isolated-branch `cairn archive` and post-archive validation/receipt remain mandatory separate lifecycle steps under the proposal and design. r[mantle.store_backends.claim_boundary]
