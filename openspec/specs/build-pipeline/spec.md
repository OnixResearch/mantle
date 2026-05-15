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
- WHEN `mantle build` processes A again (directly or as an input)
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
- WHEN mantle dispatches the sandboxed build
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
- WHEN mantle finalizes that output
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
- WHEN `mantle build` runs it
- THEN the build log (stdout + stderr) is printed, and mantle exits
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
- WHEN `mantle build` is run for the first time
- THEN the store directory is created (or an error explains what
  permissions are needed)

### Requirement: Parallel builds

The build pipeline MUST dispatch independent ready derivations concurrently.
The maximum number of in-flight builds MUST be bounded by the configured
`max_jobs` value.

The scheduler MUST deduplicate goals by derivation path so the same derivation
is not built twice while concurrency is enabled.

#### Scenario: Independent derivations build concurrently

- GIVEN two derivations whose dependencies are already satisfied
- AND `max_jobs` is at least 2
- WHEN the pipeline runs
- THEN the worker may dispatch both builds without waiting for the first one to finish

#### Scenario: Jobs cap serializes dispatch

- GIVEN two derivations whose dependencies are already satisfied
- AND `max_jobs` is 1
- WHEN the pipeline runs
- THEN the worker dispatches at most one build at a time
- AND the second build waits until the first one reaches a terminal state

### Requirement: Runtime eval backend selection stays command-scoped

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
- WHEN mantle finalizes the build result
- THEN it computes and persists artifact attestations for the successful outputs
- AND only then returns those outputs as successful build outcomes

### Requirement: Provenance generation uses native build facts

The build pipeline MUST derive observed provenance facts from mantle's own
native build and store data instead of rescanning exported filesystem trees as
a separate source of truth.

At minimum the pipeline MUST use derivation inputs, lockfile/project facts
when available, `PathInfo`, final output hashes, and recorded runtime
references as the observed-facts inputs to provenance generation.

#### Scenario: Intermediate output without exported disk path still gets provenance

- GIVEN an intermediate output that exists in castore and `PathInfo` but is not exported to the host filesystem
- WHEN mantle needs its artifact attestation for closure assembly
- THEN mantle derives the observed facts from native build/store data
- AND provenance generation does not require rescanning a host-visible output directory

### Requirement: Build pipeline carries explicit hermeticity mode

The build pipeline MUST accept and preserve an explicit hermeticity mode for all
build-entry runs.

At minimum the pipeline MUST distinguish between `practical`, `strict`, and
`impure` execution modes and make that selection available to build-finalization,
reporting, attestation, cache-publication, and proof-classification code.

#### Scenario: Pipeline receives strict mode unchanged

- GIVEN a build-entry command selected hermeticity mode `strict`
- WHEN the pipeline starts the build run
- THEN the pipeline retains that exact mode selection
- AND later build stages can branch on `strict` without guessing from CLI flags

#### Scenario: Pipeline receives impure mode unchanged

- GIVEN a build-entry command selected hermeticity mode `impure`
- WHEN the pipeline starts the build run
- THEN the pipeline retains that exact mode selection
- AND later build stages can block deterministic proof or cache-publication paths
  without guessing from CLI flags

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
- WHEN mantle dispatches a sandboxed build
- THEN mantle sets the configured build umask before the builder runs
- AND output permissions are determined by the build envelope, not the host shell state

### Requirement: Determinism regression coverage

The repo MUST provide automated regression coverage that reruns representative
builds while perturbing ambient host state and checks that mantle either
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

### Requirement: Fixed sandbox hostname

The sandbox MUST set a fixed hostname inside the UTS namespace.
The hostname MUST be `localhost`.

#### Scenario: Build script reads hostname

- GIVEN a derivation whose build script runs `hostname`
- WHEN the build executes inside the sandbox
- THEN the output is `localhost` regardless of the host machine's hostname

#### Scenario: bwrap args include hostname flag

- GIVEN the bwrap argument list in `COMMON_BWRAP_ARGS`
- WHEN the sandbox is initialized
- THEN the args MUST include `--hostname localhost`

### Requirement: Masked /proc entries

The sandbox MUST hide host-specific hardware topology files under
`/proc` that can leak into build outputs.

The following paths MUST be masked (bind-mounted from `/dev/null`):

