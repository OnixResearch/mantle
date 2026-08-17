# Design: Repair CA archive final NAR metadata

## Root cause

Mantle's multi-output CA build path intentionally has two identities:

1. a marker-normalized NAR hash used to derive a stable CA store path before final paths exist; and
2. the final NAR hash of the node after every same-length marker is rewritten to its final store path.

`compute_ca_intermediates` calculates the first identity. `finalize_ca_outputs` creates the final node but currently persists the first NAR size/hash as both `PathInfo.nar_*` and `PathInfo.ca`. The CA field is the intended path-identity input; the NAR fields are stale and must instead describe the final node.

Archive import currently passes `PathInfo.ca` to `ingest_nar_and_hash`, incorrectly asking the NAR ingester to prove that the final payload hashes to the marker-normalized identity. The retained Bison fixture proves this is not true for a valid self-rewritten output.

## Functional core

Add pure deterministic helpers for two decisions:

- CA finalization metadata: retain the marker-normalized hash as `CAHash`, while selecting the separately measured final NAR size and SHA-256 for `PathInfo`.
- CA path identity: when CA metadata is present, derive the expected store path from the CA hash and require it to match the signed path. Accept only the repository's explicit supported forms: the ordinary reference-aware form where representable, or the current marker-normalized empty-reference form used by derivation and CA planning. Reject metadata that derives neither path.

These helpers take values and return values/errors only. They do not read the store, render NARs, write archives, or persist PathInfo.

## Imperative shell

### CA output finalization

After `rewrite_markers_to_final`, the shell renders the final node through `SimpleRenderer::calculate_nar`. It passes the measured final size/hash plus the intermediate marker hash to the pure metadata helper, then persists the final node and final NAR facts. Store-path registration and marker rewriting remain unchanged.

### Archive export

Before writing archive magic or headers, export planning verifies each selected closure member:

- signature policy;
- complete castore content;
- supported CA metadata deriving the declared logical store path; and
- freshly rendered final NAR size and SHA-256 matching `PathInfo`.

The later streaming render still computes the Mantle-owned payload BLAKE3. Castore nodes are immutable by digest, so the preflight and stream passes refer to the same content identity. This adds bounded repeated traversal rather than buffering a whole closure in memory or scratch space.

### Archive import

Import validates CA metadata as path identity before payload ingestion. It calls `ingest_nar_and_hash` without an expected CA content hash because marker-normalized CA identity is not necessarily the final NAR hash. Acceptance still requires all of:

- archive payload length and BLAKE3 match;
- actual final NAR SHA-256 and size match the signed `PathInfo`;
- reconstructed node exactly matches the declared node;
- store prefix and CA-derived store path match;
- signature/trust policy accepts the PathInfo unless the explicit unsigned escape hatch is selected.

No state is persisted until these checks pass.

## Security and claim boundaries

A Nix-style PathInfo signature binds logical store path, final NAR SHA-256, final NAR size, and references. It does not bind the CA field directly. Requiring the CA field to derive the signed store path binds that metadata through the signed path identity without pretending it hashes final bytes. Payload BLAKE3 is an archive integrity check; signature policy plus final NAR facts provide trust.

Existing stale signed PathInfo cannot be silently repaired because changing NAR facts invalidates its signature. Export must fail closed with expected/observed diagnostics. A rebuild or future explicit migration must create newly measured and signed facts.

This repair proves archive consistency for the supported Mantle CA path forms. It does not prove compiler correctness, general fixed-point correctness, external nario compatibility, or that marker-normalized CA equals final NAR content.

## Alternatives rejected

- **Overwrite CA with final NAR hash:** changes the already-derived store-path identity and breaks self-reference convergence.
- **Continue validating final payload directly against CA:** rejects valid marker-rewritten outputs, as the retained Bison state demonstrates.
- **Ignore CA entirely:** permits unsigned CA metadata drift because Nix-style signatures do not directly cover the field.
- **Buffer every rendered closure before writing:** avoids a second traversal but creates closure-proportional scratch usage and complicates bounded failure cleanup.
- **Silently repair old PathInfo on export/import:** mutates signed facts without an explicit migration authority and is therefore rejected.

## Validation

- Pure positive test: differing marker/final hashes retain CA identity and select final NAR facts.
- Pure negative tests: CA metadata deriving another store path is rejected; malformed/unsupported path identity is rejected.
- Store positive test: a record whose CA hash differs from final NAR hash exports and imports successfully with exact final node/NAR facts.
- Store negative tests: stale final NAR hash/size fail export before any archive bytes; tampered CA path identity fails before persistence; payload/node/NAR tampering remains rejected.
- Builder tests: final marker rewriting persists independently measured final NAR metadata and keeps the marker CA hash.
- Existing archive CLI, first-party quality, Tiger Style, machine-contract, blocker, Cairn, Tracey, and Nix rails remain green.
- The authenticated full-source fresh-clone fixed-point proof is rerun from committed final source because the builder persistence path is proof-relevant.
