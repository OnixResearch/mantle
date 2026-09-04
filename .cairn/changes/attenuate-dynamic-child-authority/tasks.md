# Tasks

## Phase 1: Freeze the current boundary

- [ ] [serial] I1 Run and record the existing `crunch-build` dynamic admission, Worker dynamic discovery, execution-profile, and network-policy tests before core changes. r[dynamic_derivation_admission.authority_attenuation]
- [ ] [serial] I2 Accept or revise ADR 0119 before any Nix builder protocol or transport code enters a Mantle core surface. r[build_tool_boundary.builder_protocol_edge]

## Phase 2: Add pure authority attenuation

- [ ] [serial] I3 Add Mantle-owned parent-ceiling, child-request, admitted-decision, ordered-blocker, and BLAKE3 decision-identity types in a pure `dynamic_authority` module. r[dynamic_derivation_admission.authority_attenuation]
- [ ] [serial] I4 Project producer execution profiles and effective action policies into the parent ceiling without performing I/O. r[dynamic_derivation_admission.authority_attenuation]
- [ ] [serial] I5 Project native units and traditional generated derivations into child effect requests, including fixed-output network acquisition. r[dynamic_derivation_admission.authority_attenuation]
- [ ] [serial] I6 Route native plans and traditional `.drv` compatibility children through the same attenuation decision. r[dynamic_derivation_admission.authority_attenuation]
- [ ] [serial] I7 Require an admitted child profile and authority-decision identity for dynamic registry insertion. Remove silent profile defaults from both dynamic paths. r[dynamic_derivation_admission.authority_binding]
- [ ] [serial] I8 Recheck parent profile and decision identities before Worker mutation or dispatch. r[dynamic_derivation_admission.authority_binding]

## Phase 3: Harden the Nix compatibility edge

- [ ] [parallel] I9 Classify required system features from supported Nix ATerm and derivation-JSON inputs through a closed adapter table. r[foreign_derivation_import.required_system_features]
- [ ] [serial] I10 Reject `builder-rpc-v0`, malformed declarations, and unknown mandatory features before successful graph or package-index publication. r[foreign_derivation_import.required_system_features]
- [ ] [parallel] I11 Add the architecture guard that confines Nix, Varlink, descriptor-passing, daemon, and builder-RPC types to declared adapters and fixtures. r[build_tool_boundary.builder_protocol_edge]

## Phase 4: Test positive and negative behavior

- [ ] [parallel] V1 Add pure positive tests for equal authority, narrower authority, deterministic blocker order, and stable BLAKE3 decision identity. r[dynamic_derivation_admission.authority_attenuation]
- [ ] [parallel] V2 Add pure negative tests for network, setid, syscall, writable-prefix, host-path, substitution, shell, environment, and store-scope widening. r[dynamic_derivation_admission.authority_attenuation]
- [ ] [serial] V3 Add Worker tests that prove a rejected child leaves registry, goals, waiters, scheduler state, and fetch-adapter call count unchanged. r[dynamic_derivation_admission.authority_binding]
- [ ] [parallel] V4 Add native plan compatibility tests that preserve version-one wire bytes and bind `inherit` to the actual parent profile. r[dynamic_derivation_admission.authority_binding]
- [ ] [parallel] V5 Add positive and negative Nix 2.36 fixtures for empty, supported, `builder-rpc-v0`, unknown, malformed, and structured required-system-feature forms. r[foreign_derivation_import.required_system_features]
- [ ] [serial] V6 Run the architecture guard's positive fixture and every negative provider-leak fixture. r[build_tool_boundary.builder_protocol_edge]

## Phase 5: Document and validate

- [ ] [serial] V7 Update the dynamic-admission and Nix compatibility guides with migration behavior, stable reason classes, and explicit non-claims. r[dynamic_derivation_admission.authority_attenuation] r[foreign_derivation_import.required_system_features]
- [ ] [serial] V8 Run `nix develop -c cargo test -p crunch-build --lib dynamic`, `nix develop -c cargo test -p crunch-build --lib dynamic_authority`, `nix develop -c cargo test -p crunch-build --lib worker`, and the focused Nix producer tests. r[dynamic_derivation_admission.authority_binding] r[foreign_derivation_import.required_system_features]
- [ ] [serial] V9 Run `nix develop -c cargo fmt --check`, `nix develop -c cargo test --workspace`, `nix develop -c cargo clippy --workspace --all-targets -- -D warnings`, `./scripts/check-first-party-quality.sh`, `./scripts/check-first-party-tigerstyle.sh`, `git diff --check`, and `nix flake check -L`. r[build_tool_boundary.builder_protocol_edge]
- [ ] [serial] V10 Run `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .`, `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- tracey coverage --root .`, and all three gates for `attenuate-dynamic-child-authority`. r[dynamic_derivation_admission.authority_attenuation] r[foreign_derivation_import.required_system_features] r[build_tool_boundary.builder_protocol_edge]

## Phase 6: Complete lifecycle evidence

- [ ] [serial] R1 Record baseline, focused, negative, architecture, full-gate, and lifecycle transcripts under the change evidence directory. r[dynamic_derivation_admission.authority_binding]
- [ ] [serial] R2 Sync accepted requirements, archive the change, and rerun post-archive validation only after every implementation and validation task passes. r[build_tool_boundary.builder_protocol_edge]
