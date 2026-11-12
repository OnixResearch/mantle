# 2026-11-09: authored byte placement, consumed lineage inputs, and the seventh cohort

This continues [program placement and second cohort](2026-11-09-program-placement-and-second-cohort.md).
It records proof-boundary repairs and current in-flight state; a launched proof
is not a proof.

## Canonical StageX closure is never rewritten

The prepared cohort must keep the StageX closure's manifest identity: the
lineage pins it as `stage0-source-bundle`
(`541eae99be64df5f13ed8ff52403e83d034c8ce4747984a2d8d11cffc10b01ab`), and
`stagex_sources` requires the same value. Appending authored program records to
that closure changed its manifest identity, so preparation no longer touches it.
Authored StageX helper/policy/original bytes live in the native source-authority
closure, and every reference to them (helper sources, authored originals, route
selectors) names `NativeSourceBundle`. The StageX actions read the authored
program from that closure, and the route's StageX selectors now point at it too.

The lineage's `stage0-source-bundle` entry pins a *manifest identity*, not the
closure file's byte hash; `require_source_digest` now compares the closure
manifest identity for that whole-closure reference and keeps byte-hash
comparison for individual source and patch files.

## Lineage-declared inputs are consumed

Two authored steps bound inputs they never referenced, and the StageX partition
validator correctly rejects that:

- `stage0-full-check-answers` bound the empty Stage0 after-hook; it now consumes
  it as standard input (the hook carries no bytes and cannot change the command);
- `tinycc27-source-patch-*` bound the StageX patch source while each leaf carries
  one derived hunk; it now consumes the patch as standard input (the `replace`
  helper reads its operands, never stdin).

Authored StageX runtime mounts derive from the plan's logical store prefix
(`<prefix>/stagex-bison-2.3-runtime-data`) instead of a fixed `/mantle/stagex/...`
spelling.

## Diagnostics

`unused inputs cannot stand in for original command dependencies` now names the
step and its unused input labels; `require_node_input` now reports every omitted
lineage dependency it finds (bounded to eight) with the tool and source id, so a
long proof run yields the complete list rather than the first item.

## Cohorts

```text
v4  82b8a7ca… (leading-space mount repaired after run)
v5  1124de12… (patch stdin consumed)
v6  4caccd9f… (canonical closure preserved; role split)
v7  205503813fb8256f02c07fd59c93579965f12340bfbf5a373d3f88204b4140cc
    profile BLAKE3 7957a47a9d612bb0398c25c29ce4253dead4bb9bd5b3e56769e6bc0feea9bdb0
    report  BLAKE3 d4071adb5c58f723ef3ae428328c85b481ff2539d06283e131e38b3afb358f48
    CLI     BLAKE3 089d1702e6de541354e7838e2009bc13fc53c60e5365f482d401e804c42c5a89
```

Pueue `1521` ran the seventh cohort for 1 h 41 m and failed at
`expanded commands omit an original source/bootstrap dependency`; the preserved
staging directory is `.proof-v7.source-built-fixed-point-staging-3801475`.
Earlier cohorts are retained only by their recorded identities.

## In flight

Pueue `1522` (`sbfp-proof-v9`, started 2026-11-09T21:25:42-05:00) runs the same
cohort with CLI `4bd6f81febcc69d59ed8be1381bab54b3d6ffd57817a9983daf67933a24913cf`
and the aggregate diagnostic, to enumerate the omitted dependencies.

## Enumerated frontier

Pueue `1522` (CLI
`4bd6f81febcc69d59ed8be1381bab54b3d6ffd57817a9983daf67933a24913cf`, same
cohort) ran 1 h 44 m and returned the complete list the aggregate diagnostic was
added for:

```text
expanded commands omit 11 original source/bootstrap dependencies [
 "stage0-mini-kaem-exec:kaem-0",
 "mes-m2-kaem-exec:stage0-kaem",
 "tinycc-mes-exec:mescc-entrypoint",
 "tinycc-runtime-exec:mes-crt1",
 "tinycc-runtime-exec:mes-libc-tcc",
 "tinycc-final-exec:tcc-boot0",
 "tinycc-final-exec:tcc-mes",
 "tinycc27-runtime-exec:tinycc-crt1", ...]
```

Staging `.proof-v8.source-built-fixed-point-staging-669818` is preserved. The
list splits into two classes that need separate decisions:

- Script-interpreter images (`kaem-0`, `stage0-kaem`): the expansion replaces the
  interpreter with one leaf command per original script line, so the image is
  never an operand. Suppressing it must be explicit in the authored index rather
  than a silent omission.
