# Oracle checkpoint: implementation/test evidence for archived self-package binding

## Question

Review reported that the supplied archive diff did not include `src/rust_plan.rs`, so are the implementation and regression-test task checkboxes justified?

## Inspected evidence

- Implementation commit: `7f5e6da4 keep native self dependencies target-specific`.
- Archive commit: `87a7e328 archive native self dependency binding`.
- `git show --name-only --oneline 7f5e6da4 --` includes `src/rust_plan.rs` plus the active Cairn change files.
- `git show --stat --oneline 7f5e6da4 -- src/rust_plan.rs` reports:

```text
7f5e6da4 keep native self dependencies target-specific
 src/rust_plan.rs | 97 +++++++++++++++++++++++++++++++++++++++++++++++++-------
 1 file changed, 85 insertions(+), 12 deletions(-)
```

- Current source contains the implementation helper and regression test:

```text
3986:fn native_target_dependency_artifacts(
12055:    fn native_unit_graph_filters_package_self_dependency_from_lib_unit() {
```

- Focused suite evidence from pueue task `180` shows the regression test ran in the broader native graph suite:

```text
test rust_plan::tests::native_unit_graph_filters_package_self_dependency_from_lib_unit ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 522 filtered out; finished in 0.00s
```

- Clean probe summary at `target/mantle-self-rust-plan-probe-self-package-clean/blocker-summary.txt` records:

```text
head: 7f5e6da4055d5183c1922d297b62f119934ad221
git_status_short_bytes=0
probe_status=0
topology_execution_status=blocked
executions=602
blocker={"class":"rustc-failed","message":"error[E0463]: can't find crate for `crunch_glue`..."}
```

- Push transcript from pueue task `183`:

```text
To github.com:OnixResearch/mantle.git
   10537c72..87a7e328  main -> main
```

- `git rev-parse HEAD origin/main` after push returned the same full commit ID:

```text
87a7e32865221929ea1f76c2a1641c49a184eff6
87a7e32865221929ea1f76c2a1641c49a184eff6
```

- Cairn archive command created `cairn/archive/1970-01-01-rust-topology-self-package-lib-binding`; per repo guidance, it was manually renamed to `cairn/archive/2026-05-28-rust-topology-self-package-lib-binding` before validation.

## Decision

The implementation and regression-test tasks were legitimately completed in commit `7f5e6da4`. The later archive commit `87a7e328` only moved the active change to archive and synced the canonical spec, so a review limited to the archive commit could miss the code/test diff.

Do not reopen this archived change solely for missing implementation hunks in the archive commit. Treat `7f5e6da4..87a7e328` together as the review range for this change, or use this checkpoint when reviewing only the archive commit.

## Owner

Current agent / Mantle maintainer.

## Next action

Continue with the next native topology frontier: `crunch-system` direct rustc execution cannot find selected normal dependency crate `crunch_glue`.
