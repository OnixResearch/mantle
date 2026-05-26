# Rust package planning spec delta

## ADDED Requirements

### r[rust_package_planning.native_rlib_crate_type_derivation_planning]

Mantle MUST treat Cargo target-kind arrays containing `rlib` as library-compatible for native Rust unit derivation planning.

#### Scenario: rlib target kind is planned as a library

- GIVEN a Cargo unit target kind array contains `rlib`
- WHEN unit derivation planning classifies the unit
- THEN it MUST select `target_kind = "lib"`
- AND it MUST NOT emit `unsupported-target-kind` solely because the kind array lacks the broader `lib` marker.

#### Scenario: mixed cdylib and rlib target keeps bounded claim

- GIVEN a Cargo unit target kind array contains both `cdylib` and `rlib`
- WHEN Mantle plans reviewable Rust unit derivations
- THEN it MUST plan the unit through the rlib/library facet
- AND it MUST NOT claim cdylib output production.

#### Scenario: non-rlib unsupported target remains fail-closed

- GIVEN a target kind array contains no supported `lib`, `rlib`, `bin`, `custom-build`, `proc-macro`, or bounded test marker
- WHEN unit derivation planning classifies the unit
- THEN it MUST emit a deterministic unsupported-target blocker before execution.
