# Design: Build Cargo units as dynamic-plan derivations

## Goal and scope

Build Cargo projects as one sandboxed derivation per Cargo compilation unit. A
producer plans the graph with Cargo and the Rust-planning core, and the worker
executes the lowered units through the generic dynamic-plan path. This change
covers compilation units without build-script execution.

## Current behavior

- Default lane: `mantle.offlineCargoPackage` lowers a package to one
  input-addressed derivation that runs Cargo offline in the sandbox
  (`lib/offline_cargo.ncl:87-227`).
- Verification lane: `mantle rust-plan` records Cargo-oracle evidence or
  bounded native `--no-cargo-oracle` topology evidence and can execute
  supported units in its separate host-side verification workflow. Neither
  receipt makes it the default project build lane.
- The Rust-planning core and application cutover are implemented:
  `mantle-rust-plan-core` accepts explicit existing-unit facts and returns
  ordered unit effects. This lane's adapter lowers those effects; it does not
  reintroduce planning into `src/rust_plan.rs`.
- Dynamic plans: this producer writes `mantle-plan-v2` with source slices.
  Builders must be store paths, argument and environment values may not
  contain absolute non-store paths, and the generic plan caps both source
  slices and direct unit inputs at 256. The measured full Mantle workspace
  has 711 package sources and cannot pass the source-slice cap.
- Sandbox inputs include the reference closure of every direct input.

## Approach review

