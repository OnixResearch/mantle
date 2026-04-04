# Build Pipeline Specification

## Purpose

Defines how derivations are built, cached, logged, and persisted.

## Requirements

### Requirement: Build caching / rebuild avoidance

Before building a derivation, the system MUST check whether the output
path already exists in the store (via `PathInfoService`). If the output
is already present, the build MUST be skipped entirely.

This applies to both derivation inputs and the top-level derivation.

#### Scenario: Already-built derivation

- GIVEN derivation A was previously built and its PathInfo exists in the store
- WHEN `crunch build` processes A again (directly or as an input)
- THEN no sandbox is invoked; the existing output is reused

#### Scenario: Input already built

- GIVEN derivation B depends on A, and A is already built
- WHEN B is built
- THEN only B's build runs; A's output is fetched from the store

### Requirement: Source input validation

For each `Input::Source` (pre-existing store path), the system MUST
verify that the path exists in the store before starting the build.
If a source path is missing, the build MUST fail with a clear error
before sandbox invocation.

#### Scenario: Missing seed path

- GIVEN `inputs = ["/nix/store/...-bash-5.2"]` but that path does not
  exist in the store
- WHEN build is attempted
- THEN an error is returned: "source input /nix/store/...-bash-5.2 not
  found in store"

### Requirement: Build execution

The system MUST execute builds via `snix-build`'s `BuildService::do_build`.
On Linux, this uses bwrap for sandboxing. The sandbox MUST:

- Mount only declared inputs (both `input_sources` and built
  `input_derivations` outputs) into the build environment
- Provide scratch directories for the build
- Set environment variables from `Derivation.environment`
- Execute the builder with the specified args

### Requirement: Build output persistence

After a successful build, the system MUST:

1. Compute the NAR hash and size of each output via `NarCalculationService`
2. Scan output contents for references to input store paths
3. Construct a `PathInfo` with the output node, references, NAR hash/size,
   and deriver
4. Persist the `PathInfo` via `PathInfoService`

### Requirement: Build logs

Build stdout and stderr MUST be captured and:

- Displayed to the user on build failure (exit code != 0)
- Available via `--verbose` on success
- Stored in a log directory (location TBD — `$CRUNCH_LOG_DIR` or
  `$XDG_STATE_HOME/crunch/logs/`) keyed by derivation hash

#### Scenario: Build failure

- GIVEN a derivation whose build script exits non-zero
- WHEN `crunch build` runs it
- THEN the build log (stdout + stderr) is printed, and crunch exits
  with code 1

#### Scenario: Successful build with verbose

- GIVEN a successful build
- WHEN run with `--verbose`
- THEN the build log is printed after the output path

### Requirement: Store initialization

The system MUST handle store initialization:

- If the store directory (`/nix/store` by default) does not exist, the
  system MUST create it (or fail with a permission error and a helpful
  message)
- The system MUST work in single-user mode (no daemon) for v0
- If `--store` specifies a non-default location, that path is used

#### Scenario: First run

- GIVEN no store exists
- WHEN `crunch build` is run for the first time
- THEN the store directory is created (or an error explains what
  permissions are needed)

### Requirement: Parallel builds (future)

v0 MAY build derivations sequentially. The system SHOULD be designed
so that independent derivations in the dependency graph can be built
in parallel in a future version.

The dependency graph (DAG) naturally exposes parallelism: derivations
with no unbuilt dependencies can start simultaneously. The `BuildService`
trait is async and supports concurrent calls.
