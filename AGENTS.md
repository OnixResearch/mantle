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
1. Copies a fixed allowlist of checkout entries into a staged `*-crunch-src` tree (pure Rust filesystem copy, no host `git archive` / `cargo vendor` / `cp` glue)
2. Computes a deterministic content fingerprint for that staged source tree
3. Builds the full bootstrap chain (musl-gcc → make → dash → binutils → musl → gcc → busybox → bwrap → rust)
4. Compiles crunch from source inside a bwrap sandbox
5. Bakes the crunch-built busybox path into the binary as the sandbox shell
6. Verifies the output binary

First bootstrap requires the source tree with `vendor-deps/` and
`.cargo/vendor-config.toml`, plus `bwrap` on PATH and a static sandbox shell.
The self-build source guard validates staged vendored Cargo inputs directly
from `Cargo.lock` and Cargo's `.cargo-checksum.json` metadata without invoking
host `cargo`; the SHA-256 there is Cargo interoperability, while crunch-owned
fingerprints stay BLAKE3. After the first successful self-build, the
crunch-built bwrap and busybox are used for subsequent builds — no external
sandbox tools needed.
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
- **Workspace test gotchas**: `vendor/fuse-backend-rs` had a Linux test that registered a dup of stdout with epoll; on this host stdout isn't epollable, so the test now uses a pipe read-end and must explicitly close/drop the pipe write end. `vendor/snix-castore` had a doctest for `ServiceBuilder` that now needs `#[async_trait::async_trait]` on the impl example to compile. `crates/crunch-pipeline/tests/integration_build.rs` should reuse `can_build()` for any `build()` integration test, even fetcher-only ones, because `crunch_pipeline::build()` is `Error::Build("building is only supported on Linux (requires bwrap)")` on non-Linux hosts. Host-env leakage tests in that file should poison env/umask in a subprocess, not with in-process `set_var`, so the test stays thread-safe under libtest. When those tests poison `PATH`, build it with `std::env::split_paths`/`join_paths` (not a raw joined PATH string), and if they poison `TMPDIR`/`TEMP`/`TMP`/`TEMPDIR`, point them at real temp directories or `tempfile::tempdir()` setup fails before the actual hermeticity assertion runs. `tests/integration_build.rs` fetcher e2e coverage should use `DispatchBuildService<FetchBuildService, ...>` plus `file://` fixtures; a plain `DummyBuildService` with a local HTTP server can hang forever because no request reaches the server and `server.join()` blocks. PATH-sensitive `src/self_build.rs` tests must serialize with `PATH_MUTEX`, point PATH at tempdirs they control (including an empty tempdir for no-tool cases), use fake executables instead of the host environment, and assert the full error string with `assert_eq!` so the tests stay deterministic. Optional-feature audit note: `cargo check -p fuse-backend-rs --features async-io` now gets past the old `unexpected cfg(feature = "async_io")` warning, but the feature still fails deeper with upstream dyn-compat / async trait errors in `src/api/vfs/mod.rs` and `src/api/filesystem/async_io.rs`. Backward-compat audit note: recreating `crates/crunch-build/src/export.rs` only restores downstream API if `crates/crunch-build/src/lib.rs` exposes it as `pub mod export;`; keep an integration test that imports `crunch_build::export::export_castore_to_disk` so the path stays checked. Shared `~/.cargo-target` can also poison rustdoc/doctest runs with cross-toolchain artifacts on this host; for validation, prefer `cargo test --lib --tests ...` or isolate with `CARGO_TARGET_DIR` instead of trusting plain `cargo test` doctest failures. It can also fill the root filesystem after large rebuilds; if edits fail with `ENOSPC`, check `df -h .` and prune `~/.cargo-target/debug/incremental` first (freed ~20 GiB on 2026-04-25).
- **fetchGit local transport gotcha**: `gix::clone::PrepareFetch` still tried to invoke `git-upload-pack` for `file://` repos on this host. `crates/crunch-build/src/fetcher.rs::fetch_git()` now special-cases `file://` URLs by opening the local repo with `gix::open_opts(..., gix::open::Options::isolated())`, resolving the requested tree, and materializing it via `repo.worktree_stream(...)` instead of cloning.
- **fetchGit PATH tests**: any `crates/crunch-build/src/fetcher.rs` test that mutates `PATH` must lock `PATH_MUTEX` before repo fixture setup, not just before the final `fetch_git()` call. Those fixtures still create local repos with host `git`, so parallel PATH poisoning otherwise breaks unrelated fetchGit tests.
- **fetchGit remote transport coverage**: cheapest non-`file://` coverage is a local `git daemon` served through a test-owned `TcpListener` on `127.0.0.1:0`, with each accepted socket handed to `git daemon --inetd`. This avoids the bind-drop-rebind race from `reserve_local_port()` + later spawn and makes readiness the owned listener itself, not some unrelated process on the chosen port. Capture the real `git` executable path before poisoning `PATH`; otherwise the per-connection inetd child can accidentally run the fake `git` helper instead of the host git binary needed for the fixture server. Also bound inetd-child teardown: poll `try_wait()`, kill on shutdown/time budget expiry, then reap, or `GitDaemonGuard::drop()` can hang forever on a stuck child.
- **OpenSpec MUST keyword gotcha**: `openspec validate` can reject a requirement when the first wrapped line of its prose lacks `MUST`/`SHALL`, even if the next physical line contains it. Keep the keyword on the first line of each requirement's opening sentence.
- **Typed OpenSpec gates need extra metadata beyond `openspec validate`**: `openspec_gate` typed mode wants `ID:` lines with dotted lowercase IDs (`review.traceability.coverage` style), typed task IDs like `I1` / `V2` / `H1` / `R3`, `[covers=...]` tags on typed tasks, and `[evidence=...]` on `V`/`H` tasks. Evidence files must exist and their `Task-ID` / `Covers` metadata must match the task. `openspec validate` can pass while `openspec_gate` still fails on this typed traceability layer.
- **Archived changes are not `openspec_gate` targets either**: if you need a post-archive tasks/proposal/design gate transcript, temporarily restore the archived dir under `openspec/changes/<name>/`, run `openspec_gate ... change=<name>`, then remove the temporary active copy again.
- **OpenSpec gate cwd gotcha under `.pi/worktrees/`**: `openspec_gate` can evaluate the original repo cwd instead of a scratch `.pi/worktrees/...` checkout, even when shell commands are running in the worktree. If gate findings look stale (for example all tasks appear unchecked), verify `/home/brittonr/git/crunch/crunch` state and cherry-pick/sync worktree commits before re-running the gate.
- **Full-source bootstrap root blocker**: `full-source-bootstrap-root` cannot be honestly completed by stubbing a provider. The required source-root provider must produce a real normalized seed contract (`x86_64-linux-musl-gcc`, g++, binutils, headers, `libgcc_s.so.1`, `libc.so`, provider metadata) without the legacy musl.cc tarball. A fail-closed `--source-root` manifest-validation path is useful progress, but do not mark provider/materialization/self-build proof tasks done until a real cross-toolchain bootstrap exists.
- **Archived live-bootstrap changes are partial, not done**: `openspec/changes/archive/2026-04-26-live-bootstrap-seed-chain/` and `...live-bootstrap-intermediate-tools/` were archived with deferred validation and placeholder derivations still present. Treat them as historical scaffolding only; current active successor `repair-live-bootstrap-archive-status` owns the unfinished work. Also watch `bootstrap/seed-legacy.ncl`: it must contain the concrete reduced musl.cc provider, not `import "seed-legacy.ncl"` self-recursion.
- **Retroactive no-std validation changes must budget checker-driven API cleanup explicitly**: if a change widens API-shape enforcement to already-landed core crates, same-family review treats "spec-only/no API redesign" wording as incoherent once public borrowed helpers are tightened or replaced to satisfy the checker. Proposal/design must allow minimal boundary-normalizing signature changes up front.
- **Same-family tasks review is more literal than the deterministic typed check**: even with valid `[evidence=...]` files, the tasks-stage review can still fail if `tasks.md` hides a spec-mandated exact command list behind a wrapper name or if the only proof summary lives in the evidence file. Keep the exact spec-required commands in the `V` task text and add a short inline evidence summary under the task so the review packet can trust completed boxes without opening the evidence artifact.
- **Ownership-review completeness must be derived, not hard-coded**: for `no-std-functional-core`, the ownership checker must derive touched std Rust files from the union of git history for the active change path `openspec/changes/no-std-functional-core/` and the archived path `openspec/changes/archive/*-no-std-functional-core/` when present, then require `ownership-review.md` to classify every touched non-core, non-legacy std Rust path as `adapter-only` or `unrelated`. Looking at only one side of the archive boundary yields false-clean results.
- **Proposal-gate rule for no-std/core-shell changes**: if a delta spec names exact modules/functions moving into new core crates, proposal gate also wants deterministic proof semantics for ownership, shell-boundary test assertions, crate feature model (`std` feature forbidden, not just non-default), and crate-scoped duplicate-logic bans. Name the legacy std source paths and define what counts as a thin adapter vs forbidden business logic.
- **`no-std-functional-core` API-shape gotcha**: the planned checker bans public traits and reference-based public core signatures. `crunch-attestation::Canonicalize` and `crunch-project` refresh helpers cannot move into `*-core` unchanged; keep resolver traits / extension-trait ergonomics in std adapters and budget an explicit core API reshaping pass.
- `crunch-attestation-core` and `crunch-project-core` both compile for `wasm32-unknown-unknown` with `serde` in `default-features = false, features = ["alloc", "derive"]`. `crunch-project-core` foundation modules also needed `HashSet`/`HashMap` replacements with `BTreeSet`/`BTreeMap`, no-std test modules need `use alloc::vec;` for the `vec![]` macro, and tests that call `std::...` helpers like `std::panic::catch_unwind` need `#[cfg(test)] extern crate std;` in the crate root.
- **Current no-std adapter split**: `crates/crunch-attestation/src/adapter.rs` now owns the std `Canonicalize` trait plus ref-based `evaluate_policy(...)` / `binary_digests_match(...)` wrappers, while legacy `src/{canonical,policy,release}.rs` are re-export shims and `crunch-attestation-core/src/{canonical,policy,release}.rs` own the pure logic. `crates/crunch-project/src/{refresh_adapter,attestation_adapter,upgrade_adapter}.rs` now hold the borrowed resolver / borrowed attestation / error-mapping wrappers, while legacy `src/{refresh,attestation,upgrade}.rs` are re-export shims and `crunch-project-core/src/{refresh,attestation,upgrade}.rs` own the owned-data core requests. Boundary proof tests live at `crunch-attestation::discovery::tests::shell_adapter_keeps_discovery_outside_core` and `crunch-project::refresh_adapter::tests::shell_adapter_keeps_refresh_io_outside_core`.
- **Attestation independence policy is selector-driven**: `crunch-attestation-core::ReleasePolicy.independence_field` is live, not decorative. Current supported values are `witness_identity`, `signer_key_name`, and `rebuild_environment_summary.host_class`; if you add another value, wire the selector in `policy.rs` and extend the core tests so the field changes the counted independence domain.
- **No-std boundary validation now has stable spec-local assets**: `./scripts/check-no-std-core.sh` is the umbrella rail for the first-wave functional-core boundary. It validates the main `functional-core` spec, runs host + `wasm32-unknown-unknown` checks, focused core/adapter tests, and the five boundary scripts (`deps`, `purity`, `scope`, `api-shape`, `ownership`). The dependency checker reads `openspec/specs/functional-core/validation/deps-allowlist.txt`, the ownership review lives at `openspec/specs/functional-core/evidence/ownership-review.md`, and the umbrella script must also work in rustup-less environments by accepting a preinstalled wasm target under the active `rustc` sysroot.
- **Second-wave no-std validation inventory is now centralized**: `openspec/specs/functional-core/validation/adopted-core-inventory.toml` is the single checked-in source of truth for adopted core crate dirs, lib/manifests, required exports, first-wave legacy std files, and required second-wave std adapter files. `scripts/no_std_core_checks.py` should derive core-directory filtering from that inventory too; if it still excludes only first-wave core dirs, `crunch-shell-core` and `crunch-release-core` source files get misclassified as touched std workspace files in ownership review.
- **Current third-wave `crunch-delta` split**: `crates/crunch-delta-core/src/{model,negotiation,planner}.rs` now own the normalized no-std logic over `DeltaDigest`, ordered `BTreeSet`, deterministic `Vec`, and typed errors. `crates/crunch-delta/src/{model,negotiation,planner}.rs` must keep the std-facing façade, not just `pub use` the core types: public `ReceiverManifest`/node digests stay on `HashSet<B3Digest>` / `B3Digest`, and std-only `impl std::error::Error` wrappers for `PlanError` / `NegotiationError` live in `crunch-delta`, not the core. `crates/crunch-delta/src/{fixtures,manifest,substitution}.rs` stay std-owned fixture-building and async store/network shell code. Boundary proof tests now live at `crunch-delta::substitution::tests::substitution_adapter_keeps_async_store_and_network_in_shell` and `crunch-delta::planner::tests::delta_facade_reexports_core_planner_types`.
- **Deterministic runner-branch proof for the no-std rail lives in `scripts/test-check-no-std-core-runner.sh`**: the harness copies the runner into a temp dir and fakes `rustup`/`cargo`/`openspec` so `bootstrap-wasm-target` proves `rustup target add ...` happens before cargo checks, while `missing-wasm-target` proves the runner exits before validation work when wasm support is unavailable. Keep a real `bash` directory in the fake PATH or the copied `#!/usr/bin/env bash` runner fails before the probe starts.
- **`openspec status --change` is artifact-only**: it reports whether proposal/design/specs/tasks files exist, not whether `tasks.md` is fully checked. A change can show `Progress: 4/4 artifacts complete` while still having unchecked task boxes, so grep `tasks.md` before treating it as ready to archive.
- **Archive OpenSpec only after implementation commit is ready**: if `openspec archive` lands before the code/docs/scripts commit, `openspec/specs/...` can assert requirements that no committed tree satisfies yet. Commit the implementation first, then archive (or squash/reorder before sharing history).
- Piping `openspec archive -y <change>` through `tee` can crash with Node `write EPIPE` after the archive already succeeded. Run it directly, then inspect repo state separately.
- After syncing a `MODIFIED` delta into a main spec, re-read the rebuilt main requirement. The sync can replace the old body and accidentally drop unrelated clauses/scenarios that still apply.
- **PathInfoService list+put hazard**: `crunch-store::store_sign()` must not call `svc.put()` while iterating `svc.list()` on the same backend. `LruPathInfoService` can hang in that pattern. Collect matching `PathInfo`s first, then persist updates in a second pass.
- **Store fallback policy is now mode-dependent**: `StoreConfig.fallback_mode` threads practical vs strict handling into `crunch-store`. If `pathinfo.redb` cannot be opened, practical mode falls back to in-memory PathInfo and reports `StoreAuditKind::PathInfoFallback`; strict mode raises `Error::PathInfoFallbackRejected`, and `crunch-pipeline` turns that into a preflight root failure with no degraded audit event. Closure walking now returns `ClosureResolution { paths, audit_events }`; practical mode records `StoreAuditKind::ClosureResolutionDegraded` for missing closure facts, while strict mode raises `Error::MissingClosureFacts` before sandbox start.
- **Build-envelope hermeticity now lives in `crunch-build`**: `HermeticityMode`, `HermeticityAuditKind`, and `HermeticityAuditEvent` are defined in `crates/crunch-build/src/hermeticity.rs` and re-exported from `crunch-pipeline`. `derivation_to_build_request(...)` now takes `HermeticityMode` and returns a `BuildRequestEnvelope { build_request, audit_events }`, so public callers must choose a mode explicitly and audits are surfaced on the public API instead of being dropped inside the helper. `Builder` defaults to `Practical`; pipeline/CLI callers that want strict env enforcement must call `builder.set_hermeticity_mode(...)` and drain `builder.take_hermeticity_audit_events()` after the worker run. `normalize_build_environment()` in `crates/crunch-build/src/build_request.rs` is the enforcement hook for protected sandbox vars, and `vendor/snix-build/src/bwrap/mod.rs` now fixes the child umask to `0022` with a `pre_exec` hook.
- **`crunch-eval::Error` display must render Nickel diagnostics, not `{:?}`**: formatting `NickelError` through the derived `#[error("...")]` debug path can stack overflow on contract failures (`tests::eval_error_display_has_context`). Use `NickelError::format(..., nickel_lang::ErrorFormat::Text)` and fall back to a plain summary only if diagnostic rendering itself fails.
- **Signed PathInfo invariant**: `StoreHandle::persist_and_export_signed_output()` rejects unsigned `PathInfo`s and store-path mismatches. `crunch store sign --all` is a migration path for unsigned entries only; already-signed entries must be left unchanged.
- **Store GC state now has three durable pieces**: `state_dir/pathinfo.redb`, `state_dir/directories.redb`, and `state_dir/gc-roots.json`. GC could not safely survive restart while directories lived only in a temporary RedbDirectoryService; reachable directory roots now require the persisted `directories.redb` file.
- **Store mutation lock**: `state_dir/store-mutation.lock` is the cross-process guard for local store mutation. `crunch_pipeline::build()` acquires it for the full build, `crunch bootstrap --fetch` holds it around fetch + reduction, mutating `crunch store` commands use it, and `crunch store gc` uses a try-lock so it fails fast when another local build/substitution/store mutation is active.
- **Configurable store prefix**: `--store-prefix /mantle/store` (default),
  explicit `/crunch/store` for legacy compatibility, or `--nix-compat` for `/nix/store`. The prefix flows through
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
- **Current performance levers**: `crates/crunch-pipeline/src/lib.rs::build()`
  now uses `crunch_eval::evaluate_and_extract_named_roots()` instead of a
  whole-program JSON export. That direct path only works because
  `crates/crunch-glue/src/types.rs::Input` uses a manual `Deserialize`
  (`deserialize_any` + raw-record dispatch) instead of `#[serde(untagged)]`,
  which Nickel direct deserialization rejected for nested derivation inputs
  with enum-tag fields. `crates/crunch-build/src/orchestrate.rs`
  `resolve_and_ingest_sources()` now memoizes source-closure walks per build
  session and uses `crunch_store::StoreHandle::cached_node_for_path()` before
  falling back to `ingest_path(...)`, so repeated source inputs and locally
  known dependency outputs stop re-walking PathInfo and stop re-ingesting the
  same on-disk tree. `crates/crunch-store/src/closure.rs` now asks
  `PathInfoService::get_references()` for closure metadata; the default trait
  implementation still falls back to `get()`, but vendored
  `NixHTTPPathInfoService` overrides it to parse and verify `.narinfo`
  signatures without downloading the NAR payload. `crates/crunch-build`
  worker hot paths now share `Arc<Derivation>`, move the full `BuildRequest`
  straight into the spawned build task, and drain waiter vectors with
  `mem::take()` instead of cloning them. Remaining hot spots are now outside
  those paths (for example deeper scheduler structure and remote metadata
  transport itself), not the old clone-heavy ready/dispatch flow. New parallel
  root-forcing boundary: `crates/crunch-eval/src/session.rs` shares only
  `IsolatedWorkerInput` (source text + import paths + source name) across
  threads. Raw Nickel / `crunch_eval::Error` values are not `Send`, so worker
  threads must stringify failures before crossing the thread boundary; do not
  try to return raw `crunch_eval::Error` or share a live `EvaluationSession`
  across threads.
