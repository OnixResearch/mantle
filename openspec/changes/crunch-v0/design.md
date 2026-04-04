## Context

crunch is a build system that replaces Nix-the-language with Nickel while
reusing the Nix store protocol (store paths, NAR, content addressing) and
snix's build/store infrastructure. The goal for v0 is end-to-end: a `.ncl`
file goes in, a built store path comes out, with no Nix evaluator in the loop.

Reference code lives in sibling directories: `../snix/snix/` for the build
and store crates, `../nickel/` for the Nickel language.

## Goals / Non-Goals

**Goals:**
- Evaluate Nickel → construct Derivation → sandbox build → store result
- Vendor the minimum snix crates needed
- A working `crunch build hello.ncl` that produces a store path
- Nickel contracts that catch bad derivation descriptions at eval time
- Bootstrap from existing binaries (Nix store or static)

**Non-Goals:**
- Nixpkgs compatibility or package set
- Community adoption or ecosystem
- Nix language support of any kind
- Remote building, binary caches, substitution (future work)
- Multi-user store daemon (future work)
- IFD (import-from-derivation) — Nickel evaluation is pure

## Decisions

### 1. Nickel evaluation strategy: direct serde deserialization

**Choice:** Evaluate the `.ncl` file with `Program::eval_full_for_export()`,
then deserialize the resulting `NickelValue` directly into typed Rust structs
via serde. No JSON intermediate.

**Rationale:** `NickelValue` implements `serde::Deserializer`
(`nickel-lang-core/src/deserialize.rs`). This means we can `#[derive(Deserialize)]`
on our own Rust types and deserialize straight from the evaluated Nickel value:

```rust
#[derive(Deserialize)]
struct CrunchDerivation {
    name: String,
    builder: String,
    system: System,
    #[serde(default)]
    args: Vec<String>,
    #[serde(default)]
    inputs: Vec<CrunchDerivation>,
    fixed_output: Option<FixedOutput>,
    // ...
}

let value = program.eval_full_for_export()?;
let drv: CrunchDerivation = CrunchDerivation::deserialize(value)?;
```

Nickel's deserializer handles records → structs, arrays → Vec, enum
variants → Rust enums, Option for nullable/optional fields. The
`eval_full_for_export` path still does the heavy lifting:
- Forces all thunks (deep evaluation)
- Resolves all merge priorities and defaults
- Strips `not_exported` fields
- Validates all contracts

But instead of serializing to a JSON string and parsing it back, we
skip that round-trip entirely. The serde `Deserializer` impl on
`NickelValue` walks the value tree directly.

`crunch eval` still exports to JSON for human-readable debug output,
but the build pipeline never touches JSON.

**Alternative rejected:** JSON as intermediate format. Adds a
serialization round-trip for no benefit. We already depend on
`nickel-lang-core` for evaluation — using its serde integration
directly is strictly better.

**Implementation:** `crunch-eval` crate wraps `nickel_lang_core::program::Program`.
Returns `NickelValue`. `crunch-glue` defines `#[derive(Deserialize)]` structs
and deserializes from `NickelValue` directly.

### 2. Input tracking: explicit, not contextual

**Choice:** Derivations declare their inputs explicitly via an `inputs` field.
No string context tracking.

**Rationale:** Nix's string context is a source of complexity and subtle bugs.
It conflates "this string mentions a store path" with "this derivation depends
on that path." Making inputs explicit is simpler to implement, simpler to
reason about, and catches missing dependencies as build failures (sandbox
blocks access to undeclared inputs) rather than silently carrying hidden state.

**Alternative rejected:** Implementing NixString-style context tracking in
Nickel (as organist does with `NixString` contracts and fragment arrays).
This adds a complex tagged-string DSL to work around Nickel not having native
context. The ergonomic cost is high and the complexity infects every string
value in the system.

**Trade-off:** Users must list inputs manually. This is deliberate — the core
is minimal. External Nickel packages can provide builder templates that
pre-populate `inputs` via merge defaults, but that's a layer on top, not
baked into crunch. If a store path is interpolated into a string but not in
`inputs`, the sandbox rejects the build. Correct failure mode — loud and
immediate.

### 3. Vendoring strategy: copy and own

**Choice:** Copy required snix crates into `crunch/vendor/`. Modify freely.
No upstream tracking.

**Rationale:** snix is a moving target with its own priorities. Vendoring
gives us full control. The crates we need (`nix-compat`, `snix-build`,
`snix-castore`, `snix-store`) are relatively stable and well-defined. We
don't need snix-eval, snix-glue, or any CLI crates.

