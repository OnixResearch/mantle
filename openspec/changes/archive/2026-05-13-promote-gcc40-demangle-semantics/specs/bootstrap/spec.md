## MODIFIED Requirements

### Requirement: Bootstrap parity claim gating

Crunch MUST fail closed on any operator-facing claim that Crunch has reached full live-bootstrap, Guix full-source bootstrap, or StageX no-quorum parity unless the parity map and gap report show every required stage complete with the required provider/proof evidence.
ID: bootstrap.parity.claim-gating

GCC 4.0 MUST remain evidence-backed partial unless native compiler correctness is proven, and its checked evidence MUST include a native-frontier receipt that names remaining non-native blockers. The libiberty demangle frontier MUST expose a checked bounded-demangle semantic marker for the supported Itanium zero-argument function slice rather than a disabled-demangle marker.

#### Scenario: GCC 4.0 libiberty demangle has bounded semantics
- GIVEN `bootstrap/gcc-4.0.ncl` writes the libiberty demangle bridge
- WHEN the derivation-local smoke calls `cplus_demangle` and `cplus_demangle_v3`
- THEN `_Z3foov` demangles to `foo()`
- AND malformed or unsupported names return null
- AND parity checks require `gcc40_cplus_demangle_bounded_itanium_v0_boundary`
- AND parity checks reject `gcc40_cp_demangle_disabled_boundary`
- AND `gcc.4.0` remains `partial` until native compiler correctness is proven
