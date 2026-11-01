# Derivation closure carriage (2026-10-31)

Scope: carry the resolved derivation closure with a pool goal so a worker can
execute it from uploaded inputs without the Nickel closure and without inferring
input identities.

## Shape

- `crunch_build::distributed::PoolClosureEntry` (path, derivation-modulo hash,
  canonical derivation JSON); `PreparedPoolGoal.closure` is bounded by entry count
  (4096) and total bytes (64 MiB) with absolute unique paths and 64-hex HDMs.
- `Builder::prepare_build` collects the closure from the registry: the root plus
  every transitive input derivation the registry knows; missing entries are
  skipped so a receiver fails closed instead of guessing.
- The wire payload `NixDerivation` gains `closure: Vec<RemoteClosureEntry>`.
- The receiver validates the closure: bounded, unique, absolute paths, each
  derivation's recomputed path matches, the root entry matches the payload bytes,
  and every HDM recomputes from the closure map.
- `local_derivation_registry` now registers a `nix-derivation` closure into a
  `DerivationRegistry` and requires the root entry; the previous explicit
  rejection is gone.

## Verification

| Gate | Result |
| --- | --- |
| `cargo test -p mantle --bin mantle nix_derivation_payload` | 2 passed (planning, then missing-closure and HDM-drift denials, and local registration) |
| `cargo test -p mantle --bin mantle pool_` | 14 passed |
| `cargo test -p crunch-build --lib worker::tests::pool_dispatch` | 1 passed (selector also requires the root closure entry to match the goal) |
| `cargo test -p crunch-build --lib distributed::tests::prepared_goal` | 1 passed |
| `cargo clippy -p crunch-build -p mantle --bin mantle --no-deps -- -D warnings` | exit 0 |
| nightly `rustfmt` on changed files | exit 0 |

## Not claimed

- No live routed build has executed yet; the next proof is a loopback build
  through `build --builder-pool` with a real worker and caller descriptors.
- Closure collection is best-effort over registry knowledge; a receiver that
  needs an entry the Worker did not have fails closed.
