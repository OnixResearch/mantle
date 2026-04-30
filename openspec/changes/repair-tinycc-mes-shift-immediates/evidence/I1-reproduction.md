Task-ID: I1
Covers: bootstrap.part.tinycc.0.9.26.shift-immediates

# Mes-built TinyCC immediate-shift reproduction

Result: PASS. The bug is reproduced with a minimal object disassembly.

Compiler:

```text
target/live-part-tinycc-0-9-27/run-current/store/miymhdmqink0c81drn4y1f1chc73vdf9-tinycc-0.9.26/bin/tcc
```

Reproducer:

```c
unsigned f(unsigned x){ return x >> 8; }
unsigned g(unsigned x){ return x << 3; }
unsigned h(unsigned x){ return x & 31; }
```

Observed disassembly:

```text
f: shr $0x0,%eax
g: shl $0x0,%eax
h: and $0x1f,%eax
```

Conclusion: the Mes-built TinyCC 0.9.26 predecessor preserves normal immediate `and`, but emits zero-count immediate shifts. TinyCC 0.9.27 source-level workarounds cannot scale until this predecessor x86_64 codegen bug is repaired.

Full transcript: `evidence/I1-reproduction-full.log`.
