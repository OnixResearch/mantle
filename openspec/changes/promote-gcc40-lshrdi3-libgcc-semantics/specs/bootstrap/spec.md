## ADDED Requirements

### Requirement: GCC 4.0 Lshrdi3 Libgcc Semantics [r[gcc40-libgcc-lshrdi3-semantics]]
Crunch MUST be able to promote `_lshrdi3` from a placeholder body to verified unsigned 64-bit logical-right-shift semantics without broadening the GCC 4.0 pass1 bridge into a full native GCC rewrite.

#### Scenario: Lshrdi3 has non-placeholder semantics [r[gcc40-libgcc-lshrdi3-semantics.1]]
- GIVEN the GCC 4.0 bootstrap artifact is built
- WHEN `_lshrdi3.o` is extracted from the produced `libgcc.a`
- THEN the symbol table exposes `_lshrdi3`
- AND a semantic smoke linked against `libgcc.a` proves representative unsigned 64-bit logical right shifts, including high-bit values, zero shifts, and wider shift counts below 64

#### Scenario: Archive shape remains deterministic [r[gcc40-libgcc-lshrdi3-semantics.2]]
- GIVEN `_lshrdi3` is promoted
- WHEN the produced `libgcc.a` is inspected
- THEN existing promoted `_negdi2` and `_muldi3` remain present
- AND the archive continues to use the deterministic hand-written ar(5) member list
