## Context

crunch can build packages (derivations from `.ncl` files) and bootstrap itself,
but has no machinery for system configurations — the NixOS-style pattern where
many service modules are evaluated against an inventory, merged, and assembled
into a full system derivation.

onix-modules proves the pattern works in Nickel: 19 service modules, topological
evaluation, providers/exports threading, inventory-driven multi-machine configs.
But it depends on a Nix shim (`nix/assemble.nix`) for evaluation orchestration
and NixOS assembly. crunch needs to own this pipeline natively.

The specs define 8 layers (loader, evaluator, fragment collector, assembler,
CLI, inventory, error model, evaluator trait) with 55 requirements total. This
design maps those requirements to concrete Rust modules, Nickel contracts, data
flow, and integration points with the existing crunch crate tree.

## Goals / Non-Goals

**Goals:**
- Evaluate Nickel service modules against an inventory without Nix.
- Thread exports and providers between modules in topological order.
- Merge output fragments per machine and assemble buildable derivations.
- Wire into crunch's existing pipeline so `crunch system build` works end-to-end.
- Ship a phase-1 NixOS assembler backend that proves the pipeline, not a
  complete NixOS system closure generator.

**Non-Goals:**
- Importing existing NixOS modules or nixpkgs directly (separate compat layer).
- Full NixOS closure assembly (initrd, systemd units, `/etc` generation).
- Secrets management (sops-nix).
- Deployment/activation (`crunch deploy`).
- Cloud provisioning (OpenTofu).

## Decisions

### 1. Single crate: `crunch-system`

**Choice:** All five layers (loader, evaluator, fragment collector, assembler,
CLI wiring) live in `crates/crunch-system/`. Each layer is a submodule.

**Layout:**
```
crates/crunch-system/
  src/
    lib.rs              -- public API, re-exports
    loader.rs           -- LOADER-1..5
    evaluator.rs        -- EVAL-1..12
    fragment.rs         -- FRAG-1..6
    assembler.rs        -- ASM-1..6, trait + NixOS backend
    inventory.rs        -- INV-1..6 (Rust types for deserialized inventory)
    error.rs            -- ERR-1..4
    eval_trait.rs       -- TRAIT-1..7, NickelEvaluator definition
    threading.rs        -- dedicated evaluator thread, channel protocol
```

**Rationale:** The layers share types (`ValidatedModule`, `EvaluatedFragment`,
`MergedConfig`) and the dependency graph is linear (loader → evaluator →
collector → assembler). Splitting into 5 crates would create circular-dep risk
and compile-time overhead with no versioning benefit.

### 2. NickelEvaluator trait defined in `crunch-system`, implemented via `crunch-eval`

**Choice:** The `NickelEvaluator` trait (TRAIT-1) is defined in
`crates/crunch-system/src/eval_trait.rs`. The concrete implementation wraps
`crunch_eval::evaluate()`, `crunch_eval::Expr::to_serde()`, and Nickel
`Context` merge operations.

The concrete evaluator struct lives in `crunch-system` (not `crunch-eval`)
to avoid a `crunch-eval → crunch-system` dependency. It constructs a
`nickel_lang::Context` and adapts it to the trait interface.

The trait includes `get_field(value, key) -> Result<NickelValue>` and
`is_function(value) -> Result<bool>` alongside `evaluate_file`, `merge`,
`call`, and `to_json`. `get_field` enables the loader to extract metadata
fields (`inputs`, `consumes_providers`, `produces_providers`, `priority`)
from the evaluated module record without serializing the whole module to
JSON. `is_function` validates that `impl` is callable (LOADER-2).

**Rationale:** `crunch-eval` is a leaf crate that should not know about
system-config concepts. The trait boundary also enables mock evaluators
for unit testing the pure pipeline without a Nickel runtime.

### 3. NickelValue = `nickel_lang::Expr` (existing crunch-eval type)

**Choice:** `NickelValue` is a newtype over `crunch_eval::Expr`, which is
already a public re-export of `nickel_lang::Expr`. No new wrapper needed —
the existing `Expr` type satisfies TRAIT-2 (Clone, Debug, !Send, !Sync).