| Family | Mechanism | Disposition | Required check |
| --- | --- | --- | --- |
| Offline Cargo plus wrapper cache | Keep one derivation per package and cache rustc calls through the ADR 0035 wrapper daemon | Complementary: a performance device without derivation authority or per-unit remote dispatch | None |
| Host-executed rust-plan | Promote `rust-plan` topology execution to the project lane | Rejected: no sandbox and no PathInfo authority | None |
| `cargo-dyndrv` port | Emit Nix `.drv` files through a builder store socket | Rejected: ADR 0011 keeps `.drv` discovery as compatibility only and keeps store protocols out of sandboxes | None |
| Native planner only | Use `--no-cargo-oracle` as the sole producer oracle | Not selected for this lane: native path-workspace topology has its own bounded verification class, not the complete Cargo unit graph required here | Matrix rows |
| Cargo-planned dynamic plan | The producer runs Cargo planning and lowers units to a dynamic plan | Selected | Lowering goldens and the work-reduction bundle |
| [figsoda/drowse at `3ec05474eb82f7d6b482b66298759b41d9603d35`](https://github.com/figsoda/drowse/tree/3ec05474eb82f7d6b482b66298759b41d9603d35) (MPL-2.0) | Its [`tests/default.nix`](https://github.com/figsoda/drowse/blob/3ec05474eb82f7d6b482b66298759b41d9603d35/tests/default.nix) compares `crate2nix` and `crate2nixDynamic` output contents | Adopt only the matched-output test idea for Mantle's supported default-feature app; reject its `nix-instantiate`, `.drv` output, and `recursive-nix` runtime dependency | Compare actual app stdout, stderr, and exit against the default offline Cargo lane; do not assert raw binary equality |

## Contract and component ownership

- **Rust-planning core** (`mantle-rust-plan-core`): unit topology, identities,
  and ordered unit effects over admitted facts, as delivered by
  `separate-rust-plan-hexagon`.
- **Pure lowering adapter**: maps unit effects plus a binding table from
  logical identities (package sources, toolchain, helper, and dependency
  units) to plan references. It performs no I/O and returns typed blockers for
  unsupported units and limits.
- **Producer shell**: a sandboxed derivation that loads workspace facts,
  captures the Cargo oracle offline, runs the core and the lowering adapter,
  copies package source roots into its `sources` output, and writes `plan` and
  an evidence record.
- **Unit helper**: a static Mantle-built executable that runs inside each
  unit. It reads output paths from the environment, writes rustc arguments to
  an argument file, runs rustc, and writes the dependency manifest.
- **Worker and store**: unchanged generic dynamic-plan admission, source-slice
  admission, scheduling, sandboxing, and caching.

## Decisions

### Decision: Plan with Cargo, execute with the worker

**Choice:** The producer uses Cargo metadata and the Cargo unit graph through
the existing oracle capture. It records the native planner's comparison when
the native fragment covers the workspace.

**Rationale:** The unit graph already covers features, targets, proc macros,
and profiles. The evidence class states that Cargo planned the graph, so the
lane makes no Cargo-free claim.

### Decision: Lower core unit effects in an application adapter

**Choice:** Lowering is an adapter over the core's explicit unit effects, not
new logic in `src/rust_plan.rs`.

**Rationale:** The hexagon change supplies explicit ordered unit effects.
Emitting a plan unit is another way to execute an effect beside the existing
verification workflow.

### Decision: Input-addressed units first

**Choice:** Every lowered unit is input-addressed.

**Rationale:** Unit-output placeholders require static paths, and
content-addressed identity gives no early cutoff until
`resolve-content-addressed-inputs-before-dispatch` lands. Input-addressed
units still reuse every unchanged unit, because an identical plan yields
identical derivations.

### Decision: Direct dependencies plus library manifests

**Choice:** A unit declares only its direct dependency units as inputs. Each
library unit writes a manifest that lists the output paths of its own direct
dependencies, so the reference scanner records them and every consumer's
sandbox closure contains the transitive set. The helper walks the manifests
and passes one `-L dependency=` path per closure member through the argument
file.

**Rationale:** Declaring transitive dependencies would exceed the 256-input
limit for large binaries, and the sandbox already mounts reference closures.

### Decision: Unit identity comes from rust-plan

**Choice:** `-C metadata` and output names use the unit identity that rust-plan
already computes (`rustc_metadata_hash`, `src/rust_plan.rs:746`).

**Rationale:** Symbol names and output bytes must be stable across hosts and
Rust releases. The reference derives metadata from
`std::hash::DefaultHasher`, whose algorithm is not stable across releases.

### Decision: Remap source store paths

**Choice:** Every unit passes `--remap-path-prefix` from its own package
source-slice store path to a stable package label.

**Rationale:** Embedded source paths would let the reference scanner pull
source trees into every binary's runtime closure and would tie output bytes to
store paths.

### Decision: One static unit helper as the builder

**Choice:** Units run a single Mantle-built static helper. It reads `$out` and
other output variables from the environment instead of receiving them in
arguments.

**Rationale:** Plan argument validation rejects absolute non-store strings, so
own-output placeholders cannot appear in arguments. One helper keeps the tool
closure small and gives one place to validate inputs.

### Decision: Opt-in lane

**Choice:** The lane has its own Nickel entry point and evidence class, with
project build status `opt-in-project-build-lane`.

**Rationale:** The default lane changes only after the compatibility matrix
shows parity, through a separate change.

## Failure behavior and ordering

Stable blockers are `unit-plan-build-script-run-unsupported`,
`unit-plan-mode-unsupported`, `unit-plan-source-unsupported`,
`unit-plan-triple-unsupported`, `unit-plan-host-path`, `unit-plan-limit`, and
`unit-plan-oracle-failure`. A blocked producer writes its evidence record with
blockers and no plan, and the build fails; it never runs Cargo compilation as a
fallback. Units are emitted in topological order with ties broken by unit id,
and the plan core canonicalizes the result.

## Tests

- Lowering goldens: library, binary, and proc-macro units; dependency edges;
  placeholders; remapping; metadata; an identical plan digest for equivalent
  inputs supplied in different orders.
- Negative lowering: a build-script execution unit, an unsupported mode, a git
  source, an undeclared triple, a host path in effect arguments, and
  over-limit units, inputs, and bytes.
- Producer: network denied, no host tools, and oracle failures reported.
- Matched output observation: run the same supported default-feature
  two-package app under the default `offlineCargoPackage` lane and the unit
  plan with identical source, lock, pinned toolchain, target, profile, app
  arguments, and environment. Compare observed stdout, stderr, and exit;
  compiler/linker settings may be noted as diagnostics, not as output-parity
  gates or raw-binary evidence. Feature activation remains unexercised in the
  `unit_plan` matrix.
- End to end: the representative fixtures build; an unchanged rerun executes
  zero units; one-package and outside-package edit counts; clean-client shared
  hits.
- Measurement: Mantle's own workspace unit count, plan bytes, and maximum
  per-unit inputs against named limits, recorded without a pass or fail claim.

## Risks / Trade-offs

- Per-unit sandbox setup costs time for every small crate. The still-open
  work-reduction bundle must record counts; elapsed time stays diagnostic.
- Mantle's measured full graph has 711 package sources, exceeding the
  `mantle-plan-v2` 256-slice bound before plan-byte admission; it is not
  supported by this bounded producer. Any capacity change needs its own
  reviewed limit decision.
- The unit graph is an unstable Cargo interface. The producer pins the
  toolchain and fails closed on unknown unit-graph versions.
- The producer reruns on every workspace edit. It runs Cargo planning only,
  not compilation.
- Manifest-driven closures depend on reference scanning of text files, which
  the fixtures check directly.

## Claim boundary

An accepted build receipt for the bounded fixture proves that its recorded
plan was produced from declared inputs, admitted, and executed as sandboxed
units with the observed reuse counts. No complete work-reduction bundle,
matched default-lane app output comparison, or uninterrupted final-source cold
build is claimed yet. This evidence does not prove Cargo equivalence,
Cargo-free execution, compiler correctness, output reproducibility, or release
eligibility.
