# Pool preflight slice (2026-10-31)

Scope: the operator-facing preflight that admits an explicitly selected pool
before any dispatch exists. It reads configuration, current bindings, and
caller-owned descriptors only; it opens no transport, uploads nothing, reserves
nothing, and executes no build.

## Configuration surface

- `RemoteEndpointBindingConfig` gains `build_bindings` (repeatable
  `BuildEndpointBindingConfig`) beside the existing single `build`. Names must be
  unique across both and every descriptor keeps the existing kind/scope/trust
  checks, so the singleton build-binding shape no longer blocks a pool.
- `lib/remote-builders.ncl` already carries the pool id, endpoints, per-endpoint
  credential reference and full output keys, and the bounded session policy from
  the previous slice.

## Command

`mantle remote pool check --builder-config PATH --builder-pool NAME
--remote-binding-config PATH --builder-ticket-fd ENDPOINT=FD ...`

1. Evaluate the Nickel farm configuration and load the operator binding config.
2. Parse `ENDPOINT=FD` values: malformed, standard-stream, duplicate, and
   over-limit inputs are rejected; each FD must be open in this process
   (`/proc/self/fd`), and it is never read, duplicated, or transferred.
3. Project the selected pool into `crunch_remote_core::pool::PoolSelection`:
   each endpoint must have a configured binding, a credential reference, and its
   separate output keys.
4. Resolve the current binding table from the endpoint binding store and apply
   `admits_pool`, which enforces current/revoked/expired bindings, supported
   transports and modes, unambiguous output keys, caller credentials, bounded
   session policy, and the no-inferred-capacity rule.
5. Print `mantle-remote-pool-preflight-v1` with the admitted endpoint names and
   session bounds.

## Verification

| Gate | Result |
| --- | --- |
| `cargo test -p mantle --bin mantle remote_build::pool_config::tests::` | 3 passed |
| real CLI: admitted pool with two current bindings and two owned descriptors | exit 0, report `mantle-remote-pool-preflight-v1` |
| real CLI: unknown pool / unowned FD / revoked binding | exit 3 with `remote-pool:pool-unknown`, `remote-pool:pool-ticket-fd-not-owned`, `remote-pool:pool-endpoint-binding-not-current` |
| `scripts/check-operator-command-contract.sh` | PASS (commands=213) after regenerating descriptors, surfaces, catalog, reference |
| `rustfmt --check` on changed files | exit 0 |

The CLI smoke used a hand-authored private binding state file
(`<state-dir>/remote-builders/endpoint-bindings.json`, mode 600, state dir mode
700) because no operator command creates bindings.

## Not claimed

- Preflight is not dispatch, capacity, transport readiness, or execution
  evidence; a passing pool still needs the S4 adapter and S5 worker wiring.
- No bearer was read; descriptors were only checked for ownership.