- **Delta substitution integration boundary**: `crates/crunch-store` cannot
  import `crates/crunch-delta` directly because `crunch-delta` already depends
  on `crunch-store`. The current runtime integration slice therefore lives in
  `StoreHandle::try_substitute_remote()` as same-authority
  `/delta/capabilities` probing, advertisement validation, a same-authority
  candidate-request POST, a bounded local has-set POST built from receiver
  `PathInfo` + castore presence, NDJSON stream-frame decode, chunk/blob frame
  ingestion into local castore, and final signed-`PathInfo` acceptance through
  `persist_and_export_signed_output()`. Current wire gap: `DeltaArtifactNode`
  still has no directory-entry names, so crunch-store can reconstruct blob or
  symlink roots directly, and can only accept directory roots when the full
  directory closure is already present locally by digest; new directory shapes
  still require fallback. Query-string gotcha: substituter URLs may carry
  `trusted_public_keys[...]`; `normalized_cache_base_url()` must clear query /
  fragment before joining delta endpoints so trust config does not leak into
  `/delta/*` requests. Reporting note: successful remote substitutions now
  record per-output mode/bytes/fallback data on `StoreHandle`; when a full
  substitution's local castore tree is incomplete (common in in-memory remote
  test fixtures), report `transferred_bytes` falls back to `PathInfo.nar_size`.
  Test harness note: `test_handle_with_remote()` uses an in-memory
  `LruPathInfoService`, so fallback-path tests there prove crunch-store
  decision logic but do NOT exercise `NixHTTPPathInfoService` narinfo
  signature verification on the final full-fetch path. Stronger fixture note:
  `remote_substitution_stream_failure_falls_back_through_real_http_cache`
  now seeds a real store, persists a signed output into
  `state_dir/pathinfo.redb` + `state_dir/blobs`, reopens that state, renders a
  real NAR from reopened castore data, and serves it over a tiny HTTP server to
  exercise `NixHTTPPathInfoService` trust + fallback end to end. Cheapest
  closure-scoped delta e2e pattern: build a candidate whose `sender.outputs`
  includes the requested output plus one sibling output already present
  locally, make the requested output reuse a chunk/blob reachable from that
  sibling, and assert the receiver manifest advertises only that sibling within
  closure scope while final reporting still records a delta hit. Keep the
  repo-local deterministic compatibility test between crunch-store wire
  constants and `crunch-delta`'s protocol-v1 helpers when changing
  chunk-profile or endpoint defaults.
