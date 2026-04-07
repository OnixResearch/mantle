# Agent Notes

## Repo Layout

This is a workspace directory, not a single project. Each subdirectory is an
independent repo with its own VCS, upstream remote, and build system. There is
no top-level Cargo.toml, flake.nix, or Makefile — always `cd` into a
subdirectory before building or running commands.

| Directory                | What it is                                       | Language    | VCS   | Default branch |
|--------------------------|--------------------------------------------------|-------------|-------|----------------|
| `adios`                  | Nix module system                                | Nix         | git   | master         |
| `json-schema-to-nickel`  | Converts JSON Schema → Nickel contracts          | Rust        | git   | main           |
| `nickel`                 | Nickel configuration language (interpreter, LSP)  | Rust        | git   | master         |
| `nixtamal`               | Nix input version pinning (like niv/npins)       | OCaml/dune  | darcs | —              |
| `npins`                  | Simple Nix dependency pinning                    | Rust        | git   | master         |
| `organist`               | Unified project tooling config via Nickel         | Nix/Nickel  | git   | main           |
| `snix`                   | Rust re-implementation of Nix components          | Rust        | git   | canon          |

All projects are in the Nix/Nickel ecosystem.

## Build Systems

- **Rust projects** (`nickel`, `npins`, `json-schema-to-nickel`, `snix`): use Cargo. `nickel` and `snix` are Cargo workspaces with multiple crates.
- **OCaml project** (`nixtamal`): uses dune 3.20. Has `default.nix` and `shell.nix` for Nix-based dev.
- **Nix projects** (`adios`, `organist`): pure Nix, no separate build step. Both have `flake.nix`.
- Most projects have `flake.nix` or `default.nix` / `shell.nix` for entering a dev environment.

## VCS Gotchas

