# Napkin

## Build Environment
- cargo/rustc are at `$HOME/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin` and `$HOME/.cargo/bin` — NOT on default PATH. Must prepend to PATH in pueue_run commands.
- Full build env command prefix:
  ```
  export PATH="$HOME/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:$HOME/.cargo/bin:/nix/store/6jafhh81cf85d0vqwrnhl5yfc4wibxvq-protobuf-29.6/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH"
  export PKG_CONFIG_PATH="/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig:${PKG_CONFIG_PATH:-}"
  export SNIX_BUILD_SANDBOX_SHELL=/bin/sh
  ```

## Nickel Gotchas
- **Recursive record scoping kills inline contracts in returned records.** If `fetch.ncl` defines `let Hash = ...` and a function returns a record like `{ hash = the_hash, ... }` where `the_hash` was bound via `params.hash | Hash`, the record's recursive scoping creates infinite recursion: the record field `hash` resolves to itself. Fix: extract ALL values into `let` bindings BEFORE the record literal — `let the_hash = params.hash in let the_fixed_output = { hash = the_hash, ... } in { ... fixed_output = the_fixed_output ... }`. Also: `args` is a common field name in Derivation records, so a function parameter named `args` conflicts. Use `params` instead.
- **`optional` is a record contract annotation, not valid in function destructuring patterns.** Use `? default_value` for optional fields: `fun { x, y ? null } => ...`
- **`std.contract.apply` inside the same recursive record also recurses.** If the record re-exports `Hash = Hash`, applying `Hash` from inside any function in the record triggers recursion through the record's own `Hash` field.

## Code Gotchas
- Rust raw strings already preserve quotes. For JSON test fixtures, use `r#"{ "k": "v" }"#` with plain quotes inside — don't double-escape them as `\"` or serde_json will fail with `key must be a string`.
- `crunch-build::Worker` goal keys are still built with `StorePath::to_absolute_path()` (hardcoded `/nix/store`). If a caller exposes `FailedGoal.drv_key` to higher layers while using a custom `store_dir`, normalize it back to `to_absolute_path_with_prefix(store_dir)` first or `--fix` / label lookups will miss.
- `vendor/fuse-backend-rs` test `test_new_channel` is environment-sensitive if it uses stdout; a pipe read-end is epollable and works reliably here, but `nix::unistd::pipe()` returns two owned raw fds, so the write end must be closed/dropped explicitly or the test leaks it. `vendor/snix-castore`'s `ServiceBuilder` doctest needs `#[async_trait::async_trait]` on the example impl.
- Done-review only sees what is in scope for the current turn. If a checked OpenSpec task needs proof, add in-repo evidence for that turn (command output excerpts, file-path/function evidence) instead of future-dated status notes or claims in AGENTS.md.
- For OpenSpec checklists, avoid historical wording you can no longer prove from the current tree (`types only, no logic`, phase-local compile claims, etc.). Reword tasks to current end-state facts or leave them unchecked.
- Nickel `Expr::to_serde()` does NOT convert enum tags to strings for plain `String` fields. Fixed with `NickelString` serde wrapper in crunch-glue that accepts both strings and enum tags via `deserialize_any` + `visit_enum`. Applied via `#[serde(deserialize_with)]` on `system`, `algo`, `mode` fields in `CrunchDerivation`. Direct `to_serde()` path now works — no JSON intermediate needed for `CrunchDerivation`. The `evaluate_str_and_deserialize()` JSON path still exists for non-CrunchDerivation types.
- `bstr::BString::as_bytes()` is `pub(crate)` — not usable from outside bstr. Use `Vec::from(bstring)` or `<BString as AsRef<[u8]>>::as_ref()` instead.
- `NarCalculationService` trait must be imported to call `calculate_nar()` on `SimpleRenderer`.
- snix-castore has no `MemoryDirectoryService` — use `RedbDirectoryService::new_temporary()` for in-memory directory service.
- `BuildRequest.outputs` paths must be relative (strip leading `/`).
- Workspace `cargo check` succeeds from cached state in ~1s; full build ~18s.

## Architecture Notes
- Recursive async functions need Box::pin indirection (E0391 cycle in layout computation). See `build_derivation` → `build_derivation_inner` pattern in orchestrate.rs.
- `BubblewrapBuildService` is behind `#[cfg(target_os = "linux")]` in snix-build. Added `pub use bwrap::BubblewrapBuildService` to vendored mod.rs.
- `RedbDirectoryServiceConfig` fields were private in vendored snix-castore. Made them `pub` to construct from crunch binary.
- Generated Nickel code in Rust: `push_str` does NOT format-expand `{{` to `{` — those are literal. Only `format!`/`write!` macros expand `{{`.
- Nickel destructuring (`let { X } = ...`) is exact by default. Use `let { X, .. } = ...` for partial destructuring.
- crunch-build crate added in Phase 4: `crates/crunch-build/` with `build_request.rs`, `orchestrate.rs`, `error.rs`
- `KnownPaths.get_by_drv_path()` added in Phase 4 — linear scan over values. Fine for small graphs, may need indexing later.
- Build caching uses PathInfoService (redb) + filesystem existence check. Both must agree for a cache hit.
- **Builds require /nix/store to be writable** for output files to land on disk. On read-only /nix/store, the build "succeeds" (output in castore + PathInfo db) but files aren't written. `export_castore_to_disk()` silently skips on read-only or permission-denied.
- **To test on read-only /nix/store**: use `unshare --mount --map-root-user` + overlay: `mount -t overlay overlay -o lowerdir=/nix/store,upperdir=$UPPER,workdir=$WORK /nix/store`. bwrap must be in PATH inside the namespace.
- **CA provisional paths**: CA derivations use input-addressed paths as provisional `$out` (NOT `hash_placeholder`). Both provisional and final CA paths are store paths with the same name → same length → byte-level self-reference rewriting works. Previous approach using `hash_placeholder` (63 chars) failed because final CA paths are shorter (varies with name).
- **CA output name**: `drv_path.name()` includes `.drv` suffix. Must strip it with `.strip_suffix(".drv")` for the CA output path name.
- **Source input closure**: bwrap only mounts declared inputs. Dynamically linked binaries need their entire Nix closure mounted. `resolve_nix_closure()` calls `nix-store -qR` for this. Returns empty vec if nix-store unavailable (graceful degradation).
- **`--store <custom>` FIXED**: now works. Derivation path computation always uses `/nix/store` (logical prefix). `--store` controls only the physical output directory on the host. Source inputs read from `/nix/store/`, built outputs written to `--store`. `LOGICAL_STORE_DIR` constant in orchestrate.rs and main.rs. `resolve_host_path()` picks the right prefix based on whether a path is a source input or built output.
- **Proto cache**: after editing `.proto` files in `vendor/proto/`, must `rm -rf ~/.cargo-target/debug/build/snix-store-* ~/.cargo-target/debug/.fingerprint/snix-store-*` to force regeneration.
- **blake3::Hasher::finalize()** collides with `digest::Digest::finalize()` when both traits are in scope. Use fully-qualified `blake3::Hasher::finalize(&hasher)`.
- **blake3 in snix-store HashingReader**: can't use `DynDigest` trait (blake3::Hasher doesn't implement it). Refactored `ToHash` trait to have its own `update_hash()` method. `Blake3Wrapper` struct bridges the gap.
- **Castore export**: `export_castore_to_disk()` walks the Node tree and writes files/dirs/symlinks. `SymlinkTarget::as_ref()` returns `&[u8]`, needs `OsStr::from_bytes()` for `std::os::unix::fs::symlink`. `PathComponent::as_ref()` returns `&[u8]`, needs `std::str::from_utf8()`.
- The `Builder` struct is generic over `BS: BlobService`, `DS: DirectoryService`, `BServ: BuildService`.

