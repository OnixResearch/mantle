# CLI Specification

## Purpose

Defines the crunch command-line interface.

## Requirements

### Requirement: Build command

The CLI MUST provide a `crunch build <file.ncl>` command that:

1. Evaluates the Nickel file
2. Constructs derivation(s)
3. Builds them in a sandbox
4. Persists outputs to the store
5. Prints the output store path(s) to stdout

#### Scenario: Build a single derivation

- GIVEN `hello.ncl` describing a valid derivation
- WHEN `crunch build hello.ncl` is run
- THEN the build executes and the output path is printed

#### Scenario: Build failure

- GIVEN a derivation whose build script exits non-zero
- WHEN `crunch build` is run
- THEN the error is reported with the build log and crunch exits non-zero

### Requirement: Eval command

The CLI SHOULD provide `crunch eval <file.ncl>` that evaluates and prints
the derivation JSON without building. Useful for debugging.

#### Scenario: Eval only

- GIVEN `hello.ncl`
- WHEN `crunch eval hello.ncl` is run
- THEN the evaluated derivation record is printed as JSON

### Requirement: Store configuration

The CLI MUST support configuring the store location. Default SHOULD be
`/nix/store` for compatibility, but a `--store` flag MAY allow an
alternative path.

### Requirement: Verbosity

The CLI MUST support `-v` / `--verbose` for debug logging and SHOULD
support `--log-level` for fine-grained control via tracing.

### Requirement: Exit codes

- `0` — success
- `1` — build failure
- `2` — evaluation error (Nickel parse/type/contract error)
- `3` — internal error
