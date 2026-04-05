## Context

crunch's Nickel stdlib has `Derivation` (raw contract), `fetchurl`,
`fetchTarball`, `fetchGit`, plus validators. Every example that builds
something writes raw bash with manual PATH wiring. The Rust engine
handles caching, CA derivations, parallel builds, partial failure, and
dynamic derivations — but the authoring experience is painful.

Nix's stdenv is ~4000 lines of bash across setup.sh, make-derivation.sh,
default-builder.sh, plus Nix-side mkDerivation. It handles phases,
hooks, propagated inputs, cc-wrapper, strip, patchelf, audit, and more.
We don't need all of that. We need the 20% that makes 80% of packages
writable: phases, PATH, and a few environment variables.

The Nickel stdlib already uses record merging and contracts. mkDerivation
is a function that returns a Derivation record with computed fields.
Nickel's merge system lets users override individual phases without
rewriting the whole builder — `{ configurePhase = "cmake ." }` replaces
just that one phase.

## Goals / Non-Goals

**Goals:**
- `crunch.mkDerivation { name, src, build_inputs }` produces a working
  build that unpacks source, runs configure+make+make install
- Phases are individually overridable via record fields
- `build_inputs` automatically appear on `$PATH`
- A C hello-world compiles and links via `mkDerivation` + seed gcc
- `crunch.mkShell` sets up a dev environment from inputs
- A `setup.sh` script that the sandbox runs, implementing phases

**Non-Goals:**
- cc-wrapper (automatic -I/-L flag injection from deps) — v2
- patchelf / RPATH fixup — v2
- Strip / audit phases — v2
- Cross-compilation — v2
- Propagated inputs that auto-wire CFLAGS — v2 (PATH propagation only for now)
- Hooks system (pre/post per phase) — v2
- Multiple outputs with split install — already supported by the Derivation
  contract, mkDerivation doesn't add new output logic

## Decisions

### 1. mkDerivation is a Nickel function, not a new Rust type

**Choice:** `mkDerivation` is a Nickel function in `lib/mk-derivation.ncl`
that returns a record satisfying `Derivation`. The Rust glue layer sees
a normal `CrunchDerivation` — no changes to the Rust pipeline.

**Rationale:** Nickel's merge system is the right tool for phase
overrides. A Rust-side `MkDerivation` type would duplicate Nickel's
record semantics in Rust. By keeping it in Nickel, we get contracts,
merging, and `nickel query` discoverability for free.

**Alternative rejected:** Rust-side mkDerivation struct with serde. Would
require teaching the glue layer about phases, and lose Nickel's merge
semantics.

### 2. setup.sh is a store path, not an inline script

**Choice:** `setup.sh` is embedded as a file in the crunch binary
(via `include_str!`) and written to a temp store path at build time.
The builder is `bash -e /nix/store/<hash>-setup.sh`. Each phase function
is defined in setup.sh and called in order.

**Rationale:** Inlining the entire phase runner into `args` makes
derivation ATerm hashes huge and hard to debug. A separate file is
inspectable (`crunch store info` shows the script path). The script
is deterministic (embedded at compile time) so the store path is stable.

Wait — embedding in the binary means the script must be written somewhere
the sandbox can see it. Two options: (a) write it to the output store
dir and add as an input source, or (b) just inline it in args.

**Revised choice:** Inline the setup script via `args = ["-e", script]`
where `script` is a Nickel string built from the phase definitions. This
avoids the store-path-for-setup problem entirely. The script is ~50 lines
for v1 — small enough to inline. Nickel's multiline strings (`m%"..."%`)
make this readable.

Nix does this too in the simple case: `genericBuild` is sourced from
`$stdenv/setup`, but the actual builder is `bash -e default-builder.sh`
which is 3 lines. Our inline approach is equivalent.

### 3. Phases: unpack, configure, build, install

**Choice:** Four phases for v1, each a bash function with a default
implementation. Users override by setting the corresponding field.

```
unpackPhase     → tar xf $src (if src is a directory, cp -r)
configurePhase  → ./configure --prefix=$out (if configure exists)
buildPhase      → make -j$NIX_BUILD_CORES
installPhase    → make install
```

