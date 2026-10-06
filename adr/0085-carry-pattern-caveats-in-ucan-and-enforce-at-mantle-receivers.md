# ADR 0085: Carry pattern caveats in UCAN and enforce at Mantle receivers

## Status

Proposed (2026-10-04). Pure filtering and isolated stack fixtures do not establish production remote/store/project grant admission.

## Context

A remote access ticket is a server-verified bearer. Appending an unsigned caveat to it cannot make attenuation non-bypassable: the bearer holder can omit the caveat or present the parent. Existing `OutputLookup` is an exact store-path read capability, and project selector lowering has no transferable project-goal grant. A receiver must decide whether a **signed**, holder-delegated grant, rather than the mere possession of a ticket, authorizes its concrete action.

UCAN's pinned signed compact-token surface carries opaque `{domain,type,key,payload}` caveats, preserves complete parent caveat identities and payloads across a signed delegation, verifies proof chains and signed holder invocation, and exposes caller-owned revocation and replay ports. It neither implements Mantle's pattern rewrites nor decides business-site policy. Basalt's live `verify_ucan_enforcement_request` has a *closed* action enumeration for documents and OnixOS/state sync: remote builds, store views and project declarations are not members. Basalt's **generic** `authority_core::evaluate_authority` can instead consume already-verified grant and receipt facts against a caller-trusted policy reference and exact contract/resource/ability/request binding. It does not authenticate token bytes or provide revocation/replay freshness.

## Decision drivers

- No new cryptographic token, HMAC caveat chain, implicit action alias or unauthenticated bearer suffix.
- Revocation, expiry, holder proof and replay stay explicit Mantle shell prerequisites; a digest or Basalt receipt cannot substitute for live verification.
- Bounded rewrite/reject/alternatives semantics compose newest first without widening a parent's accepted input set.
- The remote, store and project receiver checks occur before ticket redemption, store read or project goal construction, respectively; rejection cannot reveal the rejected path or grant.

## Decision

Use the reviewed UCAN signed resource/ability delegation and holder-proof implementation at its exact Basalt-vendored revision `6f888f6c91a4ea26f0bd52b6486e6643c8f6d271`. UCAN transports opaque Mantle `mantle/filter-v1` caveats; a Mantle `#![no_std]` filter compiles at most eight caveats, eight alternatives per caveat, 256-byte patterns/templates, 8192-byte serialized payloads, 512-byte values and 16 path segments. Fixed-width ordinal keys `00`–`07` identify the preserved chain. Unknown caveat types reject everything, and malformed, duplicated, reordered or gapped ordinal records fail closed. This is Mantle-local semantics, **not** a UCAN guarantee. A parent chain must accept the *original input* independently of every appended rewrite; both the original input and each parent's output remain subject to the site's policy. Rejection is silent at the peer protocol boundary.

The Nickel-authored `config/authority-caveats/basalt-policy.ncl` proposes three distinct **Mantle-specific** generic Basalt contracts: `mantle-remote-build` (`mantle://remote-build/`, `remote/build`), `mantle-store-view` (`mantle://store-view/`, `store/read`), and `mantle-project-goal` (`mantle://project-goal/`, `project/declare`). These source contracts are human-reviewable; authoring or exporting them does not by itself establish policy-owner acceptance or a shell-trusted reference. After review, exact signed resource/ability and site-specific context must both match. The receiver must supply a trusted policy artifact reference, verified UCAN grant/receipt bindings, non-consuming caveat decision and replay disposition to the generic authority core. It must perform genuine signature/proof, key resolution, current time, revocation and replay checks first; an unauthenticated DTO cannot manufacture verified facts. Basalt's *closed* live UCAN action adapter still does not recognize these actions.

Original tickets remain backward-compatible but are **not** cryptographically holder-attenuable. A separate verified signed delegation is required before reporting an attenuated grant. The daemon may issue grants but is not in the receiver admission round trip. Store metadata/content reads must be gated before `OutputLookup`; unrestricted listing under a restricted view must be denied instead of filtered after the read. Project rewrite never substitutes another project's namespace. Any successful grant presentation may report a redacted effective caveat chain, never secret bearer material.

## Rejected alternatives

- Central policy check alone: no transferable holder restriction.
- Unsigned appended ticket conditions or locally invented macaroons: bypassable parent bearer or second unreviewed authority format.
- Mapping remote build onto Basalt's `DocumentWrite`, or store read onto `DocumentRead`: mislabels the reviewed action and makes receipts misleading.
- Treating Basalt's generic fact DTO as cryptographic proof: its inputs explicitly require an authenticating shell.
- Treating UCAN proof preservation as a pattern rewrite or semantic monotonicity proof: UCAN does not interpret the payload.

## Consequences and non-claims

A signed `filter-v1` rewrite/reject is not yet a signed remote
job-set/output-class/deadline record. The standalone `RemoteJobScope` is a
receiver-supplied site predicate, not a transferable grant. Its scope
payload/version, issuer authority, signed producer-identity/job-set binding,
holder-request binding, transport and receiver current-time check require
an agreed UCAN contract with the remote owner. The proposed Basalt policy
likewise has no policy-owner acceptance yet. Neither gate may be replaced
by fabricated verification receipt references or by the bare ticket's
untrusted time field.

The pure filter only decides bounded normalized strings; it does not verify the enclosing credential, issue grants, consult revocation, consume replay, fence concurrent tickets, read store data or execute projects. Basalt's generic policy enforces the exact contract only when trusted, freshly verified facts are correctly supplied. Production authority requires full receiver adapters and positive/negative effect tests; no such integration follows from a green isolated fixture. The aligned exact UCAN/Basalt standalone fixture passed 3/3 signed delegation, holder invocation/replay and store/project effect tests, plus 11/11 core tests; its signed zero-caveat case forbids an undeclared job before any caveat callback. These observations do not prove a production receiver, policy acceptance or durable replay. No new Basalt `EnforcementAction` may be added without its policy owner's explicit review in Basalt's own repository.
