# Operator workflows

This page complements the top-level README.

Current operator-facing command families:

- `mantle doctor`
- `mantle build` / `mantle build --plan`
- `mantle shell` / `mantle develop`
- `mantle run`
- `mantle attest`
- `mantle release`

Use `mantle --help` for the full command list. Use this page for the common
operator loops.

## Validation tiers

Use the checked-in toolchain from `rust-toolchain.toml`.

### Ordinary first-party gate

Run the checked-in wrapper from the repo root:

```bash
./scripts/check-first-party-quality.sh
```

It runs three ordinary edit-time checks in order:

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
cargo test --workspace --lib --tests \
  --exclude fuse-backend-rs \
  --exclude nix-compat \
  --exclude nix-compat-derive \
  --exclude snix-build \
  --exclude snix-castore \
  --exclude snix-store \
  --exclude snix-tracing \
  -- --test-threads 1
```

Notes:

- Vendored workspace-member test binaries stay on their focused rails because
  some require host capabilities such as a usable FUSE mount. Their libraries
  still compile through first-party consumers.
- The wrapper serializes libtest cases so process-global environment, lock,
  and resource-pressure fixtures cannot interfere across otherwise unrelated
  tests. Tests that own concurrency still exercise it internally.

- The root `-p mantle` rustfmt leg covers the root package's `src/`,
  `examples/`, and `tests/`, including `tests/benchmark_harness.rs`.
- `./scripts/check-first-party-clippy.sh` excludes vendored workspace members
  `fuse-backend-rs`, `nix-compat`, `nix-compat-derive`, `snix-build`,
  `snix-castore`, `snix-store`, and `snix-tracing` so first-party warnings
  fail cleanly.
- The wrappers assume the documented build environment. If `clang`, `mold`,
  `pkg-config`, or the OpenSSL pkg-config path are missing, fix the shell env
  first instead of treating that as a code failure.

### Tigerstyle lane

Run the repo-pinned Tiger Style consumer check when you want the structural lint
pass:

```bash
./scripts/check-first-party-tigerstyle.sh
```

That wrapper delegates to this flake entry point:

```bash
nix run .#tigerstyle -- check
```

Notes:

- default package scope comes from `[workspace.metadata.tigerstyle]` in
  `Cargo.toml`
- workspace-specific lint rollout config lives in `dylint.toml`
- vendored workspace members stay out of the default Tiger Style scope

### Heavyweight rails

Keep these checks separate from the ordinary gate:

```bash
cargo test -p crunch-pipeline --test integration_build \
  pipeline_determinism_probe_ -- --ignored --nocapture
./scripts/prove-self-hosting.sh --check
./scripts/check-release-determinism-quality.sh
./scripts/check-release-nix-witness-quality.sh
nix build .#checks.x86_64-linux.release-determinism-quality --no-link -L
nix build .#checks.x86_64-linux.release-nix-witness-quality --no-link -L
```

- The release determinism quality rail runs the generated deterministic proof
  smoke and validates its receipt/log BLAKE3 in one checked-in entry point.
- The `release-determinism-quality` flake check is the Nix/CI-callable heavy
  gate for the same underlying generated proof regression.
- The `release-nix-witness-quality` package/check is the Nix/CI-callable heavy
  gate for the bounded `mantle release nix-witness` CLI path; keep it opt-in
  rather than part of ordinary developer builds.
- The determinism probe is an ignored integration rail, not part of every edit.
- `./scripts/prove-self-hosting.sh --check` is only self-hosting preflight.
- The full ignored proof remains a separate heavier run:

```bash
./scripts/prove-self-hosting.sh
```

### Vendored maintenance lane

Workspace-wide vendored upkeep stays outside the first-party gate. Do not treat
vendored clippy debt as an ordinary edit blocker for tracked mantle code.

If you bypass the checked-in wrappers and run direct compile-heavy `cargo`
commands, keep `TMPDIR` and `CARGO_TARGET_DIR` on disk-backed scratch and use
the PATH / `PKG_CONFIG_PATH` / `SNIX_BUILD_SANDBOX_SHELL` prerequisites called
out in the self-hosting workflow.

## Plan before building

Start with the no-mutate preflight:

```bash
# Default build host checks
mantle doctor

