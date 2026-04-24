# Output Selection Specification

## Purpose

Defines how consuming derivations reference specific outputs of
multi-output dependencies.

## Requirements

### Requirement: Selective Output Dependency

A derivation MUST be able to depend on a single named output of a
multi-output dependency, so that only that output is mounted in the
build sandbox.

#### Scenario: Select dev output

- GIVEN a multi-output derivation `libfoo` with outputs `["out", "dev", "lib"]`
- WHEN a consuming derivation includes `crunch.select libfoo "dev"` in its inputs
- THEN `input_derivations` maps `libfoo`'s drv path to `{"dev"}`
- AND only `libfoo`'s `dev` output path is mounted in the sandbox

#### Scenario: Bare derivation selects all outputs

- GIVEN a multi-output derivation `libfoo` with outputs `["out", "dev"]`
- WHEN a consuming derivation includes `libfoo` directly in its inputs
- THEN `input_derivations` maps `libfoo`'s drv path to `{"out", "dev"}`
- AND both output paths are mounted in the sandbox

### Requirement: Invalid Output Rejection

The glue layer MUST reject output selections that name a nonexistent
output at convert time.

#### Scenario: Nonexistent output name

- GIVEN a derivation `libfoo` with outputs `["out", "dev"]`
- WHEN a consuming derivation selects output `"headers"` from `libfoo`
- THEN conversion fails with an error naming the invalid output and listing the available outputs

### Requirement: Coalescing Duplicate Selections

The system MUST coalesce duplicate selected-output dependencies into a
single `input_derivations` entry with the union of selected outputs when
the same dependency appears multiple times in `inputs`.

#### Scenario: Two selections from same dep

- GIVEN a derivation `libfoo` with outputs `["out", "dev", "lib"]`
- WHEN a consumer's inputs include both `crunch.select libfoo "dev"` and `crunch.select libfoo "lib"`
- THEN `input_derivations` maps `libfoo` to `{"dev", "lib"}`

### Requirement: Select Helper Function

The stdlib MUST provide a function `crunch.select` that takes a
derivation record and an output name string, returning a structured
record suitable for use in the `inputs` array.

#### Scenario: Helper produces correct structure

- GIVEN `let pkg = { name = "foo", builder = "/bin/sh", outputs = ["out", "dev"], ... } | crunch.Derivation`
- WHEN evaluating `crunch.select pkg "dev"`
- THEN the result is `{ drv = pkg, output = "dev" }`

### Requirement: Input Contract Extension

The Nickel `Input` contract MUST accept three forms:
1. A string (source store path)
2. A derivation record (all outputs)
3. A record with `drv` and `output` fields (selected output)

#### Scenario: Mixed inputs array

- GIVEN inputs `[seed.bash, dep_pkg, crunch.select dep_pkg "dev"]`
- WHEN the array is validated against `Array Input`
- THEN all three entries pass validation
