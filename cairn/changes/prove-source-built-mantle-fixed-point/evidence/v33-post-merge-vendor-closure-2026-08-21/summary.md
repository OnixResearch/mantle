# V33 post-merge vendor-closure refresh

## Goal

Integrate current `origin/main` without carrying a new uncaptured Cargo source
into the next diagnostic or promoted proof.

## Starting evidence

The merged reconciliation pilot added this sourced dev dependency:

```text
transactional-reconciliation-core
revision 606489b5f40298181214bb76bc3457b607f225d9
```

It did not add a Cargo vendor replacement or checked vendor directory. Pueue
task `6545` used an empty `CARGO_HOME`, `--offline`, and `--locked`. Source
resolution failed on that exact missing Git checkout.

## Repair

Pueue task `6551` ran locked Cargo vendoring and fetched only the declared
source closure. Cargo produced
`vendor-deps/transactional-reconciliation-core/` with per-file checksum
metadata. The tracked Cargo vendor config now maps the exact Radicle URL and
revision to `vendor-deps/`.

Pueue task `6554` proved that the complete merged source resolves with an empty
`CARGO_HOME`, `--offline`, and `--locked`. Cargo's complete generated source
config exactly matches the tracked config.

Pueue task `6560` recorded `local-validation.log`. The source/profile vendor
pair, materialized proof preflight, native feature grammar, and deterministic
manifest-path tests passed. Empty-`CARGO_HOME` resolution, focused strict
Clippy, and `git diff --check` also passed.

## Non-claims

This is source preparation, not proof execution. It does not admit a different
revision, ambient cache, provider output, or network access inside the proof.
