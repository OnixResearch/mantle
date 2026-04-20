# System config modules for crunch

## Why

crunch can build packages and bootstrap itself, but cannot build system
configurations (NixOS-style). onix-modules proves the module pattern works
(19 service modules, inventory, topological evaluation, providers, exports)
but depends on a Nix shim for assembly and derivation building. crunch
needs its own module evaluation and assembly layer so system configs can
be built without Nix.

The gap is not "rewrite onix-modules in crunch." It is: give crunch the
machinery to evaluate a set of Nickel service modules against an inventory,
collect their output fragments, and produce buildable derivations — without
coupling crunch to any specific module schema, NixOS, or nixpkgs.

## What

A modular, loosely coupled system-config pipeline with these layers:

1. **Module loader** — discovers and loads Nickel module files from a
   directory. Pure function: path → list of typed module records. No
   knowledge of what the modules contain beyond the structural contract.

2. **Module evaluator** — resolves dependency order (topological sort on
   declared inputs), threads exports/providers between modules, validates
   settings against interface contracts via Nickel merge, and calls each
   module's `impl` function. Pure core: modules × inventory × overrides →
   evaluated output fragments.

3. **Fragment collector** — merges per-module output fragments into a
   single config tree per machine. Pure function: evaluated fragments ×
   machine list → merged config per machine.

4. **System assembler** — turns a merged config tree into buildable
   derivations using crunch's existing `mkDerivation`/builder pipeline.
   This is the only layer that touches the build engine. Pluggable:
   different assembler backends for different targets (NixOS system,
   container image, s6-based system, etc.).

5. **CLI surface** — `crunch system build <inventory.ncl>` and
   `crunch system eval <inventory.ncl>` entry points that wire the
   pipeline together.

### Boundaries and coupling

- Layers 1–3 are pure Nickel + pure Rust. No I/O, no async, no store access.
- Layer 4 is the imperative shell: it calls into `crunch-pipeline::build()`.
- Each layer is a Rust module within a single new `crunch-system` crate.
  Promotion to a separate crate is warranted only if a layer needs
  independent versioning or becomes a binary target. Default: one crate,
  one `lib.rs`, submodules per layer.
- **Crate dependency direction:** `crunch-system` depends on `crunch-eval`
  (for the `NickelValue` type and the production `NickelEvaluator`
  implementation). `crunch-system` depends on `crunch-pipeline` and
  `crunch-glue` (for building assembled derivations). The
  `NickelEvaluator` trait is defined in `crunch-system`; `crunch-eval`
  does NOT depend on `crunch-system`. The concrete evaluator
  implementation is constructed in `crunch-system` from `crunch-eval`
  APIs and adapts them to the trait interface.
- The module schema (interface/impl/roles/providers) lives in Nickel
  contracts, not hardcoded in Rust. crunch validates structural shape
  (has `interface`, has `impl`, `impl` is a function) but does not
  encode domain-specific contracts like `Port` or `Hostname`.
- Assembler backends are trait objects. The NixOS assembler is one
  implementation; others can be added without changing the pipeline.

### FCIS split

**Functional core** (deterministic, testable, no I/O):
- Topological sort of module dependencies (pure graph algorithm)
- Fragment merging per machine (pure record merge)
- Config tree → derivation record conversion (pure transformation,
  split out of the assembler as a separate pure function;
  the assembler trait's `assemble()` calls this internally but
  the conversion logic has no I/O and is independently testable)

**Nickel evaluation boundary** (deterministic but requires a runtime):
- Settings validation via Nickel contract merge
- Module evaluation (calling `impl` with validated args)

Nickel evaluation is deterministic (same inputs → same outputs) but
requires an evaluator runtime. The pure-core Rust functions accept a
`NickelEvaluator` trait as an injected dependency. This trait is
object-safe and has two methods: `merge(base, overlay) → Result` and
`call(function, args) → Result`. The trait is defined in the
system-config crate; `crunch-eval` provides the concrete
implementation. Tests inject a mock evaluator.

**Imperative shell** (I/O, async, side effects):
- Filesystem discovery of module files
- CLI argument parsing
- Calling `crunch-pipeline::build()` with assembled derivations
- Store interaction for output persistence

