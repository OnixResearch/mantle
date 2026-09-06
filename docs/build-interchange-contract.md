# Build interchange contract

`mantle-build-contract` publishes a versioned `no_std + alloc` boundary for external CI consumers.

<!-- r[impl mantle.build_interchange.boundary] -->

## Ownership

Mantle owns derivation evaluation, scheduling, sandbox execution, store and cache behavior, and native build evidence.

The contract owns only bounded request and observation values, deterministic BLAKE3 identities, and pure admission.

Consumers own candidate truth, process or transport behavior, cancellation, persistence, retry policy, CI state, required product meaning, and release decisions.

## Request

`mantle.build-request.v1` binds:

- idempotency, effect, and attempt identities;
- candidate and pipeline-revision identities;
- plan and policy identities;
- platform;
- sorted unique required product names.

Changing any field requires a new request identity.

## Observation

`mantle.build-observation.v1` binds:

- the exact request identity;
- outcome;
- sorted product observations;
- builder, worker, and store identities;
- cache kind and source identity;
- log, metric, and receipt identities;
- required non-claims.

Success requires every requested product exactly once. A cache or substitution observation cannot bypass product admission.

Failed, cancelled, and timed-out observations require a receipt identity. Unknown observations carry no receipt and remain unresolved.

## Native compatibility

The current aggregate Mantle report remains `crunch-build-report-v1`. A consumer shell maps one bounded report into this contract. The contract does not execute Mantle or parse host files.

## Fixtures

`fixtures/mantle-build-contract/producer/fixtures-v1.json` contains one request, a direct success, a cache-backed success, and a failure.

The Nix check regenerates this fixture and rejects drift. Nickel fixtures check positive request and success shapes. Negative fixtures reject schema and product-order drift.

## Hash dependency compatibility

The standalone contract permits compatible BLAKE3 `1.8.2` selections. It uses
only inherent `Hasher` and `Hash` APIs, with default features disabled and
`pure` enabled. It does not enable `traits-preview` or import digest traits.

The store's exact BLAKE3 1.8.2 pin remains separate. That pin preserves its
digest 0.10 trait boundary. The owner workspace lock remains unchanged.
Consumers retain their own reviewed hash selections and lockfiles.

Two independent test workspaces select exactly BLAKE3 1.8.2 and 1.8.7. Both
admit the original producer identities and reject malformed observations.
Their local paths refer only to this repository's contract. They are test
fixtures, not sibling-worktree product dependencies.

Run the maintained Nix matrix:

```sh
nix build .#checks.x86_64-linux.mantle-build-contract-hash-minimum \
  .#checks.x86_64-linux.mantle-build-contract-hash-current --no-link -L
```

Each check runs the shared consumer controls and a wasm compilation. The
matrix certifies neither every future compatible hash version nor native
build execution. Published downstream evidence must name the actual linked
contract and authority cohorts. See ADR 0122 for the dependency boundary.

## Non-claims

An admitted observation does not prove build correctness, sandbox completeness, cache truth, product semantics, reproducibility, or release readiness.

A plan is not evidence that a build ran. A receipt records bounded facts; it does not grant consumer CI or release authority.