# Extra nightly-toolchain checks for self-build hosts
mantle doctor --profile self-build
```

Then preview what mantle would do per root:

```bash
mantle build --plan .#hello
mantle --json build --plan .#hello
```

Plan output uses four action labels:

- `cached` - all outputs are already accepted locally
- `substitute` - cache metadata says a remote hit is available
- `build` - no accepted hit exists, but local build preflight passed
- `preflight-error` - mantle cannot take the local build path yet

When you want degraded hermetic behavior to fail instead of warn, opt into
strict mode on the build-entry commands that expose it:

```bash
mantle build --strict-hermetic .#hello
mantle self-build --strict-hermetic --store /tmp/mantle-store -j 4 --no-substitute
```

Structured build reports surface the same operator facts in stable fields:

- `hermeticity_mode`
- `hermeticity_audit_events[]`
- `build_environment_reports[]` — normalized strict child-environment BLAKE3 digest plus redacted denied-variable summaries
- `network_policy_reports[]` — ordinary builds are offline by default, fixed-output fetchers record their declared source boundary, and denied compatibility capabilities are blocked before strong evidence is emitted
- `effect_policy_version` (`mantle-build-effects-v1` for deterministic proof receipts)
- `declared_effects[]` and `observed_effects[]`; deterministic release verification
  fails closed when observed effects are missing or exceed the declared set
- `failed[]`
- `outcomes[].outputs[].artifact_attestation`

That last block gives both the logical store path and the persisted sidecar
path for the artifact attestation.

## Offline build runbook

Use this runbook when the target host must build from already collected source
material without substitutes. It is command evidence, not a new proof class: the
source bundle evidence proves declared source/input availability and identity
only. When `--offline-source-preflight` is ready, Mantle can materialize matching
fixed-output fetcher inputs from imported and pinned source state before the
ordinary local build consumes them.

On a connected or source-rich host, export the source records required by the
selected root:

```bash
mantle source bundle export --build-root ./package.ncl --import-path lib --to source-bundle.json
mantle source bundle verify --from source-bundle.json
```

Copy `source-bundle.json` to the offline host, then import and pin it into the
selected state directory:

```bash
mantle --state-dir ./offline-state source bundle import --from source-bundle.json --pin
mantle --state-dir ./offline-state source bundle verify --from source-bundle.json --imported
mantle --state-dir ./offline-state source bundle preflight --build-root ./package.ncl --import-path lib
```

Run the build with source preflight and substitute lookup disabled:

```bash
mantle --state-dir ./offline-state build --offline-source-preflight --no-substitute ./package.ncl
mantle --json --state-dir ./offline-state build --offline-source-preflight --no-substitute ./package.ncl > build-report.json
```

Inspect these fields before making a status claim:

- `ready_class`, `source_state_blake3`, and `next_actions[]` from source-bundle
  preflight. Missing, stale, unsupported, untrusted, network-required, or
  unpinned state names a bounded next action such as re-exporting, importing
  with `--pin`, inspecting adapter metadata, or choosing an explicit non-offline
  workflow.
- `network_policy_reports[]` from build JSON. Offline builds should show only
  declared fixed-output source boundaries or denied compatibility capabilities.
- `cargo_build_evidence[]` for accepted `mantle.offlineCargoPackage` outputs and
  `cargo_build_evidence_diagnostics[]` for malformed sidecars that must be
  inspected or rebuilt before an offline Cargo evidence claim is made.
- Route-plan output when `mantle build --plan` is used. A selected or eligible
  `source-bundle` route means only that source/input realization can use the
  named source-state digest. It is separate from `cached` local output reuse,
  remote `substitute` output reuse, and final output trust.

Do not report source readiness, source import, route eligibility, source-bundle
input realization, or offline Cargo evidence as build success, output trust,
Cargo-free execution, full Cargo compatibility, compiler correctness, release
reproducibility, or bootstrap correctness. Fixed-output verification still runs
on materialized source-bundle inputs; final outputs need separate build/cache and
attestation evidence.

## Offline bootstrap source-bundle profile

Use `mantle source bundle bootstrap-profile` to enumerate and verify every
source/input record a bootstrap or self-build workflow needs before it starts.
This is input-availability evidence, not bootstrap proof: the profile proves
provider archive identity, provider metadata, bootstrap source archives, Mantle
source tree identity, vendored Cargo inputs, and proof inputs are locally
available and identity-matched. It does not prove provider trust removal,
compiler correctness, or self-build success.

On a connected host, build the profile and export it:

```bash
mantle source bundle bootstrap-profile \
  --mode self-build-proof \
  --provider-archive ./provider-archive \
  --provider-manifest ./provider.json \
  --bootstrap-source ./bootstrap-src \
  --mantle-source ./mantle-src \
  --vendor-deps ./vendor-deps \
  --toolchain-source-root ./toolchain \
  --proof-input ./proof-input \
  --to bootstrap-source-bundle.json
```

Copy `bootstrap-source-bundle.json` to the offline host, then import, pin, and
preflight it:

```bash
mantle --state-dir ./offline-state source bundle import --from bootstrap-source-bundle.json --pin
mantle --state-dir ./offline-state source bundle bootstrap-profile \
  --mode self-build-proof \
  --provider-archive ./provider-archive \
  --provider-manifest ./provider.json \
  --bootstrap-source ./bootstrap-src \
  --mantle-source ./mantle-src \
  --vendor-deps ./vendor-deps \
  --toolchain-source-root ./toolchain \
  --proof-input ./proof-input \
  --preflight