**What to vendor:**
- `nix-compat` (+ `nix-compat-derive`) — Derivation, StorePath, NAR, ATerm
- `snix-build` — BuildService, BuildRequest, sandbox (bwrap/OCI)
- `snix-castore` — BlobService, DirectoryService, Node types
- `snix-store` — PathInfoService, NarCalculationService, import utilities
- `snix-serde` — serialization helpers used by store crates
- `snix-tracing` — tracing/progress infrastructure

**What NOT to vendor:**
- `snix-eval` — replaced by nickel-lang-core
- `snix-glue` — replaced by crunch-glue (though we'll reference its code)
- `nix-daemon`, `nar-bridge`, all CLI crates — not needed for v0

**Implementation:** `cp -r` the crate directories, rename workspace
references to local paths, strip unused features.

### 4. Bootstrap: Nix store as seed, then static binaries

**Choice:** For v0, the simplest bootstrap is to reference paths from an
existing Nix store on the machine. A `seed.ncl` file maps tool names to
store paths, validated by the `StorePath` contract. Longer term, switch
to a self-contained static binary tarball.

**Rationale:** Every developer working on crunch already has Nix installed
(this is a Nix ecosystem project). Reusing existing store paths avoids
building a toolchain before we can build anything. The seed is just a
Nickel record of strings — no Nix evaluation happens.

**Implementation:**
```nickel
let { StorePath, .. } = import "crunch/lib.ncl" in

# seed.ncl — generated by `crunch bootstrap` or written by hand
# Contract validates all paths match /nix/store/ format
{
  bash      | StorePath = "/nix/store/xxxxxxxx-bash-5.2p37",
  gcc       | StorePath = "/nix/store/xxxxxxxx-gcc-wrapper-13.3.0",
  coreutils | StorePath = "/nix/store/xxxxxxxx-coreutils-9.5",
  gnumake   | StorePath = "/nix/store/xxxxxxxx-gnumake-4.4.1",
  binutils  | StorePath = "/nix/store/xxxxxxxx-binutils-wrapper-2.43.1",
}
```

To get these paths: `nix-build '<nixpkgs>' -A bash --no-out-link` etc.
No Nix evaluator runs at build time — we just point at pre-existing paths.
The `StorePath` contract catches typos and malformed paths at eval time.

### 5. Workspace structure

**Choice:** crunch is a Cargo workspace with internal crates plus vendored
crates.

```
crunch/
├── Cargo.toml              (workspace root)
├── src/main.rs             (CLI binary)
├── crates/
│   ├── crunch-eval/        (Nickel evaluation wrapper)
│   └── crunch-glue/        (Nickel record → Derivation)
├── vendor/
│   ├── nix-compat/
│   ├── nix-compat-derive/
│   ├── snix-build/
│   ├── snix-castore/
│   ├── snix-store/
│   ├── snix-serde/
│   └── snix-tracing/
├── lib/                    (Nickel stdlib)
│   ├── lib.ncl
│   ├── derivation.ncl
│   └── seed.ncl
└── tests/
    └── integration/
```

### 6. KnownPaths: reimplement, don't vendor snix-glue

**Choice:** Reimplement `KnownPaths` in crunch-glue rather than vendoring
all of snix-glue.

**Rationale:** snix-glue is tightly coupled to snix-eval's `Value` type and
`EvalIO` trait. The `KnownPaths` struct itself is ~150 lines and
straightforward. The builder translation (`derivation_into_build_request`)
is ~400 lines and mostly mechanical. Reimplementing both in crunch-glue,
adapted to work with JSON input, is less work than untangling snix-glue
from snix-eval.

### 7. Nickel language features: use them all

**Choice:** The stdlib is built on Nickel idioms, not on wrapping a dumb
JSON schema.

| Feature | How we use it |
|---|---|
| **Enum tags** | `system`, `HashAlgo`, `HashMode` are enums, not strings. Typos caught at contract time. |
| **Custom validators** | `StorePath`, `Name` contracts use `from_validator` with descriptive error messages. |
| **Closed record contracts** | `Derivation` is closed — extra fields rejected unless opened with `..`. |
| **`optional`** | `fixed_output \| FixedOutput \| optional` — present only for FODs. |
| **Pattern matching** | `system_to_string`, `hash_algo_to_string` dispatch on enum variants. |
| **`doc` annotations** | Every stdlib field has `\| doc "..."` for `nickel query` discoverability. |
| **Recursive records** | `env.APP_NAME = name` just works — Nickel records are recursive by default. |
| **Type annotations** | Helper functions use type signatures for static checking. |
| **serde Deserializer** | `NickelValue` implements `serde::Deserializer`. We deserialize directly into `#[derive(Deserialize)]` Rust structs. No JSON round-trip. Enum variants, optional fields, nested records all handled automatically. |

Features like **merge (`&`)**, **priorities (`default`, `force`)**, **`not_exported`**,
**partial records**, and **piecewise syntax** are part of the language and available
to users and external packages. The core stdlib doesn't impose patterns built
on them — it defines the derivation schema and gets out of the way. Builder
templates, build phases, and input set helpers are legitimate uses of these
features, but they belong in separate packages layered on top.

**Rationale:** Nickel's value over JSON/YAML comes from these features. The
core stdlib uses them for the derivation schema (enums, validators, contracts,
matching). External packages use the full set (merge, priorities, partial
records, not_exported) for builder templates and convenience layers. This
split keeps the core minimal while enabling a rich ecosystem on top.

### 8. Modularity: core ships schema, not opinions

**Choice:** crunch's core stdlib defines only the derivation contract and
primitive types. Builder templates, build phase abstractions, stdenv
equivalents, input set helpers, and module systems are NOT part of crunch.
They are separate Nickel packages.

**Rationale:** Nix's mistake was bundling everything — the language, the
package manager, the module system, stdenv, and nixpkgs conventions — into
one monolith. snix improved on this by splitting into separate crates. We
continue that principle into the Nickel layer: crunch is the build engine,
not the build framework.

**What this means concretely:**
- crunch ships `Derivation`, `StorePath`, `System`, `HashAlgo`, etc.
- A hypothetical `crunch-builders` package ships `bash_builder`,
  `mk_derivation`, build phases, input set helpers
- A hypothetical `crunch-modules` package ships a NixOS-style module system
- These are independent packages with their own repos and versioning
- crunch's test suite uses raw `Derivation` records, not convenience builders

**Alternative rejected:** Shipping builder templates in the core "for
convenience." This couples the core to opinions about how builds should
be structured. Once it's in the core, it's hard to remove.

**Evidence:** The Nickel modules blog (Tweag, June 2024) demonstrates that
a NixOS-style module system is a single line of Nickel:
`{ Module = { Schema | not_exported = {}, config | Schema } }`.
Merge + contracts + `not_exported` provides the full module pattern —
schema definition, config values, composition via `&`, LSP integration,
and precise error messages. This is not infrastructure that needs to be
baked in. It's a pattern that falls out of language primitives.

### 9. Nickel ecosystem position: clean break, not integration

**Choice:** crunch does not attempt to integrate with Nix, coexist with
nixpkgs, or transpile between languages. It uses Nickel as a standalone
configuration language feeding into the Nix store protocol.

**Rationale:** As of January 2026, the Nickel-in-Nix integration problem
remains unsolved. The Nickel team at Tweag has no plans to dedicate
substantial time to it (per yannham on NixOS Discourse, Jan 2026). The
community discussion focuses on pluggable evaluators, common bytecodes,
and transpilation — all approaches that try to make two languages coexist
in one evaluation. These are technically hard and politically fraught.

crunch sidesteps this entirely. Nickel evaluates to JSON. Rust converts
JSON to Derivation structs. snix-build executes them. No Nix evaluator
in the loop, no cross-language interface, no transpilation.

The Nix→Nickel transpiler (Tweag, Jan 2023) explored AST-level
translation but hit walls with `with`, `inherit`, and builtins. It exists
to consume nixpkgs from Nickel. We don't consume nixpkgs, so this work
is irrelevant to us.

Nickel's `nix_ffi` experimental feature (calling Nix from Nickel via
C++ FFI) is similarly irrelevant — it evaluates Nix eagerly and returns
plain data. We have no Nix code to call.

