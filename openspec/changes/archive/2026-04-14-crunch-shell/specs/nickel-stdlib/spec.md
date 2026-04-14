## MODIFIED Requirements

### Requirement: Builder package location

A builder package MUST exist at `builders/lib.ncl` that provides
mkDerivation, mkStdenv, mkShell, callPackage, and MkDerivationArgs.
This package is separate from the core stdlib.

`builders/mk_derivation.ncl` imports `derivation.ncl` and
`contracts.ncl` from the stdlib via the Nickel import path (not
relative imports, to avoid circular imports with `builders/lib.ncl`).

`mkShell` MUST build a derivation that materializes shell activation
metadata as `$out/.crunch-shell.json`.

That sidecar MUST:
- carry a required `version` field set to `1`
- carry `env` entries derived from the shell author's declared shell env vars
- carry `path_entries` derived from the shell author's declared inputs
- carry an optional `hook` string, represented as null/absent when not set

The sidecar format exists so shell activation metadata is machine-readable and
independent from the core derivation contract.

#### Scenario: Builder package exists

- GIVEN the crunch source tree
- WHEN `builders/lib.ncl` is inspected
- THEN it contains mkStdenv, mkDerivation, mkShell, callPackage

#### Scenario: mkShell writes a sidecar with versioned metadata

- GIVEN a Nickel expression using `builders.mkShell` with env vars and
  build inputs
- WHEN that shell derivation is evaluated and built
- THEN its output contains `.crunch-shell.json`
- AND the sidecar JSON contains `version = 1`
- AND the sidecar `env` contains the declared shell env vars
- AND the sidecar `path_entries` contains the derived shell PATH entries

#### Scenario: mkShell without hook records null or absent hook

- GIVEN a Nickel expression using `builders.mkShell` with no hook field
- WHEN the shell derivation is built
- THEN `.crunch-shell.json` exists
- AND its `hook` field is null or absent

#### Scenario: Bootstrap uses raw Derivation contract

- GIVEN `bootstrap/hello.ncl`
- WHEN it needs to build something
- THEN it uses `crunch.Derivation` directly (bootstrap files do not
  use mkDerivation)