- **StorePath Ord only compares digest, not name.** `nix_compat::StorePath<S>::Ord` compares `self.digest.iter().rev()` — the name is ignored. Two store paths with the same 20-byte digest but different names are `Equal` under Ord, so they collide in BTreeMap. Test helpers must use unique digests (derive from name bytes), not a shared constant.
- **`resolve_host_path` method**: Checks `derivation.input_sources.contains(path)` to decide `/nix/store/` vs `output_dir`. Used in `collect_sandbox_inputs` fallback path.
- **`output_exists_on_disk` (was `path_exists_on_disk`)**: renamed to clarify it checks the physical output dir, not /nix/store.
- **Integration tests with --store temp dirs**: `KnownPaths` must use default (`/nix/store`), not the temp dir. Builder gets the temp dir as `output_dir`. Test assertions check `to_absolute_path_with_prefix(output_dir_str)` for disk paths.

## Lazy Goals Scheduler (2026-04-05, replaces Build DAG)
- `dag.rs` DELETED (613 lines). Replaced by `goal.rs` (610 lines) + `worker.rs` (670 lines).
- `goal.rs`: pure functional core. `Goal` struct with `GoalState` enum (6 states), `GoalRegistry` for dedup. No I/O, no async, no Builder refs.
- `worker.rs`: imperative shell. `Worker::want()` lazily creates goals + wires deps from KnownPaths. `Worker::run()` dispatches via `JoinSet` + `Semaphore`.
- `orchestrate.rs::build_all()` now creates a `Worker`, calls `want()` per root, calls `run()`. 4 lines of delegation.
- FCIS boundary: goal.rs imports only nix_compat + std + crate::error. worker.rs imports goal + orchestrate + tokio. No cycles.
- `PreparedBuild`, `PrepareResult`, `prepare_build()`, `finish_build()`, `build_service()` are `pub(crate)` in orchestrate.rs for worker access.
- `BuildService` stored as `Arc<BServ>` in Builder. Requires `BServ: 'static` for `tokio::spawn`.
- `max_jobs` parameter on `build_all`, wired from CLI `--jobs`/`-j` flag. Default: `available_parallelism` clamped [1,16].
- 24 goal tests + 7 worker tests + 4 build_all integration tests = 35 scheduler tests.

## mkDerivation Default Phases (2026-04-05)
- `default` is a reserved keyword in Nickel. Can't use it as a function parameter name. Use `fallback` instead.
- `include ident` in Nickel record literals works as field pun (`ident = ident`), not record-merge. Functions can be included this way.
- Nickel `nickel eval` on a file that has required record fields (from `include MkDerivationArgs`) fails with "missing definition" — expected since the file is meant to be imported, not evaluated directly.
- Integration tests with mkDerivation use fake store paths (`/nix/store/000...000-bash`). The 32-char hash prefix is required by the StorePath contract.

## Binary Cache Substitution (2026-04-05)
- `NixHTTPPathInfoServiceConfig` has private fields. Must construct via `TryFrom<Url>` which expects `nix+https://...` scheme prefix. Prepend `"nix+"` to user-provided URL before parsing.
- `LruPathInfoService` does NOT implement `Clone`. To share across multiple Builders in tests, wrap in `Arc<LruPathInfoService>` (PathInfoService is auto_impl for Arc).
- FOD unit tests: mock build output won't match declared `ca_hash`, so `verify_fod_hash` fails. Test FOD-skips-remote by asserting `do_build` was called (proving substitution was skipped), regardless of whether the build itself succeeds.
- Remote substitution only helps when crunch produces derivations with identical ATerm hashes to Nix (same output path digests). Currently crunch-specific derivations have unique hashes that won't match cache.nixos.org. Feature becomes useful when crunch can evaluate Nix-compatible derivation specs.

## FUSE Mount Fix (2026-04-05, FIXED)
- fuse-backend-rs 0.12.0's `fuse_fusermount_mount()` passed kernel-level opts (`fd=N`, `rootmode=`, `user_id=`, `group_id=`) to fusermount3. Stripped these in vendored patch.
- **NixOS PATH ordering matters**: `/run/current-system/sw/bin/fusermount3` is a symlink to the NON-suid binary. `/run/wrappers/bin/fusermount3` is the suid wrapper. PATH must have `/run/wrappers/bin` BEFORE `/run/current-system/sw/bin`. The `.envrc` and `crunch-run.sh` helper must include `/run/wrappers/bin` early in PATH.
- fuse-backend-rs vendored at `vendor/fuse-backend-rs/`. Workspace `[patch.crates-io]` ensures all deps use patched version. snix-castore Cargo.toml also points to local path.

