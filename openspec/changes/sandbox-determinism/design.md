## Context

crunch's bwrap sandbox inherits several host-leaking behaviors from
the snix-build upstream. The current sandbox uses `--unshare-uts`
without setting a hostname, mounts a full `/dev` and `/proc`, does
not normalize output permissions or timestamps, and defaults
`NIX_BUILD_CORES=0` (all cores). Each vector independently can cause
bit-for-bit divergence across machines.

## Goals / Non-Goals

**Goals:**
- Eliminate all known host-dependent information leakage from the sandbox
- Normalize output artifacts so identical builds produce identical on-disk state
- Keep the changes minimal and backward-compatible with existing derivations

**Non-Goals:**
- Seccomp filtering (deeper defense, separate effort)
- Full `/proc` virtualization (masking specific files is sufficient)
- Changing the NAR/castore hashing model (already content-addressed)
- Time namespace isolation (bwrap lacks support; kernel 5.6+ feature)

## Decisions

### 1. Fixed hostname via `--hostname localhost`

**Choice:** Add `--hostname localhost` to `COMMON_BWRAP_ARGS`.

**Rationale:** bwrap's `--unshare-uts` creates a new UTS namespace but
copies the host's hostname into it. `--hostname` sets an explicit value.
Nix does the same. This is the cheapest fix with the highest impact for
autoconf-style builds.

**Alternative:** Leave it unfixed and document it. Rejected because the
fix is a single arg.

### 2. Mask `/proc` topology files with `--ro-bind /dev/null`

**Choice:** Bind-mount `/dev/null` over `/proc/cpuinfo`,
`/proc/meminfo`, `/proc/stat`, `/proc/loadavg`, `/proc/uptime`,
and `/proc/version` after the `--proc /proc` mount.

