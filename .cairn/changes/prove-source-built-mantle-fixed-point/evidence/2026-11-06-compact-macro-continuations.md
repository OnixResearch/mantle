# 2026-11-06: compact macro continuations and deferred namespace handoff

## Outcome and ownership

Rust 1.90 source-program authoring now admits the unchanged 5,612-slot macro
reservation through five fixed recipe declarations instead of expanding a
recipe per reservation. The existing controller command ceiling bounds
`2 * slots + 1` actual compiler/plugin command turns. Source-owned hints,
executable authority, event reconciliation and resource limits are unchanged.
Mantle's Rust source-authoring/controller owners maintain this mechanism;
ADR 0140 records its design and current consumer.

This is local source work over base
`49bb15d908940a580a38b12d7add0fd52b3c6bdb`, not a frozen proof cohort.
`2026-11-06-macro-source-identities.txt` binds the relevant source files and the
release binary exercised by real preparation.

## Behavioral repairs

The controller retains original compiler argv[0], typed operands, invocation
metadata, cwd, environment, binary plugin protocol, generated tmp and executor
postorder timestamp frames. Only a genuine observed compiler exit zero can
complete it. Plugin replies require exactly one successful admitted plugin leaf.
Malformed/forged observations, skipped or double execution, drifted images,
changed history and reservation exhaustion fail closed.

Protected exhaustion exposed Rust main's default error exit 1 colliding with
the controller done predicate. Controller CLI errors now exit 2 explicitly;
start/resume reserve 1 for genuine completion and prepare uses 1 for unselected.

A real deferred-decoder reproducer then rejected controller-produced commands:
its original namespace was hardcoded to `/outputs/translation`, while the compact
compiler emits `/outputs/workspace/translation`. The author now binds the exact
namespace as comparison data. The decoder checks that closed declaration before
projecting response operands and captured environment paths. It never guesses a
namespace from generated text. Both direct and controller producers use the
same explicit contract; the obsolete hardcoded relocation helper was removed.

## Checks actually exercised

Pueue 63 (`macro-checks.sh`, 13:05:00–13:12:19 -0500), exit 0:

- Authoring tests: `test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 2904 filtered out; finished in 0.00s`.
- Emitted helper protocol tests: `test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s`.
- Protected smoke: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2909 filtered out; finished in 13.43s`.
  It exercised early compiler completion, repeated binary plugin replay,
  exhausted reservations and wrong-image rejection through real protected
  commands and `ActionObservationState`. Exhaustion and image drift produced
  no authenticated done outcome or translation publication.
- The smoke compiled the actual C decoder and exercised direct/controller
  namespaces plus mismatched and unknown declarations. Valid cases checked exact
  projected response and environment bytes and retained original command bytes.
- Focused release Clippy (`-p mantle --bin mantle --no-deps -- -D warnings`)
  and the release CLI build completed successfully.

Changed macro Rust files passed leaf-only rustfmt checking. The operator guide
checker passed after its final edit. Cairn validation returned exit 0,
`valid: true`, with no issues/findings. Structural validation is not lifecycle
acceptance. The ordinary quality rail remains unaccepted: its earlier run
stopped at unrelated formatting differences, as recorded in the resolver
handoff; it was not rerun just to reproduce that known failure.

## Real preparation boundary

Pueue 64 ran the unchanged authenticated `/home/brittonr/mantle-cold-20261105/prep.sh`
after Pueue 63, 13:12:19–13:18:06 -0500, exit 1. It retained the same source
profile, source/lineage/native-provider pins and resource bounds documented in
`2026-11-06-bounded-minicargo-resolution.md`. The macro-capacity rejection is no
longer the observed boundary. The new diagnostic is:

```json
{"error":"Rust successor source topology: original vendor identity memchr 2.7.1 is not unique","code":1,"kind":"build"}
```

No prepared profile, executed provider, stage1/stage2 equality, full-source
fixed point, deterministic proof receipt or release eligibility is claimed.
Cairn I3–I5 and V1–V3 remain unaccepted.

## Retained evidence

- `2026-11-06-macro-focused.log`: exact Pueue 63 command, tests, Clippy and build.
- `2026-11-06-macro-protocol.log`: emitted-helper protocol test transcript.
- `2026-11-06-macro-real-preparation.log`: exact Pueue 64 command and boundary.
- `2026-11-06-macro-source-identities.txt`: base commit and source/binary BLAKE3.
- `2026-11-06-macro-doc-check.log`: final operator-guide check.
- `2026-11-06-macro-cairn-validate.log`: structural lifecycle validation.
- `/home/brittonr/mantle-macro-20261106/protected-v8/`: actual protected events,
  helper fixtures, decoder cases and failure records. These are bounded protocol
  fixtures, not an original Rust compiler or provider/bootstrap proof.