The eval_trait module defines `pub type NickelValue = crunch_eval::Expr;`
and the pipeline passes `Expr` handles through the loader and evaluator
layers on the dedicated evaluator thread.

**Rationale:** `Expr` wraps `nickel_lang_core::term::RichTerm` behind the
stable `nickel_lang` API. It has `to_serde()` for deserialization and can
be merged via `Context::merge()`. Adding a newtype would just add
boilerplate with no abstraction benefit. The proposal's reference to
`RichTerm` described the internal representation; the public API type is
`Expr`.

### 4. Dedicated evaluator thread with typed channel (TRAIT-7)

**Choice:** `threading.rs` spawns a single `std::thread::spawn` that owns
the `nickel_lang::Context`. The async pipeline communicates via a pair of
channels: `std::sync::mpsc::Sender<EvalRequest>` for requests (caller →
eval thread) and `std::sync::mpsc::Sender<EvalResponse>` for responses
(eval thread → caller). Each `EvalRequest` carries a
`oneshot::Sender<EvalResponse>` so responses are routed back to the
specific caller. Timeout enforcement is on the caller side via
`tokio::time::timeout` on the oneshot receive.

**Protocol:**
```rust
enum EvalRequest {
    EvaluateFile { path: PathBuf, opts: EvalOptions, reply: oneshot::Sender<EvalResponse> },
    GetField { value_id: ValueId, key: String, reply: oneshot::Sender<EvalResponse> },
    IsFunction { value_id: ValueId, reply: oneshot::Sender<EvalResponse> },
    Merge { base_id: ValueId, overlay_id: ValueId, opts: EvalOptions, reply: oneshot::Sender<EvalResponse> },
    Call { func_id: ValueId, args_id: ValueId, opts: EvalOptions, reply: oneshot::Sender<EvalResponse> },
    ToJson { value_id: ValueId, reply: oneshot::Sender<EvalResponse> },
    DropValue { id: ValueId },
    Shutdown,
}

enum EvalResponse {
    Value(ValueId),
    Bool(bool),
    Json(serde_json::Value),
    Error(EvalError),
}
```

`ValueId` is a `u64` handle into a `HashMap<u64, Expr>` on the evaluator
thread. Values never cross the thread boundary — only IDs and serialized
JSON do. The evaluator thread drops its value table on `Shutdown`.

**Layering:** The `NickelEvaluator` trait (TRAIT-1) is called directly on
the evaluator thread with `&NickelValue` (`&Expr`) references. The
channel protocol is the async-to-sync bridge used by the CLI/pipeline
layer to communicate with the evaluator thread from async code. The
concrete `NickelEvaluator` implementation lives on the eval thread and
never sees `ValueId` — it works with `Expr` values directly. The
`EvalThreadHandle` translates between `ValueId`-based channel messages
and trait method calls. Pipeline code on the async side never calls
trait methods directly; it sends channel messages and receives
`ValueId`s or `serde_json::Value` results.

**`EvalThreadHandle` public API:**
```rust
impl EvalThreadHandle {
    fn evaluate_file(&self, path: PathBuf, opts: EvalOptions) -> Result<ValueId, EvalError>;
    fn get_field(&self, value: ValueId, key: String) -> Result<ValueId, EvalError>;
    fn is_function(&self, value: ValueId) -> Result<bool, EvalError>;
    fn merge(&self, base: ValueId, overlay: ValueId, opts: EvalOptions) -> Result<ValueId, EvalError>;
    fn call(&self, func: ValueId, args: ValueId, opts: EvalOptions) -> Result<ValueId, EvalError>;
    fn to_json(&self, value: ValueId) -> Result<serde_json::Value, EvalError>;
    fn drop_value(&self, value: ValueId);  // explicit cleanup
    fn shutdown(self);  // joins the eval thread
}
```

Each method sends the corresponding `EvalRequest` with a `oneshot`
sender, then blocks on the response with a timeout. `drop_value` sends
a `DropValue { id }` request to free the entry in the value table.

**`ValueId` lifecycle:** IDs are allocated sequentially by the eval
thread (atomic u64 counter). The value table grows as modules are loaded
and evaluated. Explicit cleanup via `drop_value` is called when a
`ValidatedModule` is consumed (after its `impl` has been invoked and
serialized). On `Shutdown`, the entire value table is dropped.