```

Run legacy fetch-mode bootstrap with offline source preflight so the provider
tarball is materialized from source state instead of fetched live:

```bash
mantle --state-dir ./offline-state bootstrap --fetch --offline-source-preflight --output seed.ncl
```

Do not report bootstrap source-bundle readiness as provider trust removal,
compiler correctness, self-build success, release reproducibility, or full
bootstrap correctness. Those claims still require the existing proof commands
and evidence gates.

## Offline Cargo project-build lane

Use `mantle build .#name` for the supported near-term Rust project workflow.
A project package can use `mantle.offlineCargoPackage { ... }` to lower a
declared Cargo package into an ordinary derivation that runs `cargo build
--locked --offline` inside Mantle's sandbox with isolated `HOME`, `CARGO_HOME`,
and `CARGO_TARGET_DIR`.

For existing Cargo workspaces, use `mantle import cargo --plan` first. The plan
prints deterministic file operations, content digests, selected package/binary
facts, source input placeholders, accepted vendored-source facts, and blockers
without mutating files. It accepts registry or git dependencies only when
pre-existing vendored source material is declared through `.cargo/config.toml` /
`.cargo/config`, bound to `Cargo.lock`, and verified against Cargo
`.cargo-checksum.json` metadata. Use `mantle import cargo --apply` only after
reviewing that plan; apply writes only the accepted Mantle-owned files and fails
before writing if conflicts, missing vendored packages, stale checksums,
unsupported source replacement, or unsupported Cargo surfaces remain.

Required inputs are explicit: package source, lockfile identity in the source,
bootstrap Rust toolchain, seed C toolchain, musl runtime, and optional vendored
registry/git source material passed as `vendor_src` / `vendor_name`. Missing
source material fails closed before a result is accepted; the helper and import
scaffold do not search ambient Cargo caches, target directories, or network
sources, and they never run `cargo vendor` or claim network vendoring,
Cargo-free execution, compiler correctness, or full Cargo compatibility.

The representative Rust compatibility rail (`examples/rust_compatibility_rail.rs`,
`examples/rust_compatibility_surface_matrix.ncl`, and
`tests/rust_compatibility_rail.rs`) is the maintained surface matrix for
practical Rust project claims. It covers a binary, local library, feature
activation, target-specific dependency declaration, workspace inheritance,
sibling binary package, vendored registry source, proc macro, and build script
metadata. Its offline Cargo result is `cargo-inside-mantle-sandbox`; its
rust-plan result is either `cargo-free-bounded-topology` for the supported
path-workspace subset or a deterministic `blocked-unsupported-surface` blocker
for blocked surfaces such as vendored git, pkg-config, rustc-link metadata, or
native C compilation. It is not proof of full Cargo compatibility.

Structured JSON reports include `cargo_build_evidence[]` when the output
contains `share/mantle/offline-cargo-build.json`. Current v2 evidence is
`cargo-inside-mantle-sandbox` plus digest-bound Cargo.lock, package-source,
optional vendor, command, target/profile, output, and offline network-policy
facts. Legacy v1 sidecars are labeled as legacy path evidence, and malformed or
stale sidecars are reported under `cargo_build_evidence_diagnostics[]`. This is
evidence that the declared Cargo action produced the inspected output under
Mantle's sandbox policy; it is not evidence of Cargo-free execution, full Cargo
compatibility, compiler correctness, release reproducibility, bootstrap
correctness, or module-layer semantics.

## WebAssembly component materialization

Use the typed `lib/wasm_component.ncl` contract to author component build
configuration, then provide the resolved pipeline request to the production
shell:

```bash
nix build .#checks.x86_64-linux.wasm-component-toolchain-identity --no-link -L
nix build .#checks.x86_64-linux.wasm-component-toolchain-compatibility --no-link -L
mantle --json wasm-component build component-request.ncl \
  --out target/component-evidence
```

The request must name absolute immutable source/package inputs, the checked
`wkg.lock`, exact local WAC dependency bindings, and the generated-input owner
receipts. Package resolution and every build tool run with network access
unshared. The pinned cohort includes the immutable Octet source revision,
`cargo-octet` package, checked profile configuration and identity, and its
wasm-tools cohort identity.

Successful output includes validated compiled, composed, virtualized, and
metadata-normalized portable bytes; exact Octet receipt/provenance/verification
files; stage receipt objects; `materialization-bundle.json`;
`component-attestation.json`; `component-release-binding.json`; and
`execution-report.json`. Consumers should deserialize the execution report and
call `crunch_wasm_component::verify_pipeline_execution_report_files` before
using any locator. That verifier rehashes published files and the copied source
closure, reconstructs canonical stage/report/bundle identities, and rebuilds
the attestation and release binding.

`wasm-tools strip` is an explicit identity-changing normalization stage: its
output is validated again and is the only portable object admitted to Octet,
Wasmtime smoke validation, optional AOT, and release binding. A Wasmtime
precompile remains target/CPU/configuration-bound trusted native output; it does
not replace the portable component.

