# Owned transfer-object capability (2026-10-31)

Scope: an owned transfer-object authority so a long-lived pool executor can
render NARs, read blobs/directories, ingest inputs, and export outputs without
borrowing a `StoreHandle` for the executor's whole lifetime.

## Change

- `crunch_store::OwnedTransferObjectStore` holds the castore blob and directory
  services plus the store and output directory strings; it exposes the same
  bounded surface as the borrowed `TransferObjectStore`: `render_nar`,
  `open_blob`, `read_directory_canonical_bytes`, `ingest_host_path`,
  `ingest_nar`, `export_node_to_output`.
- `StoreHandle::owned_transfer_objects()` clones that authority.
- No raw blob/directory/pathinfo service or signing authority is exposed, so the
  existing capability boundary is preserved.

## Verification

| Gate | Result |
| --- | --- |
| `cargo test -p crunch-store --lib owned_transfer_objects` | 1 passed |
| strict Clippy `crunch-store`, `cargo fmt --check -p crunch-store` | exit 0 |
| `cairn validate --root . --strict` | 0 issues |

The fixture opens a real store, drops the handle, then ingests a host file,
renders its NAR, and exports it to the physical output directory through the
owned capability alone.

## Not claimed

- The capability grants transfer-object reads and export only; it is not
  authorization, signing, or output-trust evidence.
- No executor uses it yet; the bin-side executor and pipeline plumbing remain.
