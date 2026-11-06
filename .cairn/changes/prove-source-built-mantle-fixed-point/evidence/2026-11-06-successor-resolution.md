# 2026-11-06: consuming-workspace successor resolution

## Outcome and owner

Mantle's Rust source-authoring owners maintain this repair for Cargo-free Rust
successor graph construction. The durable capability is one borrowed consuming
lock/patch context per topology, shared across transitive registry dependencies
without copying workspace identities into every package key. ADR 0141 records
that compiler, library, Cargo and rustdoc groups remain separate resolutions.

The resolver no longer reads a dependency's own Cargo.lock for transitive
registry selection. Missing consuming locks cannot fall back to a dependency or
parent workspace lock. Consuming-root local registry patches apply transitively;
nonlocal patches and mixed consuming workspaces remain errors. Manifest
workspace inheritance stays separate. No source/version fallback was added.

## Source diagnosis

The authenticated Rust 1.90 archive used by the retained preparation contains
memchr 2.5.0, 2.7.2, 2.7.4 and 2.7.5, not 2.7.1. Its consuming Cargo.lock and
library/Cargo.lock pin memchr 2.7.5. Vendored bstr 1.12.0 carries its own upstream
lockfile pinning memchr 2.7.1. The old nearest-lock search selected that stale
identity; the diagnostic's `not unique` did not mean duplicate vendor sources.
Original manifest/lock facts are retained in
`2026-11-06-successor-lock-source-facts.json`.

## Checks exercised

Pueue 28 ran the before-fix regression suite, exit 101:
`test result: FAILED. 21 passed; 6 failed; 0 ignored; 0 measured; 2889 filtered out; finished in 0.03s`.
All six new consumer-visible cases failed: consuming-lock precedence, missing
consuming-lock denial, transitive consuming patches, nonlocal patch denial,
nested workspace isolation and mixed-root denial. The first case selected
memchr 2.7.1 instead of the consuming lock's 2.7.5.

Pueue 30 ran the repaired suite, focused release Clippy and release CLI build,
exit 0, from 14:17:41 to 14:28:42 -0500:
`test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 2889 filtered out; finished in 0.03s`.
The substring filter also exercised existing minicargo and project-resolution
cases. Clippy used `-p mantle --bin mantle --no-deps -- -D warnings`.

Changed resolver files passed leaf-only rustfmt checking. The operator-guide
checker passed. The ordinary workspace quality rail remains unaccepted because
of the previously captured unrelated formatting failure; it was not rerun merely
to reproduce that known result.

The lock repair's base, source/binary and inspected archive BLAKE3 identities are
in `2026-11-06-successor-scope-identities.txt`. The base is
`49bb15d908940a580a38b12d7add0fd52b3c6bdb`; this is not a frozen proof cohort.
Those identities precede the separately observed atomic-predicate boundary.

## Real source preparation

Pueue 31 ran the unchanged `/home/brittonr/mantle-cold-20261105/prep.sh` after
Pueue 30, from 14:28:42 to 14:35:07 -0500, exit 1. Source/lineage/native-provider
pins and resource bounds were unchanged from the retained resolver/macro runs.
Real preparation passed the previous memchr boundary and stopped at:

```json
{"error":"Rust successor source topology: unclassified original target cfg target_has_atomic = \"64\"","code":1,"kind":"build"}
```

The original target source declares `max_atomic_width = Some(64)`, and the
pinned compiler's fixed-target query reports 8/16/32/64/ptr atomic widths.
This is a separate bounded target-classification repair, not permission to turn
unclassified predicates into false or substitute another dependency version.

No prepared profile, provider execution, stage1/stage2 equality, full-source
fixed point, deterministic proof receipt or release eligibility is accepted.
Cairn I3–I5 and V1–V3 remain unchecked.

## Retained transcripts

- `2026-11-06-successor-scope-before.log`: Pueue 28, six regression failures.
- `2026-11-06-successor-scope-after.log`: Pueue 30, tests, Clippy and build.
- `2026-11-06-successor-scope-preparation.log`: Pueue 31, original-source boundary.
- `2026-11-06-successor-scope-doc-check.log`: operator-guide check.
- `2026-11-06-successor-atomic-target-source.json`: original Rust target declaration.

## Atomic classification checkpoint

Pueue 36 (14:37:55–14:48:40 -0500), exit 0, exercised the exact source-declared
8/16/32/64/ptr atomic widths, exclusion of 128-bit dependency edges, and retained
unknown-predicate denial. It also reran the consuming-workspace regressions:
`test result: ok. 29 passed; 0 failed; 0 ignored; 0 measured; 2889 filtered out; finished in 0.03s`.
Focused release Clippy and CLI build passed. Exact transcripts and source/binary
identities are retained in `2026-11-06-successor-atomic-focused.log` and
`2026-11-06-successor-atomic-identities.txt`.

Pueue 38 (14:48:40–14:58:03 -0500), exit 1, then passed the atomic boundary and
reported `unclassified original target predicate miri`. Inspection of original
syn manifests found `cfg(not(miri))` tables containing only dev-dependencies.
The resolver was classifying predicates before checking whether their dependency
kind was selected. Skipping unselected kinds, not assigning miri a guessed false
value, is the appropriate repair. `2026-11-06-successor-atomic-preparation.log`
retains that run. The derived original-clause inventory is diagnostic data, not
an admission policy for unknown flags.

## Selected-kind and native-profile checkpoints

Pueue 53 (15:02:54–15:16:02 -0500), exit 0, verified that dev-only target tables
are not interpreted, while an unclassified predicate on a selected dependency
kind still fails:
`test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 2889 filtered out; finished in 0.06s`.
Focused Clippy/build passed. Pueue 54 (15:16:02–15:23:36 -0500), exit 1, passed
that boundary and reached `unclassified original target predicate rustix_use_libc`.
Its transcript and source/binary identities are retained in the `successor-kind`
log and identity files.

