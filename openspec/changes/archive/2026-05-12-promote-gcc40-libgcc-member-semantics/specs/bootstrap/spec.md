## ADDED Requirements

### Requirement: GCC 4.0 Libgcc Member Semantics [r[gcc40-libgcc-semantic-member]]
Crunch MUST be able to promote individual GCC 4.0 `libgcc.a` members from placeholder bodies to verified semantics without requiring a full native GCC rewrite in the same change.

#### Scenario: One member has non-placeholder semantics [r[gcc40-libgcc-semantic-member.1]]
- GIVEN the GCC 4.0 bootstrap artifact is built
- WHEN the selected `libgcc.a` member is extracted and smoke-tested
- THEN its behavior matches the documented semantics and is not merely `return 0`

#### Scenario: Archive shape remains valid [r[gcc40-libgcc-semantic-member.2]]
- GIVEN the member is promoted
- WHEN host `ar` and `nm` inspect `libgcc.a`
- THEN expected member names and symbols remain visible
