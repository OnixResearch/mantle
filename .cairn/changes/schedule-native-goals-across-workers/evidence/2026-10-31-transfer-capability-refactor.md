# Transfer capability refactor (2026-10-31)

Scope: let the remote input/output transfer preparation run on the owned
transfer-object capability, so a long-lived pool executor can prepare transfers
without borrowing the pipeline's `StoreHandle`.

## Change

- `src/remote_transfer.rs` transfer-artifact preparation takes
  `&crunch_store::OwnedTransferObjectStore` instead of the borrowed
  `TransferObjectStore<'_>`.
- `prepare_remote_production_input_transfer` and its input-NAR context take the
  owned capability; the host-path resolution reads the capability's output
  directory instead of the handle.
- Output-transfer NAR preparation and the receiver-side host-path callers use the
  handle's `output_dir_str()` / `owned_transfer_objects()`.
- `OwnedTransferObjectStore` exposes `store_dir()` and `output_dir_str()`.
- CLI call sites pass `store.owned_transfer_objects()`.

## Verification

| Gate | Result |
| --- | --- |
| `cargo test -p mantle --test remote_transfer_production` | 12 passed (real resumable production stdio transfers) |
| `cargo check -p mantle --all-targets` | exit 0 |
| strict Clippy (mantle targets), rustfmt on changed files | exit 0 |
| `cairn validate --root . --strict` | 0 issues |

## Not claimed

- The capability still grants transfer-object reads/export only; this refactor
  changes no policy, verification, or signing behavior.
- The pool executor that consumes it is not implemented yet.
