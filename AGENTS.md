# Repository Guidelines

## Project Overview

Mantle is a Nickel-authored, Rust-implemented build system on the Nix store
protocol: it evaluates typed derivations (Nickel contracts reject malformed
inputs before execution), schedules builds lazily, executes them in bounded
bwrap sandboxes, persists output content and PathInfo state (Ed25519
signatures, BLAKE3 identity) in one store backend per state directory (default
Snix castore and PathInfo files, or Casita under `<state-dir>/casita`), and
emits structured, bounded evidence.
It is a build tool only — frontends like Onix own module evaluation and system
configuration (ADR 0010). Active research software: every proof/receipt is
bounded to its declared inputs; do not promote observations into correctness
claims.

**Naming:** "Mantle" is the project-facing name for all prose. `crunch` /
`crunch-*` remain exact crate, package, path, and binary identifiers — the root
package `mantle` builds both `mantle` and `crunch` binaries from one
`src/main.rs`. Opportunistically rephrase stale Crunch references in nearby
text, but never rename actual crate/path identifiers unless that is the task.

## Architecture & Data Flow

Core architectural law: **Functional Core / Imperative Shell** (Tiger Style).
Pure decision logic lives in `#![no_std]` + alloc `*-core` crates; all
filesystem/process/network/clock/env effects live in std adapter crates or
`src/` shell modules. ~9 core/adapter pairs exist (attestation, delta, project,
shell, rust-cache, hardware-simulation, kernelscript, spacewasm,
wasm-component) plus standalone no_std cores (action-result, composition,
eval-budget, evaluation-stream, gc, overlay, repair, bootstrap, mantlepkgs,
portable-client, build-contract). The boundary is enforced by
`scripts/check-no-std-core.sh`.

Build data flow (code-verified):

```text
src/main.rs (clap, ~140 commands) -> src/build_cmd.rs -> crunch_pipeline::build()
  |- crunch-eval EvaluationSession      Nickel eval; lib/ stdlib embedded via include_str!
  |- crunch-store StoreHandle           split into capability views
  |- tokio::join!(worker.run_streaming(...), stream_roots_into_worker(...))
  |     root -> crunch_glue::convert -> (StorePath, nix_compat::Derivation)
  |     -> EvalMessage over mpsc(16) -> Worker + GoalRegistry
  |        (lazy goals, dedupe by drv path, semaphore-bounded jobs,
  |         rank_ready_goals under SchedulingPolicy)
  |     -> DispatchBuildService: "builtin:fetchurl" -> FetchBuildService,
  |        else RemoteFirstBuildService (remote -> local bwrap sandbox)
  |     -> signed PathInfo persisted; outputs exported to physical --store
  |- post-build: closure resolution (own PathInfo + remote narinfo, NO nix-store),
     attestations (crunch-attestation: canonical JSON -> BLAKE3), GC roots
```

Layering facts an assistant must not get wrong:

- `crunch-eval` does **not** call `crunch-glue`; conversion happens inside
  crunch-pipeline's evaluation stream. `crunch-attestation` is a post-build
  consumer of store data, not a pipeline stage.
- **Logical vs physical store**: derivation hashes and ATerm serialization use
  `--store-prefix` (default `/mantle/store`; `--nix-compat` forces `/nix/store`;
  legacy `/crunch/store` survives in older tests/bootstrap). `--store` is only
  the physical export directory (default `/nix/store`). `--state-dir` holds
  `store-identity.json` (the recorded backend and prefix; a mismatched
  `--store-backend` or `--store-prefix` fails), logs, attestations, roots, and
  backend data: `pathinfo.redb`, `directories.redb`, and `blobs/` under the
  default `snix`, or the Casita repository in `<state-dir>/casita` under
  `casita`, which keeps the Snix castore only as in-memory scratch (see
  `docs/store-backends.md`). Different prefixes produce different derivation
  hashes.
- Store authority is split into narrow capabilities (ADR 0058): `BuildStore`,
  `OutputLookup`, `RootRegistry`, `ActionResultPort`, `SourceAdmission`,
  `StoreAdmin` via `into_pipeline_store_parts()` / `into_builder_store_parts()`.
  Do not pass `StoreHandle` around; it is a compat facade.
- The vendored snix crates (`vendor/`) provide only the data layer (derivation
  format, castore, PathInfo, `BuildService` trait). Scheduling, orchestration,
  and the eval→build pipeline are Mantle's own (ADR 0001).