When Wizer is enabled, the request must identify Cargo's pre-component core
module, the exact `wasm-component-ld --skip-wit-component` linker split and
compile environment, the Wizer initializer, a denied import set, and the pinned
`wasm-tools component new` configuration. Non-empty virtual-import declarations
fail closed until explicit receipt-bound stub modules are supported. Mantle
validates the core module, runs Wizer twice from clean output paths, and
requires equal BLAKE3 digests and sizes before re-componentizing. Compilation,
both Wizer runs, and
componentization retain receipt-bound environments and subordinate tool
identities. The core and transformed-core artifacts remain explicitly distinct
from the later Component Model artifact; only the latter proceeds through
component validation, composition, virtualization, normalization, Octet, and
consumer remeasurement.

All outputs retain the non-claims `not-component-behavior-correctness`,
`not-runtime-authority`, `not-runtime-sandboxing`, `not-release-eligibility`,
and `not-octet-policy-interpretation`. Mantle records and binds Octet's decision
and findings without reinterpreting them.

## Rust project verification lane

Use `mantle rust-plan` only when you want explicit native Rust planner evidence.
Ordinary project builds stay on `mantle build .#name` and must not silently claim
Cargo-free topology execution. Rust-plan receipts classify their bounded role in
`cargo_mode.compatibility_class`:

- `cargo-oracle-evidence` means Cargo metadata or unit-graph material was used
  as an oracle for comparison.
- `cargo-free-bounded-topology` means `--no-cargo-oracle` planned or executed the
  supported native topology without invoking Cargo.
- `blocked-unsupported-surface` means native planning failed closed on an
  unsupported surface instead of falling back to hidden Cargo orchestration.

The same receipt records `cargo_mode.project_build_status =
"not-default-project-build"`. Do not treat rust-plan evidence as proof of full
Cargo compatibility, compiler correctness, release reproducibility, or bootstrap
correctness.

## Structured refactor sessions

Mantle migrations should be represented as structured refactor session records instead of ad hoc text rewrites. The built-in `crunch-to-mantle-project-identity` session records `mantle` as canonical, `crunch` as a retained compatibility alias, `mantle-project.ncl`/`mantle.lock`/`.mantle/` as canonical project surfaces, `crunch-project.ncl`/`crunch.lock`/`.crunch/` as legacy surfaces, `/mantle/store` as the canonical store prefix, and `/crunch/store` as an explicit compatibility prefix.

Operators can inspect and run sessions with:

```sh
mantle refactor list
mantle refactor plan crunch-to-mantle-project-identity
mantle refactor check crunch-to-mantle-project-identity --refactor-store-prefix /mantle/store
mantle refactor apply crunch-to-mantle-project-identity
```

`plan` and `check` are no-mutate paths: they report file/path/store-prefix operations and typed conflicts without changing project files, lockfiles, generated directories, or store state. Mixed canonical/legacy files and ambiguous `/mantle/store` plus `/crunch/store` defaults fail with remediation. `apply` requires an explicit session id and only runs bounded file/directory renames that the plan marks apply-supported.

## Semantic build graph queries

Mantle records semantic graph data with immutable node identities for source trees, recipes, store outputs, sandbox profiles, providers, proof receipts, witness requests, and release evidence. Human names and aliases are metadata records that point at immutable identities; changing an alias must not change the underlying BLAKE3-backed node identity.

Graph-capable tools can persist a `mantle-semantic-build-graph-v1` JSON file under the state directory as `semantic-graph.json`. Operators can query that file directly:

```sh
mantle graph <output-or-alias>
mantle why <output-or-alias>
mantle dependents <identity-or-alias>
mantle --json why <output-or-alias>
```

`mantle graph` prints the connected node/edge view for a selected root. `mantle why` reports the producing recipe plus linked source, provider, sandbox, proof receipt, witness request, and release evidence nodes when present. `mantle dependents` reports graph nodes that declare dependency edges to the selected identity.

Missing legacy graph data is fail-closed: queries return a typed `mantle-incomplete-semantic-graph-v1` diagnostic instead of inventing source, recipe, proof, or witness edges. Legacy store entries without graph links remain usable for ordinary cache/build reuse.

## Remote realization adapter boundary

Remote derivation realization uses a provider-neutral hash-negotiated handshake
before transport-specific execution. The scheduler sends the handshake version,
realization key, recipe digest, root input digests, platform/profile facts, and
declared capabilities. A worker either reports an unsupported
profile/capability denial or returns the exact receiver-missing content set.

The supported local production path carries that protocol through framed stdio:
the client launches `remote serve --binding stdio-once --executor local-build`,
streams bounded BLAKE3-verified input artifacts before execution, receives the
output through receiver-issued chunk credit, and admits it only after ordinary
signed PathInfo/content/store-prefix/attestation checks. Fenced checkpoints are
receiver-reprobed on retry; equal-content chunks may share a digest while their
artifact positions remain distinct canonical indices.

See [`remote-transfer.md`](remote-transfer.md) and the checked
[`remote-build-loopback`](../examples/projects/remote-build-loopback/) workflow
for the deterministic interruption/resume and receiver-tamper rails. Those
local stdio fixtures do not prove a production P2P listener, SSH deployment,
REAPI compatibility, independent-machine behavior, exactly-once delivery,
worker honesty, or release reproducibility. Provider and cluster-control details
remain outside the core scheduler contract.

