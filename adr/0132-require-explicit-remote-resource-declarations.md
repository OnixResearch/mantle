# ADR 0132: Require explicit remote resource declarations

## Status

Proposed (2026-10-26). Implements the resource-declaration boundary needed by
`declare-remote-worker-demand` and `add-evidence-driven-resource-policy`;
execution and lifecycle acceptance remain separate.

## Context

The ordinary remote derivation request constructor leaves quantified resource
requirements absent. Ordinary plans therefore cannot report a quantified
placement blocker. The gateway rejects that same absent declaration before it
can execute its existing resource selection and fenced dispatch path.

A configured machine class, a connected worker, concurrency, and prior work are
not declarations of a new job's minimum requirements. Deriving a requirement
from any of them would confuse requested resources with available capacity.

## Decision Drivers

- Preserve the existing no-capacity-inference and fenced-reservation rules.
- Make ordinary plan/build reports describe actual declared placement inputs.
- Admit gateway work only with an explicit operator declaration for its system.
- Reuse the existing canonical resource-vector contracts and normal commands.

## Decision

Ordinary remote builds accept an explicitly selected, bounded runtime export of
the existing Nickel resource-requirements contract. Plans and execution use the
same canonical declaration. Plans may observe placement and derived demand but
must not register workers, create attempts, reserve capacity, or grant work.

The gateway requires an explicit operator resource declaration indexed by the
reviewed derivation system. Missing, malformed, or incompatible declarations
fail closed; neither the class catalog nor worker presence supplies a default.
The declaration enters the existing request before resource selection and
ordinary coordinator admission. A selected class never lowers its minima.

Demand remains an observation of an unmet request, not worker registration,
quota, a lease, result authority, or permission to provision a worker. The
original placement blocker remains visible when coordination is unavailable.
No evidence-only product command or alternate admission path is introduced.

## Consequences

- Operators must provide declarations instead of relying on an absent vector.
- Existing unquantified ordinary builds keep their existing path.
- Gateway configurations must name the required per-system declarations.
- Real long-lived publication, retraction, and report observations still require
  execution through the normal owning adapter; a constructed fact or unit test
  does not establish that runtime behavior.
- Mantle scheduling and gateway maintainers own this declaration boundary. It
  does not grant deployment, provisioning, or release authority.
