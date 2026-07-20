# Validation: Mantle artifact-auth operational receipt

Validated on 2026-07-19 from implementation commit `b0ff83ac`.

## Focused evidence

- `cargo test -p crunch-build operational_receipt` passed positive persist/reopen/replay plus wrong same-name key, unknown/revoked/stale trust, policy/window drift, malformed/oversized receipt, immutable replacement, symlink substitution, carrier drift, and unrelated-failure cases.
- `cargo fmt -p crunch-build -p mantle --check` passed. The repository-wide unscoped formatter still reports the preserved vendored `vendor/snix-castore/src/blobservice/combinator.rs`; the repo-owned first-party formatter deliberately excludes that unrelated vendor surface.
- `cargo clippy -p crunch-build -p mantle --all-targets --no-deps -- -D warnings` passed.
- `./scripts/check-first-party-tigerstyle.sh -p crunch-build -- --lib` passed for the modified library surface. The complete Nix Tiger Style consumer check also passed.
- Cairn validate plus proposal, design, and tasks gates passed.

## Full evidence

- `./scripts/check-first-party-quality.sh` passed package-scoped rustfmt, strict first-party Clippy, and serialized first-party workspace tests.
- `nix flake check -L --option builders ''` passed the complete x86_64-linux check matrix.
- Cairn sync dry-run passed with receipt `f48bd786802be9c1e8e1d65271339107042f66b477b3447fcabc3012b4c3ed92`.
- Cairn sync execution passed with receipt `ebf8b81fa7c27e1983734a43133df71f8a755d2cc8ae51213e9a68437592d2e7`; all four accepted requirement IDs are present in `cairn/specs/artifact-auth-operational-receipt/spec.md`.

## Bounded result and blocker

The exercised shell uses Mantle's full-key portable-receipt trust context, writes an immutable bounded local action-result receipt, reopens it, re-derives currentness from a fresh context, and independently replays the exact standalone carrier. Mantle still has no remote trust discovery or revocation-refresh authority, and this receipt does not perform registry publication, cache/build admission, or release selection. Standalone authority therefore remains unadmitted and rollback remains available.

The advisory VibeThinker audit was unavailable with `fetch failed`; deterministic source, test, Tiger Style, Cairn, first-party quality, and Nix evidence remains authoritative.
