# Typed CMake/Ninja native projects

`crunch-cmake-core` owns bounded, effect-free validation and configure/build/install
planning. `crunch-cmake` checks admitted members, tool compatibility, executes the
plan, audits the staged tree and exports existing Cargo native-input roles.
`lib/cmake.ncl` is the typed authoring surface. The generated value is an ordinary
input-addressed derivation: no CMake scheduler branch, package resolver, implicit
fetcher, global installation or deployment authority is introduced.

This is a **bounded Linux adapter**, not universal CMake compatibility. A proof
is accepted only after `scripts/prove-cmake.py` finishes and emits `evidence.json`.
The checked-in driver and fixtures are not evidence that real tools ran.

## Supported contract

* Build platform: `x86_64-linux` or `aarch64-linux`. Target: Linux little-endian
  ELF64 `x86_64` or `aarch64`. GCC and Clang driver interfaces, GNU/LLVM ordinary
  non-thin ar archives, Ninja single-configuration builds, standard CMake
  configure/build/staged-install commands. No Windows/macOS, multi-config Ninja,
  Makefiles, CTest invocation, target deployment or target hardware execution.
* The source tree, CMake, Ninja, both compilers, linker, ar, ranlib, sysroot,
  compiler builtin/standard includes, dynamic runtime trees, dependency roots,
  optional toolchain fragment and optional emulator are **separate declared
  roles**. Roles name whole immutable store roots admitted by the existing
  signed PathInfo/source authority. A version string, an arbitrary filesystem
  path, or a caller's claimed digest is not admission. The complete tool/runtime
  closure must be supplied; no closure search or host loader fallback occurs.
* `Tool = {role, member, version}` and `Member = {role, member}` refer to safe
  relative members under these roots. Symlinks may resolve only inside the
  declared role closure. Tool first-line `--version` output must contain its
  explicitly pinned version; both compilers' `-dumpmachine` must exactly match
  the target triple (Clang receives `--target`). These compatibility checks
  supplement, and do not replace, immutable store identity.
* Proof cohort pins **CMake 3.31.8, Ninja 1.12.1**, the exact admitted C/C++
  compiler/linker/archive tools and their full roots in independently pinned
  plan identities. Its cross target is exactly `aarch64-unknown-linux-gnu` on
  x86_64 Linux. Compiler releases are explicit manifest inputs, not guessed
  defaults or a claim that every GCC/Clang release is compatible. Sources in
  `tests/fixtures/cmake/fixture-lock.json` are independently BLAKE3-pinned.
* `source_subdir` is relative (empty selects root). Configuration contains a
  bounded string map of project definitions; `CMAKE_*`, `FETCHCONTENT_*`, raw
  host paths, list injection and escaping role suffixes are rejected. A path
  value must be exactly `{{role:dependency}}[/safe/member]`.
* `build_targets` selects explicit Ninja targets; empty builds the project
  default. `outputs` lists required installed files. The complete installed
  tree is still checked, not merely the selected files. Supported top-level
  required paths are `include/`, `lib/`, `bin/`, and `share/`.

The source project remains executable build code. CMake-language guards are
compatibility controls, **not an alternative sandbox or a proof against malicious
source**. The ordinary Mantle sandbox must supply only the admitted closure and
must deny networking. Never run `mantle-cmake run` directly on a host as a substitute
for that sandbox. The adapter does not mount host `/usr`, `$HOME`, or `/etc`.

## Build versus target and host generators

All CMake configurations use an explicit system/toolchain, including native
architecture builds. Compiler probes use static-library try-compiles. `try_run`
requires an explicit admitted `emulator`; no emulator is inferred, even for native
architecture. Compiler wrappers remove execute permissions from produced target
artifacts during configuration/build. Direct target invocation fails; an emulator
that reads ELF bytes may execute them when explicitly selected. Installation
restores executable output permissions for consumers. An explicitly requested
post-build native smoke is different from implicit configuration-time execution.

`host_tools` contains only build-platform generators. Their executable directories
enter the controlled PATH. Target dependency `bin/` directories never enter PATH.
Generators that must execute during the build must be separately built/admitted
for the build platform; building a target executable and immediately executing it
as a generator is intentionally unsupported. Cross emulators requiring execution
bits on the target file rather than reading it are outside this adapter's bounded
support. No supplied fixture implicitly executes an AArch64 binary.

## Isolation and dependency acquisition

The runner uses a fresh `/build/mantle-cmake` configure tree, separate HOME/XDG/TMP,
`LC_ALL=C`, `TZ=UTC`, explicit SOURCE_DATE_EPOCH and `env_clear` for every subprocess.
It does not reuse CMakeCache, presets, user toolchain files, host CPATH/CFLAGS,
LD_LIBRARY_PATH, package registries or user pkg-config directories. Dynamic tools
must carry their real store-bound runtime interpreter/library references.