## CA Single-Output Rewriting Bug (2026-04-05, FIXED)
- `compute_ca_output()` used `vec![0u8; provisional_bytes.len()]` as the marker for self-reference replacement.
- ELF binaries have zero-padded alignment regions (BSS, section header padding) that match the marker length (~54 bytes for a store path).
- `replace_marker_with_final()` replaced ALL matching-length zero sequences with the final CA path → corrupted section headers, inflated the binary with repeated store paths.
- Fix: use `blake3::hash("crunch-ca-marker:{output_name}")` cycled to fill the marker, matching `finish_build_multi_ca()`'s approach.
- Symptom: `readelf` shows `<corrupt>` section 0, `ld.so` assertion failure when running the binary.

## find_git on NixOS (2026-04-05, FIXED)
- `find_git()` only checked `/usr/bin/git`, `/bin/git`, `/usr/local/bin/git` — none exist on NixOS.
- Fallback used `which git` — `which` also not on PATH in stripped crunch env.
- Fix: added `/run/current-system/sw/bin/git`, per-user profile paths (`/etc/profiles/per-user/$USER/bin/git`), and direct PATH scanning.

## BuildFailed Display (2026-04-05, FIXED)
- `Error::BuildFailed` thiserror Display was `"build failed for {name} (exit code {exit_code})"` — omitted the `log` field.
- Fetcher errors showed generic message with no details. Fixed to include `\n{log}` in the format string.