## Publish and recover an admitted OCI layout

First produce a verified local layout with `mantle artifact oci-export`. Then
publish its ordinary image, subject-bound Mantle metadata, and a detached-
signature artifact under an explicit typed Nickel trust policy:

```sh
mantle --json artifact oci-push \
  --layout ./bundle.oci \
  --registry https://registry.example.test \
  --repository onix/kernel-bundle \
  --reference reviewed \
  --trust-policy ./registry-trust-policy.ncl \
  --signing-key /secure/registry-signing-key \
  --bearer-token-file ./registry-token \
  --receipt-out push.json
```

Use all three manifest SHA-256 values from `push.json` and the reviewed policy
for a fresh-state pull:

```sh
mantle --json --state-dir ./fresh-state artifact oci-pull \
  --registry https://registry.example.test \
  --repository onix/kernel-bundle \
  --reference reviewed \
  --expected-manifest-digest "$(jq -r .manifest_digest push.json)" \
  --expected-metadata-manifest-digest "$(jq -r .metadata_manifest_digest push.json)" \
  --expected-signature-manifest-digest "$(jq -r .signature_manifest_digest push.json)" \
  --trust-policy ./registry-trust-policy.ncl \
  --bearer-token-file ./registry-token \
  --out ./pulled.oci \
  --report-out import.json \
  --receipt-out pull.json
```

Pull verifies exact image, metadata, and signature tags; signature artifact
subject/metadata/domain linkage; required non-revoked signers; distinct full-key
threshold; and detached Ed25519 signatures before downloading the content
closure. Success then requires verified descriptor bytes, atomic local layout
publication, and ordinary OCI import with `admitted` state. HTTPS is the default;
controlled local registries require explicit `--allow-http`. Mantle does not read
ambient Docker credentials or proxies and does not follow redirects.

See [`kernel-bundle-oci.md`](kernel-bundle-oci.md) and the
[`kernel-bundle-oci-registry`](../examples/projects/kernel-bundle-oci-registry/)
runbook. Signature evidence authenticates only the immutable image/metadata
digest pair under supplied local policy. It does not prove registry authorization,
transparency, revocation freshness, tag immutability, arbitrary-registry
compatibility, exactly-once publication, upload resumption, artifact correctness,
kernel compatibility, deployability, or release eligibility.

## Explain ready-goal priority

Mantle preserves its lazy goal graph while ranking eligible ready goals from
explicit bounded facts. Run a human build with `--verbose` to print the bounded
priority summary, or use JSON mode for complete machine-readable rows:

```sh
mantle --verbose build package.ncl
mantle --json build package.ncl > build-report.json
```

Inspect top-level `scheduler_policy` for the selected typed defaults and
`scheduler_priority_decisions[]` for `policy_id`,
`policy_digest_blake3`, `scheduling_epoch`, the redacted
`selected_goal_key_blake3`, `candidate_snapshot_digest_blake3`, redacted
`runner_up`, `starvation_class`, known-graph path/work/root pressure, normalized
resource/locality/transfer classes, history basis/digest, and
`selection_reason`. Priority-decision rows never contain the selected raw goal
path or provider credentials. `claim_scope = "configured-known-fact-ordering"` and the
`non_claims` list are normative: a priority row does not prove global makespan
optimality, knowledge of future dynamic goals, execution success, output trust,
or release reproducibility.

Route eligibility remains separate. Capability, output-trust, upload privacy,
network, store-prefix, and hard-resource rejection must be resolved before a
candidate enters preference ranking; neither locality nor age can bypass one of
those blockers.

## Enter a dev shell or run a package

`mantle shell` resolves a `devShells` target from the compatibility-named
`crunch.ncl` package root, builds it, then reads `$out/.crunch-shell.json` to
construct the runtime environment.
`mantle develop` is the deprecated alias for the same implementation.

```bash
# Run one command inside the resolved shell
mantle shell --command env

# Run a script through $SHELL -c inside the shell environment
mantle shell --run 'echo "$CRUNCH_SHELL"'

# Add extra PATH entries from another output or directory
mantle shell --with /path/to/extra-tools --command my-tool

# Skip or enforce shell hooks
mantle shell --no-hook --command true
mantle shell --strict-hooks --command true

# Deprecated alias
mantle develop
```

Notes:

- The built shell output must contain `.crunch-shell.json`. If it does not,
  mantle fails clearly and points back at `mkShell`.
- `CRUNCH_SHELL` is always set to the built shell output path.
- `--command` and `--run` are mutually exclusive.

`mantle run` is the package-side entry point. It builds a package target from
the compatibility-named `crunch.ncl` package root and executes the first program
under `bin/`.

```bash
# Run the default package from the compatibility-named crunch.ncl package root
mantle run

# Run a named package
mantle run .#hello

# Pass arguments through after --
mantle run .#hello -- --help
```

If the selected package output does not contain `bin/`, `mantle run` fails
instead of guessing.