The generated toolchain fixes sysroot, compiler/linker/archive tools, Ninja,
include/library/package root modes, CMake dependency prefixes and permitted host
program paths. Compiler wrappers disable default C/C++ include search and add
only explicit `c_includes`/`cxx_includes` (C++ directories precede C directories
so standard-library `include_next` remains correct). GCC may still need an
explicitly admitted build-host assembler in `host_tools`; Clang's integrated assembler does
not. Compiler driver runtime/startup objects must be in the admitted compiler and
sysroot closure. No ambient `/usr` include/library path is mounted.

CMake package registries, environment package paths and system-prefix lookup are
disabled. CONFIG packages under `dependencies` are supported. Optional pkg-config
host tools see only declared dependency `lib/pkgconfig` roots, with the explicit
sysroot. Their `.pc` files must be authored for that sysroot convention; arbitrary
host `.pc` files and automatic pkg-config closure discovery are unsupported.

Dependencies must be acquired separately. `fetch_sources` maps a FetchContent name
to an already-admitted source role and sets `FETCHCONTENT_SOURCE_DIR_<UPPER_NAME>`.
FULLY_DISCONNECTED, UPDATES_DISCONNECTED and CMP0170 NEW reject missing populated
sources with CMake 3.31.8. The project-scope offline guard rejects
`file(DOWNLOAD/UPLOAD)` even if the caller requests a STATUS variable to swallow
errors. `file(LOCK)` is explicitly unsupported rather than silently changing
function-scope lock lifetimes through the guard's forwarding function.
`ExternalProject` is rejected: build it as a separate ordinary derivation and
supply its output role instead. Custom scripts that perform acquisition,
replace the isolation toolchain, or bypass CMake's supported project interface
are unsupported; network denial remains the non-bypassable execution boundary.

## Output and provenance

CMake configures with the logical store output as install prefix. Installation
uses DESTDIR under the private stage, then the runner copies only that prefix
into `$out`. Escaping/absolute installed symlinks, special files, thin archives,
wrong-architecture ELF files/archive members, missing selected files and reserved
provenance-path collisions fail. Bounds: 64 roles/entries per authoring collection,
131072 serialized plan bytes, 64 jobs, 100000 installed entries, 64 path components,
1024 installed path bytes, 256 MiB/file and 2 GiB/tree.

```
$out/include/...                       headers
$out/lib/...                           static/shared libraries and configs
$out/bin/...                           installed executable consumers
$out/share/...                         declared project data
$out/share/mantle/cmake-provenance.json plan, BLAKE3 identity, versions, artifacts
```

Every regular installed artifact has BLAKE3, size and executable mode. Relative
symlinks record their target and BLAKE3 of target bytes, not a misleading file-byte
hash. The manifest excludes itself (no recursive self-hash); normal output
PathInfo/NAR authority binds the complete output, including the manifest.

Unchanged admitted roots/configuration reuse the ordinary derivation result.
Source/tool/sysroot/runtime/dependency root or configuration changes change the
ordinary derivation identity. No separate CMake cache or mutable tool registry
exists. Store immutability is required: mutating bytes behind an admitted name
violates the existing store contract rather than producing a new input identity.

## Minimal Nickel authoring and Cargo consumption

Admission is deliberately outside this recipe. `admitted-cmake-inputs.ncl` is an
operator-authored record containing the actual role roots, relative tool members,
exact versions and complete plan; do not fill it with host paths or dummy hashes.

```nickel
let cmake = import "cmake.ncl" in
let admitted = import "./admitted-cmake-inputs.ncl" in
cmake.mkProject { plan = admitted.plan, runner = admitted.runner }
```

The same API is exported by the embedded stdlib as `CMake` after registration.
`plan` is exactly `crunch_cmake_core::Plan` (see its public fields); no private
adapter-only JSON shape is accepted. Explicit empty fields for the freestanding
fixture are `runtime_roles = []`, `dependencies = []`, `host_tools = []`,
`c_includes = []`, `cxx_includes = []`, `fetch_sources = {}`, `definitions = {}`,
`build_targets = []`, `toolchain_file = null`, `emulator = null`; its required
outputs are `["include/answer.h", "lib/libanswer.a", "bin/answer-bin"]`,
`build_type = "Release"`, `jobs = 1`, `source_date_epoch = 1` and
`source_subdir = "project"`. A real libc/C++ standard-library project must declare
its actual sysroot, standard includes and runtime closure instead of these empty
freestanding-fixture inputs.

