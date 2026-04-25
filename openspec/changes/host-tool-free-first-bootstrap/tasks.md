# Tasks: host-tool-free first bootstrap

## Phase 1: Protected execution boundary

- [x] I1 Add a pure `ProtectedExecPolicy` model that classifies allowed ✅ 3m 51s (started: 2026-04-25T21:20:12Z → completed: 2026-04-25T21:24:03Z)
      protected-phase executables from the stage0 inventory and rejects every
      undeclared host path, forbidden Nix command, disallowed seed role,
      missing provenance category, and non-BLAKE3 digest lacking an
      interoperability reason. [covers=bootstrap.hosttoolfree.exec.boundary]
      Evidence: pueue task 45 passed `cargo test -p crunch protected_exec -- --nocapture`
      with 8 protected-exec tests, covering allowed declared seeds,
      undeclared path rejection, digest mismatch rejection, forbidden Nix
      executable rejection, provenance rejection, relative path rejection,
      non-BLAKE3 interoperability enforcement, and source URL allowlisting.
- [x] I2 Add `bootstrap/stage0-inventory.ncl` as the typed Nickel source ✅ 3m 51s (started: 2026-04-25T21:20:12Z → completed: 2026-04-25T21:24:03Z)
      inventory plus a Rust `Stage0Inventory` validation entry point. The model
      must cover executable seed entries and non-executable source entries with
      schema version, stable id, role, phase, absolute executable path for
      executable entries, URLs, extraction rules, owner, provenance
      category/text, allowed reason, required flag, digest algorithm/hex,
      optional interoperability reason, and phase/role fields.
      [covers=bootstrap.hosttoolfree.exec.boundary]
      Evidence: `src/protected_exec.rs` defines `Stage0Inventory`, executable
      entries, source entries, digest/provenance validation, and source URL
      allowlisting; `bootstrap/stage0-inventory.ncl` provides the typed Nickel
      inventory schema. Pueue task 45 passed the focused model tests.
- [x] I3 Add the generated proof inventory helper that writes ✅ 6m 32s (started: 2026-04-25T21:30:38Z → completed: 2026-04-25T21:37:10Z)
      `target/host-tool-free-stage0/stage0-inventory.ncl` only from explicit
      `CRUNCH_STAGE0_SEED_SANDBOX_ENTRY`, `CRUNCH_STAGE0_SEED_SANDBOX_SHELL`,
      `CRUNCH_STAGE0_SEED_TOOLCHAIN_ROOT`, and explicit bootstrap-build-tool
      inputs, expands each executable under the toolchain/build-tool inputs into
      per-executable `bootstrap-toolchain-tool` or `bootstrap-build-tool`
      entries with paths/digests/provenance, computes BLAKE3 digests, and never
      searches `PATH` or `/nix/store`.
      [covers=bootstrap.hosttoolfree.exec.boundary,bootstrap.hosttoolfree.proof.mode]
      Evidence: pueue task 56 passed isolated-target
      `cargo test -p crunch protected_exec -- --nocapture` with 12 tests. New
      generated-inventory tests cover missing explicit env inputs, expansion of
      sandbox/shell/toolchain/build-tool paths, ignoring unrelated `PATH` and
      `NIX_STORE`, BLAKE3 digest rendering, Nickel file writing, and empty
      toolchain fail-closed behavior.
- [x] I4 Route protected-phase network fetches through `Stage0Inventory` source ✅ 1m 19s (started: 2026-04-25T21:38:21Z → completed: 2026-04-25T21:39:40Z)
      URL allowlisting and extraction-rule validation so undeclared source URLs,
      missing source entries, digest mismatches, and missing provenance fail
      closed before fetch or extraction. [covers=bootstrap.hosttoolfree.exec.boundary]
      Evidence: pueue task 61 passed isolated-target
      `cargo test -p crunch protected_exec -- --nocapture` with 13 tests.
      `ProtectedExecPolicy::source_fetch_plan` now returns only declared source
      URLs with digest and extraction rules, `verify_source_digest` rejects
      mismatches before extraction, and malformed source URL/extraction entries
      fail during inventory validation.
- [x] I5 Route self-build/proof protected-phase process launches through one ✅ 0m 51s (started: 2026-04-25T21:44:48Z → completed: 2026-04-25T21:45:39Z)
      non-shell protected process-launch adapter that enforces
      `ProtectedExecPolicy` before crunch-owned `execve` calls and records
      executable path, digest, reason, phase, inventory entry, and policy
      decision. [covers=bootstrap.hosttoolfree.exec.boundary]
      Evidence: pueue task 67 passed isolated-target
      `cargo test -p crunch protected_exec -- --nocapture` with 16 tests.
      `ProtectedProcessLauncher` now computes executable BLAKE3, plans the
      launch through `ProtectedExecPolicy`, records path/digest/reason/phase/
      inventory-entry/policy decision audit events, returns a `Command` only
      after the policy permits launch, and rejects digest mismatches before a
      command is returned.
- [x] I6 Replace protected-phase source staging helper use (`git`) with ✅ 0m 8s (started: 2026-04-25T21:46:25Z → completed: 2026-04-25T21:46:33Z)
      Rust-owned source staging logic. [covers=bootstrap.hosttoolfree.exec.boundary]
      Evidence: pueue task 69 passed
      `cargo test -p crunch self_build -- --nocapture` with 87 filtered
      `self_build` tests, including source staging and staged-source validation.
      `src/self_build.rs` source staging uses Rust filesystem copying and grep
      found no `Command::new("git")` / `git archive` staging path.
