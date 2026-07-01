# Design: Project input retention roots

## Architecture

Retention policy is decided in pure project logic and committed by a store/source-state shell.

- Pure core: retention schema validation, generation selection, root action planning, stale-root detection, GC-eligible classification, lock-generation binding, and diagnostics.
- Imperative shell: creating/removing roots, persisting project retention state, querying store/source state, and performing atomic filesystem updates.

Retention records should bind project identity, input name, lock generation or lock digest, source/content digest, root kind, and creation metadata. BLAKE3 should be used for new retention state fingerprints.

## Retention policy

A project may define a default input retention policy and per-input overrides. Policies should distinguish untracked inputs, tracked current inputs, and tracked recent generations with a named generation limit. Generation retention should be deterministic from lock history or a Mantle-owned generation ledger; it must not depend on filesystem mtime ordering.

## Atomicity and diagnostics

Lockfile updates, generated input updates, source-state imports, and retention root updates should either commit consistently or report a partial state that cannot be mistaken for durable retention. `mantle check` and `mantle show` should distinguish pinned, unpinned, stale-root, missing-root, and GC-eligible source records.

## Validation strategy

Pure positive tests should cover default policy, per-input override, generation cutoff, root action ordering, stale-root detection, and lock-digest binding. Pure negative tests should cover invalid generation limits, roots for unknown inputs, mismatched lock digests, interrupted root updates, and unpinned records being reported as GC-eligible.

Shell tests should use temp stores/source state to prove roots are written atomically and no undeclared inputs are retained.
