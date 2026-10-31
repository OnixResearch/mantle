# Proposal: Distribute C and C++ compilation units

## Why

`examples/build-from-source.ncl` contains independent `greet.o` and `main.o`
compiles, an archive and a link inside one Make derivation. A remote whole-root
build cannot place those compiles on different workers. The existing C/C++
classifier (`crunch-rust-cache-core::cc`) classifies bounded compile-only
invocations, but its local cache and slot driver do not expose a native unit
graph or authorize remote execution.

An explicit opt-in C/C++ unit-plan frontend can use the same generic pool as
Rust without an execution socket in the sandbox or a new compiler protocol.
It also makes generated-header ordering and include-root identity reviewable.

## What Changes

- Add a typed Nickel compilation graph and pure lowering to ordinary native
  dynamic plans: generator edges, one compile per translation unit, ordered
  archives/links and declared installed outputs.
  r[distributed_compilation.cc_explicit_graph]
- Bind complete source/header/include-search trees, generated inputs and
  compiler/sysroot/runtime identity before execution, not just a learned
  depfile. r[distributed_compilation.cc_complete_inputs]
- Reuse the existing compiler classifier for a declared supported C/C++
  fragment; keep unknown flags and side-effectful modes out of distributed
  lowering. r[distributed_compilation.cc_supported_fragment]
- Execute complete object/archive/link actions through generic native goals
  and standard verified output admission, with no cache-daemon authority.
  r[distributed_compilation.cc_artifact_execution]
- Adopt the existing `greet` workload and a small real C++/generated-header
  companion as repeatable local/pool acceptance consumers.
  r[distributed_compilation.cc_acceptance]

## Impact

- **Immediate consumer:** the existing multi-file `greet` example, preserving
  its ordinary Make route as an independent semantic baseline.
- **Immediate outcome:** its two compiler actions run on distinct workers and
  the resulting installed program/library/header work as declared.
- **Durable capability:** an explicit, content-bound C/C++ compile DAG frontend
  usable by recipes that can declare their generators, inputs and command graph.
- **Maintenance owner:** Mantle native-build frontend owner; generic pool,
  transfer, trust and store behavior stays with the build/remote owners.
- **Repeatability evidence:** actual C and C++ compilers, generated headers,
  local/pool artifact comparison, header-shadow/tool perturbations, and final
  executable/library consumption from independent stores.
- **Adoption path:** first expose a sibling unit-plan entry point for `greet`;
  later object-heavy StageX/GCC recipes may adopt the same graph after declaring
  their real generator/tool dependencies. This delivery does not claim those
  bootstrap recipes or arbitrary build systems were converted.

## Dependencies

`metadata.json` requires `schedule-native-goals-across-workers`. The existing
pure C classifier can be reused without depending on the local cache change's
separate performance/protected-bootstrap acceptance. No compile-cache daemon,
local slot authority or new remote protocol is required. See
[proposed ADR 0138](../../../adr/0138-distribute-compiler-units-through-native-build-goals.md).

## Non-Goals

Transparent interception or inference of arbitrary Make/Ninja/CMake graphs;
implicit headers, host compilers or PATH; local/remote shared object-cache
trust; distributed preprocessing as a substitute for complete inputs; plugins,
modules/PCH, LTO or side-effectful compiler modes outside the admitted fragment;
whole GCC/bootstrap speedup or reproducibility claims.

## Success Criteria

Both original `greet` semantics and a C++ generated-header application work
through real multi-worker compilation. A newly introduced shadowing header or
changed compiler changes input/action identity; unbound declared generator
references, undeclared include roots and unknown modes fail before remote
execution. Entirely omitted source dependencies fail in the sandbox without
ambient discovery; frontend admission does not claim preprocessor completeness.
Objects are verified before linking, and a compiler failure or stale object
cannot produce an accepted final application.
