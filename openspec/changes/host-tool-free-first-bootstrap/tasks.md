# Tasks: host-tool-free first bootstrap

## Phase 1: Protected execution boundary

- [ ] I1 Add a pure `ProtectedExecPolicy` model that classifies allowed
      protected-phase executables from the stage0 inventory and rejects every
      undeclared host path, forbidden Nix command, disallowed seed role,
      missing provenance category, and non-BLAKE3 digest lacking an
      interoperability reason. [covers=bootstrap.hosttoolfree.exec.boundary]
- [ ] I2 Add `bootstrap/stage0-inventory.ncl` as the typed Nickel source
      inventory plus a Rust `Stage0Inventory` validation entry point. The model
      must cover executable seed entries and non-executable source entries with
      schema version, stable id, role, phase, absolute executable path for
      executable entries, URLs, extraction rules, owner, provenance
      category/text, allowed reason, required flag, digest algorithm/hex,
      optional interoperability reason, and phase/role fields.
      [covers=bootstrap.hosttoolfree.exec.boundary]
- [ ] I3 Add the generated proof inventory helper that writes
      `target/host-tool-free-stage0/stage0-inventory.ncl` only from explicit
      `CRUNCH_STAGE0_SEED_SANDBOX_ENTRY`, `CRUNCH_STAGE0_SEED_SANDBOX_SHELL`,
      `CRUNCH_STAGE0_SEED_TOOLCHAIN_ROOT`, and explicit bootstrap-build-tool
      inputs, expands each executable under the toolchain/build-tool inputs into
      per-executable `bootstrap-toolchain-tool` or `bootstrap-build-tool`
      entries with paths/digests/provenance, computes BLAKE3 digests, and never
      searches `PATH` or `/nix/store`.
      [covers=bootstrap.hosttoolfree.exec.boundary,bootstrap.hosttoolfree.proof.mode]
- [ ] I4 Route protected-phase network fetches through `Stage0Inventory` source
      URL allowlisting and extraction-rule validation so undeclared source URLs,
      missing source entries, digest mismatches, and missing provenance fail
      closed before fetch or extraction. [covers=bootstrap.hosttoolfree.exec.boundary]
- [ ] I5 Route self-build/proof protected-phase process launches through one
      non-shell protected process-launch adapter that enforces
      `ProtectedExecPolicy` before crunch-owned `execve` calls and records
      executable path, digest, reason, phase, inventory entry, and policy
      decision. [covers=bootstrap.hosttoolfree.exec.boundary]
- [ ] I6 Replace protected-phase source staging helper use (`git`) with
      Rust-owned source staging logic. [covers=bootstrap.hosttoolfree.exec.boundary]
- [ ] I7 Replace protected-phase archive/copy helper use (`tar`, `cp`) with
      Rust-owned archive and copy logic. [covers=bootstrap.hosttoolfree.exec.boundary]
- [ ] I8 Replace protected-phase shell/cargo/helper execution (`sh`, `cargo`, ad
      hoc helper commands) with Rust-owned validation, vendored-input checks, or
      declared seed artifacts carrying inventory digests.
      [covers=bootstrap.hosttoolfree.exec.boundary]
- [ ] I9 Add unit tests for allowed stage0 crunch, allowed declared seed roles,
      declared seed digest mismatch rejection, missing required seed artifact
      fail-closed diagnostics, forbidden absolute host executable, forbidden
      PATH command execution, forbidden Nix seed entries even with matching
      digests, non-executable source URL allowlisting, extraction-rule
      validation, provenance-category rejection, non-BLAKE3 interoperability
      validation, and each removed protected-phase helper family.
      [covers=bootstrap.hosttoolfree.exec.boundary,bootstrap.hosttoolfree.sandbox.entrypoint]

## Phase 2: Host-tool-free sandbox entry

- [ ] I10 Replace protected-mode host `bwrap` fallback with the declared seed
      `sandbox-entry` executable and declared `sandbox-shell`, both validated by
      inventory digest/provenance before use. [covers=bootstrap.hosttoolfree.sandbox.entrypoint]
- [ ] I11 Add Linux seccomp user-notification exec supervision for the declared
      seed sandbox and descendants. The supervisor must trap `execve`/`execveat`
      before execution, validate path/digest against `Stage0Inventory`, append
      audit events, and fail closed when supervision, filter inheritance, or
      path resolution is unavailable. [covers=bootstrap.hosttoolfree.exec.boundary,bootstrap.hosttoolfree.sandbox.entrypoint]
