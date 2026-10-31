# Pool selection schema and admission slice (2026-10-31)

Scope: the S2 configuration/admission half of this change — per-endpoint caller
credential references, separate full output keys, bounded pool-session policy,
and an explicit admission decision for a selected pool. No pool flag exists yet
and no dispatch uses this decision, so nothing here is dispatch, capacity, or
execution evidence.

## Nickel contract (`lib/remote-builders.ncl`)

- `RemoteEndpoint` gains `credential_reference` (shell-owned handle name; never
  a bearer value or FD number) and `trusted_output_keys` (separate, full
  material). Both remain optional so configured-but-unselected pools keep
  working; a selected pool's admission decides them.
- New `RemotePoolSessionPolicy` with `max_concurrency`, `job_count_max`,
  `session_bytes_max`, attached to `RemoteBuilderPool` as `session`.
- `TrustRoot` moved above `RemoteEndpoint` because Nickel `let` bindings are not
  recursive; `nickel typecheck` and the evaluated contract both pass.

## Rust evaluation and validation (`src/remote_farm_config.rs`)

- Mirrored fields on `RemoteEndpoint` and `RemoteBuilderPool.session`
  (`RemotePoolSessionPolicy`, defaulting to concurrency 1, 1,000,000 jobs,
  1 GiB).
- New denials, checked before any dispatch and without reading a bearer:
  session bounds outside `1..=MAX_POOL_CONCURRENCY` / `1..=MAX_SESSION_JOBS` /
  `1..=MAX_SESSION_BYTES`; a credential reference that is empty-whitespace,
  oversized, or shared by two endpoints of one pool; an output key without both
  name and full material; and a key name or material repeated on one endpoint.

## Pure admission core (`crates/crunch-remote-core/src/pool.rs`)

`admits_pool(selection, current_bindings, caller_credentials, now_unix_s)`
decides in declaration order and returns either the admitted endpoint names and
session bounds or one of: `InvalidPoolId`, `EmptyPool`,
`EndpointCountExceeded`, `DuplicateEndpoint`, `UnsupportedTransport`,
`UndeclaredCapabilities`, `UnsupportedSandboxMode`, `UnsupportedNetworkMode`,
`CapabilityCountExceeded`, `UndeclaredOutputTrust`, `AmbiguousOutputKey`,
`InvalidCredentialReference`, `DuplicateCredentialReference`,
`MissingCallerCredential`, `UnboundedSessionPolicy`, `InferredCapacity`, and
`EndpointBinding(existing endpoint error)`.

Two properties the plan states as hard gates are decided here:

- A current admitted binding is required per endpoint through the existing
  `EndpointBindingTable::resolve`, so unknown, revoked, expired, stale-generation,
  and mismatched bindings all deny before transport or upload.
- Concurrency greater than one requires at least one endpoint to declare a
  quantified resource inventory: `InferredCapacity` refuses the alternative.
  One job needs no capacity declaration.

## Verification

| Gate | Result |
| --- | --- |
| `cargo test -p crunch-remote-core --lib pool` | 7 passed |
| `cargo test -p mantle --bin mantle remote_farm_config::tests::` | 33 passed (includes both new denial tests) |
| `cargo test -p mantle --bin mantle remote_farm_config::tests::nickel_pool` | 2 passed (round-trip and type-mismatch rejection) |
| `cargo clippy -p crunch-remote-core --all-targets --no-deps -- -D warnings` | exit 0 |
| `cargo clippy -p mantle --bin mantle --no-deps -- -D warnings` | exit 0 |
| `cargo fmt --check -p crunch-remote-core` | exit 0 |
| `nickel typecheck lib/remote-builders.ncl` | exit 0 |

## Remaining S2 work

- The proposed CLI surface (`build --builder-config PATH --builder-pool NAME`,
  repeatable `--builder-ticket-fd ENDPOINT=FD`) and its descriptor-ownership and
  cardinality checks before dispatch.
- Resource-profile selection beyond the declared inventory, and migration of the
  remaining configuration/report consumers onto the pool selection.

S2 stays unchecked until those land.
