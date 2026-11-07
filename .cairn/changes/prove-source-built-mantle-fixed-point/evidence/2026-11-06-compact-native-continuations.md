# 2026-11-06: compact native Rust script continuations

## Outcome, consumer and owner

Native Rust build-script reservations now use eleven fixed recipe declarations
and the shared source controller instead of expanding script/observe/select/
native/reply operations per reserved request. The immediate consumer is the
source-built Rust provider author before its LLVM/successor handoff. Mantle's
Rust source-authoring and source-controller owners maintain this mechanism;
[ADR 0144](../../../../adr/0144-keep-native-script-reservations-inside-shared-source-controllers.md)
records the contract. Source-owned hints/overrides and existing declaration,
command, event, time, disk and byte ceilings are unchanged.

This is local work over base `b94fdad41af227ed12b750c14ceccefcbce1c14a`, not a
frozen proof cohort. `2026-11-06-native-source-identities.txt` binds the final
source files and release CLI. Its executable BLAKE3 is
`b831321311fb68aa67d73919046606f8361ffff69cb5878a6f787b231a29ff9c`.

## Authority and state repairs

The shared C source-entry hook executes the original script, decodes its genuine
exit-75 standard-library request, executes the exact admitted native image,
acknowledges the actual normal status and descriptors, and replays the script.
Only actual script zero completes. Native nonzero is supplied to the script,
not converted into success or a guessed failed compilation. Whole `/tmp`,
original request bytes, descriptor aliases/flags/positions and shared timestamp
records remain in the common protocol. No coordinator spawns a child process.

The script is packaged into the Produced environment/cwd Tree; its separate
Produced executable identity remains the inventory authority. Environment-only
text fields are consumed into the frame rather than admitted as readonly Tree
roots. Final extraction consumes the appender's actual typed checkpoint result,
not a reconstructed producer identifier. All appender result consumers were
migrated. The obsolete expanded adapter, classifier/reply merger, guarded
appender facade, native-seed/native-result modes and unused LLVM helper export
were removed, without a compatibility path.

Readonly history inputs alone were insufficient: the script could mutate the
workspace duplicate. Pueue 46's protected regression genuinely changed an
acknowledged status from 23 to 0 and the old driver admitted completion. The
retained `native-history-before.log` is explicitly an excerpt from the delivered
queue output because the daemon later removed that task's full log. The fixed
driver retains exact names, kinds, modes and bytes outside the workspace and
compares the restored history before accepting another request or final zero.
The same adversarial scenario now rejects completion.

Shared evaluator fatal errors now exit 2 rather than collide with the genuine
controller-completion status 1. A script's actual exit 1 remains a failure. The
malformed-cwd smoke checks the captured status artifact: denied start statuses
are not emitted as accepted normal command observations. Compiling the composed
C driver also exposed calls to a nonexistent GNU/Perl Args destructor; those
owned argument vectors now use the existing shared destructor. No missing
symbol warning or fatal error was suppressed.

## Exercised verification

Pueue 28 (`compact-native-final-checks`, 18:50:57–18:59:06 -0500) exercised these
results, retained in `2026-11-06-native-focused.log` and the separate emitted
macro protocol log:

| Check | Passed | Failed | Scope |
|---|---:|---:|---|
| Native reservation boundaries | 1 | 0 | Runtime ceiling, declaration capacity, overflow and empty per-request denial |
| Protected native controller smoke | 1 | 0 | Eight scenarios: early zero, repeated suspension with native 23/0, request exhaustion, image drift, native-command exhaustion, history corruption, script failure, malformed cwd |
| Protected macro regression | 1 | 0 | Early completion, binary replay, exhaustion, image drift and direct/controller decoder namespaces |
| Emitted native utility tests | 2 | 0 | Unique environment/cwd admission, successful state projection, private-state removal and unfinished/nonzero denial |
| Emitted macro protocol tests | 7 | 0 | Exact images, observations, history, timestamps, command transitions and binary state |

That task then failed strict Clippy on an unused checkpoint field; it is not an
overall successful gate. The repair returns the existing typed controller
products and consumes the real checkpoint in completion. Pueue 30
(`compact-native-typed-products-final`, 19:01:03–19:14:59 -0500), exit 0, compiled
all migrated consumers, reran the reservation regression, passed focused release
Clippy (`-p mantle --bin mantle --no-deps -- -D warnings`) and built the release
CLI. `2026-11-06-native-final-checks.log` retains that full transcript. The C
controller/protocol and emitted utility sources did not change after the passing
protected/helper scenarios.

The actual compiled completion utility was also invoked directly over the real
protected nonzero-replay checkpoint. It returned the exact binary final script
stdout, preserved whole-tmp products with matching BLAKE3 identities, and removed
only private request/random members. Its command, bytes and observed products are
in `2026-11-06-native-completion-smoke.json`; this is not a mock or a source-text
check.

All changed Rust leaves passed leaf-only rustfmt checking, including the final
result-contract migrations. The operator proof-guide checker passed; its output
is retained in `2026-11-06-native-doc-check.log`. The ordinary workspace quality
rail remains unaccepted because of its previously captured unrelated formatting
failure; it was not rerun merely to reproduce that known result.

No provider execution, stage1/stage2 equality, full-source fixed point,
deterministic proof receipt, full parity or release eligibility is accepted.
Cairn I3–I5 and V1–V3 remain unchecked. The protected producers are bounded C/Rust
protocol fixtures, not original Rust compilers or provider construction.

## Real source preparation

Pueue 31 (`compact-native-real-preparation`, 19:14:59–19:26:41 -0500), exit 1,
ran the unchanged `/home/brittonr/mantle-cold-20261105/prep.sh` after the successful
release build. The release executable and launcher identities are recorded at
the start of `2026-11-06-native-real-preparation.log`. Source profile, StageX
lineage, expected native-provider pins and resource bounds were unchanged.
Preparation passed the prior `source recipe capacity exhausted before original
LLVM graph` boundary and reached a later successor-resolution diagnostic:

```json
{"error":"Rust successor source topology: ambiguous original locked identity home","code":1,"kind":"build"}
```

No prepared profile is accepted. The next repair must preserve consuming-lock
authority and resolve the actual `home` identity; choosing a version by order,
falling back to a dependency lock or weakening ambiguity denial is not justified.

## Lifecycle check

`nix develop --offline --no-write-lock-file -c cairn validate --root .` returned
exit 0 with `valid: true` and no issues/findings. The captured structural report
is `2026-11-06-native-cairn-validate.json`. This is lifecycle structure validation,
not an implementation, archive, provider, proof or release acceptance gate.
No change was archived and no unchecked proof task was promoted.
Temporary validation/preparation launch scripts were removed after their exact
commands and output were retained; protected fixture products remain evidence.