### Cross-layer types

- `ValidatedModule` — loader output. Contains module identity, parsed
  interface (roles, providers), `impl` as an opaque Nickel function
  handle, declared inputs/priorities.
- `NickelValue` — opaque newtype wrapping `nickel_lang_core::term::RichTerm`,
  defined in `crunch-eval` and re-exported by `crunch-system`. Implements
  `Clone`, `Debug`. NOT `Send`/`Sync` — all Nickel evaluation stays on
  one thread per evaluator instance.
- `EvaluatedFragment` — evaluator output per instance. Contains
  `module_name`, `machine_name`, `role_name`, `priority: u32`, and
  `data: serde_json::Value` (the Nickel record serialized to JSON by
  the evaluator before crossing the Nickel boundary).
- `MergedConfig` — fragment collector output per machine. A struct
  containing `config: BTreeMap<String, serde_json::Value>` (the merged
  output namespaces) and `provenance: Vec<FragmentSource>` (which
  module contributed each top-level key). Implements `Send + Sync`.
- `CrunchDerivation` — assembler output. The existing `crunch-glue`
  input type (`crunch_glue::CrunchDerivation`), compatible with
  `crunch-pipeline::build()`.

### What this does NOT include

- Importing existing NixOS modules or nixpkgs directly. That requires a
  Nix compatibility shim which is a separate concern.
- Secrets management (sops-nix integration). Orthogonal.
- Deployment/activation (applying a built system). Separate command.
- Cloud provisioning (OpenTofu). Separate layer.

## Constraints

- MUST NOT require Nix at runtime. The whole point is replacing the shim.
- MUST reuse crunch's existing eval, build, and store infrastructure.
- MUST keep module schema in Nickel, not Rust. Schema changes should not
  require recompiling crunch.
- MUST support the onix-modules module shape as the first backend without
  being permanently coupled to it.
- Module evaluation MUST be deterministic: same inputs → same outputs.
- Nickel evaluation of user modules MUST be bounded by a configurable
  wallclock timeout (default 60s per module). Memory bounding SHOULD be
  enforced if the Nickel runtime supports it.
- The Nickel evaluator runs on a dedicated OS thread (not `spawn_blocking`,
  because `NickelValue` is `!Send`). The pipeline communicates with it
  via a typed channel. Timeout is enforced by the caller via
  `tokio::time::timeout` on the channel receive. Only serialized results
  (`serde_json::Value`, error strings) cross the thread boundary.
- Assembler trait MUST be object-safe for runtime dispatch.
- Each layer MUST be independently testable with no filesystem or store.
- Nickel `import` inside user modules is allowed but sandboxed: the
  `NickelEvaluator` resolves imports only within the module directory
  and crunch's stdlib. Imports outside these paths MUST be rejected
  with a diagnostic naming the offending import path.

## Traceability

| Proposal layer       | Spec file              | Requirement IDs          |
|----------------------|------------------------|--------------------------|
| Module loader        | `module-loader.md`     | LOADER-1 through LOADER-5|
| Module evaluator     | `module-evaluator.md`  | EVAL-1 through EVAL-12   |
| Fragment collector   | `fragment-collector.md`| FRAG-1 through FRAG-6    |
| System assembler     | `system-assembler.md`  | ASM-1 through ASM-6      |
| CLI surface          | `cli-surface.md`       | CLI-1 through CLI-6      |
| Inventory (input)    | `inventory.md`         | INV-1 through INV-6      |
| Error model          | `error-model.md`       | ERR-1 through ERR-4      |
| Evaluator trait      | `evaluator-trait.md`   | TRAIT-1 through TRAIT-7  |

## How to validate

1. Unit tests for each pure-core layer with in-memory module definitions.
2. Integration test: load 3+ modules with dependencies, evaluate against
   a 2-machine inventory, verify correct topological order, export
   threading, and merged output fragments.
3. End-to-end: `crunch system eval examples/system-config/` produces
   correct derivation records for a minimal system config.
4. Assembler test: NixOS assembler backend (phase 1) produces a
   derivation whose builder writes the merged `output.nixos` config
   tree as `$out/system-config.json`.