## Key Directories

| Directory | Purpose |
|---|---|
| `src/` | Binary crate (package `mantle`). One `*_cmd.rs` module per command family; `main.rs` holds only clap defs + thin dispatch. `src/lib.rs` re-exports shell modules for integration tests. |
| `crates/` | 37 library crates: crunch-pipeline, crunch-build (scheduler/sandbox), crunch-store, crunch-eval, crunch-glue, crunch-nar, crunch-delta, crunch-project, crunch-attestation, crunch-shell + their `*-core` pairs; mantlepkgs-core, mantle-build-contract, wasm/hardware-simulation research crates. |
| `vendor/` | Vendored snix fork — workspace members (`nix-compat`(+derive), `snix-build`, `snix-castore`, `snix-store`, `snix-tracing`) plus `fuse-backend-rs` via `[patch.crates-io]`. Modified (BLAKE3 support in nix-compat), `publish=false`, excluded from first-party gates. |
| `vendor-deps/` | Full crates.io + git vendored registry; consumed only through `.cargo/vendor-config.toml`. Never merge or hand-edit. |
| `lib/` | Nickel stdlib (18 files), embedded at compile time via `include_str!` in `crates/crunch-eval/src/stdlib.rs`. Editing `lib/` changes the binary. |
| `builders/` | `mk_derivation.ncl` / mkStdenv Nickel builders — eval-time imports, not embedded. |
| `config/` | Typed Nickel policy sources + `generated/*.json` embedded via `include_str!` in crunch-store/crunch-build. Regenerate with `nix run nixpkgs#nickel -- export --format json config/<area>/default.ncl > config/<area>/generated/<policy>.json`. Never edit generated JSON directly. |
| `bootstrap/` | ~200 self-hosting bootstrap recipes (`.ncl`) + `evidence/`, `seeds/`, `patches/`. |
| `tests/` | 55 integration suites + `support/` helpers + `fixtures/` tree. |
| `scripts/` | Proof drivers (bash), single-file `cargo -Zscript` guard rails, quality wrappers. |
| `docs/` | ~57 guides + `generated/` (checked-in, drift-gated) + release-notes. Root `README.md` is the documentation index. |
| `adr/` | Numbered decision records 0001–0082; ADR 0082 (explicit store backends, Casita) is Proposed; index in `adr/README.md`. |
| `.cairn/` | Active Cairn lifecycle tree (prose writes `cairn/`): `changes/`, dated `archive/`, `specs/`. |
| `openspec/` | Legacy spec system; still hosts functional-core validation assets consumed by the no-std rail. |
| `tools/` | Generator bins (`generate-operator-command-contract`) + `tracey_refs.rs` coverage bridges. |
| `store` | Gitignored symlink → `datapool/mantle-store`; dev builds use `--store store --state-dir store/state` to keep outputs off the root FS. |

## Development Commands

Always work from the repo root inside the checked-in shell:

```bash
nix develop                                # nightly toolchain, clang/mold, pinned nickel 1.17.0,
                                           # cargo-deny/nextest/watch, cargo-tigerstyle, wasm toolchain
cargo build -p mantle
target/debug/mantle doctor                 # workflow preflight, no state mutation
./scripts/check-first-party-quality.sh     # ordinary gate: gcc40-bridge self-test -> fmt -> clippy
                                           # -> workspace lib/tests (serialized)
./scripts/check-first-party-tigerstyle.sh  # = nix run .#tigerstyle -- check
nix flake check                            # full check set (fmt/clippy/nextest/tigerstyle)
cargo deny check                           # reads deny.toml
```

`scripts/quality-gate-common.sh` owns the canonical package lists:
`FIRST_PARTY_PACKAGES` selects the registered first-party packages for the fmt
check, while `VENDORED_WORKSPACE_EXCLUDES` lists `fuse-backend-rs`, `nix-compat`,
`nix-compat-derive`, `snix-build`, `snix-castore`, `snix-store`, and
`snix-tracing`. For each changed first-party workspace package outside the fmt
selection, separately run `cargo fmt --check -p <package>`; passing the selected
fmt check alone does not verify those packages. Strict Clippy and workspace
lib/tests still cover the workspace except the listed vendored members. Plain
`cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --all`
are **not** valid gates — vendored members fail first. Manual gate equivalents:

```bash
./scripts/check-gcc40-configure-bridge.rs --self-test
source scripts/quality-gate-common.sh
cargo_fmt_first_party
cargo clippy --workspace --all-targets --no-deps \
  --exclude fuse-backend-rs --exclude nix-compat --exclude nix-compat-derive \
  --exclude snix-build --exclude snix-castore --exclude snix-store --exclude snix-tracing \
  -- -D warnings
cargo test --workspace --lib --tests \
  --exclude fuse-backend-rs --exclude nix-compat --exclude nix-compat-derive \
  --exclude snix-build --exclude snix-castore --exclude snix-store --exclude snix-tracing \
  -- --test-threads 1
```

Focused test legs (package selector is `-p mantle`; `-p crunch` is stale —
`crunch` is only a bin alias):

```bash
cargo test -p mantle --test smoke
cargo test -p mantle --bin mantle rust_plan::tests:: -- --test-threads=1   # embedded unit module
cargo test -p mantle --test self_hosting -- --list                          # compile+discovery only
cargo test -p mantle --test self_hosting -- --ignored --nocapture           # full ~30 min proof
```

Run / examples:

```bash
target/debug/mantle eval examples/hello.ncl
target/debug/mantle --store /tmp/mantle-store --state-dir /tmp/mantle-state \
  build examples/hello.ncl --no-substitute
./scripts/prove-self-hosting.sh --check      # prereq preflight ONLY — not proof evidence
```

Formatting gotcha: formatting crate roots (`src/main.rs`, `crates/*/src/lib.rs`)
can cascade rustfmt into sibling modules and dirty unrelated files — format
leaf files or revert the churn immediately.

## Code Conventions & Common Patterns

- **Tiger Style** (nightly `#![register_tool(tigerstyle)]`): ≥2 assertions per
  non-trivial function (positive AND negative); explicit `MAX_*` fixed limits
  with fail-fast; functions <70 lines; explicit integer widths (`u32`/`i64`,
  not `usize`); saturating/checked arithmetic; unit-suffixed names
  (`latency_ms_max`, `capacity_bytes`); nested ifs/early returns over compound
  conditions. Pre-existing violations are frozen behind `HARDENING-BACKLOG`
  allow blocks — never extend them.
- **Dependency injection seams**: snix `BuildService` (async `do_build`) with
  `DispatchBuildService` routing on `command_args[0]`; `RefreshResolver` in
  crunch-project (callers in `src/` implement network/git resolution; the crate
  stays pure); `Canonicalize` in crunch-attestation (stable canonical bytes over
  sorted nodes/edges/roots → BLAKE3 `AttestationDigest`).
- **Async**: tokio. Eval and build interleave via `tokio::join!` + bounded mpsc;
  raw Nickel / `crunch_eval::Error` values are not `Send` — stringify failures
  before crossing thread boundaries. `block_in_place` panics from current-thread
  callers (CRUNCH_NO_FUSE materialization fallback).
- **Error handling**: typed error enum per crate; `RunError` (`src/errors.rs`).
  Nickel errors must render via `NickelError::format(..., ErrorFormat::Text)` —
  formatting through `{:?}` can stack overflow. Audit events and typed report
  structs flow through `PipelineResult` rather than being dropped.
- **Identity**: BLAKE3 for all Mantle-owned digests, cache keys, receipts, plan
  IDs; SHA-256 only at Nix/Cargo interop boundaries. Sign every build output;
  cache hits verify trusted keys (dedup verifying keys by full key material,
  never by key name alone).
- **Compile-time embedded inputs**: `lib/*.ncl` (crunch-eval stdlib),
  `config/*/generated/*.json` (crunch-store/crunch-build). Change the Nickel
  source, regenerate the JSON, rebuild.
- **CLI/wire enums** deliberately stay unboxed (`#[allow(clippy::large_enum_variant)]`
  with justification comment) — don't box or narrow them.
- **Requirement markers**: preserve `// r[impl <change>.<requirement>]`
  comments linking code to Cairn requirements.
- **`--json` mode** must keep stderr machine-readable: tracing stays
  uninitialized unless `--verbose`/`--log-level`/`RUST_LOG` is set.
- **ADRs**: any decision that constrains future work (tool choice, pattern,
  rejected alternative) gets `adr/NNNN-short-title.md` (next number from
  existing files; follow the newest ADR shape: Status / Context / Decision
  Drivers / Decision / Consequences) plus an index row in `adr/README.md`.
  Write it during the session, not as an afterthought.
