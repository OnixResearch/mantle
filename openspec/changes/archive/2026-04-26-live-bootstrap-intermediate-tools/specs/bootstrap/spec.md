# Intermediate Tools Specification

## Purpose

Defines requirements for the intermediate tool chain between tinycc-0.9.27
and gcc-4.0.4 in the live-bootstrap seed chain.

## ADDED Requirements

### Requirement: Intermediate tools build from tcc only

Each intermediate tool derivation MUST use only tinycc and previously built
intermediate tools as compilers and build tools.  No host compiler, host libc,
or host shell tools may appear in the sandbox.

#### Scenario: Tool builds without host leakage

- GIVEN a tcc-compiled make and tcc
- WHEN an intermediate tool derivation is built
- THEN the build succeeds using only chain-internal tools

### Requirement: musl-1.1.24 as libc boundary

musl-1.1.24 MUST be built using tcc.  All tools built after musl MUST link
against musl rather than mes libc.

#### Scenario: Post-musl tool links against musl

- GIVEN musl-1.1.24 built by tcc
- WHEN m4, flex, bison, or grep is built
- THEN the resulting binary links against musl libc

### Requirement: binutils-2.30 functional

binutils-2.30 MUST produce working `as`, `ld`, and `ar` binaries that
gcc-4.0.4 can use as its assembler and linker.

#### Scenario: Assembler produces object files

- GIVEN binutils-2.30 built from the intermediate chain
- WHEN `as` assembles a trivial `.s` file
- THEN a valid ELF object file is produced
