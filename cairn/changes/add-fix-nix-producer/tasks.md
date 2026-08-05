# Tasks: add fix Nix producer

## Phase 1: Baseline and source pinning

- [ ] [serial] I1 Run the existing foreign-derivation-import, direct-`.drv` producer, and source-record tests before any change. Record the baseline outputs. r[fix_nix_producer.validation]
- [ ] [serial] I2 Pin the `fix` upstream repository and exact revision as a fixed-output Mantle source record. r[fix_nix_producer.pinned_source]
- [ ] [parallel] I3 Add positive admission fixtures and negative hash-mismatch and floating-revision fixtures for the source record. r[fix_nix_producer.pinned_source]

## Phase 2: Toolchain and fix build

- [ ] [serial] I4 Add the pinned Zig binary toolchain as a fixed-output derivation input. r[fix_nix_producer.mantle_built_toolchain]
- [ ] [serial] I5 Add the `fix` build derivation with declared libcurl, libgit2, and pkg-config inputs, running `zig build --release=fast` in the Mantle sandbox. r[fix_nix_producer.mantle_built_toolchain]
- [ ] [serial] I6 Admit the built `fix` output with signed PathInfo and an artifact attestation. r[fix_nix_producer.mantle_built_toolchain]
- [ ] [parallel] I7 Add negative fixtures for host-toolchain leakage and undeclared build inputs. r[fix_nix_producer.mantle_built_toolchain]

## Phase 3: Producer adapter

- [ ] [serial] I8 Implement the adapter shell that runs `fix` evaluation and instantiation into a bounded output directory. r[fix_nix_producer.producer_adapter]
- [ ] [serial] I9 Route the output directory through the existing direct-`.drv` closure producer to emit `foreign-derivation-graph-v1` and `foreign-package-index-v1` artifacts. r[fix_nix_producer.producer_adapter]
- [ ] [serial] I10 Enforce the evaluation boundary: consumption paths must not invoke `fix`, Nix, Lix, or a daemon. r[fix_nix_producer.evaluation_boundary]
- [ ] [serial] I11 Add the bounded process policy with named wall-time, memory, and output limits plus owned teardown. r[fix_nix_producer.bounded_execution]
- [ ] [serial] I12 Enforce hash-domain separation at adapter identity validation. r[fix_nix_producer.hash_domain_boundary]
- [ ] [parallel] I13 Add negative fixtures for evaluation errors, malformed `.drv` output, oversized output, budget timeout, daemon-requiring commands, and wrong-domain digests. r[fix_nix_producer.validation]

## Phase 4: Evidence and documentation

- [ ] [serial] I14 Record bounded compatibility evidence with typed pins and agreement counts for the language suites and the nixpkgs differential. r[fix_nix_producer.compatibility_evidence]
- [ ] [serial] I15 Add stale-evidence marking when the `fix` or nixpkgs pin drifts from the recorded evidence. r[fix_nix_producer.compatibility_evidence]
- [ ] [serial] I16 Document the producer trust model, the bounded-evidence scope, the binary-Zig trust input, platform limits, and non-claims. r[fix_nix_producer.compatibility_evidence]

## Phase 5: Parity and validation

- [ ] [serial] I17 Add a parity fixture that instantiates one bounded expression through both the `fix` adapter and the host-Nix producer and compares root identities and graph structure. r[fix_nix_producer.producer_adapter]
- [ ] [serial] V1 Run the focused adapter, source-record, build-derivation, and foreign-import tests. Record exact outputs. r[fix_nix_producer.validation]
- [ ] [serial] V2 Run formatting and Clippy with warnings denied on touched crates, plus `git diff --check`. r[fix_nix_producer.validation]
- [ ] [serial] V3 Run `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .` plus the proposal, design, and tasks gates for this change. Record exact outputs before archive. r[fix_nix_producer.validation]
