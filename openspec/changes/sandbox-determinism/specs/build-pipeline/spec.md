# Sandbox Determinism Specification

## Purpose

Defines the requirements for eliminating host-dependent and
run-dependent information leakage from the crunch build sandbox,
ensuring bit-for-bit reproducible outputs given identical inputs.

## ADDED Requirements

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

The sandbox MUST unshare the cgroup namespace so that build scripts
cannot read host cgroup paths or resource limits.

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

## MODIFIED Requirements

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
