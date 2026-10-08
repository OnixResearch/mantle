# Mantle

Mantle is a Nickel-authored, Rust-implemented build system on the Nix store
protocol. It evaluates typed derivations, schedules builds lazily, executes them
in bounded Linux sandboxes, persists output content and PathInfo metadata in the
selected store backend, and emits structured evidence for later inspection.

Mantle is a build tool, not an operating-system module layer. Frontends such as
Onix own inventory, policy, and system configuration, then lower concrete
build inputs into Mantle. See [ADR 0010](adr/0010-keep-mantle-build-tool-boundary.md).

> **Status:** active research software. Mantle has working build, store, cache,
> project, evidence, bootstrap, and self-build paths. Each proof or receipt is
> bounded to its declared inputs and policy. It does not prove compiler
> correctness, full-source bootstrap, global reproducibility, or deployment
> safety.

## Why Mantle

- **Typed configuration:** Nickel contracts reject malformed derivations before
  execution.
- **Modern identity:** Mantle-owned content identity, cache keys, and receipts
  use BLAKE3 unless an interoperability surface requires another algorithm.
- **Explicit builds:** inputs, outputs, network policy, store prefix, and
  hermeticity mode are visible rather than inferred from ambient context.
- **Independent store logic:** closure resolution uses Mantle PathInfo and
  castore services. Ordinary builds do not invoke `nix-store`.
- **Evidence-first operations:** machine-readable reports distinguish observed
  facts, policy decisions, and explicit non-claims.

## Quick start

The supported development path uses the checked-in Nix shell and Rust nightly:

```bash
nix develop
cargo build -p mantle

target/debug/mantle doctor
target/debug/mantle eval examples/hello.ncl
target/debug/mantle build --plan examples/hello.ncl
target/debug/mantle \
  --store /tmp/mantle-store \
  --state-dir /tmp/mantle-state \
  build examples/hello.ncl --no-substitute
```

To build the packaged CLI directly:

```bash
nix build
./result/bin/mantle --help
```

Derivation execution currently requires Linux, `bwrap`, and enabled user
namespaces. `mantle doctor` checks the selected workflow without starting a
build or mutating store state.

## A minimal derivation

```nickel
let mantle = import "lib.ncl" in
{
  name = "hello",
  builder = "/bin/sh",
  args = ["-c", "echo 'Hello, mantle!' > $out"],
} | mantle.Derivation
```

Evaluate it without building:

```bash
mantle eval examples/hello.ncl
```

For strict time, CPU, address-space, protocol, and teardown bounds, use an
[owned evaluation worker](docs/evaluation-resource-budgets.md):

```bash
mantle eval examples/hello.ncl \
  --budget-policy config/evaluation/default-policy.json \
  --budget-report target/evaluation-report.json
```

Preview and realize it:

```bash
mantle build --plan examples/hello.ncl
mantle build examples/hello.ncl --no-substitute
```

For an interactive local build that follows edits to the file and its imported
Nickel sources, run `mantle build --watch examples/hello.ncl --no-substitute`.
The command prints admitted, retained, and retracted goals and their settled
outputs; invalid edits leave the last admitted goals in place. Removing a
running goal cancels its sandbox and waits for teardown before admitting the
replacement set. Ctrl-C or SIGTERM also waits for active sandboxes to stop.
Watch output is human-readable, not the stable `--json` aggregate report;
`--watch` cannot be combined with remote builders, `--impure`, `--plan`,
`--causal-trace`, or `--evaluation-stream`.

Use `mantle --json build ...` for the stable aggregate build report. Its compatibility identifier is `crunch-build-report-v1`.

External CI consumers can bind normalized requests and observations through the host-independent [`mantle-build-contract`](docs/build-interchange-contract.md) component. This contract does not import build or store authority.

Use `mantle build --evaluation-stream ...` for bounded NDJSON selected-root events. Stdout contains only `mantle-evaluation-stream-v1` records. See [the evaluation stream contract](docs/evaluation-stream-contract.md) for ordering, status, cancellation, and compatibility rules.

To diagnose a local build, opt in to `mantle --state-dir "$STATE_DIR" --json build
examples/dependency-chain.ncl --causal-trace`. Mantle writes a bounded,
redacted `mantle-build-trace-v1` JSON diagnostic to
`$STATE_DIR/logs/build-trace-<pid>.json` (or a numbered `-<n>.json` suffix
when that path already exists), separate from stdout, build logs, attestations,
and receipts. The trace stays under `$STATE_DIR/logs` even when `CRUNCH_LOG_DIR`
sends ordinary build logs elsewhere. Follow `caused_by` action IDs from a
failure to its root requirement and external trigger; an interleaved
independent root is not its cause. Cache hits name their admission check.
Unknown, missing, dangling, cross-goal, oversized, or unredacted records fail
validation. Traces describe observed scheduler actions only, not a complete
build proof, output trust, or release eligibility. If strict store preflight
fails before the Worker starts, the original report and exit status remain;
stderr says the trace is unavailable and no synthetic file is written.
`--causal-trace` cannot be combined with `--evaluation-stream`, `--plan`,
`--fix`, `--watch`, or a remote `--builder`.

