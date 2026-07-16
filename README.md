# mantle

Nickel build system on the Nix store protocol.

mantle evaluates `.ncl` files describing derivations, constructs store
paths using BLAKE3 hashing, and executes builds in a bwrap sandbox. No
Nix evaluator in the loop.

## Quick Start

```bash
# Build mantle (requires Rust nightly, clang, mold, openssl-dev)
cargo build --release

# Generate a seed toolchain from your Nix store
mantle bootstrap -o seed.ncl

# Write a derivation
cat > hello.ncl << 'EOF'
let mantle = import "lib.ncl" in
{
  name = "hello",
  builder = "/bin/sh",
  args = ["-c", "echo 'Hello, mantle!' > $out"],
} | mantle.Derivation
EOF

# Evaluate (print JSON, no build)
mantle eval hello.ncl

# Build (requires Linux + bwrap)
mantle build hello.ncl

# Machine-readable build summary for audit tooling
mantle --json build hello.ncl
```

`mantle --json build` writes a stable `crunch-build-report-v1` JSON
object to stdout. It includes per-root outcomes, cache hits, failure
records, output paths, hermeticity audit data, `network_policy_reports`,
`workspace_reports`, `cargo_build_evidence[]`, `cargo_build_evidence_diagnostics[]`,
`ast_grep_structural_evidence[]`, `ast_grep_structural_evidence_diagnostics[]`,
and per-output `artifact_attestation` references (`logical_path` + sidecar
`path`) so tests and operators can assert on structured data instead of scraping
human text. The
optional `log_file` fields are only present when the corresponding log was
actually written to disk.

For successful cache hits, each output may also include a `substitution`
object. `mode` is `delta` when mantle completed the hit through delta
reuse and `full` when it completed through ordinary full-artifact fetch.
`transferred_bytes` reports bytes fetched from the cache, `reused_bytes`
reports receiver-local reuse, and `fallback_reason` only appears when
mantle started delta negotiation but finished through full fetch.

Human output reports same facts inline on cached outputs, for example:

```text
…/result-path (cached, substitution=delta, transferred_bytes=12, reused_bytes=34)
…/result-path (cached, substitution=full, transferred_bytes=55, reused_bytes=0, fallback_reason=stream_application_failed)
```

Remote build data uses receiver-driven bounded chunks, durable fenced resume,
and ordinary output admission. See [Resumable remote transfer](docs/remote-transfer.md)
for policy, checkpoint, fallback, completion, and non-claim semantics. External
worker allocation can use digest-pinned direct or Slurm batch dispatchers without
changing coordinator or output authority; see
[External batch dispatchers](docs/external-batch-dispatchers.md).

Retained tool caches are explicit, bounded, fenced, and claim-downgraded. See
[Stateful tool workspaces](docs/stateful-workspaces.md) for Nickel policy,
local and remote lifecycle rules, quarantine/retention behavior, and clean
comparison evidence.

### Machine artifact contracts

Public machine-JSON families are explicitly classified in the typed Nickel
registry under `schemas/machine-contracts/`. Stable contracted surfaces bind
the Rust owner, exact schema snapshot, generated Nickel review contract,
fixtures, consumer/version policy, non-claims, and freshness with BLAKE3.
Runtime commands remain Rust-only; Nickel contract evaluation is test evidence,
not a product dependency.

```bash
nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs
nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs --self-test
nix develop -c cargo -Zscript scripts/check-nickel-export-core-pin.rs --root .
nix develop -c cargo -Zscript scripts/check-nickel-export-core-pin.rs --self-test
```

Contract conformance proves shape and declared linkage only—not build
correctness, cache trust, reproducibility, release eligibility, attestation
truth, or deployability. See
[Machine artifact contracts](docs/machine-artifact-contracts.md).

## Operator diagnostics

Use `mantle doctor` before a fresh build host or self-build run:

```bash
# Default profile: build
mantle doctor

# Self-build prerequisites
mantle doctor --profile self-build
```

Current doctor profiles:

- `build` — default. Checks `bwrap`, sandbox shell resolution,
  `fusermount3`, writable state dir, and writable output store dir.
- `self-build` — everything in `build`, plus nightly `cargo +nightly`
  and `rustc +nightly` visibility.

Doctor is read-only: it does not start builds, download substitutes, or mutate
store state.

For bootstrap runtime-validation work, `mantle bootstrap validate` combines the
build doctor preflight, a no-substitute build, captured stdout/stderr logs, a
coarse host-path leakage scan, and OpenSpec-ready JSON/Markdown evidence:

```bash
mantle --json --store .mantle-drain/make-store \
  --state-dir .mantle-drain/make-state \
  bootstrap validate bootstrap/make-tcc.ncl \
  --evidence-dir openspec/changes/live-part-make-3-82-runtime-validation/evidence \
  --resume
```

Use `mantle build --plan` to preview what mantle will do per root without
building:

```bash
mantle build --plan hello.ncl
mantle --json build --plan hello.ncl
```

Planned action labels:

- `cached` — every output is already accepted from local PathInfo + castore
- `substitute` — at least one output is missing locally but remote narinfo says
  a cache hit is available, and no local build is needed
- `build` — no acceptable cache hit exists, but local build preflight passed
- `preflight-error` — mantle cannot perform the local build path for that root
  (for example missing output store dir, missing `bwrap`, or missing sandbox
  shell)

### Cache substitution diagnostics

Mantle models remote binary caches as a bounded ordered set of **cache
candidates** (not a single URL). The `--substituters` flag accepts a
comma-separated list. Each entry becomes a candidate with deterministic
priority matching configuration order:

```bash
mantle build --substituters "https://cache.example.com,https://backup.example.com" hello.ncl
```

Per-output cache admission diagnostics are available in both plan and
build report output. JSON output includes a `cache_admission` block:

```json
{
  "cache_admission": {
    "reason": "local-hit"
  }
}
```

Stable reason codes: `local-hit`, `remote-hit`, `miss`,
`untrusted-signature`, `store-prefix-mismatch`, `fixed-output-remote-hit`,
`offline-network-required`, `local-castore-incomplete`, and
`duplicate-identity-trust-mismatch`.

The `cache_admission` field is omitted when no admission data is available,
preserving compatibility with existing consumers.

### Shared action results

Mantle publishes a successful derivation as a signed immutable
`mantle-action-result-v1` record only after every named output has durable,
signed PathInfo and complete castore content. The action-result index is a
bounded discovery hint, not trust: every discovered candidate is rechecked
against the requested derivation, object and PathInfo identities, signatures,
producer identity, receipt linkage, sandbox/network policy, and reference-scan
facts before Mantle skips execution. Conflicting fully admitted output sets fail
with `conflicting-action-results`; source order and CA mappings never choose a
winner.

Local records live under `<state>/action-results/v1`. Configured HTTP
substituters expose provider-neutral sidecars under `/action-results/v1/` next
to their `.narinfo` and NAR objects. Offline store sets never open HTTP action
sources. Human plans/builds print `shared-action-result` lines; JSON reports
carry bounded `action_result_reports` with selected/rejected candidates, trust
basis, source class, conflict class, diagnostics, and explicit non-claims.

The typed source/limits/trust/offline/publication/GC policy is
[`config/action-result-policy/default.ncl`](config/action-result-policy/default.ncl).
GC never treats candidate metadata as an output root: it retains local result
metadata only while the referenced store paths are independently live.

### Remote metadata cache

Remote cache metadata is cached under the state directory in
`advisory-meta-cache.json`. Cached metadata is advisory only — it can
accelerate planning but never admits an output without final PathInfo
signature, content hash, castore completeness, and attestation
verification. Features: TTL-based expiry (15 minutes for narinfo, 5
minutes for negative misses), schema versioning, trust-policy isolation,
explicit refresh via `--no-substitute` or `force-refresh`, and bounded
capacity (10,000 entries).

### Castore completeness

Before reporting a local cache hit, Mantle verifies the full castore
tree is present (not just the root node). Directory outputs require
recursive checking of child blobs and subdirectories. The check uses an
iterative stack-based traversal bounded to 100,000 nodes and 128 depth
levels, with global in-memory completeness markers that skip redundant
probing for verified directory nodes. Symlinks are always complete.
Incomplete trees return `local-castore-incomplete` instead of silently
falling through to a remote lookup or rebuild.

### Cache hit reporting

For successful cache hits, each output may include a `substitution`
block:

```json
{
  "substitution": {
    "mode": "delta",
    "transferred_bytes": 12,
    "reused_bytes": 34
  }
}
```

`mode` is `delta` when mantle completed the hit through delta reuse and
`full` when it completed through ordinary full-artifact fetch.
`fallback_reason` only appears when mantle started delta negotiation but
finished through full fetch.

Recommended operator workflow:

1. Run `mantle doctor` for the intended workflow profile.
2. Run `mantle build --plan ...` to see which roots are cached,
   substitutable, buildable, or blocked by preflight.
3. Run the real build only after doctor and plan look sane.
4. If a build fails, read the structured failure summary first, then open the
   saved log path when one is present.

JSON build failures in `crunch-build-report-v1` now use a typed failure
envelope under `failed[]` with these fields:

- `root` — operator-facing root label
- `drv_key` — derivation store key
- `phase` — `preflight` or `build`
- `error_class` — typed class such as `preflight`,
  `fixed-output-hash-mismatch`, or `sandbox`
- `message` — original failure text
- `saved_log_path` — only present when mantle wrote a saved failure log

Human failure output prints the same facts in the same order before any deeper
error text.

