# Design: Attenuate build authority with pattern caveats

## Goal and scope

Add one bounded filter language for build authority, apply it at three grant
sites, and reuse the stack's authority components where they fit.

## Current behavior

Remote builder access uses tickets with declared job and quota bounds checked
by the coordinator. Store capability views are Rust values with fixed
operations (ADR 0058). Project scoping lives in `mantle-project`. Each site is
correct, and each site re-derives policy at admission time.

## Approach review

| Family | Mechanism | Disposition | Required check |
| --- | --- | --- | --- |
| Central policy re-check only | Extend admission checks | Rejected as sufficient: the restriction cannot travel with the grant | Holder-attenuation fixture |
| Pattern caveats in the grant | Rewrite, reject, alternatives, unknown-rejects | Selected direction | Filter and composition fixtures |
| Consume UCAN caveats | Use the stack token component | Preferred when the contract supports pattern filters | Component evaluation |
| New macaroon-style format | Local HMAC chain | Deferred unless the stack evaluation fails | Not blocking |

## Contract and component ownership

- Pure core: filter parsing and validation, pattern matching with bindings,
  template instantiation, right-to-left composition, and attenuation of a
  grant value.
- Shell: grant issuance, transport, and admission calls that consume the
  filter result.
- Policy: the accepted caveat set and the maximum chain length are declared.
  Unknown caveats reject everything.

## Decisions

### Decision: Silent discard on rejection

**Choice:** A rejected assertion or message produces no protocol-level error.

**Rationale:** The reviewed model gives no feedback, because a live peer cannot
distinguish rejection from death. Mantle's admission already fails closed
without explaining itself to the sender; this matches that behavior. Debugging
aid stays in local logs.

### Decision: Right-to-left composition, no widening

**Choice:** Caveats apply right to left, and composing chains cannot widen a
grant.

**Rationale:** The reviewed order keeps newer restrictions closest to the
input. A monotonic restriction property is testable and prevents a delegated
grant from exceeding its source.

### Decision: Evaluate stack authority first

**Choice:** Before any new token construction, evaluate whether UCAN can carry
the required pattern caveats under Basalt policy.

**Rationale:** A second authority format in one stack is a trust cost. The
stack already owns issuance, verification, proof chains, and revocation.

## Risks / Trade-offs

- A filter language is a security surface. It stays bounded, with chain-length
  and pattern-size limits and fail-closed parsing.
- Rewrites can surprise reviewers. Every grant reports the effective chain.
- Project scoping changes what a project can request. The isolation fixture
  proves a cross-project request is refused.

## Non-Claims

- An attenuated grant proves the restriction over the declared filter language.
- It does not prove that the authorized build was correct or trustworthy.
