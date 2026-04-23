## Phase 1: Crate scaffold and core types

- [x] Create `crates/crunch-system/Cargo.toml` with deps on `crunch-eval`, `crunch-glue`, `crunch-pipeline`, `serde`, `serde_json`; add to workspace `Cargo.toml`
- [x] Define `SystemConfigError` enum in `crates/crunch-system/src/error.rs` with variants: `Cli`, `Loader`, `Inventory`, `CrossRef`, `Eval`, `Fragment`, `Assembler` (ERR-1)
- [x] Define `SystemConfigWarning` enum in `crates/crunch-system/src/error.rs` for non-fatal warnings such as orphan provider consumption (ERR-1, EVAL-6e)
- [x] Define `EvalError` enum in `crates/crunch-system/src/eval_trait.rs` with variants: `NickelError(String)`, `Timeout`, `ImportDenied` (TRAIT-5)
- [x] Define `EvalOptions` struct with `timeout: Option<Duration>`, `import_paths: Vec<PathBuf>` (TRAIT-3)
- [x] Define `NickelEvaluator` trait with `evaluate_file`, `merge`, `call`, `get_field`, `is_function`, `to_json` methods (TRAIT-1)
- [x] Define `NickelValue` type alias for `crunch_eval::Expr` (TRAIT-2); re-export from `crates/crunch-system/src/lib.rs`
- [x] Define cross-layer types: `ValidatedModule`, `EvaluatedFragment`, `MergedConfig`, `FragmentSource` in `crates/crunch-system/src/lib.rs`
- [x] Define `Inventory`, `MachineRecord`, `ServiceRecord`, `InstanceRecord` structs in `crates/crunch-system/src/inventory.rs` with `serde::Deserialize` (INV-1, INV-2)
- [x] Define `SystemPipelineResult`, `MachineOutcome` result types with separate `errors` and `warnings` collections (ERR-4)
- [x] Verify: `cargo check -p crunch-system` compiles with all type definitions
  - Evidence: `cargo check -p crunch-system` passed on 2026-04-22 after adding the new workspace crate and scaffold types.

## Phase 2: Nickel contracts

- [x] Write `lib/system_module.ncl` — structural contract for service modules: `interface` (record with `roles`), `impl` (function), optional `inputs`, `consumes_providers`, `produces_providers`, `priority`
- [x] Write `lib/inventory.ncl` — structural contract for inventory files: `machines` (record of machine records with `system`, optional `class`), `services` (record of service records with `instances` array), instance shape (`machine`, `role`, optional `settings`, `tags`) (INV-4)
- [x] Add `lib/system_module.ncl` and `lib/inventory.ncl` to embedded stdlib in `crates/crunch-eval/src/stdlib.rs`
- [x] Unit test: Nickel contract validation — evaluate a valid module against `system_module.ncl` using `crunch_eval` directly (not eval thread), verify no blame error
- [x] Unit test: Nickel contract rejection — evaluate a module missing `interface` using `crunch_eval` directly, verify blame error
  - Evidence: `cargo test -p crunch-system contracts:: -- --nocapture` passed with `2 passed; 0 failed` on 2026-04-22.

## Phase 3: Module loader (pure core + I/O shell)

- [x] Implement `discover_module_files(dir: &Path) -> Result<Vec<(String, PathBuf)>>` — reads `*.ncl` files at top level, returns `(stem, path)` pairs (LOADER-1)
- [x] Implement duplicate-name detection in `discover_module_files` — error naming both files (LOADER-3)
- [x] Implement module count limit check (default 1024, configurable) (LOADER-5)
- [x] Implement `validate_module(name: &str, value_id: ValueId, handle: &EvalThreadHandle) -> Result<ValidatedModule, SystemConfigError>` — checks structural shape via `get_field` and `is_function` on the eval thread handle (LOADER-2). Uses `ValueId` because validation runs on the async side, not on the eval thread directly.
- [x] Extract module metadata during validation: declared role names plus `inputs`, `consumes_providers`, `produces_providers`, `priority` (default 1000) (LOADER-2, EVAL-12)
- [x] Unit test: discover 3 `.ncl` files in tempdir, verify stems
- [x] Unit test: discover ignores subdirectories
- [x] Unit test: duplicate stem detection
- [x] Unit test: module count limit exceeded (1025 files)
- [x] Unit test: validate_module with mock `EvalThreadHandle` — valid module returns ValidatedModule
- [x] Unit test: validate_module with mock `EvalThreadHandle` — missing `interface` returns Loader error
- [x] Unit test: validate_module with mock `EvalThreadHandle` — missing `impl` returns Loader error
- [x] Unit test: validate_module with mock `EvalThreadHandle` — `impl` is not a function returns Loader error
  - Evidence: `cargo test -p crunch-system loader:: -- --nocapture` passed with `8 passed; 0 failed` on 2026-04-22.

