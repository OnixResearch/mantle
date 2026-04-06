## Context

crunch self-build works end-to-end but still depends on:
1. Two externally-provided static binaries (bwrap, busybox) from the Nix store
2. The `/nix/store` path prefix hardcoded in ~90 places across vendored
   nix_compat and crunch-owned code

The bootstrap chain already builds musl-gcc → make → dash → binutils →
musl → gcc → rust from source. Adding busybox and bwrap extends this chain.
The store prefix change is orthogonal but coupled — both are needed for
crunch to be fully independent of Nix.

## Goals / Non-Goals

**Goals:**
- Zero Nix binary invocations in the self-build path
- Zero Nix store paths referenced at runtime (except via `--nix-compat`)
- crunch builds its own bwrap and busybox
- Store prefix configurable, default `/crunch/store`

**Non-Goals:**
- Binary cache compatibility with cache.nixos.org (already broken)
- Removing the vendored nix_compat crate (still needed for derivation format)
- Distributed builds or remote store support (separate change)
- Solving the chicken-and-egg problem for the very first bootstrap on a
  bare machine (always needs some external bwrap)

## Decisions

### 1. Store prefix as runtime parameter, not compile-time constant

**Choice:** Thread the store prefix as a `&str` parameter through all
functions that reference it. No global/thread-local state.

**Rationale:** Compile-time constants (current approach) require recompiling
to change the prefix. Thread-locals are implicit and hard to test. An
explicit parameter makes the dependency visible and testable — you can run
tests with `/test/store` and production with `/crunch/store` in the same
binary.

**Alternative rejected:** `Cow<'static, str>` in a `OnceCell`. Simpler call
sites but hides the dependency. Testing with multiple prefixes in the same
process becomes racy.

**Implementation:**
- Patch `nix_compat::store_path::STORE_DIR` usages. Don't change the
  constant itself (breaks vendor diff tracking). Instead, add parallel
  functions that accept a `store_dir: &str` parameter:
  `build_store_path_from_fingerprint_parts_with_prefix()`,
  `to_absolute_path_with_prefix()` (already exists).
- `ConversionCache::new(prefix)` — already takes prefix
- `DerivationRegistry::new(prefix)` — already takes prefix
- `BuildRequest` — add `store_dir` field
- `Worker` — gets prefix from Builder
- `Builder::new()` — gets prefix from StoreConfig
- CLI: `--store-prefix` flag, stored in `Args`, passed to `cmd_build`

The prefix flows: CLI → cmd_build → ConversionCache + DerivationRegistry +
StoreConfig → Builder → Worker → BuildRequest → sandbox env.

### 2. Build bwrap without meson

**Choice:** Compile bwrap's C sources directly with `gcc -static`.

**Rationale:** bwrap is ~2000 lines of C in 4-5 source files. Its meson
build system exists for portability (BSD, non-Linux) and feature detection
(selinux, libcap). crunch targets Linux x86_64 with musl — we know exactly
what's available. A direct compile avoids pulling in meson → python →
python's 200+ source files.

**Alternative rejected:** Build meson and python as bootstrap stages. Too
much complexity for one small binary.

**Implementation:**
- Fetch bwrap 0.11.0 tarball via `crunch.fetchTarball`
- Compile with: `gcc -static -DHAVE_MOUNT_API -DHAVE_PIDFD_OPEN
  -D_GNU_SOURCE -o bwrap bubblewrap.c bind-mount.c network.c
  utils.c parse-mountinfo.c -I.`
- The `-DHAVE_*` flags match what our kernel (5.x+) supports
- Config header written inline (like the make/dash bootstrap approach)
- Test: `bwrap --version` in a post-build check

### 3. busybox defconfig + static musl link

**Choice:** Build busybox with `make defconfig` then `make LDFLAGS=-static
CC=musl-gcc`.

**Rationale:** defconfig includes all common applets (~350). We need ~30
for the sandbox. Building with defconfig is simpler than maintaining a
custom .config file, and the size penalty is small (~1-2 MiB for a static
busybox).

**Alternative rejected:** Minimal .config with only needed applets. Saves
~1 MiB but requires maintaining a custom config that breaks on busybox
version bumps.

**Implementation:**
- Fetch busybox 1.37.0 tarball
- `make defconfig`
- Patch `.config`: set `CONFIG_STATIC=y`, `CONFIG_INSTALL_NO_USR=y`
- `make -j$JOBS CC=gcc LDFLAGS=-static`
- Output: `$out/bin/busybox` + symlinks for all applets
- Test: `busybox sh -c 'echo ok'`

### 4. Default prefix `/crunch/store`

**Choice:** `/crunch/store` as the default, `/nix/store` via `--nix-compat`.

**Rationale:** Makes the Nix independence explicit. Users who see
`/crunch/store/...` paths know these aren't Nix paths. The prefix is
the same length as `/nix/store` (13 bytes including the trailing slash
vs 11) — wait, that's different. `/nix/store` is 10 chars, `/crunch/store`
is 13. This matters for self-reference rewriting in CA derivations (marker
length must match final path length).

**Correction:** Store path format is `/<prefix>/<32-char-hash>-<name>`.
The hash is always 32 chars. The name varies. For self-reference rewriting,
the provisional and final paths must have the same *total* length, which
they do because they have the same name and both are under the same prefix.
The prefix length doesn't affect this — both provisional and final use the
same prefix. Cross-prefix rewriting (building with one prefix, installing
with another) is not supported and not needed.

**Implementation:** Change the constant, update all tests. The `--nix-compat`
flag is a single check in `main.rs` that overrides the default.

### 5. Bootstrap chain ordering

**Choice:** busybox and bwrap build after gcc + musl (same stage as the
existing binutils build), before rust.

```
musl-gcc (fetched) → make → dash → binutils → musl → gcc
                                                       ↓
                                              busybox ← ┘
                                              bwrap   ← ┘
                                              rust    ← ┘
                                                       ↓
                                              crunch  ← ┘
```

**Rationale:** Both need gcc + musl for static linking. They don't depend
on each other or on rust. Building them in parallel with rust is possible
but not worth the complexity — they're fast builds (~30s each).

## Risks / Trade-offs

**[State invalidation]** Changing the default prefix invalidates all
existing pathinfo.redb and ca_mappings.json. Mitigation: document in
release notes, auto-detect and warn on incompatible databases (check if
stored paths start with the wrong prefix).

**[nix_compat patch scope]** STORE_DIR is referenced in ~15 functions
across store_path.rs, derivation/*.rs. Missing one site produces wrong
hashes silently. Mitigation: add an integration test that builds the same
derivation with two different prefixes and asserts the hashes differ.

**[bwrap feature detection]** Compiling bwrap without meson skips feature
detection. If our config.h is wrong for a kernel, bwrap may fail at
runtime with obscure syscall errors. Mitigation: test on the actual
target kernel; pin bwrap version; the config.h flags are for syscalls
available since Linux 5.2 (2019).

**[First bootstrap chicken-and-egg]** The very first `crunch self-build`
on a bare machine still needs an external bwrap. This is inherent — you
can't sandbox a build without a sandbox. After the first successful
self-build, the crunch-built bwrap is available for subsequent builds.
Mitigation: document the one-time dependency clearly; provide a static
bwrap download URL in the README.
