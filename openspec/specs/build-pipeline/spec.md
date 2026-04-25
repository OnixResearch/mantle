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
  nameserver configuration owned by crunch (not the host's resolv.conf)

#### Scenario: FOD build sees synthetic services

- GIVEN a fixed-output derivation with network access
- WHEN the build executes inside the sandbox
- THEN `/etc/services` contains exactly the fixed, deterministic service
  table owned by crunch (not the host's services database)

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