## Phase 4: Evaluator thread and channel protocol

- [x] Define `EvalRequest` / `EvalResponse` enums and `ValueId` type in `crates/crunch-system/src/threading.rs` — `EvalRequest` must include all variants: `EvaluateFile`, `GetField`, `IsFunction`, `Merge`, `Call`, `ToJson`, `DropValue`, `Shutdown` (TRAIT-7)
- [x] Implement eval-thread value table: `HashMap<u64, Expr>` with atomic `u64` counter for `ValueId` allocation and `DropValue` cleanup
- [x] Implement on-thread evaluator struct wrapping `crunch_eval` APIs and `nickel_lang::Context` — handles `EvalRequest` dispatch, calls `crunch_eval::evaluate()` for files, Nickel `Context` for merge/call/get_field, `Expr` type inspection for `is_function`
- [x] Implement `EvalThread::spawn(import_paths: Vec<PathBuf>) -> EvalThreadHandle` — spawns dedicated OS thread running the on-thread evaluator, returns handle with request channel and `tokio::sync::oneshot` per-request responses
- [x] Implement `EvalThreadHandle` methods: `evaluate_file`, `get_field`, `is_function`, `merge`, `call`, `to_json`, `drop_value`, `shutdown` — each sends request and awaits response with configurable timeout
- [x] Implement the concrete on-thread `NickelEvaluator` using `crunch_eval` APIs; keep `EvalThreadHandle` as the async `ValueId`/JSON bridge rather than the trait implementation (TRAIT-1, TRAIT-7)
- [x] Implement import path sandboxing: configure Nickel `Context` import paths to module dir + stdlib only (TRAIT-4)
- [x] Unit test: spawn eval thread, evaluate a simple `.ncl` file, get JSON back
- [x] Unit test: eval thread `get_field` — evaluate a Nickel record, extract a field by key via channel
- [x] Unit test: eval thread `is_function` — returns true for a function value, false for a record
- [x] Unit test: timeout enforcement — send request to eval thread with 1ms timeout, verify `EvalError::Timeout`
- [x] Unit test: shutdown — verify thread joins cleanly after `Shutdown` request
- [x] Unit test: drop_value — verify value table entry is freed after `drop_value` call
- [x] Unit test: eval thread panic — verify `EvalThreadHandle` methods return `EvalError` after thread panic
- [x] Integration test: import sandboxing — module that imports outside allowed paths, verify `ImportDenied` error (TRAIT-4)
  - Evidence: `cargo test -p crunch-system threading:: -- --nocapture` passed with `8 passed; 0 failed` on 2026-04-22.

## Phase 5: Topological sort and dependency graph

- [ ] Implement `build_dependency_graph(modules: &[ValidatedModule]) -> Result<DependencyGraph, SystemConfigError>` — builds directed graph from `inputs` edges (EVAL-1)
- [ ] Add provider edges to dependency graph: `produces_providers` → `consumes_providers` (EVAL-6b)
- [ ] Implement Kahn's algorithm with stable tiebreak (priority then name) for topological sort (EVAL-1)
- [ ] Implement cycle detection: if Kahn's queue empties early, report all remaining nodes and involved provider types (EVAL-2, EVAL-6c)
- [ ] Enforce fixed limits: max chain depth 256, max provider types per module 32, max total provider edges 4096 (EVAL-8)
- [ ] Unit test: 5 modules, linear chain A→B→C→D→E, verify order
- [ ] Unit test: diamond dependency A→B, A→C, B→D, C→D, verify D before B,C before A
- [ ] Unit test: provider-only ordering — A produces "firewall", C consumes "firewall", verify A before C with no explicit inputs
- [ ] Unit test: stable tiebreak — modules with no deps, different priorities, verify lower priority number first
- [ ] Unit test: stable tiebreak — equal priority, verify alphabetical
- [ ] Unit test: cycle detection — A→B→C→A, verify error names all three
- [ ] Unit test: provider cycle — A produces P/consumes Q, B produces Q/consumes P, verify error names modules and provider types
- [ ] Unit test: chain depth limit exceeded
- [ ] Unit test: provider edge count limit exceeded

