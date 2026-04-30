## ADDED Requirements

### Requirement: Mes-built TinyCC emits x86_64 immediate shifts correctly
Crunch MUST build the Mes-hosted TinyCC 0.9.26 predecessor so x86_64 constant shift expressions emit nonzero immediate shift counts.
ID: bootstrap.part.tinycc.0.9.26.shift-immediates

The predecessor compiler MUST compile a shift reproducer containing `x >> 8` and `x << 3` to an object whose disassembly contains `shr $0x8` and `shl $0x3` (or equivalent nonzero immediate encodings). The rebuilt TinyCC 0.9.27 MUST then compile both a trivial C source and GNU make 3.82 `getopt.c` to non-empty objects, and malformed C MUST fail with a controlled nonzero status rather than a timeout or signal. Evidence MUST include source-pin audit, build transcript, shift disassembly transcript, object compile transcripts, malformed-input transcript, and host-leakage scan.

#### Scenario: Shift reproducer emits nonzero immediates

- GIVEN the repaired `bootstrap/tinycc-mes.ncl` output
- WHEN it compiles `unsigned f(unsigned x){ return x >> 8; }` and `unsigned g(unsigned x){ return x << 3; }`
- THEN object disassembly contains nonzero immediate shift counts for both functions

#### Scenario: TinyCC 0.9.27 compiles make getopt

- GIVEN `bootstrap/tinycc.ncl` has been rebuilt from the repaired predecessor
- WHEN the compiler compiles GNU make 3.82 `getopt.c`
- THEN the command exits successfully
- AND the produced object is non-empty