- **BLAKE3 everywhere**: `HashAlgo::Blake3` + `NixHash::Blake3` in nix-compat,
  `NAR_BLAKE3`/`FLAT_BLAKE3` in pathinfo.proto, blake3 branches in
  `nar_hash()`/`hash_blob()`/`verify_flat_hash()`/`HashingReader`. The
  `blake3::Hasher` doesn't implement `digest::DynDigest` — use the
  `Blake3Wrapper` shim in `snix-store/src/nar/hashing_reader.rs`.
- **No protoc required**: gRPC/protobuf replaced with postcard serialization.
  Proto-generated types are now plain Rust structs with serde derives.
  Directory digests use postcard encoding (not protobuf canonical form).
  Existing redb databases are incompatible after this change.

## crunch-shell Crate (2026-04-22)
- `crates/crunch-shell-core/` is the second-wave no-std shell activation core: sidecar JSON validation plus env/path/hook planning over plain `String`/`BTreeMap` data.
- `crates/crunch-shell/` is now the std adapter: it keeps `PathBuf`/`OsString` shell-facing types, derives host PATH entries with `split_paths`, maps them into the core, then restores `ExecTarget` and `PathBuf` plans for `src/shell_cmd.rs`.
- The compiler-enforced boundary is intentionally UTF-8-only at the core edge. Adapter tests prove non-UTF-8 `--with` paths fail loudly with `ShellError::NonUtf8Path` instead of silently lossy-converting them.

## crunch-release-core Crate (2026-04-22)
- `crates/crunch-release-core/` is the no-std functional core for release-evidence manifests: canonical compact JSON, bundle-member/path validation, proof-linkage validation, and full-proof manifest parsing all live on owned `String`/`Vec` data with `serde`/`serde_json` in `alloc` mode.
- `src/release_evidence.rs` is now the std shell/adaptor: it keeps file copying, directory hashing, bundle I/O, and proof-manifest digesting on the shell side, then delegates manifest/proof validation to `crunch-release-core`.
- Cheap validation rail for this slice: `cargo test -p crunch-release-core`, `cargo check -p crunch-release-core --target wasm32-unknown-unknown`, and `cargo test -p crunch --bin crunch release_evidence::` with an isolated `CARGO_TARGET_DIR` under `/tmp`.

## crunch-project Crate (2026-04-07)
- `crates/crunch-project/` owns manifest, lock, refresh, stale detection, upgrade, drift, mirrors, and generated inputs.
- No Nickel dep — the binary crate uses `crunch-eval::evaluate_and_deserialize()` to load `crunch-project.ncl` into `ProjectManifest`.
- `RefreshResolver` trait: callers implement network-dependent resolution (git ls-remote, content hashing). The crate itself is pure logic.
- Tarball input hashes in `crunch.lock` / `.crunch/inputs.ncl` must match `crunch.fetchTarball`: recursive/NAR hash of the unpacked tree, not a flat hash of the downloaded archive bytes.
- Local patch locking also depends on the resolver: `apply_outcomes()` calls `RefreshResolver::hash_local_file()` for `PatchSource::Local`, so a "live" resolver that only implements git + URL hashing still leaves patch locks unresolved.
- Generated `.crunch/inputs.ncl` is a plain Nickel record (no stdlib import). Package code imports it and passes data to fetch helpers.
- Generated inputs now fail fast if an input references a patch name that is missing from `Lockfile.patches`, and quoted Nickel field names must escape embedded quotes in input/patch names.
- `crunch.lock` is JSON with explicit `SchemaVersion`. Upgrade path: 0.9.0 -> 1.0.0 (structural noop, exercises the migration machinery).
- CLI commands: `init`, `check`, `show`, `refresh`, `list-stale`, `upgrade` — all in `src/project_cmd.rs`, delegating to `crunch-project`.
- Live refresh I/O stays in the binary crate (`src/project_resolve.rs`), not in `crunch-project`. `RefreshResolver` now distinguishes flat vs recursive URL hashing and has a separate git-checkout hash hook.
- `crunch refresh` writes successful lock/input updates even when sibling inputs fail, but exits non-zero on any input or patch resolution failure. `crunch list-stale` prints stale inputs on stdout, failed checks on stderr, and exits non-zero if any check failed.
- Tarball and git lock hashes must be recursive/NAR hashes of the unpacked tree / checked-out work tree. Plain files and local/remote patches use flat content hashes. There is no fallback to manifest `expected` values during refresh.
- Test count: 90 (85 unit + 5 integration using crunch-eval for Nickel validation).

## crunch-attestation Crate (2026-04-10)
- `crates/crunch-attestation/` now holds the pure Phase-1 native attestation core: schema types, canonicalization, and BLAKE3 digesting only. No store/build/CLI I/O belongs here.
- The crate models claims and observed facts separately for artifact, closure, and project attestations.
- Canonical bytes are stable JSON over normalized structs: nodes, edges, roots, aliases, and referenced artifact digests are sorted before serialization, so insertion order and closure discovery order do not affect the digest.
- `Canonicalize` provides both `canonical_bytes()` and `canonical_digest()`. Digest fields use a dedicated `AttestationDigest` type serialized as 64-char lowercase hex.
- `SchemaVersion` is a nonzero `u32` newtype with `V1` as the default.
- Closure canonicalization must validate root node kind against the node set, not just membership in the members list. Tests now cover duplicate nodes, invalid artifact/closure/project roots, empty required fields, and collection limits.