- **Lifecycle**: Cairn is active — change dirs under `.cairn/changes/<name>/`
  (`proposal.md`, `design.md`, `tasks.md`, `metadata.json`, `specs/` delta,
  `evidence/`). Validate with
  `nix run path:/home/brittonr/git/cairn#cairn -- validate --root .`.
  Commit implementation *before* `cairn archive`. Capture gate transcripts to
  files and append them to evidence (avoid piping repeated `nix run` gates
  through `tee`). The repo-owned runner is
  `scripts/cairn-lifecycle-evidence.rs` driven by `evidence/commands.txt`.
  OpenSpec is legacy (archive only), but its `specs/functional-core/`
  validation assets stay live.

## Important Files

| File | Role |
|---|---|
| `src/main.rs` | Clap surface (~140 commands), global flags (`--store`, `--store-prefix`, `--nix-compat`, `--json`, `--state-dir`, `--base-store`), dispatch |
| `src/build_cmd.rs` | `mantle build` → `crunch_pipeline::BuildConfig`, signing keys, trusted keys |
| `crates/crunch-pipeline/src/lib.rs` | `build()` / `build_with_evaluation_stream()` orchestration; Linux-only build path |
| `crates/crunch-build/src/orchestrate.rs` | `Builder<BServ>`: cache-check vs build, derivation → BuildRequest → sandbox → PathInfo |
| `crates/crunch-build/src/worker.rs`, `goal.rs` | Lazy goal state machine, `GoalRegistry` with `MAX_GOALS`, drv-path dedupe |
| `crates/crunch-build/src/dispatch_build_service.rs` | fetchurl vs sandbox routing |
| `crates/crunch-store/src/handle.rs`, `capability.rs`, `closure.rs` | Store facade, capability split, closure resolution |
| `crates/crunch-eval/src/session.rs`, `stdlib.rs` | Nickel session (root discovery, bounded workers), embedded stdlib table |
| `crates/crunch-glue/src/convert.rs` | `CrunchDerivation` → `nix_compat::Derivation` + `StorePath` |
| `scripts/quality-gate-common.sh` | Canonical first-party / vendored-exclusion package lists |
| `Cargo.toml` | 44-member workspace; pins `blake3 =1.8.2`, `digest =0.10.7` (blake3 1.8.3+ pulls digest 0.11 and breaks snix-castore), `bstr =1.12.1` |
| `deny.toml` | cargo-deny policy (reasoned advisory ignores, license allowlist, `private.ignore`) |

## Runtime/Tooling Preferences

- **Rust nightly** via `rust-toolchain.toml` (unpinned channel; required for
  `#![feature(register_tool)]`). Self-built stable toolchains need
  `RUSTC_BOOTSTRAP=1`.
- **Linker/build env**: clang + mold + `-Wl,--allow-multiple-definition`
  (`.cargo/config.toml`), pkg-config + openssl-dev — all supplied by
  `nix develop`. Bare host cargo typically fails with `linker 'clang' not found`.
- **`SNIX_BUILD_SANDBOX_SHELL`** must be set when compiling vendored snix-build
  (`env!` at compile time). For real sandbox builds it must be a
  **statically-linked busybox** — the bwrap FUSE sandbox has no host glibc.
  `/bin/sh` is treated as a placeholder; snix-build then auto-discovers
  `/nix/store/*busybox-static*/bin/busybox`.
- **Runtime needs for derivation builds**: `bwrap` on PATH, Linux user
  namespaces, writable `--store`, `/run/wrappers/bin` before
  `/run/current-system/sw/bin` (suid fusermount3), TMPDIR with ~4 GiB free.
  `nix-store` is NOT required.
- **Shared global target dir**: the host's `~/.cargo/config.toml` redirects all
  builds to `~/.cargo-target/`. Use `CARGO_TARGET_DIR=./target` (docs) or a
  scratch dir like `/tmp/mantle-dev-target` (parallel/validation runs). Never
  rebuild the shared binary mid-proof.
- **Nickel is pinned** to cohort 1.17.0 in `flake.nix` — never floating
  nixpkgs nickel.
- **`autoexamples = false`**: every new example/benchmark entry point needs an
  explicit `[[example]]` entry in `Cargo.toml` or `cargo run --example` fails.