## Core workflows

Use the checked [canonical operator workflow](docs/generated/canonical-operator-workflow.md)
for the short daily path. Use the generated
[command reference](docs/generated/operator-command-reference.md) for support tiers,
mutation classes, network classes, exit classes, and machine-output contracts.

Clap remains authoritative for parser behavior. The typed Nickel inventory owns
reviewed operator policy. `docs/operator-workflows.md` contains longer runbooks
that supplement the checked catalog.

## Store model

Mantle separates logical identity from physical placement:

| Setting | Default | Meaning |
|---|---|---|
| `--store-prefix` | `/mantle/store` | Logical paths used in derivation identity and ATerm serialization |
| `--store` | `/nix/store` | Physical directory where final outputs are exported |
| `--state-dir` | `$CRUNCH_STATE_DIR`, `$XDG_STATE_HOME/crunch`, or `~/.local/state/crunch` | Backend state (Snix PathInfo and castore files, or the Casita repository in `casita/`), plus logs, roots, and evidence |
| `--store-backend` | `snix` | Durable backend for the state directory: `snix` or `casita` |
| `--base-store` | None | Repeatable, ordered read-only base state directory below the writable store |

Use `--store /tmp/mantle-store` for an unprivileged physical output directory.
Use `--nix-compat` when the logical prefix must be `/nix/store` for
interoperability. The legacy `CRUNCH_STATE_DIR` spelling remains a compatibility
surface. New operator prose and project-facing names use Mantle.

An empty or relative selected `CRUNCH_STATE_DIR`, `XDG_STATE_HOME`, or `HOME`
fails before state/store access. An explicit `--state-dir` takes precedence,
including relative paths such as `.`. If `HOME` is unset, the fallback remains
`/tmp/.local/state/crunch`.
Relative `--state-dir` values are anchored to the invocation's working
directory only when exported to child processes; the parent retains the
operator-selected path.

`CRUNCH_LOG_DIR` overrides `<state-dir>/logs` for build logs and `mantle log`;
an empty selected log directory is rejected before command effects. Absolute
non-UTF-8 state environment paths and non-UTF-8 log directory paths are
preserved without fallback.

Content-addressed outputs are the default. Input-addressed derivations and Nix
hash algorithms remain available where compatibility requires them. Mantle signs
PathInfo records with Ed25519 keys and rechecks signatures, content hashes, and
castore completeness before admitting cached outputs.

Before cache lookup or execution, a direct or transitive dependent of
content-addressed outputs resolves completed input paths and uses the resulting
derivation for output, cache, and shared action-result identity. Signed results
and matching signed PathInfo under trusted full public keys authorize both
content-addressed outputs and resolved input-addressed intermediates; a child
can then pass its admitted output identity to its own children. A bare
`ca_mappings.json` entry or PathInfo-only cache hit is not authority, so legacy
mapping-only results rebuild. CA-independent subgraphs retain their identities.
Build reports show original and resolved derivations, resolved identity, and
reuse decisions. This bounded reuse does not prove output determinism, executor
correctness, or release eligibility; see
[ADR 0098](adr/0098-resolve-content-addressed-inputs-before-dispatch.md).

Each state directory records its store backend.
Mantle rejects a different `--store-backend` before it changes any file.
`casita` is a pinned, pre-release backend with a smaller capability profile and a destination-owned signer policy.
See [Store backends](docs/store-backends.md) for profiles, validation, and the Snix-to-Casita migration.

### Read-only overlay composition

Use `--base-store <state-dir>` to add an ordered read-only base.
The current `--state-dir` remains the only writable overlay.
All layers must use the same logical store prefix.
Overlay composition requires the `snix` backend.

Prepare a base only after all base writes finish:

```bash
printf '%s\n' '<name>:<base64-ed25519-public-key>' \
  > /srv/mantle-base/overlay-trusted-public-keys
chmod -R a-w /srv/mantle-base
mantle --base-store /srv/mantle-base store list
mantle --base-store /srv/mantle-base --json store info '<path-filter>'
```

The base must contain `store-identity.json` and one accepted trust-key file.
Mantle also accepts a local `signing-key` when the public-key file is absent.
Do not distribute a private signing key only to enable base reads.