- **nixtamal uses darcs**, not git. Its history is in `_darcs/`, not `.git/`. Standard git commands will fail inside it.
- **snix's default branch is `canon`**, not `main` or `master`.
- The top-level directory itself was not a git repo initially (the `.git` was created as bare during exploration — it's not meaningful).

## Upstreams

| Directory               | Remote                                                  |
|-------------------------|---------------------------------------------------------|
| `adios`                 | `git@github.com:adisbladis/adios.git`                   |
| `json-schema-to-nickel` | `git@github.com:nickel-lang/json-schema-to-nickel.git`  |
| `nickel`                | `git@github.com:tweag/nickel.git`                       |
| `nixtamal`              | (darcs, no git remote)                                  |
| `npins`                 | `git@github.com:andir/npins.git`                        |
| `organist`              | `https://github.com/nickel-lang/organist/`              |
| `snix`                  | `https://git.snix.dev/snix/snix.git`                    |

## Nickel Ecosystem Relationships

`nickel` is the language. `organist` uses Nickel for project config. `json-schema-to-nickel` generates Nickel contracts from JSON Schema. `nixtamal` includes Nickel schema files (`ncl/` dir). These projects share concepts but are versioned independently.

## Documentation

### adios

- **mdbook** (HTML): `nix-build doc/default.nix -A doc` → output in `adios/result/`
- Source markdown in `adios/doc/src/` (module system concepts, examples, lib reference)
- Uses `mdbook-cmdrun` preprocessor — needs nix to build

### json-schema-to-nickel

- **rustdoc**: `nix develop --command cargo doc --no-deps` → `json-schema-to-nickel/target/doc/json_schema_to_nickel/`
- Only a `README.md` otherwise

### nickel

- **User manual** (markdown files): `nix build .#userManual` → `nickel/result-manual/`
  - tutorial, syntax, typing, contracts, merging, modular configs, CLI, cookbook
- **Stdlib API docs** (markdown): `nix build .#stdlibMarkdown` → `nickel/result-stdlib/std.md`
- **Stdlib API docs** (JSON): `nix build .#stdlibJson`
- **rustdoc** (full workspace): `nix develop --command cargo doc --workspace --no-deps` → `nickel/target/doc/`
  - Key crates: `nickel_lang_core`, `nickel_lang_parser`, `nls` (LSP), `nickel_lang_package`, `nickel_lang_flock`
- Source markdown in `nickel/doc/manual/` — same files as the user manual nix output
- Also: `CONTRIBUTING.md`, `HACKING.md`, `RATIONALE.md`, `RELEASES.md`, `RELEASING.md`

### nixtamal

- **Man pages**: built via dune — `nix-shell --run "dune build doc/nixtamal-manifest.5"`
  - `doc/manifest.rst` → `nixtamal-manifest.5` (man page for the manifest format)
  - `nixtamal.1` is auto-generated from `--help=groff`
- `README.rst` is the main documentation (reStructuredText, not Markdown)

### npins

- **rustdoc**: `nix-shell --run "cargo doc --no-deps"` → `npins/target/doc/npins/`
- `README.md` + `CHANGELOG.md` — the README covers installation, usage, all source types

### organist

- No built docs — all documentation is markdown files:
  - `organist/README.md` — overview and getting started
  - `organist/doc/dependency-management.md`
  - `organist/doc/services.md`
  - `organist/doc/filegen.md`
  - `organist/doc/bootstrap-no-flake.md`
  - `organist/doc/migrate-from-0.1.md`
  - `organist/roadmap.md`, `organist/RELEASES.md`

### snix

- **rustdoc** (full workspace): → `snix/snix/target/doc/`
  - Key crates: `snix_eval`, `snix_store`, `snix_build`, `snix_castore`, `nix_compat`, `snix_glue`, `nar_bridge`, `nix_daemon`
  - Build: `cd snix/snix && SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix-shell -p cargo rustc pkg-config openssl protobuf --run "cargo doc --workspace --no-deps"`
- **Website/docs** (hugo source): `snix/web/content/docs/` — architecture, components, guides, reference
  - Build website: needs `hugo` + `npm` (see `snix/web/default.nix`), but the markdown source is readable as-is
  - Component docs: eval (VM loop, opcodes, builtins, bindings), store, castore, build (OCI, reference scanning)
  - Guides: building, contributing, performance, store composition, use as library
  - Reference: nix-daemon protocol (handshake, changelog)
- Per-crate READMEs scattered through `snix/snix/*/README.md`

## Doc Build Gotchas

- All Rust projects share `~/.cargo-target/` via global `~/.cargo/config.toml`. Running `cargo doc` in multiple projects overwrites the shared doc directory. Use `CARGO_TARGET_DIR=./target` to keep docs separate.
- snix requires `SNIX_BUILD_SANDBOX_SHELL=/bin/sh` env var at compile time — without it, `snix-build` fails.
- snix also needs `protobuf` (protoc) in PATH for prost-wkt-types.
- nickel needs `nix develop` for the right Rust toolchain (pinned to 1.91.1).
- nixtamal man page build needs `rst2man` — available inside `nix-shell`.

## Architecture Decision Records

Major decisions go in `adr/` as numbered markdown files. See
[`adr/README.md`](adr/README.md) for the format and index.

**When to write an ADR:** any decision that constrains future work —
tool choices, architectural patterns, conventions, rejected alternatives.
If you're picking between approaches and the reasoning isn't obvious,
write it down.

**How:**
1. Pick the next number (`ls adr/` to see what exists).
2. Create `adr/NNNN-short-title.md` using the template in the README.
3. Add an entry to the index table in `adr/README.md`.

Do this during the session, not as an afterthought.

## crunch Project Goal

crunch aims to **replace Nix entirely** — not just swap the config language
(Nickel for Nix), but own the full stack: evaluation, store, builder, scheduler,
and eventually the daemon. The vendored snix crates provide the data layer
(derivation format, content-addressed storage, PathInfo, BuildService trait),
but scheduling, orchestration, and the eval→build pipeline are crunch's own.

snix has no scheduler — just recursive async calls with a TODO comment.
Nix C++ has a Worker/Goal system (coroutines, fork+select, 6 goal types).
crunch takes the conceptual model (lazy goals, waiter notification, slot-limited
dispatch) but implements it with Rust/tokio (explicit state enum, JoinSet,
Semaphore, mpsc channel for streaming eval).

See `adr/0001-lazy-goals-vs-eager-dag.md` for the full rationale.

## crunch Build Environment

crunch is a Cargo workspace. It needs:

- **Rust nightly** (`~/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin`)
- **clang + mold** (linker config in `.cargo/config.toml`)
- **pkg-config + openssl-dev** (for reqwest/tls deps)
- **`SNIX_BUILD_SANDBOX_SHELL=<static-shell>`** at compile time (snix-build bwrap/oci).
  **MUST be a statically-linked binary** (e.g., busybox-static). The bwrap sandbox
  mounts inputs via FUSE and does not include the host's glibc, so dynamically-linked
  shells fail with `No such file or directory` inside the sandbox.
  After a full self-build, crunch uses its own bootstrapped busybox
  (`bootstrap/busybox.ncl`) and bwrap (`bootstrap/bwrap.ncl`) instead
  of external binaries. For development builds, any static shell works.
- **bwrap** (bubblewrap) on PATH for sandbox builds (first bootstrap only;
  subsequent self-builds use the crunch-built bwrap)

Nix store paths for dev-build tools (on the current machine):
```
clang:      /nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin
mold:       /nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin
pkg-config: /nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin
openssl:    /nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig
```

**Version pins**: `blake3 =1.8.2` and `digest =0.10.7` in snix-castore to avoid
digest 0.10/0.11 trait mismatch. blake3 1.8.3+ pulls digest 0.11 which breaks
snix-castore's `HashingReader<blake3::Hasher>` (the `traits-preview` feature
implements the wrong digest version).

