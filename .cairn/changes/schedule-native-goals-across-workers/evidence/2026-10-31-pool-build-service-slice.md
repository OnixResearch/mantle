# Pool build service slice (2026-10-31)

Scope: the `BuildService` adapter that will let the ordinary lazy `Worker` send
one ready goal to one admitted pool endpoint. It owns no placement policy,
reservation, or fallback; it validates the request, asks an injected selector for
one endpoint, and delegates to that endpoint's own service.

## Contract (`crunch_build::distributed`)

- `PoolEndpointService<S>::new(endpoint_id, service)` pairs an admitted endpoint
  id with the service that executes for it.
- `PoolEndpointSelector` receives the concrete `BuildRequest` and the admitted
  endpoint ids in order, and returns an index into that same order.
- `PoolBuildService<S, Sel>::new(endpoints, selector)` rejects an empty pool and
  empty/duplicate endpoint ids (`PoolSelectionError::{NoEndpoints,
  InvalidEndpoint}`), and exposes `endpoint_ids()`.
- `do_build` validates the request through the existing
  `validate_remote_build_service_request` before any selection, maps selector
  denials to `remote-pool:*` invalid-input errors (`NoFit`, out-of-range), and
  delegates to exactly the selected endpoint.
- `do_build_cancellable` applies the same validation and selection and delegates
  the watch cancellation receiver, so watch teardown semantics stay with the
  selected service.

## Verification

| Gate | Result |
| --- | --- |
| `cargo test -p crunch-build --lib distributed::tests::pool_service` | 3 passed |
| strict Clippy `crunch-build`, `cargo fmt --check -p crunch-build` | exit 0 |
| `cairn validate --root . --strict` | 0 issues |

Fixtures use two recording services: the selected endpoint receives the request
and the other receives none; a `NoFit` selector denies without calling either; an
out-of-range index, an empty pool, and duplicate endpoint ids are rejected; and an
invalid request is rejected before selection.

## Not claimed

- The service is not wired into the pipeline builder yet; S5 supplies the selector
  (pool placement + scheduling policy) and the real per-endpoint services.
- Delegation proves routing only; it does not prove reservation, transport,
  output verification, or execution correctness.

## Seam finding

The service cannot be wired to a real selector yet: `BuildService::do_build`
receives only `BuildRequest`, which does not carry the realization key, required
system/features, sandbox/network modes, semantic accelerator classes, or prepared
resource requirement that pool placement needs. The design records the two
admissible resolutions (extend the prepared envelope, or select in the existing
`ReadyDerivationGoal` scheduling path). No selector may infer those facts from a
bare request, so dispatch wiring remains blocked rather than guessed.