**Eval thread panic recovery:** If the eval thread panics, the
request channel's receiver half is dropped. All pending and future
`EvalThreadHandle` method calls receive a `RecvError`, which is mapped
to `EvalError::NickelError("evaluator thread panicked")`. The pipeline
treats this as a fatal error and stops.

**Rationale:** `Expr` is `!Send`, so all Nickel values must stay on one
thread. A channel protocol is simpler than `spawn_blocking` (which would
require `Send` closures) and gives explicit control over the value
lifetime. The `ValueId` indirection avoids serialization overhead for
intermediate pipeline values that are only consumed by later Nickel
operations on the same thread.

### 5. Inventory deserialized via `crunch-eval` existing path

**Choice:** The inventory file is evaluated by the existing
`crunch_eval::evaluate_and_deserialize::<Inventory>(path, import_paths)`.
The `Inventory` Rust type in `inventory.rs` mirrors the Nickel contract
shape (INV-1, INV-2) and derives `serde::Deserialize`. Fixed-limit
validation (INV-5) runs on the deserialized struct, not in Nickel.

The `MachineRecord` type mirrors a single machine entry:
```rust
pub struct MachineRecord {
    pub system: String,
    pub class: String, // default "nixos"
    pub extra: BTreeMap<String, serde_json::Value>,
}
```
`extra` captures any user-defined metadata fields. The full record is
passed to the assembler (Decision 10, INV-1).

**Rationale:** The inventory is pure data (INV-6) with no opaque function
handles. JSON round-trip through serde is fine — there are no lazy Nickel
values to preserve. This avoids putting inventory loading on the evaluator
thread.

### 6. Module loading: two-phase (file eval on eval thread, validation in pure Rust)

**Choice:** Module loading splits into:
1. **I/O shell:** `std::fs::read_dir` to discover `*.ncl` files (LOADER-1).
2. **Eval boundary:** For each file, send `EvaluateFile` to the evaluator
   thread, get back a `ValueId`.
3. **Pure core:** `validate_module(name, value_id, evaluator)` — sends
   targeted queries to the eval thread to check structural shape (has
   `interface`? has `impl`? `impl` is a function?) (LOADER-2). Returns
   `ValidatedModule` with extracted metadata (inputs, providers, priority).

`ValidatedModule` holds `ValueId` handles for `interface` and `impl`, not
full `Expr` clones. It is consumed on the evaluator thread only.

**Rationale:** Structural validation requires inspecting Nickel values
(checking field existence, asserting `impl` is a function). These checks
go through the eval trait. The validate function is pure in the sense
that it has no filesystem I/O — its only dependency is the evaluator
channel.

### 7. Topological sort with provider edges (EVAL-1, EVAL-6b)

**Choice:** Build a directed graph where edges are:
- Explicit: module A declares `inputs = ["B"]` → edge B→A.
- Provider: module A declares `produces_providers = ["firewall"]`,
  module C declares `consumes_providers = ["firewall"]` → edge A→C.

Sort with Kahn's algorithm. Stable tiebreak: lower priority number first
(EVAL-1), then alphabetical name.

**Rationale:** Kahn's algorithm naturally detects cycles (EVAL-2, EVAL-6c)
— if the queue empties before all nodes are processed, the remaining nodes
form cycles. The algorithm is O(V+E) and well-suited to the fixed-limit
bounds in EVAL-8.

**Orphan provider handling (EVAL-6e):** During graph construction, if a
module declares `consumes_providers` with a type not in any module's
`produces_providers`, no edge is added and a warning is pushed to
`SystemPipelineResult.errors` as a non-fatal `SystemConfigError::Eval`
with detail `"orphan provider type '<type>' consumed by '<module>' but
not produced by any loaded module"`. At evaluation time, the consuming
module receives `providers.<type> = []` (empty array).

### 8. EvaluatedFragment uses serde_json::Value, not Expr

**Choice:** After calling each module's `impl` function (EVAL-4), the
evaluator immediately serializes the result to JSON via `to_json()`
(TRAIT-1). The `EvaluatedFragment` struct carries
`data: serde_json::Value` across the thread boundary to the fragment
collector.

