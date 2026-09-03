# Tasks: Thin the CLI composition root

## Phase 1: Application architecture

- [x] [serial] [depends:complete-store-capability-migration] [depends:separate-remote-build-hexagon] [depends:separate-rust-plan-hexagon] [depends:extract-build-planning-core] I1 Inventory every `main.rs` responsibility and map it to CLI parsing, runtime context, adapter selection, application dispatch, presentation, or an owning application capability. Reject unowned policy and hidden effects. r[application_architecture.thin_composition_root]
  - Evidence: `config/cli-application-architecture.ncl` maps every responsibility and limits `src/main.rs` to 450 lines. The checker measured 365 lines.
- [x] [serial] I2 Define capability-scoped application commands, results, blockers, effect plans, and ports for each command family. Keep contracts Mantle-owned and dependencies explicit. r[application_architecture.application_owned_ports]
  - Evidence: `mantle-application-core` owns typed commands, effects, observations, outcomes, and blockers. `mantle-application` owns 12 family ports.
- [x] [serial] I3 Move domain policy, trust decisions, state transitions, receipt construction, provider translation, filesystem traversal, process behavior, and effect orchestration out of `main.rs` into their owning cores, application operations, or adapters. r[application_architecture.thin_composition_root] r[application_architecture.effect_observation_boundary]
  - Evidence: Clap records moved to `src/cli_inbound.rs`. Compatibility operations moved to `src/cli_application.rs`. Typed adapters live under `src/cli_architecture/`.
- [x] [serial] I4 Split Clap DTO mapping from human and JSON presentation. Preserve accepted command syntax, output fields, diagnostics, redaction, and exit codes. r[application_architecture.thin_composition_root]
  - Evidence: the inbound and presentation adapters are separate. The 12-case CLI corpus is byte-identical, including both status-2 failures.
- [x] [serial] I5 Replace CLI-owned errors in cores and application ports with typed domain and capability errors. Map them to `RunError` only at the presentation boundary. r[application_architecture.typed_error_ownership]
  - Evidence: cores return `ApplicationCoreError`; ports return `ApplicationPortError`; the presentation adapter maps typed failures to `RunError`.
- [x] [serial] I6 Make each application operation execute typed effect plans through explicit ports and classify typed observations before terminal reporting. r[application_architecture.effect_observation_boundary]
  - Evidence: all 12 families dispatch through explicit ports. Family, command, effect, and failure bindings fail closed on drift.
- [x] [serial] I7 Add a maintained deterministic Rust architecture checker for core purity, port ownership, adapter direction, explicit composition, error ownership, presentation separation, root responsibilities, and declared no-std targets. r[application_architecture.dependency_guard]
  - Evidence: `scripts/check-cli-application-architecture.rs` reports zero repository findings and checks the declared host, WASM, root, port, adapter, and presentation boundaries.

## Phase 2: Verification

- [x] [parallel] V1 Add positive command tests for valid mapping, dependency wiring, dispatch, effect execution, human output, JSON output, and exit codes. Add negative missing-dependency, invalid DTO, domain blocker, adapter fault, wrong observation, and rendering-failure tests. r[application_architecture.thin_composition_root] r[application_architecture.typed_error_ownership] r[application_architecture.effect_observation_boundary]
  - Evidence: `evidence/validation-2026-09-03/focused-tests.log` records 108 passing focused tests, including two compile-fail cases and zero failures.
- [x] [parallel] V2 Add positive and negative architecture fixtures. Reject CLI, Snix, filesystem, process, async-runtime, environment, clock, random, network, provider, store-service, and rendering authority in cores or application contracts. r[application_architecture.application_owned_ports] r[application_architecture.dependency_guard]
  - Evidence: `evidence/architecture-validation.md` records 13 source-fixture diagnostics and two passing compile-fail fixtures.
- [x] [serial] V3 Run focused command parity tests for build, plan, Rust-plan, remote-build, store, source, release, project, bootstrap, and artifact families. Preserve exact output in `.cairn/changes/thin-cli-composition-root/evidence/command-parity.md`. r[application_architecture.thin_composition_root]
  - Evidence: both 12-case manifests have BLAKE3 `04993d806d317478dd5e4b6d5011e71afa158c2f572149f591605dfcbf9b6fa6`; the recursive diff is empty.
- [x] [serial] V4 Run the architecture checker, its positive fixture, and every negative fixture. Preserve exact diagnostics and dependency paths in `.cairn/changes/thin-cli-composition-root/evidence/architecture-validation.md`. r[application_architecture.dependency_guard]
  - Evidence: the maintained rail reports zero repository findings and names every rejected owner, authority class, and dependency path.
- [ ] [serial] V5 Run `nix develop -c cargo fmt --check -p mantle -v`, focused first-party Clippy with `-D warnings`, `git diff --check`, `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .`, `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate proposal thin-cli-composition-root --root .`, `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate design thin-cli-composition-root --root .`, `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate tasks thin-cli-composition-root --root .`, and `nix flake check -L`. r[application_architecture.dependency_guard]
  - Pending: commit the implementation, then record the committed-source checks and the independent full-flake boundary.
