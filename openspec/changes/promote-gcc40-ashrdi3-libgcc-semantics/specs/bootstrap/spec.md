## ADDED Requirements

### Requirement: GCC 4.0 Ashrdi3 Libgcc Semantics [r[gcc40-libgcc-ashrdi3-semantics]]
Crunch MUST be able to promote `_ashrdi3` from a placeholder body to verified signed 64-bit arithmetic-right-shift semantics without broadening the GCC 4.0 pass1 bridge into a full native GCC rewrite.

#### Scenario: Ashrdi3 has non-placeholder semantics [r[gcc40-libgcc-ashrdi3-semantics.1]]
- GIVEN the GCC 4.0 bootstrap artifact is built
- WHEN `_ashrdi3.o` is extracted from the produced `libgcc.a`
- THEN the symbol table exposes `_ashrdi3`
- AND a semantic smoke linked against `libgcc.a` proves representative signed 64-bit arithmetic right shifts, including negative sign extension, positive values, zero shifts, and wider shift counts below 64

#### Scenario: Archive shape remains deterministic [r[gcc40-libgcc-ashrdi3-semantics.2]]
- GIVEN `_ashrdi3` is promoted
- WHEN the produced `libgcc.a` is inspected
- THEN existing promoted `_negdi2`, `_muldi3`, `_lshrdi3`, and `_ashldi3` remain present
- AND the archive continues to use the deterministic hand-written ar(5) member list
