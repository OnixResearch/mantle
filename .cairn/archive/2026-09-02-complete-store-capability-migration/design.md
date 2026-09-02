# Design: Complete store capability migration

## Context

ADR 0058 introduced concrete store capability views. Builder and pipeline paths already use several of them. Root command and remote-transfer paths still receive `StoreHandle` or raw Snix services.

The target flow is:

```text
application request
  -> narrow store capability
  -> crunch-store shell
  -> Snix and local-state adapters
  -> typed observation or result
```

## Decisions

### Decision: expose operation-specific concrete capabilities

`crunch-store` will expose concrete values for coherent authorities. Initial owners are output lookup, output admission, transfer object reads and writes, archive access, attestation lookup, root registration, action-result access, and administration.

Each capability will expose named high-level operations. It will not return raw PathInfo, directory, blob, signing, HTTP, overlay, or database services.

**Rationale:** Concrete values make authority reachable only through reviewed methods. A generic store trait or broad handle would recreate the current problem.

### Decision: keep vendor translation inside the store shell

Public application requests and results will use Mantle-owned values. `crunch-store` adapters will translate those values to Snix records and services.

Compatibility surfaces that must expose a Nix PathInfo will use one explicit projection at the adapter boundary. The projection will not become a general application port type.

**Rationale:** Mantle owns the capability contract. Snix remains an implementation and compatibility dependency.

### Decision: separate admission from publication execution

Successful persistence and output admission will return an admitted-output result plus a bounded publication effect plan. The application shell will execute configured publishers and submit typed observations for report construction.

Publication failure will not retroactively erase truthful local admission. It will remain a separate failed effect observation.

**Rationale:** An effect plan requests work. It does not prove that publication occurred.

### Decision: migrate callers by capability slice

The migration will proceed through transfer, remote build, store commands, pipeline, and remaining cache or archive callers. Each slice will remove broad-handle access before the next slice starts.

Temporary adapters must be private and must not add new `StoreHandle` consumers.

**Rationale:** Small ownership slices preserve behavior and make authority changes reviewable.

### Decision: enforce topology with compiler and source guards

Compile-fail fixtures will prove that callers cannot obtain raw services or call methods from an unrelated capability. A deterministic architecture checker will inspect dependency direction and the remaining broad-handle allowlist.

Positive fixtures will prove each intended operation through its narrow capability.

## Error ownership

Core store decisions retain typed domain errors. Capability methods return store capability errors. Snix, filesystem, network, and database errors remain adapter sources and are mapped once. CLI presentation remains outside `crunch-store`.

## Compatibility and evidence

Golden fixtures will compare PathInfo bytes, signatures, reports, logical paths, publication plans, and accepted terminal outcomes. Evidence proves capability reachability and observed effects only. It does not prove content correctness, publisher honesty, or release eligibility.

## Risks

- A capability can become a renamed broad handle. Reviews must reject unrelated methods on one view.
- A compatibility projection can leak back into application ports. Dependency guards must reject that path.
- Publication ordering can drift. Golden tests must bind local admission and publisher observation order.