## Castore-Only Store (2026-04-05)
- `check_cache` now uses `castore_has_content()` (probes blob_service/directory_service) instead of `PathBuf::exists()`. No writable store dir needed.
- `persist_and_export_output` takes `is_root: bool`. Only root outputs get exported to disk.
- `is_root` threaded: `prepare_build` -> `PreparedBuild` -> `finish_build` -> `process_output` -> `persist_and_export_output`. Worker extracts from `goal.is_root`.
- `build_fetcher` also takes `is_root` (fetchers run inline in `prepare_build`).
- Cache tests: must put blobs in `MemoryBlobService` for cache hits. Use `put_blob()` helper. Fake `B3Digest::from(&[0u8; 32])` won't match anything in the blob service.
- `stdlib_tests.rs` has a pre-existing compile error (`Expr` doesn't impl `Debug`). Not related to crunch-build changes.
- Test count: crunch-build 211->213.

## Stdlib/Builders Split (2026-04-05)
- **Circular import danger with `builders/lib.ncl`**: `mk_derivation.ncl` can't `import "lib.ncl"` because Nickel resolves relative to file first, finding `builders/lib.ncl` (which imports `mk_derivation.ncl` -> circular). Fix: import `"derivation.ncl"` and `"contracts.ncl"` directly (unique names that resolve through the import path to `lib/`).
- **Closed Derivation contract breaks mkDerivation output**: mkDerivation adds `pname`, `version`, `meta`, `passthru`, `overrideAttrs` to the record. With closed contract, `| Derivation` rejects those. Fix: mkDerivation no longer applies `| Derivation` -- returns a plain record. Extra fields use `| not_exported` so they're stripped from JSON. The Rust glue layer has `#[serde(default)]` for missing fields (system, addressing_mode, outputs) so defaults still work.
- **Integration tests need `-I crunch_root()`**: builder import `"builders/lib.ncl"` resolves via import paths. Tests must add the crunch repo root as an import path alongside the stdlib dir.
- **Bootstrap .ncl files DON'T use mkDerivation**: they use raw `crunch.Derivation` directly. No bootstrap changes needed for the split.

## Output Selection (2026-04-05)
- `Input::OutputSelection(Box<OutputRef>)` variant added. Serde untagged ordering: Source, OutputSelection, Derivation. OutputSelection must come before Derivation (both are records; distinguishing field is `drv` vs `name`).
- `OutputRef { drv: CrunchDerivation, output: String }` — the `output` field is a single output name, not a set.
- `resolve_inputs` uses `entry().or_default().insert()` for coalescing — same dep with different selections merges into one `input_derivations` entry.
- `Error::InvalidOutputSelection` variant validates at convert time (not build time).
- Nickel `select` function: must use `let the_drv = drv in` before the record literal to avoid recursive record self-reference (classic gotcha, already in napkin).
- `integration_build.rs` had exhaustive match on `Input` — new variant needed a branch (`Input::OutputSelection(_)`).
- `stdlib_tests.rs` pre-existing `{result:?}` compile error fixed (Expr doesn't impl Debug).
- Test counts: crunch-glue 58→65, crunch-build 213→215, stdlib_tests 17→19.

## Nix-Free Bootstrap (2026-04-05)
- `crunch bootstrap --fetch` downloads musl-gcc tarball, persists as FOD in crunch store, generates seed.ncl. No Nix required.
- `bootstrap_fetch()` constructs `CrunchDerivation` objects in Rust (mirroring Nickel's `fetchTarball`) and runs them through the normal convert→build pipeline.
- Musl-gcc hash: `sha256-XpcI34j9YwAQj7qw4DpvXqT1CX00vHcUQbAk/do46jw=` (NAR hash of unpacked tree).
- FOD output path: `/nix/store/kz129c215qy2xkbd7z77n8adidy5ywd0-musl-gcc` (deterministic from hash+name).
- **Fetcher disk-cache**: `build_fetcher` now checks if output exists on disk before downloading. Handles case where bootstrap fetched the tarball but a later `crunch build` starts with empty in-memory castore.
- **Closure-free inputs**: `is_crunch_built()` method checks `output_nodes` (built this session) or custom `--store` dir. Skips `nix-store -qR` for these paths.
- `resolve_and_ingest_sources` tries crunch output dir first, then /nix/store as fallback for source path ingestion.
- `resolve_state_dir()` duplicated in bootstrap.rs (can't import from main.rs binary). Same logic as `state_dir()` in main.rs.
- `generate_fetch_seed_ncl()` uses `(field, path, doc)` tuples; no GC root comments (unlike Nix-based seed).
- FETCH_SEEDS const has `field` (ncl record name) and `doc` (annotation text) per seed package.
- **GNU Make from source DONE**: bypasses autoconf entirely (busybox grep too limited). Hand-written config.h for musl. Key config.h fixes: `HAVE_UMASK` (mode_t conflict), `ST_MTIM_NSEC st_mtim.tv_nsec` (nanosecond timestamps), `GLOB_ALTDIRFUNC 0` (musl lacks it), `_GNU_SOURCE` via CFLAGS not config.h (must precede system headers), `-include stddef.h` for ptrdiff_t, `#undef HAVE_GUILE` but `guile.c` still compiled (stub branch). lib/ glob/fnmatch skipped (musl's system versions used), dir_setup_glob patched out via sed. concat-filename and findprog-in replaced with stubs (not needed without dlopen modules).
- **Dash from source DONE**: much simpler than bash (28 source files vs 150+). Code generators (mksignames, mknodes, mksyntax, mkinit) must be compiled with `-static` (bwrap has no dynamic linker). `mkbuiltins` shell script needs `nl` in busybox symlinks. config.h: `SMALL=1` (disables libedit history), `HAVE_GLOB` undef (musl lacks GNU glob extensions, dash has fallback), `HAVE_SIGSETMASK` undef (BSD legacy, use sigprocmask), `-DSHELL` in CFLAGS (bltin/*.c need it). Nickel imports: use relative `import "make.ncl"` from bootstrap/ dir, not `"bootstrap/make.ncl"`.
- **GNU Make `HAVE_CASE_INSENSITIVE_FS` bug**: code uses `#ifdef` not `#if`, so `#define HAVE_CASE_INSENSITIVE_FS 0` enables case-insensitive mode. Must use `/* #undef */` instead. Symptom: make can't find `Makefile` — looks up `makefile` in directory cache, matches `Makefile` case-insensitively, tries to open `makefile` (lowercase), fails.
- **Nickel m%"..." and Makefile tabs**: Nickel multiline strings don't preserve tab characters. Write Makefiles via `dash -c '...'` using `printf "\t"` for tab generation.
- **Binutils from source**: Uses configure (unlike make/dash). Key fixes: `GREP`/`SED`/`AWK` must be pre-set to bypass long-line probes. `am_cv_ar_interface=ar` pre-set to skip ar interface test. Pre-generated yacc/lex files must be touched (newer than .y/.l) since we have no bison/flex. musl.cc toolchain has unprefixed binutils (`ar` not `x86_64-linux-musl-ar`). Output is dynamically linked against musl — needs musl libc at `/lib/ld-musl-x86_64.so.1` to run.
- **Sandbox read-only source trees**: `cp -a` fails in bwrap sandbox (can't chown). Use `cp -r 2>/dev/null` then `chmod -R u+w` to get a writable copy.
- **Goal scheduler**: `notify_dep_done()` must tolerate Failed state (returns `Ok(false)`) — happens when goal has 2+ deps, one fails (propagates), then another completes. Previously errored on this race.

## Self-Build (2026-04-05)
- **tls-aws-lc -> tls-ring**: eliminates cmake dep. Changed in snix-store, snix-build, snix-castore Cargo.toml.
- **Pre-generated proto files**: `vendor/*/src/generated/*.rs` with build.rs fallback when protoc absent. Eliminates protoc build dep.
- **file:// URL support**: `open_url_reader()` in fetcher.rs. Enables local tarballs as fetchTarball sources.
- **detect_algo in fetch.ncl**: auto-detects hash algo from SRI prefix (`blake3-...` -> `'blake3`). All fetchers use it.
- **Rust standalone installer**: musl-linked `rust-1.94.1-x86_64-unknown-linux-musl.tar.xz`. Skip install.sh (needs bash), copy components directly. MUST `chmod -R u+w $out/lib/` after copying rustc's lib — tarball permissions are read-only, second `cp -r` (std libs) fails with Permission denied.
- **libgcc_s.so.1**: Rust musl binaries need it for unwinding. Set `LD_LIBRARY_PATH` and `LIBRARY_PATH` pointing to musl-gcc toolchain's lib/.
- **LIBRARY_PATH env var**: GCC/ld search path for `-lgcc_s` during proc-macro .so linking. Without it, `ld: cannot find -lgcc_s`.
- **Heredoc quoting**: `<< 'EOF'` prevents shell expansion but also blocks `$VAR` substitution. Use `<< EOF` (unquoted) when vars need expanding (e.g., `$GCC_LIB` in .cargo/config.toml).
- **pathinfo.redb locations**: TWO databases — `~/.local/state/crunch/pathinfo.redb` AND `~/.local/share/snix/pathinfo.redb`. Must clear BOTH when debugging stale cache.
- **Castore export empty files**: FIXED. Was caused by MemoryBlobService (all blobs in RAM). Replaced with ObjectStoreBlobService backed by `~/.local/state/crunch/blobs/`. Blobs now persist on disk. Added `new_local(path)` constructor to vendored ObjectStoreBlobService.
- **Source tarball**: `git archive HEAD` + `tar rf ... vendor-deps/`. No `--prefix` — fetchTarball's strip logic handles single-dir tarballs but flat tarballs work better. Hash must be obtained via `--fix` or dummy hash probe.
- **Build time**: ~9 minutes for full cargo build inside bwrap (668 crates, -j 4, release mode).

## Decouple Build from Glue (2026-04-05)
- `KnownPaths` split into `ConversionCache` (crunch-glue) + `DerivationRegistry` (crunch-build).
- `ConversionCache` has no `resolve_output()`, `get_output_path()`, or `resolved_outputs` -- those are build-time concerns in `DerivationRegistry`.
- `ConversionCache::iter_entries()` yields `(StorePath, [u8;32], Derivation, bool)` for bridge.
- `populate_registry(&mut reg, cache.iter_entries())` bridges the gap. Called in main.rs after all `convert()` calls.
- `DerivationRegistry::insert()` takes `(drv_path, hdm, derivation, content_addressed)` -- no aterm_hash param (that's conversion-time state).
- crunch-glue in `[dev-dependencies]` only: integration tests still need `convert()` + `CrunchDerivation` for end-to-end tests.
- `Error::Glue(#[from] crunch_glue::Error)` removed from crunch-build error.rs.
- `known_paths.rs` renamed to `conversion_cache.rs` in crunch-glue.

## Self-Build Command (2026-04-05)
- `crunch self-build --store <dir> -j N --no-substitute` builds crunch from its own source.
- Stages source via `git archive HEAD | tar -x -C staging` + `cp -a vendor-deps/` into staging. Do NOT use `git archive --prefix` + `tar rf --transform` — GNU tar's `--transform` doesn't apply to directory entries, causing fetch pipeline's strip logic to produce wrong results.
- **No NAR hash, no tarball, no FOD.** Source tree is copied directly into the store as a plain path. Derivation references it as `Input::Source("/nix/store/HASH-crunch-src")`. Store path name uses blake3 fingerprint of file listing (paths + sizes) encoded as nix-base32 (first 20 bytes → 32 chars).
- Store path names must use nix-base32 (`0-9a-z` minus `e,o,t,u`). Hex hashes are invalid. Use `nix_compat::nixbase32::encode()`.
- Generated .ncl uses `format!()` with `r#"..."#`. `{{` in format strings → literal `{` in output. Shell `${VAR}` in Nickel `m%"..."` passes through to shell (Nickel uses `%{...}` for interpolation).
- Source tree with vendored deps is ~787 MiB. Needs that much free in the store dir.
- `busybox-static` path from AGENTS.md may get GC'd. Re-fetch with `nix-build '<nixpkgs>' -A pkgsStatic.busybox --no-out-link`.
- Output: 31 MiB static-pie musl-linked ELF at `$store/*-crunch/bin/crunch`.
- `find_self_built_binary` scans output dir for `*-crunch/bin/crunch`.
- `find_source_dir` walks up from cwd looking for `Cargo.toml` + `bootstrap/`.

## crunch-store Extraction (2026-04-05)
- `crunch-store` crate: owns export, ca_mapping, StoreHandle, query. crunch-build re-exports via thin wrappers.
- `Builder<BS, DS, BServ, PIS>` -> `Builder<BServ>`. Uses `Arc<dyn BlobService>` etc internally.
- `Builder::new()` is generic over concrete types (wraps in Arc). `Builder::with_state_dir()` takes pre-wrapped `Arc<dyn ...>` for StoreHandle callers.
- `BubblewrapBuildService<Arc<dyn BlobService>, Arc<dyn DirectoryService>>` works fine — auto_impl on Arc means trait methods dispatch correctly.
- `SimpleRenderer<BS, DS>` internal type params: replaced with `SimpleRenderer<Arc<dyn BlobService>, Arc<dyn DirectoryService>>` in the one `compute_ca_output` signature.
- Tests that call `with_state_dir` directly need explicit `Arc::new(bs) as Arc<dyn BlobService>` wrapping.
- `output_nodes`, `built_outputs`, `ca_mappings`, `output_dir_str` all moved INTO StoreHandle. Builder reduced to 3 fields: `store: StoreHandle`, `build_service: Arc<BServ>`, `verbose: bool`.
- `StoreConfig` needs `output_dir: PathBuf` (from CLI `--store`).
- sed-based bulk replacements of `self.X` -> `self.store.X` can hit fields on other structs in the same file (e.g., MockBuildService.blob_service). Always grep for false positives after sed.
- `StoreHandle::from_services()` now takes `output_dir_str: String` param (for tests that don't use StoreConfig).

## Tiger Style Applied (2026-04-05)
- orchestrate.rs decomposed: 2165→1341 lines. `build_derivation_inner` from ~350→111 lines via 6 extracted methods.
- New modules: `hash.rs`, `export.rs`, `fod.rs`, `references.rs` extracted from orchestrate.rs.
- Production assertions added: convert.rs (5), known_paths.rs (5), build_request.rs (7), fod.rs (1), orchestrate.rs (1), export.rs (1), fetcher.rs (1).
- Fixed limits added: MAX_RECURSION_DEPTH=512 (convert), MAX_BUILD_DEPTH=256 (orchestrate), MAX_EXPORT_DEPTH=128 (export), MAX_DOWNLOAD_BYTES=4GiB (fetcher), MAX_TAR_ENTRIES=500k (fetcher).
- Compile-time assertion: `SANDBOX_ENV_VARS` non-empty.
- Remaining >70 line functions: cmd_build (220), cmd_store (161), build_fetcher (136), convert_inner (115), extract_tar (98).

### FCIS Decomposition (same session)
- `cmd_build` (220→41): split into `evaluate_and_convert` (pure core), `deserialize_derivations` (pure core), `execute_builds` (shell), `open_pathinfo_service` (shell), `build_one_derivation` (shell), `handle_fod_mismatch` (pure).
- `cmd_store` (161→34): split into `cmd_store_list`, `cmd_store_info`, `cmd_store_verify`.
- `convert_inner` (115→15): split into `resolve_inputs`, `build_nix_derivation` (pure, no KnownPaths), `finalize_and_register`.
- `build_fetcher` (136→~110): reused `persist_and_export_output` eliminating 20 lines of duplicated persist+register code.
- Remaining >70: execute_builds (77, shell), parse_fetch (76, pure match), extract_tar (106, tar dispatch).

## Multi-Output Derivations (2026-04-05)
- `outputs` env var now set in `build_nix_derivation` (space-separated list). Changes the ATerm hash of every derivation — all hardcoded store path assertions in tests needed updating.
- FOD output paths are NOT affected by env changes (computed from declared hash, not HDM). Only drv paths changed.
- `Goal.mark_build_failed()` accepts `Ready` state now — needed for inline failures from `prepare_build` (fetcher hash mismatches). Two FOD integration tests were broken since partial failure was added.
- Multi-output CA uses two-pass rewriting: pass 1 replaces ALL provisionals with blake3-derived markers (unique per output name), hashes canonical form; pass 2 replaces markers with final CA paths. Handles cross-output references.
- `build_and_register_multi()` in test_support creates multi-output derivations for tests.
- CLI labels non-"out" outputs: `path (dev)`, `path (lib, cached)`.
- `contract_catches_extra_field` stdlib test was always wrong (Derivation contract has `..`). Changed to `contract_allows_extra_fields`.

## Smoke Tests (2026-04-06)
- `tests/smoke.rs`: 10 end-to-end tests that build derivations and verify outputs on disk.
- All use `--store <tempdir>` — no writable `/nix/store` needed.
- `build_ncl()` helper: writes .ncl to tempdir, runs `crunch build --store --no-substitute`, returns stdout.
- `first_output_path()` / `all_output_paths()` extract paths from stdout (strips " (cached)" suffixes).
- Tests cover: flat file, directory structure, executable scripts (built + executed), failures, cache hits, CA, fetchurl (local HTTP server + sha256 SRI), multi-root .ncl, symlinks, deterministic paths.
- Fetchurl test uses `sha2` + `base64` (dev-deps) to compute expected SRI hash of known content.
- Stale `ca_resolve_output_updates_known_paths` test removed from crunch-glue (methods moved to DerivationRegistry in crunch-build). Equivalent test already at `registry.rs:267`.
- `--workspace` test run still fails on vendored fuse-backend-rs `test_new_channel` (epoll EPERM). Use `-p crunch -p crunch-glue -p crunch-build -p crunch-eval -p crunch-store` to test crunch crates only.

## Lazy Goals (2026-04-05)
- `goal.rs`: pure functional core. `Goal` struct with `GoalState` enum (Pending/Waiting/Ready/Building/Done/Failed). `GoalRegistry` for dedup + tracking. 24 unit tests.
- `worker.rs`: imperative shell. `Worker` struct with `want()` (lazy goal creation + recursive dep discovery + waiter wiring) and `run()` (dispatch loop with JoinSet + Semaphore). 7 unit tests.
- Worker borrows `&mut Builder` for I/O — never does I/O itself. Builder's `prepare_build`, `finish_build`, `build_service` made `pub(crate)`. `PreparedBuild` fields also `pub(crate)`.
- `want()` is callable mid-run — KnownPaths is `&mut`, goal registry grows, ready queue extends. This is the key invariant for future dynamic derivations.
- `fail_goal` + `propagate_failure` scaffolded but not wired (build errors abort via `?`, matching current DAG behavior). Will be wired for partial-failure support.
- FCIS boundary: Goal (core) knows nothing about Builder, Worker, BuildService, KnownPaths, or async. Worker (shell) orchestrates I/O via Builder.
- `run_streaming()` receives `EvalMessage`s via `mpsc::Receiver`, interleaves channel drain + dispatch + build completion via `tokio::select!`. Channel close → `eval_done` flag. Terminates when eval_done AND all roots terminal.
- `cmd_build` pipeline: eval+deserialize (sync) → convert (sync) → mpsc channel → `run_streaming`. The old `evaluate_and_convert` is removed; `execute_builds` kept as dead_code fallback.
- Channel interface is ready for true eval/build overlap: move convert loop to `spawn_blocking` and the Worker builds leaf deps while later roots are still converting.

## Package Authoring (2026-04-05)
- **Nickel record merge (`&`) can't combine unequal scalars or arrays.** If both sides have the same field with different string/array values, merge fails. Use remove+insert (`apply_patches`) for Nix-like `//` update semantics. `std.record.to_array` + fold over `std.record.remove`/`std.record.insert`.
- **Nickel records are recursive by default.** `{ mkStdenv = mkStdenv }` self-references infinitely. Use `include mkStdenv` to bind from outer scope without shadowing.
- **`not_exported` needed on function fields.** `overrideAttrs` is a closure — can't serialize to JSON. Mark with `| not_exported` so `nickel export` and `Expr::to_serde()` skip it.
- **`fun` keyword required in lambda syntax.** `(old => ...)` is invalid — must be `(fun old => ...)`.
- **`let rec` for self-referencing functions.** `mk_derivation_with_shell` calls itself via `overrideAttrs`. Needs `let rec mk_derivation_with_shell = ...`.
- **Recursive record scoping shadows let bindings.** In mkStdenv, `let shell = "%{seed.bash}/bin/bash"` was shadowed by the record field `shell = seed.bash` inside `{ mkDerivation = fun attrs => ... mk_derivation_with_shell shell ..., shell = seed.bash }`. The `mkDerivation` closure captured the field value (bare store path) instead of the let binding (with `/bin/bash`). Fix: rename the let binding to `shell_bin`.
- **Opening the Derivation contract** (adding `..`) doesn't break existing code. The serde layer ignores unknown fields (no `deny_unknown_fields`). Contract still enforces required fields (name, builder).
- **stdenv.mkDerivation prepends base_inputs.** Uses remove+insert on `buildInputs` to avoid array merge conflict.
- New files: `lib/mk_derivation.ncl`, `examples/mk-hello.ncl`, `examples/override.ncl`, `examples/package-set.ncl`.

## Partial Failure (2026-04-05)
- `fail_goal()` + `propagate_failure()` now wired into the Worker. Build errors from `finish_build` and `do_build` are caught and routed to `fail_goal` instead of aborting via `?`.
- `FailedGoal { drv_key, error }` replaces `String` in `WorkerResult.failed`. Carries per-goal error context.
- `process_join_result()` added: unwraps `JoinSet` results, routes `(key, Err(e))` to `fail_goal`.
- JoinSet type changed: `Result<(String, BuildResult), Error>` → `(String, Result<BuildResult, Error>)`. The key is always returned so `fail_goal` can identify the failed goal.
- `dispatch_ready()` catches `prepare_build` errors → `fail_goal`. Previously `?`-propagated.
- `handle_build_completion()` catches `finish_build` errors → `fail_goal`. Previously `?`-propagated.
- `propagate_failure()` now handles `AwaitingDerivation` state (dynamic derivation goals whose producer failed).
- `fail_goal()` takes `error_msg: &str` parameter. Propagated failures say "dependency X failed".
- main.rs: `parse_fod_mismatch_error()` extracts FOD mismatch info from `FailedGoal.error` for `--fix` support.
- main.rs: failed goals printed with per-package error messages before the summary.
- `FailingMockBuildService` in test_support: fails for name-matched builds, succeeds for others.
- 4 integration tests: one-fails-one-succeeds, dep-fails-propagates, all-fail, streaming-partial-failure.
- Test count: crunch-build 201→205.

## Dynamic Derivations (2026-04-05)
- `dynamic.rs`: pure detection/parsing module. `is_drv_output()` checks name ends `.drv` + is file + size < 4MiB. `parse_drv_bytes()` checks ATerm prefix + `Derivation::from_aterm_bytes()`. `register_dynamic_drv()` registers in KnownPaths.
- `GoalState::AwaitingDerivation` added. `Goal::new_awaiting(drv_path, producer_key)` creates goals waiting for a producer. `Goal::set_derivation()` transitions AwaitingDerivation → Pending.
- `GoalRegistry::awaiting_producer(key)` finds AwaitingDerivation goals whose producer matches a key.
- `Worker::detect_dynamic_derivations()` runs after every `handle_build_completion()`. Scans outputs for `.drv` files, reads blob, parses ATerm, registers in KnownPaths, activates awaiting goals or creates new root goals via `want()`.
- `Builder::read_blob(node)` reads file blob content from castore (used for `.drv` output parsing).
- `DrvProducingMockBuildService` in test_support: mock that returns ATerm bytes as build output for name-matched builds.
- **ATerm format requires output paths**: `Derivation::from_aterm_bytes()` rejects derivations where output paths are `None` ("MissingOutputPath"). Must call `calculate_output_paths()` before `to_aterm_bytes()` in tests.
- **Parser error type**: `nix_compat::derivation::parser::Error<&[u8]>` inner `NomError` doesn't impl Display. Use `{e:?}` not `{e}`.
- `Goal.producer_key: Option<String>` links AwaitingDerivation goals to their producer. `notify_dep_failed()` now accepts AwaitingDerivation state too.
- Test count: crunch-build 198→201 (14 dynamic derivation tests: 8 in dynamic.rs, 3 in goal.rs registry/lifecycle, 3 in worker.rs integration).

### Warning Cleanup + Test Coverage (same session)
- All crunch-owned warnings eliminated (was 9, now 0). Only 12 vendor snix-castore deprecation warnings remain (upstream object_store `.child()` → `.join()`).
- Files fixed: derivation/mod.rs (unused imports), crunch-eval/lib.rs (PathBuf), ca_mapping.rs (StorePath), rewrite.rs (B3Digest, size, STORE_PATH_HASH_LEN), orchestrate.rs (store_dir allow), main.rs (RedbPathInfoService), build_request.rs (aterm_hash).
- Tests added: export.rs (12 tests: file/symlink/dir export, permissions, nesting, error paths, depth limit), hash.rs (15 tests: all 5 algos for nar_hash + hash_blob, determinism, empty/large, directory, missing blob), error.rs (12 tests: Display formatting for all variants, From conversion, Send+Sync).
- Test count: crunch-build went from 80->118 unit tests. Workspace total ~185.

## gRPC -> postcard Migration (2026-04-05)
- **`#[async_trait]` must precede `#[auto_impl]`** on trait definitions. auto_impl runs first (outermost), sees raw async methods, generates impls without the async_trait desugaring. Put `#[async_trait]` before `#[auto_impl]` so async_trait transforms the trait first.
- **`include!("...")` shares the parent module's namespace.** Generated type files included via `include!()` must not re-import types already in scope (e.g., `use bytes::Bytes` conflicts with parent's import). Remove duplicate imports from included files.
- **`prost::Enumeration` provided `From<Enum> for i32`.** When replacing with `#[repr(i32)]` serde enums, must add explicit `impl From<Enum> for i32 { fn from(h: Enum) -> i32 { h as i32 } }` for code that does `.into()` on the enum.
- **`postcard::from_bytes` needs type annotations** when the return type isn't constrained by context. Old `proto::Type::decode(bytes)` was explicit. New `postcard::from_bytes(bytes)` needs turbofish: `postcard::from_bytes::<proto::Type>(bytes)`.
- **`use async_trait::async_trait;` must come AFTER `//!` module doc comments.** `sed -i '1i ...'` inserts before doc comments, causing E0753. Check for `//!` lines and insert after the last one.
- **`bytes::Bytes` with `serde` feature** serializes directly. No `serde_bytes` needed. Enable via `bytes = { version = "1", features = ["serde"] }`.
- **postcard `to_stdvec` needs `use-std` feature**, not just `alloc`.
- Removed: tonic, tonic-build, tonic-health, tonic-reflection, prost, prost-build, tower, tower-http. Added: postcard (use-std), async-trait, bytes/serde.
- Directory digests now use postcard encoding (not protobuf). All stored data (redb) is incompatible with old format. Delete `~/.local/state/crunch/` to reset.

## Bootstrap Busybox from Source (2026-04-06)
- **HOSTCC must include `-static`**: busybox kbuild compiles host tools (scripts/basic/fixdep) in a single step. `HOSTLDFLAGS=-static` is too late — it's appended after the output. Use `HOSTCC="gcc -static"` to ensure static linking for all host tool compilations.
- **Dynamic linker symlink needed**: even with `HOSTCC="gcc -static"`, the bootstrap GCC itself and intermediate tools need `/lib/ld-musl-x86_64.so.1`. Symlink from musl's `lib/libc.so`.
- **Binutils unprefixed symlinks**: bootstrap GCC looks for `as` via PATH but binutils installs as `x86_64-linux-musl-as`. Create unprefixed symlinks in /tmp/tools/.
- **bzip2 required**: busybox's mkconfigs script uses bzip2. Add to busybox symlink set along with gzip, which, env, cut.
- **NIX_BUILD_CORES=0**: sandbox sets this to "0" which is invalid for `make -j`. Default to 4.
- **`set -e` in bootstrap scripts**: without it, failed make commands don't abort the build. The sandbox reports success with an empty output dir.
- Bootstrap chain order: musl-gcc → make → dash → binutils → musl → gcc → busybox. ~5.5 min total.

## Store Prefix Integration Fix (2026-04-06)
- **Builder::new/with_state_dir hardcoded STORE_DIR**: Both constructors created StoreHandle via `from_services()` which defaults to `/nix/store`. The CLI-configured `--store-prefix` was never threaded through. Fixed by adding `store_dir: &str` parameter to both constructors.
- **Output::path_str() hardcoded `/nix/store`**: `build_request.rs` used `o.path_str()` → `to_absolute_path()` (hardcoded STORE_DIR). Fixed to `o.path_str_with_prefix(store_dir)`. The sandbox found outputs at `crunch/store/HASH-name` but the BuildRequest looked for `nix/store/HASH-name`.
- **Smoke tests need busybox prefix**: nixpkgs busybox-static doesn't have `CONFIG_FEATURE_SH_STANDALONE`. External commands like `mkdir`/`chmod`/`ln` aren't found when `PATH=/path-not-set`. Use `BB=/bin/busybox; $BB mkdir` pattern.
- **Pathinfo.redb lock contention in parallel tests**: Smoke tests share `~/.local/state/crunch/pathinfo.redb`. When another test holds the lock, fallback to in-memory db loses cache between runs. `smoke_build_cached_on_second_run` fails intermittently.
- **smoke_build_multi_derivation_file**: Pre-existing eval failure — array of derivations syntax not supported.
- **Stale CA mappings**: After switching prefix, `~/.local/state/crunch/` must be cleared. The migration warning exists but old entries cause `invalid CA mapping path` errors.

## Configurable Store Prefix (2026-04-06)
- `_with_store_dir` variants added to nix_compat: `to_aterm_bytes_with_store_dir`, `serialize_with_store_dir`, `serialize_hdm_with_store_dir`, `fod_digest_with_store_dir`, `hash_derivation_modulo_with_store_dir`, `from_absolute_path_with_prefix`, `from_absolute_path_full_with_prefix`, `path_str_with_prefix`.
- ATerm serialization: `write_outputs_with_prefix`, `write_input_sources_with_prefix`, `write_input_derivations_with_prefix`, `write_store_path_with_prefix` in write.rs.
- FOD digests DO differ with store dir changes — the absolute path string in `sha256("fixed:out:...")` includes the prefix.
- `StoreConfig` has `store_dir: String` field. `StoreHandle::store_dir()` accessor.
- Worker accesses store_dir via `known_paths.store_dir()`. If `register_dynamic_drv` needs `&mut known_paths` plus `store_dir`, extract the string first to avoid borrow conflict.
- CLI: `--store-prefix /crunch/store` (default) + `--nix-compat` (shorthand for `/nix/store`). Resolved in `run()` before the match to avoid partial-move issues with `args`.
- `sed -i 's/FOO/self.method()/g'` is dangerous in constructors where `self` doesn't exist yet. Always check the result.

## Own Closure Tracking (2026-04-05)
- `crunch_store::resolve_closure()` replaces `resolve_nix_closure()` (nix-store -qR subprocess). Lives in crunch-store/src/closure.rs.
- `PathInfoService` trait error type is `snix_store::pathinfoservice::Error` (a type alias for `Box<dyn Error + Send + Sync>`), NOT `snix_store::Error`.
- `PathInfoService::list()` returns `BoxStream<'static, ...>` — the `'static` lifetime matters, not `'_`.
- MockPathInfoService in tests needs `#[tonic::async_trait]` (not `async_trait::async_trait`). tonic added as dev-dep to crunch-store.
- `StoreHandle::remote_pathinfo()` returns `Option<Arc<dyn PathInfoService>>`. To get `Option<&dyn PathInfoService>` for the closure walker, use `remote_ref.as_deref()`.
- Test count: crunch-store 15->24 (9 closure tests). crunch-build 218. Total: 242.

## Eval/Build Streaming Overlap (2026-04-07)
- Convert loop now runs on `tokio::task::spawn_blocking`. Each root's `convert()` + `cache.drain_pending()` sends entries over `mpsc::channel` to the Worker. Worker calls `accept_eval_message` which inserts entries into `DerivationRegistry` before `want()`.
- `ConversionCache::drain_pending()` returns entries added since last drain. Uses a `pending: Vec<...>` field populated in `insert_ca()`. `MAX_PENDING = 16_384`.
- `EvalMessage.new_entries: Vec<(StorePath<String>, [u8; 32], Derivation, bool)>` carries registry tuples. Existing tests pass `new_entries: vec![]` since they pre-populate the registry.
- `accept_eval_message` takes `&mut DerivationRegistry` (was `&DerivationRegistry`). `drain_eval_messages` also changed to `&mut`.
- `tx.blocking_send()` from `spawn_blocking` context (can't use `.await`). Channel capacity 16 provides backpressure.
- Convert thread returns `Ok::<_, RunError>(drv_paths)`. Main task awaits the `JoinHandle` after Worker completes to get `drv_paths` for output display. Convert errors take priority over Worker results.
- `Derivation`, `StorePath<String>`, `ConversionCache`, `CrunchDerivation` are all `Send` — no issues crossing thread boundary.
- Pipeline FOD mismatch integration tests are easiest with two `crunch.fetchurl` roots pointing at local `file://` URLs: one good hash, one bad hash. No network needed, and `PipelineResult.fod_mismatches` gets exercised for real. Still gate the test with `can_build()` because `crunch_pipeline::build()` returns a platform error on non-Linux hosts.
- Test count: crunch-glue 65->68 (3 drain_pending tests). crunch-build 222->224 (2 entries-in-message tests). Total workspace: 385.
