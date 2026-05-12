## ADDED Requirements

### Requirement: GCC 4.0 Ucmpdi2 Libgcc Semantics [r[gcc40-libgcc-ucmpdi2-semantics]]
Crunch MUST be able to promote `_ucmpdi2` from a placeholder body to verified unsigned 64-bit comparison semantics without broadening the GCC 4.0 pass1 bridge into a full native GCC rewrite.

#### Scenario: Ucmpdi2 has non-placeholder semantics [r[gcc40-libgcc-ucmpdi2-semantics.1]]
- GIVEN the GCC 4.0 bootstrap artifact is built
- WHEN `_ucmpdi2.o` is extracted from the produced `libgcc.a`
- THEN the symbol table exposes `_ucmpdi2`
- AND a semantic smoke linked against `libgcc.a` proves the GCC libgcc unsigned comparison contract: 0 for less-than, 1 for equality, and 2 for greater-than across representative low, high-bit, and maximum 64-bit values

#### Scenario: Archive shape remains deterministic [r[gcc40-libgcc-ucmpdi2-semantics.2]]
- GIVEN `_ucmpdi2` is promoted
- WHEN the produced `libgcc.a` is inspected
- THEN existing promoted `_negdi2`, `_muldi3`, `_lshrdi3`, `_ashldi3`, `_ashrdi3`, and `_cmpdi2` remain present
- AND the archive continues to use the deterministic hand-written ar(5) member list
