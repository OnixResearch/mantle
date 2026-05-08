# V4 gcc-4.7 smoke status

Task-ID: V4
Covers: bootstrap.gcc47.runtime-validation
Captured: 2026-05-08T21:09:49Z

## Result

C, C++, and minimal C++11 smoke tests are blocked because V2 has no gcc-4.7 compiler output. The prerequisite gcc-4.0 archive is negative and records no `gcc-4.0.4` output; consequently `bootstrap/gcc-4.7.ncl` cannot provide `$out/bin/gcc`, `$out/bin/g++`, or a C++11-capable compiler to smoke.

Future promotion requires all of the following evidence in a follow-up repair slice:

1. gcc-4.0 emits a real compiler output;
2. gcc-4.7 builds with that provider and records transcript/provider/fallback fields;
3. C and C++ compile/link smokes pass;
4. a minimal C++11 smoke passes without host fallback.

No compiler-smoke success is claimed in this evidence.
