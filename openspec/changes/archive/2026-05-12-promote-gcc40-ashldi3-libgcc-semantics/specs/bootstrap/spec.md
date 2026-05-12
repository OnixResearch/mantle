## ADDED Requirements

### Requirement: GCC 4.0 Ashldi3 Libgcc Semantics [r[gcc40-libgcc-ashldi3-semantics]]
Crunch MUST be able to promote `_ashldi3` from a placeholder body to verified 64-bit left-shift semantics without broadening the GCC 4.0 pass1 bridge into a full native GCC rewrite.

#### Scenario: Ashldi3 has non-placeholder semantics [r[gcc40-libgcc-ashldi3-semantics.1]]
- GIVEN the GCC 4.0 bootstrap artifact is built
- WHEN `_ashldi3.o` is extracted from the produced `libgcc.a`
- THEN the symbol table exposes `_ashldi3`
- AND a semantic smoke linked against `libgcc.a` proves representative 64-bit left shifts, including zero shifts, low-bit movement, and high-bit-producing counts below 64

#### Scenario: Archive shape remains deterministic [r[gcc40-libgcc-ashldi3-semantics.2]]
- GIVEN `_ashldi3` is promoted
- WHEN the produced `libgcc.a` is inspected
- THEN existing promoted `_negdi2`, `_muldi3`, and `_lshrdi3` remain present
- AND the archive continues to use the deterministic hand-written ar(5) member list