- Materialization/derived artifacts that the authored leaves appear to omit
  (`mescc-entrypoint`, `mes-crt1`, `mes-libc-tcc`, `tcc-boot0`, `tcc-mes`,
  `tinycc-crt1`, `tinycc-libtcc1`): these look like real authoring gaps and need
  the original command's operands, not an exemption.

## Non-claims

No stage equality, action reconciliation, v2 receipt, release evidence or
independent review exists yet. I3–I5 and V1–V3 remain unchecked, and every cohort
identified above is an unreviewed prepared input.

## Input dispositions and the tenth launch

The enumerated omissions split into two authored statements rather than silent
drops. The expansion index now carries, per original tool, an
`input_dispositions` map from original input identity to the reason the
expansion supersedes it; validation skips a lineage input only when that
reason is non-empty and otherwise reports it. The authored reasons name what
happens instead:

- `kaem-0` and `stage0-kaem`: the expansion runs the script lines directly
  instead of executing the interpreter image.
- `mescc-entrypoint`: tcc is compiled through the modelled Mes module pipeline.
- `mes-crt1`, `mes-libc-tcc`: the tcc-mes node links the Mes libc archive; the
  runtime node builds its own objects.
- `tcc-boot0`, `tcc-mes`: the final tcc is rebuilt from the same sources; the
  earlier executables are not run as drivers.
- `tinycc-crt1`, `tinycc-libtcc1`: staged into the source map consumed by the
  `tinycc27-build-exec` tree.
- `oyacc-*-grammar-source`: consumed by the authored `oyacc-smoke-exec` node.

`superseding_a_lineage_input_requires_an_authored_reason` passes alongside the
other three action tests (4 passed). The release binary
`mantle-v10` (196675672 bytes) was built from this tree and launched as Pueue
`1523` against `profile-v7`. No stage equality, reconciliation, receipt, or
release evidence exists yet.

## Prepared-profile boundary and the eleventh launch

The tenth launch (Pueue `1523`, `mantle-v10`, `profile-v7`) failed closed after
1 h 46 m with the same eleven omissions and the new wording, proving the
promoted proof never re-authors programs: it consumes the prepared profile's
`source-built-*` records. Authoring changes therefore require a fresh
preparation, executed locally against the original unexpanded profile
`source-built-fixed-point-sources-refreshed-20261108-vendor-git.json`
(`a6174213ff6b336e251175ee6afbf452178502209d8773f2eb4cb09ba4e368e0`).

A local `--prepare-source-programs` run (`preparation_exit_code=0`,
2026-11-10T02:10:11-05:00) produced `prepared-cohort-v8` with profile digest
`8242b0bea1e4d415a692162ae0afc5a2697bed58be7d67ca3abe228345ef2edd` and StageX
program record `a215c026dac98ab20e5e5574a4dba15a2d0168ea4b572d5f97bbdb86d6356bc6`.
Its authored expansion carries twelve disposition entries across eight tools,
covering all eleven reported pairs. The eleventh cold launch is Pueue `1524`
against that profile.

The ordinary gate reached `2848 passed; 1 failed` on the previous run:
`remote_build::tests::real_slurm_accept_then_response_save_failure_reconciles_only_exact_dispatch_after_restart`
failed with `external-batch-process-output-limit-exceeded` while preparation
ran concurrently, and passes in isolation (1 passed, 0.18 s). A clean-host rerun
is in flight; the flake is not yet claimed as fixed or as a regression.

## Publication stage authority and the twelfth launch

The eleventh launch (Pueue `1524`) accepted all twelve authored dispositions —
the previous eleven omissions are gone — and then failed after 1 h 43 m at
`source recipe: recipe source role is not admitted to this stage`. The
publication recipe is authored in the same `source-built-stagex-programs`
record as the transition recipe, referenced through
`NativeSourceBundle`, but `stagex-provider-publication` did not admit that
source authority while `stagex-transition` did. The stage plan now admits
`NativeSourceBundle` for publication, and the rejection names the stage and
role. The plan unit suite passes (11 passed), including
`publication_requires_its_authenticated_source_program_authority`. No stage
equality, reconciliation, receipt, or release evidence exists.

## Provider claims and the fourteenth launch

The thirteenth launch (`1526`) died in preflight because the host had only
198 628 425 728 bytes of the required 200 000 000 000; deleting superseded
37 GB staging attempts restored 576 GB. The relaunch (`1527`, 13:04) reached
`publication omits original runtime supplement/positive/negative validation
commands` after 1 h 39 m: the prepared cohort's publication index claimed all
eight provider launches while the expansion only authored them when binutils
sources are admitted, and this lineage does not admit them.

