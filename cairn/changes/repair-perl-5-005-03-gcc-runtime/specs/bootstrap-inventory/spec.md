## ADDED Requirements

### Requirement: Perl 5.005_03 GCC generator runtime is causally validated

r[bootstrap_inventory.perl_5_005_03_gcc_runtime] Mantle MUST promote `bootstrap/perl-5.005_03-gcc.ncl` as a source-built generator only when a bounded comparison identifies the smallest required correction and the resulting interpreter passes positive execution and negative parser behavior under its declared bootstrap inputs.

#### Scenario: precursor failure is not Perl evidence

GIVEN a canonical or diagnostic build fails in stage0, source acquisition, provider construction, store materialization, or another predecessor before the Perl builder runs
WHEN Mantle records the comparison outcome
THEN the outcome MUST identify the precursor and exact failure class
AND it MUST NOT classify the run as evidence for or against Perl optimization, ABI configuration, compiler-generation behavior, or runtime success.

#### Scenario: one-variable candidates preserve causal isolation

GIVEN optimization mode, LP64 size configuration, and compiler generation are plausible independent causes
WHEN Mantle compares diagnostic candidates
THEN each first-round candidate MUST vary only one mechanism while preserving the same Perl source, generated-header path, libc, binutils, bounded source list, and smoke contract
AND the evidence MUST record the command, derivation identity, terminal status, saved log, and first discriminating failure or success.

#### Scenario: viable generator passes positive and negative behavior

GIVEN a Perl 5.005_03 candidate builds and installs an interpreter
WHEN Mantle evaluates it for canonical promotion
THEN the interpreter MUST report version 5.005_03, execute the declared arithmetic program with the expected result, reject malformed Perl source with nonzero status and empty stdout, and have the declared ELF64 shape
AND compilation, installation, version output, or ELF inspection alone MUST NOT satisfy promotion.

#### Scenario: canonical correction is minimal and temporary variants are removed

GIVEN one or more candidates satisfy the full behavioral contract
WHEN Mantle updates the canonical derivation
THEN it MUST select the smallest evidence-backed semantic correction, preserve named ABI and smoke constants, and add deterministic regression coverage
AND temporary hidden diagnostic derivations MUST be deleted after their outcomes are preserved in lifecycle evidence.

#### Scenario: generator claims remain bounded

GIVEN the corrected Perl 5.005_03 interpreter passes its focused runtime contract
WHEN operators or downstream bootstrap stages cite the result
THEN the claim MUST identify the source, declared predecessor chain, store identity, runtime checks, and evidence location
AND it MUST NOT claim compiler correctness, normalized-provider admission, independent reproducibility, downstream Perl 5.6.2 success, or whole-bootstrap correctness without separate evidence.