- `/proc/cpuinfo`
- `/proc/meminfo`
- `/proc/stat`
- `/proc/loadavg`
- `/proc/uptime`
- `/proc/version`

#### Scenario: Build script reads /proc/cpuinfo

- GIVEN a derivation whose build script reads `/proc/cpuinfo`
- WHEN the build executes inside the sandbox
- THEN the file contents are empty (reads from `/dev/null`)

#### Scenario: /proc/self remains accessible

- GIVEN a running process inside the sandbox
- WHEN it reads `/proc/self/status` or `/proc/self/fd`
- THEN the read succeeds with the process's own information

### Requirement: Deterministic /dev entropy surface

The sandbox MUST provide a deterministic `/dev` entropy surface that
prevents build-time entropy from entering outputs.

The sandbox MAY use bwrap's `--dev /dev` baseline for standard device
nodes, but it MUST NOT expose `/dev/random` or `/dev/urandom` as
working entropy sources. These paths MUST either be absent or
bind-mounted from `/dev/null`.

The sandbox MUST still provide `/dev/null`; other standard bwrap device
nodes such as `/dev/zero`, `/dev/full`, `/dev/tty`, or `/dev/console`
MAY be present when provided by bwrap's baseline.

#### Scenario: Build script reads /dev/urandom

- GIVEN a derivation whose build script reads `/dev/urandom`
- WHEN the build executes inside the sandbox
- THEN the read returns empty (EOF from `/dev/null`) or fails,
  preventing non-deterministic bytes from entering the output

#### Scenario: Build script writes to /dev/null

- GIVEN a derivation whose build script writes to `/dev/null`
- WHEN the write occurs
- THEN it succeeds as normal

### Requirement: Output permission normalization

When exporting a castore node to the filesystem, the export MUST
set deterministic file permissions on every entry.

- Regular non-executable files MUST have mode `0o444`
- Executable files MUST have mode `0o555`
- Directories MUST have mode `0o555`
- Symlinks are not permission-normalized (symlink permissions are
  irrelevant on Linux)

#### Scenario: Non-executable file exported

- GIVEN a castore file node with `executable: false`
- WHEN `export_castore_to_disk` writes the file
- THEN the file's mode is `0o444`

#### Scenario: Executable file exported

- GIVEN a castore file node with `executable: true`
- WHEN `export_castore_to_disk` writes the file
- THEN the file's mode is `0o555`

#### Scenario: Directory exported

- GIVEN a castore directory node
- WHEN `export_castore_to_disk` creates the directory
- THEN the directory's mode is `0o555`

### Requirement: Output timestamp normalization

When exporting a castore node to the filesystem, the export MUST
set a fixed mtime on every exported file, directory, and symlink.

The fixed epoch MUST be Unix timestamp `1` (1970-01-01T00:00:01Z),
matching the `SOURCE_DATE_EPOCH` sandbox default.

#### Scenario: Exported file has fixed mtime

- GIVEN a castore file node
- WHEN `export_castore_to_disk` writes the file
- THEN the file's mtime is Unix timestamp `1`

#### Scenario: Exported directory has fixed mtime

- GIVEN a castore directory node
- WHEN `export_castore_to_disk` creates the directory
- THEN the directory's mtime is Unix timestamp `1`

#### Scenario: Exported symlink has fixed lmtime

- GIVEN a castore symlink node
- WHEN `export_castore_to_disk` creates the symlink
- THEN the symlink's lmtime (not following the link) is Unix timestamp `1`

### Requirement: Deterministic NIX_BUILD_CORES default

The sandbox MUST set the default value of `NIX_BUILD_CORES` to `1`,
not `0`.

Derivations MAY override `NIX_BUILD_CORES` via their environment
(it remains in `ALLOWED_SANDBOX_ENV_OVERRIDES`).

#### Scenario: Default build uses single core

- GIVEN a derivation that does not set `NIX_BUILD_CORES`
- WHEN the sandbox environment is constructed
- THEN `NIX_BUILD_CORES` is `1`

#### Scenario: Derivation overrides NIX_BUILD_CORES

- GIVEN a derivation that sets `NIX_BUILD_CORES` to `4`
- WHEN the sandbox environment is constructed
- THEN `NIX_BUILD_CORES` is `4`
- AND no hermeticity audit event is emitted (it is an allowed override)

### Requirement: Cgroup namespace isolation

