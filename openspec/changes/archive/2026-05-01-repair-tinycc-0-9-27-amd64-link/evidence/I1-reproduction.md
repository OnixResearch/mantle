Task-ID: I1
Covers: bootstrap.compiler.tinycc.0.9.27.amd64.static-link
Status: reproduced

Diagnostic derivation:

    bootstrap/diag-tcc-link.ncl (scratch derivation, removed after capture)

Commands inside Crunch sandbox:

    tcc -v
    tcc -c hello.c
    chmod 644 hello.o
    tcc -static -o hello hello.o

Transcript:

    evidence/I1-reproduction-full.log

Result:

The Mes-linked `tinycc-0.9.27` output reports `tcc version 0.9.27 (x86_64 Linux)`, successfully compiles `hello.c` to `hello.o`, then exits 139 with `Segmentation fault (core dumped)` during the static link step.

This proves the `make-3.82-tcc` failure is blocked by a lower-level TinyCC 0.9.27 amd64 static-link defect, not by Make-specific source code alone.
