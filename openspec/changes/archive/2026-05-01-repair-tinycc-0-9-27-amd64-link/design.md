## Context

The Make runtime-validation successor initially failed in GNU Make `main.c` and then advanced to the final link step after source-level Make pass1 hardening. A minimal diagnostic derivation showed the blocker is not specific to Make objects: the Mes-linked `tinycc-0.9.27` can compile `hello.c` but segfaults at `tcc -static -o hello hello.o`.

## Goals / Non-Goals

**Goals:**
- identify whether the failure is in TinyCC's implicit CRT/libc/libtcc1 lookup, archive loading, relocation handling, or Mes runtime support;
- patch the bootstrap source/build recipe rather than relying on host tools;
- prove both compile and link for a minimal executable before revalidating Make.

**Non-Goals:**
- redesigning the whole compiler chain;
- completing post-Make bootstrap stages.

## Decisions

### 1. Treat link as a TinyCC repair boundary

**Choice:** isolate the fix in a TinyCC repair change before claiming Make runtime validation.

**Rationale:** Make V1 now reaches the compiler link step, and a trivial hello link reproduces the segfault. Continuing to tweak Make sources would hide the real boundary.

**Alternative:** keep adding Make-specific link workarounds. Rejected because the trivial link diagnostic fails independently of Make.

## Validation Plan

1. Reproduce the trivial link failure with a scoped diagnostic derivation.
2. Patch `bootstrap/tinycc.ncl` or its source-normalization seam.
3. Build `bootstrap/tinycc.ncl` with Crunch and preserve the transcript.
4. Run a diagnostic derivation that compiles and statically links `hello.c`, then executes it.
5. Resume `repair-make-tcc-amd64-varargs-runtime-validation`.

## Risks / Trade-offs

**TinyCC diagnostics are varargs-corrupted** → use traced commands and minimal repros instead of relying on formatted filenames in TCC errors.

**Fix may belong in predecessor Mes/TCC runtime** → if TinyCC source patching cannot repair link without hiding a predecessor runtime bug, create a narrower predecessor repair and record the handoff evidence.