The sandbox MUST request cgroup namespace isolation with bwrap's
best-effort `--unshare-cgroup-try` flag. On kernels that support cgroup
namespace unsharing, build scripts MUST NOT see host cgroup paths through
the cgroup namespace. On kernels that do not support it, the sandbox MUST
continue instead of failing during startup.

#### Scenario: bwrap args include cgroup unshare

- GIVEN the bwrap argument list
- WHEN the sandbox is initialized
- THEN the args MUST include `--unshare-cgroup-try`

The `--unshare-cgroup-try` form MUST be used (not `--unshare-cgroup`)
because cgroup namespace unsharing requires kernel support that may
not be available on all hosts. The sandbox MUST NOT fail if cgroup
unsharing is unsupported.

### Requirement: Sysfs isolation

The sandbox MUST NOT expose the host's `/sys` filesystem to build
processes. The `/sys` mount point MUST either be absent or contain
only an empty tmpfs.

#### Scenario: Build script tries to read /sys/devices

- GIVEN a derivation whose build script reads `/sys/devices/system/cpu/`
- WHEN the build executes inside the sandbox
- THEN the read fails or returns empty (no host CPU topology exposed)

#### Scenario: /sys is not mounted

- GIVEN the bwrap argument list and sandbox root tmpfs
- WHEN the sandbox is initialized
- THEN no `--bind` or `--ro-bind` argument maps a host `/sys` path
  into the sandbox

### Requirement: FOD /etc isolation

The sandbox MUST NOT bind-mount the host's `/etc/resolv.conf` or
`/etc/services` into the sandbox for network-enabled (fixed-output
derivation) builds. The sandbox MUST provide synthetic versions of
these files with deterministic content.

The fixed synthetic `/etc/resolv.conf` content MUST be:

```text
nameserver 127.0.0.1
nameserver 8.8.8.8
```

The fixed synthetic `/etc/services` table MUST contain exactly these
service names and protocol pairs: `tcpmux` `1/tcp`, `echo` `7/tcp` and
`7/udp`, `discard` `9/tcp` and `9/udp`, `systat` `11/tcp`, `daytime`
`13/tcp` and `13/udp`, `qotd` `17/tcp`, `chargen` `19/tcp` and
`19/udp`, `ftp-data` `20/tcp`, `ftp` `21/tcp`, `ssh` `22/tcp`,
`telnet` `23/tcp`, `smtp` `25/tcp`, `time` `37/tcp` and `37/udp`,
`nameserver` `42/tcp`, `nicname` `43/tcp`, `domain` `53/tcp` and
`53/udp`, `bootps` `67/udp`, `bootpc` `68/udp`, `tftp` `69/udp`,
`gopher` `70/tcp`, `http` `80/tcp`, `kerberos` `88/tcp` and `88/udp`,
`pop3` `110/tcp`, `ident` `113/tcp`, `sftp` `115/tcp`, `nntp`
`119/tcp`, `ntp` `123/udp`, `imap` `143/tcp`, `snmp` `161/udp`,
`snmp-trap` `162/udp`, `bgp` `179/tcp`, `irc` `194/tcp`, `ldap`
`389/tcp`, `https` `443/tcp`, `smtps` `465/tcp`, `submission`
`587/tcp`, `ldaps` `636/tcp`, `imaps` `993/tcp`, and `pop3s`
`995/tcp`.

#### Scenario: FOD build sees synthetic resolv.conf