- [x] I7 Replace protected-phase archive/copy helper use (`tar`, `cp`) with ✅ 0m 8s (started: 2026-04-25T21:46:25Z → completed: 2026-04-25T21:46:33Z)
      Rust-owned archive and copy logic. [covers=bootstrap.hosttoolfree.exec.boundary]
      Evidence: pueue task 69 passed the focused `self_build` suite; staging
      copy paths use `std::fs::copy`, directory traversal, and symlink handling,
      while grep found no protected-phase `Command::new("tar")` or
      `Command::new("cp")` helper launch.
- [x] I8 Replace protected-phase shell/cargo/helper execution (`sh`, `cargo`, ad ✅ 0m 8s (started: 2026-04-25T21:46:25Z → completed: 2026-04-25T21:46:33Z)
      hoc helper commands) with Rust-owned validation, vendored-input checks, or
      declared seed artifacts carrying inventory digests.
      [covers=bootstrap.hosttoolfree.exec.boundary]
      Evidence: pueue task 69 passed `self_build` tests covering vendored-input
      validation, missing/extra vendored packages, checksum mismatches, and
      malformed lock data. `require_checked_vendor_inputs` validates Cargo
      inputs in Rust instead of shelling out, and grep found no protected-phase
      `Command::new("sh")` or `Command::new("cargo")` helper path.
- [x] I9 Add unit tests for allowed stage0 crunch, allowed declared seed roles, ✅ 0m 7s (started: 2026-04-25T21:47:44Z → completed: 2026-04-25T21:47:51Z)
      declared seed digest mismatch rejection, missing required seed artifact
      fail-closed diagnostics, forbidden absolute host executable, forbidden
      PATH command execution, forbidden Nix seed entries even with matching
      digests, non-executable source URL allowlisting, extraction-rule
      validation, provenance-category rejection, non-BLAKE3 interoperability
      validation, and each removed protected-phase helper family.
      [covers=bootstrap.hosttoolfree.exec.boundary,bootstrap.hosttoolfree.sandbox.entrypoint]
      Evidence: pueue task 72 passed isolated-target
      `cargo test -p crunch protected_exec -- --nocapture` with 18 protected
      exec tests. Added explicit fake-`PATH` helper-family coverage for `git`,
      `tar`, `cp`, `sh`, `cargo`, `bwrap`, and Nix commands; added missing
      required seed path fail-closed coverage; existing tests cover allowed
      stage0 crunch/self launch, declared roles, digest mismatch, source URL /
      extraction validation, provenance, and non-BLAKE3 interoperability.

## Phase 2: Host-tool-free sandbox entry

- [x] I10 Replace protected-mode host `bwrap` fallback with the declared seed ✅ 3m 19s (started: 2026-04-25T21:53:59Z → completed: 2026-04-25T21:57:18Z)
      `sandbox-entry` executable and declared `sandbox-shell`, both validated by
      inventory digest/provenance before use. [covers=bootstrap.hosttoolfree.sandbox.entrypoint]
      Evidence: pueue task 89 passed chained isolated-target tests:
      `cargo test -p crunch declared_seed_bootstrap_tools -- --nocapture`
      (2 focused self-build tests) and
      `cargo test -p crunch protected_exec -- --nocapture` (18 protected exec
      tests in lib and bin targets). `self-build --no-host-tools
      --stage0-inventory <path>` now loads and validates the stage0 Nickel
      inventory, selects `sandbox-entry` as `BwrapSource::DeclaredSeed`, sets
      the declared `sandbox-shell` as `SNIX_BUILD_SANDBOX_SHELL`, and does not
      record a host-bwrap fallback event for that protected-mode selection.
- [x] I11 Add Linux seccomp user-notification exec supervision for the declared ✅ 3m 24s (started: 2026-04-25T22:21:44Z → completed: 2026-04-25T22:25:08Z)
      seed sandbox and descendants. The supervisor must trap `execve`/`execveat`
      before execution, validate path/digest against `Stage0Inventory`, append
      audit events, and fail closed when supervision, filter inheritance, or
      path resolution is unavailable. [covers=bootstrap.hosttoolfree.exec.boundary,bootstrap.hosttoolfree.sandbox.entrypoint]
      Evidence: pueue task 135 passed chained isolated-target
      `cargo test -p crunch seccomp_supervisor -- --nocapture` and
      `cargo test -p crunch protected_exec -- --nocapture`; the lib/bin
      protected-exec runs each passed 21 tests, including seccomp user-notify
      tests for allowed `execve`, allowed descendant `execveat`, and digest
      mismatch denial before execution. `src/protected_exec_seccomp.rs` installs
      a Linux seccomp user-notification filter for `execve`/`execveat`, reads
      the target path from `/proc/<tid>/mem` with notification-id revalidation,
      validates BLAKE3/path against `ProtectedExecPolicy`, appends
      `ProtectedSeccompAuditEvent`, continues allowed syscalls, returns EACCES
      for denied syscalls, and fails closed on unsupported arch/kernel/filter or
      unresolved/relative/empty target paths. `self-build --no-host-tools
      --stage0-inventory` now installs this supervisor after declared seed
      selection.
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
