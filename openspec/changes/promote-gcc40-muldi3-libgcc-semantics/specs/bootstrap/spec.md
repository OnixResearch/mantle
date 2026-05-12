## ADDED Requirements

### Requirement: GCC 4.0 Muldi3 Libgcc Semantics [r[gcc40-libgcc-muldi3-semantics]]
Crunch MUST be able to promote `_muldi3` from a placeholder body to verified signed 64-bit multiplication semantics without broadening the GCC 4.0 pass1 bridge into a full native GCC rewrite.

#### Scenario: Muldi3 has non-placeholder semantics [r[gcc40-libgcc-muldi3-semantics.1]]
- GIVEN the GCC 4.0 bootstrap artifact is built
- WHEN `_muldi3.o` is extracted from the produced `libgcc.a`
- THEN the symbol table exposes `_muldi3`
- AND a semantic smoke linked against `libgcc.a` proves representative positive, negative, and zero 64-bit products

#### Scenario: Archive shape remains deterministic [r[gcc40-libgcc-muldi3-semantics.2]]
- GIVEN `_muldi3` is promoted
- WHEN the produced `libgcc.a` is inspected
- THEN existing promoted `_negdi2` remains present
- AND the archive continues to use the deterministic hand-written ar(5) member list
