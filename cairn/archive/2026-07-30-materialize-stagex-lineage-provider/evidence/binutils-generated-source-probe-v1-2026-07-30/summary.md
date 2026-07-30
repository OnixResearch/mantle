# Binutils initial generated-source probe v1

Date: 2026-07-30

## Result

Mantle prepared the authenticated binutils 2.30 source, ran all nine configure classes, generated the initial BFD and intl sources, restored the authenticated BFD configure script, and completed the second BFD configure.

The exact ignored test passed in 128.16 seconds.

## Source preparation

The probe:

- repaired one `i386-init.h` dependency in each of two opcode Makefiles;
- saved the authenticated BFD configure script;
- removed exactly 31 selected release-generated files;
- removed exactly two BFD bootstrap input references for the first configure;
- replaced exactly 70 `/usr/bin/file` literals with explicit bounded authority;
- required `config.sub sun4` to emit `sparc-sun-sunos4.1.1`.

Six negative compiler-wrapper probes rejected:

- missing probe authority;
- a mismatched current directory;
- a source other than `conftest.c`;
- an unknown configure class;
- a source larger than 64 KiB;
- an exhausted 4,096-invocation budget.

None of these rejected probes wrote an accepted audit entry.

## Generated-source boundary

GNU Make built and executed BFD `chew` with explicit `CFLAGS_FOR_BUILD`. Mantle required the executable producer and semantic markers in all three BFD headers.

GNU Make ran the declared Bison 2.3 producer for `intl/plural.c`. Mantle required the Bison version marker.

Mantle restored the authenticated BFD configure script, reapplied only the bounded file-utility relocation, and required the second configure to produce a resolved `bfd-in3.h`.

The recursive BFD Make run emitted `make: the '-l' option requires a positive integral argument`. The command still exited successfully, and all independent producer and output checks passed. The evidence keeps this diagnostic.

## Identities

- BFD `chew`: `8df489a85fdb18b2bcff0e78f6fd5462ac2bf24b0c0b425ce742049c8f16cecd`
- `bfd-in2.h`: `c5958d72cad86a4cb1fc72f9701b651bfda9117972df943d4466dff91b6645f3`
- `libbfd.h`: `476e428708ea9d431a82e8b285fd0358550783ae66c329e1bf322be7acb5c124`
- `libcoff.h`: `f8aade524d94d1a701ca861b31a361dea27538a40823144a24e7736ba46b2a7a`
- `intl/plural.c`: `c96f79434b1bfdb3a59eae605233186145760d5661e744fac32621a1bb955f85`
- `bfd-in3.h`: `7a8b51a14f0914f8f425d0c21878670dbc2e86dc5290822062143ff2abbeb658`
- Authenticated BFD configure copy: `784e4147c4f954a34ce25f11baf280f77d0651f30436ba4054649ce0e628f1b3`
- This run's canonical sed audit: `e744dee6df31c310bea0ab737e7ecf09e6d0755a620f5b6651ce92f354f02ee9`

The run accepted 145 configure-preprocessor probes and 3,337 sed bridge invocations.

## Validation

- Generated-source integration test: passed.
- Focused unit tests: 10 passed.
- Strict first-party Clippy: passed. The existing vendor dead-code warning remained.
- `cargo fmt --check -p mantle -v`: passed.
- `git diff --check`: passed.
- Cached Cairn validation: passed.
- Cached Cairn proposal, design, and tasks gates: passed.

## Boundary

This probe does not add BFD `chew`, GNU Make children, Bison children, or generated outputs to the protected transition authorization graph.

It does not prove the remaining parser and scanner generation, full binutils compilation, binutils behavior, compiler correctness, provider normalization, or provider admission.