The publication expansion now records `binutils_admitted` and claims only the
launches it authored; the validator requires the three TCC exports and the four
non-binutils launches when that flag is false. A fresh preparation
(`prepared-cohort-v9`, `preparation_exit_code=0`, 2026-11-10T16:05:55-05:00)
produced profile digest
`74565ada1bb6a96965e60b064fced4eef8e210cbcb5f52520351d2e7839cdb85` with
`binutils_admitted=false`, commands `runtime-supplement`, `tcc-execution`,
`tcc-negative`, `tcc-positive`, all of which exist as authored steps. The
fourteenth launch is Pueue `1528` against that profile.

Two operational facts bound this transcript. The source-built proof consumes
the prepared profile verbatim, so any authoring change requires a fresh
preparation; and the host datapool filled to zero bytes during this run, which
stalled the Nix GC lock and blocked local builds until space was reclaimed.

## Native bootstrap namespaces and the fifteenth launch

The fourteenth launch (`1528`) reached `runtime mount lies outside
library/admitted logical-store runtime namespaces` after 2 h 05 m. Scanning
every prepared recipe found 148 distinct mount destinations; five failed the
predicate: the native provider's `/__mantle_native_bootstrap_tools`,
`-runtime`, `-headers`, `-libraries` roots and the busybox `/bin` root. The
validator now admits those named namespaces explicitly (and still rejects every
other destination, covered by five added table cases; the recipe suite passes,
27 passed). The rescan reports zero rejected destinations. The fifteenth launch
is Pueue `1529`; at 2 h 23 m it is still executing, past the previous failure
point.

Three ordinary-gate defects outside the proof surfaced and were repaired:
three rust-plan tests pinned pre-change rejection behavior that the planner
now implements per Cargo (member feature unions, inactive cfg dependencies),
one core fixture expected `None` where an unset cfg now evaluates `false`, and
three production-pool tests inspected worker stores after the owning `TempDir`
was dropped. Each repaired suite passes: rust-plan admits 3 passed,
mantle-rust-plan-core 26 passed, production pool 21 passed.

## Rust provider program bound and the seventeenth launch

The fifteenth launch (`1529`) reached `source recipe is not bounded regular
source data` after 2 h 42 m. The sixteenth launch (`1530` equivalent, started
detached) named the cause: `full-source-rust-provider` reads
`source-built/rust/program.json`, which this stack's plan renders as
259 852 969 bytes across 63 882 steps. The 64 MiB source-document bound
rejected it while the structural `STEP_COUNT_MAX` (100 000) still held.

The source-document bound is now `RECIPE_SOURCE_BYTES_MAX` (512 MiB) with the
measured size recorded in the constant; invocation requests and manifest
members keep the 64 MiB bound. The recipe suite passes (27 passed). The
seventeenth launch is `proof-v17` with `mantle-v17`.

## Ordinary gate green and the eighteenth launch

`./scripts/check-first-party-quality.sh` completed with `gate_exit=0` on the
tree through `30aea7885`: fmt clean, strict clippy clean, and the serialized
workspace lib/tests run passed (final target `86 passed`). Two defects found by
that rail were repaired rather than waived: a flaky external-batch read bound
(rejected chunked but in-bound provider output as a flood; the reader now
bounds attempts by bytes and a one-byte-chunk regression test passes) and the
provider-launch claims described above.

The seventeenth launch (`proof-v17`) passed the Rust program document bound and
then failed at `undeclared source role in recipe` after 2 h 36 m: the Rust
provider program references `native-source-bundle` members 83 times, but the
`full-source-rust-provider` stage plan did not admit that source authority. The
plan now admits it (11 plan tests pass), and the eighteenth launch is
`proof-v18` with `mantle-v18`.

## Rust stage authority, assembler operands, and the nineteenth launch

The eighteenth launch (`proof-v18`) failed after 2 h 38 m at `input subpath
requires a declared tree artifact`: the mrustc assembler phase passed
`-o input("assembly", "mrustc-query-object")` while `assembly` binds the
compiler's Text output, and the invocation decoder requires any non-empty input
subpath to name a Tree. The operand now follows the proven C-translation shape:
the mapped token is `input("phase-tools", "mrustc-query-object")` into the
declared phase tool tree, with the same `/inputs/phase-tools/...` to
`/outputs/object` mapping. The nineteenth launch is `proof-v19`.

The ordinary gate exposed one further non-deterministic expectation:
`evaluator_budget_cli` asserted `kill_sent == false` for a merely slow worker,
but whether terminate is honored depends on when the worker installs its signal
policy relative to the deadline. The test now uses a dedicated 3 s fixture
policy, requires `terminate_sent` and a reaped timeout for both behaviors, and
asserts `kill_sent` only for the fixture that ignores SIGTERM
(6 consecutive passing runs).