**Vendored crates**: nix-compat has BLAKE3 modifications (sha256! macro, hash_derivation_modulo,
build_text_path, build_store_path_from_fingerprint_parts). snix-serde was NOT vendored
(depends on snix-eval). snix-store's snix-cli dep was removed (unused in source).

## crunch Self-Build

crunch can build itself from source with zero Nix runtime dependency:

```sh
crunch self-build --store /tmp/crunch-store -j 4 --no-substitute
```

This command:
1. Creates a source tarball (git archive + cargo vendor)
2. Computes the NAR hash via crunch's own castore pipeline
3. Builds the full bootstrap chain (musl-gcc → make → dash → binutils → musl → gcc → busybox → bwrap → rust)
4. Compiles crunch from source inside a bwrap sandbox
5. Bakes the crunch-built busybox path into the binary as the sandbox shell
6. Verifies the output binary

First bootstrap requires: `git`, `cargo`, `tar`, `xz`, `cp`, `sh` on PATH,
plus `bwrap` on PATH for the initial build. After the first successful
self-build, the crunch-built bwrap and busybox are used for subsequent
builds — no external sandbox tools needed.
Needs ~2 GiB free in /tmp for staging + hash computation.

Output: ~31 MiB static-pie musl-linked ELF binary.

## crunch Runtime Requirements

Building derivations (not just compiling crunch) requires:

- **bwrap** in PATH (`nix-shell -p bubblewrap`)
- **Writable /nix/store** for output files to land on disk. On read-only
  /nix/store (most NixOS machines), builds succeed (PathInfo persisted)
  but files aren't on disk. Test with:
  ```sh
  unshare --mount --map-root-user sh -c '
    mount -t overlay overlay -o lowerdir=/nix/store,upperdir=/tmp/upper,workdir=/tmp/work /nix/store
    crunch build hello.ncl
  '
  ```
- **nix-store** is NOT required. Closure resolution uses crunch's own
  PathInfo (local redb + remote binary cache narinfo). See `crunch-store/src/closure.rs`.
- **`--store <custom>`** works: derivation paths always use `/nix/store`
  (logical prefix). `--store` controls only where crunch writes outputs
  on the host filesystem. Source inputs always read from `/nix/store/`.

## crunch Key Design Decisions

- **Pipeline crate split**: `crates/crunch-pipeline` now owns eval -> deserialize -> convert -> build wiring. The binary shell is split across `src/build_cmd.rs`, `src/fix.rs`, `src/log_cmd.rs`, `src/store_cmd.rs`, and `src/self_build.rs`; `src/main.rs` is only CLI parsing and dispatch.
- **Worker goal keys still use `/nix/store` formatting internally**: `crunch-build::Worker`/`GoalRegistry` key derivations with `StorePath::to_absolute_path()`. `crunch-pipeline` normalizes failed goal keys back to the configured `store_dir` before returning `PipelineResult`, otherwise `--fix` and root-label lookup break under non-default prefixes.
- **Workspace test gotchas**: `vendor/fuse-backend-rs` had a Linux test that registered a dup of stdout with epoll; on this host stdout isn't epollable, so the test now uses a pipe read-end and must explicitly close/drop the pipe write end. `vendor/snix-castore` had a doctest for `ServiceBuilder` that now needs `#[async_trait::async_trait]` on the impl example to compile. `crates/crunch-pipeline/tests/integration_build.rs` should reuse `can_build()` for any `build()` integration test, even fetcher-only ones, because `crunch_pipeline::build()` is `Error::Build("building is only supported on Linux (requires bwrap)")` on non-Linux hosts.
- **Configurable store prefix**: `--store-prefix /crunch/store` (default) or
  `--nix-compat` for `/nix/store`. The prefix flows through
  ConversionCache → DerivationRegistry → StoreConfig → Builder → Worker.
  All ATerm serialization, hash computation, and path formatting use the
  configured prefix. Different prefixes produce different derivation hashes.
  The `_with_store_dir()` variants in nix_compat handle the plumbing.