**Rationale:** These files are the most commonly read by build scripts
(autoconf, cmake, meson, and Rust's `num_cpus` crate). Masking them
makes reads return empty. `/proc/self/*` remains accessible because
processes need their own PID namespace info.

**Alternative:** Don't mount `/proc` at all. Rejected because many
builds legitimately need `/proc/self`.

**Implementation:** Add `--ro-bind-try /dev/null /proc/cpuinfo` etc.
after the existing `--proc /proc` in `COMMON_BWRAP_ARGS`. Use
`--ro-bind-try` so the sandbox does not fail if bwrap lacks support.

### 3. Mask `/dev/random` and `/dev/urandom`

**Choice:** After `--dev /dev`, bind-mount `/dev/null` over
`/dev/random` and `/dev/urandom`.

**Rationale:** bwrap's `--dev` creates a full devtmpfs including
entropy sources. Build-time randomness (UUID generation, nonce
creation) is a direct source of non-determinism. Masking them makes
reads return EOF, which causes most random-reading code to fail
visibly rather than silently producing different output.

**Alternative:** Use `--tmpfs /dev` and manually create only the
needed devices. Rejected because `--dev` handles device permissions
and ownership correctly, and we only need to mask two entries.

**Implementation:** Add `--ro-bind /dev/null /dev/random` and
`--ro-bind /dev/null /dev/urandom` after `--dev /dev`.

### 4. Permission normalization in `export_castore_to_disk`

**Choice:** After writing each file, directory, or symlink, set
permissions to the Nix convention: `0o444` for non-executable files,
`0o555` for executables and directories.

**Rationale:** File permissions are not part of the NAR hash, but
they affect downstream tools that `stat()` outputs (tar, rsync,
`make` mode-dependent rules). Nix normalizes them in the store; we
should too.

**Implementation:** In `export_file_to_disk`, set `0o444` for
non-executable files (currently no permission call) and keep `0o555`
for executables. In the directory creation path, call
`set_permissions(0o555)` after `create_dir_all`. Symlinks need no
change.

### 5. Timestamp normalization in `export_castore_to_disk`

**Choice:** After writing each file and directory, set mtime to
Unix timestamp `1` using `filetime::set_file_mtime`.

**Rationale:** NAR hashing does not include timestamps, but on-disk
files carry real mtimes. Any tool that reads mtimes (tar, make, rsync,
`ls -l`) sees different values across runs. Nix sets all store path
mtimes to epoch `1`.

**Alternative:** Use epoch `0`. Rejected because `SOURCE_DATE_EPOCH=1`
is the sandbox default, and some tools treat epoch `0` specially.

**Implementation:** Add the `filetime` crate to `crunch-store`. Call
`filetime::set_file_mtime(path, FileTime::from_unix_time(1, 0))`
after each file/directory write. For symlinks, use
`filetime::set_symlink_file_times` to set lmtime without following
the link.

### 6. `NIX_BUILD_CORES` default to `1`

**Choice:** Change the default from `"0"` to `"1"` in
`SANDBOX_ENV_VARS`.

**Rationale:** `0` means "use all available cores", which varies by
machine. Builds that embed `$(nproc)` or `NIX_BUILD_CORES` into
config headers produce host-dependent output. The override mechanism
is preserved so derivations that want parallelism can set it.

**Alternative:** Keep `0` and document it. Rejected because
determinism is the default; parallelism should be opt-in.

### 7. `--unshare-cgroup-try`

**Choice:** Add `--unshare-cgroup-try` to `COMMON_BWRAP_ARGS`.

**Rationale:** Without cgroup namespace unsharing, build scripts can
read `/sys/fs/cgroup/` paths that expose host resource limits and
cgroup hierarchy. The `-try` variant is safe on kernels without
cgroup namespace support.

**Alternative:** `--unshare-cgroup` (hard fail). Rejected because
older kernels may not support it, and the sandbox should degrade
gracefully.

### 8. HashMap audit in orchestrator

**Choice:** Audit `crates/crunch-build/src/orchestrate.rs` for
`HashMap` usage. Replace with `BTreeMap` in paths where iteration
order reaches output content or user-visible reporting.

**Rationale:** `HashMap` iteration is randomized per-process via
`RandomState`. While current code paths appear to use per-key
lookups (safe), `output_infos` and `substitutions` are iterated
in `finish_build` and `BuildOutcome` consumers. Converting to
`BTreeMap` makes the ordering deterministic by output name.

**Implementation:** Change `output_infos: HashMap<String, PathInfo>`
to `BTreeMap<String, PathInfo>` in `BuildOutcome` and `finish_build`.
Same for `substitutions`. Internal lookup tables (like
`source_closure_cache`) can stay `HashMap` since they are never
iterated into output.

### 9. Keep host `/sys` absent

**Choice:** Do not mount host `/sys` into the sandbox. The `--tmpfs /`
root leaves `/sys` absent unless a future code path adds it.

**Rationale:** sysfs exposes CPU topology (`/sys/devices/system/cpu/`),
block device info (`/sys/class/block/`), network interfaces
(`/sys/class/net/`), and DMI/BIOS data (`/sys/class/dmi/`). Build
scripts that auto-detect hardware features via sysfs produce
host-dependent output. Nix does not mount `/sys` at all in its
sandbox.

**Alternative:** Don't mount `/sys` at all. This is simpler and
matches Nix, but some builds may try to read `/sys/fs/cgroup`.
Since we add `--unshare-cgroup-try`, the cgroup tree is already
isolated, so a full `/sys` absence is acceptable.

**Implementation:** bwrap already does not explicitly mount `/sys`.
Verify that no code path adds a host `/sys` bind mount. If the
`--tmpfs /` root already excludes `/sys`, this is a no-op validation.
Add an assertion test.

### 10. Synthetic `/etc` for FOD builds

**Choice:** Replace host bind-mounts of `/etc/resolv.conf` and
`/etc/services` with synthetic files for network-enabled (FOD) builds.

**Rationale:** `vendor/snix-build/src/bwrap/mod.rs` currently does
`--ro-bind /etc/resolv.conf /etc/resolv.conf` and
`--ro-bind /etc/services /etc/services` when `allow_network` is true.
This leaks the host DNS resolver configuration and service database
into FOD build outputs. While FODs are hash-checked, the leaked
content can affect build behavior (e.g., which DNS server is queried
first, timeouts, search domains) and log output.

**Alternative:** Keep host files. Acceptable for now since FOD outputs
are hash-verified, but the leak is still observable in build logs.

**Implementation:** Write synthetic `resolv.conf`
(`nameserver 127.0.0.1` plus deterministic `nameserver 8.8.8.8`
fallback) and `services` (minimal deterministic subset) alongside
the existing synthetic `/etc/passwd`, `/etc/group`, and `/etc/hosts`.
Bind-mount the synthetic versions instead of the host files.

### 11. Mask `/dev/shm`

**Choice:** Bind-mount a read-only tmpfs over `/dev/shm` after
`--dev /dev`.

**Rationale:** bwrap's `--dev` creates a full devtmpfs that includes
`/dev/shm`. Shared memory segments from the host or from other
sandbox runs can leak state. A fresh empty tmpfs isolates each build.

**Implementation:** Add `--tmpfs /dev/shm` after `--dev /dev` in
`COMMON_BWRAP_ARGS`. This gives each sandbox a private, empty
shared memory filesystem.

## Risks / Trade-offs

**[Broken builds from masked /dev/urandom]** Some build scripts
legitimately need randomness (e.g., Rust's `HashMap` hash seed at
build time, Python's `os.urandom` in test suites). These builds
will fail visibly, which is the correct outcome for reproducibility.
Derivations that need entropy should use fixed-output derivations
with network access.

**[Performance regression from NIX_BUILD_CORES=1]** Single-core
default slows parallel-capable builds. Mitigated by the override
mechanism: bootstrap derivations and known-parallel builds should
set `NIX_BUILD_CORES` explicitly.

**[/proc masking breaks introspection-heavy builds]** Builds that
read `/proc/cpuinfo` for feature detection (e.g., SIMD flags) will
get empty results. This is intentional: host-specific optimizations
should be declared in the derivation, not auto-detected.

**[filetime dependency]** Adds one new crate to `crunch-store`.
The crate is well-maintained and widely used (1.3B downloads).

**[Synthetic resolv.conf breaks some FOD fetches]** A `nameserver
127.0.0.1` default requires a local DNS resolver. If no resolver is
running, DNS resolution can fall back to the deterministic
`nameserver 8.8.8.8` entry. This keeps host DNS config out of the
sandbox while preserving common FOD fetch behavior.

**[/sys absence breaks rare builds]** Some builds legitimately read
sysfs for hardware feature detection. These builds will fail, which
is the correct outcome for reproducibility. The derivation should
declare the detected feature as an explicit input.