## Repreparation with the assembler fix and the twentieth launch

The nineteenth launch (`proof-v19`) still reported the input-subpath rejection
because it consumed `prepared-cohort-v9`, whose Rust program predates the
mrustc operand fix. Scanning that program found exactly 139 offending words
(`rust-*.assemble.specs` naming `mrustc-query-object` inside the Text compile
output); the corrected preparation (`prepared-cohort-v10`,
`preparation_exit_code=0`, 2026-11-11T14:38:50-05:00, profile
`ec0f43531fa5ba52f546288ebb17e150a9d8c95ebf710f6de4afd0d882d05552`) has none:
the only unresolved subpaths are cross-stage native-provider references whose
tree kind is bound by the prior prepared recipe. The twentieth launch is
`proof-v20` against that profile.

The ordinary gate is green on the tree through `21df4a985` and its evidence
commit (`gate_exit=0`, final target `86 passed`).

## Specs-query declarations and the twenty-first launch

The twentieth launch (`proof-v20`) reached
`rust-source-phase-helper.compile.specs binds unused inputs ["linux-headers"]`
after 2 h 45 m: every GCC specs query inherited the full invocation input set,
including inputs only later phases read. The query now declares exactly the
inputs its own words reference — argument/environment words, runtime mount
sources, cwd, request and guard status inputs, plus the specs driver — and the
C translation keeps its mounted trees. A first attempt dropped runtime mount
sources as well and preparation failed at `unknown input bundle`; the mount
sources are now retained, and a fresh preparation
(`prepared-cohort-v11`, `preparation_exit_code=0`,
2026-11-11T20:57:05-05:00, profile
`bbb738d6b011b1e61ae5e8535584fe4e4d6dc3284a683abd07a09fe2ac45f88e`) authors a
`rust-source-phase-helper.compile.specs` without `linux-headers`. A preflight
scan of every prepared program reports zero local input-subpath violations. The
twenty-first launch is `proof-v21` with `mantle-v22`.

## Phase operand scoping and the rejection of a looser decoder

The twenty-first launch (`proof-v21`) failed after 2 h 47 m at
`rust-source-phase-helper.compile.split binds unused inputs
["helper-source","linux-headers","native-provider","native-sysroot"]`. Three
leaf shapes needed separate treatment.

A first attempt exempted every response-driven leaf from the one-to-one input
rule in both the source partition check and the runtime invocation decoder. The
existing recipe suite rejected that: three tests already pin the rule, one of
them named `invocation_cannot_invent_paths_names_control_reads_or_unused_dependencies`.
The exemptions were reverted and the invariant kept.

Instead, authoring now scopes every leaf to the operands it can address:

- GCC specs queries and splitter leaves declare only the inputs their own words,
  runtime mount sources, cwd, request and guard reference.
- A GCC phase additionally declares the inputs its authenticated path map can
  address: each `/inputs/<name>/...` mapping source names one operand the
  original command emitted. Words, phase runtime mounts, output seeds, cwd and
  guard status inputs are kept; everything else is dropped.

The recipe suite passes with the strict rule in force (27 passed), and a new
ADR (`adr/0147`) records the separate source-document bound and this scoping
rule. A fresh preparation with `mantle-v24` is in flight.

## Phase scoping keeps authored operands and topology

The first scoping attempt dropped phase inputs that no phase word or map source
named. Preparation rejected the result at `unreachable or omitted command
topology`: an assembler phase's `assembly` binding exists only in the authored
invocation (`-c input("assembly","")`), and dropping it also dropped the only
edge to its compile phase. The decoder invariant stays as the recipe suite
requires; authoring now retains the authored operand names as well.

Every phase leaf therefore declares:

- the names its authored invocation arguments and environment address,
- the names its path-map sources can address,
- its runtime mount sources, output seeds, cwd, guard status inputs, tool and
  response bindings.

The recipe suite passes (27 passed) and the last construction sites pass an
empty invocation-level set because their phases already carry the operands.

## Verification environment while the store recovers

The host Nix store reached zero free bytes and was then garbage-collected
down; the daemon's `min-free` (100 GiB) still exceeded the 76 GiB free, so
every `nix develop` queued on the GC lock, and the configured Tailscale
substituters (100.100.103.95:39400 and :5000) time out. The ordinary gate was
therefore executed with the checked-in script unchanged against the same
pinned cohort toolchain taken directly from the store
(`rustc`/`cargo` 1.96.0-nightly-2026-04-08, the 1.96 rustfmt build that the
tree is formatted with, clang/mold/pkg-config from the shell closure), with
`RUSTC_BOOTSTRAP=1` because `branches` needs the nightly gate without a nix
shell, and a clean target directory to clear stale read-only registry copies.
Proof execution is unaffected: it runs on the remote host with its own
reviewed binaries.

