# Evidence: repkgs source review

## Source

- Repository: https://github.com/mic92/repkgs
- Reviewed revision: `1cd7b8b` (2026-09-10), local clone at
  `/tmp/repkgs-review` during the 2026-09-10 session.

## Mechanism mapped by this change

- `pkgs/ji/jigd/src/slots.go` and `cache_client.cc` `ReleaseSlot`: build
  slots served by the cache daemon; cc and rustc take one slot per run, Go
  through `-toolexec`, GHC through a semaphore served from slots.
- `docs/design.md` "jig and jigd": "384 sandboxes each running `make -j384`
  would oversubscribe the machine, so a compiler starts only when jigd
  grants a slot."

## Adaptation boundary

Mantle's authority is served by its own daemon (`mantle-rust-cache-daemon`
family), composed with the driver seam from
`extend-compile-cache-to-cc`. Scheduling receipts stay outside the
derivation graph; nothing is copied.
