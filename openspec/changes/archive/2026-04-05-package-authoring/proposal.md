## Why

crunch has a working build engine but every package is a wall of bash.
The hello-world example manually wires `$PATH`, interpolates store paths
into shell scripts, and handles output directory creation by hand. The
self-hosting example admits it doesn't work. There's no pattern for
"unpack source, run configure, run make, install to $out" — the workflow
that 95% of real packages follow.

Nix solved this with stdenv and mkDerivation: a set of shell functions
(phases) that run in order, with automatic PATH/CFLAGS/LDFLAGS propagation
from inputs. crunch needs the same thing in Nickel, adapted to crunch's
strengths (contracts, merging, no string contexts).

Without this, every package author reinvents the same boilerplate and
gets it wrong in different ways. With it, a C library build looks like:

```nickel
crunch.mkDerivation {
  name = "zlib-1.3.1",
  src = crunch.fetchTarball { url = "...", hash = "..." },
  build_inputs = [seed.gcc, seed.gnumake],
}
```

## What Changes

- **`mkDerivation` function in Nickel stdlib.** A higher-level builder
  that wraps the raw `Derivation` contract. Provides phases (unpack,
  patch, configure, build, install), automatic `$PATH` from
  `build_inputs`, and standard environment variables (`$src`, `$out`,
  `$CC`, `$prefix`).

- **`setup.sh` builder script.** A bash script that implements the
  phase system. `mkDerivation` sets `builder = seed.bash` and
  `args = ["-e", setup_script]`. The script sources phase functions
  and runs them in order. Each phase is overridable via the derivation
  record.

- **Input propagation.** `build_inputs` are added to `$PATH`.
  `propagated_build_inputs` carry forward to downstream dependents.
  C compiler and linker flags are wired automatically when gcc/clang
  are in `build_inputs`.

- **`mkShell` for dev environments.** A derivation that doesn't build
  anything but sets up an environment with the given inputs on `$PATH`.
  Used for interactive development.

- **Working self-hosting example.** `examples/crunch.ncl` updated from
  placeholder to a real build definition using `mkDerivation`. May not
  fully work yet (needs all Rust build deps in the seed) but the
  structure is real.

## Capabilities

### New Capabilities

- `mk-derivation`: higher-level package builder with phases and input
  propagation
- `setup-script`: bash phase runner (unpack/patch/configure/build/
  install/fixup)
- `mk-shell`: development shell builder
- `input-propagation`: automatic PATH/CFLAGS/LDFLAGS from build_inputs

### Modified Capabilities

- `nickel-stdlib`: `lib.ncl` gains `mkDerivation`, `mkShell`,
  `setup_hook`
- `examples`: hello-world rewritten to use `mkDerivation`, self-hosting
  example fleshed out

## Impact

- **Files**: `lib/mk-derivation.ncl` (new), `lib/setup.sh` (new),
  `lib/mk-shell.ncl` (new), `lib/lib.ncl` (re-exports),
  `examples/hello-world.ncl` (rewritten), `examples/crunch.ncl`
  (fleshed out), `examples/zlib.ncl` (new)
- **Dependencies**: None new (bash is a seed dep already)
- **Testing**: Eval-time tests that mkDerivation produces correct
  derivation records. End-to-end build tests with bwrap for a C
  hello-world via mkDerivation.
