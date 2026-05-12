## ADDED Requirements

### Requirement: GCC 4.0 Cmpdi2 Libgcc Semantics [r[gcc40-libgcc-cmpdi2-semantics]]
Crunch MUST be able to promote `_cmpdi2` from a placeholder body to verified signed 64-bit comparison semantics without broadening the GCC 4.0 pass1 bridge into a full native GCC rewrite.

#### Scenario: Cmpdi2 has non-placeholder semantics [r[gcc40-libgcc-cmpdi2-semantics.1]]
- GIVEN the GCC 4.0 bootstrap artifact is built
- WHEN `_cmpdi2.o` is extracted from the produced `libgcc.a`
- THEN the symbol table exposes `_cmpdi2`
- AND a semantic smoke linked against `libgcc.a` proves the GCC libgcc signed comparison contract: 0 for less-than, 1 for equality, and 2 for greater-than across representative negative, zero, and positive 64-bit values

#### Scenario: Archive shape remains deterministic [r[gcc40-libgcc-cmpdi2-semantics.2]]
- GIVEN `_cmpdi2` is promoted
- WHEN the produced `libgcc.a` is inspected
- THEN existing promoted `_negdi2`, `_muldi3`, `_lshrdi3`, `_ashldi3`, and `_ashrdi3` remain present
- AND the archive continues to use the deterministic hand-written ar(5) member list