## Phase 6: Module evaluator (orchestration)

- [ ] Implement inventory cross-reference validation: each service matches a module, each instance role matches module roles, each instance machine matches inventory machines (EVAL-12)
- [ ] Implement settings validation: merge instance settings with module interface defaults via `evaluator.merge()` (EVAL-3)
- [ ] Implement module `impl` invocation: call with `{settings, machine_name, role_name, upstream, providers}` record, serialize result to JSON (EVAL-4, EVAL-6d)
- [ ] Implement export threading: collect `output.exports` from each module's result, pass to downstream modules as `upstream.<module_name>` (EVAL-5)
- [ ] Implement provider collection: collect `output.providers` by type, pass as array to consuming modules (EVAL-6d)
- [ ] Implement per-module wallclock timeout (default 60s) via channel receive timeout (EVAL-10)
- [ ] Implement fail-open error collection: failed module does not block independent modules; dependent modules chain the failure diagnostic (EVAL-9, ERR-2)
- [ ] Unit test: cross-reference validation — service references nonexistent module, verify CrossRef error
- [ ] Unit test: cross-reference validation — instance references nonexistent role, verify CrossRef error
- [ ] Unit test: cross-reference validation — instance references nonexistent machine, verify CrossRef error
- [ ] Unit test: settings merge with fake handle-compatible boundary — valid settings returns merged value
- [ ] Unit test: impl invocation with fake handle-compatible boundary — verify args record shape
- [ ] Unit test: export threading — module A exports, module B sees upstream.A
- [ ] Unit test: provider merging — two modules produce same type, consumer gets ordered array
- [ ] Unit test: fail-open — module A fails, independent module B succeeds, B's result present
- [ ] Unit test: orphan provider warning — module consumes type no module produces, verify warning in `warnings` and empty providers array (EVAL-6e)
- [ ] Unit test: fail-open — module A fails, dependent module C also fails with chained diagnostic
- [ ] Unit test: determinism — run same modules+inventory twice through the evaluator, verify identical `EvaluatedFragment` output (EVAL-7)
- [ ] Call `handle.drop_value()` for consumed `ValidatedModule` value IDs after `impl` invocation and JSON serialization to prevent value table growth

## Phase 7: Fragment collector

- [ ] Implement `group_by_machine(fragments: &[EvaluatedFragment], inventory: &Inventory) -> BTreeMap<String, Vec<&EvaluatedFragment>>` (FRAG-1)
- [ ] Implement `merge_fragments(fragments: &[EvaluatedFragment]) -> Result<MergedConfig, Vec<SystemConfigError>>` — deep JSON merge with priority tiebreak (FRAG-2)
- [ ] Implement output namespace preservation — pass through `output.nixos`, `output.files`, etc. without interpretation (FRAG-3)
- [ ] Implement optional machine filter (FRAG-4)
- [ ] Implement merge provenance tracking: record which module contributed each top-level key (FRAG-5)
- [ ] Enforce merge depth limit (default 128, configurable) (FRAG-6)
- [ ] Unit test: group 4 fragments from 2 machines, verify grouping
- [ ] Unit test: deep merge — two fragments with compatible nested records
- [ ] Unit test: priority tiebreak — higher-priority fragment wins on scalar conflict
- [ ] Unit test: equal-priority conflict — error naming both modules and path
- [ ] Unit test: array replacement — fragment B's array replaces fragment A's at same path
- [ ] Unit test: namespace preservation — `output.nixos` and `output.files` both present
- [ ] Unit test: machine filter — only selected machine's fragments collected
- [ ] Unit test: provenance — verify each top-level key attributed to correct module
- [ ] Unit test: depth limit exceeded — tree deeper than 128, verify error

