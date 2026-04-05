## MODIFIED Requirements

### Requirement: Phase system

Each phase MUST have a default implementation:

| Phase | Default |
|-------|---------|
| `unpackPhase` | If `$src` is a directory, `cp -r`. If a file, `tar xf` and enter single subdirectory. |
| `configurePhase` | `./configure --prefix=$out` if `./configure` exists, else no-op. |
| `buildPhase` | `make -j$NIX_BUILD_CORES` |
| `installPhase` | `make install` |

Setting a phase to the empty string `""` MUST skip that phase.
Omitting a phase entirely MUST use the default.

### Requirement: src field wiring

When `src` is set in mkDerivation, the system MUST:
- Add it to `inputs` for sandbox mounting
- Set `$src` as an environment variable pointing to the source path

## ADDED Requirements

### Requirement: mkShell function

The stdlib MUST provide `crunch.mkShell` for development environments.

`mkShell` MUST:
- Accept `build_inputs` (array of store paths or derivation records)
- Accept `env` (extra environment variables)
- Produce a `Derivation` record that passes contract validation
- Fail deliberately if actually built

#### Scenario: mkShell produces valid derivation

- GIVEN `crunch.mkShell { bash = seed.bash, buildInputs = [seed.gcc] }`
- WHEN evaluated
- THEN the result satisfies `crunch.Derivation`
- AND `inputs` includes the gcc store path

#### Scenario: mkShell build fails

- GIVEN a mkShell derivation is submitted to `crunch build`
- WHEN the sandbox executes the builder
- THEN it exits nonzero with a message indicating the derivation is not buildable
