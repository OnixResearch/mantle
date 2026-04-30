# Design: GCC 4.0.4 transition stage

## Context

`bootstrap/gcc-4.0.ncl` is the first GCC stage in the mandatory ladder. It may
run only after `live-bootstrap-binutils-tcc-chain` produces TinyCC/musl/binutils
2.30 outputs. It must produce C and C++ compiler outputs for later gcc-4.7.4.

## Decisions

### 1. Consume only explicit chain outputs

`bootstrap/gcc-4.0.ncl` imports `bootstrap/binutils-tcc.ncl` and resolves only
its declared output plus prior TinyCC/musl toolchain outputs mounted by the
builder. The build script constructs `PATH` from those store paths and must not
probe `/usr`, host compiler paths, Nix commands, or the legacy musl.cc provider.
Any fallback marker other than `fallback-event=none` blocks completion.

### 2. Pin source and support artifacts at first consumption

The first consumer is `bootstrap/gcc-4.0.ncl`. It records this concrete source
selection:

- gcc 4.0.4 C/C++ full source URL `https://ftpmirror.gnu.org/gcc/gcc-4.0.4/gcc-4.0.4.tar.bz2`;
- SHA-256 SRI `sha256-kJLkxw84mjCJeH/VHgVVVfWq8LtTa4nRwHjy+e/hdf0=` from the checked-in placeholder for fetch compatibility;
- BLAKE3 digest for any crunch-owned carried patch/generated artifact;
- upstream path/commit provenance for any live-bootstrap-carried patch;
- first-consuming derivation metadata naming `bootstrap/gcc-4.0.ncl`.

### 3. Validate compiler behavior directly

Build success alone is insufficient. Validation compiles one C smoke source and
one C++ smoke source with the produced compiler binaries. Each transcript records
command, provider selection, exit status, output path or failure class, fallback
status/event marker, placeholder rejection result, and host-leakage audit result.

## Verification commands and artifacts

- `./scripts/check-bootstrap-source-pins.rs bootstrap/gcc-4.0.ncl` for source,
  patch, generated-artifact, digest, provenance, and first-consumer coverage.
- `crunch build bootstrap/gcc-4.0.ncl` for the build transcript saved to
  `openspec/changes/live-bootstrap-gcc-4-0-stage/evidence/V2-build.md`.
- `./scripts/check-bootstrap-transcript.rs --reject-host-tools openspec/changes/live-bootstrap-gcc-4-0-stage/evidence/V2-build.md`
  for the no-host audit, rejecting host compiler/libc/shell, Nix command,
  legacy provider path, and hidden fallback markers. If the transcript checker
  does not exist yet, this change must add it before V3 can pass.
- C smoke: compile and run a trivial `int main(void){return 0;}` program with
  the produced C compiler, recording metadata in
  `openspec/changes/live-bootstrap-gcc-4-0-stage/evidence/V4-compiler-smoke.md`.
- C++ smoke: compile and run a trivial program using the produced C++ compiler,
  recording metadata in the same evidence file.

## Risks / Trade-offs

**C++ scope vs gcc-core sources** → The stage must not accidentally pin only
`gcc-core`; it needs the C++ frontend/runtime source inputs required to produce a
working C++ compiler.

**Old compiler fragility** → GCC 4.0.4 may need live-bootstrap-carried patches or
generated files. Those artifacts are allowed only when pinned and provenance is
recorded.

**Host fallback temptation** → Configure scripts may find host tools unless PATH
and environment are tightly controlled. The no-host audit and fallback markers
are required evidence, not optional diagnostics.
