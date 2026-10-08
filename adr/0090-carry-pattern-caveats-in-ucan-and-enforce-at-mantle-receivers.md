# ADR 0090: Carry pattern caveats in UCAN and enforce at Mantle receivers

## Status

Proposed (2026-09-30). The stack boundary is evaluated; local filter and grant-site integration evidence remain required before acceptance.

## Context

Mantle's existing remote tickets are opaque server-verified bearers with expiry, use count and endpoint limits. Possession of an unrestricted parent bearer cannot be made into a non-bypassable holder-attenuated child merely by appending an unsigned restriction: the holder can pass the original bearer instead. Store capability views and project selectors likewise have no transferable holder proof. The change requires rewrite, reject, alternatives, unknown-rejects-all, bounded right-to-left composition and independent receiver checks, without creating a second cryptographic authority format.
The stack ownership contract (`../../WIKI.adoc`, UCAN and Basalt rows; `../../VOCABULARY.html`, authority decision and revocation freshness) puts token issuance, signature verification, proof chains and revocation hooks in UCAN, and application policy in Basalt. UCAN's public API (`../../ucan/README.md`, `../../ucan/docs/public-api.md`, `../../ucan/docs/ucan-1-authorization.md`) provides signed delegated compact tokens, holder-signed invocation, opaque domain/type/key/payload caveats, bounded proof traversal, proof-chain caveat preservation, and caller-supplied revocation and replay ports. The compact-token verifier checks preservation of the *entire caveat identity and payload* at delegation; it does not implement Mantle's pattern semantics or claim that adding a rewrite is monotonically restrictive. The final UCAN 1.0 profile has a distinct bounded policy language; it does not silently inherit compact-token or Mantle rewrite semantics. Basalt's current contracts (`../../basalt/README.md`, `../../basalt/docs/authority-core.md`, `../../basalt/docs/ucan-semantics-contract.md`, `../../basalt/docs/ucan-protocol-profiles.md`) accept previously verified UCAN facts and enforce exact resource/action/request bindings through reviewed policy. They do not parse tokens, infer a profile, evaluate arbitrary caveat payloads, fetch revocations, or operate Mantle's build/store/project effects. Basalt vendors a reviewed UCAN snapshot; it does not automatically track the adjacent checkout.

## Decision drivers

- A holder must be able to delegate less authority without consulting the issuer or enabling the recipient to reconstruct the parent grant.
- The original Mantle site policy must continue to hold for both the submitted assertion and any rewritten result.
- Unknown or malformed caveats must reject before an effect; rejection sends no protocol-level explanation.
- The UCAN token profile and Basalt policy identities are explicit and pinned, not inferred from token bytes.

## Decision

Use UCAN's existing signed delegation/proof-chain and verified-holder invocation surfaces for portable holder-attenuated grants, subject to reviewed license and pinned UCAN/Basalt compatibility. Mantle owns a small `no_std` pattern-filter core and its bounded caveat payload vocabulary. Its application adapter interprets only registered Mantle caveat domain/type/key values via UCAN's explicit caveat-policy hook, carries all parent restrictions across delegation, and checks both the receiver's local site policy and the reviewed Mantle-specific policy through Basalt's generic exact-authority core before effects. It rejects unknown, missing-policy, over-bound and malformed caveats without attempting another codec. Caveat observation is non-consuming; quota redemption and execution remain downstream atomic boundaries.

Basalt's live compact-token adapter currently has a *closed* set of
`EnforcementAction` values for document and OnixOS/state-sync actions; it
does not expose remote build, store view, or project declaration actions.
Mantle must not relabel these operations as one of those reviewed actions
or claim that Basalt's closed `verify_ucan_enforcement_request` authorizes
them. The intended integration uses UCAN's pinned signed resource/ability
delegation and holder verification, Mantle's bounded receiver-side filter,
and Basalt's **generic** `authority_core::evaluate_authority` with reviewed
Mantle-specific policy/contract/resource/ability and a shell-trusted
`Policy` reference. The generic authority core consumes already-verified
facts; it does not verify signatures, proof chains, revocation freshness
or replay durability. Mantle's receiver shell remains responsible for
these prerequisites, for accurate binding of verified grant/receipt facts,
and for effects. Neither an unsigned caveat appended to a bearer nor
generic strings pretending to be a reviewed Basalt action are authority.