**Rationale:** The fragment collector (FRAG-1..6) is pure Rust with no
Nickel dependency. JSON is the serialization boundary. This is the same
pattern as the existing `crunch-pipeline` which deserializes
`CrunchDerivation` from Nickel via serde.

### 9. Fragment collector: deep merge with priority tiebreak

**Choice:** `merge_fragments(fragments: &[EvaluatedFragment]) -> MergedConfig`
performs recursive JSON object merging. On scalar conflict at the same path:
- Higher priority (lower number) wins.
- Equal priority: error naming both modules and the conflicting path (FRAG-2).

Arrays are replaced wholesale (not concatenated). The collector does not
interpret namespace contents — `output.nixos`, `output.files`, etc. are
just JSON subtrees (FRAG-3).

**Rationale:** Deep merge with explicit conflict rules matches onix-modules'
existing behavior (its Nix shim does priority-ordered `lib.recursiveUpdate`).
Array replacement is simpler and avoids order-dependent array semantics.

### 10. Assembler trait with NixOS phase-1 backend

**Choice:** The assembler trait (ASM-1) is:
```rust
pub trait Assembler: Send + Sync {
    fn name(&self) -> &str;
    fn assemble(
        &self,
        machine_name: &str,
        machine: &MachineRecord,
        config: &MergedConfig,
    ) -> Result<Vec<CrunchDerivation>, AssemblerError>;
}
```

The NixOS phase-1 backend (ASM-2) emits a single derivation per machine
whose `env` contains the merged `output.nixos` config as a JSON string
and whose builder writes `$out/system-config.json`. This uses
`crunch.mkDerivation` via the existing `crunch-glue` path.

**Rationale:** Object-safe trait for runtime dispatch (ASM-5). Phase 1 is
deliberately minimal — it proves the pipeline end-to-end without tackling
real NixOS system closure assembly. The assembler emits `CrunchDerivation`
(ASM-4) which feeds directly into `crunch-pipeline::build()`.

### 11. CLI integration via new subcommand group

**Choice:** `crunch system eval|build` as new subcommands in `src/main.rs`.
The CLI module (`src/system_cmd.rs`) does:
1. Deserialize inventory via `crunch-eval`.
2. Discover module files (I/O shell).
3. Spin up evaluator thread.
4. Run loader → evaluator → collector → assembler pipeline.
5. For `eval`: print derivation records as JSON (dry-run, ASM-6).
6. For `build`: feed derivation records to `crunch-pipeline::build()`.

**Rationale:** Follows the existing pattern (build_cmd.rs, store_cmd.rs,
project_cmd.rs). The system subcommand group keeps system-config commands
namespaced away from package-level commands.

### 12. Nickel contracts for module structure and inventory

**Choice:** Two new Nickel contract files in `lib/`:
- `lib/system_module.ncl` — structural contract for service modules.
  Validates `interface`, `impl`, optional `inputs`/`consumes_providers`/
  `produces_providers`/`priority`. Does NOT validate domain-specific
  role schemas.
- `lib/inventory.ncl` — structural contract for inventory files (INV-4).
  Validates `machines`, `services`, instance shapes. Cross-reference
  checks deferred to Rust (EVAL-12).

These contracts are embedded in the Nickel stdlib
(`crates/crunch-eval/src/stdlib.rs`) alongside `lib/lib.ncl`.

**Rationale:** Nickel contracts give blame-tracked validation errors with
source locations. Structural shape is contract territory; cross-reference
validation (module names matching service names) requires loaded module
metadata and belongs in Rust.

### 13. Error collection and partial success (ERR-1..4)

**Choice:** `SystemConfigError` is an enum with per-layer variants. The
pipeline returns `SystemPipelineResult` which carries both successful
per-machine outcomes and collected non-fatal errors:

```rust
pub struct SystemPipelineResult {
    pub machines: BTreeMap<String, MachineOutcome>,
    pub errors: Vec<SystemConfigError>,
}
pub enum MachineOutcome {
    Built(Vec<BuildOutcome>),
    DryRun(Vec<CrunchDerivation>),
    Failed,
}
```

Propagation follows ERR-2: inventory failure is fail-closed (pipeline
stops), other layers are fail-open per independent subgraph/machine.

