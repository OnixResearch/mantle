# Design: Separate the remote-build hexagon

## Context

Remote execution contains deterministic policy and extensive host effects in the same modules. Existing `distributed` submodules provide useful pure kernels, but the public seam still exposes Snix and store types.

Implementation status is intentionally narrower than this target diagram:
the checked I4/I5 local stdio composition uses the new core's bounded
attempt/transfer/output/receipt effects and application-owned output facts,
while std adapters retain credentials, signature and wire checks, physical
store admission, and process-local session effects. It does not add automatic
retry. Quantified resource/locality policy (I2), all SSH/local/external-batch
application ports (I3), and V1–V5 validation remain open; see
[`evidence/focused-validation.md`](evidence/focused-validation.md) for the
observed partial proofs, not a full hexagon completion receipt.

The target flow is:

```text
wire request
  -> inbound protocol adapter
  -> remote application command
  -> imperative application shell
  -> remote functional core
  -> decision, new state, events, and effect plan
  -> application shell
  -> capability ports
  -> transport, executor, store, persistence, and telemetry adapters
```

## Decisions

### Decision: create a strict remote core

A new `no_std + alloc` core will own protocol admission, normalized identities, state transitions, fencing, retry classification, resource and transfer decisions, output-admission inputs, deterministic events, and receipt preimages.

The core will consume owned bounded values. It will not import Snix, `crunch-store`, Tokio, filesystem paths, processes, environment state, clocks, random sources, credentials, network types, or renderers.

**Rationale:** These decisions define remote execution meaning and must remain replayable without host authority.

### Decision: use application-owned contracts

The application layer will define remote commands, observations, effects, outcomes, blockers, and port errors. Remote outputs will carry Mantle-owned output facts instead of Snix `PathInfo` or substitution reports.

Protocol and Snix adapters will translate at the boundary. Accepted wire bytes will remain unchanged.

**Rationale:** Provider-neutral ports cannot depend on one provider or store implementation.

### Decision: model effects explicitly

The core can request bounded effects such as transport send, attempt load or persist, lease reserve or release, input transfer, executor launch, output admission, and telemetry publication.

Each effect will have an identity, authority class, limits, and expected observation type. The shell will execute effects and feed observations into the next core transition.

**Rationale:** This keeps retry, authorization, and phase truth in the core without moving I/O inward.

### Decision: keep ports narrow and capability-based

Ports will describe genuine external capabilities. Initial groups are transport, attempt persistence, executor, remote store admission, credential verification, clock observation, random identifier generation, and telemetry publication.

A port will not combine unrelated capabilities. The composition root will select stdio, SSH, local, external-batch, store, and telemetry adapters.

### Decision: preserve active behavior work

Gateway, resource-policy, nominal-type, and Trellis changes remain behavior owners. This change moves their accepted logic behind the new boundary without redefining semantics.

Compatibility fixtures will run old and extracted decision paths over identical observations until cutover.

## Error ownership

The core returns typed blockers and rejected transition reasons. Ports return capability-specific failures. Adapters retain transport, process, store, and serialization errors. Presentation maps these values only at the outer boundary.

## Testing and evidence

Tests will include accepted transitions, malformed requests, stale fencing, retry rejection, untrusted outputs, transfer limits, adapter failures, replay equivalence, compile-fail dependency cases, and wire golden fixtures.

Evidence proves deterministic decisions and bounded observed adapter behavior. It does not prove worker honesty, transport confidentiality, effect success without observations, compiler correctness, or output trust beyond admitted evidence.

## Risks

- A wire DTO can become the core model. The inbound adapter must keep structural and admitted values separate.
- A large remote application service can replace the old module. Effect handling must stay capability-scoped.
- Active remote changes can conflict during migration. Each behavior owner must retain its requirement and evidence links.
