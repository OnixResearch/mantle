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

### Requirement: Build session reuses source preparation work

The build pipeline MUST memoize source-closure expansion and reusable
input-node materialization within a single build session. Repeating the same
source path in multiple derivations MUST reuse the previously resolved closure
member set and previously known castore node when that node is still valid.

#### Scenario: Shared source closure resolved once per session

- GIVEN two derivations in one build session depend on the same source input
  path
- WHEN the first derivation resolves that source path's runtime closure
- THEN the second derivation reuses the memoized closure member set
- AND the pipeline does not repeat a fresh transitive closure walk for that
  same source path

#### Scenario: Existing local node reused without disk re-ingest

- GIVEN a dependency output already has local `PathInfo` and castore content
- WHEN a later derivation in the same session needs that output as a sandbox
  input
- THEN the pipeline reuses the known node metadata
- AND it does not re-ingest the same filesystem tree from disk before mounting
  it

### Requirement: Build execution

The system MUST execute builds via `snix-build`'s `BuildService::do_build`.
On Linux, this uses bwrap for sandboxing. The sandbox MUST:

- Mount only declared inputs (both `input_sources` and built
  `input_derivations` outputs) into the build environment
- Provide scratch directories for the build
- Set environment variables from `Derivation.environment`
- Execute the builder with the specified args

#### Scenario: Builder runs with only declared inputs mounted

- GIVEN a derivation with explicit source inputs and built input derivations
- WHEN crunch dispatches the sandboxed build
- THEN the build service mounts only those declared inputs into the sandbox
- AND the builder runs with the declared args and environment variables

### Requirement: Build output persistence

After a successful build, the system MUST:

1. Compute the NAR hash and size of each output via `NarCalculationService`
2. Scan output contents for references to input store paths
3. Construct a `PathInfo` with the output node, references, NAR hash/size,
   and deriver
4. Persist the `PathInfo` via `PathInfoService`

#### Scenario: Successful build persists output metadata before reporting success

- GIVEN a derivation output that completed successfully
- WHEN crunch finalizes that output
- THEN it computes the output NAR hash and size
- AND it records runtime references and persists the resulting `PathInfo`

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

### Requirement: Parallel builds

The build pipeline MUST treat eval root-force backend selection as runtime host
policy that is separate from `crunch-eval` forcing semantics.

The shipped runtime MAY prefer a local threaded eval backend when forcing roots
for build streaming, but it MUST keep the required inline backend as the
fallback when the preferred non-inline backend is unavailable, unsupported, or
not selected for the current request.

The runtime build pipeline MUST preserve root-label association, converted
output association, and labeled eval failure reporting regardless of whether the
current request used the preferred non-inline backend or the inline fallback.

The build pipeline MUST keep backend selection command-scoped. It MUST NOT
introduce a resident eval daemon or require library-mode binary self-spawn in
order to build derivations.

#### Scenario: Runtime root-force preference falls back to inline

- GIVEN the build runtime prefers a local threaded eval backend for root
  forcing
- AND that backend is unavailable, unsupported, or not selected for the current
  request
- WHEN the runtime streams build roots through the eval boundary
- THEN it falls back to the required inline backend for that request
- AND root-label association and converted output association remain unchanged

#### Scenario: Runtime eval failure stays labeled across backend choice

- GIVEN a build run with one root that fails during eval forcing
- WHEN the runtime executes that request through either the preferred
  non-inline backend or the inline fallback
- THEN the final build report identifies the same failed root label
- AND the runtime does not depend on partial-success results from that failed
  request

### Requirement: Build finalization persists artifact attestations

The build pipeline MUST materialize native artifact attestations during build
finalization after final output identity, runtime references, and content
hashes are known.

Artifact attestation persistence MUST happen before the build is reported as a
successful outcome to the caller.

#### Scenario: Successful build reports only after attestation persistence

- GIVEN a derivation whose outputs have been built and hashed successfully
- WHEN crunch finalizes the build result
- THEN it computes and persists artifact attestations for the successful outputs
- AND only then returns those outputs as successful build outcomes