Mantle rejects writable members, symlinks, special files, prefix mismatches, duplicate bases, and invalid signatures.
Reads do not copy PathInfo, directories, or blobs into the overlay.
An invalid higher layer blocks fallback to lower layers.

Store and GC reports include the exact layer, shadows, descriptor, generation, trust policy, and signer names.
Route and build reports bind selected base layers to their observed descriptors and generations.
Attestation envelopes report the selected artifact layer or the selected closure layers.
Mantle revalidates base generations before execution, GC, and output admission.
GC can remove only overlay-owned state.

To roll back, remove all `--base-store` options.
The writable store then uses normal single-store behavior.
This action does not copy or delete base content.

These checks do not prove source correctness, permanent immutability, release eligibility, or whole-database atomicity.
See [ADR 0012](adr/0012-overlay-store-composition.md) for the full decision and non-claims.

Historical signed PathInfo created before final-NAR metadata fixes can be inspected
and explicitly migrated one exact path at a time:

```bash
mantle store repair-final-nar /mantle/store/<digest>-<name>
mantle store repair-final-nar /mantle/store/<digest>-<name> \
  --execute --signing-key ./cache.key
```

The first command is a non-signing dry run. Execution requires complete local
castore content, preserves CA/path/node/reference identity, discards signatures
bound to stale facts, and emits a replacement local signature. This does not
recover historical signer authority or prove output correctness.

Final-NAR repair requires the default `snix` backend.
Under `--store-backend casita`, both commands fail with `casita-repair-final-nar-unsupported`.

GC and final-NAR repair use separate pure decision cores. The cores receive
bounded, normalized facts and return ordered plans with BLAKE3 identities.
`crunch-store` still owns service reads, NAR rendering, signing, filesystem
changes, persistence, verification, and rollback execution. A plan records a
bounded decision. It does not prove content correctness, provenance,
reproducibility, release eligibility, or successful execution.

Use the explained retention workflow for store cleanup:

```bash
mantle store roots --migrate
mantle store usage
mantle store gc
mantle store gc --execute --plan-id <blake3-plan-id>
```

Legacy `gc-roots.json` remains readable without modification until the explicit
`store roots --migrate` command publishes per-interest records and archives the
legacy file. Migrated roots retain `legacy-unmanaged` provenance; migration does
not infer an owner or authorize deletion. New interests are bounded, versioned
records under `<state-dir>/retention-interests`, named by the BLAKE3 digest of
their canonical JSON bytes. For independently scoped local operator interests:

```bash
mantle store pin /mantle/store/<digest>-<name> --owner alice --reason ci
mantle store pin /mantle/store/<digest>-<name> --owner bob --reason investigation
mantle --json store roots
mantle store unpin /mantle/store/<digest>-<name> --owner alice --reason ci
```

The first release leaves Bob's record and the retained path intact. Commands
`store roots`, `store usage`, and `store info` report owner, reason, and physical
record count; their JSON reports carry explicit schema versions. `--owner` is a
local label, **not** authenticated authority: a caller with access to the
state directory can choose another operator label. `store usage` reports
observed, retained, reclaimable, quarantined, shared, and unknown bytes.
Unknown closure or size facts stay visible instead of becoming zero-byte
estimates.

The first GC command only creates a plan. Execution replans from current facts
and rejects a stale plan ID. It commits authoritative metadata before file
cleanup. It then continues across independent safe cleanup attempts and reports
candidate paths, completed operation classes, and observed failures.

Build, lookup, root, source, action-result, and administrative store authority
use separate Rust capabilities. See
[`docs/store-authority-capabilities.md`](docs/store-authority-capabilities.md).

## Project workflow

A Mantle project uses `mantle-project.ncl`, `mantle.lock`, and generated
`.mantle/` state:

```bash
mantle init
mantle check
mantle refresh
mantle list-stale
mantle build
mantle run .#tool -- --help
```

Mantle's project layer handles declared source inputs, lock state, generated
Nickel bindings, package selectors, shell profiles, and reviewed file generation.
It does not own Onix-style module evaluation or system configuration.

Project commands plan their filesystem and resolution capabilities before
execution. `refresh` and `upgrade` observe lock, generated-input, and retention
writes separately; a failed later write can leave earlier files changed, and a
partial refresh still exits nonzero even when successful inputs were locked.
When a write fails, `refresh` does not render its outcome listing; inspect the
files before retrying. `init` checks each scaffold write and reads its content
back before announcing success. A project check reports static soundness, not
build success, remote freshness, or trust proof.
Refresh and stale checks reject more than 256 inputs to resolve before invoking
the resolver; this is its existing batch limit, not proof of network completion.

