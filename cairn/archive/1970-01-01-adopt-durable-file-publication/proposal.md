# Adopt durable file publication in Mantle

## Why

Mantle still owns a local no-replace publication shell for immutable remote-attempt segments and anchors. The reviewed `durable-file-publication` repository now provides the shared bounded mechanism. Mantle must consume that exact mechanism without moving product policy, manifests, retention, or authority into the dependency.

## What changes

- Pin Radicle RID `rad:z3tAR4For7qw8ZirkJzoDw1VNDDLM` and revision `951c27f59003cea9bfdb40ed4d89653d50fada1f` in Cargo and Nix.
- Route immutable remote-attempt segment and anchor files through the shared capability-relative shell.
- Keep manifest replacement, content comparison, retention, deletion, receipts, and diagnostics in Mantle.
- Preserve the current local publisher as an explicit rollback backend.
- Replay the shared 17-case corpus and add positive and negative Linux integration tests.
- Emit typed adoption evidence with explicit non-claims.

## Impact

The change affects only immutable remote-attempt object publication and its source-verification rails. It does not change release-bundle directory publication, manifest replacement, retention policy, deployment authority, or remote-build admission.
