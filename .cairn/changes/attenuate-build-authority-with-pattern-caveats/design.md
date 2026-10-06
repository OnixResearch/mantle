# Design: Attenuate build authority with pattern caveats

## Goal and scope

Add one bounded filter language for build authority, apply it at three grant
sites, and reuse the stack's authority components where they fit.

## Existing isolated-branch behavior

The e7d91c31 isolated branch descends from current `origin/main` at
7dcc8a84882a28aee8871898feba1a4c63ecef2a. This proves ancestry,
not that this already-existing review worktree was newly created for T1.1.
`remote_build::commit_remote_ticket_admission_for_state` authenticates and
validates a ticket before durable redemption, but has no signed delegated
job-set caveat. `OutputLookup` accepts exact store identities without a
pattern-limited view. `project_build::resolve_project_target` maps selectors
into extraction plans, but not transferable project namespace grants.
Production sites and the shared manifest remain unmodified by this slice.

## Approach review

| Family | Mechanism | Disposition | Required check |
| --- | --- | --- | --- |
| Central policy re-check only | Extend admission checks | Rejected as sufficient: the restriction cannot travel with the grant | Holder-attenuation fixture |
| Pattern caveats in the grant | Rewrite, reject, alternatives, unknown-rejects | Selected bounded local semantics | Filter and composition fixtures |
| Consume UCAN caveats | Pinned signed UCAN proof chain carries opaque Mantle caveats; local receiver checks them | Selected: ADR 0085; UCAN does not implement Mantle rewrites | Authenticated holder-delegation and local-recheck fixture |
| Basalt generic exact authority | Reviewed Mantle-specific Nickel contract plus trusted policy reference and previously verified UCAN facts | Selected boundary; closed Basalt action adapter has no Mantle actions | Policy-export and true receiver fact-binding fixture |
| New macaroon-style format | Local HMAC chain | Rejected: another authority format and bearer-bypass risk | None |

## Contract and component ownership

`config/authority-caveats/default.ncl` declares eight caveats per chain, eight
rewrite alternatives, 256 pattern/template bytes, 8192 serialized caveat
bytes, 512 input/output bytes, 16 segments and 32 declared jobs. Its
standalone pure `crates/crunch-authority-core` implementation accepts
`rewrite`, `reject`, `alternatives` and unknown-deny-all; ordinals `00`–`07`
must be consecutive. Alternatives are tried in declared order; the first
matching rewrite wins and a complete nonmatch discards the value. Every
candidate is bound and size-checked, including its transformed output.
A later caveat executes first, but every older prefix must independently
admit the same original input. The local receiver policy checks both the
original request and each ancestor output; a rewritten parent-rejected
input is never newly admitted. This core requires no cryptographic/runtime
dependency and is not yet a workspace member.

`config/authority-caveats/basalt-policy.ncl` proposes three disjoint
resource-prefix/ability contracts for Basalt's **generic**
`authority_core::evaluate_authority`; policy-owner review and an
independently trusted artifact reference are still required. The receiver
shell must verify UCAN signatures, signed holder invocation, chain
preservation, key resolution, revocation freshness, replay durability and
binding of verified facts to the exact request. Basalt's closed
`EnforcementAction` has no Mantle build, store or project variant; never
use another action as a shortcut. The standalone remote job set/class/
deadline is receiver-supplied, not bound to a signed producer identity or
an agreed UCAN scope payload. The remote owner must define issuer-to-producer
identity binding and a holder-delegable request carrier before T3.1.
The coordination daemon is not the receiver admission service.
No success frame, ticket redemption, metadata/content read or project
goal construction may precede local admission. Existing unsigned bearer
tickets cannot claim non-bypassable holder attenuation.

## Decisions

### Decision: Silent discard on rejection

**Choice:** A rejected assertion or message produces no protocol-level error.

**Rationale:** A peer must not learn which caveat or path failed. This is a
required production receiver behavior, not a claim that existing ticket or
store command error frames already satisfy it. Keep diagnostics local.

### Decision: Right-to-left composition, no widening

**Choice:** Caveats apply right to left, and composing chains cannot widen a
grant.

**Rationale:** The reviewed order keeps newer restrictions closest to the
input. A monotonic restriction property is testable and prevents a delegated
grant from exceeding its source.

### Decision: Evaluate stack authority first

**Choice:** Use UCAN's exact pinned signed proof/holder verification and
Basalt's generic exact authority with a reviewed Mantle-specific policy.
Never pass a Mantle action as Basalt's unrelated closed live action.

**Rationale:** An unsigned bearer suffix cannot enforce a delegable
restriction. Generic Basalt takes verified facts, not token bytes; Mantle's
receiver must validate cryptography, revocation and replay before providing
those facts and must perform its local effect check.

## Risks / Trade-offs

- A filter language is a security surface. Bound the chain, each serialized
  payload, patterns, values and branch count; malformed and unknown deny.
- New rewrites can make a previously rejected input appear acceptable to a
  parent. Evaluate every ancestor independently on the original input and
  recheck its output under the local site policy.
- Restricted store views must check normalized logical paths *before*
  PathInfo or content reads. Deny a broad `store list` under a restricted
  view unless enumeration itself is restricted at the backend.
- Project rewrites must retain original project identity and reject
  nested or cross-project names before constructing a generated goal.
- Effective chains may be reported only after a signed grant is verified;
  redact bearer secrets and never send a rejection reason to its sender.

## Non-Claims

The isolated pure filter cannot verify UCAN, provide Basalt policy
references, promise correct revocation/replay, authorize ticket redemption,
enforce store reads or construct project goals in the live CLI. Basalt
generic fact evaluation is not its closed live UCAN action adapter. No
production grant-site behavior is accepted before actual callsite fixtures
and a coherent shared Cargo/source handoff.