For reviewed generated files, run `mantle filegen plan --plan-out plan.json`,
review the plan, then `mantle filegen apply --plan plan.json`. The plan
publication is an observed write, not an application of generated files.
Apply reports success only after the state file and every managed generated
file read back consistently with the applied plan; a failed write can leave
partial files, so re-plan before retrying.

An unsupported reviewed-plan schema returns a typed blocker before generated-file
or filegen-state writes; valid reviewed plans retain drift-checked apply behavior.

`mantle log --list` fails rather than presenting an incomplete listing when a
selected log cannot be read. Log inspection admits at most 4,096 directory
entries and 16 MiB of log content per invocation; exceeding either bound fails
without printing a partial listing.

## Offline source workflow

Prepare a source bundle on a connected host, then import and pin it before an
offline build:

```bash
mantle source bundle export \
  --build-root ./package.ncl \
  --import-path lib \
  --to source-bundle.json
mantle source bundle verify --from source-bundle.json
mantle --state-dir ./offline-state source bundle import \
  --from source-bundle.json --pin
mantle --state-dir ./offline-state source bundle verify \
  --from source-bundle.json --imported
mantle --state-dir ./offline-state source bundle preflight \
  --build-root ./package.ncl --import-path lib
mantle --state-dir ./offline-state build \
  --offline-source-preflight --no-substitute ./package.ncl
```

For pre-existing `/nix/store` inputs that cannot be embedded in a source bundle,
import each direct physical output with its signed PathInfo and an explicit
public key. Use the same store prefix, state directory, output store, and key
when checking or building the root:

```bash
mantle --nix-compat --store /srv/offline-store --state-dir /srv/offline-state \
  source bundle import-store-path \
  --logical /nix/store/<hash>-name \
  --physical /srv/offline-store/<hash>-name \
  --trusted-public-keys 'cache:PUBLIC_KEY' --pin
mantle --nix-compat --store /srv/offline-store --state-dir /srv/offline-state \
  source bundle preflight --build-root ./package.ncl \
  --trusted-public-keys 'cache:PUBLIC_KEY'
mantle --nix-compat --store /srv/offline-store --state-dir /srv/offline-state \
  build --offline-source-preflight --trusted-public-keys 'cache:PUBLIC_KEY' \
  --no-substitute ./package.ncl
```

Virtual-only pinned store paths are not ready. Import verifies the signed
PathInfo identity, references and actual NAR; strict preflight rechecks those
physical bytes before admitting the build.

Source readiness proves declared input availability and identity only. Build
success, output trust, compiler correctness, and release eligibility require
separate evidence.

Source commands admit bounded read, optional connected-fetch, publication, and
state-observation effects before executing them. Export, bootstrap-profile
publication, and profile refresh read back their published bundle; import and
hydration check durable records and pins, and hydration rechecks the published
vendor tree. A successful write alone is not a completed source operation.
These observations cover only the named artifacts and declared limits; they do
not establish source authenticity beyond admitted metadata or any downstream
build result.

## Mantlepkgs catalogs

Mantlepkgs generates a bounded package catalog from locked, concrete Nixpkgs
derivation graphs. Nix runs only during explicit catalog production. Later
verification, selection, planning, and building use published artifacts without
Nix.

The producer can resolve reported package versions to exact historical revisions before catalog generation. See the [version-resolution guide](mantlepkgs/versions/README.md).

```bash
mantle mantlepkgs validate --manifest mantlepkgs/live-cohort/manifest.ncl
mantle mantlepkgs verify --generation <generation-directory>
mantle mantlepkgs plan \
  --generation <generation-directory> \
  --package hello --system x86_64-linux \
  --plan-out target/hello.plan.json \
  --import-receipt-out target/hello.import-receipt.json
```

See the [Mantlepkgs guide](mantlepkgs/README.md) for production, source
preparation, no-Nix consumption, package dispositions, and claim boundaries.

## Examples

Start with the supported [examples guide](examples/README.md). Its catalog is the
source of truth for prerequisites, support tiers, and validation rails.

| Example | Purpose |
|---|---|
| [`examples/hello.ncl`](examples/hello.ncl) | Smallest local derivation |
| [`examples/multiple-roots.ncl`](examples/multiple-roots.ncl) | Independent roots and lazy scheduling |
| [`examples/dependency-chain.ncl`](examples/dependency-chain.ncl) | Producer/consumer ordering |
| [`examples/projects/generated-site/mantle-project.ncl`](examples/projects/generated-site/mantle-project.ncl) | Project selectors and checks |
| [`examples/projects/offline-source-bundle/mantle-project.ncl`](examples/projects/offline-source-bundle/mantle-project.ncl) | Connected-to-offline source handoff |
| [`examples/projects/nixpkgs-tool-use/workflow.ncl`](examples/projects/nixpkgs-tool-use/workflow.ncl) | Cache-imported Nixpkgs tool used by a Mantle build |

