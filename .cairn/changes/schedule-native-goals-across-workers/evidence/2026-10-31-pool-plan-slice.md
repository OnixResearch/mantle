# Pool route plan slice (2026-10-31)

Scope: `remote pool plan`, the side-effect-free route renderer the design asks
for. It resolves current bindings, verifies declared routes and transport
identities, and prints them. It takes no credential, opens no socket, uploads
nothing, reserves nothing, and executes nothing.

## Admission split

`crunch_remote_core::pool` now separates the two decisions:

- `plans_pool(selection, bindings, now)` performs every pool check except
  caller credentials (identity, count, session bounds, transports, capability
  modes, output trust, current bindings, no-inferred-capacity).
- `admits_pool(...)` adds exactly one caller-authorized credential per selected
  endpoint and delegates to `plans_pool`.

This keeps the plan path from needing a credential while never duplicating the
admission rules.

## Command

`mantle remote pool plan --builder-config PATH --builder-pool NAME
--remote-binding-config PATH`

Per endpoint it reports: name, transport, transport program path and remeasured
BLAKE3, transport arguments, output key names, worker generation, declared
concurrency, and whether a resource inventory was declared; plus the bounded
session policy. Only `stdio` and `ssh-stdio` are planned today; a declared `p2p`
endpoint fails closed with `remote-pool:pool-transport-unsupported` rather than
pretending a transport exists.

## Verification

| Gate | Result |
| --- | --- |
| `cargo test -p crunch-remote-core --lib pool` | 12 passed (adds plan-only admission without credentials) |
| `cargo test -p mantle --bin mantle remote_build::pool_config::tests::` | 5 passed (adds route rendering and p2p denial) |
| real CLI `remote pool plan`, two endpoints, no descriptors supplied | exit 0, `mantle-remote-pool-plan-v1` with both routes and remeasured digests |
| `scripts/check-operator-command-contract.sh` | PASS (commands=214) after regenerating descriptors, surfaces, catalog, reference |
| strict Clippy (mantle targets), rustfmt, `cairn validate --strict` | exit 0 / 0 issues |

## Not claimed

- A plan is not admission, dispatch, transport readiness, capacity, or execution
  evidence; it does not check descriptor ownership or consume a ticket use.
- Declared inventory and concurrency are configuration facts, not observed
  capacity.
