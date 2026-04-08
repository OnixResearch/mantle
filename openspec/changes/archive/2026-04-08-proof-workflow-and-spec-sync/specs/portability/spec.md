## MODIFIED Requirements

### Requirement: Layered platform abstraction

The core crates MUST stay structured so platform-specific build execution can
be replaced behind traits, but the main spec MUST distinguish shipped behavior
from future portability work.

For the current implementation, build execution is only shipped on Linux.
Evaluation, manifest handling, and other pure logic remain portable Rust code.

#### Scenario: Non-Linux host keeps portable core but not build execution

- GIVEN crunch compiled on a non-Linux host
- WHEN a user evaluates Nickel or runs project-management commands
- THEN the pure core behavior still works
- AND a build attempt is handled by the platform-specific build boundary

### Requirement: Native sandbox as default, Linux-only build support in v0

v0 build execution MUST use the native Linux bubblewrap path.

On non-Linux hosts, `crunch build` and other build-entry commands MUST fail with
an explicit error that building is only supported on Linux and requires bwrap.

#### Scenario: Non-Linux build fails clearly

- GIVEN a non-Linux host
- WHEN `crunch build hello.ncl` is attempted
- THEN crunch returns a clear error explaining that building is only supported on Linux and requires bwrap

### Requirement: Future backends stay labeled as future work

WASM, OCI, and remote builders MUST stay labeled as future work in the main
spec and repo docs until code for them ships in the runtime path.

#### Scenario: Docs do not advertise unimplemented builders as available

- GIVEN the repo documentation and main specs
- WHEN they describe supported build backends
- THEN bubblewrap on Linux is listed as the shipped implementation
- AND WASM, OCI, and remote builders are labeled as future work

### Requirement: BuildService trait as the portability boundary

The `BuildService` trait MUST remain the abstraction boundary for future
portability work, and the current implementation list in the main spec MUST
match what the tree actually ships.

#### Scenario: Current implementation table is honest

- GIVEN the main portability spec
- WHEN it lists concrete build-service implementations
- THEN it marks bubblewrap as the current shipped build path
- AND it does not present unimplemented services as current runtime options