**What we gain by not integrating:** Simplicity. The entire system fits in
a few thousand lines of Rust + a few hundred lines of Nickel contracts.
No impedance mismatch between two evaluation models. No political
dependency on upstream Nix decisions about pluggable evaluators. And we
can make better technical choices (like BLAKE3 hashing) without needing
consensus from the Nix ecosystem.

### 10. BLAKE3 as default derivation hash

**Choice:** crunch uses BLAKE3 for derivation-level hashing instead of
Nix's SHA-256. The store path format stays the same
(`/nix/store/<nixbase32(hash[0:20])>-<name>`), only the hash function changes.

**Rationale:** We vendor nix-compat and don't need Nix store path compatibility.
BLAKE3 is already in our dependency tree (snix-castore uses it for content
addressing). It's faster than SHA-256, and using it everywhere gives us
consistency across all hashing levels:

| Level | Hash |
|---|---|
| Castore (blobs, directories) | BLAKE3 (inherited from snix-castore) |
| Derivation (ATerm → drv path) | BLAKE3 (changed from SHA-256) |
| Output path computation | BLAKE3 (changed from SHA-256) |
| FOD content hashes | User-specified (sha256, sha512, etc.) |

**What changes in vendored nix-compat:**
- `hash_derivation_modulo`: `Sha256::digest(aterm)` → `blake3::hash(aterm)`
- `build_output_path`, `build_text_path`, `build_store_path_from_fingerprint`:
  swap hash function
