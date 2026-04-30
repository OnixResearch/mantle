## Context

The first TinyCC 0.9.27 repair proved that tcc-0.9.26's generated TinyCC 0.9.27 could be made to compile trivial C by avoiding power-of-two byte shifts in `x86_64-gen.c` byte emitters. GNU make's `getopt.c` still fails because larger sources exercise general x86_64 shift code and relocation paths.

Concrete reproducer:

```c
unsigned f(unsigned x){ return x >> 8; }
unsigned g(unsigned x){ return x << 3; }
```

Compiled with the current Mes-built `tinycc 0.9.26`, objdump shows:

```text
shr $0x0,%eax
shl $0x0,%eax
```

That means patching individual TinyCC 0.9.27 source shifts is not scalable. The predecessor compiler must emit immediate shift counts correctly.

## Goals / Non-Goals

**Goals:**

- Mes-built `tinycc 0.9.26` emits nonzero x86_64 immediate shift counts for constant shifts.
- The rebuilt TinyCC 0.9.27 compiles `hello.c` and GNU make `getopt.c` to non-empty objects.
- Malformed input remains a controlled nonzero failure, not a timeout or segfault.

**Non-Goals:**

- Full make execution proof.
- Full TCC conformance.
- Replacing Mes libc or the bootstrap compiler chain.

## Repair Strategy

1. Patch the TinyCC 0.9.26 x86_64 `gen_shift` immediate-count emission in `bootstrap/tinycc-mes.ncl` so constant shift counts are emitted as the actual immediate byte.
2. Validate the predecessor directly with a tiny shift-codegen object and `objdump`.
3. Rebuild TinyCC 0.9.27 and remove workaround pressure from downstream make sources.
4. If the repaired predecessor exposes a remaining TinyCC 0.9.27 varargs/path crash, apply only a narrow documented patch tied to that crash, such as replacing relocation-section `snprintf` with direct string construction.

## Risks / Trade-offs

- Fixing the predecessor shift emitter is riskier than patching one TinyCC 0.9.27 source expression, but avoids an unbounded series of source-level shift workarounds.
- A narrow TinyCC 0.9.27 varargs/path patch may still be needed because Mes libc varargs remain fragile; any such patch must name the exact crash it prevents.
- This change proves compiler readiness for make's first object boundary, not full GNU make runtime behavior.

## Validation Plan

1. Run source-pin audit for `bootstrap/tinycc-mes.ncl` and `bootstrap/tinycc.ncl`.
2. Build `bootstrap/tinycc-mes.ncl` and compile/disassemble the shift reproducer.
3. Build `bootstrap/tinycc.ncl`.
4. Smoke `tcc -version`, `tcc -c hello.c`, `tcc -c getopt.c`, and malformed-input failure with exit status that is nonzero, not timeout-derived, and not signal-derived.
5. Scan derivations and logs for undeclared `/usr/bin`, host compiler/binutils/libc paths, Nix commands, and undeclared `/nix/store/*-{gcc,binutils,glibc}` references.