## Inspect and verify attestations

Successful builds, accepted cache hits, and remote substitutions persist native
artifact attestations under the mantle state directory. Use `mantle attest` to
inspect those sidecars and the closure or project views derived from them.

```bash
# Show one artifact attestation
mantle attest show /nix/store/<hash>-hello

# Assemble one runtime closure attestation from rooted outputs
mantle attest closure /nix/store/<hash>-hello

# Verify persisted sidecars against canonical reconstruction
mantle attest verify artifact /nix/store/<hash>-hello
mantle attest verify closure /nix/store/<hash>-hello

# Compare two native attestation documents or artifact selectors
mantle attest diff /nix/store/<hash>-a /nix/store/<hash>-b

# Render or verify a project attestation
mantle attest project /nix/store/<hash>-root
mantle attest verify project --file saved-project-envelope.json /nix/store/<hash>-root
mantle attest verify project --digest <canonical-hex> /nix/store/<hash>-root
```

Selector rules:

- `show` and `verify artifact` accept a logical store path, an exported store
  path, or a raw store-path string.
- `closure` and `verify closure` accept one or more rooted outputs.
- `project` and `verify project` need at least one selected built root.
- `diff` accepts either saved attestation JSON or store-path selectors.

## Package and verify release evidence

Release evidence starts from a full proof run, not from
`./scripts/prove-self-hosting.sh --check`.

```bash
# Produce a full proof bundle first
./scripts/prove-self-hosting.sh

# Package release evidence from tracked worktree files, witness workflow driver,
# stage0 inventory, self-hosting test files, and verified vendored Cargo inputs
mantle release create \
  --release-id mantle-<version> \
  --binary target/self-hosting-proof/run-.../binaries/stage2-mantle \
  --proof-bundle target/self-hosting-proof/run-...

# Rebuild published artifacts into an isolated output area and compare bytes
mantle release reproduce target/release-evidence/<release-id> \
  --rebuild-output-dir target/release-rebuild/<release-id> \
  --rebuild-command ./scripts/rebuild-release-artifacts.sh

# Smoke the generated deterministic proof path and save a rail receipt/log
cargo -Zscript scripts/release-determinism-smoke.rs
cargo -Zscript scripts/check-release-determinism-smoke-receipt.rs \
  target/release-determinism-smoke/latest/receipt.json

# Run the real release-specific proof rail: full proof bundle -> release bundle
# -> two clean deterministic proof rebuilds -> verified deterministic receipt
./scripts/prove-real-release-determinism.sh

# Reuse an existing full self-hosting proof bundle instead of rerunning it
./scripts/prove-real-release-determinism.sh \
  --proof-bundle target/self-hosting-proof/run-...

# Re-check a saved real deterministic proof rail output
cargo -Zscript scripts/check-real-release-determinism-receipt.rs \
  target/release-evidence/<release-id>

# Write portable JSON/Markdown summary artifacts for archival/review
cargo -Zscript scripts/summarize-real-release-determinism.rs \
  target/release-evidence/<release-id>

# Inspect bootstrap parity's checked self-build proof descriptor consumption
mantle --json bootstrap parity-report

# Save and validate a bounded bootstrap parity snapshot receipt under target/
./scripts/check-bootstrap-parity-snapshot.sh

# Re-check a saved bundle using only bundle-local contents
mantle release verify target/release-evidence/<release-id>

# Require a verified bit-for-bit reproducibility report
mantle release verify target/release-evidence/<release-id> --require-reproducible

# Require Valence stack-provenance sidecar and graph-report evidence for Onix stack releases
mantle release verify target/release-evidence/<release-id> --release-profile onix-stack
```

Repeat `--binary` when one release bundle should carry multiple executables.
The checked-in proof bundle keeps durable copies of stage1 and stage2 under
`binaries/`, so the packaged release binary can be the proven stage2 output
rather than a scratch-store path that disappears when the proof exits.

`mantle release create --bundle-dir <path>` requires `<path>` to be absent. Do
not pre-create an empty destination. Mantle computes a deterministic BLAKE3
publication plan before mutation, stages through a private capability root under
the destination's real parent, writes the canonical manifest after every planned
artifact, verifies that complete stage with the production verifier, and then
uses one atomic no-replace rename. Existing and concurrently created files,
symlinks, empty directories, and nonempty directories are never overwritten.

A failed pre-commit attempt leaves the final path absent and normally removes its
current stage. If cleanup was interrupted, retry may quarantine only a sibling
whose bounded ownership marker matches both the publication-plan identity and
final destination name. Unrecognized siblings stay untouched. Staging names are
fresh for each attempt and are excluded from plan identity. This is an atomic
local visibility and no-clobber guarantee; rename alone is not evidence of
filesystem crash durability, power-loss persistence, artifact correctness, or
release eligibility.

