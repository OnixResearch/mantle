## Why

We want a build system that uses Nickel as the configuration/evaluation language
and the Nix store protocol for content-addressed storage and sandboxed builds.
No Nix language code in the evaluation path. The project is called **crunch**.

Today, building software with the Nix store requires writing Nix. snix
reimplements the Nix evaluator in Rust but is still bound to the Nix language.
Nickel is a better configuration language (gradual typing, contracts, proper
merge semantics) but has no build system — organist bridged the two but still
shells out to Nix for the actual `derivation` call.

crunch removes Nix from the evaluation loop entirely. Nickel evaluates to
records describing derivations. Rust glue converts those records into
`nix-compat::Derivation` structs. snix-build executes them in a sandbox.
snix-store persists results in a content-addressed store. The Nix language is
not involved at any point.

## What Changes

- **Nickel evaluator integration**: Use `nickel-lang-core` as a library to
  evaluate `.ncl` files to fully-reduced records.

- **Derivation glue**: Rust code that reads evaluated Nickel records (via
  serde_json or direct NickelValue access) and constructs
  `nix_compat::Derivation` structs with correct store path hashing.

- **Vendored snix crates**: Vendor `nix-compat`, `snix-build`, `snix-castore`,
  `snix-store`, and supporting crates into `crunch/`. Modify as needed — these
  are dependencies, not upstream we track.

- **Nickel contracts/stdlib**: A minimal set of `.ncl` files defining only the
  `Derivation` contract, enum types, validators, and conversion helpers.
  No builder templates, no build phases, no stdenv equivalent — those belong
  in separate packages layered on top.

- **CLI**: `crunch build <file.ncl>` — evaluates, constructs derivations,
  builds, stores results.

- **Bootstrap**: Use static binaries (musl-based gcc, coreutils, bash) or
  a Nix-built seed tarball as stage 0. Just enough to compile C programs.
  The goal is to build crunch with crunch as soon as possible.

## Capabilities

### New Capabilities

- `nickel-eval`: Evaluate `.ncl` files to derivation records using
  nickel-lang-core.
- `derivation-glue`: Convert Nickel records → `nix_compat::Derivation` with
  correct ATerm serialization and store path calculation.
- `sandboxed-build`: Execute derivations in a sandbox via snix-build
  (bwrap on Linux).
- `store-persist`: Persist build outputs in a content-addressed Nix store
  via snix-store/snix-castore.
- `nickel-stdlib`: Derivation contracts and builder helpers in Nickel.
- `cli`: `crunch build` command that drives the full pipeline.
- `bootstrap`: Seed toolchain for building from scratch.

## Impact

- **Files**: Everything under `crunch/`. New Rust crates, Nickel stdlib files,
  vendored snix code.
- **APIs**: No external API yet. Internal crate boundaries between eval, glue,
  build, store.
- **Dependencies**: `nickel-lang-core`, vendored `nix-compat`/`snix-*` crates,
  existing deps from crunch scaffold (tokio, serde, clap, etc.).
- **Testing**: Unit tests for derivation construction and store path hashing.
  Integration test that builds a trivial derivation end-to-end.
