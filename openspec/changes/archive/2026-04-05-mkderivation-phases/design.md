## Context

`mk_derivation.ncl` has a `make_script` function that emits phase
calls. When a phase field is present in attrs, it emits the user's
commands. When absent, it emits nothing. The spec requires defaults.

## Goals / Non-Goals

**Goals:**
- Default unpack, configure, build, install phases
- `mkShell` that produces a valid Derivation but fails if built
- `src` field wired to `$src` env var and `inputs` list
- Integration tests for default-phase builds on Linux

**Non-Goals:**
- Cross-compilation support
- Automatic dependency propagation (propagatedBuildInputs)
- Package search / resolution
- `crunch shell` command (mkShell just produces the record)

## Decisions

### 1. Default phases in make_script

**Choice:** When a phase field is absent from attrs (not in the
record at all), emit the default implementation. When present but
empty string, skip the phase. When present with content, use it.

Three states: absent (default), empty (skip), string (override).

**Implementation:** In `make_script`, change each `phase` call to
check `has_field` AND whether the value is empty:

```
let phase_or_default = fun name default =>
  if std.record.has_field name attrs then
    let val = std.record.get name attrs in
    if std.string.length val > 0 then val
    else ""  # empty string = skip
  else default
```

### 2. Default phase content

| Phase | Default |
|-------|---------|
| unpackPhase | `if [ -d "$src" ]; then cp -r "$src/." .; elif [ -f "$src" ]; then tar xf "$src"; fi; if [ -d */ ] && [ $(ls -d */ | wc -l) -eq 1 ]; then cd */; fi` |
| configurePhase | `if [ -x ./configure ]; then ./configure --prefix="$out"; fi` |
| buildPhase | `make -j${NIX_BUILD_CORES:-1}` |
| installPhase | `make install` |

The unpack phase handles both directory and tarball sources, and
enters the single subdirectory if one exists (standard tarball
layout). `NIX_BUILD_CORES` defaults to 1 if not set.

### 3. src wiring

**Choice:** When `src` is in attrs, add it to `inputs` and set
`src = <path>` in `env`. The unpack phase references `$src`.

**Implementation:** Already partially done — `src_inputs` extracts
the src and adds it to inputs. Need to also set `$src` in env.

### 4. mkShell

**Choice:** A function that takes `{ buildInputs, env, ... }` and
returns a Derivation with `builder = "<bash>/bin/bash"` and
`args = ["-c", "echo 'This derivation is not meant to be built'; exit 1"]`.

The key outputs are the `env` record and `inputs` list, which a
future `crunch shell` command can inspect to construct a dev
environment.

**Implementation:** In `mk_derivation.ncl`, add `mkShell` that
wraps `mk_derivation_with_shell` with a fixed `buildPhase` that
exits nonzero.

### 5. NIX_BUILD_CORES env var

**Choice:** Set `NIX_BUILD_CORES` in the environment to the
number of available CPUs (clamped to a reasonable max). For now,
hardcode "4" — runtime detection happens in the build script
itself since we can't query CPUs at eval time from Nickel.

## Risks / Trade-offs

**[Phase detection fragility]** The default configure phase checks
`-x ./configure`. This fails for projects using cmake, meson, etc.
Those projects must override `configurePhase`. This matches Nix's
stdenv behavior.

**[Tarball unpack assumptions]** The default unpack assumes standard
tarball layout (single top-level directory). Tarballs with multiple
top-level entries or no directory wrapper require a custom
`unpackPhase`.