Generic release verification leaves Valence stack provenance optional unless
`--stack-provenance required` is selected. The `onix-stack` release profile makes
that requirement profile-declared: verification fails closed unless the bundle
carries the Valence sidecar, graph-report receipt, matching BLAKE3 digests,
expected roles/schemas, `identity-linkage-sidecar` claim scope, release-binary
identity, and Mantle's opaque-boundary non-claim. Mantle still validates only
bundle-local evidence linkage; Valence owns stack semantics.

Kani receipts can be bundled with `--external-evidence-role
kani-model-check-receipt` plus `--kani-toolchain-evidence <json>`. The dedicated
metadata records Kani, Rust, CBMC, solver, wrapper, closure, and receipt linkage
identity only; Valence remains responsible for Kani semantic validation. See
[`docs/kani-release-evidence.md`](kani-release-evidence.md).

When `--provider-fixed-point-proof <dir>` is supplied to release creation, the
provider proof must validate and its fixed-point stage binary digest must match
one of those packaged `binaries/` artifacts. Required verification applies the
same binding for bundled and external provider proof evidence and reports the
matched release artifact path/digest. This is provider-backed artifact evidence,
not a deterministic-release, compiler-correctness, full-bootstrap, deploy, or
full Cargo compatibility claim.

Current bounded provider-bound evidence is summarized in
[`docs/release-notes/provider-bound-release-evidence-2026-06-28-provider-remap-fixed.md`](release-notes/provider-bound-release-evidence-2026-06-28-provider-remap-fixed.md).
The durable local artifact copy is recorded there; publish that wording as
provider-bound/local quorum evidence, not as an external independent rebuild
unless a separate operator supplies the counted witness sidecar.

`mantle release verify` proves bundle-local integrity and proof-context by
itself. It checks that required bundled artifacts exist, that the manifest stays
canonical, that recorded digests still match, and that the nested proof bundle
is a full proof artifact. The human success marker is terminal: it is written
only after the complete selected policy decision is accepted. Policy rejection
uses `release evidence rejected`, emits ordered check diagnostics, and exits
nonzero without first printing the success marker.

Machine output is versioned as `mantle-release-verify-v2`. Both accepted and
policy-rejected decisions contain top-level `valid`, `disposition`, ordered
`checks`, and ordered `diagnostics`. A policy-rejected decision is still one
parseable JSON value on stdout and still exits nonzero; operational failures
that prevent a decision remain ordinary command errors. JSON consumers migrating
from `mantle-release-verify-v1` MUST require both a zero process status and
`valid: true`, branch on the `kind` version, and MUST NOT infer acceptance from
the presence of a parsed manifest or verification payload.

Reproducibility is reported separately as `absent`,
`matched`, or `mismatched`. The bit-for-bit reproducible release label requires
a verified canonical reproducibility report whose artifact set matches the
published release artifact set; ordinary bundle-local integrity never implies
that label. This still does not prove a full-source bootstrap root,
independent rebuild agreement, or global reproducibility. `release verify` now
reports global reproducibility as `not-evaluated`. To prepare release-derived
surface evidence first, use `mantle release global-reproducibility-evidence
--universe <json> --policy <json> --bundle-dir <release-evidence>
--verification-dir <release-verification> --release-verify-json <json>
--evidence-path <json>`. Then use `mantle release global-reproducibility
--universe <json> --policy <json> --evidence <json> --report-path <json>` to
produce the separate `mantle-global-reproducibility-report-v1` admission report
for an explicit universe. Provider fixed-point release artifacts are admitted
only when the bundled provider proof verifier is valid and its stage digest
matches the release artifact; invalid or incomplete provider proof material
stays blocker-producing evidence. The helper output alone is not an eligible
report; the evaluator remains the gate. `bootstrap parity-report` also consumes the checked-in compact
descriptor at
`bootstrap/evidence/real-self-build-proof-parity.json` for the
`crunch.self-build` row; that descriptor surfaces the bounded proof digest and
provider kind, but the row remains partial and still blocks Guix/StageX parity
until the separate source-root/lineage blockers are closed.

## Sign and verify decentralized release material

Once a release bundle verifies locally, sign it into a verification directory:

```bash
mantle release attest target/release-evidence/<release-id>
```

That writes `target/release-verification/<release-id>/release-attestation.json`
and a matching `.sig` sidecar by default.

Before witness publication, export the trusted public key, scaffold the
verifier-local social policy files, and export a portable witness-request
directory:

```bash
RELEASE_TRUSTED_KEY=$(mantle attest key-show --signing-key /path/to/release.key)
RELEASE_SIGNER_NAME="${RELEASE_TRUSTED_KEY%%:*}"

mantle attest policy-init target/release-verification/<release-id> \
  --profile single-witness \
  --trusted-release-signer "$RELEASE_SIGNER_NAME" \
  --trusted-witness-identity <witness-identity>

mantle release witness-export target/release-evidence/<release-id> \
  --verification-dir target/release-verification/<release-id> \
  --request-dir target/release-witness-requests/<release-id>
```

