# ADR 0040: Validate StageX generated sources before second configure

## Status

Accepted (2026-07-30)

## Context

The binutils 2.30 recipe removes release-generated parser, scanner, table, header, and documentation outputs. It must then regenerate selected sources with authenticated GNU Make, TinyCC, Bison, Flex, M4, Gawk, sed, and shell artifacts.

A successful Make exit is not sufficient evidence. The BFD `headers` recipe can continue after its `chew` compiler fails. It can then create nonempty but incomplete headers and still exit successfully.

The first BFD configure also uses a bounded repair that removes two `bfd-in3.h:bfd-in2.h` bootstrap references. After the first header exists, the recipe must restore the authenticated configure script and run BFD configure again.

## Decision Drivers

- Preserve authenticated configure and Make semantics.
- Remove selected release-generated outputs before generation.
- Do not port Bison, BFD `chew`, or Make behavior into Rust.
- Reject false-green Make results.
- Keep configure preprocessing limited to declared classes and `conftest.c`.
- Keep the second BFD configure distinct from the first bootstrap configure.

## Decision

Mantle applies exact source preparation before configure:

- repair one `i386-init.h` dependency in each authenticated opcode Makefile;
- save the authenticated BFD configure script;
- remove exactly 31 selected release-generated files;
- remove exactly two BFD bootstrap input references;
- replace exactly 70 ambient `/usr/bin/file` literals with explicit utility authority;
- run authenticated `config.sub sun4` and require `sparc-sun-sunos4.1.1`.

The compiler wrapper has six negative probes. They reject missing authority, the wrong directory, a non-`conftest.c` source, an unknown configure class, a source larger than 64 KiB, and an exhausted 4,096-invocation budget.

Mantle passes `CFLAGS_FOR_BUILD` to configure and Make. This lets TinyCC build BFD `chew` against the declared native-musl headers.

After GNU Make reports success, Mantle requires:

- an executable BFD `chew` producer;
- semantic markers in `bfd-in2.h`, `libbfd.h`, and `libcoff.h`;
- the Bison 2.3 marker in `intl/plural.c`.

Mantle then restores the authenticated BFD configure script, reapplies only the explicit file-utility relocation, runs the second BFD configure, and rejects unresolved configured-type markers in `bfd-in3.h`.

## Alternatives Considered

### Keep release-generated sources

Rejected because it bypasses the declared source-built generators.

### Reimplement header and parser generation in Rust

Rejected because it would replace authenticated Make, Bison, and `chew` semantics.

### Trust Make exit status and file size

Rejected because the BFD recipe demonstrated a false-green path after `chew` failed.

### Keep the bootstrap BFD configure repair for the second pass

Rejected because `bfd-in2.h` exists after the first generation pass. The authenticated two-reference configure behavior must then run.

## Consequences

- The initial generated-source probe closes with 145 configure-preprocessor probes and 3,337 sed bridge invocations.
- The first BFD header pass, Bison parser pass, and second BFD configure complete over the declared artifacts.
- The current GNU Make emits a nonfatal `-l` option diagnostic during recursive BFD generation. Evidence preserves this diagnostic.
- Generated executables and generator child executions are still outside the protected transition authorization graph.
- This decision does not prove all parser/scanner generation, the full binutils build, binutils behavior, compiler correctness, or provider admission.
