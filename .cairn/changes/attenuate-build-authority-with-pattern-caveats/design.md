# Design: Attenuate build authority with pattern caveats

## Goal and scope

Add one bounded filter language for build authority, apply it at three grant
sites, and reuse the stack's authority components where they fit.

## Current behavior

Remote builder tickets are verified and redeemed in the production receiver
before it advertises admission; they currently bind expiry, remaining uses,
endpoint, build time and upload size, not a holder-delegated job set. Store
`OutputLookup` can read any exact store path through its fixed read authority
(ADR 0058). Project selectors resolve within the current project but do not
carry portable namespace restrictions. None of these legacy values proves a
holder-attenuated delegation.

## Approach review

| Family | Mechanism | Disposition | Required check |
| --- | --- | --- | --- |
| Central policy re-check only | Extend admission checks | Rejected as sufficient: the restriction cannot travel with the grant | Holder-attenuation fixture |
| Pattern caveats in the grant | Rewrite, reject, alternatives, unknown-rejects | Selected direction | Filter and composition fixtures |
| Consume UCAN caveats | Pinned signed UCAN proof chain carries opaque registered caveats; Mantle checks patterns and sites; Basalt generic exact-authority core consumes verified facts under a reviewed Mantle-specific policy | Selected: ADR 0090; neither UCAN nor Basalt implements Mantle rewrites | Authenticated holder-delegation, reviewed policy and local-recheck fixture |
| New macaroon-style format | Local HMAC chain | Rejected: second cryptographic authority and bearer-bypass risk | None |

## Contract and component ownership

- Pure core: filter parsing and validation, pattern matching with bindings,
  template instantiation, right-to-left composition, and attenuation of a
  grant value.
- Shell: grant issuance, transport, and admission calls that consume the
  filter result.
- Policy: the accepted caveat set and the maximum chain length are declared.
  Unknown caveats reject everything.

The typed Nickel policy at `config/authority-caveats/default.ncl` declares
eight caveats per chain, eight rewrite alternatives, 256 pattern/template
bytes, 8192 serialized caveat bytes, 512 canonical input/output bytes and 16
path segments. The pure core
matches complete slash-delimited segments; `:name` captures one segment and a
template may reuse only names in its pattern. Duplicate names, empty/dot
segments, unbound names, malformed known variants and oversized inputs fail
closed. An unknown variant is a deny-all filter.

The adapter must first authenticate the verified UCAN holder invocation and
preserve all parent caveat payloads in the proof chain. A caveat observation
must be non-consuming; the receiver binds the operation to the original
request, checks original policy and every parent-prefix output, then performs
the final rewritten request check before any ticket redemption, store read or
project goal construction. The receiver supplies its own current time,
revocation, replay and site-policy facts; it may not trust daemon policy
observations as a substitute for local admission. It sends no protocol frame
on denial. Legacy bearers stay usable without being described as delegated.

### Receiver adapter map (integration gate)

The production remote server's `commit_remote_ticket_admission_for_state`
already holds the ticket-state mutation guard, loads current state, obtains
server time, authenticates the bearer, validates `ConcreteBuildRequest`, then
redeems the ticket durably before writing `AuthOk` or `MissingInputs`. An
attenuated route must validate the verified UCAN holder's bound operation and
explicit proof-chain caveats inside this same pre-redemption interval. The
declared job identity is the concrete `request_id` (and if a coordinator
attempt exists, its independently validated `production_attempt.job_id`),
not a client clock/endpoint claim; the output class is the validated payload
kind and declared output names, not arbitrary caller narrative. Check the
current server time against both the ticket's expiry and caveat deadline.
Failure returns before redemption, runtime construction, input transfer and
all success frames. Sender-visible error text must not include the denied job,
path, caveat or reason. Legacy ticket admission remains unchanged when no
attenuated grant is presented.

`OutputLookup::find`, `find_with_layer`, `find_remote` and the shell
`store info`/`store list` paths are potential information-bearing reads. A
restricted view gates an exact normalized logical path before every
PathInfo or content read. A broad `store list` under a restricted view must
be denied before scanning, unless a future backend can enumerate only the
authorized pattern; filtering an unrestricted listing afterward is not a
restricted view. A read outside the view has no path disclosure.
`project_build::resolve_project_target` lowers `.#selector` into
`ProjectTarget`; before generated Nickel evaluation, a project grant lowers
`/project/<owner>/declared/<goal>` into
`/project/<owner>/goal/<goal>`, rejecting a mismatched owner, nested/unadmitted
goal or a rewrite into another project's namespace. The original declaration
and every parent-prefix output are checked under the same owner. No ambient
daemon round trip authorizes either site.

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

**Rationale:** The reviewed order keeps the newest restriction closest to
input. Composition alone does not preserve acceptance: a rewrite can map a
previously rejected input into a value accepted by its parent. Re-evaluate
each parent prefix on the same original input, including the old output's
site-specific policy check, before admitting a newly appended caveat.

### Decision: Evaluate stack authority first

**Choice:** Pinned UCAN supplies cryptographic issuance, holder proof and
proof-chain attenuation with caller-owned revocation/replay ports. Mantle
owns the bounded rewrite semantics, receiver-local checks and truthful
binding of verified facts. Basalt's **generic** `authority_core` consumes
those facts under a reviewed Mantle-specific policy and shell-trusted
reference. Its closed live `EnforcementAction` adapter has no Mantle
build/store/project action; never map an unrelated action to those sites.
Neither generic Basalt nor the filter proves signature/proof/revocation/
replay freshness, and no signed-UCAN parent can be attenuated by appending
an unsigned restriction to an existing bearer (ADR 0090).

### Security review cases for integration

| Attempt | Receiver invariant | Observable fixture |
| --- | --- | --- |
| Holder drops, changes, duplicates or reorders a parent caveat | Verify UCAN proof and exact parent payload preservation; reject ordinal gap/duplicate/order ambiguity locally | No redeem, store read, goal evaluation or success frame |
| Caveat is unknown, malformed, over 8192 serialized bytes, or exceeds any component bound | Deny the entire request before effect; never treat an unknown variant as identity | No success frame or path/caveat diagnostic to sender |
| New rewrite maps a parent-rejected input to an apparently allowed output | Recheck every ancestor on the original input and every ancestor output under site policy | Parent-rejected input remains rejected |
| Sender changes client time, endpoint, job ID, output class or deadline | Bind holder signature to exact normalized request; use receiver clock, authenticated endpoint and current grant state | Second/expired/wrong-class job starts no work and redeems no ticket |
| Revocation port unavailable or stale; holder proof/replay missing | Fail closed before admission; no daemon observation stands in for local freshness | No server success frame or effect |
| Store path uses another prefix, prefix lookalike, dot segment, or unmatched digest/name | Gate normalized logical path before PathInfo or remote reads | No unauthorized path value or metadata disclosure |
| Declaration selects another project or a nested goal after rewriting | Bind original project to grant; require one normalized goal segment in the same namespace | No Nickel evaluation of another project's selector |

The scratch signed-grant fixture checks only the subset it actually executes.
Production acceptance requires the same cases at the receiving CLI/shell
boundaries with both an admitted request and a rejected request that leaves
the real effect state unchanged.

## Risks / Trade-offs

- A filter language is a security surface. It stays bounded, with chain-length
  and pattern-size limits and fail-closed parsing.
- Rewrites can surprise reviewers. Every grant reports the effective chain.
- Project scoping changes what a project can request. The isolation fixture
  proves a cross-project request is refused.

## Non-Claims

- An attenuated grant proves the restriction over the declared filter language.
- It does not prove that the authorized build was correct or trustworthy.