Advanced cache, imported-tool, remote-build, OCI, release, WebAssembly, bootstrap,
and benchmark examples are indexed in [`examples/README.md`](examples/README.md).

## Architecture

```text
Nickel source
    │
    ▼
crunch-eval       evaluate and deserialize typed values
    │
    ▼
crunch-glue       lower values into derivations and store identities
    │
    ▼
crunch-pipeline   connect evaluation, conversion, and realization
    │
    ▼
crunch-build      schedule goals, substitute, fetch, or sandbox builds
    │
    ▼
crunch-store      persist PathInfo, output content, roots, and attestations
```

The `crunch-*` crate names are retained compatibility identifiers. New
project-facing prose uses Mantle. Pure decision logic is split into functional
core crates where adopted. Filesystem, process, network, and CLI effects remain
in thin Rust adapters.

The `mantle-plan-v1` boundary decodes structural wire records before it admits
checked IDs, output names, logical store paths, and role-specific BLAKE3 values.
See [Nominal dynamic-plan types](docs/nominal-dynamic-plan-types.md) and
[Nominal trust boundaries](docs/nominal-trust-boundaries.md).

The scheduler creates goals lazily, deduplicates them by store identity, and
dispatches eligible work under explicit concurrency and policy bounds. A build
report describes what was observed. It does not turn scheduling, sandbox, or
cache evidence into a whole-system correctness proof.

[Native dynamic derivation admission](docs/dynamic-derivation-admission.md) requires
complete parent BLAKE3 facts before registry or scheduler mutation. It rejects
missing parents instead of using the former all-zero fallback.

## Evidence and trust boundaries

Mantle keeps build observations separate from stronger claims:

- Build reports describe selected actions, outputs, failures, policy events,
  cache decisions, and evidence sidecars.
- Attestations bind canonical facts and identities. They do not prove that a
  builder, compiler, source, or dependency was correct.
- Release verification checks the selected bundle and policy. Consumers must
  require a successful exit status and the report's final accepted disposition.
- Optional build witnesses preserve valid individual evidence without requiring
  quorum. Quorum applies only under an explicit positive-threshold policy.
  Witnesses do not prove source review, compiler correctness, or broad
  reproducibility.
- [Content-bound requirement evidence](docs/content-bound-requirement-evidence.md)
  links selected Cairn and Valence identities to exact release evidence bytes.
- Foreign import receipts bind the admitted graph and policy, not foreign
  frontend correctness, realization success, or output trust.
- Foreign realization receipts bind observed execution and store facts.
  Cache-only plans can preserve exact Nix paths after trusted closure hydration.
  They do not prove local rebuild compatibility, package correctness, evaluator parity, or reproducibility.
- [Nario v2 read compatibility](docs/nario-v2-import.md) lists and imports exact
  Determinate Nix store records. It does not import recipes or Nix evaluation meaning.
- [Experimental composition roots](docs/composition-roots.md) merge exact castore
  directory roots with canonical BLAKE3 plan identity and explicit conflict decisions.
  They do not select packages, execute, activate, deploy, or prove release eligibility.
- [Evaluation resource budgets](docs/evaluation-resource-budgets.md) run strict Nickel
  evaluation in an owned worker with bounded protocol and teardown facts. Budget
  compliance does not prove evaluator correctness, reproducibility, or release eligibility.
- Foreign provenance audits scan signed castore facts under explicit limits.
  The scanner handles bounded gzip and zstd streams and normalizes safe store
  suffixes. Unknown executable bytes still fail closed.
  These audits do not prove dynamic behavior, package correctness, or release eligibility.
- Bootstrap and fixed-point evidence applies only to the recorded seed,
  source, tools, platform, and proof mode.
- AeneasVerif proof and translation policy remains owned by Octet. Mantle can
  build tools and generated artifacts, but it does not promote their claims.
  See [ADR 0034](adr/0034-keep-aeneasverif-proof-and-translation-authority-in-octet.md).

The representative Rust compatibility rail uses
[`examples/rust_compatibility_surface_matrix.ncl`](examples/rust_compatibility_surface_matrix.ncl)
as its surface matrix. It records supported sandboxed offline Cargo
(`cargo-inside-mantle-sandbox`) and native `cargo-free-bounded-topology` lanes,
alongside blocked surfaces and `blocked-unsupported-surface` receipts. This is
not proof of full Cargo compatibility, compiler correctness, release
reproducibility, or bootstrap correctness.