The isolated scratch proof against Basalt's **exact vendored UCAN revision**
`c483c7b58c42ec6636e9b8b8c0f73a2b609bf21e` passed 2/2 signed
delegation/holder fixtures, plus 1/1 signed store/project filesystem-effect
fixture. These are UCAN cryptographic and local-filter observations, **not**
proof of a Mantle-specific reviewed Basalt policy mapping, production receiver
admission, durable revocation or replay. The missing Mantle policy and live
grant-site integration remain explicit acceptance blockers.


The reviewed compact-token caveat identity uses a registered Mantle domain,
versioned filter type, and fixed-width decimal ordinal key in the existing
UCAN caveat fields; its payload is a bounded, canonical rewrite/reject/
alternatives description, **not another bearer or token format**. The
receiver rejects duplicate or gapped ordinals, conflicting payloads, excess
caveats, and any unknown type. UCAN proves parent caveat identity/payload
preservation, but not this ordering or the semantic nonwidening property:
Mantle checks those locally. Remote job sets/output class/deadline, store
logical-path patterns, and project identity are site-specific restrictions
checked conjunctively over every retained parent grant; replacing a parent
scope with a different payload is not attenuation.

Append-only caveats apply newest first (right to left). A receiver evaluates every parent-chain prefix against the **same original request** before accepting the new composed result, then rechecks both original and rewritten requests against its site policy. This extra parent gate is necessary: merely composing rewrite functions in reverse order can transform a previously rejected input into one accepted by an ancestor. For project declarations, the rewritten goal is materialized only after original project ownership passes; a rewrite never changes which project's grant is checked. Grant presentations report an effective, redacted caveat chain without emitting bearer secrets or rejected-path details. Store views reject paths outside their admitted logical prefix before any metadata or content read. Remote receivers recheck authenticated grant, requested job, output class and deadline before admission or state-consuming redemption. Coordination from ADR 0080 may issue grants but cannot substitute a receiver's local admission.

Original remote bearer tickets remain backward compatible; they are not described as cryptographically holder-attenuable. An attenuated grant must be presented via a verified UCAN delegation, not an unsigned ticket suffix or a claim that a receipt reconstructs live authority. Mantle's filter does not mint a token, sign a message, choose key custody, retrieve proofs, consult revocation/replay state or parse an alternate bearer.

## Rejected alternatives

- Central policy check alone: cannot transfer a holder's non-bypassable restriction.
- An unsigned sidecar or a locally derived HMAC/macaroons format: either bypassable by the parent bearer or a second, unreviewed cryptographic authority surface.
- Treat UCAN `CaveatPolicySet` or Basalt's decision-fact DTO as an implementation of pattern rewrites: these preserve and evaluate supplied facts but do not implement this language, prove ordering, or execute effects.
- Pretend UCAN's final-profile `like` policy supplies assertion rewriting: it matches a different protocol profile and has no such documented rewrite contract.

## Consequences and non-claims

The Mantle filter can prove monotonic acceptance only for its bounded, normalized inputs and explicit parent-prefix gate. UCAN signature and proof validation, correct key resolution, revocation freshness, durable replay, atomically spent quotas, site-specific request normalization, filesystem confinement, authorized build correctness, cache trust, and release eligibility remain separate responsibilities. A Basalt receipt records an observed exact admission and cannot recreate live authority. Before integrating the transport, confirm the UCAN/Basalt license and pinned snapshot compatibility; do not elevate this Proposed decision into a claim of completed remote/store/project integration.