**Rationale:** Partial success is critical for multi-machine configs. If
machine-A's sshd module has a contract error, machine-B's build should
still proceed.

### 14. Orphan provider consumption is a warning, not an error

**Choice:** If a module declares `consumes_providers = ["metrics"]` but no
loaded module declares `produces_providers` containing `"metrics"`, the
evaluator emits a warning (not an error). The consuming module receives
`providers.metrics = []` (empty array).

**Rationale:** Provider types may be optional — a module that can consume
firewall rules should still work in a config that has no firewall module.
Making this a hard error would force users to load all provider sources
even when they're not needed. The warning surfaces potential config gaps
without blocking evaluation.

### 15. `--format nickel` deferred to follow-on

**Choice:** CLI-6 specifies `--format nickel` for eval output. Phase 1
implements `--format json` only. `--format nickel` is accepted by the CLI
parser but returns an error: `"Nickel output format is not yet implemented.
Use --format json."`

**Rationale:** Re-serializing `serde_json::Value` back to valid Nickel
syntax requires a Nickel pretty-printer that handles records, arrays,
strings, numbers, and enums correctly. This is non-trivial and orthogonal
to proving the pipeline. Deferring it avoids blocking the core work.

### 16. Import sandboxing (TRAIT-4)

**Choice:** The concrete `NickelEvaluator` configures the Nickel
`Context` with a restricted import resolver that only allows paths under:
1. The module directory (where `*.ncl` files live).
2. crunch's stdlib directory (`lib/`).

Imports resolving outside these directories produce `EvalError::ImportDenied`.

**Implementation:** Nickel's `Context` accepts custom import paths. The
concrete evaluator passes only the allowed directories. When the Nickel
resolver tries to access a path outside these roots, it fails with an
import error that the evaluator wraps as `ImportDenied`.

**Rationale:** Without sandboxing, a malicious module could `import
"/etc/shadow"` and exfiltrate data through its output fragments.

## Data Flow

```
                      ┌──────────────┐
                      │ inventory.ncl│
                      └──────┬───────┘
                             │ crunch-eval::evaluate_and_deserialize
                             ▼
                      ┌──────────────┐
                      │  Inventory   │ (Rust struct, Send+Sync)
                      └──────┬───────┘
                             │
  ┌──────────────┐           │
  │ modules/*.ncl│───────────┤
  └──────┬───────┘           │
         │ fs::read_dir      │
         ▼                   │
  ┌──────────────────────────┴──────────────────────┐
  │            Evaluator Thread                      │
  │                                                  │
  │  evaluate_file → validate_module → [ValidatedModule]  │
  │       │                                          │
  │  topo_sort(modules, inventory)                   │
  │       │                                          │
  │  for each module in topo order:                  │
  │    merge(interface.defaults, instance.settings)  │
  │    call(impl, {settings, machine, role, upstream})│
  │    to_json(result) ──────────────────────────┐   │
  │                                              │   │
  └──────────────────────────────────────────────┼───┘
                                                 │
                                                 ▼
                                    ┌────────────────────┐
                                    │[EvaluatedFragment]  │
                                    │ (serde_json::Value) │
                                    └────────┬───────────┘
                                             │
                                    merge_fragments(by machine)
                                             │
                                             ▼
                                    ┌────────────────────┐
                                    │ MergedConfig        │
                                    │ per machine         │
                                    └────────┬───────────┘
                                             │
                                    assembler.assemble()
                                             │
                                             ▼
                                    ┌────────────────────┐
                                    │ [CrunchDerivation]  │
                                    └────────┬───────────┘
                                             │
                          ┌──────────────────┴──────────────────┐
                          │ eval mode              build mode   │
                          ▼                        ▼            │
                    print JSON             crunch-pipeline      │
                                           ::build()            │
                          └──────────────────┴──────────────────┘
```

## Verification Strategy