## Provenance Storage (2026-04-10)
- `crates/crunch-store/` now persists artifact attestations under `<state>/attestations/artifacts/*.json`, keyed by a BLAKE3 hash of the logical store path string.
- Runtime closure attestations are cached under `<state>/attestations/closures/*.json`, keyed by sorted root logical paths plus closure semantics.
- Persisted attestation files must be the canonical attestation bytes themselves, not pretty JSON wrappers. `crates/crunch-store/src/attestation.rs` writes `canonical_bytes()` and recomputes digests on load.
- Closure attestation lookup must not trust the cached closure file blindly. Member artifact attestations can be rewritten in place (for example `_unknown` synthesized members later replaced by real output metadata), so `load_or_create_runtime_closure_attestation()` recomputes the fresh closure and rewrites the cached file when member digests change.
- `StoreHandle::persist_and_export_signed_output()` now takes `output_name` so the stored artifact attestation records the correct output label.
- `StoreHandle` synthesizes artifact attestations from `PathInfo` on successful build persistence, local cache hits, and remote substitution hits. Closure assembly must also synthesize a missing member artifact attestation from `PathInfo` instead of failing on a missing file.
- Cache/substitution trust must dedup verifying keys by full key material, not just key name. Fresh local stores reuse the same generated key name (`crunch-<hostname>-1`), and name-only dedup drops an explicitly trusted remote signer so substitution silently falls back to local builds.
- Remote substitution of a root output must export the substituted castore node to the physical `--store` path when the file is missing, not just persist `PathInfo` and keep the node in memory. Otherwise `--json build`, `crunch attest show`, and CLI follow-up checks see a missing output on disk.
- `crates/crunch-build/src/orchestrate.rs` now threads declared source inputs and input-artifact outputs into artifact attestation generation via `ArtifactProvenance`, so successful local builds record `build-input` and `fetched-from` edges.
- Builder-layer provenance claims now flow from `builders/mk_derivation.ncl` through `CrunchDerivation.provenance` into registry entries and final artifact attestations. Those claims are ignored by `crunch-glue::convert()` when constructing the hashed `nix_compat::Derivation`, so changing claims does not change derivation hashes by default.
- The closed core derivation contract in `lib/derivation.ncl` still rejects a `provenance` field; only the builder layer exports it.
- `crates/crunch-project/src/attestation.rs` now synthesizes canonical project attestations from `crunch-project.ncl` + `crunch.lock` content digests, lock-resolved source facts, mirrors, patch records, and selected root artifact references. Patch edges are `source --patched-by--> patch`; project membership edges are `node --declared-by-project--> project`.
- `crunch_store::artifact_attestation_file_path()` and `crunch_store::closure_attestation_file_path()` are the stable helpers for surfacing persisted sidecar locations.
- `src/attest_cmd.rs` owns the CLI surface for `crunch attest show|closure|verify|diff|project`. It prints envelope JSON (`kind`, `digest`, optional `stored_path`, `attestation`) for show/closure/project, verifies persisted artifact/closure sidecars by byte-for-byte comparison with canonical reconstruction, resolves `diff` inputs as artifact selectors before falling back to JSON files (so existing exported store paths don't get misread as JSON), and requires `crunch attest verify project` to compare the fresh reconstruction against either `--file <saved-envelope.json>` or `--digest <hex>`.
- `crunch release create` auto-builds its staged-source tar from current tracked worktree files with `git ls-files -z` plus live file reads, explicitly adds verified `vendor-deps/` files even when the directory is Git-ignored, and must include `scripts/prove-self-hosting.sh`, `docs/bootstrap-stage0-inventory.md`, `tests/self_hosting.rs`, and `tests/audit_support.rs` because witness rebuild extracts the release source archive and executes that checked-in workflow driver via `cargo test -p crunch --test self_hosting`. It does NOT accept a user-supplied source archive anymore and does NOT read stale `HEAD` content. The command still depends on running inside a git checkout whose tracked files overlap the self-build allowlist and whose source-tree vendor inputs pass `require_checked_vendor_inputs()`. Because the release source archive comes from the live worktree, rerun the self-hosting proof after any release-source or witness-rebuild code changes before packaging; otherwise the bundled binary digest can describe an older source tree than the witness rebuild sees.
- Fresh release-evidence validation succeeded on 2026-04-17 after fixing two regressions: `crates/crunch-build/src/fetcher.rs` now rebases safe absolute in-tree tar symlinks (unblocking musl.cc `lib/ld-musl-x86_64.so.1 -> /lib/libc.so`), and `crates/crunch-build/src/orchestrate.rs` now records cached CA output paths back into `DerivationRegistry` so later dependent builds do not fail with `output has no store path`. Proof evidence: pueue task `69` (`release-evidence-full-proof-rerun4`) with `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 41 filtered out; finished in 3079.20s`, proof bundle `target/self-hosting-proof/release-evidence-fresh5`, and bundle-local verification via `crunch release verify target/release-evidence/release-evidence-fresh5`.
- Current decentralized-release-verification seam: the pure core is already in place (`crunch-attestation-core` release/witness attestation schemas + policy evaluation, plus `crunch-attestation::discovery::VerificationDirectory` for `release-attestation.json`, `witnesses/*.json`, `policy.json`, and optional `revocations.json`). The shipped CLI now covers the first operator slice: `crunch release attest <bundle-dir> [--verification-dir ...] [--signing-key ...]`, `crunch attest key-show [--signing-key ...]`, `crunch attest policy-init <verification-dir> --profile <...> --trusted-release-signer <name>... [--trusted-witness-identity <identity>...] [--force]`, `crunch attest witness-create <verification-dir> --rebuilt-binary <path>... --system <label> --toolchain <label> --host-class <label> [--identity ...] [--signing-key ...]`, `crunch attest release-show <verification-dir>`, `crunch attest witness-show <verification-dir> [identity]`, and `crunch attest release-verify <verification-dir> --trusted-public-key <name:base64>...`.
- Important trust-model nuance for the new release-attestation CLI: `policy.json` still governs which release signer names and witness identities count socially, but it does NOT carry Ed25519 key material. Cryptographic verification comes from `--trusted-public-key` (or configured `trusted-public-keys`) keyed by detached-signature name. Current CLI treatment of `trusted_witness_signers` is identity allowlisting (`witness_identity`), not raw signature-key matching.
- `crunch attest key-show` is intentionally side-effect free: it reuses the explicit/default signing-key lookup path but MUST fail on a missing key instead of auto-generating one, because key inspection cannot mutate trust roots. Human output is the raw verifier token (`name:base64`) for direct shell capture; JSON output also names the source path.
- Fresh auto-generated signing keys in different `CRUNCH_CONFIG_DIR`s can reuse the same signer name (`crunch-<hostname>-1`). `release-verify` therefore must not stop at the first trusted key whose name matches the detached signature; it has to try every trusted key with that signer name and accept the first one whose signature check succeeds. Otherwise witnessed workflows with separately generated publisher/witness keys silently drop valid witnesses back to `self-proof-valid`.
- Cross-machine witness handoff is now a checked-in CLI workflow: publisher runs `crunch release witness-export <bundle-dir> [--verification-dir ...] [--request-dir ...]` to copy only public verification material (verified release-evidence bundle + signed release attestation + `request.json` metadata, no keys/policy), the witness replays that request through `./scripts/rebuild-witness-request.sh <request-dir> ...` / `crunch release witness-rebuild <request-dir> ...`, and the publisher runs `crunch attest witness-import <verification-dir> <source>` to merge returned sidecars from the scratch verification dir (`<request-dir>.work/release-verification/<release-id>` by default). `witness-rebuild` must preserve `ReleaseProofLinkage.proof_mode`; non-Nix-host release proofs need the helper `--non-nix-host` argument during replay. `witness-import` accepts either a single witness `.json` file or a directory/verification dir containing `witnesses/`, fails closed on missing `.sig`, release-digest mismatch, or conflicting duplicate identity, and treats exact duplicate bytes as idempotent.
- Release-verify witness handling is intentionally tolerant of mixed witness sets: unknown-key or bad-signature witnesses are counted as discovered but skipped before policy evaluation, so a verifier can still reach quorum with the subset signed by trusted keys. Keep CLI coverage for both the unknown-key skip path and the wrong-release-digest policy-insufficient path.
- `crunch release witness-rebuild` derives a deterministic plan from `request.json`, the copied release-evidence bundle, and the copied `release-attestation.json`; it only accepts the recorded workflow identity pair `./scripts/prove-self-hosting.sh` + `crunch-self-hosting-proof-v2`, keeps the request dir immutable, writes sidecars plus `witness-rebuild-audit/meta.json` under the scratch root, and rejects attestation/manifest output-name drift before launching the rebuild workflow.
- `./scripts/rebuild-witness-request.sh` pre-creates helper-owned `tmp/` and `cargo-target/` directories under the chosen scratch root before it invokes `crunch release witness-rebuild`. The Rust scratch validator therefore must allow exactly those two root entries (and still reject everything else, especially symlinks), or helper-driven rebuilds fail before the workflow driver starts.
- `tests/release_cli.rs` `make_valid_bundle()` still uses a fake proof bundle, so real witness-rebuild CLI coverage uses the hidden `CRUNCH_WITNESS_REBUILD_DRIVER` seam to supply a tiny script that writes a proof-bundle-shaped `manifest.json`; do not try to run the real self-hosting proof helper in those tests.
- The fake witness-rebuild driver seam must consume a launch signal path (`CRUNCH_TEST_WITNESS_DRIVER_LAUNCH_SIGNAL`) and write it as its first side effect. Happy-path witness rebuild tests should assert the signal exists; preflight-failure tests should assert it stays absent so “driver never launched” cannot pass vacuously.
- `crunch attest witness-create` maps `--rebuilt-binary` arguments onto the published `release-attestation.json` binary list by positional order, reuses the published binary names in the signed payload, defaults `witness_identity` to the signer key name when `--identity` is omitted, and rejects count mismatches before writing any witness sidecars.
- `crunch attest witness-create` now matches `crunch release attest` output style: human summary on stdout by default, full JSON envelope only under global `--json`. Its `witness_identity` is also part of the filesystem surface (`witnesses/<identity>.json`), so the shell validates control characters and caps the UTF-8 byte length before writing sidecars.
- For post-proof release-evidence packaging, the proof helper can clean the final `.tmp*/store/...-crunch/bin/crunch` path before later CLI runs. The stage2 binary still survives under `target/self-hosting-proof/work-.../tmp/crunch-builds/*/scratches/nix/store/*-crunch/bin/crunch`; use that preserved scratch copy as the `crunch release create --binary ...` input.
- `src/build_report.rs` uses the artifact path helper so JSON build outputs now carry a separate `artifact_attestation { logical_path, path }` reference block.
- `StoreHandle::get_artifact_attestation()` and `StoreHandle::runtime_closure_attestation()` are the retrieval entry points.
- The cheapest real substitution e2e for attestations is: build once locally, reopen the signed `PathInfo` from `state_dir/pathinfo.redb`, render a NAR from `state_dir/blobs`, serve that `.narinfo` + NAR from a tiny local HTTP server, then rebuild in a fresh `state_dir` with `--substituters <url>` and the first build's verifying key. That exercises `NixHTTPPathInfoService`, remote sidecar persistence, and `crunch attest verify artifact|closure` without an external cache.

## System-config VM seam (2026-04-23)
- `crunch system eval --stop-after fragments` exposes per-machine `machines.<name>.merged_config.data.output.nixos` on stdout (see `tests/system_cli.rs`). That is the natural VM-smoke bridge for the current phase-1 pipeline because it exercises live loader/evaluator/collector behavior without requiring the build path.

## Dependency Audit

- Root `cargo deny` policy now lives in `deny.toml`; without it, `cargo deny check` falls back to a default config and produces a giant wall of license rejections that hides the real advisory set.
- `cargo-deny` 0.18 uses the newer config schema (`[graph]`, `[output]`, advisory `ignore = [{ id, reason }]`). Old template fields like `unmaintained = "deny"` no longer parse.
- Vendored workspace crates under `vendor/` have no `license` fields. To keep license checks honest without inventing licenses, mark them `publish = false` and keep `licenses.private.ignore = true` in `deny.toml`.
- Direct dependency-audit wins on this tree were straightforward lock/manifest bumps: `astral-tokio-tar -> 0.6.0`, `lru -> 0.16.4`, `rand -> 0.8.6/0.9.3/0.10.1`, `rustls-webpki -> 0.103.12`, `fastrand -> 2.4.1`.
- `vm-memory` / `vmm-sys-util` are NOT isolated to a dead optional path here. `cargo tree -e features -i vm-memory` shows them in the shipped `snix-castore -> fuse-backend-rs -> fusedev` path, but direct upgrades are blocked by the older `vhost` / virtio stack pins. Treat them as upstream-blocked waiver candidates unless you are ready to move that whole stack together.

## Verification Evidence Rules

When claiming test results in commit messages or completion summaries:

- **Run the command in the same tool call** that produces the summary.
  Use `Bash` with a timeout for fast tests, `pueue_run` + `pueue_log`
  for slow ones. The tool output IS the evidence.
- **Never quote test counts from memory** or from a prior session.
  Re-run and show the output.
- **For pueue tasks**, always `pueue_log` the relevant task ID and
  include the `test result:` line in the commit message or summary.
  `pueue_wait` alone proves the task finished, not what it printed.
- **For ignored integration tests** (like `tests/self_hosting.rs`),
  `cargo test --test X -- --list` proves compilation + discovery.
  Only `--ignored --nocapture` with captured output proves execution.
- `crunch --json build ...` now emits a stable `crunch-build-report-v1`
  JSON object on stdout. It includes `hermeticity_mode`,
  `hermeticity_audit_events`, counts, per-root outcomes, failure records,
  and output paths. `tests/smoke.rs` and the new build JSON CLI tests use it
  instead of scraping human stdout. `log_file` fields are optional and must
  only appear when the log file actually exists on disk.
- Audit-grade integration tests now write bundles under
  `target/test-audit/<suite>/.../` with `meta.json`, `stdout.txt`,
  `stderr.txt`, and BLAKE3 digests for produced artifacts. Smoke tests
  and `tests/self_hosting.rs` both use `tests/audit_support.rs`.
- `cargo clippy --workspace --all-targets -- -D warnings` is not a stable
  first-party gate on this repo by itself: vendored workspace members like
  `vendor/fuse-backend-rs` fail first. For first-party lint triage, add
  `--no-deps`, exclude vendored members explicitly, then peel first-party
  crates one by one.
- `fuse-backend-rs` also needs to stay in the explicit first-party clippy
  exclusion list alongside `nix-compat`, `nix-compat-derive`, `snix-build`,
  `snix-castore`, `snix-store`, and `snix-tracing`.
- `cargo fmt --check -p crunch -v` covers the root package's `src/`,
  `examples/`, and `tests/`, including `tests/benchmark_harness.rs`. Use that
  root package leg plus first-party crate package flags instead of
  `cargo fmt --all --check` when vendor/ must stay out of scope.
- Store-pull URL gotcha: keep `crunch store pull --from` as `String` until
  explicit `Url::parse`. On this host `PathBuf::from("http://cache.example.com")`
  stringifies as `http:/cache.example.com`, which silently breaks HTTP cache
  dispatch.
- Current HTTP pull semantics: `import_paths_from_http_cache()` strips query /
  fragment from the cache base URL, rejects URL userinfo credentials, always
  runs `nix-cache-info` preflight before any already-present short-circuit,
  treats only preflight 404 as “missing”, and warns/continues on other
  non-success preflight statuses (for example 401/403/410/5xx) plus redirect /
  transport failures.
- Rustfmt gotcha: formatting crate roots like `src/main.rs` or
  `crates/*/src/lib.rs` can cascade into sibling modules and dirty unrelated
  files. Prefer leaf-file formatting or revert the unrelated churn immediately.
- The new first-party clippy helper assumes the documented build env. Without
  the clang/mold/pkg-config PATH prefix and OpenSSL `PKG_CONFIG_PATH`, it fails
  early with `linker 'clang' not found` before any first-party lint results.
- Tigerstyle consumer integration has two crunch-specific flake gotchas: use
  `src = ./.` for the lint check (not `craneLib.cleanCargoSource ./.`) because
  `crates/crunch-eval/src/stdlib.rs` `include_str!`s `lib/*.ncl`, and set
  `SNIX_BUILD_SANDBOX_SHELL` in the check env (plain `/bin/sh` is enough for
  linting) because vendored `snix-build` uses `env!("SNIX_BUILD_SANDBOX_SHELL")`
  at compile time.

## Self-Build and Self-Hosting Proof

- `scripts/prove-self-hosting.sh` is the checked-in entry point for the self-hosting proof. Run `./scripts/prove-self-hosting.sh --check` to validate the toolchain/linker/pkg-config setup without starting the ~30 minute proof.
- The proof helper now ignores ambient `TMPDIR` / `CARGO_TARGET_DIR`: it picks scratch from `CRUNCH_PROOF_SCRATCH_DIR` when set, else repo-local `target/self-hosting-proof/work/`, rewrites both env vars under that root, and reports the root plus derived `tmp/` and `cargo-target/` paths in startup diagnostics. It also pins `RUSTC` to the post-PATH rustc and clears rustc wrapper/rustflags environment so witness rebuilds do not inherit stale sccache or proof-path rustc shims from the caller.
- `tests/self_hosting.rs` proof-helper script tests should serialize child script launches with `lock_proof_env()`. Without that guard, parallel fixture runs on this host can hit `Text file busy` when executing temp-copied `prove-self-hosting.sh` files.
- `docs/bootstrap-stage0-inventory.md` is the stricter first-bootstrap trust inventory. README/self-hosting docs must distinguish the default fixed-point proof from the stricter `./scripts/prove-self-hosting.sh --non-nix-host` mode, which now scrubs `nix-build`, `nix-store`, `nix-shell`, and `nix` from the proof-runner `PATH` before `cargo test` while keeping the same stage1==stage2 fixed-point check.
- `src/self_build.rs` stage0 source staging is now pure Rust: it copies a fixed allowlist of top-level repo entries (`.cargo`, `Cargo.{toml,lock}`, `bootstrap`, `builders`, `crates`, `lib`, `rust-toolchain.toml`, `src`, `vendor`, `vendor-deps`) and requires the checked-in `.cargo/vendor-config.toml` to point at `vendor-deps`. No non-test self-build path shells out to `git`, `tar`, `sh`, `cp`, or `cargo vendor` anymore. Keep NixOS-specific path probes labeled as host convenience, not proof evidence.
- On this host Cargo still builds to the shared `~/.cargo-target/` by default. For real self-build validation, run `/home/brittonr/.cargo-target/debug/crunch ...`, not the stale repo-local `target/debug/crunch`, unless you explicitly set `CARGO_TARGET_DIR=target`.
- `crates/crunch-glue/src/convert.rs::resolve_inputs()` must parse source inputs with the configured store prefix. Using `StorePath::from_absolute_path(...)` there broke stage3 self-build under the default `/crunch/store` prefix even after earlier CA-mapping and Nickel contract fixes; `StorePath::from_absolute_path_with_prefix(..., known_paths.store_dir())` fixed the remaining `/crunch/store/...-busybox` failure.
- End-to-end validation now succeeds with the rewritten stage0 staging path and the default logical store prefix. Successful evidence command on this host:
  `/home/brittonr/.cargo-target/debug/crunch self-build --store /tmp/crunch-stage0-self-build-final-store --state-dir /tmp/crunch-stage0-self-build-final-state --no-substitute -j 4 --verbose --log-level info`
  with `PATH` including `/nix/store/csk28n2yj6pzwkslf3mn2gdxz33xxazr-bubblewrap-0.11.0/bin`, `PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig`, and `SNIX_BUILD_SANDBOX_SHELL=/nix/store/d7fc5i7y71rj8cr5jwmaxwjnyvfiybdp-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox`. The successful output root was `/tmp/crunch-stage0-self-build-final-store/xsrdwvrp6nlkr5xhxqiw9fnmqxs0i4vk-crunch`.
- Successful proof runs now write a shareable bundle under `target/self-hosting-proof/run-*` and refresh `target/self-hosting-proof/latest`. The helper must honor `CRUNCH_SELF_HOSTING_PROOF_BUNDLE_DIR` when `--bundle-dir` is absent because `crunch release witness-rebuild` launches `scripts/prove-self-hosting.sh` that way and then reads `<scratch>/proof-bundle/manifest.json`; if the helper ignores the env, the rebuild can pass internally but fail afterward with `reading .../proof-bundle/manifest.json: No such file or directory`. Relative `--bundle-dir` values are anchored to the repo root before export or `latest` updates.
- `--json` CLI mode must keep stderr machine-readable by default. `src/main.rs` now leaves tracing uninitialized unless `--verbose`, `--log-level`, or `RUST_LOG` requests logs; otherwise dependency WARN lines from `redb` / `snix_build` can corrupt JSON stderr and break tests like `tests/smoke.rs::smoke_build_failure_reports_error`.
- `./scripts/prove-self-hosting.sh --check` is only a host-prereq preflight, not proof evidence. On 2026-04-10 the full proof initially still failed in stage0 while building `busybox.drv` with `bwrap: execvp /bin/sh: No such file or directory`; use the reported audit bundle + `stage0-diagnostics.txt` when triaging, and do not archive proof-related changes from `--check` alone.
- `scripts/prove-self-hosting.sh` must resolve a real `busybox-static` for `SNIX_BUILD_SANDBOX_SHELL` when the env is unset or `/bin/sh`. A `bash-static` fallback is not enough for the proof because the sandbox also bind-mounts the shell at `/bin/busybox`, and bootstrap scripts expect busybox applets like `mkdir`, `ln`, and `chmod`. The helper scans installed `busybox-static` paths first and now FAILS fast if none is available; it no longer realizes `pkgsStatic.busybox` through `nix-build`.
- Fast preflight for vendored stage0 source completeness: `cargo metadata --offline --locked --format-version 1 --config .cargo/vendor-config.toml` must pass. If it says `no matching package named ... found` under `vendor-deps/`, the next self-hosting proof will die inside stage0 when the sandboxed cargo build reaches that missing vendored crate.
- The fetched bootstrap Rust toolchain in `bootstrap/rust.ncl` is currently stable `rust-1.94.1`, but the workspace still uses nightly-only `#![feature(register_tool)]` for tigerstyle lint namespaces. The generated self-build derivation therefore needs `export RUSTC_BOOTSTRAP=1` before `cargo build`, or stage0 fails on crates like `crunch-store` with `E0554: #![feature] may not be used on the stable release channel`.
- The helper does not require a rich login shell PATH: it falls back to `~/.rustup/toolchains/` for nightly cargo/rustc and scans common NixOS locations (`/run/wrappers/bin`, `/run/current-system/sw/bin`, `/nix/store/*-clang-wrapper-*`, `/nix/store/*-mold-*`, `/nix/store/*-pkg-config-wrapper-*`, `/nix/store/*-openssl-*-dev/lib/pkgconfig`).
- `df -Pk "$TMPDIR"` can hang on this host even in `--check` mode. `scripts/prove-self-hosting.sh` now uses `stat -f -c '%a %S'` for the free-space probe instead.
- On this host bootstrap source fetches can fail transiently with `Network unreachable`, `http status: 502`, or `failed to lookup address information: Try again` even when a retry succeeds seconds later. `crates/crunch-build/src/fetcher.rs` now retries bounded transient fetch failures; keep retry classification focused on transport/server errors and never retry fixed-output hash mismatches or 4xx permanent misses.
- On this host `ftp.gnu.org` can hang or return `Network is unreachable` during stage0 bootstrap fetches even when other sites work. The checked-in bootstrap GNU tarball URLs now use `https://ftpmirror.gnu.org/...` instead of `https://ftp.gnu.org/gnu/...`; keep new GNU source URLs on the mirror front door unless you have evidence the direct host is reliable again.
- `cmd_self_build` has 4 steps: [1/4] stage source, [2/4] build
  bootstrap tools (bwrap.ncl + busybox.ncl as separate roots),
  [3/4] build crunch, [4/4] verify.
- Step [2/4] exports bwrap and busybox to the `--store` directory on
  disk. Without this, they only exist in castore/PathInfo (intermediates
  are not exported). Missing bwrap.ncl or busybox.ncl is a hard error.
- After step [2/4], self-build must use the exact bwrap/busybox output
  paths returned by the bootstrap-tool root builds, not a fresh `read_dir`
  scan of the output store or a `$NIX_STORE/*-bwrap` / `*-busybox` scan in the
  generated build script. Shared proof stores can contain older siblings, and
  globbing can make the proof report one tool path while the crunch build uses
  another.
- The stage0 seed provider is centralized in `bootstrap/seed.ncl`.
  Checked-in bootstrap derivations and `src/self_build.rs::generate_self_build_ncl()`
  must both import that module; don't re-inline the fetched seed in one path
  or seed swaps drift between normal bootstrap and self-build.
- `bootstrap/seed.ncl` now exposes a normalized wrapper toolchain
  (`musl-seed-toolchain`) around the raw musl.cc tarball. Bootstrap stages
  should consume the normalized contract: target-prefixed binutils in `bin/`,
  headers at `<target>/include`, and `libgcc_s.so*` under `<target>/lib`.
- Seed reduction milestone: `bootstrap/seed.ncl` now drops locale catalogs,
  Fortran payload, gcov/LTO helpers, and gold/profile extras from the public
  provider output, and writes provenance to
  `share/crunch-bootstrap/provider.json` inside the store path.
- Keep the provider metadata schema aligned between `bootstrap/seed.ncl` and
  `src/bootstrap.rs`: `reduction.raw_size_bytes`,
  `reduction.reduced_size_bytes`, `reduction.retained_tools`,
  `reduction.dropped_components`, plus top-level `notes`.
- `crunch bootstrap --fetch` must stay host-shell-free: fetch the raw musl.cc
  tarball through `FetchBuildService`, then reduce it on the host in Rust.
  If fetch bootstrap starts trying to build `musl-seed-toolchain` through bwrap,
  it regresses to needing a static sandbox shell before bootstrap even starts.
- `bootstrap/seed.ncl` runs under `/bin/sh` with only `/bin/busybox` mounted.
  Use `$BB cat` / `$BB cp` / `$BB rm` etc. A plain `cat > ...` in the seed
  derivation fails during self-build with `/bin/sh: cat: not found`.
- musl.cc's raw tarball has unprefixed binutils (`bin/ar`, `bin/ld`,
  `bin/ranlib`, ...) but no `bin/x86_64-linux-musl-ar`. The normalized
  seed must materialize target-prefixed copies explicitly; do not rely on
  self-referential symlinks there. A failed stage0 self-hosting run showed
  musl invoking `/nix/store/...-musl-seed-toolchain/bin/x86_64-linux-musl-ar`
  and getting `No such file or directory`.
