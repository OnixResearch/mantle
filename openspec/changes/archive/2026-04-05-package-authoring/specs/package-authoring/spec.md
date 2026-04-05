# Package Authoring Specification

## Purpose

Defines the higher-level package builder API in Nickel: `mkDerivation`,
the phase system, input propagation, and `mkShell`. These sit above the
raw `Derivation` contract and make real packages writable without
boilerplate.

## Requirements

### Requirement: mkDerivation function

The Nickel stdlib MUST provide a `crunch.mkDerivation` function that
accepts a record with at least `name` and produces a valid `Derivation`.

The function MUST:

- Accept `src` (store path or derivation) as the source input
- Accept `build_inputs` as an array of store paths or derivation records
- Generate a builder script that runs phases in order
- Pass the `Derivation` contract validation

The function SHOULD:

- Set `$PATH` from `build_inputs` automatically
- Set `$src` in the build environment
- Set `$prefix` equal to `$out`
- Set `$NIX_BUILD_CORES` to the number of available CPUs

#### Scenario: Minimal mkDerivation

- GIVEN `crunch.mkDerivation { name = "foo", src = "/nix/store/...-src" }`
- WHEN evaluated
- THEN the result satisfies `crunch.Derivation`
- AND `builder` points to a bash executable
- AND `args` contains a build script with phase calls
- AND `inputs` includes the `src` path

#### Scenario: build_inputs wire PATH

- GIVEN `build_inputs = ["/nix/store/...-gcc", "/nix/store/...-coreutils"]`
- WHEN the derivation is built
- THEN `$PATH` in the sandbox includes `<gcc>/bin` and `<coreutils>/bin`

#### Scenario: Override a single phase

- GIVEN `crunch.mkDerivation { name = "foo", configurePhase = "cmake ." }`
- WHEN the derivation is built
- THEN the configure phase runs `cmake .` instead of the default

#### Scenario: Skip a phase

- GIVEN `crunch.mkDerivation { name = "foo", configurePhase = "" }`
- WHEN the derivation is built
- THEN the configure phase is skipped entirely

### Requirement: Phase system

The builder script MUST execute phases in this order:

1. `unpackPhase` — extract or copy source into the build directory
2. `configurePhase` — run configure/cmake/meson
3. `buildPhase` — compile
4. `installPhase` — install to `$out`

Each phase MUST have a default implementation:

| Phase | Default |
|-------|---------|
| `unpackPhase` | If `$src` is a directory, `cp -r`. If a file, `tar xf` and enter single subdirectory. |
| `configurePhase` | `./configure --prefix=$out` if `./configure` exists, else no-op. |
| `buildPhase` | `make -j$NIX_BUILD_CORES` |
| `installPhase` | `make install` |

Each phase MUST be overridable by setting the corresponding field in
the mkDerivation record to a string (the shell commands to run).

Setting a phase to the empty string `""` MUST skip that phase.

#### Scenario: Default unpack handles tarball output

- GIVEN `src` is a store path containing a directory tree
- WHEN `unpackPhase` runs
- THEN the directory contents are copied into the build working directory

#### Scenario: Default configure detects autotools

- GIVEN the source tree contains a `./configure` script
- WHEN `configurePhase` runs
- THEN `./configure --prefix=$out` is executed

#### Scenario: No configure script present

- GIVEN the source tree does not contain `./configure`
- WHEN `configurePhase` runs
- THEN the phase completes without error (no-op)

### Requirement: mkShell function

The stdlib MUST provide `crunch.mkShell` for development environments.

`mkShell` MUST:

- Accept `build_inputs` (array of store paths or derivation records)
- Produce a `Derivation` record with environment variables set for
  interactive use (`$PATH`, etc.)
- Fail if actually built (it exists only for environment extraction)

`mkShell` MAY defer interactive activation to a future `crunch shell`
command. For v1, the derivation record is the output.

#### Scenario: mkShell produces valid derivation

- GIVEN `crunch.mkShell { build_inputs = [seed.gcc, seed.coreutils] }`
- WHEN evaluated
- THEN the result satisfies `crunch.Derivation`
- AND `inputs` includes both gcc and coreutils store paths

### Requirement: Input handling

`mkDerivation` MUST handle two kinds of inputs in `build_inputs`:

1. **Store path strings** — added to `inputs` as source inputs and to
   `$PATH` as `<path>/bin`
2. **Derivation records** — added to `inputs` as derivation inputs and
   to `$PATH` as `<output>/bin` (resolved at build time)

For v1, automatic `$PATH` wiring MAY be limited to store path strings.
Derivation-record build_inputs MUST still be added to `inputs` for
sandbox mounting.

### Requirement: src field

The `src` field in mkDerivation MUST accept:

- A store path string (output of fetchTarball, fetchGit, etc.)
- A derivation record (built first, output used as source)
- Being absent (no unpack phase; the build script handles everything)

When `src` is set, the unpack phase MUST make the source available in
the build working directory.

#### Scenario: fetchTarball as src

- GIVEN `src = crunch.fetchTarball { url = "...", hash = "..." }`
- WHEN the derivation is built
- THEN the tarball is fetched, unpacked, and available in the working directory

### Requirement: Builder script requires bash

`mkDerivation` MUST require a `bash` parameter (a store path to bash)
or locate bash from `build_inputs`.

The function MUST NOT hardcode `/bin/sh` or `/bin/bash` — the sandbox
has no host filesystem access.

#### Scenario: bash from seed

- GIVEN `crunch.mkDerivation { name = "foo", bash = seed.bash, build_inputs = [seed.gcc] }`
- WHEN evaluated
- THEN `builder` is `<seed.bash>/bin/bash`

#### Scenario: bash inferred from build_inputs

- GIVEN `build_inputs` includes a store path ending in `-bash` or `-bash-5.x`
- WHEN evaluated
- THEN `builder` is set to `<bash_path>/bin/bash` automatically

### Requirement: Examples use mkDerivation

The repository MUST include at least two working examples using
mkDerivation:

1. A C hello-world program compiled with gcc from the seed
2. A package that fetches source via fetchTarball and builds it

These examples MUST build successfully with `crunch build` given a
valid seed.

#### Scenario: C hello-world builds

- GIVEN `examples/seed.ncl` exists (from `crunch bootstrap`)
- WHEN `crunch build examples/hello-mkdrv.ncl -I examples` runs
- THEN the output contains a working `hello` binary
