# Tasks: host-tool-free first bootstrap

## Phase 1: Protected execution boundary

- [ ] I1 Add a pure `ProtectedExecPolicy` model that classifies allowed
      protected-phase executables from the stage0 inventory and rejects every
      undeclared host path. [covers=bootstrap.hosttoolfree.exec.boundary]
- [ ] I2 Define/update the stage0 executable seed inventory schema and source of
      truth with executable path, digest, provenance, allowed reason, and phase
      fields, and define the protected-phase allowlist of direct kernel
      interfaces that remain permitted before crunch-built tools exist.
      [covers=bootstrap.hosttoolfree.exec.boundary,bootstrap.hosttoolfree.sandbox.entrypoint]
- [ ] I3 Route self-build protected-phase process launches through a single
      shell adapter that records executable path, digest when available, reason,
      and policy decision. [covers=bootstrap.hosttoolfree.exec.boundary]
- [ ] I4 Replace protected-phase source staging helper use (`git`) with
      Rust-owned source staging logic. [covers=bootstrap.hosttoolfree.exec.boundary]
- [ ] I5 Replace protected-phase archive/copy helper use (`tar`, `cp`) with
      Rust-owned archive and copy logic. [covers=bootstrap.hosttoolfree.exec.boundary]
- [ ] I6 Replace protected-phase shell/cargo/helper execution (`sh`, `cargo`, ad
      hoc helper commands) with Rust-owned validation, vendored-input checks, or
      declared seed artifacts carrying inventory digests.
      [covers=bootstrap.hosttoolfree.exec.boundary]
- [ ] I7 Add unit tests for allowed stage0 crunch, allowed declared seed
      executable, declared seed digest mismatch rejection, missing required seed
      artifact fail-closed diagnostics, forbidden absolute host executable,
      forbidden PATH command execution, and each removed protected-phase helper
      family. [covers=bootstrap.hosttoolfree.exec.boundary,bootstrap.hosttoolfree.sandbox.entrypoint]

## Phase 2: Host-tool-free sandbox entry

- [ ] I8 Replace protected-mode host `bwrap` fallback with either a Rust-native
      sandbox entry point or a declared seed sandbox executable with digest
      validation. [covers=bootstrap.hosttoolfree.sandbox.entrypoint]
- [ ] I9 Add tests proving a declared seed sandbox executable with a matching
      digest starts successfully and is recorded as a declared seed artifact in
      proof/audit output. [covers=bootstrap.hosttoolfree.sandbox.entrypoint,bootstrap.hosttoolfree.proof.mode]
- [ ] I10 Add tests proving host `bwrap` on `PATH` is ignored/rejected in
      host-tool-free mode when no declared sandbox seed exists.
      [covers=bootstrap.hosttoolfree.sandbox.entrypoint]

## Phase 3: Proof rail

- [ ] I11 Extend `scripts/prove-self-hosting.sh` with a no-host-tools mode that
      poisons common host commands, launches only through the protected
      execution boundary, and preserves existing `self-build-proof:
      fallback-event=...` summary markers. [covers=bootstrap.hosttoolfree.proof.mode]
- [ ] I12 Extend `tests/self_hosting.rs` to run no-host-tools mode while
      preserving the existing stage1, stage2, busybox, and bwrap fixed-point
      checks. [covers=bootstrap.hosttoolfree.proof.mode]
- [ ] I13 Write `protected-exec-audit.json` into proof bundles and summarize the
      audit in `summary.txt`, including the machine-readable execution audit,
      stage0 inventory digest, blocked host command set, declared seed sandbox
      artifact records, and final result. [covers=bootstrap.hosttoolfree.proof.mode]
- [ ] I14 Add a no-host-tools proof failure fixture that injects an undeclared
      protected-phase executable, verifies the proof fails, and verifies the
      proof bundle identifies the hidden host tool.
      [covers=bootstrap.hosttoolfree.proof.mode]
- [ ] I15 Update README and `docs/bootstrap-stage0-inventory.md` with the
      protected-phase boundary, exact allowed kernel-interface list,
      predeclared seed-artifact rules, and no-host-tools proof claim.
      [covers=bootstrap.hosttoolfree.exec.boundary,bootstrap.hosttoolfree.proof.mode]

## Validation

- [ ] V1 Run `openspec validate host-tool-free-first-bootstrap --strict` and
      record the result. [covers=bootstrap.hosttoolfree.exec.boundary,bootstrap.hosttoolfree.sandbox.entrypoint,bootstrap.hosttoolfree.proof.mode]
- [ ] V2 Run fake-PATH runner tests proving `git`, `tar`, `cp`, `sh`, `cargo`,
      `bwrap`, and Nix commands are not invoked in the protected phase, declared
      seed sandbox acceptance/reporting succeeds, declared seed digest mismatches
      are rejected, missing required seed artifacts fail closed with diagnostics,
      and protected-phase behavior stays within the documented kernel-interface
      allowlist. [covers=bootstrap.hosttoolfree.exec.boundary,bootstrap.hosttoolfree.sandbox.entrypoint]
- [ ] V3 Run the full no-host-tools self-hosting proof and record proof bundle
      path, stage1/stage2/busybox/bwrap fixed-point status, protected execution
      audit summary, stage0 inventory digest, blocked host command set, declared
      seed sandbox artifact records, final result, and preserved
      `self-build-proof: fallback-event=...` markers.
      [covers=bootstrap.hosttoolfree.proof.mode]
- [ ] V4 Run the hidden-host-tool proof failure fixture and record that the
      proof fails while identifying the injected undeclared host tool in the
      proof bundle. [covers=bootstrap.hosttoolfree.proof.mode]