Output lands in `/nix/store/<hash>-hello` by default (the `--store`
default). Derivation hashes are computed under the `/mantle/store`
logical prefix (`--store-prefix`). Use `--store /tmp/mystore` to
write outputs elsewhere, or `--nix-compat` to switch the logical
prefix to `/nix/store` for interop testing. See
[Store Paths and Prefixes](#store-paths-and-prefixes) for details.

Pass `--strict-hermetic` to `mantle build`, `mantle self-build`, or
`mantle bootstrap validate` when degraded hermetic behavior should fail instead
of warn. Pass the explicit Nix-style `--impure` escape hatch only for diagnostic
or compatibility work that intentionally permits ambient host dependencies;
`--impure` is mutually exclusive with `--strict-hermetic`. Human and JSON
reports both surface `hermeticity_mode` and any
`hermeticity_audit_events` recorded during the run, including the typed
`impure-mode-selected` event for impure runs. Ordinary derivation builds also
run offline by default: network access is admitted only for declared
fixed-output fetchers (`fetchurl`, `fetchTarball`, `fetchGit`) or for a future
scoped compatibility capability that names its action, policy basis, and audit
class. Denied compatibility requests appear as blocked `network_policy_reports`
and do not produce strong build-correctness evidence. Strict builds also bind a
BLAKE3 digest of the normalized child environment in `build_environment_reports`;
denied dynamic-linker, compiler-wrapper, proxy, token-like, locale, or temp-root
variables are reported by class with secret values redacted.

## Operator workflows

Current operator loops:

- Plan and build: `mantle doctor`, `mantle build --plan`, `mantle build`,
  `mantle self-build --strict-hermetic`
- Develop and run: `mantle shell`, `mantle develop`, `mantle run`
- Inspect and publish: `mantle attest`, `mantle release`

Short examples:

```bash
# Shells and package execution from the compatibility-named crunch.ncl package root
mantle shell --command env
mantle shell dev --command env
mantle develop            # deprecated alias for shell
mantle run .#hello -- --help
mantle run ./tool.ncl --bin tool -- --version

# Declared Nickel export and generated-file handoff workflows
mantle export config.ncl --format json --out generated/config.json
mantle filegen plan --plan-out target/filegen-plan.json
mantle filegen apply --plan target/filegen-plan.json

# Pinned, network-denied WebAssembly component materialization
mantle wasm-component build component-request.ncl --out target/component-evidence

# Attestation and release evidence entry points
mantle attest show /nix/store/<hash>-hello
mantle release verify target/release-evidence/<release-id>
mantle release attest target/release-evidence/<release-id>
mantle attest release-verify target/release-verification/<release-id> --trusted-public-key <name:base64>
```

For the full command path, examples, and sidecar rules, see
[`docs/operator-workflows.md`](docs/operator-workflows.md).

For frontend-neutral action/object/reference-scan evidence contracts, see
[`docs/build-correctness-primitives.md`](docs/build-correctness-primitives.md).
For the pinned ast-grep profile, sidecar schema, build-report fields, BLAKE3
identity rules, release attachment, and structural-only non-claims, see
[`docs/ast-grep-structural-evidence.md`](docs/ast-grep-structural-evidence.md).
For foreign derivation import receipt boundaries, policy digests, cache/source
trust, sandbox capabilities, Guix-like and Nix-like examples, and admission-only
non-claims, see
[`docs/foreign-derivation-import-trust-model.md`](docs/foreign-derivation-import-trust-model.md).
For the typed component manifest, exact `wkg.lock`/package handoff, pinned Octet
rail, normalized portable bytes, optional target-specific Wasmtime AOT output,
consumer remeasurement, and Wizer core-module boundary, see the WebAssembly
component section in [`docs/operator-workflows.md`](docs/operator-workflows.md).

For self-build, Cargo-free fixed-point, Nix-free demo-bundle proof operations,
and foreign import receipt trust-model links, see
[`docs/operator-proof-guide.md`](docs/operator-proof-guide.md). For focused
native `rust-plan` serial, parallel, and ambient-env validation rails, see
[`docs/native-rust-plan-validation.md`](docs/native-rust-plan-validation.md).
For the disabled-by-default KernelScript beta profile, authoritative source pin,
pure offline planning/admission core, separate pinned probe/VM observation,
exact blockers, upgrade procedure, and strict non-claims, see
[`docs/kernelscript-experiment.md`](docs/kernelscript-experiment.md).
For repeatable Cairn change validation, sync/archive, post-archive validation,
and final status evidence capture, see
[`docs/cairn-lifecycle-evidence-runner.md`](docs/cairn-lifecycle-evidence-runner.md).
For project-facing name rules and exact legacy compatibility exceptions, see
[`docs/mantle-naming.md`](docs/mantle-naming.md). Keep the proof guide current
with:

```bash
nix develop -c cargo -Zscript scripts/check-operator-proof-guide.rs
nix develop -c cargo -Zscript scripts/check-foreign-import-trust-model.rs
nix develop -c cargo -Zscript scripts/check-foreign-import-trust-model.rs --self-test
nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs
nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs --self-test
```

Named shell profiles may be declared under `shells`. Mantle resolves
`mantle shell` to an explicit default, then `dev`, then `default`, and profile
values may either be shell derivations directly or records with a `derivation`
field plus profile metadata. Shell activation is non-mutating convenience
evidence only: it does not refresh locks, regenerate files, change package build
identity, start services, or prove release reproducibility.

Generated files are declared under `files` and materialized only through
`mantle filegen apply` after a reviewed `mantle filegen plan`. The plan/apply
rail records BLAKE3 content identity, rejects target escapes and unmanaged
conflicts, and treats typed-content validation as a bounded generated-content
claim rather than deployability or build proof.

`mantle export` is the explicit Nickel export primitive. It evaluates declared
relative Nickel sources/import paths to JSON, can write an explicit `--out`, and
emits a deterministic receipt binding source/dependency digests, evaluator
identity, output digest, and bounded non-claims. Evaluator-neutral admission,
BLAKE3 identity, freshness, and the Mantle v1 projection delegate to
`nickel-export-core` at exact revision
`257fafc1c746f1faf156207043a4c826bfb16d49`; `crunch-eval`, no-follow
filesystem/root admission, destination writes, build evidence, and release
authority remain Mantle-owned. See
[Standalone Nickel export core](docs/nickel-export-core-cutover.md).

## Validation tiers

Use the checked-in toolchain from [`rust-toolchain.toml`](rust-toolchain.toml)
for all validation commands.

**Ordinary first-party gate**

Run the checked-in wrapper from the repo root:

```bash
./scripts/check-first-party-quality.sh
```

That wrapper runs three ordinary edit-time checks in order:

```bash
cargo fmt --check \
  -p mantle \
  -p crunch-attestation \
  -p crunch-build \
  -p crunch-delta \
  -p crunch-eval \
  -p crunch-glue \
  -p crunch-pipeline \
  -p crunch-project \
  -p crunch-shell \
  -p crunch-store
./scripts/check-first-party-clippy.sh
cargo test --workspace --lib --tests
```

The root `-p mantle` rustfmt leg covers the root package's `src/`,
`examples/`, and `tests/`, including `tests/benchmark_harness.rs`.
The strict clippy helper excludes vendored workspace members
`fuse-backend-rs`, `nix-compat`, `nix-compat-derive`, `snix-build`,
`snix-castore`, `snix-store`, and `snix-tracing` so first-party warnings fail
cleanly.

**Pinned ast-grep structural rail**

The default development shell includes ast-grep `0.42.1`. The standalone
profile carries a generated BLAKE3 executable identity record, and its focused
smoke recomputes that identity before accepting the package:

```bash
nix run .#ast-grep-toolchain -- --version
nix build .#checks.x86_64-linux.ast-grep-package-identity --no-link -L
```

See [`docs/ast-grep-structural-evidence.md`](docs/ast-grep-structural-evidence.md)
for repository-owned scan/rule-test sidecars and claim boundaries.

**Tigerstyle lane**

Run the repo-pinned Tiger Style consumer check when you want the structural lint
pass:

```bash
./scripts/check-first-party-tigerstyle.sh
```

That wrapper delegates to the flake-pinned `cargo-tigerstyle` runner:

```bash
nix run .#tigerstyle -- check
```

Default scope comes from `[workspace.metadata.tigerstyle]` in `Cargo.toml`, so
vendored workspace members stay out of the lint pass. Workspace-specific lint
rollout config lives in `dylint.toml`.

**Heavyweight rails**

Keep these heavier checks separate from the ordinary gate:

```bash
cargo test -p crunch-pipeline --test integration_build \
  pipeline_determinism_probe_ -- --ignored --nocapture
./scripts/prove-self-hosting.sh --check
./scripts/check-mantle-transcript-quality.sh
./scripts/check-release-determinism-quality.sh
./scripts/check-release-nix-witness-quality.sh
nix build .#checks.x86_64-linux.mantle-transcript-quality --no-link -L
nix build .#checks.x86_64-linux.release-determinism-quality --no-link -L
nix build .#checks.x86_64-linux.release-nix-witness-quality --no-link -L
```

The Mantle transcript rail runs the executable-transcript parser/runner tests and
fast Markdown fixtures, and is exposed as both
`packages.<system>.mantle-transcript-quality` and
`checks.<system>.mantle-transcript-quality` so transcript examples get a maintained
local/CI entrypoint before they graduate into release walkthroughs. The release
determinism quality rail runs the generated deterministic proof
smoke and validates its receipt/log BLAKE3 in one checked-in entry point. The
flake check is the Nix/CI-callable heavy gate for the same underlying generated
proof regression. The Nix witness rail exercises the bounded `mantle release
nix-witness` CLI path and is exposed as both
`packages.<system>.release-nix-witness-quality` and
`checks.<system>.release-nix-witness-quality`, keeping it opt-in rather than part
of ordinary developer builds. Deterministic verification-gauntlet report schemas
and CLI aggregation are documented in
[`docs/verification-gauntlets.md`](docs/verification-gauntlets.md). `bootstrap
parity-report` consumes the compact release-rebuild descriptor in
`bootstrap/evidence/real-self-build-proof-parity.json` only when it carries
`mantle-deterministic-proof-receipt-v2`, genuine rebuild authority, and descriptor
and authority-plan BLAKE3 identities. Accepted evidence marks
`crunch.self-build` partial without completing Guix or StageX parity; missing or
legacy v1 evidence marks it blocked. The checked descriptor is currently legacy
v1 and therefore blocked until refreshed from a genuine release rebuild. The
determinism probe is an ignored integration rail,
not part of every edit. `./scripts/prove-self-hosting.sh --check` is only
self-hosting preflight. The full ignored proof run stays heavier:

```bash
./scripts/prove-self-hosting.sh
```

**Vendored maintenance lane**

Workspace-wide vendored upkeep stays separate from the first-party gate.
Do not treat vendored clippy debt as an ordinary edit blocker for tracked mantle
code.

If you bypass the checked-in wrappers and run direct compile-heavy `cargo`
commands, use the same PATH / `PKG_CONFIG_PATH` / `SNIX_BUILD_SANDBOX_SHELL`
prerequisites described in the self-hosting section below, and put `TMPDIR` and
`CARGO_TARGET_DIR` on disk-backed scratch before treating failures as code
failures.

## Examples

The repo ships a supported examples gallery under [`examples/`](examples/). The source of truth is [`examples/catalog.ncl`](examples/catalog.ncl), which classifies support tier, prerequisites, network use, and validation rails. Start with [`examples/README.md`](examples/README.md) for the progressive guide.

Fast local examples:

- [`examples/hello.ncl`](examples/hello.ncl) — smallest derivation
- [`examples/multi-step.ncl`](examples/multi-step.ncl) — multi-line output using shell builtins only
- [`examples/build-environment.ncl`](examples/build-environment.ncl) — declarative environment values persisted in a directory artifact
- [`examples/cowsay.ncl`](examples/cowsay.ncl) — runnable cowsay-compatible script with default and custom messages
- [`examples/static-site.ncl`](examples/static-site.ncl) — HTML and CSS directory artifact without network access
- [`examples/multiple-roots.ncl`](examples/multiple-roots.ncl) — independent top-level derivations in one build
- [`examples/local-output-layout.ncl`](examples/local-output-layout.ncl) — named output layout without generated seed material
- [`examples/dependency-chain.ncl`](examples/dependency-chain.ncl) — a producer derivation consumed by a root derivation
- [`examples/selected-output.ncl`](examples/selected-output.ncl) — one selected named output mounted downstream
- [`examples/diamond-dependency.ncl`](examples/diamond-dependency.ncl) — a shared dependency reused by converging branches
- [`examples/projects/generated-site/mantle-project.ncl`](examples/projects/generated-site/mantle-project.ncl) — complete generated-site package with a content check
- [`examples/projects/codegen-pipeline/mantle-project.ncl`](examples/projects/codegen-pipeline/mantle-project.ncl) — model-to-generated-source application project
- [`examples/projects/reproducible-release/mantle-project.ncl`](examples/projects/reproducible-release/mantle-project.ncl) — independently built normalized release archives with BLAKE3 and tamper checks
- [`examples/projects/locked-dependency-lifecycle/mantle-project.ncl`](examples/projects/locked-dependency-lifecycle/mantle-project.ncl) — offline check, stale detection, selected refresh, and lock upgrade lifecycle
- [`examples/projects/offline-source-bundle/mantle-project.ncl`](examples/projects/offline-source-bundle/mantle-project.ncl) — fresh-state source bundle export, pinned import, preflight, and tamper rejection
- [`examples/projects/reviewed-file-generation/mantle-project.ncl`](examples/projects/reviewed-file-generation/mantle-project.ncl) — non-mutating filegen planning and drift-checked reviewed apply
- [`examples/projects/developer-shell-run/mantle-project.ncl`](examples/projects/developer-shell-run/mantle-project.ncl) — runnable package with named `dev` and `minimal` shell profiles
- [`examples/fail.ncl`](examples/fail.ncl) — intentional failure for diagnostics

Cookbook and advanced examples:

- [`examples/fetch-crate-crc64.ncl`](examples/fetch-crate-crc64.ncl) — fetch a real crates.io source tarball (`crc64` 2.0.0)
- [`examples/build-crate-crc64.ncl`](examples/build-crate-crc64.ncl) — build that real crate with Mantle's bootstrap Rust toolchain and shared reduced seed provider
- [`examples/build-from-source.ncl`](examples/build-from-source.ncl) — build a multi-file C project with `make`
- [`examples/projects/c-library-cli/mantle-project.ncl`](examples/projects/c-library-cli/mantle-project.ncl) — fixed local C sources built as a tested library and CLI project
- [`examples/projects/rust-workspace/mantle-project.ncl`](examples/projects/rust-workspace/mantle-project.ncl) — dependency-free multi-package Rust workspace built with offline Cargo
- [`examples/projects/fetched-and-patched/mantle-project.ncl`](examples/projects/fetched-and-patched/mantle-project.ncl) — pinned crates.io source with BLAKE3-fixed positive and negative patches
- [`examples/projects/multi-output-sdk/mantle-project.ncl`](examples/projects/multi-output-sdk/mantle-project.ncl) — C SDK split into runtime, development, documentation, and debug outputs
- [`examples/projects/schema-codegen/mantle-project.ncl`](examples/projects/schema-codegen/mantle-project.ncl) — one bounded schema generating tested C and Rust applications
- [`examples/projects/signed-cache-roundtrip/mantle-project.ncl`](examples/projects/signed-cache-roundtrip/mantle-project.ncl) — signed publication and fresh-store substitution with untrusted/corrupt negative paths
- [`examples/projects/cross-compiled-host-tool/mantle-project.ncl`](examples/projects/cross-compiled-host-tool/mantle-project.ncl) — host code generator separated from a musl target compiler and artifact role
- [`examples/projects/store-gc-lifecycle/mantle-project.ncl`](examples/projects/store-gc-lifecycle/mantle-project.ncl) — persistent roots, dry-run collection, garbage collection, and mutation locking
- [`examples/projects/delta-substitution/mantle-project.ncl`](examples/projects/delta-substitution/mantle-project.ncl) — partial chunk reuse, full fallback, and fail-closed sender-data validation
- [`examples/projects/release-witness-handoff/mantle-project.ncl`](examples/projects/release-witness-handoff/mantle-project.ncl) — canonical signed attestation handoff, witness quorum, wrong-release, and revocation policy paths
- [`examples/bootstrap-no-nix.ncl`](examples/bootstrap-no-nix.ncl) — compile C with the shared reduced bootstrap seed provider
- [`examples/project/crunch.ncl`](examples/project/crunch.ncl) — project-aware `mantle build .#name` layout
- [`examples/hardware_simulation_plan.rs`](examples/hardware_simulation_plan.rs) — frontend-owned hardware request lowering to generic `mantle-plan-v1`; real Verilator execution is capability-gated
- `mantle --json build examples/hello.ncl` — local build report with artifact attestation sidecar references; not a release or witness proof

## Benchmark suite

Checked-in benchmark entry points live under [`examples/`](examples/):

- `cargo run --example benchmark_eval_smoke -- --bundle-out target/benchmarks/eval-smoke.json --repeat-count 2`
- `cargo run --example benchmark_suite -- --bundle-out target/benchmarks/suite.json --repeat-count 2`
- `cargo run --example benchmark_compare -- --baseline target/benchmarks/baseline.json --fresh target/benchmarks/suite.json --absolute-threshold-ns 1000 --percent-threshold 5`
- `cargo run --example benchmark_scheduler_priority`

The smoke path keeps local checks cheap. The full matrix covers evaluation,
conversion, substitution planning, and build-graph preparation using checked-in
fixtures only. The compare entry point matches workloads by stable name and
highlights the largest regressions or wins. See
[`docs/benchmark-suite.md`](docs/benchmark-suite.md).

## Fetchers

Download files, tarballs, and git repos as fixed-output derivations. These are
the normal network boundary: the fetcher action records URL, hash mode, expected
digest, and retry policy, and downstream ordinary builds consume only the
verified store output with network disabled by default.

```nickel
let mantle = import "lib.ncl" in

# Single file
mantle.fetchurl {
  url = "https://example.com/foo.tar.gz",
  hash = "sha256-...",
}

# Tarball (auto-decompress + unpack + strip top-level dir)
mantle.fetchTarball {
  url = "https://github.com/user/repo/archive/v1.0.tar.gz",
  hash = "sha256-...",
}

# Git repository at a specific rev
mantle.fetchGit {
  url = "https://github.com/user/repo.git",
  rev = "abc123...",
  hash = "sha256-...",
}
```

Hash mismatches show the correct hash. Use `--fix` to auto-update:

```bash
mantle build --fix hello.ncl
```

Supported hash algorithms: sha256, sha512, sha1, md5, blake3.
Supported compression: gzip, xz, bzip2, zstd.

## Content-Addressed Derivations

The default `addressing_mode` is `'content-addressed`: output paths are
computed from the build output content (BLAKE3 hash of the NAR), not from
the derivation inputs. Identical outputs get identical paths.

```nickel
# CA is the default — no opt-in needed
{ name = "my-tool", builder = "...", ... } | mantle.Derivation

# Opt into input-addressed if needed
{ name = "my-tool", addressing_mode = 'input-addressed, ... } | mantle.Derivation
```

CA self-reference rewriting works: if a build output embeds its own
store path (RPATHs, shebangs), the provisional path is rewritten to
the final content-addressed path. Multi-output CA derivations use
two-pass rewriting to handle cross-output references.

## Multi-Output Derivations

Split a package into separate outputs (binary, headers, man pages):

```nickel
{
  name = "hello",
  outputs = ["out", "dev", "man"],
  builder = "/bin/bash",
  args = ["-c", m%"
    for output in $outputs; do
      mkdir -p "${!output}"
    done
    mkdir -p $out/bin && echo 'hello' > $out/bin/hello
    mkdir -p $dev/include && echo '#define V 1' > $dev/include/hello.h
    mkdir -p $man/share/man/man1 && echo '.TH HELLO 1' > $man/share/man/man1/hello.1
  "%],
  ...
} | mantle.Derivation
```

Each output gets a distinct store path. The builder receives `$outputs`
(space-separated list) and individual path variables (`$out`, `$dev`,
`$man`). Works with both input-addressed and content-addressed modes.

To depend on a single output of a multi-output package, use
`mantle.select`:

```nickel
let mantle = import "lib.ncl" in
# Only mount the `dev` output of libfoo (not `out` or `lib`)
{
  name = "consumer",
  inputs = [
    mantle.select libfoo "dev",
    mantle.select libfoo "lib",
  ],
  ...
} | mantle.Derivation
```

Multiple selections from the same dependency are coalesced into a single
`input_derivations` entry. A bare derivation in `inputs` mounts all its
outputs.

## Architecture

```
.ncl file
    │
    ▼
crunch-eval (nickel-lang)     Evaluate Nickel → record
    │
    ▼
crunch-glue (nix-compat)      Record → nix_compat::Derivation + BLAKE3 store paths
    │
    ▼  mpsc channel (EvalMessage per root)
    │
crunch-build (goal + worker)  Lazy goal scheduler → bwrap sandbox → PathInfo
    │                         (or builtin fetcher for fetchurl/fetchTarball/fetchGit)
    ▼
/nix/store/<hash>-<name>      Output on disk (--store default)
```

The scheduler is a lazy goal-based system (not an eager DAG). Each
derivation becomes a `Goal` with a state machine
(Pending→Waiting→Ready→Building→Done). The `Worker` creates goals on
demand, deduplicates by store path, and dispatches concurrent builds
via `JoinSet` + `Semaphore`. Roots arrive over an mpsc channel, so the
Worker can start building leaf deps while later roots are still being
processed.

Eligible ready goals are not dispatched by arrival-order FIFO. Mantle ranks a
validated snapshot through a deterministic lexicographic policy: operator
class, bounded starvation class, known-graph blocked-root/critical-path
pressure, provider-neutral resource fit, verified content locality/transfer
cost, then stable goal identity. The path estimate is deliberately limited to
the graph known at that scheduling epoch; it is not a global critical path and
does not assume future streamed or dynamic goals. Hard capability, trust,
upload, network, store-prefix, and resource constraints run before preference,
so age or locality can never make an ineligible route dispatchable.

The runtime default is typed by [`lib/scheduling.ncl`](lib/scheduling.ncl) and
carried explicitly in `BuildConfig`. JSON build reports include the selected
`scheduler_policy` plus bounded, redacted `scheduler_priority_decisions`;
`--verbose` human output prints a
bounded summary. Evidence identifies policy and history digests, normalized
classes, epoch/age, known-graph pressure, and stable reason codes while omitting
raw goal paths and provider secrets. It proves only the configured known-fact
ordering—not global makespan optimality, future-graph knowledge, execution
success, output trust, or release reproducibility. See
[ADR 0001](adr/0001-lazy-goals-vs-eager-dag.md) and
[ADR 0014](adr/0014-deterministic-lazy-goal-priority.md). A comparative debug
fixture report (chain, diamond, shared dependency, locality, fairness, and
bounded ready-set stress) is produced with
`cargo run -p mantle --example benchmark_scheduler_priority`; its timings are
ready-selection overhead evidence for those fixtures (excluding graph snapshot
and pressure recomputation), not production-throughput or global optimality
claims.

The goal system is extensible: substitution goals, native dynamic plans,
and remote build dispatch can be added without restructuring the scheduler.

### Native Dynamic Plans

Mantle's core dynamic-build API is `mantle-plan-v1`: a bounded canonical JSON
artifact produced by a sandboxed build output that was explicitly declared in
the producing derivation's `dynamic_plan_outputs` list. After the producer
finishes, the worker reads only those declared outputs, validates the plan,
records BLAKE3 raw/canonical digests, registers accepted units, and calls
`want()` only for the plan roots. The existing lazy scheduler then builds the
root dependency closure in the same run.

Native plan report rows appear in JSON build reports under
`native_dynamic_plans`. Each row identifies the producer derivation key, declared
output name, plan artifact path when present, digest fields, accepted unit IDs,
rejection reason when validation fails, scheduler action, and `mode = "native"`.

Build outputs that happen to look like `.drv` files are still supported as a
compatibility/debug discovery path. That path is separate from the native ABI:
undeclared plan-looking outputs are ignored by native plan scanning, and `.drv`
discovery must be treated as compatibility behavior rather than Mantle's core
dynamic-plan interface.

The [hardware simulation reference slice](docs/hardware-simulation.md) exercises
typed frontend-owned semantics against this generic boundary. Its accepted
13-action generation/compile/link/smoke graph is also composed with the
[provider-free external batch fixture](docs/external-batch-dispatchers.md):
ordinary resource/locality placement, worker registration, CAS transfer,
signed output admission, and shared-result checks remain generic, with no
HDL-specific worker or scheduler logic.

## Crate Layout

| Crate | Role |
|---|---|
| `mantle` | CLI binary — build, eval, bootstrap, store, log, project management, self-build |
| `crunch-eval` | Nickel evaluation, stdlib embedding |
| `crunch-glue` | `CrunchDerivation` → `nix_compat::Derivation`, ConversionCache |
| `crunch-build` | `Derivation` → `BuildRequest`, goal scheduler, build orchestration, fetchers |
| `crunch-hardware-simulation-core` | `no_std` typed hardware profile, plan, smoke, and evidence logic |
| `crunch-hardware-simulation` | bounded std adapter for fixture Git, tool observation, and strict sandbox actions |
| `crunch-pipeline` | eval → deserialize → convert → build wiring |
| `crunch-project` | Project manifest, lockfile, refresh, stale detection, upgrade |
| `crunch-store` | Store export, closure resolution, StoreHandle, signing, query |
| `vendor/nix-compat` | Store paths, ATerm, NAR (BLAKE3-modified) |
| `vendor/snix-build` | `BuildService` trait, bwrap sandbox |
| `vendor/snix-castore` | Blob and directory content-addressed storage |
| `vendor/snix-store` | `PathInfoService`, NAR calculation |

## Nickel Stdlib

The stdlib (`lib/`) defines the derivation schema and fetch helpers:

```nickel
let mantle = import "lib.ncl" in
{
  name = "myapp",              # | Name (validated)
  builder = "/bin/sh",         # | String
  system = 'x86_64-linux,      # | System enum (default)
  args = ["-c", "..."],        # | Array String (default [])
  outputs = ["out"],           # | Array String (default ["out"])
  env = { CC = "gcc" },        # | { _ : String } (default {})
  inputs = [                   # | Array Input (default [])
    "/nix/store/...-bash",     #   StorePath string → source input
    { name = "lib", ... },     #   Derivation record → built first
  ],
  fixed_output = {             # | FixedOutput | optional
    hash = "sha256-...",
    algo = 'sha256,            #   sha256 | sha512 | sha1 | md5 | blake3
    mode = 'flat,              #   flat | recursive
  },
  addressing_mode              # | default = 'content-addressed
    = 'content-addressed,      #   content-addressed | input-addressed
  sandbox                      # | default = 'native
    = 'native,                 #   'native today; 'oci and 'wasm are reserved for future backends
} | mantle.Derivation
```

Contracts catch errors at eval time:
- Missing required fields (`name`, `builder`)
- Invalid store paths, derivation names
- Wrong enum variants

## How It Differs From Nix

| | Nix | mantle |
|---|---|---|
| Config language | Nix | Nickel |
| Derivation hash | SHA-256 | BLAKE3 |
| Default addressing | Input-addressed | Content-addressed |
| Default store prefix | `/nix/store` | `/mantle/store` (`--nix-compat` for `/nix/store`; explicit `/crunch/store` remains legacy-compatible) |
| Inputs | String context (implicit) | Explicit `inputs` field |
| Hash algorithms | sha256/sha512/sha1/md5 | + blake3 (first-class) |
| Sandbox | Nix sandbox | bwrap (via snix-build) |
| Closure resolution | `nix-store -qR` | Own PathInfo walk (no Nix required) |
| Serialization | protobuf/gRPC | postcard |

## Store Paths and Prefixes

mantle separates two concepts:

- **Logical prefix** (`--store-prefix`, default `/mantle/store`): used for
  derivation hash computation, ATerm serialization, and output path names.
  Different prefixes produce different derivation hashes.
- **Physical directory** (`--store`, default `/nix/store`): where mantle
  writes build outputs on disk. Source inputs (seed packages) are always
  read from `/nix/store`.

```bash
# Default: logical paths under /mantle/store, outputs written to /nix/store
mantle build hello.ncl

# Write outputs to a custom directory
mantle build --store /tmp/mystore hello.ncl

# Nix-compatible hashes (for interop testing)
mantle build --nix-compat hello.ncl
```

`--nix-compat` is shorthand for `--store-prefix=/nix/store`.

## Signing and Trust

mantle signs PathInfo entries with ed25519 keys. If no `--signing-key` is
provided, an auto-generated key is created at
`$CRUNCH_CONFIG_DIR/signing-key` (or `$state_dir/signing-key` when
`CRUNCH_CONFIG_DIR` is unset; default state dir is
`~/.local/state/mantle`).

```bash
# Build with an explicit signing key
mantle build --signing-key ./my-key hello.ncl

# Verify signatures against trusted public keys
mantle store verify --trusted-public-keys "mykey-1:base64pubkey..."

# Sign all unsigned PathInfo entries (migration from unsigned stores)
mantle store sign --all

# Accept unsigned PathInfo on cache hits (escape hatch)
mantle build --trust-unsigned hello.ncl
```

By default, cached PathInfo must be signed by a trusted key. The default
trust set includes `cache.nixos.org-1`. Use `--trusted-public-keys` to
override.

### Retained roots and manual GC

Successful top-level outputs from `mantle build`, `mantle self-build`, and
`mantle bootstrap --fetch` are retained automatically as GC roots. Use:

```bash
mantle store roots
mantle store pin /nix/store/<hash>-name
mantle store unpin /nix/store/<hash>-name
mantle store gc --dry-run
mantle store gc
```

`mantle store gc --dry-run` reports the retained-root count, candidate path
count, reclaimable bytes, and each candidate logical store path without
mutating state. Real GC sweeps unreachable exported outputs, PathInfo rows,
attestation sidecars, and unreachable castore content. GC fails closed if a
retained root's reachability metadata is missing or unreadable.

Current safety bounds are internal and fail closed rather than partially
collect: exported-output byte walks stop at depth 128 or 100,000 visited
entries, and filesystem/blob scans stop at 200,000 entries.

## Binary Cache Sharing

Share build results between machines using the Nix binary cache directory
layout (`.narinfo` + `.nar` files). Push exports from the local store;
pull imports into it.

```bash
# Push all signed paths to a cache directory
mantle store push --all --to /srv/cache

# Push specific paths
mantle store push --to /srv/cache /mantle/store/<hash>-hello

# Include unsigned entries (normally skipped)
mantle store push --all --to /srv/cache --trust-unsigned

# Pull all paths from a cache directory
mantle store pull --all --from /srv/cache

# Pull specific paths from a cache directory
mantle store pull --from /srv/cache /mantle/store/<hash>-hello

# Pull specific paths from an HTTP cache
mantle store pull --from https://cache.example.com /mantle/store/<hash>-hello

# Pull from HTTP with explicit trust (signature verification)
mantle store pull --from https://cache.example.com \
  --trusted-public-keys "builder-1:base64pubkey..." \
  /mantle/store/<hash>-hello

# Accept unsigned narinfos
mantle store pull --all --from /srv/cache --trust-unsigned
```

Push writes `nix-cache-info` into the target directory if absent, so the
result is directly servable by `nix-serve`, `harmonia`, nginx, or S3
sync. Paths already present in the target are skipped.

Pull verifies narinfo signatures against the configured trusted keys
(same trust set as `mantle build --substituters`). Paths that fail
signature verification, have a store-directory prefix mismatch, or whose
NAR content does not match the declared hash are skipped with a warning.
HTTP pull is explicit-path only: `--all` works for directory caches, but
HTTP/HTTPS sources require one or more logical store paths on the command
line.

Both commands acquire the store mutation lock, so they are safe alongside
concurrent builds on the same state directory.

Typical CI workflow:

```bash
# Builder machine: build and publish
mantle build my-package.ncl
mantle store push --all --to /shared/cache

# Consumer machine: import pre-built results
mantle store pull --all --from /shared/cache \
  --trusted-public-keys "ci-builder-1:base64pubkey..."
mantle build my-package.ncl   # cache hit, no rebuild
```

## Bootstrap

mantle needs some trusted inputs to build anything. The repo tracks four
different bootstrap paths, and they do not all prove the same thing.

### Bootstrap maturity levels

- **Seed-assisted bootstrap**: mantle can start from declared external seeds
  and build later bootstrap stages.
- **Self-hosting proof**: a mantle-built `mantle` can rebuild `mantle` from
  the same staged source tree.
- **Packaged release evidence**: a release bundle carries the exact binary,
  source archive with tracked worktree files plus verified vendored Cargo inputs,
  proof bundle, and prerequisite inventory so
  later verification can check bundle-local integrity and proof linkage.
- **Full-source bootstrap**: the trusted root has been reduced to small,
  explicitly audited source or bootstrap seeds.
- **Reproducible release evidence**: independent rebuilds produce the same
  final artifact and can be compared or signed.

Today the repo has seed-assisted bootstrap, a checked-in self-hosting proof,
and packaged release evidence. It does not yet claim a full-source bootstrap
or bit-for-bit reproducible release outputs.

### StageX-class no-quorum target

The repo defines a `stagex-verified-no-quorum` release profile that requires
a fully auditable seed-to-binary chain without trusting host compilers.
It is stricter than seed-assisted or source-root evidence:

- The bootstrap provider must be `stagex-lineage` — the only accepted seed
  class is `hex0-seed` (a small hand-audited byte seed, max 4096 bytes).
- The self-build proof must include StageX lineage metadata: seed/lineage
  manifest/stage-graph/provider/staged-source/stage1/stage2/bootstrap-tool
  digests plus a protected-exec audit digest, all lowercase BLAKE3 hex.
- Protected execution must not observe undeclared host compiler, build tool,
  archive tool, Nix command, or legacy provider executables.
- A verified reproducibility report must be present and matched.
- Quorum policy is intentionally deferred: `quorum_status` is always
  `not_evaluated`. The profile never emits `quorum-satisfied`.

Seed-assisted bootstrap, source-root evidence, and self-proof-only evidence
do not satisfy this profile. External witness agreement alone does not
satisfy it either — StageX-class lineage proof is required.

Remaining environmental assumptions not yet eliminated:

- The Rust compiler used for stage0 is a fetched stable binary, not
  bootstrapped from the hex0 seed.
- The Linux kernel and FUSE/bwrap sandbox runtime are trusted host components.
- Network transport for bootstrap artifact fetches is trusted.

Verify with: `mantle release verify <bundle-dir> --require-stagex-no-quorum`

### Trust inventory by entry point

For the stricter stage0 view, see [`docs/bootstrap-stage0-inventory.md`](docs/bootstrap-stage0-inventory.md).
That inventory groups first-bootstrap dependencies into host prerequisites,
pinned fetched artifacts, mantle-built outputs, and host-convenience probes.

#### `mantle bootstrap`

- **Current claim**: seed-assisted bootstrap from an existing Nix installation.
- **Trusted inputs today**: the host's Nix tooling and the `/nix/store` paths
  returned by `nix-build` or `nix build`.
- **Host prerequisites**: Nix installed, selected packages available in
  `nixpkgs`, and a machine where those store paths remain reachable.
- **Evidence today**: `mantle bootstrap -o seed.ncl ...` writes a
  `StorePath`-validated seed file, and later builds mount the declared source
  closures in the sandbox.
- **Not yet proven**: the imported Nix seed is not reduced to an auditable
  minimal root inside mantle itself.

#### `mantle bootstrap --fetch`

- **Current claim**: seed-assisted bootstrap without Nix.
- **Trusted inputs today**: the pinned musl.cc native tarball declared in
  `bootstrap/seed.ncl`, plus the checked-in reducer that turns it into the
  normalized `musl-seed-toolchain` provider.
- **Host prerequisites**: Linux, network access to fetch the raw tarball,
  and a working mantle binary with enough local disk for fetched outputs.
- **Evidence today**: the fetch path evaluates `bootstrap/seed.ncl`, builds
  the reduced provider, writes `seed.ncl`, and installs provider provenance at
  `<seed>/share/crunch-bootstrap/provider.json`.
- **Not yet proven**: the reduced provider is still derived from a trusted
  binary tarball, not a source-built root.

#### `mantle self-build`

- **Current claim**: seed-assisted self-build from the current checkout.
- **Trusted inputs today**: the current source tree, the source-tree
  `vendor-deps/` directory and `.cargo/vendor-config.toml` validated against
  `Cargo.lock` and Cargo's `.cargo-checksum.json` metadata, the reduced
  `musl-seed-toolchain` provider derived from the pinned musl.cc tarball, and
  the host sandbox entry points used for the first bootstrap stage.
- **Host prerequisites**: `bwrap` and a static sandbox shell for the first
  bootstrap stage.
- **Evidence today**: `mantle self-build --store /tmp/mantle-store -j 4 --no-substitute`
  builds the bootstrap chain `seed -> make -> dash -> binutils -> musl -> gcc -> busybox -> bwrap -> rust -> mantle`.
- **Cargo-free topology mode**: `mantle self-build --cargo-free --out /tmp/mantle-cargo-free`
  builds the Mantle binary through the explicit verification lane
  `rust-plan --no-cargo-oracle --execute-topology`, writes the binary plus
  receipt evidence under `--out`, and fails if the Cargo guard is invoked. Keep
  `--out` outside the source root so evidence does not change native source
  digests. Use `mantle self-build --cargo-free --fixed-point --strict-hermetic --out /tmp/mantle-cargo-free`
  when the output is intended for proof admission rather than local diagnostics.
  The rust-plan receipt labels this as `cargo-free-bounded-topology` when
  supported, `blocked-unsupported-surface` when native planning stops at an
  unsupported surface, and always `not-default-project-build`. Current refreshed
  evidence is diagnostic unless it records strict proof admission; see
  `docs/operator-proof-guide.md` before reporting any Cargo-free fixed-point status.
- **Not yet proven**: this first build still relies on host tooling and the
  reduced seed provider, while Cargo-free topology mode is bounded Rust topology
  evidence rather than compiler correctness, release reproducibility, bootstrap
  correctness, or full Cargo compatibility.

#### `./scripts/prove-self-hosting.sh`

- **Current claim**: checked-in self-hosting proof.
- **Trusted inputs today**: a checkout-built stage0 `mantle` binary, the same
  reduced seed provider and host-tool assumptions as `mantle self-build`, and
  the staged source tree recorded by stage0.
- **Host prerequisites**: Linux, the repo's nightly Rust toolchain, `clang`,
  `mold`, `pkg-config`, OpenSSL development files, `bwrap`, `git`, `cargo`,
  a static sandbox shell, and about 4 GiB free in the proof scratch
  filesystem (`target/self-hosting-proof/work/` by default, or
  `CRUNCH_PROOF_SCRATCH_DIR`).
- **Evidence today**: the helper prepares the environment and runs
  `cargo test -p mantle --test self_hosting -- --ignored --nocapture`, which
  drives the stage0 -> stage1 -> stage2 proof path.
- **Not yet proven**: the proof still stops short of a full-source bootstrap
  root, independent rebuild agreement, or reproducible release artifacts.

### Bootstrappable Builds checklist

| Best-practice item | Status | Evidence today | Gap that remains |
|---|---|---|---|
| Provide an alternative way to build the build system | Yes | `cargo build --release` builds the checkout binary, and `mantle self-build` provides the in-repo bootstrap path | The bootstrap path still starts from host tooling and a reduced fetched seed provider |
| Label where bootstrap binaries or tarballs came from | Partial | `mantle bootstrap` names the Nix-backed path, and `mantle bootstrap --fetch` reuses the checked-in `bootstrap/seed.ncl` metadata and writes provider provenance into `provider.json` inside the fetched store path | The repo still trusts that reduced provider; it does not yet derive it from a smaller source bootstrap |
| Reproduce bootstrap binaries from source end-to-end | Not yet | The repo can build `make`, `dash`, `binutils`, `musl`, `gcc`, `busybox`, `bwrap`, `rust`, and `mantle` from the reduced seed provider | The reduced provider itself still comes from a trusted musl.cc binary tarball |
| Automate bootstrap traceability or self-hosting checks | Partial | `./scripts/prove-self-hosting.sh` runs the checked-in stage0 -> stage1 -> stage2 proof, `--check` verifies prerequisites first, successful runs write a proof bundle with `manifest.json`, `summary.txt`, and per-stage logs under `target/self-hosting-proof/`, and `mantle release create` can package that bundle with the release binary and a source archive containing tracked worktree files plus verified vendored Cargo inputs for later bundle-local integrity and proof-context checks with `mantle release verify` | The proof and release bundle still stop short of independent reproducibility evidence for release outputs |

## Self-Build

Source-root support is operation-specific. `mantle bootstrap capabilities`
reports live support: `mantle bootstrap --source-root <manifest.json>` is a
host-assisted source materialization path, while `mantle self-build
--source-root` is intentionally not accepted because no executable full-source
self-build provider chain exists. See
[`docs/source-root-capabilities.md`](docs/source-root-capabilities.md) for the
host influences and non-claims.

mantle can rebuild itself from source once the stage0 prerequisites are already present:

```bash
mantle self-build --store /tmp/mantle-store -j 4 --no-substitute
```

Pass `--strict-hermetic` when degraded hermeticity should fail instead of
warn during the self-build path.

This builds the full bootstrap chain (`seed -> make -> dash ->
binutils -> musl -> gcc -> busybox -> bwrap -> rust -> mantle`) inside a
bwrap sandbox. Output is a statically linked musl binary.

This is still seed-assisted bootstrap. It does not yet prove that the first
bootstrap works on a host with Nix commands absent from `PATH`.

First bootstrap requires the source tree with `vendor-deps/` and
`.cargo/vendor-config.toml`; `mantle self-build` validates the staged vendored
inputs against `Cargo.lock` and Cargo checksum metadata before building them.
It also requires `bwrap` and a static sandbox shell on `PATH`.
After the first self-build, the mantle-built `bwrap` and `busybox` are used
for subsequent builds.

### Proving self-hosting

To verify the fixed-point self-hosting claim, run the checked-in helper from
the repo root:

```bash
./scripts/prove-self-hosting.sh
```

The helper sets the nightly Rust toolchain, `CC`, linker or tool lookup,
`pkg-config` or OpenSSL lookup, and `SNIX_BUILD_SANDBOX_SHELL` before it
invokes the canonical ignored proof test:

```bash
cargo test -p mantle --test self_hosting -- --ignored --nocapture
```

Use `./scripts/prove-self-hosting.sh --check` to validate prerequisites,
including temporary-disk headroom, and print the proof command without
starting the full build.

Use `./scripts/prove-self-hosting.sh --non-nix-host` for the stricter proof
mode. That mode keeps the same stage0 -> stage1 -> stage2 fixed-point check,
but it also scrubs `nix-build`, `nix-store`, `nix-shell`, and `nix` from the
proof runner `PATH` before it invokes `cargo test`, so hidden Nix-command
fallbacks fail loudly.

Use `./scripts/prove-self-hosting.sh --no-host-tools --stage0-inventory <file>`
for the host-tool-free stage0 proof mode. The inventory is a concrete Nickel
file matching `bootstrap/stage0-inventory.ncl`; it must predeclare absolute
operator-supplied seed artifacts with BLAKE3 digests, bounded version evidence,
and provenance notes. Required executable seed roles are `sandbox-entry` and
`sandbox-shell`; additional allowed seed roles are `bootstrap-toolchain-tool` and
`bootstrap-build-tool`. In this mode the helper
captures an absolute Cargo path before proof PATH poisoning, then removes
`git`, `tar`, `cp`, `sh`, `cargo`, `bwrap`, and Nix commands from the proof
PATH used by stage0. The stage0 `self-build` command passes `--no-host-tools
--stage0-inventory <file>` and installs the protected exec supervisor before
sandbox startup.

The protected stage begins at no-host-tools stage0 `mantle self-build` entry and
ends only after mantle-built `bootstrap/bwrap.ncl` and `bootstrap/busybox.ncl`
outputs have been built, exported, verified, selected, and recorded in the proof
audit. Linux direct kernel interfaces allowed in that protected phase are the
seccomp user-notification listener, `execve`/`execveat` interception,
`/proc/<pid>/mem` reads for syscall path bytes, file metadata and content reads
needed to compute BLAKE3 digests, and normal process wait/exit reporting. Any
undeclared executable, digest mismatch, unsupported supervisor setup, relative
or unreadable exec path, or forbidden host helper fails closed before execution.

This helper still does not prove a full-source bootstrap root or reproducible
release artifacts. It proves either a fixed-point self-hosting rebuild, the same
rebuild under the stricter non-Nix-host command-availability contract, or the
host-tool-free stage0 boundary against explicitly declared seed artifacts.

Successful runs write a proof bundle to `target/self-hosting-proof/run-...`
and refresh `target/self-hosting-proof/latest` to point at that bundle. Pass
`--bundle-dir <dir>` to choose a different destination. Relative bundle paths
are anchored to the repo root before the helper exports them to the proof test
or updates `latest`.

The helper does not inherit ambient `TMPDIR` or `CARGO_TARGET_DIR`. It picks a
proof scratch root from `CRUNCH_PROOF_SCRATCH_DIR` when set, else from the
repo-local default `target/self-hosting-proof/work/`, then rewrites both env
vars under that root before it launches `cargo test`. `--check` reports the
selected scratch root, provenance, and free-space preflight result.

Host prerequisites:
- Linux
- `rustup` with the repo's `nightly` toolchain installed
- `clang`, `mold`, `pkg-config`, and OpenSSL development files visible to `pkg-config`
- `bwrap`, `git`, and `cargo`
- a static `SNIX_BUILD_SANDBOX_SHELL`
- about 4 GiB free in the selected proof scratch filesystem (`target/self-hosting-proof/work/` by default, or `CRUNCH_PROOF_SCRATCH_DIR`)

If `pkg-config --exists openssl` does not work in your current shell, export
`CRUNCH_PROOF_OPENSSL_PKGCONFIG=/path/to/openssl/lib/pkgconfig` before running
the helper.

If no installed static busybox is discoverable, set `SNIX_BUILD_SANDBOX_SHELL`
explicitly before running the helper. The helper no longer realizes one through
hidden `nix-build` fallback.

If you bypass the helper for a compile-heavy non-proof Cargo command, start
from the prerequisite PATH / `PKG_CONFIG_PATH` / `SNIX_BUILD_SANDBOX_SHELL`
setup documented in `AGENTS.md`, then move both temp files and Cargo artifacts
onto disk-backed scratch yourself:

```bash
export TMPDIR="$PWD/target/manual-work/tmp"
export CARGO_TARGET_DIR="$PWD/target/manual-work/cargo-target"
mkdir -p "$TMPDIR" "$CARGO_TARGET_DIR"
cargo test -p mantle --test self_hosting -- --list
```

That example is scratch guidance only. For self-hosting evidence, keep using
`./scripts/prove-self-hosting.sh`.

This runs two stages: the checkout binary builds stage1, then the stage1
binary rebuilds mantle as stage2 from the same staged source tree. The test
asserts that stage2 selected mantle-built `bwrap` and `busybox`, not host
fallbacks. Expect about 30 minutes and about 4 GiB free in the selected proof
scratch filesystem.

Each proof bundle contains:
- `manifest.json` — stable machine-readable digests for the checkout, stage1,
  and stage2 binaries, plus the stage2 `bwrap` and `busybox` paths and digests,
  store or state locations, copied stage metadata, proof mode, and recorded
  stage0 prerequisite paths
- `summary.txt` — a short human-readable digest summary, including protected
  execution audit digest and protected transition summary when present
- `binaries/stage1-mantle` and `binaries/stage2-mantle` — durable copies of
  the fixed-point binaries, so release packaging and witness rebuilds do not
  depend on temporary proof scratch paths surviving after the proof exits
- `stage0/` and `stage2/` — copied `meta.json`, `stdout.txt`, `stderr.txt`, and
  `diagnostics.txt` files from the stage audit bundles
- `stage0-prerequisites/inventory.md` — copied stage0 trust inventory used by
  the proof bundle
- `protected-exec-audit.json` — machine-readable protected execution audit with
  no-host-tools mode, optional stage0 inventory digest, declared seed roles,
  bounded version-evidence digests, blocked host command set, fallback markers,
  mantle-built sandbox transition records, and final result

The proof does demonstrate a fixed point: stage1 and stage2 must match
byte-for-byte, and the stage0 and stage2 mantle-built `busybox` and `bwrap`
outputs must match too. In `--non-nix-host` mode it also proves that the stage0
command path completed with `nix-build`, `nix-store`, `nix-shell`, and `nix`
absent from `PATH`. In `--no-host-tools` mode it additionally proves that
stage0 used declared seed sandbox artifacts under protected exec supervision and
that common host helper names were absent from the stage0 PATH. It does not yet
demonstrate bit-for-bit reproducible release artifacts or a full-source
bootstrap root.

### Release evidence bundles

Release tree inputs are fail-closed. Mantle accepts regular files, real
directories, and relative symlinks that lexically resolve inside the planned
input tree and name another planned entry. It rejects absolute, escaping,
unplanned, non-UTF-8, or otherwise unsupported symlinks and rejects sockets,
FIFOs, devices, and other special files.

Tree discovery, copying, hashing, and verification use no-follow entry metadata;
a symlink is hashed as its target text and is never traversed. Before mutation,
a pure planner validates deterministic ordering, parent-directory shape, unique
normalized paths, and the named release limits of 4,096 entries, 128 path
components, and 4,096 UTF-8 bytes per relative path. Destination operations run
beneath one capability-opened root, reject symlinked roots or parents, create
links only after directory and file writes, and revalidate source kinds and
modes at execution time.

This confinement proves bounded path and byte handling for bundle assembly. It
does not validate the semantics, correctness, or trustworthiness of copied proof
or release artifacts.

Function-address evidence uses the versioned Mantle-to-Cairn binding described
in [`docs/function-address-release-binding.md`](docs/function-address-release-binding.md).
Canonical Preserves sidecars use the opaque profile and manifest-driven CLI flow
documented in [`docs/function-address-preserves-sidecars.md`](docs/function-address-preserves-sidecars.md).
The receipt binds typed sidecar, Valence, optional Kamacite, source, and binary
identities without promoting them to Rust semantic or release-eligibility claims.

Trellis proof evidence uses the recorded-only opaque profile documented in
[`docs/trellis-proof-release-sidecars.md`](docs/trellis-proof-release-sidecars.md).
Mantle preserves Kamacite's `recorded-only` and `formal-proof-candidate` producer
roles, but current Valence authority counts Trellis imports only as
`recorded_only`; required accepted-proof mode therefore fails closed.

A successful proof bundle can be packaged as release evidence:

```bash
mantle release create \
  --release-id mantle-<version> \
  --binary target/self-hosting-proof/run-.../binaries/stage2-mantle \
  --proof-bundle target/self-hosting-proof/run-...
```

Repeat `--binary` when one release bundle should carry multiple executables.
If a separate byte-for-byte reproduction run has already produced a canonical
report, package it explicitly:

```bash
mantle release create \
  --release-id mantle-<version> \
  --binary target/self-hosting-proof/run-.../binaries/stage2-mantle \
  --proof-bundle target/self-hosting-proof/run-... \
  --reproducibility-report target/release-evidence/<release-id>/reproducibility/reproducibility-report.json
```

That command builds a staged-source tarball from the current tracked worktree
allowlist, the checked-in witness workflow driver (`scripts/prove-self-hosting.sh`),
the stage0 inventory document it consumes, the self-hosting test target/support
files needed by that driver, and verified vendored Cargo inputs, then copies the
release binary, proof bundle, and prerequisite inventory into a
new release-evidence bundle under `target/release-evidence/<release-id>/` by
default. The requested final directory must be absent: even a pre-created empty
directory is rejected. Mantle plans the deterministic artifact layout and input
BLAKE3 identities before mutation, assembles under a private capability-scoped
sibling, writes `manifest.json` last, runs normal production verification against
the stage, and publishes with one same-parent atomic no-clobber rename. Readers
therefore see no Mantle-created final path before commit and one complete verified
bundle afterward. A preexisting or concurrent file, symlink, empty directory, or
nonempty directory remains unchanged.

Failed attempts clean their current private stage when possible. A later retry
may quarantine only a sibling with the exact Mantle ownership marker, matching
publication-plan BLAKE3 identity, and matching final-name binding; similarly named
or malformed siblings are left untouched. Retry always uses fresh random staging
state, while that randomness is excluded from the deterministic plan identity.
Atomic rename establishes local no-clobber visibility, not filesystem crash
persistence or power-loss durability; no durability claim follows from rename
alone.

The top-level `manifest.json` records BLAKE3 digests for the source archive,
bundled binary or binaries, proof-bundle directory, prerequisite inventory, and
the proof-linkage facts copied from the full self-hosting proof.

Cairn lifecycle evidence can be measured and bound to that exact bundle with
`--cairn-handoff <descriptor.json>`. Assembly copies the measured artifact and
policy bytes under `cairn-handoff/`; verification remeasures them and rejects
stale bytes, role/schema swaps, removed required receipts, or receipts copied to
another bundle. The Onix release profile additionally requires this handoff and
strict deterministic/sandbox proof evidence:

```bash
mantle release verify target/release-evidence/<release-id> \
  --release-profile onix-stack \
  --deterministic-proof <receipt.json> \
  --deterministic-sandbox-isolation-evidence <isolation.json>
```

The handoff now requires and remeasures the pinned archived Cairn
`authenticate-stack-provenance-inputs` dependency and records
`archive-authentication-prerequisite-bound-v1`. This proves dependency and
bundle-local linkage only; Mantle does not independently claim producer
authorization, release, build, source, or deployment correctness. See
[`docs/cairn-release-handoff.md`](docs/cairn-release-handoff.md).

<!-- r[related mantle.release_provenance.opaque_boundary.visible] -->
<!-- r[related mantle.release_provenance.valence_required_policy] -->
Operators may also attach Valence stack-provenance sidecars as opaque external
evidence when Valence has already produced the stack graph report:

```bash
mantle release create \
  --release-id mantle-<version> \
  --binary target/self-hosting-proof/run-.../binaries/stage2-mantle \
  --proof-bundle target/self-hosting-proof/run-... \
  --stack-provenance-sidecar target/valence/stack-provenance-sidecar.json \
  --stack-provenance-valence-receipt target/valence/stack-provenance-graph-report.json

mantle release verify target/release-evidence/<release-id> \
  --stack-provenance required
```

When a bundle carries multiple `--binary` inputs, add `--stack-provenance-binary
<path>` so Mantle can bind the sidecar to exactly one packaged release binary.
`--stack-provenance optional` is the verify default and records an `absent`
disposition when no sidecar is bundled. `--stack-provenance required` fails
closed unless the bundle has the sidecar, the Valence graph report receipt,
matching BLAKE3 digests, expected roles/schemas, `identity-linkage-sidecar`
claim scope, release-binary identity, and the required non-claim boundary.
Mantle validates only bundle-local path, digest, role, schema, claim scope,
binary identity, and non-claims; Valence owns the stack semantics. This does not
prove Octet, Trellis, Valence, or Cairn behavioral correctness, verifier
soundness, release eligibility, or requirement satisfaction.

Generic external evidence remains available via `--external-evidence` plus
matching role/schema/scope/non-claim flags. Use `mantle release verify
--require-external-evidence-role <role>` when a downstream policy wants to
require a generic role without changing Mantle's default verification semantics.

Kani receipts use the same external-evidence boundary plus a dedicated optional
`--kani-toolchain-evidence` metadata file. Mantle links the bundled
`kani-model-check-receipt` sidecar to Kani, Rust, CBMC, solver, wrapper, and
closure identity facts while preserving Valence as the owner of Kani semantics;
see [`docs/kani-release-evidence.md`](docs/kani-release-evidence.md).

To produce the canonical byte-for-byte reproducibility report, run an explicitly
supported rebuild recipe into a clean output directory. The current supported
recipe identity is `mantle-release-reproducibility-v1`; unknown
`--workflow-version` values fail closed before execution.

```bash
mantle release reproduce target/release-evidence/<release-id> \
  --rebuild-output-dir /tmp/mantle-rebuild-out \
  --rebuild-command /path/to/rebuild-recipe
```

For a stronger deterministic-build proof attempt, ask `release reproduce` to run
at least two additional clean proof runs. Before launching either run, Mantle
creates a content-bound rebuild descriptor and authority plan over the exact
source archive, reviewed recipe, executable, tool/toolchain inputs, provider
identity, policy identities, and fresh run roots. The published target, every
content-identical copy or hardlink, every symlink, the complete release bundle,
prior proof outputs, and the ordinary reproduction output are excluded from read
authority. Undeclared inputs and reused or overlapping roots fail closed.

Each accepted proof run executes through `bwrap` with network denied and only the
approved regular-file capabilities mounted read-only. The complete release bundle
is deliberately absent. Only the per-run output tree and fresh store directory
(`MANTLE_DETERMINISTIC_PROOF_STORE_DIR`) are writable. The proof directory must
be separate from the ordinary output and all run roots. Unsupported sandboxing,
invalid input kinds, target aliases, or authority drift stop receipt emission.

```bash
mantle release reproduce target/release-evidence/<release-id> \
  --rebuild-output-dir /tmp/mantle-rebuild-out \
  --rebuild-command /path/to/busybox \
  --rebuild-arg sh \
  --rebuild-arg ./scripts/rebuild-release-artifacts.sh \
  --rebuild-arg /path/to/content-bound-toolchain.tar \
  --deterministic-proof-runs 2 \
  --deterministic-proof-dir /tmp/mantle-deterministic-proof
```

For a repo-maintained smoke rail that exercises the generated-proof path and
writes an operator receipt, run:

```bash
cargo -Zscript scripts/release-determinism-smoke.rs
```

The smoke rail runs the checked-in CLI regression that creates release evidence,
generates a `mantle-deterministic-proof-receipt-v2` receipt from two clean proof
stores with genuine rebuild authority, and verifies it with `mantle release verify
--require-deterministic-release`. It writes `receipt.json` and `test.log` under
`target/release-determinism-smoke/latest/` by default. Validate the rail receipt
schema and log BLAKE3 with:

```bash
cargo -Zscript scripts/check-release-determinism-smoke-receipt.rs \
  target/release-determinism-smoke/latest/receipt.json
```

The receipt is smoke evidence for the rail and does not replace a
release-specific proof receipt.

For a real release-specific rail, run the self-hosting proof, package the
stage2 mantle binary, produce two clean deterministic proof rebuilds under real
`bwrap`, and verify the generated proof in one command:

```bash
./scripts/prove-real-release-determinism.sh \
  --toolchain-archive /path/to/content-bound-toolchain.tar
```

Use `--proof-bundle target/self-hosting-proof/run-...` to reuse an existing full
self-hosting proof bundle. The required toolchain archive is an explicit
content-bound input; it is extracted only inside the fresh proof store by the
reviewed `scripts/rebuild-release-artifacts.sh` source-build recipe. The script
writes the release bundle,
`deterministic-build-proof.json`, sandbox evidence, verify receipt, and portable
`<release-id>-determinism-summary.{json,md}` under `target/release-evidence/`.
It validates the proof outputs with the checked-in real proof receipt checker
before writing the summary. Re-check a saved run with:

```bash
cargo -Zscript scripts/check-real-release-determinism-receipt.rs \
  target/release-evidence/<release-id>
```

Pass `--proof-dir` or `--verify-receipt` if the proof or verify JSON was moved
away from the default sibling paths. To write portable JSON and Markdown summary
artifacts for archival or review, run:

```bash
cargo -Zscript scripts/summarize-real-release-determinism.rs \
  target/release-evidence/<release-id>
```

That first runs the real proof receipt checker, then writes
`target/release-evidence/<release-id>-determinism-summary.{json,md}` with the
release id, provider kind, source/vendor BLAKE3, rebuild descriptor and authority
plan BLAKE3 identities, artifact digest set, proof/sandbox/verify BLAKE3 evidence,
`self-rebuild-match`, `eligible`, and explicit non-claims. The successful claim
is bounded to the packaged stage2 artifact rebuilding twice from those exact
content-bound inputs and fresh run roots under recorded
`mantle-proof-sandbox-v1:*` profiles with matching BLAKE3 digest sets. It does
not claim compiler/verifier soundness or full bootstrap reproducibility.

The report records a closed proof-class ladder:

- `bundle-consistent`: bundle-local digest/linkage checks only.
- `self-proof-valid`: the self-proof and report are valid, but the rebuild did
  not match or did not supply enough material for a rebuild-match claim.
- `self-rebuild-match`: a clean local rebuild produced the same named artifact
  set, sizes, and BLAKE3 digests.
- `external-witness-match`: an accepted external witness rebuild matched the
  published BLAKE3 artifact set.
- `policy-satisfied`: the configured witness/quorum policy was satisfied.

Rebuild-match classes require `comparison_verdict=matched`, matching per-artifact
BLAKE3 comparisons, clean rebuild store/output identities, replayable workflow
identity, environment assumptions, and BLAKE3 digests for the evidence artifacts
used to make the claim. A failed or missing comparison is represented as a failed
report with a weaker class; it is never promoted to a rebuild-match class.
Evidence marked `hermeticity-mode=impure` is also rejected for rebuild-match
proof classes by default, because explicit impure mode permits ambient host inputs
outside the declared proof boundary.

A stronger deterministic-release claim is separate from `self-rebuild-match`.
Mantle promotes only `mantle-deterministic-proof-receipt-v2`. Version 1 remains
parseable for diagnostics but is permanently non-promoting because it bound
paths without proving genuine rebuild authority. Closed v2 verdicts add
`missing-genuine-rebuild-evidence` and `target-authority-violation` to the
existing mismatch, missing-evidence, reused-store, impure, unsupported, and
malformed classes.

A v2 receipt binds the proof unit and a canonical content descriptor: selected
target identities; source closure; recipe; executable; ordered arguments; tool
and provider identities; sandbox/effect/normalization policy digests; and fresh
run-root identities. Its authority plan proves that target bytes, aliases,
symlinks, bundle-wide inputs, prior outputs, and ordinary outputs were excluded.
Every run cites the same descriptor/plan BLAKE3 and records exactly the approved
read identities with no authority violations. Strict hermetic mode, two distinct
clean stores/output roots, the ambient-host perturbation matrix, typed effects,
normalization controls, supported `mantle-proof-sandbox-v1:<blake3>` profiles,
and matching per-output BLAKE3 sets remain mandatory. Release verification,
standalone receipt checking, summaries, and Nix-witness admission all use this
same fail-closed genuine-rebuild rule.

The bounded claim is only: this named artifact rebuilt twice from these exact
content identities and policies under these isolated roots and matched. It does
not claim compiler/verifier soundness, full-bootstrap reproducibility, or global
Nix-like determinism for all Mantle builds. See
[ADR 0023](adr/0023-require-content-bound-release-rebuild-authority.md).

Provider fixed-point proof evidence is release-adjacent unless it also binds to
one packaged release binary. When `mantle release create
--provider-fixed-point-proof <dir>` is used, Mantle now requires the provider
proof's fixed-point stage binary BLAKE3 digest to match a bundled `--binary`
artifact before writing manifest evidence. `mantle release verify
--require-provider-fixed-point-proof` applies the same rule to bundled or
external provider proof evidence and reports the matched release artifact
relative path and digest. This proves only that the provider fixed-point proof
matches that packaged artifact; it does not by itself prove deterministic release
eligibility, full bootstrap reproducibility, compiler correctness, deploy
success, or full Cargo compatibility.

Later verification is bundle-local:

```bash
mantle release verify target/release-evidence/<release-id>
```

`mantle release verify` checks that every required bundled artifact exists,
that the top-level manifest stays canonical, that bundled digests still match,
and that the nested proof bundle is a full `crunch-self-hosting-proof-v2`
artifact rather than prerequisite-only `--check` output. It reports
`reproducibility: absent`, `matched`, or `mismatched` separately from bundle
integrity. Use `--require-reproducible` when callers need a verified
byte-for-byte report:

```bash
mantle release verify target/release-evidence/<release-id> --require-reproducible
```

Human verification prints `release evidence verified` only as the terminal
verdict after every selected policy check passes. Policy rejection instead
prints `release evidence rejected` with ordered diagnostics and exits nonzero.
`mantle --json release verify` emits the versioned
`mantle-release-verify-v2` shape for both outcomes, including top-level `valid`,
`disposition`, ordered `checks`, and ordered `diagnostics`. Rejected policy
results remain one JSON value on stdout and return a nonzero status. Consumers
migrating from v1 must require both a zero exit status and `valid: true`; a
parseable payload alone is not acceptance. Input or I/O failures that prevent a
final policy decision remain ordinary command errors rather than v2 decision
payloads.

The bit-for-bit reproducible release label is reserved for bundles whose
canonical reproducibility report verifies and whose artifact set matches the
published release artifact set. Ordinary bundle-local integrity is still only
packaged integrity and proof-context evidence. It lets another operator inspect
exact artifacts and verify they are internally consistent. It does not, by
itself, prove a full-source bootstrap root, independent rebuild agreement, or
global reproducibility for all Mantle builds.

Global reproducibility is a separate admitted-universe claim. Before using that
wording, evaluate an explicit universe and policy plus per-surface evidence:

```bash
mantle release global-reproducibility \
  --universe global-universe.json \
  --policy global-policy.json \
  --evidence surface-evidence.json \
  --report-path target/global-reproducibility/report.json
```

The command emits a canonical `mantle-global-reproducibility-report-v1` report,
prints its BLAKE3 digest, and exits non-zero when any included surface is
missing required receipts, strict hermeticity, output digests, source/toolchain
provenance, or policy-satisfied independent replay evidence. A scoped release
report, provider fixed-point proof, deterministic-release receipt, or external
witness agreement remains valid evidence for its own tier, but none of them is
promoted to global reproducibility unless this report is `eligible` for the
bound universe and policy.

### Release attestations and witness verification

After a release-evidence bundle verifies, the next layer is a signed release
attestation plus optional external witness attestations. The checked-in
cross-machine flow is publisher -> witness -> publisher:

```bash
# Publisher: sign the verified release bundle into a verification directory
mantle release attest target/release-evidence/<release-id>

# Publisher: export the verifier-ready public key string used by release-verify
RELEASE_TRUSTED_KEY=$(mantle attest key-show --signing-key /path/to/release.key)
RELEASE_SIGNER_NAME="${RELEASE_TRUSTED_KEY%%:*}"

# Publisher: scaffold verifier-local policy and empty revocations files
mantle attest policy-init target/release-verification/<release-id> \
  --profile single-witness \
  --trusted-release-signer "$RELEASE_SIGNER_NAME" \
  --trusted-witness-identity <witness-identity>

# Publisher: export a public-only witness-request directory for the witness
mantle release witness-export target/release-evidence/<release-id> \
  --verification-dir target/release-verification/<release-id> \
  --request-dir target/release-witness-requests/<release-id>

# Witness: replay the checked-in rebuild workflow from the exported request
./scripts/rebuild-witness-request.sh \
  target/release-witness-requests/<release-id> \
  --identity <witness-identity> \
  --system x86_64-linux \
  --toolchain rust-1.91.1 \
  --host-class nixos-25.05 \
  --signing-key /path/to/witness.key
WITNESS_TRUSTED_KEY=$(mantle attest key-show --signing-key /path/to/witness.key)

# Publisher: import the returned witness sidecars and verify the final status
mantle attest witness-import target/release-verification/<release-id> \
  target/release-witness-requests/<release-id>.work/release-verification/<release-id>
mantle attest release-verify target/release-verification/<release-id> \
  --trusted-public-key "$RELEASE_TRUSTED_KEY" \
  --trusted-public-key "$WITNESS_TRUSTED_KEY"
```

`mantle release attest` writes `release-attestation.json` plus a detached
`.sig` file. `mantle attest key-show` reads an existing signing keypair and
prints the exact `name:base64` token accepted by `--trusted-public-key`; omit
`--signing-key` to inspect the default configured signing key instead. The
release-signer name passed to `--trusted-release-signer` is the substring
before `:` in that token, so shell workflows can derive it with
`RELEASE_SIGNER_NAME="${RELEASE_TRUSTED_KEY%%:*}"` when keys were
auto-generated in a per-operator `CRUNCH_CONFIG_DIR`. `mantle attest
policy-init` writes verifier-local `policy.json` and `revocations.json` for
either `self-proof-only` or `single-witness` publication without hand-authoring
JSON. `mantle release witness-export` copies only public verification material:
a verified release-evidence bundle, the signed release attestation, and request
metadata. It does not copy signing keys or verifier-local policy files. The
checked-in witness-side wrapper `./scripts/rebuild-witness-request.sh` is the
operator-facing path for replaying that request. It preflights `bwrap`, resolves
an absolute static `SNIX_BUILD_SANDBOX_SHELL`, derives a controlled scratch
root (`<request-dir>.work/` by default or `$CRUNCH_WITNESS_SCRATCH_DIR`), and
then calls the machine-readable core command `mantle release witness-rebuild
<request-dir> ...`. The core replay preserves the published proof mode, so a
`non-nix-host` release proof is rebuilt with the helper's `--non-nix-host` path
scrub instead of silently weakening to the default fixed-point mode. The
request directory stays immutable after validation; the rebuild output lands
under the scratch verification directory together with
`witnesses/<identity>.json`, the matching `.sig`, and
`witness-rebuild-audit/meta.json` describing the replayed workflow, scratch
paths, timestamps, and rebuilt output digests. `mantle attest witness-import`
fails closed on missing signatures, release-digest mismatches, and conflicting
existing witness identities before touching the publisher verification
directory. `mantle attest release-verify` separates technical validity from
policy sufficiency, so a release can stay technically valid even when the
witness set is policy-insufficient. In JSON mode it also reports independent
agreement separately through `independent_agreement_status`,
`independent_agreement_class`, `independent_agreement_report_digest`, counted / skipped /
failed witness counts, and per-witness classification reasons. These fields
are derived from accepted witness files plus verifier-local policy,
revocations, and trusted keys; they are not publisher-authored release claims.
Unknown-key and invalid-signature witnesses are skipped, revoked witnesses are
skipped, duplicate independence domains are skipped, and signature-valid
witnesses with wrong release references or rebuilt digests are failed evidence.
A verifier-local `agreement-report.json` must be canonical and match the
derived report; duplicate report filenames are rejected. Release-evidence
bundles may also carry the optional report artifact at
`independent-agreement/agreement-report.json`, where the manifest digest check
covers it like any other bundled file. A successful single-witness workflow
therefore proves external witness agreement under the configured policy; a
satisfied independent-agreement status gives the stronger
`independent-rebuild-agreement` class for that policy-scoped witness set. It
still does not prove a full-source bootstrap root or globally reproducible
release artifacts.

### Bootstrap roadmap

The next trust-reduction steps are straightforward:

1. Replace the reduced musl.cc-derived provider with a smaller and more
   auditable bootstrap root.
2. Reduce or encapsulate the host-tool prerequisites still needed to stage the
   first self-build.
3. Add independent rebuild evidence for release outputs instead of stopping at
   the stage0 -> stage1 -> stage2 self-hosting proof.
4. Publish the trust inventory and proof outputs as first-class release
   artifacts instead of leaving them only in code and README text.

### Background reading

Related bootstrap work worth keeping handy:

- [Bootstrappable Builds](https://www.bootstrappable.org/) — broader reference material on reducing bootstrap seeds and building from source all the way down.
- [Guix blog: The full-source bootstrap: building from source all the way down](https://guix.gnu.org/en/blog/2023/the-full-source-bootstrap-building-from-source-all-the-way-down/) — a concrete walkthrough of Guix's bootstrap story and trust reduction work.
- [Stagex](https://codeberg.org/stagex/stagex) — a stage-by-stage bootstrap project with packaging and build recipes relevant to self-hosting discussions.

**Troubleshooting the proof:**

| Symptom | Cause | Fix |
|---|---|---|
| `error: nightly toolchain 'nightly' is not installed under ~/.rustup/toolchains` | nightly toolchain missing | install the repo's nightly toolchain under `~/.rustup/toolchains` |
| `error: pkg-config cannot find openssl` | OpenSSL `.pc` files are outside the default search path | export `CRUNCH_PROOF_OPENSSL_PKGCONFIG=/path/to/openssl/lib/pkgconfig` and rerun |
| `error: required tool 'mold' not found` | linker tools missing from the shell | enter the repo dev shell or add the tool directory to `PATH` before running the helper |
| `error: required tool 'bwrap' not found` | bubblewrap not installed | `nix-shell -p bubblewrap` or install bwrap from your distro |
| `error: only <n> MiB free in proof scratch root <dir>; need at least 4096 MiB for the proof. Set CRUNCH_PROOF_SCRATCH_DIR to a larger filesystem` | selected proof scratch filesystem is too small | free space under `target/self-hosting-proof/work/`, or point `CRUNCH_PROOF_SCRATCH_DIR` at a larger writable filesystem |
| `error: proof scratch root from CRUNCH_PROOF_SCRATCH_DIR is not usable:` | explicit scratch override points at a blocked or unwritable location | point `CRUNCH_PROOF_SCRATCH_DIR` at a writable directory |
| `error: default proof scratch root is not usable:` | repo-local `target/self-hosting-proof/work/` cannot be created or written | fix repo `target/` permissions or set `CRUNCH_PROOF_SCRATCH_DIR` to a writable directory |
| Stage0 fails with permission errors writing to store | unwritable output directory | The proof uses the selected scratch root for temp files, but build outputs still need writable store and state directories |
| Stage2 reports `bwrap-source=host-fallback:` | stage0 did not produce mantle-built bwrap | Clear the proof store and rerun; the bootstrap chain may have failed silently |
| Stage2 reports `busybox-path=none` | no mantle-built busybox in the proof store | Same as above — the bootstrap chain did not complete |
| Stale pathinfo.redb causes false cache hits | prior run left state in `~/.local/state/mantle/` | The proof uses per-stage state dirs to avoid this; if running manually, pass `--state-dir` to a fresh directory |
| `No space left on device` | selected proof scratch filesystem or later proof outputs still ran out of space | free about 4 GiB under `target/self-hosting-proof/work/` or set `CRUNCH_PROOF_SCRATCH_DIR` to a larger filesystem |

## Project Management

mantle has built-in dependency management for project inputs — git repos,
tarballs, and files declared in a Nickel manifest (`mantle-project.ncl`).

```bash
# Create a new project (manifest, lockfile, .mantle/ directory)
mantle init

# Validate manifest, lockfile, and generated inputs
mantle check

# Show resolved input state from the lockfile
mantle show

# Refresh all inputs (or specific ones)
mantle refresh
mantle refresh nixpkgs my-lib

# Check which inputs would change without modifying anything
mantle list-stale

# Migrate project files to the current schema version
mantle upgrade
```

The lockfile (`mantle.lock`) stores resolved revisions and NAR hashes.
`mantle refresh` resolves upstream references (git ls-remote, content
hashing) and updates both `mantle.lock` and `.mantle/inputs.ncl`
(generated Nickel bindings).

### Offline build runbook

For disconnected or no-substitute builds, first prepare a source bundle on a
connected/source-rich host, then import and pin it on the offline host before
building:

```bash
mantle source bundle export --build-root ./package.ncl --import-path lib --to source-bundle.json
mantle source bundle verify --from source-bundle.json
mantle --state-dir ./offline-state source bundle import --from source-bundle.json --pin
mantle --state-dir ./offline-state source bundle verify --from source-bundle.json --imported
mantle --state-dir ./offline-state source bundle preflight --build-root ./package.ncl --import-path lib
mantle --state-dir ./offline-state build --offline-source-preflight --no-substitute ./package.ncl
mantle --json --state-dir ./offline-state build --offline-source-preflight --no-substitute ./package.ncl > build-report.json
```

Inspect `ready_class`, `source_state_blake3`, and `next_actions[]` from source
preflight, plus `network_policy_reports[]`, `cargo_build_evidence[]`, and
`cargo_build_evidence_diagnostics[]` in `build-report.json`. The source bundle
evidence proves declared source/input availability and identity only;
source-bundle route execution is future work. Do not treat source readiness,
route eligibility, source import, or offline Cargo evidence as build success,
output trust, Cargo-free execution, full Cargo compatibility, compiler
correctness, release reproducibility, or bootstrap correctness without separate
evidence.

### Offline Cargo package builds

The near-term Rust project-build lane is sandboxed offline Cargo, not native
`rust-plan`. In a canonical `mantle-project.ncl` project root (legacy
`crunch.ncl` remains accepted), declare a package with
`mantle.offlineCargoPackage { ... }`, pass explicit source, bootstrap Rust,
seed toolchain, and musl inputs, then build it with `mantle build .#name` or run
it with `mantle run .#name`.

`mantle import cargo --plan` reads supported Cargo workspace facts and prints a
reviewable scaffold plan without writing files. It accepts registry or git
dependencies only when pre-existing vendored source material is declared through
`.cargo/config.toml` / `.cargo/config`, bound to `Cargo.lock`, and verified
against Cargo `.cargo-checksum.json` metadata. `mantle import cargo --apply`
only writes the bounded files named by that plan (`mantle-project.ncl` and
`.mantle/inputs.ncl` today) when no blockers remain; accepted vendor material is
passed explicitly as `vendor_src` / `vendor_name` to `mantle.offlineCargoPackage`.
Missing vendored packages, stale checksums, missing lockfiles, ambiguous
packages/binaries, malformed names, unsupported source replacement, ambient-only
Cargo caches, and conflicting existing files block the apply path instead of
generating partial project files. Import never runs `cargo vendor`, fetches
registry/git material, or claims network vendoring, Cargo-free execution, or full
Cargo compatibility.

The representative Rust compatibility rail lives in
`examples/rust_compatibility_rail.rs`, `examples/rust_compatibility_surface_matrix.ncl`,
and `tests/rust_compatibility_rail.rs`. The surface matrix is the source of
truth for supported lanes, blocked surfaces, stable blocker classes, and
non-claims. It covers a generated workspace with a binary, local library,
feature activation, target-specific dependency declaration, workspace metadata
inheritance, a sibling binary package, vendored registry material, proc macro,
and build script metadata. The offline lane reports
`cargo-inside-mantle-sandbox`; the native `rust-plan --no-cargo-oracle` lane
reports `cargo-free-bounded-topology` for the path-workspace subset or
`blocked-unsupported-surface` for unsupported surfaces such as vendored git or
native-link metadata. Passing that rail is evidence for that matrix entry and
lane only; it is not proof of full Cargo compatibility.

The helper writes `share/mantle/offline-cargo-build.json` into the output and
JSON build reports surface that sidecar under `cargo_build_evidence[]` when the
output is materialized locally. Current v2 evidence is digest-bound: reports
record the schema/version, `cargo-inside-mantle-sandbox` claim class,
Cargo.lock BLAKE3 digest, package-source and optional vendor BLAKE3 identities,
selected target/profile, Cargo command shape, offline network-policy result,
store-path-scoped toolchain identities, output path, and non-claims. Legacy v1
sidecars remain visible as legacy path evidence, while malformed, stale,
schema-mismatched, or wrong-claim sidecars appear in
`cargo_build_evidence_diagnostics[]` instead of being silently promoted. This
evidence explicitly does not claim Cargo-free execution, full Cargo
compatibility, compiler correctness, release reproducibility, bootstrap
correctness, or module-layer semantics.

## Build-tool boundary

Mantle is a build tool, not a NixOS-style module layer. Frontends such as Onix
own their inventory, module ABI, role/tag expansion, settings validation,
upstream/provider topology, and artifact/package policy. They should lower that
module-layer state into concrete derivations, build plans, source inputs, or
opaque evaluated data before invoking Mantle.

Mantle's stable handoff surface is build shaped: `mantle eval` for derivation
JSON, `mantle build` for realization, `mantle build --plan` for per-root action
planning, store commands for local state, and build reports for results. ADR
[`0010`](adr/0010-keep-mantle-build-tool-boundary.md) records this as an
architecture boundary, and the CLI boundary tests guard against reintroducing an
in-tree module layer.

## CLI

### Commands

```
# Build, diagnostics, bootstrap
mantle doctor                    No-mutate preflight for build or self-build hosts
mantle build [file.ncl|.#name]   Evaluate and build
mantle build --plan <target>     Preview cached/substitute/build/preflight-error
mantle build --fix <file>        Build and rewrite FOD mismatches in source
mantle source bundle <action>    Plan, export, import, verify, or preflight source/input bundles
mantle eval <file.ncl>           Evaluate and print JSON
mantle bootstrap [-o seed.ncl]   Generate a seed file (`--fetch` for Nix-free)
mantle self-build                Rebuild mantle from source
mantle self-build --cargo-free --out /tmp/mantle-out
                                 Build mantle through Cargo-free Rust topology
mantle rust-plan [--execute-topology]
                                 Explicit bounded Rust planner verification lane
mantle nix-free-demo validate <summary.json>
                                 Validate bounded demo-bundle claimability
mantle nix-free-demo readme <summary.json>
                                 Render the derived demo-bundle README
mantle nix-free-demo generate --out <dir> --proof-status <status> ...
                                 Assemble a deterministic demo bundle from explicit evidence inputs
mantle foreign-import produce-nix --derivation-json <closure.json> --root-derivation <path.drv> --out-dir <dir>
                                 Lower concrete Nixpkgs derivation facts into graph/index artifacts without making nixpkgs a consumption ABI
mantle foreign-import validate --graph <graph.json> --package-index <index.json> --policy <policy.json>
                                 Validate lowered foreign import artifacts without live Guix/Nix frontends
mantle foreign-import plan --graph <graph.json> --package-index <index.json> --policy <policy.json> --package hello
                                 Emit a receipt-bound adapter plan with explicit non-claims and no output-trust claim
mantle artifact oci-export --projection <projection.json> --spec-material <spec> --source-admissions <admissions.json> --out <layout>
                                 Project admitted frontend objects into an atomic local OCI image layout
mantle artifact oci-import --layout <layout> --report-out <report.json>
                                 Verify descriptors and admit exact OCI blobs; external layouts remain compatibility-only
mantle wasm-component build <request.ncl> --out <evidence-dir>
                                 Build, validate, Octet-check, and materialize an exact component evidence bundle

# Store, logs, attestations, release evidence
mantle store <subcommand>        List, inspect, verify, sign, pin, push, pull, or GC store state
mantle log [query]               Show a stored build log
mantle attest <subcommand>       Show, verify, diff, or synthesize attestations
mantle release <subcommand>      Create, verify, reproduce, attest, witness, or globally gate release evidence

# Project workflows
mantle init                      Initialize mantle-project.ncl, mantle.lock, .mantle/
mantle check                     Validate manifest, lockfile, and generated inputs
mantle show                      Show resolved input state
mantle refresh [names...]        Refresh selected or all project inputs
mantle list-stale                Report which inputs would change on refresh
mantle upgrade                   Migrate project files to the current schema
mantle shell [name]              Enter or execute inside a dev shell
mantle develop [name]            Deprecated alias for `mantle shell`
mantle run [target] [-- args...] Build and execute a package binary (`.#name`, bare project package, or .ncl file)

```

Nix-free fixed-point demo wording is claimable only from a validated demo-profile
proof bundle. Run `mantle --json nix-free-demo validate <summary.json>` for a
stable `mantle-nix-free-demo-cli-v1` decision, or `mantle nix-free-demo readme
<summary.json>` to render the derived operator README. The validator must see
matching stage digests, source-root and toolchain-policy evidence,
Cargo/Nix/rustup/ambient-wrapper guard denials, replay hints, and explicit
non-claims; otherwise summaries should describe only the narrower blocker or
fixed-point artifact that was actually proven.

### Global flags

```
--store <path>              Physical output directory (default: /nix/store)
--store-prefix <prefix>     Logical store prefix (default: /mantle/store)
--nix-compat                Shorthand for --store-prefix=/nix/store
--state-dir <path>          State directory for databases and blobs
                            (default: $CRUNCH_STATE_DIR or ~/.local/state/mantle)
--json                      Emit errors as JSON for tooling integration
-v, --verbose               Debug logging
--log-level <lvl>           trace|debug|info|warn|error
```

### Build flags

```
-j, --jobs <N>                   Max concurrent builds (default: CPU count, max 16)
--fix                            Auto-fix FOD hash mismatches in .ncl source
--plan                           Preview per-root action without building
--offline-source-preflight       Require imported and pinned source state before build planning/execution
-I, --import-path <path>         Additional Nickel import paths
--substituters <url>             Binary cache URLs (default: https://cache.nixos.org)
--no-substitute                  Disable binary cache substitution
--signing-key <path>             Path to ed25519 signing keypair file
--trusted-public-keys <keys>     Trusted public keys for signature verification
--trust-unsigned                 Accept unsigned PathInfo on cache hits
--strict-hermetic                Reject degraded hermetic behavior on build-entry commands
```

## Requirements

**Build time** (compiling mantle itself):
- Linux
- Rust nightly
- clang + mold (linker)
- pkg-config + openssl-dev

**Run time** (building derivations):
- Linux (bwrap sandbox requires user namespaces)
- bwrap (bubblewrap) in PATH

**Not required**:
- protoc — gRPC/protobuf replaced with postcard serialization
- Nix — closure resolution uses mantle's own PathInfo, not `nix-store`.
  A Nix installation is only needed for `mantle bootstrap` (without
  `--fetch`)
- Writable `/nix/store` — the castore is the primary store. Cache
  validation uses PathInfo + castore content probes, not filesystem
  existence. If the output directory is not writable, builds succeed
  with outputs in castore only

## Known Limitations

- **Manual GC only**: `mantle store gc` is implemented, but background or
  low-space-triggered collection is still future work.
- **Concurrent builds**: independent derivations run in parallel (up to
  `-j N`, default: CPU count, max 16). The lazy goal scheduler
  dispatches builds as their dependencies complete; sandbox execution
  is concurrent via `tokio::JoinSet`. Preparation and output processing
  are sequential.
- **Multi-output**: outputs work end-to-end. Output *selection* is
  supported via `mantle.select dep "dev"` to mount a single output of
  a multi-output dependency in the sandbox (like Nix's `pkg.dev`).

## License

Repository-owned Mantle source is `AGPL-3.0-or-later`; see [LICENSE](LICENSE). Vendored `fuse-backend-rs` remains `Apache-2.0 AND BSD-3-Clause` under the license texts and notices in its own directory. Other dependencies and generated material containing upstream code retain their original terms. License metadata does not expand Mantle's bounded build, cache, or evidence claims.

## References

- [rust-lang/rustc-dev-guide](https://github.com/rust-lang/rustc-dev-guide/tree/main) — important Rust compiler architecture reference; keep the [HIR](https://github.com/rust-lang/rustc-dev-guide/tree/main/src/hir), [MIR](https://github.com/rust-lang/rustc-dev-guide/tree/main/src/mir), and [backend](https://github.com/rust-lang/rustc-dev-guide/tree/main/src/backend) sections handy for Mantle's Rust planning, compiler-interface, and evidence-boundary work.
- [fosslinux/live-bootstrap](https://github.com/fosslinux/live-bootstrap) — reference stage order and source provenance for the hex0 → mes → tinycc → GCC bootstrap ladder.
- [stagex/stagex](https://codeberg.org/stagex/stagex) — mrustc-to-current-Rust source bootstrap route used as the reference shape for `bootstrap/rust-source-plan.ncl`.
- [FractalFir/crustc](https://github.com/FractalFir/crustc) — generated-C Rust compiler snapshot used as Rust-to-C bootstrap prior art; Mantle should treat it as non-claim research until the generator, license, LLVM wrapper sources, target-specific C generation, and sysroot route are auditable.
- [adeci/drv-thru](https://github.com/adeci/drv-thru) — P2P Nix build tickets and signed-output import model used as remote-builder prior art; Mantle adaptations should replace Nix-specific plumbing with Mantle CAS, PathInfo, attestation, and substitution semantics.
- [Mic92/tribuchet](https://github.com/Mic92/tribuchet) — remote-build hub/worker scheduling prior art for worker-dialed registration, capability queues, request dedupe, missing-input negotiation, signed output return, bounded log replay, and restart/reload survival; Mantle adaptations should keep those architecture ideas while replacing Nix external-builders, nix-daemon imports, scratch-path assumptions, and `/nix/store` pinning with Mantle-native CAS, PathInfo, attestation, and store-prefix contracts.
- [Nixtamal](https://nixtamal.toast.al/) — Nix input pinning tool used as project-input workflow prior art for custom freshness checks, mirrors, declarative patches, per-input hash algorithms, non-Git VCS sources, and future lockfile import/trust ideas.
- [OnixResearch/nickel-export](https://github.com/OnixResearch/nickel-export) — evaluator-neutral Nickel export admission, exact-byte identity, freshness, and compatibility projections consumed at immutable revision `257fafc1c746f1faf156207043a4c826bfb16d49`; Mantle retains embedded evaluation, filesystem, destination, build, and release authority.
- [nickel-lang/rules_nickel](https://github.com/nickel-lang/rules_nickel) — declared Nickel export action and evaluator toolchain prior art; Mantle adaptations should keep source closures, safe import paths, export formats, and evaluator identity while avoiding Bazel-specific repository/toolchain machinery in core.
- [nickel-lang/organist](https://github.com/nickel-lang/organist) — Nickel-managed project workflow prior art for typed generated files and named shell profiles; Mantle adaptations should keep explicit plan/apply mutation boundaries and avoid adopting service lifecycle management into core.
- [nickel-lang/json-schema-to-nickel](https://github.com/nickel-lang/json-schema-to-nickel) — JSON Schema to Nickel contract generation prior art for machine-report schema validation; Mantle adaptations should use generated contracts as checked development/release rails with positive and negative fixtures.
- [oxidecomputer/tufaceous](https://github.com/oxidecomputer/tufaceous) and [awslabs/tough](https://github.com/awslabs/tough) — TUF-style release repository and metadata prior art for signed targets, artifact tags, compatibility checks, and trust-root handling.
- [oxidecomputer/buildomat](https://github.com/oxidecomputer/buildomat) — ephemeral build worker, captured log, artifact publishing, trusted default-branch policy, and replayable job-event prior art for Mantle remote-build evidence.
- [oxidecomputer/cancel-safe-futures](https://github.com/oxidecomputer/cancel-safe-futures) — cancellation-aware async adapter prior art for worker cleanup, output-upload integrity, and final receipt preservation.
- [lovesegfault/rio-build](https://github.com/lovesegfault/rio-build) — BSD-3-Clause remote-build prior art for pure decision kernels, fenced pull assignments, resumable missing-chunk transfer, critical-path/resource-aware scheduling, immutable attempt logs, and telemetry; Mantle adaptations reuse its own lazy scheduler, castore, trust, evidence, and provider-neutral configuration instead of adopting Rio's full service stack.
- [onixcomputer/onix-modules](https://github.com/onixcomputer/onix-modules) — accepted `onix-kernel-bundle-v1` contract and golden frontend projection/import fixtures consumed by Mantle's bounded OCI adapter; the snapshots are pinned to OnixOS commit `8a99461`, and Onix semantics remain external to Mantle.
- [multikernel/kernelscript](https://github.com/multikernel/kernelscript) — KernelScript `v0.1.2` source and generated-project shape reference for Mantle's disabled-by-default bounded experiment; Mantle pins and builds the official source archive against a locked nixpkgs closure and routes exact generated-shape/receipt semantics through `crunch-kernelscript-core`, while accepted Onix target authority and the private/kfunc module route remain explicitly blocked.
- [nasa/spacewasm](https://github.com/nasa/spacewasm/tree/e24cf09355a90497148eb5029fdb8e3400bd63e3) — exact diagnostic reference cohort selected by Octet's reviewed `spacewasm-mvp` profile; Mantle retains the source, locked dependency closure, toolchain, host/wasm libraries, bounded runner, fixtures, corpus artifacts, legal material, results, replay evidence, and explicit non-claims without promoting them to correctness, memory-safety, conformance, flight, sandbox, runtime-admission, production, or release claims.
