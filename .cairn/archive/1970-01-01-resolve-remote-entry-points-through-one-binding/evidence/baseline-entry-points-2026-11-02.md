# Remote entry-point baseline (2026-11-02)

Scope: T1.1. Record every remote entry point, its name shape, its admission
checks, and the current revocation behavior on an isolated worktree created
from `origin/main` at `905144e6f4feca377c5c07c926932cbcc21a1594`
(`/tmp/resolve-baseline-wt`). This is a current-tree baseline; it does not claim
a before-change comparison, acceptance, or archive.

## Resolve step

- Pure table: `crates/crunch-remote-core/src/endpoint.rs`
  - `EndpointBinding` = spec + generation (line 42); `EndpointBindingRecord` =
    binding + revoked tombstone (line 48).
  - `EndpointBindingTable::resolve` (line 198) validates the presented
    descriptor, finds the same `(kind, name)`, compares the exact binding,
    rejects revoked and expired records, and returns a borrowed local admission
    view.
  - `EndpointBindingTable::verify_received` (line 230) first matches the
    receiver's exact kind, scope, trust requirement, and store prefix, then
    resolves.
  - `EndpointBindingTable::revoke` (line 185) retracts exactly one generation;
    tombstones count toward the bound and are never deleted by the API.
  - `EndpointBindingTable::create` (line 146) admits a first generation or
    recreates a revoked name at generation + 1 only with the exact previous
    generation.
- Durable shell: `src/remote_build/endpoint.rs`
  - `EndpointBindingStore::resolve` (line 132) and `verify_received` (line 142)
    read the durable table under a shared binding lease.
  - `EndpointBindingStore::revoke` (line 122) and `create` (line 106) mutate
    under the exclusive lease.

## Entry points

| Kind | Name shape | Admission checks | Current revocation behavior | References |
| --- | --- | --- | --- | --- |
| `BuilderTicket` | Endpoint name: lowercase alnum at both edges, interior `-_.`, no `..`, bounded bytes (`validate_endpoint_name`). Ticket identity separately bounded as lowercase hex (`TicketId`, generated/legacy validation). | Binding resolve with exact kind `BuilderTicket`; ticket credential parse and verifier check; ticket `revoked` flag; expiry; scope/trust/store-prefix match; store-prefix and kind checks in `bind_remote_production_dispatch`. | Ticket `revoked` rejects the credential; binding revocation retracts the endpoint generation; new sessions fail closed; admitted content is untouched. | `crates/crunch-remote-core/src/endpoint.rs:19,249`; `src/remote_credentials.rs:38,458,1001,1009`; `src/remote_build.rs:3368`; `src/remote_build/endpoint_service.rs:188`; `src/remote_build/pool_factory.rs:386`; `src/nix_gateway/api_host.rs:232` |
| `Transfer` | Endpoint name (same validation); transfer session manifest carries the bound descriptor and the transfer scope. | Receiver re-resolves before manifest/checkpoint/payload work via `admit_received` with exact kind, scope, trust requirement, store prefix, generation, and expiry. | Binding retraction makes new sessions fail with `Revoked`; already-admitted content is not affected. | `src/nix_gateway/native.rs:286`; `src/remote_build/endpoint.rs:142`; `crates/crunch-remote-core/src/endpoint.rs:230` |
| `Substituter` | Endpoint name (same validation); substituter URL scope plus trust requirement. | Receiver `admit_received` with exact kind/scope/trust/prefix; signed NAR and trusted key checks stay behind the resolve step. | Binding retraction refuses new sessions; signed content already locally admitted still verifies (`exact_cache_binding_pulls_signed_bytes_and_revocation_preserves_local_admission`). | `src/nix_gateway/native.rs:526`; `src/remote_build/tests/source_binding_tests.rs`; `crates/crunch-store/src/source_admission.rs` |
| `BaseLayer` | Endpoint name (same validation); base-store scope and canonical path. | Source admission through the resolved binding with exact kind/scope/trust/prefix; base-layer leases retain independent admission. | Binding retraction; existing leases preserve generation and revocation fences (`shared_cache_and_base_leases_preserve_signed_content_generation_and_revocation_fences`). | `src/remote_build/tests/source_binding_tests.rs:485`; `crates/crunch-store/src/source_admission.rs:49`; `crates/crunch-remote-core/src/endpoint.rs:19` |

## Cross-cutting fail-closed reasons

`InvalidName`, `InvalidScope`, `InvalidTrustRequirement`, `InvalidStorePrefix`,
`InvalidExpiry`, `InvalidGeneration`, `TableFull`, `DuplicateBinding`,
`UnknownBinding`, `AlreadyActive`, `Revoked`, `Expired`, `StaleGeneration`,
`BindingMismatch`, `GenerationExhausted` (`crates/crunch-remote-core/src/endpoint.rs:60`).

## Non-claims

- Current-tree baseline only; no pre-change comparison or parity claim.
- Not acceptance, spec sync, or archive evidence.
- The focused-test run for this change is recorded separately in
  `focused-tests-current-tree-2026-11-02.json`.