- **Long builds and proofs** (self-hosting, fixed-point, native provider,
  multi-stage bootstrap — minutes to hours) run detached under pueue
  (`pueue_run` without wait, output to a log in a dedicated run dir); monitor
  with `pueue_status`/`pueue_log`/`pueue_wait`. Never in session foreground.
  Follow `.pi/skills/pueue-long-builds/SKILL.md`.
- Vendored snapshots (`vendor/`, `vendor-deps/`) are not casual editable
  source; changes there need explicit justification and records.

## Testing & QA

Three layers: (1) root `tests/*.rs` — 55 integration suites driving built
binaries via `Command::cargo_bin("mantle")` / `cargo_bin("crunch")` or calling
library APIs directly; (2) embedded `#[cfg(test)] mod tests` across `src/` and
`crates/*/src` (~337 files); (3) shared helpers — `tests/audit_support.rs`
(audit bundles, schema `crunch-test-audit-v1`, written under
`target/test-audit/<suite>/`), `tests/support/oci_registry.rs` (in-process OCI
registry). Key suites: `smoke.rs` (CLI e2e over the `crunch-build-report-v1`
JSON), `self_hosting.rs` (the `#[ignore]` fixed-point proof + always-run
machinery tests), `release_cli.rs` (release/witness/attest), root and
crunch-pipeline `integration_build.rs`, `stdlib_tests.rs` (Nickel contracts),
`examples_inventory.rs` + `rust_compatibility_rail.rs` (both in the ordinary
gate), `removed_system_cli.rs` (guards against reintroducing a module layer).

Conventions an assistant must follow when adding tests:

- **Gate capability-dependent tests** with `can_build()` (`/nix/store` + bwrap;
  root `tests/integration_build.rs` uses the stronger `has_bwrap()` probe that
  actually runs `bwrap --ro-bind / / -- /bin/true` — bare `--version` is not
  enough). On skip: `eprintln!("skipping: …")` + `return`. Every
  `crunch_pipeline::build()` test needs the gate, even fetcher-only ones
  (`build()` errors on non-Linux).
- **`#[ignore]` proof/heavy tests**: `-- --list` proves compile+discovery only;
  real runs need `-- --ignored --nocapture`. They must fail loudly on missing
  prerequisites — a bare `return` creates a false green.
- **Env poisoning goes in a subprocess**, never in-process `std::env::set_var`
  (a guard test source-scans for violations). Tests that mutate `PATH` take
  `PATH_MUTEX` before fixture setup (fixtures still invoke host `git`);
  `lock_proof_env()` serializes proof-script launches (`Text file busy`).
- **Audit-grade tests** call `write_command_audit()` and audit *selected*
  artifacts — never whole proof `store/`/`state/` trees (entry-limit panic
  after a successful 30-minute run).
- **Never run the real self-hosting proof inside CLI tests** — use the
  `CRUNCH_WITNESS_REBUILD_DRIVER` fake-driver seam; the fake driver writes the
  `CRUNCH_TEST_WITNESS_DRIVER_LAUNCH_SIGNAL` file as its first side effect so
  "driver never launched" cannot pass vacuously.
- **Nickel fixtures** importing `"lib.ncl"` need the stdlib import path
  (`crunch_eval::stdlib::stdlib_import_path()` or `<repo>/lib`);
  `CRUNCH_FORCE_EMBEDDED_STDLIB=1` forces embedded stdlib for
  installed-binary behavior.
- **libtest filters are substring-based** — run name alternatives as separate
  commands, never `a|b`.
- **Doctests are outside the ordinary gate** (`--lib --tests`) and plain
  `cargo test` doctest results are untrustworthy on this host (shared target
  dir). Prefer `--lib --tests` legs.
- Evidence/rail tests must assert required **non-claims** and reject overclaim
  strings, not just happy paths.
- `mantle store verify` reports intermediates MISSING by design (intermediates
  stay in castore) — verify exported roots individually.

Evidence rules (binding for any completion/status claim):

- **No proof, no claim.** Quote test counts and `test result:` lines only from
  captured output, never memory. Pueue results are inspected via
  `pueue_wait`/`pueue_log` for the exact task ID.
- Clean/dirty tree claims require same-turn `git status --short --branch`
  output. Post-archive Cairn validation claims require the captured post-archive
  output appended to the change's evidence transcript.
- A successful build is not a bootstrap/release claim. Before making or
  reviewing self-build, Cargo-free, Nix-free, or release claims, read
  `docs/operator-proof-guide.md`; each workflow there has an exact command, an
  exact output, and an exact claimable statement with explicit non-claims.
