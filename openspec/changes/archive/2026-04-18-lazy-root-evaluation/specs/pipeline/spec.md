## MODIFIED Requirements

### Requirement: Pipeline owns eval and convert

The pipeline MUST evaluate Nickel in-process through `crunch-eval`, but it
MUST NOT require a whole-program export-ready deep evaluation merely to obtain
root labels.

The build path MUST:
- discover root labels through the lazy `crunch-eval` boundary first,
- force derivation values per root only when needed for conversion, and
- continue to use direct typed derivation extraction rather than a whole-program
  JSON export string for build execution.

`evaluate_to_json()` remains available for `crunch eval` and other debug or
reporting paths, but the build path MUST use the lazy discovery + selected-root
forcing representation that `crunch-eval` exposes.

The pipeline MUST still run `crunch_glue::convert()` for each derivation and
bridge the `ConversionCache` to a `DerivationRegistry`.

#### Scenario: Package-set build discovers labels before per-root forcing

- GIVEN a `.ncl` file exporting a record of derivations
- WHEN `build()` is called
- THEN the pipeline discovers the root labels before it begins per-root
  derivation forcing and conversion
- AND it does not need a whole-program export-ready result up front just to
  learn those labels

#### Scenario: Full-root build still works through per-root forcing

- GIVEN a `.ncl` file exporting multiple root derivations
- WHEN `build()` is called and all roots are selected for build
- THEN the pipeline may still force all roots eventually
- AND it does so through per-root forcing rather than a required whole-program
  export step before selection
