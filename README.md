# Mantle

Mantle is a Nickel-authored, Rust-implemented build system on the Nix store
protocol. It evaluates typed derivations, schedules builds lazily, executes them
in bounded Linux sandboxes, stores content in its own castore and PathInfo state,
and emits structured evidence for later inspection.

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

Preview and realize it:

```bash
mantle build --plan examples/hello.ncl
mantle build examples/hello.ncl --no-substitute
```

Use `mantle --json build ...` for the stable, machine-readable build report.
The current compatibility identifier for that report is
`crunch-build-report-v1`.

## Core workflows

| Workflow | Entry point |
|---|---|
| Evaluate, plan, and build | `mantle eval`, `mantle build --plan`, `mantle build` |
| Develop and execute packages | `mantle shell`, `mantle run` |
| Manage declared project inputs | `mantle init`, `mantle check`, `mantle refresh`, `mantle upgrade` |
| Inspect and move local state | `mantle store`, `mantle source bundle`, `mantle receipt bundle` |
| Inspect evidence | `mantle graph`, `mantle why`, `mantle attest`, `mantle release` |
| Use specialized boundaries | `mantle remote`, `mantle artifact`, `mantle wasm-component`, `mantle foreign-import` |
| Exercise verification lanes | `mantle rust-plan`, `mantle bootstrap`, `mantle self-build`, `mantle nix-free-demo` |

Run `mantle <command> --help` for the authoritative options. Common operator
loops and output contracts are documented in
[`docs/operator-workflows.md`](docs/operator-workflows.md).

## Store model

Mantle separates logical identity from physical placement:

| Setting | Default | Meaning |
|---|---|---|
| `--store-prefix` | `/mantle/store` | Logical paths used in derivation identity and ATerm serialization |
| `--store` | `/nix/store` | Physical directory where final outputs are exported |
| `--state-dir` | `$CRUNCH_STATE_DIR`, `$XDG_STATE_HOME/crunch`, or `~/.local/state/crunch` | PathInfo, castore blobs, logs, roots, and evidence |

Use `--store /tmp/mantle-store` for an unprivileged physical output directory.
Use `--nix-compat` when the logical prefix must be `/nix/store` for
interoperability. The legacy `CRUNCH_STATE_DIR` spelling remains a compatibility
surface. New operator prose and project-facing names use Mantle.

Content-addressed outputs are the default. Input-addressed derivations and Nix
hash algorithms remain available where compatibility requires them. Mantle signs
PathInfo records with Ed25519 keys and rechecks signatures, content hashes, and
castore completeness before admitting cached outputs.

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

Source readiness proves declared input availability and identity only. Build
success, output trust, compiler correctness, and release eligibility require
separate evidence.

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

Advanced cache, remote-build, OCI, release, WebAssembly, bootstrap, and benchmark
examples are indexed in [`examples/README.md`](examples/README.md).

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
crunch-store      persist PathInfo, castore data, roots, and attestations
```

The `crunch-*` crate names are retained compatibility identifiers. New
project-facing prose uses Mantle. Pure decision logic is split into functional
core crates where adopted. Filesystem, process, network, and CLI effects remain
in thin Rust adapters.

The `mantle-plan-v1` boundary decodes structural wire records before it admits
checked IDs, output names, logical store paths, and role-specific BLAKE3 values.
See [Nominal dynamic-plan types](docs/nominal-dynamic-plan-types.md).

The scheduler creates goals lazily, deduplicates them by store identity, and
dispatches eligible work under explicit concurrency and policy bounds. A build
report describes what was observed. It does not turn scheduling, sandbox, or
cache evidence into a whole-system correctness proof.

## Evidence and trust boundaries

Mantle keeps build observations separate from stronger claims:

- Build reports describe selected actions, outputs, failures, policy events,
  cache decisions, and evidence sidecars.
- Attestations bind canonical facts and identities. They do not prove that a
  builder, compiler, source, or dependency was correct.
- Release verification checks the selected bundle and policy. Consumers must
  require a successful exit status and the report's final accepted disposition.
- [Content-bound requirement evidence](docs/content-bound-requirement-evidence.md)
  links selected Cairn and Valence identities to exact release evidence bytes.
- Foreign import receipts bind the admitted graph and policy, not foreign
  frontend correctness, realization success, or output trust.
- Foreign realization receipts bind observed local execution and store facts.
  They do not prove package correctness, evaluator parity, or reproducibility.
- Foreign provenance audits scan signed castore facts under explicit limits.
  They do not prove dynamic behavior, package correctness, or release eligibility.
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

Foreign derivation admission is documented in
[`docs/foreign-derivation-import-trust-model.md`](docs/foreign-derivation-import-trust-model.md).
The [foreign realization operator guide](docs/foreign-realization-operator-guide.md)
explains source preparation, local realization, provenance audits, receipts,
and cache hydration.
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
- [Mantle naming rules](docs/mantle-naming.md)

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

- [OnixResearch/octet](https://github.com/OnixResearch/octet) owns checked Rust proof and translation execution policy.
- [OnixResearch/valence](https://github.com/OnixResearch/valence) owns canonical evidence identities, links, roles, and non-claims.
- [AeneasVerif/aeneas](https://github.com/AeneasVerif/aeneas) and [AeneasVerif/charon](https://github.com/AeneasVerif/charon) provide the current Rust-to-proof-model toolchain reference.
- [AeneasVerif/eurydice](https://github.com/AeneasVerif/eurydice) and [AeneasVerif/scylla](https://github.com/AeneasVerif/scylla) provide deferred code-generation and migration references.
- [fzakaria/guix-transfer](https://github.com/fzakaria/guix-transfer) provides MIT-licensed ATerm parsing, graph translation, and path-mapping design references. Mantle retains execution and evidence authority.