- **CA provisionals**: CA derivations compute input-addressed output paths as
  provisional `$out`. After the build, the content hash determines the final
  path. Both paths have the same name → same length → byte-level self-reference
  rewriting works. Do NOT use `hash_placeholder()` — it produces a different-
  length string.
- **Closure resolution**: `crunch_store::resolve_closure()` walks PathInfo
  references (local redb, then remote narinfo). No `nix-store` subprocess.
  All closure members are mounted in the bwrap sandbox. Without closure data,
  dynamically-linked builders fail with "library not found" (clear error).
- **Castore export**: `export_castore_to_disk()` writes build outputs from the
  in-memory castore to the filesystem. Skips silently on read-only stores.
- **Fetcher as BuildService**: `builder = "builtin:fetchurl"` derivations flow
  through `FetchBuildService` (not inline in the orchestrator). `DispatchBuildService`
  routes between `FetchBuildService` and `BubblewrapBuildService` based on
  `command_args[0]`. The orchestrator treats all derivations the same: prepare →
  dispatch → finish. `build_fetcher()` was removed from Builder.
- **Pipeline FOD mismatch tests**: the cheapest end-to-end coverage is two
  `crunch.fetchurl` roots using `file://` URLs — one correct hash, one wrong.
  That exercises `PipelineResult.fod_mismatches` and sibling-root continuation
  without needing bwrap or network. `parse_fod_mismatch_error()` also strips a
  trailing `.drv` from the reported mismatch name.
- **BLAKE3 everywhere**: `HashAlgo::Blake3` + `NixHash::Blake3` in nix-compat,
  `NAR_BLAKE3`/`FLAT_BLAKE3` in pathinfo.proto, blake3 branches in
  `nar_hash()`/`hash_blob()`/`verify_flat_hash()`/`HashingReader`. The
  `blake3::Hasher` doesn't implement `digest::DynDigest` — use the
  `Blake3Wrapper` shim in `snix-store/src/nar/hashing_reader.rs`.
- **No protoc required**: gRPC/protobuf replaced with postcard serialization.
  Proto-generated types are now plain Rust structs with serde derives.
  Directory digests use postcard encoding (not protobuf canonical form).
  Existing redb databases are incompatible after this change.

## Coding Style: Tiger Style

Follow Tiger Style. The single most important principle is **Functional Core, Imperative Shell (FCIS)**:

- Extract pure, deterministic logic into functions with no I/O, no async, no external state mutation.
- Keep side effects (network, disk, time, randomness) in a thin shell layer.
- The shell calls the core; the core never calls the shell.

Other Tiger Style rules, in priority order:

1. **Assertion density.** At least two assertions per non-trivial function. Assert positive AND negative space. Split compound assertions.
2. **Fixed limits.** Explicit upper bounds on loops, queues, collections. Fail fast on violations.
3. **Functions under 70 lines.** One job per function.
4. **Explicitly sized types.** `u32`, `i64` — not `usize`. Consistent cross-platform behavior.
5. **Saturating/checked arithmetic.** No silent overflow.
6. **Decompose compound conditions.** Nested `if`/early returns over complex boolean expressions.
7. **Naming with units.** `latency_ms_max`, `capacity_bytes` — never bare numbers.
8. **Handle ALL errors.** Test error paths with equal rigor to happy paths.

## Working in Subdirectories

Always scope work to one subdirectory at a time. Running `cargo build` or `nix build` at the top level does nothing useful. Enter the project first:

```sh
cd nickel && cargo build
cd npins && cargo test
cd nixtamal && nix-shell  # then dune build
```