Native Rust unit caching is disabled by default. Use
`--local-rust-cache read` or `--local-rust-cache read-write` with an explicit
`rust-plan` execution mode. Add `--shared-rust-cache read` or `read-write` for
signed directory or HTTP exchange. Mantle keeps existing output and local
castore reuse ahead of shared transfer. It admits a shared result only after
full-key authority, policy, object, complete-tree, artifact, and materialization
checks. See [`docs/native-rust-plan-validation.md`](docs/native-rust-plan-validation.md),
[`docs/shared-rust-unit-cache.md`](docs/shared-rust-unit-cache.md), and the
[Rust compiler cache daemon guide](rust-cache/daemon/README.md).

Foreign derivation admission is documented in
[`docs/foreign-derivation-import-trust-model.md`](docs/foreign-derivation-import-trust-model.md).
The [Nix derivation compatibility guide](docs/nix-derivation-compatibility-boundary.md)
defines the reviewed parser, package identity, limits, hash domains, and rollback.
The [foreign realization operator guide](docs/foreign-realization-operator-guide.md)
explains source preparation, local realization, provenance audits, receipts,
and cache hydration. The [Nario v2 guide](docs/nario-v2-import.md) defines the
pinned read-compatibility and source-projection boundary.
Run the positive and negative trust-model guards with:

```bash
nix develop -c cargo -Zscript scripts/check-foreign-import-trust-model.rs
nix develop -c cargo -Zscript scripts/check-foreign-import-trust-model.rs --self-test
```

## Bootstrap and proof workflows

Use [`docs/operator-proof-guide.md`](docs/operator-proof-guide.md) before making
or reviewing self-build, Cargo-free, Nix-free, or release claims.

### Bootstrap proof methodology

Mantle uses a layered, fail-closed evidence argument. A successful build does
not prove a bootstrap claim.

```text
accepted claim =
  authenticated inputs
  + authorized construction
  + bound outputs
  + bounded behavior
  + negative evidence
  + independent validation
  + reproducible fixed point
```

Each proof starts with a claim contract. The contract names the required
sources, predecessors, outputs, behaviors, rejection cases, forbidden
fallbacks, and non-claims. The parity system derives status from current
receipts instead of a manually selected completion value.

Proof construction uses strict state when the claim requires it. Strict state
disables substitutions, rejects unsigned artifacts, removes ambient tools, and
disables live source acquisition. Each stage must use its declared immediate
predecessor.

StageX adds execution authority. Its seccomp supervisor intercepts `execve` and
`execveat` before a child starts. The policy binds an absolute path, a BLAKE3
executable identity, an allowed stage, and an allowed child relationship.
Undeclared execution and identity mismatches fail closed. A complete stage report
requires each declared path and digest in the intercepted audit. This proves an
execution decision, not successful process behavior.

A complete protected transition can publish one bounded intermediate provider:

```bash
mantle bootstrap \
  --stagex-lineage /absolute/path/stagex-transition-lineage.json \
  --stagex-transition-root /absolute/path/complete-transition \
  --output /absolute/path/absent-provider-directory
```

Mantle requires all three paths to be explicit and absolute. The output must
not exist. The provider contains self-hosted TinyCC, native-musl headers and
static libraries, target-prefixed binutils, metadata, validation evidence, and
a complete receipt. This provider does not admit final GCC or prove compiler
correctness.

Receipts are evidence, not authority. The validators recompute canonical
identities from current source and artifact files. They reject missing, stale,
substituted, or malformed evidence. Positive tests demonstrate bounded
behavior. Negative and mutation tests demonstrate fail-closed behavior.

The final reproducibility argument requires exact BLAKE3 identity for stage1
and stage2 Mantle binaries. Its receipt must also record zero live fetches,
substitutions, fallback events, prebuilt Rust inputs, Cargo oracle use, and
ambient compiler discovery.

| Question | Required evidence |
|---|---|
| Which source bytes entered the build? | Authenticated source records and an offline source bundle |
| Which tools ran? | A protected execution audit |
| Which predecessor built the output? | Row-local receipts |
| Which output bytes resulted? | Canonical artifact identities |
| Does the output perform its bounded role? | Positive runtime tests |
| Does modified evidence fail closed? | Negative and mutation tests |
| Can Mantle reproduce itself? | An exact stage1-to-stage2 fixed point |
| Can the claim be published? | Bootstrap parity and Cairn gates |

This methodology proves bounded bootstrap facts for the recorded seed, source,
tools, platform, and policy. It does not prove compiler correctness, semantic
correctness, kernel correctness, or universal reproducibility.

