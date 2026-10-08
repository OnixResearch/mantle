# ADR 0094: Separate remote-build decisions from host effects

## Status

Proposed (2026-10-01). The extracted boundary remains subject to all remote
compatibility, runtime, and Cairn gates; a pure transition test alone is not
proof that a remote build ran or that its output is trusted.

## Context

The remote session shell mixes protocol admission, attempt fencing, retry,
transfer and trust decisions with stdio/SSH transport, credential resolution,
process execution, store access, durable state, clock observations, and output
rendering. Existing deterministic kernels in `crunch-build::distributed` still
expose Snix and store types in some provider-facing contracts. This prevents a
compiler-enforced inward dependency direction and makes deterministic replay
harder to distinguish from successful host effects.

## Decision Drivers

- Keep accepted framed wire bytes, ticket policy, diagnostics, output admission,
  resource and transfer semantics, and receipts unchanged.
- Make core decisions bounded and replayable without filesystem, process,
  network, clock, randomness, credentials, async runtime, store, or presentation.
- Return an effect *plan* rather than claiming an unobserved send, write, lease,
  executor launch, or output admission.
- Keep provider SDK, Snix, and `crunch-store` values in adapter projections.
- Preserve the separate gateway, resource-policy, nominal-type, and Trellis
  behavior owners and their existing requirement/evidence links.

## Decision

Place remote protocol admission, normalized identities, state transitions,
retry/fencing/resource/transfer decisions, output-admission inputs, deterministic
events, and receipt preimages in an alloc-capable `no_std` core. The application
layer passes bounded Mantle-owned commands and observations into this core and
receives typed states, blockers, outcomes, and effect plans. Every effect carries
an identity, authority class, limit, attempt binding, and expected observation;
the shell invokes only its matching capability port, then feeds the actual
success or failure observation into the next transition. The core cannot infer
success from an attempted effect. Transport, persistence, executor, store
admission, credential verification, clock, identifier generation, and telemetry
are separate capabilities. Adapters alone convert legacy wire DTOs, Snix
PathInfo/build values, and provider-specific errors. Migrate callers and delete
obsolete decision paths rather than keeping two live authorities.

Compatibility is established with old-versus-new fixture observations on exact
accepted/rejected frames, receipts, states, decisions, effects, outcomes, and
diagnostics, followed by an actual remote CLI session. The stored ticket and
backend identity checks remain before credential loading and state mutation;
selected backend forwarding is not weakened by the extraction.

## Alternatives Considered

### Wrap the current module in one remote service trait

Rejected: a single service interface would conceal host authority and retain
mixed decisions; compiling a pure crate would not enforce application-owned
ports or observed effects.

### Promote the wire frame or Snix PathInfo into the core model

Rejected: untrusted structural data and vendor-owned store authority would
become core input contracts. Inbound admission and outward projection must be
explicit.

## Consequences

The migration touches all remote callers, protocol adapters, and persistence
boundaries. Temporary dual-path fixtures may compare behavior before cutover,
but production must have one decision authority. Replay proves bounded decisions
for supplied observations only; it does not prove worker honesty, transport
confidentiality, execution success, output trust, compiler correctness, release
eligibility, or the success of an effect not yet observed.
