# I6 import plan/apply effect slice (2026-10-01)

Scope: `mantle import pins plan/apply` and `mantle import cargo --plan/--apply`. This is not I6 completion evidence or a claim of durable publication, rollback, external source correctness, or reproducible builds.

## Before-cutover CLI reproduction

The previously built `/home/brittonr/scratch/mantle-pins-bin-target/debug/mantle` was invoked against a scratch source workspace whose `.mantle` output parent symlinked to a separate scratch directory. Both `import pins apply` and `import cargo --apply` returned success and wrote the external directory's `inputs.ncl` (`escaped_write=yes`). The pre-cutover pin blocker scenario emitted the plan and an additional misleading Internal JSON error. These are real CLI observations, not a hypothetical filesystem threat.

## Changes under review

Each selected plan declares bounded, typed input-read and (for apply) output-write and read-back effects before opening the import workspace. The filesystem input and output adapters implement explicit capability-error ports. The output port opens the selected root and each parent without following symlinks, refuses overlapping selected targets before the first write, creates only new regular files, and compares independently reopened output bytes and the Linux opened-descriptor target identity to the planned destinations. An apply does not claim success on a mismatched path or bytes. A failed later write or read-back can leave earlier files in place: no rollback is claimed. Domain blockers classify as failed decisions, emit their reviewed plan once, and exit 3 without a second Internal error or an `Applied` heading.

Focused regressions cover symlinked output parents for both importers, overlapping paths before mutation, reviewed-plan byte equality and repeated apply, blocker terminal output, and a read-back adapter that opens another real file with identical bytes instead of the planned target.

## Verification boundary

Before-cutover escaped writes were reproduced with the old scratch binary as
above. Initial after-change compilation stopped on transient project/source
trait and API errors; a later pin/lock attempt stopped on the Nickel export
owner's in-flight cutover. Those owners corrected their sources. None of those
attempts emitted an import-owned compiler diagnostic.

Using `TMPDIR=/home/brittonr/scratch` and
`CARGO_TARGET_DIR=/home/brittonr/scratch/mantle-i6-import-target`, the
`nix develop --offline --no-write-lock-file -c cargo test -p mantle --locked
--offline --test pin_import_cli --test lock_importer_offline_rail --
--test-threads 1` run completed: **pin CLI 6/6 PASS; offline lock-importer
rail 5/5 PASS**. The prior scoped `cargo_import_cli` run passed **11/11**.
The targeted `cargo test -p mantle --locked --offline --bin mantle
pin_import::tests::readback_checks_real_target_and_bytes_not_just_file_count
-- --test-threads 1` run passed **1/1** (2,634 other bin tests filtered).

Actual CLI smoke used the newly built
`/home/brittonr/scratch/mantle-i6-import-target/debug/mantle` and fresh
scratch fixtures. For pin and Cargo independently, JSON plan then apply
exited 0 and every generated file exactly matched its planned bytes (3 pin
files, 2 Cargo files). Applying with `.mantle` symlinked to an external
directory exited 3 without modifying its sentinel `inputs.ncl` or writing
the project file. JSON apply against a conflicting operator-owned project
exited 3 with a plan blocker, unchanged existing content, and empty stderr
for both importers.

An earlier `examples_workflow_gallery` run passed 11/12, including both
Cargo import rows; its unrelated semantic-graph example failed with
`Contradicted { effect_count: 1 }`. The S0 graph owner retained that issue
and reports that both exact positive and incomplete-evidence gallery tests
subsequently passed 1/1 each after the canonical target fix. This import
record does not claim a full gallery rerun or the shared-root integration
gate; the latter remains with its integration owner.

After these focused runs, strict bin Clippy identified two local
`src/pin_import.rs` style findings: nested `read_checked` return type and a
needless borrowed file name in the no-follow writer. A private tuple alias
and removal of that borrow landed under the root source lease without
changing import behavior. No post-lint-edit Cargo or Clippy result is claimed
while the backend Cargo lock transaction remains unstable; the root owner
has the `IMPORT SOURCE LINT READY` handoff.