Self-hosting preflight and proof modes:

```bash
./scripts/prove-self-hosting.sh --check
./scripts/prove-self-hosting.sh
./scripts/prove-self-hosting.sh --non-nix-host
./scripts/prove-self-hosting.sh --no-host-tools --stage0-inventory <file>
```

`--check` is prerequisite validation, not proof evidence. Successful full runs
write bounded evidence under `target/self-hosting-proof/`.

Cargo-free topology paths:

```bash
mantle self-build --cargo-free --out /tmp/mantle-cargo-free
mantle self-build --cargo-free --fixed-point --strict-hermetic --out /tmp/mantle-cargo-free
```

Nix-free demo bundle paths:

```bash
mantle --json nix-free-demo validate <summary.json>
mantle nix-free-demo readme <summary.json>
mantle nix-free-demo generate --out <dir> --proof-status <status>
```

Validate proof-guide drift with:

```bash
nix develop -c cargo -Zscript scripts/check-operator-proof-guide.rs
nix develop -c cargo -Zscript scripts/check-operator-proof-guide.rs --self-test
```

These workflows intentionally preserve non-claims. A fixed point does not by
itself establish seed trust, compiler correctness, independent reproducibility,
or global release eligibility.

## Development

Development builds use a dedicated store on the datapool NVMe disk. The `store`
symlink at the repository root points to the ZFS dataset `datapool/mantle-store`
(mounted at `/datapool/mantle-store`). Pass `--store store --state-dir store/state`
to `mantle build` to keep build outputs on that disk. The symlink is gitignored.

Use the repository-owned quality wrappers from the dev shell:

```bash
./scripts/check-first-party-quality.sh
./scripts/check-first-party-tigerstyle.sh
cargo test -p mantle --test examples_inventory
cargo test -p mantle --test rust_compatibility_rail
```

The ordinary first-party gate excludes vendored workspace-member test binaries
whose focused rails require additional host capabilities. Heavy self-hosting,
release-determinism, remote-build, and sandbox integration rails remain explicit
operator actions rather than every-edit checks.

Useful documentation:

- [Operator workflows](docs/operator-workflows.md)
- [Operator proof guide](docs/operator-proof-guide.md)
- [Machine artifact contracts](docs/machine-artifact-contracts.md)
- [Build correctness primitives](docs/build-correctness-primitives.md)
- [Remote credential operations](docs/remote-credentials.md)
- [Portable remote client](docs/portable-remote-client.md)
- [Mantle naming rules](docs/mantle-naming.md)
- [Durable file publication adoption](docs/durable-file-publication-adoption.md)
- [Immutable release objects and the current pointer](docs/immutable-release-current-pointer.md)
- [Filesystem and castore NAR boundary](docs/nix-archive-nar-boundary.md)
- [Store backends](docs/store-backends.md)

## Requirements

Compiling Mantle uses the checked-in Rust nightly and requires the linker and
OpenSSL development environment supplied by `nix develop`. Running derivation
builds requires Linux and `bwrap`. Nix is not required for ordinary closure
resolution or builds. The non-fetch `mantle bootstrap` path is the main
Nix-backed compatibility workflow.

## License

Repository-owned Mantle source is licensed under
[AGPL-3.0-or-later](LICENSE). Vendored code retains its upstream licenses and
notices.

## References

