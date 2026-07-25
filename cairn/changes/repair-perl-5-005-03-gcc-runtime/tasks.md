## Phase 1: Causal isolation

- [ ] [serial] I1 Capture the canonical build under both the current store identity and the cached precursor identity, classify any failure before the Perl builder separately, and record the exact first Perl failure when reached. r[bootstrap_inventory.perl_5_005_03_gcc_runtime]
- [ ] [serial] I2 Run the bounded one-variable matrix for `-O0`, `LONGSIZE=8`, and final GCC 10 with the same source, generator, libc, binutils, and smoke contract; maintain a mechanism-level approach registry. r[bootstrap_inventory.perl_5_005_03_gcc_runtime]
- [ ] [serial] I3 Promote the smallest validated correction into `bootstrap/perl-5.005_03-gcc.ncl`, add deterministic positive and negative regression coverage, and remove the temporary diagnostic variants. r[bootstrap_inventory.perl_5_005_03_gcc_runtime]

## Phase 2: Verification

- [ ] [serial] V1 Rebuild the canonical Perl 5.005_03 derivation and prove version, arithmetic execution, malformed-source rejection, empty rejection stdout, and ELF64 shape from the produced artifact. r[bootstrap_inventory.perl_5_005_03_gcc_runtime]
- [ ] [serial] V2 Run source-pin and Nickel evaluation checks plus a focused downstream `bootstrap/perl-5.6.2-gcc.ncl` evaluation/build or record its exact next blocker without claiming downstream success. r[bootstrap_inventory.perl_5_005_03_gcc_runtime]
- [ ] [serial] V3 Run Cairn validation and proposal/design/tasks gates, sync and inspect the accepted requirement, archive the completed change, and commit exact evidence. r[bootstrap_inventory.perl_5_005_03_gcc_runtime]