- GIVEN a fixed-output derivation with network access
- WHEN the build executes inside the sandbox
- THEN `/etc/resolv.conf` contains exactly the fixed, deterministic
  nameserver configuration owned by mantle (not the host's resolv.conf)

#### Scenario: FOD build sees synthetic services

- GIVEN a fixed-output derivation with network access
- WHEN the build executes inside the sandbox
- THEN `/etc/services` contains exactly the fixed, deterministic service
  table owned by mantle (not the host's services database)

#### Scenario: Non-FOD build has no resolv.conf

- GIVEN a non-FOD derivation without network access
- WHEN the build executes inside the sandbox
- THEN `/etc/resolv.conf` is absent or contains only the synthetic
  localhost entry (no host DNS config is visible)

### Requirement: /dev/shm isolation

The sandbox MUST provide a private, empty `/dev/shm` for each build.
Shared memory segments from the host or other builds MUST NOT be
visible inside the sandbox.

#### Scenario: Build sees empty /dev/shm

- GIVEN a derivation whose build script lists `/dev/shm`
- WHEN the build executes inside the sandbox
- THEN `/dev/shm` is empty (a fresh tmpfs)

### Requirement: Ordered maps in build orchestrator

The build orchestrator MUST NOT use `HashMap` in any code path where
iteration order can affect the content or ordering of build outputs,
attestation digests, or user-visible reporting.

`HashMap` MAY be used for internal lookup tables where iteration
order has no observable effect.

#### Scenario: Output infos iterated deterministically

- GIVEN a multi-output derivation build
- WHEN the orchestrator iterates over output infos
- THEN the iteration order is deterministic (sorted by output name)

### Requirement: SOURCE_DATE_EPOCH override policy

The sandbox MUST keep `SOURCE_DATE_EPOCH` in
`ALLOWED_SANDBOX_ENV_OVERRIDES`.

When a derivation overrides `SOURCE_DATE_EPOCH`, the sandbox MUST use
the derivation-provided value in both `Practical` and `Strict` modes
without emitting a hermeticity audit event. This preserves the existing
explicit-input policy while other protected sandbox variables remain
audited in `Practical` mode and rejected in `Strict` mode.

#### Scenario: SOURCE_DATE_EPOCH override in Practical mode

- GIVEN a derivation that sets `SOURCE_DATE_EPOCH` to `315532800`
- WHEN the sandbox environment is constructed in Practical mode
- THEN `SOURCE_DATE_EPOCH` is `315532800`
- AND no hermeticity audit event is emitted

#### Scenario: SOURCE_DATE_EPOCH override in Strict mode

- GIVEN a derivation that sets `SOURCE_DATE_EPOCH` to `315532800`
- WHEN the sandbox environment is constructed in Strict mode
- THEN `SOURCE_DATE_EPOCH` is `315532800`
- AND no hermeticity audit event is emitted

### Requirement: Impure builds are labeled and proof-blocking

An impure build MUST be labeled as impure in human output, JSON build reports,
hermeticity audit facts, and any generated attestations. The label MUST survive
successful builds so downstream commands can distinguish an impure success from
a proof-eligible success.

Impure builds MUST NOT satisfy deterministic-build proof, release
reproducibility proof, witness rebuild, or policy-satisfied release classes by
default. Future policy MAY define an explicit impure evidence class, but no
existing reproducibility or determinism class may silently accept impure
material.

#### Scenario: Successful impure build remains marked impure

- GIVEN an operator runs `mantle build --impure hello.ncl`
- AND the builder exits successfully
- WHEN Mantle returns the build result
- THEN the result includes hermeticity mode `impure`
- AND any attestation or JSON report contains an impure audit fact

#### Scenario: Impure output cannot satisfy release reproducibility

- GIVEN a release artifact was produced by an impure build
- WHEN release reproducibility or deterministic proof classification runs
- THEN the artifact is rejected for existing proof classes
- AND the report names impure execution as the blocker

### Requirement: Impure mode has an explicit host-input policy

Impure mode MUST define which host inputs are allowed to vary. The first policy
MAY allow selected host environment variables, current working directory access,
and network behavior required by development workflows, but each allowed class
MUST be represented as a typed impure audit event.

The build pipeline MUST NOT treat impure mode as an unbounded privilege escape:
security isolation such as no-new-privileges and namespace containment SHOULD
remain enabled where compatible. If a platform cannot keep a security boundary in
impure mode, the result MUST record that degraded security fact.

#### Scenario: Allowed host environment is audited

- GIVEN an impure build reads an allowed host environment variable
- WHEN the build result is finalized
- THEN the result records a typed impure host-environment audit event

#### Scenario: Security boundary degradation is explicit

- GIVEN impure mode requires disabling a sandbox control on a platform
- WHEN the build starts or completes
- THEN the build report records the disabled control as a typed degraded-security
  event
- AND the event is visible in JSON output

### Requirement: Deterministic build proof receipt

The build pipeline MUST support a canonical deterministic-build proof receipt
for a single derivation output set. The receipt MUST record the derivation
identity, selected hermeticity mode, workflow version, toolchain/provider
identity, logical store prefix, physical store isolation strategy, normalized
execution envelope, ambient host perturbation matrix, typed hermeticity audit
events, per-run output store paths, and per-output canonical BLAKE3 NAR digests.

The receipt MUST have a closed verdict enum with at least `not-attempted`,
`deterministic-match`, `mismatch`, `missing-evidence`, `impure-mode`, and
`unsupported-workflow`. Only `deterministic-match` permits a deterministic build
claim.

#### Scenario: Canonical receipt bytes are stable

- GIVEN the same deterministic-build proof facts in any insertion order
- WHEN Mantle serializes the proof receipt
- THEN the canonical bytes are identical
- AND the receipt digest is identical

#### Scenario: Deterministic claim requires matching BLAKE3 digest sets

- GIVEN two or more clean build runs for the same derivation identity
- AND each run completed in strict hermetic mode without proof-blocking audit
  events
- WHEN every required output has the same canonical BLAKE3 NAR digest in every
  run
- THEN the proof receipt verdict is `deterministic-match`

#### Scenario: Digest drift fails the proof

- GIVEN repeated clean build runs for the same derivation identity
- WHEN any required output has a different canonical BLAKE3 NAR digest between
  runs
- THEN the proof receipt verdict is `mismatch`
- AND Mantle MUST NOT claim the derivation is deterministic

### Requirement: Determinism proof runs use isolated clean stores

A deterministic-build proof attempt MUST run each comparison build in a clean
store namespace or fresh physical store directory that cannot reuse prior output
artifacts for the derivation under test. Dependency substitution MAY be allowed
only when the substituted dependency identities are declared in the receipt and
are held constant across all runs.

#### Scenario: Prior output reuse is rejected

- GIVEN a requested deterministic proof for derivation A
- AND A's output already exists in the default store
- WHEN Mantle schedules proof comparison runs
- THEN it uses fresh proof stores or namespaces for A
- AND it does not satisfy the proof by reusing the existing output for A

#### Scenario: Declared dependency substitution is stable

- GIVEN proof runs use substituted dependencies
- WHEN the proof receipt is emitted
- THEN every substituted dependency identity is recorded
- AND the dependency set is identical across all comparison runs

### Requirement: Determinism proof perturbs ambient host state

The deterministic-build proof harness MUST perturb ambient host state across
comparison runs to catch accidental host leakage. The first required matrix MUST
vary at least `HOME`, `PATH`, `USER`, `LOGNAME`, `TZ`, `LANG`, `LC_ALL`, temp
directories, current working directory, umask, and host process environment
noise while preserving the canonical sandbox envelope.

#### Scenario: Host perturbation does not affect strict build output

- GIVEN a derivation eligible for deterministic proof
- WHEN Mantle reruns it under the required ambient host perturbation matrix
- THEN successful runs produce the same BLAKE3 output digest set
- AND the proof receipt records the perturbation cases used

#### Scenario: Host leakage is proof-blocking

- GIVEN a proof run records a hermeticity audit event that indicates host state
  leaked into the build
- WHEN the proof receipt is finalized
- THEN the verdict is not `deterministic-match`
- AND the receipt names the blocking event kind

### Requirement: Deterministic proof requires strict hermetic mode

A deterministic-build proof attempt MUST run in strict hermetic mode. If the
operator selects practical or impure execution, Mantle MUST emit a receipt with
`impure-mode` or `missing-evidence` rather than promoting the result to a
deterministic claim.

#### Scenario: Practical build cannot be promoted silently

- GIVEN a derivation was built successfully in practical mode
- WHEN an operator asks whether it is deterministically proven
- THEN Mantle reports that deterministic proof evidence is missing
- AND it does not infer determinism from the successful practical build

#### Scenario: Impure build blocks determinism proof

- GIVEN a derivation run used impure mode
- WHEN deterministic proof classification is requested
- THEN the proof receipt verdict is `impure-mode`
- AND no deterministic claim is emitted

### Requirement: Deterministic proof runs use an enforced sandbox envelope

The system MUST enforce a sandbox envelope for deterministic proof runs. When
an operator requests deterministic proof runs for release reproduction, Mantle
MUST execute each proof rebuild through that proof sandbox envelope instead of
directly spawning the rebuild command on the host.

The proof sandbox envelope MUST:

- mount the release evidence bundle read-only,
- mount the selected rebuild recipe/tool path read-only,
- mount only the per-run output directory and per-run proof store directory as
  writable paths,
- provide only the Mantle reproducibility environment variables plus an explicit
  minimal allowlist required to execute the recipe,
- deny network access by default,
- use a fresh writable output and proof-store directory for every run, and
- produce canonical profile evidence that can be digested and recorded in the
  deterministic proof receipt.

If deterministic proof runs are requested and the sandbox executor is missing,
unsupported on the current platform, or cannot enforce the requested network and
mount policy, Mantle MUST fail closed before writing a deterministic proof
receipt. Ordinary `release reproduce` without deterministic proof runs MAY remain
available.

#### Scenario: Deterministic proof run executes in sandbox

- GIVEN a release evidence bundle and a rebuild recipe that only needs the
  bundle, output directory, proof store directory, and declared environment
- WHEN `mantle release reproduce --deterministic-proof-runs 2` runs
- THEN each proof run executes through the proof sandbox envelope
- AND each run records the sandbox profile identity in the proof receipt

#### Scenario: Host-only recipe is not deterministic proof evidence

- GIVEN a rebuild recipe that succeeds only by reading an undeclared host path
- WHEN deterministic proof runs are requested
- THEN Mantle fails the deterministic proof run instead of producing a
  deterministic proof receipt
- AND the diagnostic identifies undeclared host access or sandbox execution
  failure as the blocker

#### Scenario: Missing sandbox executor fails closed

- GIVEN the current platform lacks a supported proof sandbox executor
- WHEN deterministic proof runs are requested
- THEN Mantle exits non-zero before writing a deterministic proof receipt
- AND the diagnostic says deterministic proof sandbox execution is unavailable

The proof sandbox envelope MUST be covered by maintained tests or equivalent
deterministic checks that prove proof-run sandbox invocations deny undeclared
host access, do not expose the main rebuild output/store as writable proof-run
inputs, and keep network access disabled by default.

#### Scenario: Isolation regression blocks undeclared host dependency

- GIVEN a deterministic proof rebuild recipe that can only succeed by reading an
  undeclared host path
- WHEN the deterministic proof run executes under the proof sandbox isolation
  regression harness
- THEN the run fails before a deterministic proof receipt is written
- AND the diagnostic identifies the undeclared host access or sandbox isolation
  denial

#### Scenario: Proof sandbox invocation excludes main output reuse

- GIVEN `release reproduce` has a main rebuild output directory and separate
  deterministic proof run directories
- WHEN Mantle constructs the proof sandbox command for a deterministic proof run
- THEN the sandbox invocation binds only the per-run output directory and per-run
  proof store as writable proof inputs
- AND it does not bind the main rebuild output directory or another run's proof
  store as a proof-run input

#### Scenario: Proof sandbox invocation denies network by default

- GIVEN deterministic proof runs are requested for a normal release
  reproducibility recipe
- WHEN Mantle constructs the proof sandbox command
- THEN the sandbox invocation uses the configured no-network proof profile
- AND regression evidence fails if the invocation opts into host networking
  without an explicit future spec change

### Requirement: Deterministic proof unit records exact rebuild inputs

The build pipeline MUST define a deterministic proof unit before executing proof rebuilds. The proof unit MUST select one target artifact/output set and MUST record workflow identity/version, selected provider kind, source tree BLAKE3, vendor/input bundle BLAKE3, toolchain/stage roots, logical store prefix, sandbox profile identity, and selected output identities.

The proof unit MAY target the Mantle self-build/release artifact or a smaller release artifact, but the receipt MUST name the selected target explicitly. The proof unit MUST NOT imply that unselected bootstrap stages, packages, platforms, or social witnesses were proven.

#### Scenario: Proof unit identifies one selected artifact

- GIVEN an operator requests a deterministic proof
- WHEN Mantle creates the proof plan
- THEN the plan names exactly one selected proof-unit target/output set
- AND records workflow/version, selected provider kind, source/vendor BLAKE3, toolchain/stage roots, and sandbox profile identity before rebuilds start

#### Scenario: Missing exact input identity blocks planning

- GIVEN a requested deterministic proof lacks source tree BLAKE3, vendor/input BLAKE3, selected provider kind, or workflow version
- WHEN Mantle plans the proof
- THEN planning fails closed before any proof claim can be emitted

### Requirement: Deterministic proof receipt compares two clean BLAKE3 rebuild sets

The build pipeline MUST support a canonical deterministic proof receipt schema `mantle-deterministic-proof-receipt-v1`. The receipt MUST be serialized as canonical compact JSON and identified by a BLAKE3 digest over the canonical bytes excluding the self-digest field.

The receipt MUST record rebuild A and rebuild B artifact digest sets using BLAKE3. The receipt verdict MUST be a closed value. `self-rebuild-match` MUST be emitted only when all proof-unit input identities validate, provider kind linkage matches, sandbox evidence is supported, proof-store/output anti-reuse checks pass, and rebuild A/B artifact digest sets match exactly.

Closed non-promoting verdicts MUST cover at least `not-attempted`, `mismatch`, `missing-evidence`, `reused-store`, `impure-mode`, `unsupported-workflow`, `unsupported-sandbox`, `provider-kind-mismatch`, and `malformed-receipt`.

#### Scenario: Canonical receipt bytes are stable

- GIVEN the same deterministic proof facts in any insertion order
- WHEN Mantle serializes the proof receipt
- THEN the canonical bytes are identical
- AND the receipt BLAKE3 digest is identical

#### Scenario: Matching clean rebuilds produce self-rebuild match

- GIVEN rebuild A and rebuild B were executed for the same proof unit
- AND each run used a distinct clean proof store and output root
- AND each run records supported `mantle-proof-sandbox-v1:*` sandbox evidence
- AND every selected output has the same canonical BLAKE3 digest in both runs
- WHEN Mantle finalizes the receipt
- THEN the verdict is `self-rebuild-match`

#### Scenario: Digest drift fails the proof

- GIVEN rebuild A and rebuild B completed for the same proof unit
- WHEN any selected output has a different BLAKE3 digest between runs
- THEN the receipt verdict is `mismatch`
- AND Mantle MUST NOT claim `self-rebuild-match`

#### Scenario: Provider kind mismatch fails the proof

- GIVEN the proof unit records one selected provider kind
- BUT rebuild evidence, proof linkage, or prerequisites record a different provider kind
- WHEN Mantle validates the receipt
- THEN the receipt verdict is `provider-kind-mismatch` or validation fails closed
- AND Mantle MUST NOT claim `self-rebuild-match`

### Requirement: Determinism proof runs use isolated clean stores and supported sandbox evidence

A deterministic proof attempt MUST run each comparison build in a clean store namespace or fresh physical store directory that cannot reuse prior output artifacts for the proof-unit target. The first implementation MUST schedule at least rebuild A and rebuild B and MUST allocate distinct proof-store and output-root identities for each run.

Every proof run MUST execute through a supported sandbox envelope whose profile identity starts with `mantle-proof-sandbox-v1:`. Direct-host execution, missing sandbox evidence, unsupported profile identity, bypassed sandbox execution, or malformed sandbox evidence MUST fail closed.

Dependency substitution MAY be allowed only when substituted dependency identities are declared in the receipt and held constant across all proof runs. The selected proof-unit output itself MUST NOT be satisfied from the default store, the main release-reproduce output, or a previous proof run.

#### Scenario: Proof run roots are distinct and fresh

- GIVEN Mantle plans rebuild A and rebuild B for the same proof unit
- WHEN it allocates proof-store and output-root paths
- THEN every run receives distinct proof-store and output-root identities
- AND any root that already contains the proof-unit output is rejected before the proof run starts

#### Scenario: Main reproduce output cannot satisfy proof runs

- GIVEN `mantle release reproduce` has already produced a main rebuild output
- WHEN deterministic proof comparison runs are requested
- THEN the proof-run sandbox does not bind the main rebuild output or main rebuild store as writable proof inputs
- AND the receipt records separate proof-store and output-root identities for each comparison run

#### Scenario: Unsupported sandbox evidence fails closed

- GIVEN a proof run is recorded as direct-host or with a sandbox profile not starting with `mantle-proof-sandbox-v1:`
- WHEN Mantle validates the proof receipt
- THEN the receipt cannot produce `self-rebuild-match`
- AND the report identifies unsupported sandbox evidence

#### Scenario: Declared dependency substitution is stable

- GIVEN proof runs use substituted dependencies
- WHEN the proof receipt is emitted
- THEN every substituted dependency identity is recorded
- AND the dependency set is identical across all comparison runs