Use `--profile self-proof-only` when the verification directory should stay at a
self-proof-only policy with `min_matching_witnesses = 0`. Use
`--profile single-witness` when one matching witness should be enough to satisfy
policy. `mantle attest key-show` prints the exact `name:base64` verifier token
accepted by `--trusted-public-key`; omit `--signing-key` to read the default
configured signing key instead. The signer name for `--trusted-release-signer`
is the token prefix before the colon, so a shell workflow can derive it with
`RELEASE_SIGNER_NAME="${RELEASE_TRUSTED_KEY%%:*}"` when the release key was
auto-generated in a per-operator `CRUNCH_CONFIG_DIR`. `mantle attest
policy-init` writes `policy.json` plus an explicit empty `revocations.json`,
and it refuses to overwrite either file unless `--force` is present. `mantle
release witness-export` copies only public verification material into the
request directory: the verified release-evidence bundle, the signed release
attestation, and request metadata. It does not copy signing keys or
verifier-local policy files.

Independent rebuilders can then replay the checked-in witness rebuild rail in a
second environment:

```bash
./scripts/rebuild-witness-request.sh \
  target/release-witness-requests/<release-id> \
  --identity <witness-identity> \
  --system x86_64-linux \
  --toolchain rust-1.91.1 \
  --host-class nixos-25.05 \
  --signing-key /path/to/witness.key
WITNESS_TRUSTED_KEY=$(mantle attest key-show --signing-key /path/to/witness.key)
```

The wrapper is the operator-facing shell. It resolves `bwrap`, picks an
absolute static `SNIX_BUILD_SANDBOX_SHELL`, derives a controlled scratch root
(`target/release-witness-requests/<release-id>.work/` by default or
`$CRUNCH_WITNESS_SCRATCH_DIR`), rewrites `TMPDIR` and `CARGO_TARGET_DIR` under
that root, and then calls the machine-readable core command `mantle release
witness-rebuild <request-dir> ...`. The replay preserves the release bundle's
recorded proof mode, including `--non-nix-host` for non-Nix-host proofs. Use
`--check` when you want prerequisite and request-validation preflight only;
successful `--check` output is not rebuild proof. The request directory stays
immutable after validation. Witness sidecars
and the rebuild audit directory land under the scratch verification output,
not back inside the exported request tree.

Inspect the discovered material with:

```bash
mantle attest release-show target/release-verification/<release-id>
mantle attest witness-show target/release-verification/<release-id>
```

Then import the returned witness sidecars and verify technical status plus
social policy against trusted key material:

```bash
mantle attest witness-import target/release-verification/<release-id> \
  target/release-witness-requests/<release-id>.work/release-verification/<release-id>
mantle attest release-verify target/release-verification/<release-id> \
  --trusted-public-key "$RELEASE_TRUSTED_KEY" \
  --trusted-public-key "$WITNESS_TRUSTED_KEY"
```

The scratch verification directory also carries `witness-rebuild-audit/meta.json`
plus captured workflow stdout/stderr so the witness can hand back both the
signed sidecars and a replay transcript. `mantle attest witness-import` fails
closed on missing signatures, release-digest mismatches, and conflicting
existing witness identities before copying anything into the publisher
verification directory.

`mantle attest release-verify --json` reports bundle-local technical validity,
social policy sufficiency, and independent rebuild agreement as separate
fields. The independent agreement fields are `independent_agreement_status`,
`independent_agreement_class`, `independent_agreement_report_digest`,
`independent_agreement_counted_witness_count`,
`independent_agreement_skipped_witness_count`,
`independent_agreement_failed_witness_count`, and per-witness classification
reasons under `independent_agreement_witnesses`. Agreement is derived from the
accepted witness files, the verifier-local `policy.json`, `revocations.json`,
and trusted keys passed on the command line. It is not a hand-authored claim.
Unknown-key or bad-signature witnesses remain visible in
`discovered_witness_count` and in the independent-agreement witness list, but
are skipped before quorum evaluation. Signature-valid witnesses with wrong
release references or rebuilt binary digests are failed evidence, not counted
agreement. If a canonical agreement report is present at
`target/release-verification/<release-id>/agreement-report.json`, verification
checks it byte-for-byte against the derived report; duplicate
`agreement-report.json` attachments, including nested
`independent-agreement/agreement-report.json`, are rejected as ambiguous.
Release-evidence bundles may carry the same optional report artifact at
`independent-agreement/agreement-report.json`, where normal manifest digest
verification applies.

A successful single-witness run proves external witness agreement only under
the configured policy; a satisfied independent-agreement status gives the
stronger verifier-local `independent-rebuild-agreement` class for the accepted
witness set. It still does not prove a full-source bootstrap root, globally
reproducible release outputs, or global reproducibility for all Mantle build
surfaces. The global claim requires an eligible
`mantle-global-reproducibility-report-v1` for a digest-bound universe and
policy; blocked reports remain useful blocker inventories but must not be
promoted into claim text.

For the current trust boundary behind those claims, see
[`docs/bootstrap-stage0-inventory.md`](bootstrap-stage0-inventory.md).
For the checked-in benchmark workflow, see
[`docs/benchmark-suite.md`](benchmark-suite.md).
