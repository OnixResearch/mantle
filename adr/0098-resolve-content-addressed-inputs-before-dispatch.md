# ADR 0098: Resolve content-addressed inputs before dispatch

## Status

Proposed (2026-10-01). Acceptance requires the change's early-cutoff, trust-negative, clean-client, and compatibility proofs.

## Context

Before this decision, a floating content-addressed output had no final path
when its derivation was registered. A dependent hashed an unresolved input
derivation edge, consulted caches under that identity, and rewrote provisional
path bytes after execution. Changing a CA producer while preserving output
bytes therefore needlessly executed its dependents. Native dynamic-plan units
also rejected references to CA unit outputs because no final path was known
at registration. The prechange baseline is retained with the Cairn change's
evidence.

## Decision Drivers

- Stop dependent execution when an admitted realized CA input has unchanged bytes, even if the producer derivation changes.
- Preserve derivation paths, output paths, and action refs for CA-independent input-addressed subgraphs.
- Do not promote an unsigned CA mapping, provisional plan binding, `Claims.extra`, or PathInfo deriver field into derivation-to-output authority.
- Bind reusable CA realisations to full-key-material trust, with deterministic limits and conflict evidence.

## Decision

After a goal's direct or transitive CA prerequisites complete, but before its first cache lookup or executor dispatch, apply a bounded pure resolution over the derivation and admitted realisation facts in canonical input order. A direct CA parent or a previously CA-resolved input-addressed parent presents a signed output realisation; substitute its concrete output path for the provisional path in argument and environment bytes, move the admitted input-derivation edge to concrete input sources, and recalculate input-addressed output paths and their environment values. Repeat through downstream dependents. Floating CA outputs retain no fixed output path. The canonical resolved ATerm receives a separate domain-tagged BLAKE3 identity (`mantle-resolved-derivation://blake3/`); its calculated derivation path keys CA mappings and PathInfo cache lookup, while its action ref keys shared results. The original derivation path remains the scheduling and user-facing goal key. A derivation in a CA-independent subgraph returns unchanged, without output recalculation.

The existing `mantle-action-result-v1` signed record is the durable realisation record: its action ref binds the resolved ATerm, its per-output entries bind output names and concrete store paths, its producer identity and detached signature bind the signer, and admission separately verifies the referenced PathInfo and content. Local publication and configured local/base/HTTP discovery already share that record format. A mapping alone never authorizes resolution. Verify records and PathInfo under the actual selected store's trust policy, matching the full verifying key rather than only its display name. A newly built CA output or CA-resolved input-addressed intermediate must have its signed record published before it is offered as an input. Revocation and backend read checks remain at the store boundary.

At most 256 CA-affected input edges, 1,024 output substitutions, 4,096 bytes per path, and 1 MiB per resolved argument or environment value are admitted; an exceeded bound fails `ca-resolution-limit`. Missing facts fail `ca-input-unrealized`; malformed or wrong-domain identity and unsigned/untrusted records fail `ca-realisation-untrusted`. Distinct trusted output paths for the same resolved identity and output fail `ca-realisation-conflict`, block reuse, and record both candidates as nondeterminism evidence. An untrusted or conflicting candidate cannot silently fall back to its unsigned CA mapping or PathInfo-only cache hit. An accepted dynamic-plan placeholder for a CA unit output may carry only its provisional identity at registration; resolution binds the realized path before lookup/dispatch.

Post-build CA *input* rewriting is removed; CA *self-reference* relocation remains a separate output-finalization responsibility. Legacy mapping-only results are not silently promoted to signed realisations: if no authentic record binds their derivation and output they must rebuild before serving a dependent.

## Alternatives Considered

- Keep unresolved derivation keys and post-build substitution: rejected, because identical CA output bytes do not stop dependent execution.
- Trust a signed PathInfo alone or a plain `ca_mappings.json` entry: rejected, because the PathInfo fingerprint does not authenticate its mutable deriver field or bind an output to a specific resolved action.
- Defer every dependent path until the entire graph evaluates: rejected, because it changes identities of derivations with no CA inputs.
- Introduce a second, weaker realisation index: rejected in favor of the existing signed action-result record and its strong reuse admission.

## Consequences

CA-dependent cache and publication identities change once, and old mapping-only CA results cannot authorize dependent resolution. Reports retain both original and resolved identities and the reuse decision. Retention follows action-result output paths: metadata is not a GC root. Recorded successful reuse proves only bounded authenticated reuse under the observed trust policy, not output determinism, executor/compiler correctness, frontend semantics, or release eligibility.
