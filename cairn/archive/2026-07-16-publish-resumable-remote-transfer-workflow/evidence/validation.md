# Implementation validation

Recorded: 2026-07-16

## Result

The supported `remote-build-loopback` project now has a deterministic `.#resumable-payload` selector whose repeated-content NAR crosses multiple production chunks. The production local stdio path interrupts after a durable acknowledgement, resumes the same fenced manifest from fresh client/server processes, reports reused bytes, writes one client output, and passes ordinary output admission. Changing the acknowledged receiver chunk blocks retry with `acknowledged-chunk-missing` and leaves the client store empty.

The gallery fixture also exposed and repaired a pure-core ambiguity: content digest was incorrectly used as artifact occurrence identity. Demand lookup now selects canonical artifact id plus artifact-local index, validates the complete descriptor, schedules each absent BLAKE3 digest once, and counts later equal-content occurrences as reuse. ADR 0027 records this boundary.

## Discovery and adversarial audit

Pueue task `45` first ran the gallery positive/negative fixtures against the existing production implementation. The tamper fixture passed, but the repeated zero-content positive fixture failed on retry:

```text
test gallery_resumable_remote_transfer_rejects_tampered_acknowledged_content ... ok
test gallery_resumable_remote_transfer_resumes_verified_chunks_and_admits_once ... FAILED
remote production stdio serve once: chunk-digest-mismatch
```

Inspection found `demanded_chunk()` selected by `(artifact_id, digest)`. Equal-content chunks at different offsets therefore selected the first occurrence. Avoiding repeated bytes in the example was rejected as false completion.

The accepted repair separates mechanisms:

| Family | Decision | Evidence |
|---|---|---|
| New protocol / second CAS | Rejected | Current production stdio already streams through the existing core, shell, and admission path. |
| Digest-only occurrence identity | Falsified | Task `45` repeated-content failure. |
| Docs-only reconciliation | Insufficient alone | Cannot prove data-plane execution or admission. |
| Artifact-local occurrence plus full descriptor | Accepted | Pure core and production repeated-content fixtures pass. |
| Cursor trust after tamper | Falsified | The production tamper fixture fails before admission. |

The audit also verifies that the deterministic interruption environment variables remain behind `cfg!(debug_assertions)`, the catalog calls the rail debug-test-only, and docs do not promote local stdio evidence into P2P/SSH/exactly-once/release claims.

## Focused core and production evidence

Pueue task `64` established the pre-repair pure-core baseline:

```text
running 11 tests
test result: ok. 11 passed; 0 failed
```

Pueue task `69` ran focused Rustfmt, the repaired pure-core suite, and both gallery production fixtures. The final production result was:

```text
running 2 tests
test gallery_resumable_remote_transfer_rejects_tampered_acknowledged_content ... ok
test gallery_resumable_remote_transfer_resumes_verified_chunks_and_admits_once ... ok
test result: ok. 2 passed; 0 failed
```

Pueue task `94` reran the core after demand-level duplicate suppression, reran both production gallery fixtures, and passed the exact catalog/docs boundary test:

```text
test remote_resumable_workflow_catalog_and_docs_name_the_production_boundary ... ok
test result: ok. 1 passed; 0 failed
```

Pueue task `108` then ran this complete serialized focused chain successfully:

```text
cargo test -q -p crunch-build 'distributed::remote_transfer::tests::' -- --test-threads=1
cargo test -q -p mantle --test remote_transfer_production 'gallery_resumable_remote_transfer_' -- --test-threads=1
cargo test -q -p mantle --test remote_transfer_production 'production_stdio_' -- --test-threads=1
cargo test -q -p mantle --test examples_inventory -- --test-threads=1
cargo test -q -p mantle --test examples_workflow_gallery -- --test-threads=1
cargo test -q -p mantle --test remote_stdio_cli -- --test-threads=1
```

The command exited successfully. The production regression includes input resume, output resume, quota rejection before checkpoint/admission, delta-unavailable bounded full fallback, 8 MiB streaming, and bounded observability cases. The gallery checks cover project evaluation, catalog/path/doc synchronization, remote ticket positive/negative behavior, and the new required production rail.

## Broad first-party evidence

Pueue task `99` ran the canonical gate:

```text
nix develop -c ./scripts/check-first-party-quality.sh
```

It completed successfully after formatting, strict first-party Clippy, and the serialized first-party library/integration test rail.

Pueue task `107` ran:

```text
./scripts/check-first-party-tigerstyle.sh
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- tracey coverage --root .
git diff --check
```

Tiger Style completed the full configured first-party scope. Tracey reported:

```text
traceability coverage ok: 145/145 referenced (profile mantle-default)
```

The new active requirement is not yet part of the accepted-spec count; its implementation/verification bridge will be added only after sync so pre-sync Tracey has no dangling reference.

## Lifecycle evidence before sync

Pueue tasks `37` and `98` ran current Cairn validation plus proposal, design, and tasks gates. Both packets reported `valid: true`; proposal/design/tasks verdicts were `PASS`. Task `98` saw 10 substantive tasks, 5 done, and 5 remaining before the final validation markers were updated.

## Accepted requirement and pre-archive packet

Pueue task `117` ran Cairn sync dry-run and execute for `publish-resumable-remote-transfer-workflow`. The execute receipt had no reasons and receipt hash `f550ff1b2200842792113c7783b1cb3991494af9f638dba3458b951d33bab9cb`. Inspection confirmed `r[examples.resumable_remote_transfer_workflow]` plus all three reviewed scenarios were appended to `cairn/specs/examples/spec.md`.

After sync, `tools/tracey_refs.rs` linked implementation to the occurrence-safe core/production gallery and verification to the pure-core, production positive/negative, and inventory drift tests. Pueue task `119` passed focused Rustfmt, `git diff --check`, Cairn validation, and all three gates. Pueue task `120` reported:

```text
traceability coverage ok: 145/145 referenced (profile mantle-default)
```

The profile count remains 145 because the default profile's accepted-spec selection is unchanged; no missing or dangling reference is inferred from the stable total.

## Archive and exact post-archive validation

Pueue task `127` ran archive dry-run and execute with `CAIRN_ARCHIVE_DATE=2026-07-16`. It created `cairn/archive/2026-07-16-publish-resumable-remote-transfer-workflow/` with no blockers and receipt hash `d96f1506ef560efa8f2d9fe30a637496ec6ad8142ee0c50a51e0fc1faf8d0845`.

Pueue task `128` then ran these exact commands against the archived tree:

```text
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- change list --root .
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- tracey coverage --root .
```

The complete unedited validation JSON is committed as `post-archive-validation.json`. It reports zero changes, empty issue/finding/substance/cross-repository lists, 28 specs validated, and `valid: true`. The exact change-list output is committed as `post-archive-change-list.json` and contains an empty `changes` array. Tracey printed:

```text
traceability coverage ok: 145/145 referenced (profile mantle-default)
```

## Budget and terminal state

The declared search budget used four mechanism families, two baseline rounds, one repeated-content counterexample, one receiver-tamper counterexample, focused repository tests, one full quality gate, one full Tiger Style pass, and pre/post-archive lifecycle/Tracey checks. No external network authority was needed. The terminal reason is validated completion with an archived change and no active Cairn packages.

## Non-claims

This evidence does not prove exactly-once network delivery, arbitrary process-kill recovery, a production P2P listener, SSH deployment, independent-machine behavior, hostile-worker honesty, compiler correctness, output trust from transfer alone, release reproducibility, or Kani execution. The debug interruption seam is deterministic validation behavior only and is ignored by release binaries.