### Requirement: Provenance generation uses native build facts

The build pipeline MUST derive observed provenance facts from crunch's own
native build and store data instead of rescanning exported filesystem trees as
a separate source of truth.

At minimum the pipeline MUST use derivation inputs, lockfile/project facts
when available, `PathInfo`, final output hashes, and recorded runtime
references as the observed-facts inputs to provenance generation.

#### Scenario: Intermediate output without exported disk path still gets provenance

- GIVEN an intermediate output that exists in castore and `PathInfo` but is not exported to the host filesystem
- WHEN crunch needs its artifact attestation for closure assembly
- THEN crunch derives the observed facts from native build/store data
- AND provenance generation does not require rescanning a host-visible output directory

### Requirement: Build pipeline carries explicit hermeticity mode

The build pipeline MUST accept and preserve an explicit hermeticity mode for
all build-entry runs.

At minimum the pipeline MUST distinguish between `practical` and `strict`
execution modes and make that selection available to build-finalization and
reporting code.

#### Scenario: Pipeline receives strict mode unchanged

- GIVEN a build-entry command selected hermeticity mode `strict`
- WHEN the pipeline starts the build run
- THEN the pipeline retains that exact mode selection
- AND later build stages can branch on `strict` without guessing from CLI flags

### Requirement: Build pipeline records typed hermeticity audit events

The build pipeline MUST record degraded execution facts as typed hermeticity
audit events.

Those events MUST be attached to the build result even when the build succeeds.
Later changes MAY promote specific event kinds to strict-mode blockers.

#### Scenario: Successful build still records degraded fact

- GIVEN a build succeeds after recording a degraded execution fact
- WHEN the pipeline returns the final result
- THEN the result includes the typed hermeticity audit event
- AND the event is available to both human and JSON reporting

### Requirement: Canonical execution envelope

The build pipeline MUST normalize a canonical execution envelope before each
sandboxed build starts.

At minimum the normalized envelope MUST set stable values for:

- `HOME`
- `PATH`
- `PWD`
- `TMP`, `TEMP`, `TMPDIR`, `TEMPDIR`
- `USER`, `LOGNAME`
- `SHELL`
- `LANG`, `LC_ALL`
- `TZ`
- `TERM`
- `SOURCE_DATE_EPOCH`
- `NIX_BUILD_CORES`
- `NIX_STORE`
- explicit `umask`

In strict mode, derivation-provided environment variables MUST NOT silently
override the reproducibility-sensitive subset of that envelope except for
explicitly allowed inputs such as `SOURCE_DATE_EPOCH`.

#### Scenario: Host locale and timezone do not leak into the build

- GIVEN the host shell has `LANG=en_US.UTF-8` and `TZ=America/New_York`
- WHEN a strict build starts
- THEN the builder sees the canonical crunch-selected locale and timezone values
- AND the host locale and timezone do not leak into the sandbox

#### Scenario: Host umask does not leak into the build

- GIVEN the host process starts with a restrictive or permissive umask
- WHEN crunch dispatches a sandboxed build
- THEN crunch sets the configured build umask before the builder runs
- AND output permissions are determined by the build envelope, not the host shell state

### Requirement: Determinism regression coverage

The repo MUST provide automated regression coverage that reruns representative
builds while perturbing ambient host state and checks that crunch either
produces identical results or fails for an explicit strict-mode blocker.

At minimum the regression matrix MUST vary `HOME`, `PATH`, `USER`, `TZ`,
`LANG`, `TMPDIR`, current working directory, and umask.

#### Scenario: Ambient host changes do not perturb output identity

- GIVEN a representative derivation selected for the determinism harness
- WHEN the harness reruns it under multiple ambient host-state combinations
- THEN the successful runs produce the same output digest
- AND the emitted hermeticity audit facts stay identical across those runs

#### Scenario: Strict-mode blocker remains stable under ambient-state variation

- GIVEN a representative strict-mode build that is expected to fail for a known blocker
- WHEN the harness reruns it under multiple ambient host-state combinations
- THEN the same blocker class is reported each time
- AND the result does not silently degrade into a different weaker execution path