- `bootstrap/gcc.ncl`'s C++ wrapper path must stay aligned: the script writes
  `/tmp/tools/seed-cxx-static`, so `configure` must use that exact path for
  `CXX`. If it points at a stale name like `/tmp/tools/musl-g++-static`, GCC
  configure fails with `A compiler with support for C++11 language features is required.`
- `bootstrap/bwrap.ncl` and `bootstrap/busybox.ncl` need raw kernel headers
  from the normalized seed sysroot in addition to musl libc headers. If stage0
  bwrap fails on `<linux/capability.h>` / `<linux/loop.h>` or busybox misses
  `linux/*.h`, fix `bootstrap/seed.ncl` so `<target>/include` has the headers;
  downstream stages should include only `-I$SEED_ROOT/<target>/include`.
- `bootstrap/seed.ncl` should materialize `<target>/include` and
  `<target>/lib/libgcc_s.so*` / `libc.so` as real files, not self-referential
  symlinks back into `$out`. A later stage0 run showed downstream bootstrap
  builds missing `.../<target>/include/linux` and `.../<target>/lib/libgcc_s.so.1`
  even though the raw toolchain had them.
- Stage3 self-build NCL must use rooted imports (`bootstrap/seed.ncl`,
  `bootstrap/make.ncl`, ..., `lib/lib.ncl`) and resolve them from the repo
  root import path plus `lib/`. A temp-file self-build NCL with plain
  `import "seed.ncl"` picked up `lib/seed.ncl` instead of
  `bootstrap/seed.ncl` (`FieldMissing toolchain`), and a root-only import path
  later made `lib/lib.ncl` fail to find its plain `import "fetch.ncl"`.
