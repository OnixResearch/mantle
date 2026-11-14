# 0149. Keep semantic and authenticated products in the execution closure

Status: Accepted

## Context

The source-built Mantle author records the selected Rust units and build-script
products as semantic roots. It published only the binary and smoke observation
as execution roots, leaving selected but otherwise unconsumed semantic products
outside the reachable action graph. Strict recipe admission correctly rejected
those declarations.

The ring PerlAsm authentication helper also published validated request bytes,
while the generator read a separate source-tree path. Its unused authentication
input was removed during normal input pruning. AWS used the same generator API
with an original source tree, but its executable Perl inputs require the
existing produced Perl-tree transformation that removes an assembler child
probe. A request record, source file and transformed tree are not interchangeable.

## Decision Drivers

- Execute the complete selected Cargo-free topology, not only the binary closure.
- Preserve strict reachability and unused-input admission rather than inventing
  dummy roots or unused dependency tokens.
- Make source authentication a dependency of the bytes the generator consumes.
- Preserve Perl logical pathname, basename selection and existing child-free
  source transformations.

## Decision

Mantle execution roots are the deterministically deduplicated selected semantic
products plus its binary and smoke observation. This does not root every helper
indiscriminately: a helper without a genuine consumer remains an admission error.

The shared PerlAsm author receives an exact produced artifact and relative
member for the executable script bytes. An empty member selects a produced file;
a nonempty member selects a file inside a produced source tree. The logical
source pathname remains a separate typed operand for Perl's `$0` and original
include/basename behavior. The generator opens the selected authenticated bytes;
the translator consumes its actual raw result transitively.

After validating the unchanged program, arguments, cwd and environment contract,
ring authentication publishes the original script bytes rather than the binary
request. AWS selects the existing produced Perl tree and its exact member,
retaining the assembler-probe cut and that tree's dependency on original source.
The old ambiguous internal argument is replaced at every caller; no marker
fallback, imported output or compatibility path is added.

## Consequences

- `source_built_mantle_programs` owns semantic execution-root selection;
  `source_built_mantle_programs_native` and the ring/AWS adapters own the script
  artifact/member contract.
- Normal unused-input pruning remains authoritative. Authentication is retained
  because a real operation reads its output, not because a map entry names it.
- Graph regressions must exercise selected-product reachability, rejection of
  unselected dummy obligations and authentication dependencies after pruning.
- Fresh source-program preparation and independent graph admission are required
  before a replacement cold launch. Mechanism tests or host Perl fixtures are
  not provider execution, a fixed point, compiler correctness or release evidence.