The source-owned native compiler profiles do not enable rustix_use_libc, miri or
rustix_use_experimental_asm. Explicitly classifying those known unset modes lets
the original x86_64/Linux raw backend predicate select its own dependencies;
no libc fallback or arbitrary unknown-predicate default was added. Target tables
also require an active optional edge before their predicate needs classification.
Enabling an optional feature still requires admission of its predicate.

Pueue 55 (15:27:25–15:37:57 -0500), exit 0, verified the source-profile backend
and inactive/active optional-feature transition:
`test result: ok. 33 passed; 0 failed; 0 ignored; 0 measured; 2889 filtered out; finished in 0.04s`.
Focused Clippy/build passed. Pueue 56 (15:37:57–15:46:11 -0500), exit 1, then
passed those boundaries and reached
`unclassified original target cfg getrandom_backend = "custom"`.
The `successor-native` log and identity files retain this checkpoint. This is an
unset global override comparison in the sealed native profile; per-crate
build-script cfg output does not select Cargo target dependencies.

## Global cfg namespace checkpoints

Pueue 58 (15:48:01–15:57:01 -0500), exit 0, verified that the unset global
getrandom override selects the original Linux-default dependency clause:
`test result: ok. 34 passed; 0 failed; 0 ignored; 0 measured; 2889 filtered out; finished in 0.09s`.
Clippy/build passed. Pueue 59 (15:57:01–16:04:57 -0500), exit 1, then reached
`unclassified original target predicate tracing_unstable`.

ADR 0142 records explicit classification of the reviewed native opt-in modes
as unset; no catch-all false rule was introduced. Cargo package features do not
enable those global modes. Pueue 63 (16:07:07–16:17:35 -0500), exit 0, exercised
this separation:
`test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 2889 filtered out; finished in 0.06s`.
Clippy/build passed. These are the `successor-mode` logs/identities, not the
earlier macro task with a reused daemon-local task number.

Pueue 64 (16:17:35–16:25:50 -0500), exit 1, passed the mode boundary and reached
`unclassified original target cfg feature = "alloc"`. The original Cargo lock
for faster-hex 0.10.0 includes heapless from its not(feature = "alloc") target
clause. A real offline Cargo metadata probe with default feature alloc enabled
selected the not(feature = "alloc") dependency and excluded the positive branch.
The target context does not contain package-local feature cfgs. The metadata
JSON and invocation stderr are retained in the `successor-cargo-feature-target-probe`
files. This was a host-Cargo semantic diagnostic, not protected bootstrap execution.

## Target-feature namespace checkpoint

Pueue 65 (16:27:32–16:36:16 -0500), exit 0, verified that an enabled package
feature alloc does not populate the global target cfg context. Its positive
target-feature branch remains absent and the original negative branch selects
its declared dependency:
`test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 2889 filtered out; finished in 0.03s`.
Focused Clippy/build passed. The exact execution identities and transcript are
retained in `2026-11-06-successor-feature-identities.txt` and
`2026-11-06-successor-feature-focused.log`.

Pueue 66 (16:36:16–16:42:53 -0500), exit 1, passed the prior cfg boundaries and
reported `source recipe: invalid recipe label`. The successor generator embeds
raw package versions and combined crate types in IDs: build metadata contains
`+`, and `rlib,dylib` contains a comma, both outside the recipe label grammar.
ADR 0143 records the label/data separation repair; it must not alter the original
Cargo values or compiler crate-type arguments or relax label admission.
`2026-11-06-successor-feature-preparation.log` retains the original diagnostic.

## Final source-label checkpoint

ADR 0143 records a lossless producer-label encoding for package names/versions
and combined crate types. Original Cargo data and compiler arguments are not
rewritten. Safe components remain borrowed; encoded punctuation cannot alias
literal escape text. Existing recipe grammar and size ceilings remain intact.

Pueue 86 (17:10:56–17:22:12 -0500), exit 0, ran final label admission and exact
successor resolver regressions, focused release Clippy and the CLI build:

- `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 2925 filtered out; finished in 0.00s`
- `test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 2912 filtered out; finished in 0.00s`

The label fixtures reject raw metadata/combined-kind labels, admit encoded
producer references, and retain distinct identities for metadata/prerelease and
comma/literal-escape inputs. `2026-11-06-successor-label-focused.log` and
`2026-11-06-successor-label-identities.txt` bind the final run and source/binary.
Changed-file rustfmt and the final operator-guide check passed.

Pueue 88 (17:22:12–17:31:11 -0500), exit 1, ran the original preparation runner
with unchanged profile pins and bounds. It passed the preceding resolution,
cfg and label boundaries, then reported:

```json
{"error":"Rust source provider authoring: source recipe capacity exhausted before original LLVM graph","code":1,"kind":"build"}
```

`2026-11-06-successor-label-preparation.log` retains the exact task and diagnostic.
No ceiling was raised, reservation lowered, source pin replaced, or incomplete
profile published as success. The remaining capacity boundary requires its own
source-graph investigation. No current prepared profile, provider execution,
stage1/stage2 equality, full-source fixed point or release eligibility is claimed.
The previously recorded ordinary-quality formatting failure remains unaccepted.

Final `nix develop --offline --no-write-lock-file -c cairn validate --root .`
returned exit 0, valid true, with empty issues/findings/change_issues. Receipt
hash: `3cb5950d98384ebf6a5a673ada833889331e32a690fc80d30fdd841d679bc795`.
Transcript: `2026-11-06-successor-cairn-validate.log`. This validates structural
lifecycle conformance, not implementation acceptance, proof or release gates.
