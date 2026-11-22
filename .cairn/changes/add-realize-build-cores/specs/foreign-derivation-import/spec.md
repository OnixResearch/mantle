## ADDED Requirements

### Requirement: Realize selects and records the build core count

r[foreign_derivation_import.realize_build_cores] `foreign-import realize` MUST accept
`--build-cores <N>` with an integer `N` in the inclusive range `1..=64` and a default of `1`. For
`N` above the default it MUST write `N` as `NIX_BUILD_CORES` into the environment of every build the
realization runs, unless the derivation declares its own value. At the default every build MUST see
exactly the environment Mantle gave it before the option existed: sandbox profiles export `1` and
Nix-protocol profiles leave `NIX_BUILD_CORES` unset. The value MUST NOT participate in derivation or
action identity.

A value other than the default MUST be accepted only when the plan carries the route
`cache-inputs-build-roots-v2`, and MUST otherwise be refused with a typed error before any store
mutation, because receipt v1 has no field to record it.

Receipt v2 MUST record a value above the default for every `built` root as
`roots[].built.build_cores` and MUST omit the field at the default. The field MUST be optional, so a
receipt written before the field existed still parses and still recomputes to its recorded
`receipt_blake3`, and a default run's receipt is unchanged. Reuse classification MUST ignore the
value: a root built with any core count is reusable.

#### Scenario: Explicit core count

- GIVEN a plan on `cache-inputs-build-roots-v2` whose selected root is absent locally
- WHEN `realize --build-cores 4` runs
- THEN the root's builder observes `NIX_BUILD_CORES=4`
- AND the published receipt records `roots[].built.build_cores = 4`

#### Scenario: Default core count

- GIVEN a plan on `cache-inputs-build-roots-v2` whose selected root is absent locally
- WHEN `realize` runs without `--build-cores`
- THEN a Nix-protocol builder observes no `NIX_BUILD_CORES`, as before the option existed
- AND the published receipt has no `roots[].built.build_cores`

#### Scenario: Other routes refuse a non-default value

- GIVEN a plan on `cache-inputs-build-roots-v1` or any other route than `cache-inputs-build-roots-v2`
- WHEN `realize --build-cores 4` runs
- THEN the command fails before opening the store for mutation
- AND no receipt or store change is produced

#### Scenario: Out-of-range value

- GIVEN any plan
- WHEN `realize --build-cores 0` or `--build-cores 65` is parsed
- THEN the command is refused and no receipt is produced

#### Scenario: Reuse ignores the recorded core count

- GIVEN the built-root index holds a receipt whose `built` object has no `build_cores`
- WHEN realization finds that root present with an intact NAR and a valid builder signature
- THEN the root still classifies as reusable
