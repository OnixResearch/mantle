# Upstream pilot candidate review, 2026-10-30

This change's `proposal.md` says the `durable-file-publication` pilot "must land first", and
`upstream-pilot-prerequisite-2026-10-27.json` records that no qualifying landing existed. This file
records Mantle's review of a candidate that now exists. **It does not accept the prerequisite, because
nothing has landed.**

## The candidate

Independent repository `/home/brittonr/git/OnixResearch/durable-file-publication` (Radicle `rad://z3tAR4For7qw8ZirkJzoDw1VNDDLM`), branch
`agent/dfp-pilot-evidence-20261030`, tip `fa26782f8fbf30d7ca5f699547fe3f8364ca789e`. It is a strict fast-forward of the local `main`
(`527c7828fa10970d42064460cd7cd15c1c5a6cca`), and exists only as a local branch. Commits on top of `main`:

- `99d4281 Add test-only I/O fault fixture pilot`
- `fa800c7 Strengthen fault fixtures: unwind restore, site-line, collision, no-replace-on-failure`
- `7580716 Fix racy counter reads in fault fixtures`
- `fa26782 Record fixture pilot verification evidence`

The first is the earlier unverified pilot commit. The others were made by a delegated worker and carry its
own evidence in the branch at `.cairn/changes/adopt-fault-injection-io-tests/evidence/fixture-pilot-verification-2026-10-30.md` (BLAKE3 `1788ec3dfa277f3af3aa8fa3ce471ee9349f72642df53374b0a486b8409ba602`).

## What the worker reported, and what was reproduced here

The worker reported that the pilot's fixtures were flaky under the default parallel test harness: two of them
read the process-global fault counter after their guard dropped, without owning it. It reported 96 failures
in 500 parallel runs of the pilot commit, and 0 in 500 at the fix.

This review rebuilt each commit from source (`git archive` into a fresh directory, `cargo test --lib --no-run
--locked --offline`, exit 0 for both) and ran the library test executable with the default parallel harness
300 times. Exact output and the script are retained in
`upstream-pilot-independent-reproduction-2026-10-30.json` (BLAKE3 `3d1a931c4be181f2be011882494fa583e59b259e1729a56ab1f5bf0a2d7ffae7`).

| Commit | Lib tests (serial run) | Parallel runs | Non-zero exits |
|---|---|---|---|
| `99d4281` (pilot as delivered) | `test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` | 300 | **43** |
| `fa26782` (candidate tip) | `test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` | 300 | **0** |

So the defect and its absence at the tip reproduce independently. The rates differ from the worker's (43/300
against 96/500) because they depend on host load and scheduling, so only the contrast is claimed. The pilot
commit's message said "22 passed"; that sum was true only when the race did not strike.

## Mantle's own fixtures

Read statically, not stressed: Mantle's `FaultFixture` in `crates/crunch-store/src/action_result.rs` already
observes restoration while holding the fixture lock (`assert_restored`, with a comment naming the same
race) and arms the counter per thread through `crate::fault_scope`. Its two direct counter reads in
`io_fault_counter_is_serialized_and_unrelated_store_reads_cannot_consume_it` happen while a fixture that
owns the lock is alive. No sibling defect was found.

## What is still not established

- **Upstream Task 10 is incomplete.** The worker's `nix flake check` attempts failed (a remote build hook
  error) or were terminated after 55 minutes with no verdict. The cargo, Clippy, Nickel and Cairn
  equivalents ran outside Nix, which is not a Nix PASS.
- **Not landed.** Landing is the repository owner's step. The owner's `main` checkout has uncommitted changes
  in 15 paths, and three of them (`Cargo.toml`, `README.md`, `flake.nix`) are also changed by the candidate,
  so a fast-forward there needs those reconciled first.
- Only the library test executable was stress-run. No non-Linux target ran.

## Effect on this change

None of Mantle's checkboxes changes. Acceptance of the prerequisite, and therefore archive, still wait for
a qualifying upstream landing record, which Mantle then reviews. This record narrows what that review needs
to look at: the candidate's evidence reproduces, and the open items are the Nix check and the owner's merge.