- Embedded Nickel stdlib in `crates/crunch-eval/src/stdlib.rs` must mirror the
  whole checked-in `lib/` directory, not a handpicked subset. Missing
  `fetch.ncl` / `project_outputs.ncl` only shows up after install, when
  `crunch bootstrap --fetch` can no longer rely on the source-tree `lib/`.
- `CRUNCH_FORCE_EMBEDDED_STDLIB=1` forces `crunch-eval` to skip source-tree
  stdlib discovery and use the embedded Nickel stdlib. Use it when proving
  installed-style `crunch bootstrap --fetch` behavior from a checkout.
- `examples/bootstrap-no-nix.ncl` must discover the seed through `$NIX_STORE`,
  not a hardcoded `/nix/store`, or the example breaks under the default
  `/crunch/store` logical prefix.
- `stage_source()` must package the current worktree contents, not
  `git archive HEAD`. Otherwise the self-hosting proof builds stage1
  from stale committed sources and stage2 can regress to already-fixed
  behavior even though the checkout binary passed stage0. The current
  implementation does this by Rust-copying a fixed allowlist of top-level
  repo entries; if a build-relevant top-level entry moves, update
  `STAGED_SOURCE_TOP_LEVEL_ENTRIES` and its tests. Its staged source
  fingerprint must include file contents, not just `path:size` pairs, or
  same-size edits silently reuse a stale `*-crunch-src` tree. Any reused
  `--source-store-path` validation must also require `lib/` alongside
  `Cargo.toml`, `bootstrap/`, and `.cargo/vendor-config.toml`, because
  `cmd_self_build()` always builds import paths from `src_dir/lib`.
- A stage1 binary can inherit a compile-time `SNIX_BUILD_SANDBOX_SHELL`
  pointing at a busybox in the stage0 temp store. `vendor/snix-build`
  must treat a non-placeholder compile default as usable only when that
  file still exists; otherwise it should fall back to a discovered
  static busybox (or `/bin/sh` as a last resort).
- The self-hosting proof test (`tests/self_hosting.rs`, `#[ignore]`)
  uses a fresh tempdir store per run. Pre-assertions verify the store
  is empty before stage0. It now enforces the stronger fixed-point claim:
  stage1 and stage2 crunch binaries must match byte-for-byte, and the
  stage0/stage2 busybox bootstrap outputs must also match.
- The proof bundle (`target/self-hosting-proof/run-*/summary.txt`) now
  records proof mode, scrubbed-vs-inherited stage0 PATH strategy,
  blocked Nix binaries for stage0, copied `stage0-prerequisites/inventory.md`,
  resolved sandbox-shell path, store inventory, stage0/stage2 bootstrap-tool
  digests, stage1==stage2 status, first differing byte offset, and embedded
  `/nix/store/...-busybox|...-bwrap|...-crunch-src` references. First
  rerun on 2026-04-12 showed stage1!=stage2 with busybox drift but bwrap
  stable; after pinning busybox kbuild metadata in `bootstrap/busybox.ncl`
  (`KCONFIG_NOTIMESTAMP=1`, `SOURCE_DATE_EPOCH=1`, fixed
  `KBUILD_BUILD_{TIMESTAMP,USER,HOST,VERSION}`), rerun bundle
  `target/self-hosting-proof/recheck-after-bwrap-assert-fix/summary.txt`
  showed `stage1_equals_stage2: true`, `stage0_bwrap_equals_stage2_bwrap: true`,
  `stage0_busybox_equals_stage2_busybox: true`, and a single shared busybox
  store path `7zf934zcyvfz7wg4xf82j97qrvdiaqax-busybox`.
  Current proof bundles must also carry durable `binaries/stage1-crunch` and
  `binaries/stage2-crunch` copies; release evidence and witness rebuilds should
  use the bundled stage2 binary instead of the temp store path printed by the
  original self-build run.
- Stage2 now proves a stronger bootstrap boundary: `cmd_self_build`
  accepts a hidden `self-build --source-store-path <...-crunch-src>`
  override, and the ignored proof passes the exact stage0 staged source,
  runs stage2 from outside the repo, and clears PATH first. If stage2
  regresses to host `git`/`cargo`/`tar`/`cp` source staging, the proof
  now fails instead of silently using host tools.
- `tests/self_hosting.rs` now writes a stage audit bundle and a
  `<stage>-diagnostics.txt` file immediately after each stage command,
  before any success assertions. If stage0/stage2 fails or expected
  proof markers are missing, the panic message includes the audit bundle
  path and the saved diagnostics snapshot from disk, not a recomputed
  view of the current filesystem. Do not hash the whole proof `store/`
  or `state/` trees into that bundle: a real stage0 self-build can exceed
  `tests/audit_support.rs`'s `MAX_AUDIT_ENTRIES` and panic after a successful
  self-build, before stage2 even starts. Audit selected artifacts instead
  (`logs-dir`, `pathinfo.redb`, diagnostics, captured streams, proof-reported
  output binaries). `tests/audit_support.rs::write_command_audit()` still
  records `started_unix_s`/`finished_unix_s` at audit-write time after the
  subprocess exits, so long runs can show identical start/finish seconds;
  do not treat those fields as elapsed-runtime evidence until that helper
  is fixed.
- Stage-specific launch/help/copy failures in `tests/self_hosting.rs`
  should use `pre_stage_context(...)` or `stage_context(...)`, not raw
  `expect(...)`, so the ignored proof keeps the same breadcrumbs even
  when a subprocess fails before the next assertion.
- Fast breadcrumb check: `cargo test -p crunch --test self_hosting
  self_hosting_controlled_failure_reports_breadcrumbs -- --nocapture`
  should PASS. It spawns the test binary in a controlled-failure mode and
  asserts that the child output contains the audit bundle path,
  diagnostics path, and saved diagnostics snapshot.
- `tests/self_hosting.rs` now runs stage0/stage2 with live teeing: child
  stdout/stderr stream to the parent test output and to `<proof_dir>/stage0-*.txt`
  / `<proof_dir>/stage2-*.txt` while the command runs. The proof stages now
  pass `--verbose --log-level info`: keep `verbose` so finished derivation logs
  are echoed inline, but clamp tracing noise so `snix_castore` DEBUG spam does
  not turn a proof run into hundreds of MiB of stderr. If the ignored proof
  looks stalled, use the printed `proof dir:` / `stageN stdout:` /
  `stageN stderr:` paths to inspect the live capture files instead of waiting
  for the stage to end. The ignored proof must fail loudly when prerequisites
  are missing; a bare `return` turns an explicitly requested proof run into a
  false green test.
