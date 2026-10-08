# Operator workflows

This page complements the top-level README.

Use the checked [canonical workflow](generated/canonical-operator-workflow.md)
for the short daily path. Use the generated
[command reference](generated/operator-command-reference.md) for the complete
public command catalog and its side-effect contracts.

Clap owns parser behavior. The typed Nickel inventory owns reviewed support and
compatibility policy. Use this page only for longer operator runbooks.

## Validation tiers

Use the checked-in toolchain from `rust-toolchain.toml`.

### Ordinary first-party gate

Run the checked-in wrapper from the repo root:

```bash
./scripts/check-first-party-quality.sh
```

It runs four ordinary edit-time checks in order: the bounded GCC 4.0
configure-bridge self-test, package-scoped rustfmt, strict first-party Clippy,
and serialized first-party workspace lib/tests. The script
`scripts/quality-gate-common.sh` owns the canonical fmt package selection and
vendored workspace exclusions. Manual equivalents:

```bash
./scripts/check-gcc40-configure-bridge.rs --self-test
source scripts/quality-gate-common.sh
cargo_fmt_first_party
./scripts/check-first-party-clippy.sh
cargo_test_workspace_lib_tests
```

Notes:

- Vendored workspace-member test binaries stay on their focused rails because
  some require host capabilities such as a usable FUSE mount. Their libraries
  still compile through first-party consumers.
- The wrapper serializes libtest cases so process-global environment, lock,
  and resource-pressure fixtures cannot interfere across otherwise unrelated
  tests. Tests that own concurrency still exercise it internally.

- For any changed first-party package outside the selected fmt packages, also
  run `cargo fmt --check -p <package>`; the selected fmt leg does not check it.
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