Rust callers transfer ownership at the pure boundary: `validate(Plan)` returns
the same admitted plan, `admit(Plan)` additionally returns ordered resolved tool
and member paths, and `admit_runner(RunnerRequest { plan, runner })` returns the
plan with its builder path. `lower(LowerRequest { plan, output })` returns
`Lowering { admission, lowered }`; the shell executes only the returned lowering.
These are owned, effect-free values, not references or executable capabilities.
The adapter's `lower`, `build`, and `compile` likewise consume their plan; `lower`
also consumes its runner. These Rust ownership changes do not alter plan JSON,
derivation environments, ordinary admission, or provenance identities.

Export the admitted native output without inventing a Cargo discovery path:

```nickel
cmake.nativeInput {
  root = admitted_native_output,
  role = "answer",
  root_env = "ANSWER_ROOT",
}
# Result: { inputs = {answer = admitted_native_output},
#           env = {ANSWER_ROOT = "{{role:answer}}"}, path = [] }
```

This is the existing `NativeInputDeclaration`, keyed by exact Cargo package ID
under `cargo_unit_plan.ncl`'s `native_inputs`. Add explicit existing role markers
for consumer-specific include/lib/pkg-config variables as needed. The Rust helper
`crunch_cmake::native_input(root, role, root_env, store_prefix)` validates with the
same existing native-input resolver. Binaries/compilers/pkg-config are additional
roles, never inferred from a library output or ambient PATH.

## Executable proof and prerequisites

Run on the parent's admitted Linux scratch host, **after** building/registering
the new crates and embedding `cmake.ncl`. Do not run while sources change:

```sh
cargo test -p crunch-cmake-core --test validation
cargo test -p crunch-cmake --test artifacts
cargo test -p crunch-cmake --lib stage_tests
python3 scripts/prove-cmake.py --manifest /absolute/cmake-proof-inputs.json \
  --work /absolute/new-evidence-directory
```

The driver requires an explicit JSON object with schema
`mantle-cmake-proof-inputs-v1` and these fields (no defaults):

| Field | Required value |
|---|---|
| `mantle`, `adapter` | immutable built Mantle and mantle-cmake executables |
| `b3sum`, `readelf` | explicit build-host BLAKE3 and GNU readelf executables |
| `sandbox_shell` | declared static busybox accepted by the ordinary sandbox |
| `tool_path` | explicit directories for required host orchestration tools/bwrap |
| `nickel_module` | absolute source-bound `lib/cmake.ncl` path |
| `signing_key`, `trusted_public_key` | existing store admission/build trust inputs |
| `input_store` | physical export of the complete already-admitted input closure |
| `store_prefix` | logical prefix matching both plans and admitted state identities |
| `store`, `state_dir` | fresh proof outputs/state with all input roles already admitted |
| `clean_store`, `clean_state_dir` | independently prepared admitted state, no cached root |
| `native_plan`, `cross_plan` | absolute JSON files containing complete Plan values |
| `native_plan_blake3`, `cross_plan_blake3` | independently pinned canonical identities from `mantle-cmake validate PLAN` |
| `runner` | Tool record pointing at admitted `bin/mantle-cmake` in both plans' roles |
| `fixture_lock` | absolute checked-in `tests/fixtures/cmake/fixture-lock.json` |

Both plans' source role must admit the exact whole `tests/fixtures/cmake` tree,
with the same logical source root. The cross plan must declare a functioning
AArch64 toolchain, not a native GCC with a changed target label. An admitted empty
sysroot is sufficient **only** for these freestanding fixture sources. The driver
hashes each source member against the checked-in lock before building. It never
fetches tools, imports an untrusted source, substitutes results, searches a host
cache, discovers Nix closures or synthesizes missing SDK/sysroot inputs.

Expected observations: native C/C++ archive/header/executable, explicit native
executable exit 0, AArch64 ELF64 executable/archive inspected by independent
readelf and never run, replay cache hit, uncached independent clean rebuild with
identical artifacts/provenance despite poisoned ambient include/config variables,
configuration invalidation that changes the library, and failures for undeclared
packages, host search, direct target execution, file download, missing FetchContent,
ExternalProject, staged symlink escape, member escape, incompatible version and
incompatible target toolchain. Every command argv/status/stdout/stderr is retained;
only a fully successful run writes `evidence.json`.

## Normalization and limits of the claim

The compiler maps admitted source and private build prefixes to stable diagnostic
paths. SOURCE_DATE_EPOCH and archive zero-date settings are explicit; the fixture
suppresses ELF build IDs and uses no timestamps, libc or standard C++ runtime.
CMake emits deterministic input-addressed output prefixes for a fixed plan; the
independent-state comparison therefore uses the same logical install prefix.
Normal NAR identity ignores filesystem mtimes. No rewriting of arbitrary binaries,
CMake config files, generated data, random seeds or embedded timestamps is claimed.
Changing the logical store prefix, tool release, sysroot or source can legitimately
change output bytes. Passing the bounded proof does not establish global
reproducibility, compiler correctness, universal CMake support or release safety.
