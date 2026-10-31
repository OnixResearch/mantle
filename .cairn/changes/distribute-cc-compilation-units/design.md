# Design: Distribute C and C++ compilation units

## State and first consumer

Proposed opt-in frontend, not transparent distributed Make support.
`examples/build-from-source.ncl:58-87` already describes a small DAG:
`greet.c -> greet.o -> libgreet.a`, `main.c -> main.o`, then the executable and
installation. The independent compiles share `greet.h`. Preserve that workload
and its default Make entry point; extract shared tracked fixture sources so the
comparison lanes cannot drift through duplicated source literals. Add a small
C++ companion with a generated header and explicit expected runtime values.
Neither fixture represents a full StageX/GCC conversion.

The current pure classifier in `crunch-rust-cache-core/src/cc.rs` rejects unknown
options, response files and side-effectful modes. Reuse it as a necessary
compile-shape gate; a cacheable classification alone is not complete remote
input, tool or execution authority. The native pool contract and
[ADR 0138](../../../adr/0138-distribute-compiler-units-through-native-build-goals.md)
own execution and trust decisions.

## Frontend contract and ownership

Add `lib/cc_unit_plan.ncl` following the existing Cargo unit-plan entry point.
Human-authored graph/policy is Nickel; exported JSON is deterministic runtime
input. The graph names a declared compiler family/toolchain and execution
system, target/sysroot, source/header roots, ordered argv/environment, generator
units, translation units, archive/link/install units, dependencies and named
outputs. Output references are typed unit references/placeholders, not guessed
paths or behavior hooks. All executable builders are declared store paths.

Put C-specific graph/argument validation and deterministic lowering in a
`no_std + alloc` frontend core, with a std producer/helper adapter following
the existing core/shell convention. Reuse generic dynamic-plan encoding,
identity and bounds; do not add a C branch to the scheduler/store or a parallel
remote API. Final package construction is an ordinary plan-root consumer.
The producer runs offline as a declared derivation and emits the standard plan,
sources and evidence outputs. No build-system probing or execution trace is
interpreted as a graph.

## Input and dependency model

Each compile declares its source content and every root that can affect header
search, including quote-relative search, ordered `-I`/`-isystem`/sysroot inputs,
forced includes, compiler resource directories and generated header outputs.
Materialization preserves the admitted source/include layout and search order.
Hash whole declared include trees, not only files mentioned by yesterday's
depfile: adding a previously absent header earlier in the search order must
change identity. Roots may be shared by units; conservative invalidation is
allowed and must not be advertised as minimal recompilation.

Generated files come from explicit generator actions whose output references
are direct compile inputs. A declared generated-file reference without its
edge is a frontend error. A dependency omitted from the manifest altogether
may instead fail in the sandbox compiler; admission does not claim to discover
all preprocessor dependencies from source. Neither case permits a worker-side
probe or client scratch-file fallback. Compile and link dependencies use normal
declared/reference closures. Every compiler,
assembler, linker, archiver, sysroot/runtime library and helper is a declared
immutable tool input; the driver must not substitute same-named host programs.
Input authority is independent of the optional local compiler cache.

Sources, generated outputs and imported trees must pass existing bounded
source/dynamic-plan admission. Only those inputs may be transferred. Relative
paths must resolve inside declared roots; absolute host paths, escape links,
undeclared search directories and environment-derived include/PATH discovery
fail before remote dispatch. No store or execution broker socket is mapped
into a compile sandbox.

## Supported compiler fragment

Begin with pinned admitted GCC-family C and C++ single-source compile-to-object
operations accepted by the existing classifier. The declared C++ runtime and
linker must match the toolchain. Preserve ordered compiler flags, language,
macros, include search, code-generation target and deterministic environment
in action identity. Worker-derived `-march=native`/equivalent host tuning is
not portable identity: reject it unless lowered to explicit admitted semantic
CPU requirements and flags. Unbound wall-clock macros/environment and
path-dependent artifacts must be rejected or qualified with explicit stable
inputs/remapping for that toolchain, never masked by path string rewriting.

Response files, unknown options, plugins, modules/PCH, coverage/profile output,
save-temps, undeclared side outputs and LTO are outside the first distributed
fragment. The explicit unit-plan request must fail with a typed blocker; it
must not forward an opaque command to a worker and call it distributed support.
The existing ordinary Make/local-driver route retains its own forward-unchanged
behavior. Do not globally narrow or change that route to implement this lane.

Dependency files, when requested, are declared named outputs with controlled
target/path semantics. Archive and link are separate explicit action kinds,
not passed through compile-only classification. Admit their ordered member/
object/library inputs and a supported deterministic archiver mode; no inferred
link flags or dependency order. Install/packaging writes only declared outputs.
Unsupported tool modes fail before publication rather than silently omit a
side output. Version the manifest if an incompatible extension becomes needed.

## Scheduling, identity and finalization

The producer lowers one ordinary derivation per action and standard plan-root
bindings. Independent objects can run on different workers; archives/links
wait for full current object verification. Final executables may link remotely
with their complete runtime/tool closure. Worker and attempt metadata do not
enter action identity. Changing source, header-tree presence/content/order,
generator, macro/flag, target, sysroot or compiler must invalidate affected
actions; moving the same concrete action between workers must not.

Use native signed PathInfo/action-result admission for all outputs. Do not
reuse the `mantle-cc-cache-request-v1` lookup/publication protocol as execution
or output-trust authority, and do not require its daemon for this lane. If a
compiler fails, the current compile goal fails and archive/link dependents
cannot run from an earlier object. Remote failures, cancellation and stale
fences follow the prerequisite generic goal contract.

## Behavioral acceptance

- Preserve `greet --version`, normal/formal greeting and count behavior, its
  installed library/header interface, and the default Make lane's separate
  baseline. Use identical source manifests; compare local/pool same-plan
  stable artifacts. Compare declared runtime/install semantics against Make,
  not assume binaries with different flags/layout must be identical.
- With real C and C++ compilers, overlap independent object executions on two
  worker identities and run the final programs; verify archive members and a
  linked consumer rather than merely check object files exist.
- Generate a header in one action and compile its consumer on another worker;
  change the generator output and observe the new runtime value.
- Add a higher-priority header, reorder include roots, edit a header, change a
  macro/source, replace a compiler/sysroot and move an unchanged action between
  workers. Inspect canonical action/input identities and independently expected
  runtime changes. Do not promise unaffected-header minimality for full trees.
- Reject cycles, missing generated-file edges, path escape/undeclared inputs,
  unknown modes/flags, wrong tool target and unbounded manifests. Cause a real
  compile error after a successful build; prove stale objects cannot satisfy
  the failed current goal or publish a new successful linked application.
- Repeat from clean stores with pinned tools and preserved output/admission
  evidence. Loopback proves integration only; final host qualification remains
  with `qualify-distributed-compilation`.

## Trade-offs and non-claims

Explicit graphs require recipe work and full include roots may over-invalidate
or transfer more than learned-file caching. This is preferable to unsound header
identity, ambient preprocessing or a second compiler scheduler. The lane is
not arbitrary build-system compatibility, remote cache trust, compiler
correctness, full bootstrap acceptance or release eligibility. Measurements
must report these costs for the declared fixtures without asserting a general
speedup.
