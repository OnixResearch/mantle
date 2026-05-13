## MODIFIED Requirements

### Requirement: Bootstrap parity claim gating

Crunch MUST fail closed on any operator-facing claim that Crunch has reached full live-bootstrap, Guix full-source bootstrap, or StageX no-quorum parity unless the parity map and gap report show every required stage complete with the required provider/proof evidence.
ID: bootstrap.parity.claim-gating

A parity claim MUST be scoped to the exact axis satisfied. Live-bootstrap parity MUST require the complete mapped stage ladder or accepted Crunch-specific replacements. Guix full-source parity MUST require source-built inputs, trust-root documentation, and final source proof comparable to Guix's full-source bootstrap claim semantics. StageX parity MUST require the audited hex0 seed lineage, no prebuilt compiler/tool root, protected execution audit when used, and `stagex-lineage` self-build proof.

GCC 4.0 MUST remain evidence-backed partial unless native compiler correctness is proven, and its checked evidence MUST include a native-frontier receipt that names remaining non-native blockers. The libiberty demangle frontier MUST use a checked disabled-demangle boundary marker rather than a generic stub marker.

#### Scenario: GCC 4.0 libiberty demangle boundary is explicit
- GIVEN `bootstrap/gcc-4.0.ncl` writes `libiberty/cp-demangle.c`
- WHEN the GCC 4.0 parity checks inspect the derivation and native-frontier receipt
- THEN they require `gcc40_cp_demangle_disabled_boundary`
- AND they reject the legacy `libiberty_cp_demangle_bootstrap_stub` marker
- AND `gcc.4.0` remains `partial` until native compiler correctness is proven