- [ ] I12 Add tests proving a declared seed sandbox executable with a matching
      digest starts successfully and is recorded as a declared seed artifact in
      proof/audit output. [covers=bootstrap.hosttoolfree.sandbox.entrypoint,bootstrap.hosttoolfree.proof.mode]
- [ ] I13 Add tests proving host `bwrap` on `PATH` is ignored/rejected in
      host-tool-free mode when no declared sandbox seed exists, and proving
      seccomp/supervisor-unavailable conditions fail closed before sandbox
      startup. [covers=bootstrap.hosttoolfree.sandbox.entrypoint]

## Phase 3: Proof rail

- [ ] I14 Implement the protected-phase transition event: after the exact
      crunch-built `bootstrap/bwrap.ncl` and `bootstrap/busybox.ncl` outputs
      are built, export them, verify them against recorded output metadata,
      select them as the later-stage sandbox entry and sandbox shell, and record
      the transition in the execution audit before relaxing protected-phase seed
      execution rules. [covers=bootstrap.hosttoolfree.exec.boundary,bootstrap.hosttoolfree.sandbox.entrypoint]
- [ ] I15 Extend `scripts/prove-self-hosting.sh` with `--no-host-tools` and
      `--stage0-inventory <path>` so the proof poisons common host commands,
      launches only through the protected execution boundary, and preserves
      existing `self-build-proof: fallback-event=...` summary markers.
      [covers=bootstrap.hosttoolfree.proof.mode]
- [ ] I16 Extend `tests/self_hosting.rs` to run no-host-tools mode while
      preserving the existing stage1, stage2, busybox, and bwrap fixed-point
      checks. [covers=bootstrap.hosttoolfree.proof.mode]
- [ ] I17 Write `protected-exec-audit.json` into proof bundles and summarize the
      audit in `summary.txt`, including the machine-readable execution audit,
      stage0 inventory digest, blocked host command set, declared seed sandbox
      and shell/toolchain artifact records, fallback-event markers, and final
      result. [covers=bootstrap.hosttoolfree.proof.mode]
- [ ] I18 Add a no-host-tools proof failure fixture that injects an undeclared
      protected-phase executable or child exec, verifies the proof fails, and
      verifies the proof bundle identifies the hidden host tool.
      [covers=bootstrap.hosttoolfree.proof.mode]
- [ ] I19 Update README and `docs/bootstrap-stage0-inventory.md` with the
      protected-phase boundary, exact allowed kernel-interface list, inventory
      schema/path, predeclared seed-artifact rules, seccomp supervisor behavior,
      and no-host-tools proof claim.
      [covers=bootstrap.hosttoolfree.exec.boundary,bootstrap.hosttoolfree.proof.mode]

## Validation

- [ ] V1 Run `openspec validate host-tool-free-first-bootstrap --strict` and
      record the result. [covers=bootstrap.hosttoolfree.exec.boundary,bootstrap.hosttoolfree.sandbox.entrypoint,bootstrap.hosttoolfree.proof.mode]
- [ ] V2 Run fake-PATH and inventory validation tests proving `git`, `tar`,
      `cp`, `sh`, `cargo`, `bwrap`, and Nix commands are not invoked in the
      protected phase, declared seed sandbox/shell/toolchain acceptance and
      reporting succeeds, declared seed digest mismatches are rejected, missing
      required seed artifacts fail closed with diagnostics, non-executable
      source URLs and extraction rules are validated, provenance categories are
      enforced, and protected-phase behavior stays within the documented
      kernel-interface allowlist. [covers=bootstrap.hosttoolfree.exec.boundary,bootstrap.hosttoolfree.sandbox.entrypoint]
- [ ] V3 Run seccomp supervisor tests proving child `execve`/`execveat` events
      are denied before execution when undeclared or mismatched, and proving
      supervisor-unavailable/filter-inheritance/path-resolution failures are
      fail-closed. [covers=bootstrap.hosttoolfree.exec.boundary,bootstrap.hosttoolfree.sandbox.entrypoint]
- [ ] V4 Run the full no-host-tools self-hosting proof and record proof bundle
      path, stage1/stage2/busybox/bwrap fixed-point status, protected execution
      audit summary, stage0 inventory digest, blocked host command set,
      declared seed sandbox/shell/toolchain artifact records, final result, and
      preserved `self-build-proof: fallback-event=...` markers.
      [covers=bootstrap.hosttoolfree.proof.mode]
- [ ] V5 Run the hidden-host-tool proof failure fixture and record that the
      proof fails while identifying the injected undeclared host tool in the
      proof bundle. [covers=bootstrap.hosttoolfree.proof.mode]