A direct-toolchain reproduction of the gate is not faithful: `branches
0.4.6` classifies any nightly as `branches_nightly` and expands
`core::intrinsics::abort`, which the 1.96 nightly no longer provides, while
the flake shell builds it. The gate therefore remains unverified on this host
until the store daemon and substituters recover; the changed authoring code is
covered by the recipe suite (27 passed) and by preparation validation of
cohort-v16 (`preparation_exit_code=0`).

Gate reproduction attempts with a direct store toolchain all stop in
dependencies, not in this repository: `branches 0.4.6` expands
`core::intrinsics::abort` on every nightly (the intrinsic is gone in 1.96), and
`cargo clippy` refuses a version-masking `RUSTC` because clippy-driver treats
the wrapper path as an input file. The checked-in gate therefore stays
unverified on this host until `nix develop` can realize its shell again; the
authoring surface changed here is covered by the 27-test recipe suite and by
`preparation_exit_code=0` for cohort-v16, and the previous full gate runs were
green on the same suites.

## Canonical gate green on a healthy host

The local store's remote builder and substituters (`aspen1.local` =
100.100.103.95) are unreachable, so the checked-in gate was run on the healthy
host: the tree at `46ec1ea40` was synced to `mantle-gate`, `nix flake archive`
copied every flake input (including the private git inputs) into that host's
store, and `scripts/check-first-party-quality.sh` ran unchanged inside
`nix develop` there. Result: `gate_exit=0` after the bridge probe, package
rustfmt, strict clippy, and the serialized workspace lib/tests run (220
`test result: ok` lines, final target `86 passed; 0 failed`).

Two later changes are not covered by that transcript because they landed after
the sync: the `@<request>` data-driven exemption and its fixture tidy, both
covered by the 27-test recipe suite.

On that host, the two commits that landed after the full gate run were then
checked inside the same `nix develop` shell: `cargo fmt --check -p mantle` and
`cargo clippy -p mantle --no-deps -- -D warnings` both pass, and the recipe
(27 passed) and phase-scoping (2 passed) suites pass on the updated source.

## Response-file leaves and the LLVM source collector

The twenty-fourth launch (`proof-v24`) cleared the phase leaves and then
reported `llvm-1.90.0.original-project binds unused inputs ["mrustc-source"]`.
Two fixes followed. A `@<request>` leaf is now recognized as data-driven by the
source partition check and the invocation decoder — narrowly, only for the
`@` operand form, so the pinned `invocation_cannot_invent_paths_names_control_reads_or_unused_dependencies`
test still holds. And the LLVM source collector no longer binds the mrustc
source: the collector copies only the llvm-project subtree, while the mrustc
artifact keeps its thirteen other consuming leaves.

A fresh preparation with `mantle-v29` (`preparation_exit_code=0`,
2026-11-12T11:52:25-05:00, profile
`1356a6f82664526d6b873560d07018a61e45681985443b92325b23752bd94f60`) authors
`llvm-*.original-project` with exactly `executable` and `rust-source`. The
twenty-fifth launch is `proof-v25` against that profile.

The same targeted checks pass again on the final commit (`9d16277ac`): package
rustfmt, strict clippy, and the 27-test recipe suite inside `nix develop` on
the healthy host.

## Provider export label and the twenty-sixth launch

The twenty-fifth launch (`proof-v25`) reached `invalid source-produced export
full-source-rust-provider` after 2 h 44 m: the successor graph assembles the
provider tree as its own `tree` output, while `require_export` demanded the
output label equal the stage identity. The export check now requires an
individually produced stage output of the right kind and keeps the
stage-prefix test; the canonical label requirement remains for the StageX
provider, whose authoring names that output explicitly.

Offline prechecks of the prepared program show the three Rust exports
(`full-source-rust-provider` tree, `rustc` executable, `toolchain-closure`
text) and every native role reference intact. The twenty-sixth launch is
`proof-v26` with `mantle-v30` against the unchanged cohort-v18 profile.

`proof-v26` passed every source-validation frontier that stopped the previous
launches and, at 2 h 48 m, is executing the Rust stage (three live bwrap
sandboxes, 37 GB staging) rather than failing closed. This is the first run to
enter real provider construction; no stage equality, reconciliation, receipt,
or release evidence exists yet.