- [NixOS/nix-eval-jobs](https://github.com/NixOS/nix-eval-jobs) provides the reviewed independent-job and JSON-lines behavior reference. Mantle retains root, identity, wire, execution, and evidence authority.
- [picolibc/picolibc](https://github.com/picolibc/picolibc) supplies the pinned x86_64 Linux static diagnostic source for the StageX libc comparison research path.
- [OnixResearch/octet](https://github.com/OnixResearch/octet) owns checked Rust proof and translation execution policy.
- [OnixResearch/valence](https://github.com/OnixResearch/valence) owns canonical evidence identities, links, roles, and non-claims.
- [AeneasVerif/aeneas](https://github.com/AeneasVerif/aeneas) and [AeneasVerif/charon](https://github.com/AeneasVerif/charon) provide the current Rust-to-proof-model toolchain reference.
- [AeneasVerif/eurydice](https://github.com/AeneasVerif/eurydice) and [AeneasVerif/scylla](https://github.com/AeneasVerif/scylla) provide deferred code-generation and migration references.
- [fzakaria/guix-transfer](https://github.com/fzakaria/guix-transfer) provides MIT-licensed ATerm parsing, graph translation, and path-mapping design references. Mantle retains execution and evidence authority.
- [fzakaria/guixpkgs](https://github.com/fzakaria/guixpkgs) provides the checked-in translated Guix package graph used by the live GuixPkgs export proof. Mantle trusts the proof exporter's separate cache key.
- [adeci/guix-by-nix](https://github.com/adeci/guix-by-nix) provides a system-level reference for consuming translated Guix packages without Guix in the target environment.
- [dtolnay/chapter-tgz](https://github.com/dtolnay/chapter-tgz) provides the pinned chapter-marker encoding used by opt-in release transport. Mantle retains digest, index, extraction, and release-verification authority.
- [OnixResearch/trellis](https://github.com/OnixResearch/trellis) provides reusable verified logic and proof evidence for selected bounded models. Mantle retains runtime, adapter, and release authority.
- [Atom Reforged](https://nrd.sh/blog/atom-reforged.html) provides architecture references for narrow store authority, source observations, monotonic ingest, and small formal protocol models. Mantle does not adopt its package registry or ownership protocol.
- `durable-file-publication` at `rad:z3tAR4For7qw8ZirkJzoDw1VNDDLM` provides the reviewed capability-relative one-file publication mechanism.
- [komora-io/fault-injection](https://github.com/komora-io/fault-injection) supplies pinned, test-only annotated I/O faults for Mantle's store-shell fixtures. The fixtures cover named failure paths only; they do not establish sandbox hermeticity, store correctness, or release eligibility.
- [ekala-project/corepkgs](https://github.com/ekala-project/corepkgs) provides package-domain, explicit-variant, deterministic-index, and separate-test design references. Mantle retains package identity, build, validation, and evidence authority.
- [ekala-project/eka-ci](https://github.com/ekala-project/eka-ci) provides base-to-head package-impact and closure-diff design references. Forge credentials and CI presentation remain outside Mantle.
- [ekala-project/ekapkgs-update](https://github.com/ekala-project/ekapkgs-update) provides source-adapter, version-policy, OSV, and Repology design references. Mantle preserves explicit unavailable states and reimplements policy in its functional core.
- [fzakaria/stage0-bazel](https://github.com/fzakaria/stage0-bazel) provides an MIT-licensed root action-audit and trust-report design reference. Mantle retains BLAKE3 identity, producer-linked authority, seccomp enforcement, and evidence authority.
- [cachix/nix-archive](https://github.com/cachix/nix-archive) provides reviewed byte-safe NAR encoding, hashing, decoding, and restoration APIs. Mantle retains castore, PathInfo, trust, transport, publication, and release authority.
- Celld provides the reviewed immutable-release and atomic-current-pointer layout reference. Mantle does not claim installer parity or implementation equivalence.
- [fzakaria/nixpkgs-multiverse](https://github.com/fzakaria/nixpkgs-multiverse) provides the compact historical Nixpkgs revision-index reference. Mantle retains producer, source-admission, package-identity, and evidence authority.
- [cachix/nix-derivation](https://github.com/cachix/nix-derivation) provides the reviewed Nix 2.34 derivation parsing, validation, serialization, and store-path compatibility candidate. Mantle retains native BLAKE3, configurable-prefix, build, store, evidence, and release authority.
- `bounded-tree` at `rad:zqhtZvsteJhxCJE96dMAZSZ9y1PX`, revision `b0fd0103bc9eed2c1b6d852045959462d105d8f1`, provides product-neutral bounded tree planning, capability-relative observation, revalidation, and copy mechanics. Mantle retains product identity, evidence, publication, and release authority.
- [nix-community/robotnix](https://github.com/nix-community/robotnix) provides the Nix-family architectural reference for Android builds: pinned prebuilt SDK admission, explicit signing keys, and `repo2nix` source materialization. Mantle stays at app-level APK scope and retains admission, identity, and evidence authority.
- [nix-community/nix-on-droid](https://github.com/nix-community/nix-on-droid) provides the host-platform boundary reference: Nix on Android hosts runs through proot without user namespaces. Mantle's bwrap sandbox requires a Linux host, so on-device operation stays out of scope.
- `transactional-reconciliation-core` at `rad:z4Tky6zvC8w4Y6c4YBzNxVbq5n752`, revision `606489b5f40298181214bb76bc3457b607f225d9`, provides immutable planning, exact reservation admission, and unknown-outcome classification. Mantle retains GC semantics, store mutation, effect authority, and evidence.
- [The Synit Manual](https://synit.org/book/) (CC BY 4.0, © 2021–2023 Tony Garnock-Jones) provides the reviewed design reference for assertion-and-retraction coordination, dataspace interest routing, capability attenuation with pattern caveats, gatekeeper-style name upgrade, and declared service state. Mantle retains scheduling, store authority, evidence, and release authority. See [Synit application notes](docs/synit-application-notes.md).
