## ADDED Requirements

### Requirement: i386 TinyCC 0.9.26 object emission repair [r[bootstrap.i386-tinycc26-object-emission.repair]]

Crunch MUST repair the i386 TinyCC 0.9.26 proof so the generated x86_64-hosted/i386-targeting compiler can emit object files without segfaulting.

#### Scenario: Object emission succeeds [r[bootstrap.i386-tinycc26-object-emission.repair.object]]

- GIVEN the generated `tcc26-i386` diagnostic compiler
- WHEN it compiles assembly or C input with `-c`
- THEN it MUST produce an object file without a segmentation fault.

### Requirement: i386 TinyCC 0.9.26 runtime proof [r[bootstrap.i386-tinycc26-object-emission.runtime-proof]]

Crunch MUST prove the repaired i386 TinyCC 0.9.26 path can produce and execute a no-libc i386 ELF before using it as a basis for later i386 bootstrap stages.

#### Scenario: i386 exit42 runs [r[bootstrap.i386-tinycc26-object-emission.runtime-proof.exit42]]

- GIVEN `tcc26-i386` can emit objects
- WHEN it links the no-libc `_start` smoke executable
- THEN the produced i386 ELF MUST execute inside Crunch's sandbox with exit code 42.
