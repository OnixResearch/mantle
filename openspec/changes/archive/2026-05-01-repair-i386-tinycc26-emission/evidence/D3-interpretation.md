# D3 Interpretation: i386 TinyCC 0.9.26 emission boundary

Task-ID: D3

Covers: `bootstrap.i386-tinycc26-emission.decision`

## Result

The blocker is earlier than static linking or i386 runtime execution. The generated `tcc26-i386` starts and reports its configured i386 target, but it segfaults on both object-emission paths:

```text
version rc=0
assemble_object rc=139
compile_c_object rc=139
link_from_asm rc=139
link_from_object rc=127
run_i386-exit42-from-asm rc=127
run_i386-exit42-from-object rc=127
```

Evidence:

- `D2-derivation.log` records the Crunch builder transcript.
- `D2-installed-summary.txt` records the installed derivation summary.
- `D2-step-logs.md` records per-step stdout/stderr/rc files.
- `D2-host-gdb-trivial-registers.txt` is supplemental host-side debugging of the exported diagnostic compiler.

## Supplemental host-side observation

Running the exported `tcc26-i386` against a trivial C object outside the builder also segfaulted. GDB showed the crash in a `strlen`-shaped loop, with `rdi=0x800` / `rcx=0x800`, i.e. an invalid small integer was treated as a string pointer:

```text
Program received signal SIGSEGV, Segmentation fault.
0x0000000000443080 in ?? ()
rax            0x0                 0
rcx            0x800               2048
rdi            0x800               2048
#0  0x0000000000443080 in ?? ()
#1  0x0000000000000000 in ?? ()
```

## Decision

The next repair target is **TinyCC 0.9.26 i386-target object emission / host-pointer handling**, not Make 3.82, not i386 kernel execution, and not static linking. The compiler can be built and can print its version, but any attempt to process either assembly or C input crashes before producing an object.

The next slice should build a source-instrumented diagnostic variant of `tcc26-i386` with symbols/markers around the file-processing path (`tcc_add_file_internal`, preprocessing/parsing, and early section/symbol setup) and identify why a target value such as `0x800` is passed to a string routine.
