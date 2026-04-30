# Complete gcc-4.7 runtime validation

## Why

Parent change `live-bootstrap-gcc-4-7-stage` implemented the gcc-4.7.4 derivation surface, but its runtime validation depends on the deferred binutils-tcc and gcc-4.0 validation chain.

## What Changes

- Build `bootstrap/gcc-4.7.ncl` after prerequisite runtime validation is available.
- Record required transcript fields, no-host-leakage evidence, and C/C++/C++11 smoke test results.

## Scope

In scope: gcc-4.7.4 validation evidence and narrowly scoped fixes required to make validation complete.

Out of scope: gcc-10 and later stages.
