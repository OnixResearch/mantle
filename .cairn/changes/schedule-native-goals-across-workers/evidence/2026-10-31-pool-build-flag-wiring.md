# Pool build flag wiring (2026-10-31)

Scope: operator selection of a native worker pool for `build`, wired through
the pipeline factory seam.

## Change

- `build` gains `--builder-pool NAME`, `--builder-config PATH`, and repeatable
  `--builder-ticket-fd ENDPOINT=FD`; they conflict with the single-endpoint,
  watch, plan, impure, strict, evaluation-stream, compile-driver, wait-diagnostic,
  offline-preflight, and fix modes.
- `run_build_command` projects the pool (`plan_pool_factory`): loads the Nickel
  farm configuration, projects the selected pool, parses and ownership-checks the
  caller descriptors, resolves current bindings, admits the pool, reads one
  ticket credential per endpoint from its descriptor, and builds per-endpoint
  client options plus declared capabilities.
- The factory travels through `BuildCommandInput`/`PreparedBuildCommand` into
  `cmd_build` and is set on `BuildConfig.pool_executor_factory`, so the pipeline
  attaches it to the lazy builder.

## Verification

| Gate | Result |
| --- | --- |
| `cargo check -p mantle --bin mantle` | exit 0 |
| `cargo test -p mantle --bin mantle pool_` | 14 passed |
| `cargo clippy -p mantle --bin mantle --no-deps -- -D warnings` | exit 0 |
| nightly `rustfmt --check` on changed files | exit 0 |

Run from a reconstructed devshell environment (nightly 1.96 from a surviving
`nix-shell-env`, vendored Cargo config, perl/clang, `TMPDIR`), because the Nix
daemon auto-GC wedge still blocks `nix develop`.

## Not claimed

- No end-to-end routed build has been run; the ticket-bearing descriptors and a
  live worker are still required for that proof.
- Placement still uses no observed capacity; a goal without a quantified
  requirement matches by capability and stable identity only.