- SHA-256 remains available for FOD content hashes

**Trade-off:** crunch store paths differ from Nix store paths. A derivation
with identical parameters produces different paths in crunch vs Nix. This
means we cannot share binary caches with Nix, reuse Nix-built outputs as
crunch derivation inputs, or mix crunch and Nix in the same dependency
chain. Seed paths from Nix are unaffected — they're referenced as source
inputs by their original paths.

**Alternative rejected:** Keeping SHA-256 for Nix compatibility. We don't
need compatibility. Using SHA-256 when BLAKE3 is available and already in
the dependency tree is carrying legacy for no benefit.

### 11. Configurable store prefix, OS-agnostic core

**Choice:** The store prefix is configurable (default TBD — `/crunch/store`
or `/nix/store`). The hardcoded `STORE_DIR` in vendored nix-compat is
replaced with a runtime-configurable value. The core crates (eval, glue,
nix-compat) contain no `#[cfg(target_os)]`. Platform-specific code lives
only in `BuildService` implementations.

**Rationale:** Targeting only Linux would be repeating Nix's mistake. The
core of crunch — evaluating Nickel, constructing derivations, computing
store paths — is pure computation with no OS dependency. The only
platform-specific part is the sandbox that executes builds. snix-build
already abstracts this behind the `BuildService` trait with bwrap (Linux),
OCI (Linux), and gRPC (any OS) implementations.

By keeping the core OS-agnostic and the store prefix configurable, adding
a new platform (macOS, FreeBSD, OpenBSD, Redox) means writing one new
`BuildService` implementation. Everything else works unchanged. The gRPC
builder is always available as a cross-platform fallback — any platform
can delegate builds to a remote Linux box.

**Trade-off:** Changing the store prefix from `/nix/store` means seed
paths from a Nix installation don't share the prefix. This is fine —
`Input::Source` paths are referenced by their full path string regardless
of prefix.

### 12. WASM sandbox: future default, current option

**Choice:** v0 uses native sandboxes (bwrap on Linux) as the default.
WASM/WASI is available as an opt-in sandbox for single-process builds.
WASM becomes the default once WASI gains subprocess spawning.

**Rationale:** WASM is the ideal sandbox — memory-safe by construction,
capability-based I/O, cross-platform, deterministic. But standard WASI
(Preview 1 and 2) has no `fork`/`exec`/`spawn`. Build scripts that
shell out to gcc, make, cp — which is nearly all of them — cannot run.
WASIX (Wasmer's non-standard extension) adds process spawning but is
not standardized and has a thin ecosystem.

WASM currently works only for single-process builds: a Rust tool compiled
to `wasm32-wasip1` that reads inputs and writes outputs without spawning
anything. Useful for custom build tools, fetchers, and code generators.
Not useful for `bash -c "gcc foo.c && cp foo $out"`.

**v0 plan:**
- Default: bwrap on Linux (inherited from snix-build)
- Option: `sandbox = 'wasm` for single-process WASM builds
- Option: `sandbox = 'oci` for OCI containers
- Option: gRPC for remote builds (any OS)

**Future:** When WASI gets `proc_spawn` (on the standards roadmap),
WASM becomes viable as the general default. The `BuildService` trait
already abstracts the sandbox, so switching the default requires zero
changes to eval, glue, or store code.

**Alternative rejected:** Claiming WASM as default now. WASI lacks
process spawning. Nearly all builds need subprocesses. Honest about
current limitations, design for the future we want.

## Risks / Trade-offs

**[nickel-lang-core API stability]** → Nickel is post-1.0 (stable since May 2023),
with active development continuing (ADTs in Sept 2024, modules patterns in
June 2024). The `Program` and export APIs are the public interface.
Mitigation: pin to a specific Nickel version, update deliberately.

**[snix crate churn]** → Vendored crates diverge from upstream. Mitigation:
we own the vendor copies. If snix makes improvements we want, we cherry-pick.
We don't need to stay in sync.

**[Sandbox availability]** → snix-build uses bwrap (Linux only) or OCI.
macOS has no bwrap. Mitigation: v0 targets Linux only. macOS support is
future work (possibly via a different sandbox or no sandbox).

**[Bootstrap fragility]** → Referencing host Nix store paths is impure.
If the user garbage-collects those paths, builds break. Mitigation: document
`nix-store --add-root` to create GC roots for seed paths. The `StorePath`
contract at least catches invalid paths at eval time. Longer term, use a
fixed-output tarball that's re-fetchable.

**[Explicit inputs are verbose]** → Users must list every dependency.
Mitigation: builder templates pre-populate `inputs` with common tools via
merge defaults. Input set helpers bundle frequently-used combinations.
