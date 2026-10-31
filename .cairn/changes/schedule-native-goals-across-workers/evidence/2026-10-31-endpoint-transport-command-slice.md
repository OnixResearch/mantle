# Per-endpoint transport command slice (2026-10-31)

Scope: the per-endpoint transport command a pool needs before any dispatch. A
single `--builder-program`/`--builder-args` cannot describe several workers, so
each endpoint now declares its own argv. This is configuration and admission
only; nothing here opens a transport.

## Contract

- `lib/remote-builders.ncl`: `RemoteEndpoint` gains `transport_program` and
  `transport_args`, admitted through the endpoint shape's extra-field list.
- `RemoteEndpoint` (Rust) mirrors both with defaults, so configured-but-unselected
  endpoints keep working.
- Validation for every configured endpoint: a declared program must be an
  absolute bounded path; arguments are bounded, and any argument containing a
  forbidden secret marker or shell fragment is rejected (same marker/fragment
  sets the batch dispatcher contract uses).
- A *selected* endpoint must declare its program: projection denies with
  `remote-pool:pool-endpoint-transport-program-missing` before any transport is
  considered.

## Verification

| Gate | Result |
| --- | --- |
| `cargo test -p mantle --bin mantle remote_farm_config::tests::` | 36 passed (includes relative-program, secret-argument, and shell-syntax denials) |
| `cargo test -p mantle --bin mantle remote_build::pool_config::tests::` | 3 passed (projection now also denies a missing transport command) |
| Nickel contract round-trip case | passes with `transport_program`/`transport_args` present |
| real CLI `remote pool check` with two transport commands and two owned descriptors | exit 0, same `mantle-remote-pool-preflight-v1` report |
| strict Clippy for the mantle targets; rustfmt on changed files | exit 0 |

## Not claimed

- No transport program is executed, resolved, or hashed here; S4 must verify the
  executable identity and bind it before use.
- The argument checks are policy filters, not a sandbox or injection proof.