**Phase execution order:**
```
unpackPhase → configurePhase → buildPhase → installPhase
```

Setting a phase to `""` (empty string) skips it.

**Rationale:** This covers autotools, plain Makefile, and cmake (with
`configurePhase = "cmake -B build -DCMAKE_INSTALL_PREFIX=$out"`) packages.
More phases (patch, fixup, check, strip) can be added in v2 without
breaking existing packages.

### 4. build_inputs → PATH

**Choice:** `build_inputs` is an array of inputs (store paths or
derivation records). mkDerivation adds them to `inputs` (for the
sandbox) and generates a `$PATH` that includes `<input>/bin` for each.

**Implementation:** mkDerivation's builder script starts with:
```bash
export PATH="<input1>/bin:<input2>/bin:..."
```

The Nickel function computes this from the `build_inputs` array. For
store path strings, the path is used directly. For derivation records,
the output path isn't known at eval time (CA derivations) — so we use
the derivation as an input and let the build system resolve the path
into `$PATH` at build time via the environment.

**Simplification for v1:** We only support store path strings in
`build_inputs` (seed packages). Derivation-as-build-input works for the
`inputs` field but doesn't auto-wire PATH — the user must set PATH
manually for deps built from source. This matches how the existing
examples work.

### 5. $src handling

**Choice:** mkDerivation takes a `src` field that is either:
- A store path string (from fetchTarball/fetchGit output)
- A derivation record (built first, output used as source)
- Absent (no unpack phase — custom build script handles everything)

The unpack phase does:
```bash
if [ -d "$src" ]; then
  cp -r "$src"/. .
elif [ -f "$src" ]; then
  tar xf "$src"
  # Enter the single top-level directory if there is one
  dirs=(*/); if [ ${#dirs[@]} -eq 1 ]; then cd "${dirs[0]}"; fi
fi
```

**Rationale:** This handles the common cases: a fetchTarball output is a
directory, a fetchurl .tar.gz output is a file. The "enter single subdir"
behavior matches Nix's unpackPhase.

### 6. mkShell: derivation that fails on build

**Choice:** `mkShell` produces a derivation record with
`builder = "/bin/sh"` and `args = ["-c", "echo mkShell is not buildable; exit 1"]`.
The point is the environment, not the output. Users enter the shell via
a future `crunch shell` command (not in this change) or by inspecting
the derivation's environment variables.

**Rationale:** Nix's mkShell works the same way — it's a derivation
that exists only for `nix-shell`/`nix develop` to extract environment
variables from. The build always fails.

For v1, mkShell just generates the correct environment variables and
`inputs` list. The `crunch shell` command is a separate change.

### 7. Environment variables set by mkDerivation

| Variable | Value | Notes |
|----------|-------|-------|
| `$src` | Store path of the source input | Only if `src` field is set |
| `$out` | Output store path | Already set by crunch glue |
| `$PATH` | `<build_input>/bin:...` | Computed from `build_inputs` |
| `$NIX_BUILD_CORES` | Number of CPU cores | For `make -j` |
| `$prefix` | Same as `$out` | Convention for `--prefix` |
| `$CC` | `gcc` | If gcc is in build_inputs (v1: just the binary name) |
| `$CXX` | `g++` | Same |

## Risks / Trade-offs

**[Inline script makes ATerm hashes input-sensitive]** The phase script
is part of the derivation's arguments, so any change to setup.sh changes
all derivation hashes. CA derivations mitigate this — output paths only
change if the actual output changes.

**[$PATH from seed paths is brittle]** We hardcode `<path>/bin` but
some packages install to `<path>/sbin` or `<path>/libexec`. Mitigation:
users can append to PATH in their build script. v2 adds proper search
path propagation.

**[No cc-wrapper means manual -I/-L flags]** Building something that
depends on zlib requires `-I <zlib>/include -L <zlib>/lib` in CFLAGS.
Nix's cc-wrapper handles this automatically. v1 requires the user to
set CFLAGS manually. Mitigation: document the pattern, add cc-wrapper
in v2.

**[mkShell without `crunch shell` is incomplete]** Users can't enter
the shell yet. The derivation record exists and the environment is
correct, but there's no CLI command to activate it. Mitigation: `crunch
shell` is a small follow-up change.
