## Why

crunch self-build still depends on two externally-provided binaries (bwrap,
busybox-static) and hardcodes `/nix/store` as the store path prefix. These
are the last Nix-ecosystem dependencies. Removing them makes crunch fully
self-hosting: given a C compiler and the musl-gcc tarball, crunch can
bootstrap its entire toolchain and rebuild itself with zero Nix involvement.

The `/nix/store` prefix also locks every derivation hash to the Nix
namespace. Since crunch already produces incompatible ATerm hashes (different
`outputs` env var, etc.), there's no real compatibility benefit. Switching to
a configurable prefix — defaulting to `/crunch/store` — makes the split
explicit and eliminates confusion about whether crunch outputs are
Nix-compatible (they aren't).

## What Changes

- **Configurable store prefix**: replace the hardcoded `STORE_DIR`/
  `LOGICAL_STORE_DIR` constant with a runtime-configurable value threaded
  through ConversionCache, DerivationRegistry, BuildRequest, Worker, and
  the sandbox env vars. Default: `/crunch/store`. `--nix-compat` flag
  sets it to `/nix/store` for interop testing.

- **Bootstrap busybox-static**: new `bootstrap/busybox.ncl` builds
  busybox with `make defconfig && make LDFLAGS=-static` using the
  existing musl + gcc stages. Produces a single static binary with
  all applets.

- **Bootstrap bwrap**: new `bootstrap/bwrap.ncl` compiles bubblewrap
  directly with gcc (bypass meson — bwrap is ~2k lines of C with no
  real dependencies beyond linux headers). Statically linked against musl.

- **Self-build uses crunch-built tools**: the self-build .ncl references
  the crunch-bootstrapped bwrap and busybox instead of external Nix store
  paths. First-ever bootstrap on a bare machine still needs an external
  bwrap (unavoidable chicken-and-egg), but after that crunch carries its
  own.

- **Vendored nix_compat patch**: `STORE_DIR` becomes a parameter or is
  replaced with the configured prefix at all call sites. `StorePath`
  construction, `to_absolute_path()`, and hash derivation functions
  accept the prefix.

## Capabilities

### New Capabilities
- `configurable-store-prefix`: derivation hashes computed against any prefix
- `bootstrap-busybox`: crunch builds its own sandbox shell
- `bootstrap-bwrap`: crunch builds its own sandbox runtime
- `nix-compat-flag`: `--nix-compat` mode for `/nix/store` interop

### Modified Capabilities
- `self-build`: uses crunch-built bwrap + busybox, `/crunch/store` prefix
- `bootstrap-chain`: extended with busybox + bwrap stages after gcc

## Impact

- **Files**: nix_compat (vendored STORE_DIR patch), all files with
  `LOGICAL_STORE_DIR`, build_request.rs (NIX_STORE env set to configured prefix),
  bootstrap/*.ncl (new files), self_build.rs, main.rs (new flags)
- **APIs**: ConversionCache::new, DerivationRegistry::new, BuildRequest,
  Worker — all gain a store_prefix parameter
- **Dependencies**: none added
- **Testing**: all hardcoded `/nix/store` assertions in tests must update
  to use the configured prefix. New tests for busybox/bwrap bootstrap.
  Self-build test with `/crunch/store` prefix.
- **Breaking**: existing pathinfo.redb and ca_mappings.json are
  invalidated (different hash prefix). Users must clear state dir.
  Document in release notes.
