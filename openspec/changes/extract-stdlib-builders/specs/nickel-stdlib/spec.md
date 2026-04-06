# Nickel Stdlib Specification — Delta

## MODIFIED Requirements

### Requirement: Scope boundary (modified)

The stdlib MUST contain only:

1. **Contracts** — Derivation, FixedOutput, Input, OutputRef
2. **Enums** — System, HashAlgo, HashMode, Sandbox
3. **Validators** — StorePath, Name
4. **Conversion helpers** — enum→string via match
5. **Fetchers** — fetchurl, fetchTarball, fetchGit (these produce
   Derivation records, not builder logic)
6. **Output selection** — the `select` function

The stdlib MUST NOT contain:

- `mkDerivation` or any phase-based builder
- `mkStdenv` or any stdenv-like wrapper
- `mkShell` or any dev environment helper
- `callPackage` or any dependency injection helper
- `MkDerivationArgs` or any builder-specific contract
- `overrideAttrs` or any override mechanism
- Default phase implementations (unpackPhase, configurePhase, etc.)

These MUST live in a separate Nickel package (e.g., `builders/` or
`packages/stdenv/`) that imports the crunch stdlib.

#### Scenario: Core stdlib has no builder logic

- GIVEN the crunch stdlib files in `lib/`
- WHEN inspected
- THEN no file contains `mkDerivation`, `mkStdenv`, `mkShell`,
  phase names, or shell script templates

#### Scenario: Builder package imports stdlib

- GIVEN the builder package in `builders/`
- WHEN it defines `mkDerivation`
- THEN it does `let crunch = import "lib.ncl" in` and applies
  `crunch.Derivation` to its output records

### Requirement: Derivation contract closure (modified)

The Derivation contract MUST be a closed record. Extra fields MUST
be rejected. The current open contract (`..`) was added to support
mkDerivation's pname/version/meta/passthru fields — those fields
belong in the builder contract, not in the core Derivation contract.

The Derivation contract MUST allow exactly these fields:

- `name`, `builder`, `system`, `args`, `outputs`, `env`, `inputs`
- `fixed_output` (optional)
- `addressing_mode`
- `sandbox`

#### Scenario: Extra field rejected

- GIVEN `{ name = "foo", builder = "/bin/sh", bogus = true } | crunch.Derivation`
- WHEN evaluated
- THEN Nickel rejects the extra field

#### Scenario: Builder package uses its own contract

- GIVEN `{ pname = "foo", version = "1.0", ... } | builders.MkDerivationArgs`
- WHEN evaluated by the builder package
- THEN pname and version are accepted by the builder contract
- AND the builder produces a record that satisfies the closed
  `crunch.Derivation` contract

### Requirement: lib.ncl entry point (modified)

`lib/lib.ncl` MUST re-export:

```nickel
{
  Derivation, FixedOutput,
  StorePath, Name,
  System, HashAlgo, HashMode, Sandbox, Input,
  system_to_string, hash_algo_to_string, hash_mode_to_string,
  fetchurl, fetchTarball, fetchGit,
  Url, Hash,
  select,
}
```

It MUST NOT re-export mkStdenv, mkShell, mkDerivation, callPackage,
or MkDerivationArgs.

## ADDED Requirements

### Requirement: Builder package location

A builder package MUST exist at `builders/lib.ncl` (or similar) that
provides mkDerivation, mkStdenv, mkShell, callPackage, and
MkDerivationArgs. This package is separate from the core stdlib.

#### Scenario: Builder package exists

- GIVEN the crunch source tree
- WHEN `builders/lib.ncl` is inspected
- THEN it contains mkStdenv, mkDerivation, mkShell, callPackage

#### Scenario: Bootstrap uses builder package

- GIVEN `bootstrap/hello.ncl`
- WHEN it needs mkDerivation
- THEN it imports `builders/lib.ncl`, not `lib/lib.ncl`
