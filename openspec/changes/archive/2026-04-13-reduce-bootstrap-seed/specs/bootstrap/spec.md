## MODIFIED Requirements

### Requirement: Fetch-based bootstrap

The system MUST support `crunch bootstrap --fetch` by resolving a pinned
bootstrap seed provider that satisfies the normalized `bootstrap/seed.ncl`
contract, persisting it as a fixed-output derivation in the crunch store, and
generating `seed.ncl`.

The fetched seed path MUST NOT require Nix to be installed. The provider
metadata MUST record explicit provenance for the trusted external seed
artifacts, and the public stage0 contract consumed by later bootstrap stages
MUST stay provider-independent.

#### Scenario: Bootstrap from fetch on a machine without Nix

- GIVEN a machine with crunch installed but no Nix
- WHEN `crunch bootstrap --fetch --store ~/crunch-store -o seed.ncl` is run
- THEN a `seed.ncl` is generated with a valid store path to the fetched seed
  provider output
- AND `crunch build hello-world.ncl -I seed.ncl --store ~/crunch-store`
  succeeds

#### Scenario: Contributor can inspect fetched seed provenance

- GIVEN a contributor wants to understand the current fetched bootstrap seed
- WHEN they inspect the repo docs or seed metadata
- THEN they can identify the trusted external seed artifacts and their pinned
  hashes
- AND they can see that later bootstrap stages consume a normalized contract
  instead of the provider's raw filesystem layout

### Requirement: Source-built toolchain

The system MUST support building core tools from fetched source tarballs using
one normalized bootstrap seed contract, not one provider-specific filesystem
layout.

These derivations MUST live in the `bootstrap/` directory as regular `.ncl`
files, not as Rust-only special cases. Later bootstrap stages MUST consume the
normalized seed contract exposed through `bootstrap/seed.ncl`.

#### Scenario: Build make from normalized seed contract

- GIVEN a fetched bootstrap seed provider that satisfies the normalized seed
  contract
- WHEN `crunch build bootstrap/make.ncl` is run
- THEN a working `make` binary is produced in the crunch store
- AND it can be used as an input to subsequent derivations

#### Scenario: Build dash from normalized seed contract

- GIVEN the normalized bootstrap seed contract and from-source `make`
- WHEN `crunch build bootstrap/dash.ncl` is run
- THEN a working POSIX shell is produced
- AND the output is usable by later bootstrap stages

### Requirement: From-source compiler toolchain

The system MUST support building a complete C compiler toolchain from source:
`binutils`, `musl`, and `gcc`.

Each tool MUST be represented as a `.ncl` derivation that chains off earlier
bootstrap stages rooted in the normalized bootstrap seed contract.

#### Scenario: Build complete toolchain from normalized seed contract

- GIVEN the bootstrap chain `seed -> make -> dash`
- WHEN `crunch build bootstrap/gcc.ncl` is run
- THEN `gcc`, `binutils`, and `musl` are all built from source
- AND the from-source `gcc` can compile C programs

#### Scenario: Self-test with from-source toolchain

- GIVEN from-source `gcc`, `binutils`, and `musl` with no direct dependency on
  the previous provider-specific seed layout
- WHEN `crunch build bootstrap/selftest.ncl` is run
- THEN a C test program compiles and passes its bootstrap self-test assertions
- AND the binary is statically linked against the from-source `musl`

## ADDED Requirements

### Requirement: Bootstrap seed reduction is explicit and staged

The repo MUST treat replacement of the current fetched bootstrap seed as named,
reviewable work instead of an implied future cleanup.

That staged work MUST define:
- the current trusted seed provider and its provenance,
- the acceptance criteria for a reduced seed provider,
- the migration path for swapping providers behind the normalized seed
  contract,
- the evidence needed to update docs and bootstrap claims once the new seed
  lands.

#### Scenario: Contributor can see the seed-reduction plan

- GIVEN a contributor reads the active bootstrap work or roadmap
- WHEN they look for the next trust-reduction step after the current proof
- THEN they can find explicit work for replacing the current fetched seed
- AND they can see the acceptance criteria for the replacement seed
- AND they can see that later bootstrap stages are expected to remain bound to
  the normalized contract

#### Scenario: Provider swap does not force bootstrap-stage rewrites

- GIVEN a future reduced seed provider is introduced
- WHEN it satisfies the normalized `bootstrap/seed.ncl` contract
- THEN later bootstrap derivations continue to consume that contract
- AND the migration does not require every bootstrap stage to learn raw
  provider-specific layout details