- `src/self_build.rs` emits stable proof progress markers on stderr:
  `self-build-proof: progress=bootstrap-tool-start:<tool>`,
  `...bootstrap-tool-done:<tool>`, `...crunch-build-start`, and
  `...crunch-build-done`. Final proof lines now also include
  `self-build-proof: hermeticity-mode=<practical|strict>`. `SelfBuildReport::parse_proof_lines()` ignores the extra progress lines, so parsers that only need the final report stay compatible.
- The generated self-build derivation still prints a misleading non-fatal
  verify line: `src/self_build.rs` runs `$out/bin/crunch --version 2>&1 | head -3 || ... --help`,
  but `crunch` has no `--version`. The pipeline succeeds because `head`
  exits 0, so the fallback `--help` branch never runs. Real proof logs can
  contain `error: unexpected argument '--version' found` immediately before
  the final verifier reports `binary OK`.
- Run with: `cargo test -p crunch --test self_hosting -- --ignored --nocapture`
- Needs: bwrap, git, cargo, tar on PATH; static busybox as
  `SNIX_BUILD_SANDBOX_SHELL`; `/run/wrappers/bin` before
  `/run/current-system/sw/bin` for suid fusermount3; `TMPDIR` pointing
  to a filesystem with ~4 GiB free (not tmpfs if it's full).
- PATH ordering alone is not enough on this host. `BubblewrapBuildService`
  now falls back to materializing castore inputs on disk only for the
  specific `FuseDaemon::new()` failure path that reports
  `Unexpected exit code when running fusermount`, so self-build can
  continue when FUSE mounts are unavailable without masking unrelated
  sandbox setup errors. If a fresh validation run hangs after `FUSE INIT`
  with no bwrap child, rerun with `CRUNCH_NO_FUSE=1` to force materialized
  inputs and keep the bootstrap evidence moving.
- Vendored `snix-build` treats `SNIX_BUILD_SANDBOX_SHELL=/bin/sh`
  (or a compile-time default of `/bin/sh`) as a placeholder, not a real
  sandbox shell. It auto-discovers a `busybox-static` binary under
  `/nix/store/*busybox-static*/bin/busybox` before falling back to `/bin/sh`.

## Benchmark Harness

- Cheapest honest store-aware benchmark path so far is a fresh temp `StoreHandle` per sample: write one deterministic blob to castore, build a signed `PathInfo`, `persist_and_export_signed_output(...)`, reopen the store, then time `cached_node_for_path(...)`.
- That persistence boundary only requires a non-empty `PathInfo.signatures` list. For local benchmark fixtures, a parsed fixed narinfo signature string is enough; trust verification is not part of `persist_and_export_signed_output()`.
- Root `Cargo.toml` sets `autoexamples = false`. New checked-in benchmark examples must get an explicit `[[example]]` entry or `cargo run --example ...` fails with "no example target named ...".
- The wide package-set fixture (`tests/fixtures/wide_package_set.ncl`) needs the Nickel stdlib import path. Benchmark examples opening it directly should pass `crunch_eval::stdlib::stdlib_import_path()` or evaluation fails on missing `lib.ncl`.
- `crates/crunch-eval/src/session.rs` `force_all_roots_bounded()` is no longer always a worker-pool path: for small all-root sets (currently up to 16 roots with effective worker count ≤2) it now reuses the coordinator session's in-process `force_all_roots()` path instead of cloning labels into isolated workers. On the fixed 16-root benchmark fixture that collapses the bounded metric to roughly the eager-all-roots cost (~65ms) and makes further worker-pool micro-tuning irrelevant for the primary workload until the fixture or threshold changes.
- `crates/crunch-eval/src/session.rs` `open_source()` now tries a parse-only static-discovery fast path before `eval_shallow()`. For safe top-level let-wrapped record literals with only static fields and no top-level `name` field, it seeds `RootShape::Record` labels directly from the Nickel parser and skips shallow evaluation entirely. On the fixed 16-root benchmark fixture that dropped discovery from ~47ms to ~1.7ms and moved the remaining bottleneck to deep export / force time.

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
- **Host-tool-free seccomp supervisor seam**: `src/protected_exec_seccomp.rs` installs a current-thread Linux seccomp user-notification filter for `execve`/`execveat`, validates absolute exec paths against `ProtectedExecPolicy`, and appends `ProtectedSeccompAuditEvent`s before responding. Tests must run supervisor installs inside subprocessed exact libtest invocations because the filter cannot be removed from the installing thread. The implementation intentionally denies relative paths, empty `execveat` paths, digest mismatches, and unreadable target memory; unsupported kernel/arch/filter setup is fail-closed. `self-build --no-host-tools --stage0-inventory ...` now installs this supervisor after declared seed selection, so incomplete inventories will fail at first undeclared protected exec.
- **Mes/tinycc bootstrap module gotcha**: `bootstrap/mes.ncl` must install both Mes-safe modules from `mes/module/` and mescc compiler modules from top-level `module/`, but must not blindly overlay either top-level `module/mes/{guile,mes-0}.scm` or NYACC's Guile-oriented compatibility files over the Mes-safe stubs. Missing top-level `module/mes/getopt-long.scm` gives `no code for module (mes getopt-long)`; overwriting the Mes stubs gives `major-version`, `ice-9 syncase`, or `datum->syntax-object` failures under mes-m2. Preserve Mes-safe overlaps (`nyacc/compat18.scm`, `nyacc/lang/c99/pprint.scm`, `rnrs/arithmetic/bitwise.scm`, `system/foreign.scm`) after merging NYACC modules.
- **coreutils-6.10 source format gotcha**: live-bootstrap uses `coreutils-6.10.tar.lzma`. Rust `tar` can enumerate that unpacked archive cleanly (3354 entries), while the `.tar.gz` path can trip old GNU header parsing. Keep crunch fetcher `.lzma` support and the NAR hash for the `.tar.lzma` source when validating the live bootstrap chain.
- **Mes libc archive build gotcha**: `bootstrap/mes.ncl` must build `crt1.o`, `libmescc.a`, `libc.a`, `libc+tcc.a`, and `libtcc1.a` with Mes' `build-aux/build-lib.sh` using `compiler=mescc`, not `gcc`; the gcc-flavored Mes sources contain GNU inline asm that mescc rejects. Invoke `mescc.scm` through `mes-m2 --no-auto-compile -e main ... --`, patch/normalize `mescc/mescc.scm::assert-system*` for mes-m2's pointer-like `system*` status values, and put busybox applets before stage0 tools for shell utilities while leaving stage0 `M1`/`hex2` on PATH.
- **tinycc-mes pass1 gotcha**: a `tcc-mes -version` smoke only proves the Mes-linked binary starts. The first self-compile still needs the live-bootstrap pass1 libc refresh: create empty `crti.o`/`crtn.o` for amd64, rebuild Mes `crt1.o`, `libc.a`, and `libtcc1.a` with `tcc-mes`, and use tcc-readable archives before boot0. The Mes `build-lib.sh` `libc+tcc.a` omits `abort`; either append an M1-assembled abort object to a local archive before linking `tcc-mes` or build a fuller initial archive. Do not skip the libc refresh and expect boot0 to link.
- **tinycc-mes current handoff detail**: `bootstrap/tinycc-mes.ncl` now gets past the first self-compile boundary by making `tcc-mes` compile `tcc-boot0` without `Segmentation fault`, then ships a Mes/mescc-built final `bin/tcc`/`bin/tcc-0.9.26` with final output runtime paths. Do not revert it to the old boot0→boot1→boot2 chain without more compiler fixes: `tcc-boot0` still has broken varargs error-reporting / bitfield / section growth paths and can hang or segfault compiling normal TCC sources. Keep this derivation `addressing_mode = 'input-addressed` because the final compiler embeds its logical `/crunch/store/...` lib/include paths; content-addressed provisionals make post-build smoke tests look in the wrong store path. Smoke tests for custom `--store` need a bwrap bind from the physical output dir to the logical `/crunch/store/<hash>-tinycc-0.9.26` path before compiling. The Mes-built final compiler also needs the `x86_64-gen.c` immediate shift count source normalization `g(vtop->c.i & (ll ? 63 : 31)) -> g(vtop->c.i)`; otherwise it emits `shr $0`/`shl $0` for nonzero constant shifts and breaks TinyCC 0.9.27/global-heavy downstream objects.
- **tinycc 0.9.27 amd64 handoff detail**: `bootstrap/tinycc.ncl` tracks live-bootstrap `tcc-0.9.27/pass1.kaem` patches, but Crunch's amd64 part intentionally builds the Mes-linked compiler with `tinycc-0.9.26` and copies the predecessor Mes runtime forward. The upstream x86 pass rebuilds Mes CRT/libc with the new tcc; on amd64 the Mes CRT inline asm currently fails with `bad operand with opcode 'mov'`. Keep the Mes-runtime-sensitive direct-string patches in `tcc.c`, `libtcc.c`, and `tccelf.c`: `snprintf(buf, sizeof(buf), REL_SECTION_FMT, s->name)` in `put_elf_reloca()` must stay spelled as `strcpy(buf, ".rela"); strcat(buf, s->name);` or `tcc -c` of global-reference inputs/GNU make `getopt.c` segfaults through Mes `vsnprintf`. Current smoke boundary is version, trivial object, GNU make `getopt.c` object, and malformed-input non-timeout/non-segfault; leave full Mes runtime/self-hosting proof to the later musl-oriented handoff.
- **OpenSpec scaffold evidence gotcha**: when generating many active OpenSpec changes, commit a validation transcript next to the generated index and say they are scaffolds if tasks remain unchecked. Do not claim broad validation from memory or from hidden command output.
- **live-bootstrap MPC mismatch**: upstream `parts.rst` currently labels the MPC section `mpc 3.2.1`, but `steps/manifest` and `steps/mpc-1.2.1/` use `mpc-1.2.1`; Crunch's `bootstrap/mpc-1.2.1.ncl` tracks the implemented `mpc-1.2.1` part and should mention the heading mismatch explicitly.
- **make-3.82 amd64 handoff detail**: `bootstrap/make-tcc.ncl` needs more than switching from Mes-linked `tinycc 0.9.27` to compile-capable `tinycc 0.9.26`. Keep the GNU Make fixed-arity `concat(...)`/varargs repairs, the TinyCC/Mes stdarg/runtime and exit-status repairs, and compile GNU Make with configure-equivalent header/POSIX wait macros (`STDC_HEADERS`, `HAVE_STDLIB_H`, `HAVE_STRING_H`, `HAVE_UNISTD_H`, `HAVE_FCNTL_H`, `HAVE_SYS_WAIT_H`, `HAVE_WAITPID`, `POSIX`). Without those macros, old fallback declarations/truncated pointer returns or wait-status paths can produce amd64 runtime crashes during recipe dispatch. The current smoke boundary is `make --version` plus a real simple-Makefile recipe that writes and greps `smoke-ok`.

## Mantle rust-plan test gotchas

- Topology CLI tests that execute `rustc` can fail with `env: bash: No such file or directory` when the derivation execution environment does not carry a PATH containing `/run/current-system/sw/bin` or a Nix bash for the clang wrapper. Passing PATH to the outer `cargo test` process is not sufficient for Mantle's topology child env in all cases. For dependency-planning changes, prefer focused planning assertions unless the test specifically needs rustc/link execution.
- `cairn archive <change> --execute` can currently create `cairn/archive/1970-01-01-<change>` in this repo even when current date is known. Rename that archive dir to the session date (`YYYY-MM-DD-<change>`), rerun `cairn validate`, and record the manual rename in evidence if it happens. Also verify the archived delta requirement was synced into `cairn/specs/...`; if archive only moved the change directory, manually copy the `ADDED` requirement into the canonical spec and rerun validation before committing.
- Repo-local oracle checkpoints for Mantle review evidence must include explicit fields: question, inspected evidence, decision, owner, and next action. A summary with only inspected evidence and decision is incomplete for same-family review.
- Native rust-plan producer closure must follow selected Cargo unit dependency artifacts, not every manifest-derived `path_dependencies` edge. Manifest edges can include dependencies that the selected unit graph did not consume; expanding through all of them fabricates producer units/blockers. Do not skip a selected dependency package just because it has a host/build-script/proc-macro sibling target; skip only host-only dependency packages. Use focused tests like `native_unit_graph_follows_selected_unit_dependencies_not_all_manifest_dependencies` and `native_unit_graph_keeps_selected_lib_dependency_with_host_target_sibling` when touching this seam.
- Ambient-env negative tests in `src/rust_plan.rs` should not call `std::env::set_var` in-process. Spawn the current libtest binary with conflicting env values plus a sentinel child test, then assert the pure planner/child-env code still derives values from selected unit state.
- Linked build-script metadata in native rust-plan is a host-edge concern: packages with `links = "..."` export safe custom `cargo:key=value` metadata to dependent build scripts as `DEP_<LINKS>_<KEY>`, and the linked dependency custom-build host unit must run before the dependent custom-build. Metadata dependencies must be derived from selected target-unit dependency artifacts, not raw manifest `path_dependencies` / `build_dependencies`; optional unselected linked deps and build-dependency-only links must not fabricate `DEP_*` needs.
- Native build-script package env parity must feed both direct rustc compilation and child runtime env: `aws-lc-sys` uses compile-time `env!("CARGO_PKG_VERSION")`. Keep package metadata bounded to explicit `CARGO_PKG_*` keys with deterministic empty defaults for omitted optional metadata.
- Native build-script target cfg env parity now derives bounded `CARGO_CFG_TARGET_*` values from the selected target triple for child runtime env. Build-script profile env parity derives `OPT_LEVEL`, `DEBUG`, and deterministic `NUM_JOBS=1` from the selected profile. After target proc-macro host extern binding and produced proc-macro search paths, `darling@0.20.11` compiles; the next known topology frontier is `derive_builder_core@0.20.2` failing because crate warnings are treated as errors without Cargo's `--cap-lints allow` behavior for registry dependencies.
- Native Rust manifest parsing must support `edition.workspace = true` inheritance just like version inheritance; using `Option<String>` for `[package].edition` makes real workspace members fail native package planning with `invalid type: map, expected a string`.
- `RustUnitTopologyExecutionReceipt` serializes unit receipts as `topology_execution.unit_executions`, not `topology_execution.executions`; use the right jq path when summarizing self-probe blocker movement.
- Rust libtest name filtering is substring-based here, not regex alternation. Run `cargo test ... host_dependency_topology` and `cargo test ... combined_unit_topology` as separate commands instead of one `host_dependency_topology|combined_unit_topology` filter.
- The host rustup nightly can inject `gcc-ld/ld.lld`, and that wrapper may reference a removed Nix store `ld-wrapper.sh`. Mantle's topology execution now appends `-C link-self-contained=no` when unit args do not already choose a linker mode, so direct rustc execution uses the selected external linker instead of the stale rustup self-contained wrapper. Match rustc `-C` option names exactly (`link-self-contained` or `link-self-contained=...`), not by loose prefix.
- Native build-script execution must run from the package root with absolute executable/OUT_DIR paths after `Command::current_dir(...)`. If the executable or `OUT_DIR` remains relative, setting `current_dir` makes compiled build scripts fail with `No such file or directory` or unable to write `OUT_DIR`.
- Native build-script `CARGO_PKG_NAME` must be the manifest package name with hyphen spelling preserved, not the custom-build target name (`build-script-build`) or crate-style underscore conversion. Keep target-derived fallback only for legacy units lacking package-name env data.
- Direct rustc proc-macro topology needs Cargo-equivalent extras: add `--extern proc_macro` for proc-macro host units, bind host-unit dependencies from Cargo-selected unit dependency artifacts (not every manifest `path_dependencies` edge), and pass selected package features as `--cfg feature="..."`. Otherwise `async-stream-impl` fails first on missing `proc_macro`/deps, then on `syn` modules gated behind features like `parsing` and `visit-mut`.
- Cargo unit-graph build-script edges can appear as `build_script_build` dependency artifacts on same-package proc-macro host units (for example `rustversion`). Do not pass that as a normal `--extern`; drop it from proc-macro dependency artifacts, order the same-package custom-build host unit first, and bind its generated metadata/`OUT_DIR` into the proc-macro host unit.

- Native rust-plan link-lib metadata parser must allow safe `+` in link names (`stdc++`) and bounded kind modifiers like `static:+whole-archive,-bundle=name`; otherwise Cargo-accepted build-script output is misclassified as malformed metadata.

- 2026-05-28: Native host planning must filter to Cargo-selected host units; planning every manifest-visible proc-macro fabricated an unselected `jiff-static` frontier. Use Cargo-derived `(package_id,target_name,target_kind)` host keys, not all host targets from metadata.

- 2026-05-28: Cargo unit graph custom-build targets can be named `build-script-main` while Mantle metadata package targets use `build-script-build`. Selected-host filtering must normalize custom-build names before matching or linked packages like `aws-lc-sys` lose their metadata producer.

- 2026-05-28: `ring` build.rs unwraps `CARGO_MANIFEST_LINKS`; native build-script env must set it from `[package].links` (empty if absent), and `append_cargo_package_env` must allow this non-`CARGO_PKG_` Cargo key.

- 2026-05-28: Native registry dependency source resolution must be package-version-aware when vendored registry sources contain same-name versions (for example `thiserror` / `thiserror-impl` 1.x and 2.x). Name-only selection linked `thiserror@2.0.18` to `thiserror-impl@1.0.69` and produced `could not find __private` / `as_dyn_error` errors.

- 2026-05-28: Cargo unit graph proc-macro target names can use crate-name underscore spelling while manifest-derived native targets keep hyphen package spelling (`curve25519_dalek_derive` vs `curve25519-dalek-derive`). Normalize proc-macro host keys with `rust_crate_name(...)` or selected proc macros are omitted; curve25519-dalek then compiles both AVX2/IFMA specialization imports unexpanded.

- 2026-05-28: After proc-macro target-name normalization, dirty native topology gets past curve25519 and fails at `aws-lc-sys@0.39.1` with internal error `aws_lc_sys:lib:build reached execution before dependency package ... aws-lc-sys ... was produced`. Treat next frontier as same-package build-script artifact binding / self-dependency ordering, not as a proc-macro selection regression.

- 2026-05-28: Same-package custom-build artifact binding must recognize Cargo's bounded aliases `build_script_main` and `build_script_build`, not just metadata's `build_script_build` crate name. After `rust-topology-custom-build-alias-binding`, clean native topology has `aws-lc-sys@0.39.1` custom-build and lib succeeding; next frontier is `spez@0.1.2` compiling `#[proc_macro]` without proc-macro crate type.

- 2026-05-28: Cargo-normalized vendored manifests can spell proc-macro libs as `[lib] proc_macro = true` (underscore), not only Cargo.toml's common `proc-macro = true` spelling; `spez@0.1.2` exposed this by being planned as a lib. Also, if Cargo unit material reports lib-shaped target kind plus `crate_types = ["proc-macro"]`, Mantle classification must let that crate type override only lib/rlib-shaped Cargo targets, and must feed the same classifier into unit derivation, host artifact discovery, and selected host-unit matching. After this fix, clean native topology gets `spez@0.1.2` proc-macro succeeding and reaches `object_store@0.13.2` failing with `can't find crate for rand`; treat next frontier as selected normal dependency binding/source resolution for `rand`, not proc-macro classification.

- 2026-05-28: `object_store@0.13.2` missing `rand` was really missing Cargo-style crate disambiguators. Native rustc units need deterministic `-C metadata=<hash>` from reviewable package/target/source/features facts so same-name versions like `getrandom` 0.2/0.4 and `rand` 0.8/0.10 can coexist in one topology. After this fix, dirty topology reaches `object_store_status=success`; next frontier is propagating compile-time env such as `SNIX_BUILD_SANDBOX_SHELL` into direct rustc child env for crates using `env!(...)`.

- 2026-05-28: Native rustc child env should stay hermetic but has a tiny compile-time env allowlist. Forward `SNIX_BUILD_SANDBOX_SHELL` from the parent into child rustc env so vendored `snix-build` `env!("SNIX_BUILD_SANDBOX_SHELL")` compiles; do not forward unrelated ambient env like `LD_PRELOAD` / tokens. After this fix, dirty topology moves to root `mantle` lib same-package self-dependency: `unit ... mantle:lib:build reached execution before dependency package path+file:///home/brittonr/git/mantle#0.1.0 was produced`.
