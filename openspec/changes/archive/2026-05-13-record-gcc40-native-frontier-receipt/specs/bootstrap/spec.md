## MODIFIED Requirements

### Requirement: Bootstrap parity claim gating

Crunch MUST fail closed on any operator-facing claim that Crunch has reached full live-bootstrap, Guix full-source bootstrap, or StageX no-quorum parity unless the parity map and gap report show every required stage complete with the required provider/proof evidence.
ID: bootstrap.parity.claim-gating

A parity claim MUST be scoped to the exact axis satisfied. Live-bootstrap parity MUST require the complete mapped stage ladder or accepted Crunch-specific replacements. Guix full-source parity MUST require source-built inputs, trust-root documentation, and final source proof comparable to Guix's full-source bootstrap claim semantics. StageX parity MUST require the audited hex0 seed lineage, no prebuilt compiler/tool root, protected execution audit when used, and `stagex-lineage` self-build proof.

GCC 4.0 MUST remain evidence-backed partial unless native compiler correctness is proven, and its checked evidence MUST include a native-frontier receipt that names remaining non-native blockers.

#### Scenario: GCC 4.0 native frontier remains partial
- GIVEN `bootstrap/evidence/gcc-4.0-native-boundary.json` has `status=boundary-only`
- AND it contains a non-empty `native_frontier.blockers` array with derivation markers for remaining non-native GCC 4.0 seams
- WHEN `crunch bootstrap parity-report` evaluates `gcc.4.0`
- THEN the row remains `partial`
- AND the evidence check passes only if each frontier marker is present in `bootstrap/gcc-4.0.ncl`
- AND the report does not mark live-bootstrap or Guix parity complete for GCC 4.0
