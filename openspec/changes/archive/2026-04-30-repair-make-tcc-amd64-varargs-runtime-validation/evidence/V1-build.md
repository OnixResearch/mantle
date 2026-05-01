Task-ID: V1
Covers: bootstrap.part.make.3.82.amd64.runtime-validation
Status: completed with concrete blocker handoff

Command:

    timeout 1800 nix shell nixpkgs#bubblewrap -c ./target/debug/crunch build --no-substitute --store .crunch-drain/make-runtime-store --state-dir .crunch-drain/make-runtime-state bootstrap/make-tcc.ncl

Transcript:

    evidence/V1-build-full.log

Result:

    Exit status: 1

Summary:

The long-budget build no longer times out in prerequisites. It reaches the `make-3.82-tcc` derivation and records a concrete compiler/link blocker.

Findings:

- The original build reached GNU Make `main.c` and failed with TinyCC's varargs-corrupted diagnostic text for `initializer element is not constant`.
- Diagnostic tracing identified `main.c` as the failing source file.
- Source-level Make pass1 repairs in `bootstrap/make-tcc.ncl` now:
  - patch GNU Make's command-switch initializer entries that TinyCC/Mes rejects;
  - compile `main.c` with `-DNO_FLOAT` to avoid the floating initializer path;
  - force `glob/fnmatch.c` and `glob/glob.c` to emit `fnmatch.o` and `glob.o`;
  - normalize generated object permissions before linking.
- After those repairs, Make object compilation completes, but the final `tcc -static -o make ...` link still segfaults.
- A minimal diagnostic derivation reproduced the lower-level compiler issue with a trivial program:

    tcc -v
    tcc -c hello.c
    tcc -static -o hello hello.o

  The compile step succeeds, but the link step exits 139 with `Segmentation fault (core dumped)`.

Diagnostic transcript:

    evidence/V1-tcc-link-diagnostic.log

Handoff:

This is now deferred to OpenSpec change `repair-tinycc-0-9-27-amd64-link`, which scopes the TinyCC 0.9.27 amd64 static-link repair before Make runtime validation can honestly continue.
