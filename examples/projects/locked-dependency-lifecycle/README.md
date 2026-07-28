# Locked dependency lifecycle

This directory is a Mantle project-input manifest rather than a build package set. It tracks a local file input, mirror metadata, a local patch, generated inputs, and `current` retention.

Run from this directory:

```sh
mantle check
mantle show
mantle list-stale --no-network

# After intentionally changing sources/message.txt:
mantle list-stale --no-network
mantle refresh message-source
mantle check

mantle upgrade
```

`mantle refresh` updates `mantle.lock`, `.mantle/inputs.ncl`, patch identity, and the current-retention record together. `list-stale` is read-only. The repository test copies the project before mutating it, proves stale detection does not alter lock state, refreshes only the selected input, rejects a missing patch, rejects the empty Git revision in `fixtures/unresolved-revision.ncl`, and exercises the 0.9.0 → 1.0.0 upgrade path.

The example mirror uses `.invalid` and the offline workflow does not contact it.

The generated lock uses SHA-256 because project fetch inputs use the interoperable fetcher schema. Other Mantle-owned identities use BLAKE3.

A content hash records bytes. It does not prove upstream authenticity.