| Spec area | Test type | Method |
|---|---|---|
| Loader structural validation (LOADER-2) | Unit | Mock evaluator returning records with missing/extra fields |
| Loader module count limit (LOADER-5) | Unit | Generate 1025 fake modules, assert limit error |
| Loader duplicate name (LOADER-3) | Unit | Two modules with same stem, assert error |
| Topo sort + stable tiebreak (EVAL-1) | Unit | 5 modules with known deps, verify order |
| Cycle detection (EVAL-2) | Unit | A→B→C→A cycle, verify error names all three |
| Provider edge ordering (EVAL-6b) | Unit | Producer→consumer ordering without explicit inputs |
| Provider cycle (EVAL-6c) | Unit | A produces P, B consumes P and produces Q, A consumes Q |
| Settings merge / contract blame (EVAL-3) | Integration | Real Nickel evaluator, module with typed interface, bad settings |
| Export threading (EVAL-5) | Unit | Module A exports, module B reads upstream.A |
| Provider merging (EVAL-6d) | Unit | Two modules produce same provider type, consumer gets array |
| Cross-reference validation (EVAL-12) | Unit | Inventory references nonexistent module/role/machine |
| Fragment deep merge (FRAG-2) | Unit | Two fragments at same path, priority tiebreak |
| Fragment conflict at equal priority (FRAG-2) | Unit | Two fragments at same path+priority, assert error |
| Fragment depth limit (FRAG-6) | Unit | Deeply nested JSON tree, assert limit error |
| Fragment provenance (FRAG-5) | Unit | Verify provenance records which module contributed each key |
| Assembler NixOS backend (ASM-2) | Unit | MergedConfig → CrunchDerivation with JSON env |
| Assembler dry-run (ASM-6) | Unit | Verify assemble returns derivations without build |
| Evaluator timeout (EVAL-10) | Integration | Module impl with infinite loop, verify timeout error |
| Import sandboxing (TRAIT-4) | Integration | Module imports `/etc/passwd`, verify ImportDenied |
| End-to-end eval (CLI-1) | Integration | 3 modules, 2 machines, `crunch system eval`, verify JSON |
| End-to-end build (CLI-2) | Integration | Same setup, `crunch system build`, verify store output |
| Inventory fixed limits (INV-5) | Unit | Inventory with 4097 machines, verify limit error |
| Inventory pure data (INV-6) | Unit | Inventory evaluated as pure Nickel, verify no side effects |
| Evaluator determinism (EVAL-7) | Unit | Run same modules+inventory twice, verify identical output |
| Evaluator fixed limits (EVAL-8) | Unit | Chain depth 257, verify error; provider edges 4097, verify error |
| Orphan provider warning (EVAL-6e) | Unit | Module consumes unproduced type, verify warning + empty array |
| JSON error output (ERR-3) | Integration | `--json` flag, verify stderr has `layer`/`module`/`machine`/`message` fields |
| Assembler override (CLI-5) | Integration | `--assembler=nixos` overrides machine class, verify backend used |
| Error partial success (ERR-2) | Integration | 3 machines, one with bad module, verify other two succeed |

## Risks / Trade-offs

**[Phase-1 NixOS backend is a stub]** It writes JSON, not a bootable system.
Real NixOS assembly (systemd units, `/etc`, activation scripts) requires
either calling nixpkgs's `nixos/eval-config.nix` or reimplementing the
assembly in Nickel/Rust. This is explicitly deferred — the phase-1 backend
proves pipeline integration, not system completeness.

**[Nickel evaluator thread is single-threaded]** All module evaluations
are sequential on one thread. For 19 modules this is fast enough
(typically <5s). If module count grows to hundreds, the evaluator thread
becomes a bottleneck. Mitigation: the architecture supports multiple
evaluator threads with separate `Context` instances, but this requires
duplicating loaded modules across contexts.

**[No Nix module compat]** Existing NixOS modules cannot be used. Users
must (re)write modules in the Nickel service-module pattern. This is the
right trade-off for crunch's goals but limits adoption. A Nix compat shim
is a separate future change.

**[JSON serialization boundary]** Serializing every module's output to
JSON before fragment merging adds overhead and loses Nickel-specific
types (functions, contracts). This is acceptable because fragment output
is data, not code — modules produce configuration records, not
functions.

**[Import sandboxing depends on Nickel resolver behavior]** The sandbox
relies on Nickel's import path resolution respecting the configured
directories. If Nickel adds new import mechanisms (e.g., URL imports),
the sandbox may need updating.
