# Evidence: repkgs source review

## Source

- Repository: https://github.com/mic92/repkgs
- Reviewed revision: `1cd7b8b` (2026-09-10), local clone at
  `/tmp/repkgs-review` during the 2026-09-10 session.

## Mechanism mapped by this change

- `pkgs/ji/jig/src/cc_mode.cc` and `cache_client.cc`: driver-mode compile
  cache with content-masked keys and depfile-learned dependency manifests;
  cached compile failures; links and configure probes cached over object and
  script identities.
- `pkgs/ji/jigd/src/store.go`, `slots.go`: bitcask-style pack store with
  in-memory index, whole-pack eviction, no fsync (cache, not state); build
  slot grants.
- `docs/design.md` "jig and jigd": the socket-in-sandbox,
  never-a-derivation-input property; measured sqlite3.c 82 s to 0.08 s and a
  stage-1 rebuild halved; explicit trust drawback with CA-output
  verifiability.

## Adaptation boundary

Mantle already owns the Rust side of this seam (`crunch-rust-cache`,
`crunch-rustc-wrapper`, daemon, strict lanes). This change adds the C/C++
driver mode inside Mantle's own inventory and daemon; no repkgs code is
copied. Machine-wide build slots are tracked separately by
`add-machine-wide-compile-slots`.