## Phase 8: System assembler trait and NixOS backend

- [ ] Define `Assembler` trait: `fn name(&self) -> &str`, `fn assemble(&self, machine_name, machine, config) -> Result<Vec<CrunchDerivation>, AssemblerError>` (ASM-1)
- [ ] Implement `NixosPhase1Assembler`: extracts `output.nixos` from MergedConfig, emits one `CrunchDerivation` per machine with JSON config in `env`, builder writes `$out/system-config.json` (ASM-2)
- [ ] Implement assembler registry: `BTreeMap<String, Box<dyn Assembler>>`, lookup by machine class (ASM-3)
- [ ] Verify emitted `CrunchDerivation` is compatible with `crunch-glue::convert()` (ASM-4)
- [ ] Implement dry-run mode: return derivations without submitting to build (ASM-6)
- [ ] Unit test: NixOS backend — MergedConfig with `output.nixos` → CrunchDerivation with correct env and `system` from `MachineRecord.system`
- [ ] Unit test: NixOS backend — MergedConfig without `output.nixos` → AssemblerError
- [ ] Unit test: assembler registry — register two backends, dispatch by machine class
- [ ] Unit test: dry-run — verify derivations returned, no build side effects
- [ ] Unit test: backend isolation — adding a new backend requires no changes to existing code

## Phase 9: Inventory validation

- [ ] Implement `validate_inventory(inv: &Inventory) -> Result<(), Vec<SystemConfigError>>` — enforce fixed limits: max machines 4096, max instances per service 4096, max total instances 65536 (INV-5)
- [ ] Unit test: valid inventory passes
- [ ] Unit test: machine count limit exceeded
- [ ] Unit test: instance per-service limit exceeded
- [ ] Unit test: total instance limit exceeded

## Phase 10: CLI integration

- [ ] Add `System { action: SystemAction }` variant to `Command` enum in `src/main.rs` with `Eval` and `Build` subcommands (CLI-1, CLI-2)
- [ ] Add `--modules <dir>` flag, default `./modules/` relative to inventory file (CLI-3)
- [ ] Add `--machine <name>` repeatable flag for machine filter (CLI-4)
- [ ] Add `--assembler <name>` flag for backend override on both `crunch system eval` and `crunch system build` (CLI-5)
- [ ] Add `--stop-after fragments|derivations` flag for eval (CLI-1)
- [ ] Add `--format json|nickel` flag for eval output (CLI-6)
- [ ] Implement `src/system_cmd.rs`: wire inventory deserialization → loader → evaluator thread → evaluator → collector → assembler → pipeline
- [ ] Add `crunch-system` and `crunch-pipeline` deps to the binary crate's Cargo.toml
- [ ] Structured JSON warning/error output on stderr when `--json` is active (ERR-3)
- [ ] `--format nickel` returns clear "not yet implemented" error message (CLI-6, design Decision 15)

## Phase 11: Integration tests

- [ ] Write example modules: `sshd.ncl` (no deps), `firewall.ncl` (provider: firewall), `nginx.ncl` (consumes: firewall, inputs: sshd), in `examples/system-config/modules/`
- [ ] Write example inventory: 2 machines, instances of all 3 modules, in `examples/system-config/inventory.ncl`
- [ ] Integration test: `crunch system eval examples/system-config/inventory.ncl` produces the expected per-machine dry-run result envelope for both machines
- [ ] Integration test: `crunch system eval --stop-after=fragments` produces merged config trees
- [ ] Integration test: `crunch system eval --machine=server1` only evaluates server1
- [ ] Integration test: `crunch system build` (gated by `can_build()`) builds derivations and produces store output
- [ ] Integration test: partial failure — inventory with bad settings on one machine, verify the other machine succeeds, stdout keeps the successful machine result, and stderr reports the error
- [ ] Integration test: module with contract violation produces Nickel blame error with module name and field path

## Phase 12: Documentation

- [ ] Add `crunch system eval` and `crunch system build` to README CLI reference
- [ ] Write `docs/system-config.md` covering module format, inventory schema, CLI usage, and assembler backends
- [ ] Add module authoring examples to `examples/system-config/README.md`
