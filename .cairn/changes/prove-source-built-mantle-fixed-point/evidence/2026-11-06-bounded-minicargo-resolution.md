# 2026-11-06: bounded minicargo resolution and canonical cc overlays

## Outcome and ownership

The current consumer is Rust 1.90 source-program authoring for the cold
source-built fixed-point route. Mantle's Rust source-program author owns these
repairs. The durable contribution is dependency-directed, finite-work version
unification plus one canonical cc overlay policy shared by the native and
successor authors. ADR 0139 records the resolver decision.

These are local edits over base `49bb15d908940a580a38b12d7add0fd52b3c6bdb`, not
a committed or frozen proof cohort. Exact changed-source and executed-binary
BLAKE3 values are retained in `2026-11-06-resolver-source-identities.txt`.
The temporary helper-export test was removed after runtime smoke verification.

## Checks actually exercised

- Pueue 39: `cargo test --release -p mantle --bin mantle
  minicargo::resolve::tests:: -- --test-threads=1`:
  `test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 2896 filtered out; finished in 0.03s`.
  The permanent fixtures cover late exact pins behind 24 independent binary
  choices within 128 steps, requiring-package/path rollback, filtered candidate
  recovery, contradictory pins, incompatible families, reuse, missing active
  dependencies, and work exhaustion. One fixture checks all 729 three-package
  constraint sets against exhaustive version assignments.
- Pueue 41: generated the complete successor helper from the real author,
  compiled it with rustc, applied `--cc-overlay` to `vendor-deps/cc`, compiled
  the transformed cc library, used it to compile/archive C, then linked and ran
  the C consumer. Observed: `cc overlay probe result=37`. This used host tools
  for a diagnostic smoke, not protected source-built provider execution.
- Negative smoke: applied `--cc-overlay` to a disposable copy of the same cc
  source with a duplicated original command boundary. Exit 1:
  `Error: Custom { kind: Other, error: "original source boundary must have one exact match" }`.
  The checked-in vendor source was not edited.
- Pueue 41: `cargo clippy --release -p mantle --bin mantle --no-deps -- -D warnings`
  and `cargo build --release -p mantle --bin mantle` completed successfully.
- Final changed-file rustfmt check passed for the resolver, its test module,
  successor cc author, and successor helper.
- `nix develop --offline --no-write-lock-file -c env
  CARGO_TARGET_DIR=/tmp/mantle-resolver-doc-checks cargo -Zscript
  scripts/check-operator-proof-guide.rs`: `operator proof guide drift check passed`.
- `nix develop --offline --no-write-lock-file -c cairn validate --root .`:
  `valid: true`, with no issues/findings. This is structural lifecycle evidence,
  not acceptance of implementation, execution, or release gates.

Full ordinary quality is **not green**. Pueue 35 passed the GCC configure bridge
but stopped at existing formatting differences in untouched source files,
including `source_built_fixed_point_stagex_actions.rs`,
`source_built_native_programs.rs`, and `source_built_recipe.rs`. Those files were
not reformatted. The ordinary workspace Clippy/test legs therefore did not run.
The retained quality log lists the exact differences.

An additional host-nightly `parallel`-feature compilation of the transformed cc
crate is blocked by upstream `src/parallel/job_token.rs:248` using deprecated
`fetch_update` under the crate's `deny(warnings)`. The non-parallel real compiler
smoke above passed. No lint suppression or vendor change was introduced.

## Real preparation and remaining blocker

Pueue 33 exercised resolution on the authenticated Rust archive and reached a
later source helper panic: `source-owned cc transformer declaration`. The
successor author was slicing a deleted function from native source. It now
consumes `cc_library_overlays` descriptors and uses the shared exact-replacement
operation. The process module imports only the dependencies of the random-state
and original Command output paths; the real transformed cc crate compiles
without suppressing its warnings.

Pueue 43 ran `/home/brittonr/mantle-cold-20261105/prep.sh`, from
`2026-11-06 11:33:50 -0500` to `11:42:37 -0500`, exit 1. Its exact command uses:

- source profile `/datapool/mantle-store/cold-inputs-20261030/profile/source-built-fixed-point-sources.json`;
- expected source-profile identity `29273e879992b713cec8414772afc4dc99d1454efef158cb86827083401d0820`;
- expected StageX lineage `e477ab39a0348812f9bd5a3af52759db3bd8dbc84f721315d1f78c766ca7d06d`;
- expected native provider `f36d3759145d09b45ce9d45fcb832eeca3677e2e75527ef0d9e1553100acf66f`;
- elapsed/disk/exec/source-record limits `86400`, `150000000000`, `524288`, `65536`;
- output `/home/brittonr/mantle-cold-20261105/prepared-cohort`.

Observed final diagnostic:

```json
{"error":"source program authoring: phase minicargo.rustc.87.rustc_data_structures.library.macro requires 5612 reserved slots, available 2624","code":1,"kind":"build"}
```

This is beyond dependency resolution and helper generation. The next boundary
is the expanded immutable macro-controller layout: translation reserves eight
recipe steps per macro slot plus six fixed steps. Its capacity comes from the
remaining recipe steps. `bootstrap/source-built-continuations.ncl` says that
source counts are sizing hints, exhaustion fails, and existing recipe/action
ceilings remain authoritative. No guessed lower reservation, new override, or
raised hard limit was used to bypass this failure. Retain the failed private
staging trees and task logs.

The next implementation unit is macro-continuation capacity, with actual command
and action-evidence semantics preserved; changing a source-owned reservation
instead requires a reviewed policy input, not an inferred execution count.
No current prepared profile, provider, stage1/stage2 equality, deterministic proof
receipt, bootstrap completion, release eligibility, or compiler correctness is
claimed. Cairn I3–I5 and V1–V3 remain unaccepted.

## Retained transcripts

- `2026-11-06-resolver-focused.log`: Pueue 39, exact tests and focused checks.
- `2026-11-06-cc-runtime-smoke.log`: Pueue 41, helper and real C consumer.
- `2026-11-06-resolver-quality.log`: Pueue 35, ordinary gate failure and focused checks.
- `2026-11-06-resolver-preparation-before-cc.log`: Pueue 33, resolver reached deleted transformer.
- `2026-11-06-resolver-preparation-after.log`: Pueue 43, exact next capacity failure.
- `2026-11-06-cairn-validate.log`: captured structural lifecycle validation.