Create or check the checkout-local `vendor-deps/` closure with
`python3 scripts/vendor-deps.py generate` or `python3 scripts/vendor-deps.py check`
from `nix develop`. `generate` never replaces an existing `vendor-deps/`. See
[Store backends](store-backends.md#pinned-casita-dependency) for the pinned Casita
source and its tracked patch.

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

If the root declares pinned flat fixed-output HTTPS fetches and the exact
archives are already cached locally, repeat `--cached-fetch
'<pinned HTTPS URL>=<absolute local archive file>'` on the export command for
each archive. The URL must match the root's original fetch declaration exactly;
export verifies the captured archive bytes against its declared fixed-output
hash and keeps that URL in the bundle. This does not download anything, cannot
be combined with `--fetch-missing`, and proves source availability and identity
only—not tool binary provenance or a successful downstream build.

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

For a bounded handoff containing only the two inputs absent from a fresh clone,
use the dedicated profile mode:

```bash
mantle source bundle bootstrap-profile \
  --mode fresh-clone-inputs \
  --provider-archive ./unpacked-legacy-provider \
  --provider-manifest ./provider.json \
  --vendor-deps ./vendor-deps \
  --to fresh-clone-inputs.json
```

`--provider-archive` is the unpacked payload expected by the legacy
`fetchTarball` input, not the reduced provider output. The pinned provider has
case-distinct Linux kernel header names, so export and hydration require a
case-sensitive filesystem; post-materialization identity validation fails
closed if those names cannot coexist. Record the reported
`manifest_blake3` through an independent channel from the bundle itself. For a
fresh clone whose ignored `vendor-deps/` is absent, hydrate
the Cargo directory source and legacy-provider source state in one no-clobber
operation:

```bash
mantle --json --state-dir ./offline-state source bundle hydrate-self-build \
  --from /media/handoff/bootstrap-source-bundle.json \
  --expected-manifest-blake3 <manifest-blake3> \
  --checkout . > self-build-source-hydration.json

empty_cargo_home="$(mktemp -d)"
CARGO_HOME="$empty_cargo_home" CARGO_NET_OFFLINE=true \
  cargo metadata --offline --locked --format-version 1 \
  --config .cargo/vendor-config.toml
```

Hydration validates the out-of-band manifest identity, requires one vendor
record plus the legacy provider archive and provider manifest, validates every
vendored package/file checksum against the fresh clone's `Cargo.lock`, publishes
`vendor-deps/` with atomic no-replace semantics, and imports and pins the bundle
under `./offline-state`. It refuses an existing `vendor-deps/` rather than
merging or replacing it. The JSON report is contracted as
`mantle-self-build-source-hydration-v1` and binds the manifest, vendor, and
provider archive BLAKE3 identities without embedding checkout or temporary
paths.

### Lock-driven vendor acquisition outside the source archive

The `fresh-clone-lock-vendor-inputs` profile keeps the legacy provider
archive/manifest authority and the ordinary source-bundle v1 format, but **does
not embed** `vendor-deps/`. Build the bounded lock-vendor producer, its canonical
selected shared-hash table, and the final vendor output as separate strict,
fixed-output/store steps first. The selected table contains only rows referenced
by `Cargo.lock`; the mutable full reviewed table
`bootstrap/pins/cargo-shared-lock-hashes-v1` is neither copied into the Mantle
source record nor bound to the producer for unrelated packages.

For the full workspace, create the bounded producer input archive from **declared**
sources before registering its fixed-output store path:

```bash
python3 scripts/vendor-source-bundle.py \
  --profile workspace \
  --index /declared/locked-sparse-index \
  --reviewed-table bootstrap/pins/cargo-shared-lock-hashes-v1 \
  --git-cache /declared/offline-cargo-git \
  --git-sidecars /declared/locked-git-sidecars.json \
  --to /handoff/lock-vendor-source.tar.gz
```

The sidecar JSON maps **each exact `Cargo.lock` Git source** under `"git"` to
`"git_db"` and `"checkout"` paths relative to `--git-cache`, plus an explicit
absolute `"git_tree"` path to its independently fetched pristine source.
Exporter admission checks every selected Git source against the reviewed
table, the Git commit and checkout origin/revision, every fetched tree's
reviewed NAR SHA-256, and pinned Git blobs/modes (including checked-in CRLF
attributes).
The archive embeds the bounded Git object databases and checkouts and a
per-source `git-transports.json` with reviewed fetch URL, revision and NAR
hash; it does **not** embed the full global table, the fetched source-tree
inputs, ambient Cargo-home paths or `vendor-deps/`. The onix-artifact
SSH-to-HTTPS fetch route requires its explicit reviewed table row; an
offline pinned checkout for an unavailable Git remote does not prove that
remote is reachable. Selected Git fetches and registry archives remain
separate fixed-output derivation inputs, not aliases for these sidecars.

```bash
mantle --nix-compat source bundle bootstrap-profile \
  --mode fresh-clone-lock-vendor-inputs \
  --provider-archive ./unpacked-legacy-provider \
  --provider-manifest ./provider.json \
  --mantle-source ./mantle-src \
  --lock-vendor-producer /nix/store/<producer>/bin/lock-vendor-producer \
  --lock-vendor-selected-table /nix/store/<selected-table> \
  --lock-vendor-output /nix/store/<completed-vendor> \
  --excluded-vendor-deps ./previously-verified-vendor-deps \
  --to lock-vendor-inputs.json
```

Use the same logical store prefix as the signed producer output PathInfo;
`--nix-compat` above binds `/nix/store`. A physical export directory under
`/tmp` is only a carrier for that signed output, not a new logical store path.

The excluded tree and new producer output must have the same bounded content
and bytes and independently match every locked package/checksum. The profile
records the producer executable identity, selected-table identity, `Cargo.lock`
identity, output logical store path and vendor-tree content receipt; its acquisition
record has zero embedded payload bytes. Verify the separate completed output's
store signature/PathInfo independently: a source manifest receipt is **not**
store-output trust, compiler correctness, or proof that the producer ran. Send
the signed output alongside the source bundle and publish the profile's
`manifest_blake3` over an independent authenticated channel.

On a fresh checkout without `vendor-deps/`, provide the separately realized
output explicitly; no command substitutes a host Cargo cache or network:

```bash
mantle --nix-compat --json --state-dir ./offline-state source bundle hydrate-self-build \
  --from /media/handoff/lock-vendor-inputs.json \
  --expected-manifest-blake3 <manifest-blake3> \
  --checkout . \
  --lock-vendor-output /nix/store/<completed-vendor> \
  > self-build-source-hydration.json

empty_cargo_home="$(mktemp -d)"
CARGO_HOME="$empty_cargo_home" CARGO_NET_OFFLINE=true \
  cargo metadata --offline --locked --format-version 1 \
  --config .cargo/vendor-config.toml
```

Hydration refuses an existing tree, checks the checkout lock against the
producer-bound lock, remeasures the output, verifies each staged vendored
package/file against `Cargo.lock`, publishes with atomic no-replace, and pins the
source state. Missing materialization fails before self-build preflight.
`fresh-clone-lock-vendor-fixed-point` adds ordinary `--proof-input` and
`--include-bundle` fixed-fetch closure inputs before offline fixed-point
proof; `fresh-clone-inputs` and `fresh-clone-fixed-point` remain embedded-vendor
compatibility profiles.

### Hydrated fresh-clone fixed-point proof

The three-record `fresh-clone-inputs` compatibility profile remains unchanged.
The selected self-build seed is the admitted full-source provider; the legacy
archive and manifest records retained by the profile are hydration compatibility
inputs, not selected-provider fallback. To prove the stronger fixed point
without live builtin fetches, a connected producer first captures the complete
evaluated full-source fixed-fetch closure through Mantle's ordinary fixed-output
verifier. Tarball records retain the compressed acquisition bytes;
offline use replays the same bounded extractor and recursive verifier. Files
larger than 64 MiB are represented by deterministic contiguous entries without
raising the 64 MiB per-entry limit:

```bash
mantle --state-dir ./producer-source-state source bundle export \
  --build-root bootstrap/bwrap.ncl \
  --build-root bootstrap/busybox.ncl \
  --build-root bootstrap/rust.ncl \
  --import-path lib \
  --fetch-missing \
  --to evaluated-self-build-fetches.json

mantle source bundle bootstrap-profile \
  --mode fresh-clone-fixed-point \
  --provider-archive ./unpacked-legacy-provider \
  --provider-manifest ./provider.json \
  --vendor-deps ./vendor-deps \
  --include-bundle evaluated-self-build-fetches.json \
  --to fresh-clone-fixed-point.json
```

Publish the resulting `manifest_blake3` through an independent authenticated
channel. On a Git clone that starts without `vendor-deps/`, source state, Cargo
cache, or prior proof outputs, hydrate the full profile and keep the JSON report:

```bash
mantle --json --state-dir ./source-only-state source bundle hydrate-self-build \
  --from /media/handoff/fresh-clone-fixed-point.json \
  --expected-manifest-blake3 <manifest-blake3> \
  --checkout . > self-build-source-hydration.json

empty_cargo_home="$(mktemp -d)"
CARGO_HOME="$empty_cargo_home" CARGO_NET_OFFLINE=true \
  cargo metadata --offline --locked --format-version 1 \
  --config .cargo/vendor-config.toml

CARGO_NET_OFFLINE=true CRUNCH_NO_FUSE=1 \
  ./scripts/prove-self-hosting.sh \
  --source-state ./source-only-state \
  --source-manifest-blake3 <manifest-blake3> \
  --hydration-report ./self-build-source-hydration.json
```

The helper copies only the authenticated `source-bundles/records` and
`source-bundles/pins` into each fresh stage state. Both stages run with
`require-override`: an unmatched builtin URL, fetch kind, or Git revision fails
before URL, proxy, DNS, Git, or HTTP acquisition. Successful stage reports bind
the same source-state BLAKE3 and zero live-fetch events. The proof bundle retains
the hydration receipt, the committed full-source provider-admission report, and
emits the path-redacted contracted `fresh-clone-fixed-point.json` report with
`provider_kind=full-source` and stage1/stage2 binary BLAKE3 equality.
Do not use `latest` or cite success unless that report has `fixed_point: true`.

This evidence is bounded to the committed evaluated closure, admitted
full-source provider, recorded platform, checkout stage0, and proof tool
boundary. It does not prove compiler correctness, bootstrap-seed correctness,
bit-for-bit release reproducibility, independent rebuild agreement, deployment
success, or full Cargo compatibility. See [ADR 0032](../adr/0032-deny-live-source-acquisition-in-hydrated-fixed-point-proofs.md).

### Source-built fixed-point source authority

Use the `source-built-fixed-point` profile when the proof must construct its
providers. This profile carries source authority only. It rejects a provider
manifest in place of the StageX lineage manifest.

```bash
mantle source bundle bootstrap-profile \
  --mode source-built-fixed-point \
  --provider-archive bootstrap/seeds/AMD64/hex0-seed \
  --provider-manifest bootstrap/stagex-transition-lineage.json \
  --bootstrap-source /media/handoff/native-source-closure.json \
  --stagex-source-bundle /media/handoff/stagex-source-closure.json \
  --mantle-source . \
  --vendor-deps ./vendor-deps \
  --proof-input /media/handoff/rust-source-archives \
  --include-bundle /media/handoff/native-source-closure.json \
  --include-bundle /media/handoff/stagex-source-closure.json \
  --to source-built-fixed-point-sources.json
```

For this mode, `--provider-archive` carries the audited hex0 seed. The
`--provider-manifest` flag carries the validated StageX lineage manifest. The
names remain CLI compatibility surfaces; neither input is a provider output.
The `--bootstrap-source` record binds exactly one original native source
manifest. The `--stagex-source-bundle` record binds the exact source bundle
accepted by the StageX transition. The two included bundles supply their
materialized source records. Exact duplicate records are merged. Conflicting
records fail. The single `--proof-input` directory carries all authenticated
Rust source archives. Files larger than 64 MiB use canonical, contiguous
source-record chunks.

Profile verification checks the bundle that you declared. It does not prove that
an older native bundle still matches the current `bootstrap/seed-full-toolchain.ncl`
graph. Before a long proof, export and verify that current build root again. The
proof's offline preflight remains the fail-closed parity check.

Hydration publishes only `vendor-deps/`. It imports and pins the seed, lineage,
source records, Rust archives, and Mantle source record. It does not create
transition, provider, Rust-provider, or Mantle output directories. Those
directories must remain absent until the source-built proof shell starts.
See [ADR 0050](../adr/0050-build-the-source-fixed-point-through-one-rust-proof-authority.md)
and [ADR 0051](../adr/0051-cut-legacy-bootstrap-edges-at-the-stagex-provider.md).

Refresh an existing verified profile after a Mantle source change:

```bash
mantle source bundle refresh-mantle-source \
  --from /media/handoff/source-built-fixed-point-sources.json \
  --mantle-source . \
  --include-bundle /media/handoff/new-host-tool-sources.json \
  --to /media/handoff/refreshed-source-built-fixed-point-sources.json
```

The output path must not exist. The command replaces exactly one Mantle source
record. It preserves every other source record and can add materialized fetch
records from repeated `--include-bundle` arguments. It recomputes the profile
BLAKE3. It rejects missing, duplicate, mixed-mode, conflicting, classified, or
changed authority metadata. Supplemental records must be materialized fetch
inputs. The proof still requires every native and StageX record to match its
bound manifest. The expected profile and combined source-manifest BLAKE3 values
bind each added record.

Run the proof with independent digests and explicit executable paths:

```bash
CRUNCH_NO_FUSE=1 mantle self-build \
  --source-built-fixed-point \
  --source-profile /media/handoff/source-built-fixed-point-sources.json \
  --expected-source-profile-blake3 "$SOURCE_PROFILE_BLAKE3" \
  --expected-stagex-lineage-blake3 "$STAGEX_LINEAGE_BLAKE3" \
  --expected-native-provider-blake3 "$NATIVE_PROVIDER_BLAKE3" \
  --proof-bwrap /absolute/path/to/bwrap \
  --proof-sandbox-shell /absolute/path/to/static-busybox \
  --out /absolute/fresh/path/source-built-proof
```

The output path and its two success aliases must be absent. A failed run keeps
its private staging directory and `attempt-status.json`. A successful run
publishes with a no-replace rename and updates both aliases atomically.

With global `--verbose`, the source-built attempt also writes
`mantle-service-readiness-v1` coordination snapshots to stderr. The six
proof-stage dependencies come from the real output-authority edges. StageX
may report `started` during isolated execution; the current observer never
asserts `complete` because accepted action-reconciliation observations and
the required final receipt are not yet available to its shell boundary.
These non-evidence snapshots never replace the proof report or receipt.

For a prepared checkout that already has its explicit vendor directory, import,
pin, and preflight the bundle directly:

```bash
mantle --state-dir ./bootstrap-state source bundle import --from bootstrap-source-bundle.json --pin
mantle --state-dir ./bootstrap-state source bundle bootstrap-profile \
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
mantle --state-dir ./bootstrap-state bootstrap --fetch --offline-source-preflight --output seed.ncl
```

`bootstrap --fetch` opens its state directory with the legacy `/crunch/store`
logical prefix, whatever `--store-prefix` says, and its offline source
preflight reads the records imported into that same directory. A state
directory whose `store-identity.json` records another prefix, for example
after `mantle build` or a `store` command with the default `/mantle/store` as
in the [offline build runbook](#offline-build-runbook), fails with
`store-identity-mismatch` before anything is fetched or built. Import the
bundle into a state directory that only source bundle commands and the legacy
bootstrap use, and pass `--store-prefix /crunch/store` to later store commands
on it, such as `store verify`. Under `--store-backend casita`, a state
directory that holds imported source bundles but no store identity fails with
`store-backend-mismatch`, and offline source preflight has no passing
validation; see [Store backends](store-backends.md#state-identity).

Do not report bootstrap source-bundle readiness or fresh-clone hydration as
provider trust removal, compiler correctness, fixed-point self-build success,
completeness for undeclared future bootstrap sources, release reproducibility,
independent rebuild agreement, or full bootstrap correctness. Those claims still
require the existing proof commands and evidence gates.

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

Both Cargo and pin import apply keep their output paths under the selected
workspace using no-follow directory capabilities; symlinked output parents and
overlapping output targets are refused before writing. A successful apply
independently reads the opened output files back and checks their actual paths
and bytes against the plan. A write or read-back failure does **not** roll back
earlier files: inspect the workspace and re-plan before retrying. A blocked
plan exits with code 3 and reports its blockers without claiming an apply.

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

## Cargo unit dynamic-plan lane

[`examples/cargo_unit_plan.ncl`](../examples/cargo_unit_plan.ncl) is an opt-in,
seed-dependent Linux example: an offline Cargo unit-graph producer emits a
`mantle-plan-v2` with separately admitted package source slices and Rust unit
outputs; a declared static verifier consumes the bound app unit root. Ordinary
`mantle build .#name` remains the project-build path, while `mantle rust-plan`
is separate native planning evidence, not this Cargo-unit execution lane.

Before running from the repository root, review and provision the example's
**eight exact static input roots** (the producer's seven declared inputs plus
the static app verifier) and their transitive closure in the local Nix store.
The example contains pinned `/nix/store` paths and a producer config JSON for
one reviewed host; on another host, replace those paths and regenerate the
declared config and expected app root. Do not treat the checked-in hashes as
portable seeds or fetch undeclared inputs automatically. Set `CACHE_DIR` to a
private absolute signed Nix file-cache directory, `SIGNING_KEY` to its reviewed
local Nix-format secret key, `TRUSTED_PUBLIC_KEY` to the matching
`name:base64` public key, `INPUT_ROOTS` to a Bash array of those exact eight
store roots, `CLIENT_STORE` and `CLIENT_STATE` to initially empty dedicated
absolute client paths, and `BWRAP` and `SANDBOX_SHELL` to reviewed pinned
Linux bubblewrap and static BusyBox executable paths. These variables require
operator substitution; the following commands are **not** host-independent:

```bash
nix copy --offline \
  --to "file://$CACHE_DIR?compression=none&secret-key=$SIGNING_KEY" \
  "${INPUT_ROOTS[@]}"
mantle --nix-compat --store "$CLIENT_STORE" --state-dir "$CLIENT_STATE" \
  store pull --from "$CACHE_DIR" \
  --trusted-public-keys "$TRUSTED_PUBLIC_KEY" --all
SNIX_BUILD_BWRAP="$BWRAP" SNIX_BUILD_SANDBOX_SHELL="$SANDBOX_SHELL" \
  mantle --json --nix-compat --store "$CLIENT_STORE" --state-dir "$CLIENT_STATE" \
  build --no-substitute --trusted-public-keys "$TRUSTED_PUBLIC_KEY" \
  -I lib examples/cargo_unit_plan.ncl
```

Inspect the import counters: reject untrusted signatures, NAR hash mismatches,
missing NARs, or parse failures rather than adding `--trust-unsigned`. Inspect
the build report's `native_dynamic_plans`: raw and canonical plan digests
must agree, both source slices must have declared and independently observed
NAR BLAKE3 equality with `disposition=admitted`, and both expected unit IDs
must be accepted. The `plan_output_bindings` row must bind the requested app
root to the realized app unit output; inspect the direct dependency manifest,
execute the app unit binary, and read the static verifier output. On the
reviewed pinned cohort, the accepted digest was
`6682e8f14649205c276f91cdd3d8330f58b57daf703d95eb9e16367d6c3ee6af`,
the app root was
`u.a1b9ca38f6231fb003fd22ff87350acddc895585f48116b9eaa7162a3800722b`,
and the physical verifier output was `cargo-unit-app-ok\n`.

For the negative check, make a separate scratch Nickel fixture importing the
same reviewed producer and verifier; change **only** its requested app root to
`u.0000000000000000000000000000000000000000000000000000000000000000`.
Run the same signed-client `build --no-substitute` command against that
scratch fixture. Expect nonzero exit with `plan_output_bindings` reporting
`status=rejected`, `failure_reason=plan-output-root-missing`, and null root
derivation/output paths; the verifier must not run. The current-root
fresh-import client required one same-state recovery after a 1800-second
first-command timeout: that recovered positive reported four successes
(two built, two cached) and zero failures; its wrong-root negative reported
zero builds and one typed failure. These are not uninterrupted cold-build or
final immutable SECOND receipts. This adapter caps source slices at 256;
the measured Mantle workspace has 925 units and 711 distinct package
sources, so it is outside this bounded lane. Do not infer full Cargo or LLVM
correctness, compiler correctness, release reproducibility, or bootstrap
correctness from this example.

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

The checked local production path carries that protocol through framed stdio:
the client launches `remote serve --binding stdio-once --executor local-build`,
streams bounded BLAKE3-verified input artifacts before execution, receives the
output through receiver-issued chunk credit, and admits it only after ordinary
signed PathInfo/content/store-prefix/attestation checks. A failed attempt may
leave partial physical bytes; it does not report an admitted output. An active
client/child session is process-local and bound to the assigned job, attempt,
and fence. There is no automatic effect-loop retry: an explicit reconnect or
reassignment re-probes receiver-owned bytes before using a fenced checkpoint.
Equal-content chunks may share a digest while their artifact positions remain
distinct canonical indices.

Remote chunks, acknowledgements, checkpoints, resource-scoped attempts, and
sessions cannot introduce their own authority: their manifest, receiver demand,
lease, assignment, or concrete request must establish the consumed identity
first. The boundary and rejection fixtures are listed in
[`remote-transfer.md`](remote-transfer.md#transient-handle-admission).

See [`remote-transfer.md`](remote-transfer.md) and the checked
[`remote-build-loopback`](../examples/projects/remote-build-loopback/) workflow
for the deterministic interruption/resume and receiver-tamper rails. Those
local stdio fixtures do not prove a production P2P listener, SSH deployment,
REAPI compatibility, independent-machine behavior, exactly-once delivery,
worker honesty, or release reproducibility. Provider and cluster-control details
remain outside the core scheduler contract.

The typed `crunch-remote-core` and `crunch-remote-app` path exercised by these
fixtures covers bounded attempt, transfer, output/receipt, and selected stdio
effects. It is not completion of quantified resource/locality migration,
SSH/local/external-batch application-port cutover, or the remote hexagon
Cairn validation gates. The std adapters retain signature, frame/digest, and
physical store checks; a receipt or acknowledgement alone is not output trust.

### Live daemon policy and readiness boundary

`mantle remote live serve --socket PATH` maintains process-local live build
facts. It has no automatic supervisor: SIGINT/SIGTERM stop it, and SIGKILL
requires an operator to relaunch it. Its declared service-readiness restart
policy is therefore `never`; manual relaunch starts with empty facts and must
establish a fresh `started` observation before any `ready` observation. A
socket inode or `listening` log is not a successful request: the readiness
contract requires a subscription to that same daemon to deliver both
`snapshot-start` and `snapshot-end`. Existing live-build facts are revocable
coordination state, not a build receipt, PathInfo, or release evidence.
The opt-in `mantle doctor --readiness-socket PATH` query writes a typed
`mantle-service-readiness-v1` coordination report to stderr; ordinary doctor
human and JSON stdout are unchanged. `classification: coordination-state`
and `evidence_eligible: false` make this diagnostic, not a build receipt,
PathInfo, or release proof. A live Rust cache listener likewise declares
`never` until it gains an actual supervisor; it can become ready only after a
validated compiler request's response is flushed. The one-shot remote stdio
binding declares `never` and can become ready only after its admitted
response is flushed, not from metadata-only transport advertising.
Proposed ADR 0091 records these boundaries; real-process acceptance fixtures
must pass before using readiness for operational claims.

Run the live daemon before opting in to a service producer by setting
`MANTLE_LIVE_STATE_SOCKET` to that daemon's socket path. A separate operator
can query the same path:

```sh
mantle --json doctor --readiness-socket /run/user/1000/mantle-live.sock \
  > doctor.json 2> readiness.json
```

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

## Use an ordered read-only base stack

Complete all base writes before you admit the base.
Add an accepted Ed25519 public key, and then remove all write permissions:

```bash
printf '%s\n' '<name>:<base64-ed25519-public-key>' \
  > /srv/mantle-base/overlay-trusted-public-keys
chmod -R a-w /srv/mantle-base
```

Declare bases in read-precedence order:

```bash
mantle \
  --base-store /srv/mantle-base-a \
  --base-store /srv/mantle-base-b \
  store list

mantle \
  --base-store /srv/mantle-base-a \
  --base-store /srv/mantle-base-b \
  --json store info '<path-filter>'
```

The current state directory is the only writable overlay.
A selected base read does not backfill overlay PathInfo, directory, or blob state.
A higher invalid layer blocks lower-layer fallback.

The reports identify `overlay`, `base[1]`, `base[2]`, and later declared bases.
They also include descriptor, generation, trust-policy, signer, shadow, and no-backfill facts.

`mantle build --plan` adds `store_overlay` and `selected_store_layers` to each route report.
Each selected base entry binds its descriptor and observed generation.

A JSON build report includes `overlay_plan_blake3`, `overlay_base_generations`, and `store_layer_selections`.
These fields record the layers that the completed build read.

`mantle attest show` reports `selected_layer` in its envelope.
`mantle attest closure` reports `selected_layers` for all observed closure paths.

Run GC with the same ordered base declarations used for planning.
Execution binds the accepted plan to the overlay descriptor and current base generations.
GC never selects base-owned paths for mutation.

Remove all `--base-store` options to return to single-store operation.
This rollback does not copy or delete base content.

Overlay composition requires the default `snix` backend.
With `--store-backend casita`, any `--base-store` fails with `casita-overlay-unsupported` before Mantle reads or creates state.

## Inspect GC and final-NAR repair plans

Use dry-run commands before a store mutation:

```bash
mantle store gc --dry-run
mantle store repair-final-nar /mantle/store/<digest>-<name>
```

The GC core computes normalized reachability, retained paths, candidates,
reclaim summaries, and dry-run mutation dispositions. The repair core decides
admission, signing and sidecar dispositions, mutation order, report status, and
rollback intent. Both cores use bounded owned data and BLAKE3 plan identities.

The `crunch-store` shell collects the facts and performs all effects. It reads
services, renders NARs, signs metadata, changes files, persists records,
verifies results, and executes rollback.

A core plan is not an execution receipt. It does not prove that a mutation
started or completed. It also does not prove content correctness, signer
authority, provenance, reproducibility, release eligibility, or global cache
availability. A missing root, unresolved reference, duplicate identity,
invalid repair fact, or exceeded bound makes planning fail closed.

With `--store-backend casita`, GC execution also writes a recovery fence, and the next guarded store command finishes an interrupted execution; see [Store backends](store-backends.md#garbage-collection).
Final-NAR repair requires the `snix` backend: under `casita`, both the dry run and `--execute` fail with `casita-repair-final-nar-unsupported`.

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

Store-backed `attest show`, `closure`, `verify`, `diff`, and `project` open the
selected state store. Even an unsuccessful lookup can initialize its state
directory and store identity; closure lookup can persist a reconstructed
attestation. Do not treat these queries as guaranteed no-write operations.
Release and witness document reads do not open this store. `attest witness
create` can initialize a configured signing key when no existing key was
selected, in addition to publishing witness files. A returned attestation
command reports its bounded result, not proof of durable writes or builder,
source, or signer correctness.

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

On Linux, `release reproduce` bounds each rebuild's stdout and stderr
separately to 16 MiB, with a 24-hour child deadline, a five-second pipe-EOF
grace after leader exit, and up to five seconds to observe teardown.
Ordinary rebuild capture/supervision failures occur before the ordinary report
is written; failed-command diagnostics retain at most the first 512 bytes of
each stream. Mantle signals its owned rebuild process group with SIGKILL
before reaping only the leader, even after apparent success. This does not
guarantee cleanup of detached descendants or separately sessioned bwrap workers.

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

### Move release evidence with chaptered transport

Use the opt-in chaptered transport after the release directory passes normal
verification:

```bash
mantle release transport pack \
  target/release-evidence/<release-id> \
  --to target/release-transports/<release-id>

mantle release transport inspect \
  target/release-transports/<release-id>

mantle release transport unpack \
  target/release-transports/<release-id> \
  --to target/imported-release-evidence/<release-id>
```

The transport directory contains `release.tgz` and `receipt.json`. The source
release directory remains canonical. Both output and unpack destinations must
be absent.

Chapter zero contains the reserved transport index and canonical release
manifest. Later chapters group source members, each immediate binary child, and
other top-level artifact groups. The format is deterministic for the same
verified tree and compression version.

The receipt binds the complete compressed archive with BLAKE3. It also binds
the source manifest, transport index, archive size, format versions, chapter
count, member count, claim scope, and non-claims. The receipt needs an
authenticated parent handoff when the transport crosses a hostile boundary.

Inspect and unpack first copy `release.tgz` into a private snapshot while they
measure the receipt-bound digest. They also validate the complete bounded gzip
stream and marker count before chapter access. These checks reject payload,
marker, trailer, malformed-stream, and excessive-chapter changes.

Unpack validates every indexed tar entry before output writes. It accepts only
regular files, directories, and validated internal relative links. It writes
through capability-relative no-follow operations, runs normal release
verification, and publishes with one atomic no-replace directory rename.

Ordinary gzip and tar readers can read `release.tgz`. A standard extraction also
writes `__mantle_release_transport_index__.json`. Mantle unpack omits this
transport-only file.

A valid transport proves compressed-byte identity, index consistency, safe
materialization, and preserved bundle verification. It does not prove build or
source correctness, semantic correctness, reproducibility, deployment safety,
or universal release eligibility. See [ADR 0063](../adr/0063-keep-chaptered-release-archives-as-receipt-bound-transport.md).

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
  --profile optional-witness \
  --trusted-release-signer "$RELEASE_SIGNER_NAME" \
  --trusted-witness-identity <witness-identity>

mantle release witness-export target/release-evidence/<release-id> \
  --verification-dir target/release-verification/<release-id> \
  --request-dir target/release-witness-requests/<release-id>
```

Use `--profile optional-witness` to verify available trusted witnesses without a
witness-count requirement. Verification reports `not-required` when the policy
minimum is zero. Valid witnesses remain visible as individual evidence.

Use `--profile self-proof-only` for the compatible zero-threshold policy that
accepts no trusted witness identities. Use `--profile single-witness` for the
compatible one-witness policy.

Select a larger quorum only with explicit values:

```bash
mantle attest policy-init target/release-verification/<release-id> \
  --profile witness-quorum \
  --trusted-release-signer "$RELEASE_SIGNER_NAME" \
  --trusted-witness-identity witness-a \
  --trusted-witness-identity witness-b \
  --min-matching-witnesses 2 \
  --independence-field witness_identity
```

Supported independence fields are `witness_identity`, `signer_key_name`, and
`rebuild_environment_summary.host_class`. The command rejects missing, zero,
unsupported, or unbounded quorum values. It also rejects duplicate names and a
quorum larger than the trusted witness identity set.

`mantle attest key-show` prints the exact `name:base64` verifier token
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

`mantle attest release-verify` writes the JSON report to standard output. It
also writes the two policy-status lines to standard error. Use the global
`--json` option for JSON-only output.

The report keeps technical validity, social policy sufficiency, and independent
rebuild agreement separate. `witness_quorum_status` uses only `not-required`,
`satisfied`, or `insufficient`. A zero threshold never emits `quorum-satisfied`
as the final class. Optional invalid evidence remains visible but cannot
satisfy policy.

The independent agreement fields are `independent_agreement_status`,
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
the configured policy. An optional witness proves the same individual digest
and signature facts without a quorum claim. A satisfied independent-agreement status gives the
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
