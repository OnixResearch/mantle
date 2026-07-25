## Phase 1: Causal isolation

- [x] [serial] I1 Capture canonical builds under the default and compatibility store identities, classify failures before the Perl 5.005_03 builder separately, and record the exact first source defect. r[bootstrap_inventory.perl_5_005_03_gcc_runtime]
  - Evidence: `evidence/validation.md` records host/sandbox, GCC, predecessor-Perl, and normalized-input failures without classifying them as target behavior.
- [x] [serial] I2 Repair the smallest predecessor boundary, then run target-local `-O0`, `LONGSIZE=8`, or GCC 10 variants only if the canonical target still fails the complete behavioral contract. r[bootstrap_inventory.perl_5_005_03_gcc_runtime]
  - Evidence: Perl 5.000 now consumes `gcc-generator-base-v4` through its relocated compiler seams; tasks `250`, `252`, `256`, and `261` prove the predecessor chain and unchanged canonical target succeed, so the registry rejects all three target variants as unnecessary.
- [x] [serial] I3 Add deterministic positive and negative regression coverage, correct the stale Perl 5.005_03 rejection diagnostic, and remove the temporary diagnostic variants. r[bootstrap_inventory.perl_5_005_03_gcc_runtime]
  - Evidence: `tests/bootstrap_eval.rs` preserves required/forbidden predecessor and runtime markers; pueue task `273` passed both focused tests, and no hidden Perl 5.005_03 diagnostic variant remains.

## Phase 2: Verification

- [x] [serial] V1 Rebuild canonical Perl 5.005_03 and prove version, arithmetic execution, malformed-source rejection, empty rejection stdout, and ELF64 shape from the produced artifact. r[bootstrap_inventory.perl_5_005_03_gcc_runtime]
  - Evidence: default-prefix build task `284` produced `/mantle/store/wx66fhkv2w4y275zqkl03kr2b9msl5s9-perl-5.005_03-gcc-v7`; direct runtime task `326` proved version 5.005_03, sum 42, nonzero malformed status, empty stdout, and nonempty stderr, while the successful builder enforced ELF64.
- [x] [serial] V2 Run source-pin and Nickel evaluation checks plus the focused downstream `bootstrap/perl-5.6.2-gcc.ncl` build under the same default store identity. r[bootstrap_inventory.perl_5_005_03_gcc_runtime]
  - Evidence: task `332` reported `2 files, 2 fetch blocks, 0 issues`; task `274` passed all 21 bootstrap evaluation tests; task `327` built `/mantle/store/6rxzzfl89lrvvwdpvqf4j0xqbww5d2by-perl-5.6.2-gcc-v19` with zero failures.
- [ ] [serial] V3 Run Cairn validation and proposal/design/tasks gates, sync and inspect the accepted requirement, archive the completed change, and commit exact evidence. r[bootstrap_inventory.perl_5_005_03_gcc_runtime]
