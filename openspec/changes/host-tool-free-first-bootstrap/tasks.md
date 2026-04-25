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
      validates the currently parsed executable path/digest against
      `ProtectedExecPolicy`, appends `ProtectedSeccompAuditEvent`, continues
      allowed syscalls, returns EACCES for denied syscalls, and fails closed on
      unsupported arch/kernel/filter or empty/relative target paths. Full
      tracee-root path resolution and unsupported `execveat` dirfd handling are
      intentionally left to I20/V4a. `self-build --no-host-tools
      --stage0-inventory` now installs this supervisor after declared seed
      selection.
- [x] I12 Add tests proving a declared seed sandbox executable with a matching ✅ 0m 7s (started: 2026-04-25T22:26:36Z → completed: 2026-04-25T22:26:43Z)
      digest starts successfully and is recorded as a declared seed artifact in
      proof/audit output. [covers=bootstrap.hosttoolfree.sandbox.entrypoint,bootstrap.hosttoolfree.proof.mode]
      Evidence: pueue task 139 passed chained isolated-target
      `cargo test -p crunch declared_seed -- --nocapture` and
      `cargo test -p crunch seccomp_supervisor -- --nocapture`. The
      declared-seed slice passed 1 lib test and 5 bin tests, including
      declared `BwrapSource` proof-line roundtrip and declared sandbox
      entry/shell audit selection; seccomp tests passed 3 lib and 3 bin tests,
      with allowed `execve`/`execveat` audit events asserting
      `inventory_entry_id == "sandbox-entry"`.
- [x] I13 Add tests proving host `bwrap` on `PATH` is ignored/rejected in ✅ 0m 14s (started: 2026-04-25T22:28:40Z → completed: 2026-04-25T22:28:54Z)
      host-tool-free mode when no declared sandbox seed exists, and proving
      seccomp/supervisor-unavailable conditions fail closed before sandbox
      startup. [covers=bootstrap.hosttoolfree.sandbox.entrypoint]
      Evidence: pueue task 145 passed chained isolated-target
      `cargo test -p crunch no_host_tools -- --nocapture` and
      `cargo test -p crunch seccomp_supervisor -- --nocapture`. The
      no-host-tools tests passed 2 bin tests, proving `--no-host-tools` rejects
      a missing `--stage0-inventory` with a diagnostic that names the pre-bwrap
      lookup boundary and rejects `--stage0-inventory` outside no-host-tools
      mode. Seccomp tests passed 4 lib and 4 bin tests, including a deterministic
      unsupported-audit-architecture fail-closed test before filter install.

## Phase 3: Proof rail

- [x] I14 Implement the protected-phase transition event: after the exact ✅ 0m 7s (started: 2026-04-25T22:31:10Z → completed: 2026-04-25T22:31:17Z)
      crunch-built `bootstrap/bwrap.ncl` and `bootstrap/busybox.ncl` outputs
      are built, export them, verify them against recorded output metadata,
      select them as the later-stage sandbox entry and sandbox shell, and record
      the transition in the execution audit before relaxing protected-phase seed
      execution rules. [covers=bootstrap.hosttoolfree.exec.boundary,bootstrap.hosttoolfree.sandbox.entrypoint]
      Evidence: pueue task 152 passed chained isolated-target
      `cargo test -p crunch protected_transition -- --nocapture` and
      `cargo test -p crunch self_build -- --nocapture`; the focused transition
      tests passed 2 bin tests, and the broader self-build filter passed 93 bin
      tests. `build_all_bootstrap_tools` now computes a
      `ProtectedPhaseTransition` after the selected bwrap/busybox paths are
      exported, executable-checked, store-name resolved, and BLAKE3-hashed;
      `SelfBuildReport` proof lines record the transition marker, selected
      paths, digests, and store names.
- [x] I15 Extend `scripts/prove-self-hosting.sh` with `--no-host-tools` and ✅ 0m 10s (started: 2026-04-25T22:36:48Z → completed: 2026-04-25T22:36:58Z)
      `--stage0-inventory <path>` so the proof poisons common host commands,
      launches only through the protected execution boundary, and preserves
      existing `self-build-proof: fallback-event=...` summary markers.
      [covers=bootstrap.hosttoolfree.proof.mode]
      Evidence: pueue task 156 passed `bash -n scripts/prove-self-hosting.sh`
      plus `cargo test -p crunch --test self_hosting prove_self_hosting_script
      -- --nocapture`, with 27 proof-script tests passed. The helper now accepts
      `--no-host-tools --stage0-inventory <path>`, exports the inventory and
      blocked tool set to the proof harness, captures an absolute cargo path
      before PATH poisoning, removes common host helpers from the proof PATH,
      rejects missing inventory before launching cargo, and leaves existing
      non-protected proof modes working.
- [x] I16 Extend `tests/self_hosting.rs` to run no-host-tools mode while ✅ 0m 16s (started: 2026-04-25T22:39:25Z → completed: 2026-04-25T22:39:41Z)
      preserving the existing stage1, stage2, busybox, and bwrap fixed-point
      checks. [covers=bootstrap.hosttoolfree.proof.mode]
      Evidence: pueue task 158 passed `cargo test -p crunch --test
      self_hosting no_host -- --nocapture` with 4 tests passed. The ignored
      self-hosting fixed-point proof now reads the no-host-tools inventory from
      the proof environment, appends `--no-host-tools --stage0-inventory <path>`
      to stage0 `self-build`, asserts the poisoned proof PATH lacks common host
      helpers, filters proof-tool inventory records so blocked helpers are
      absent rather than required, and leaves the existing stage1/stage2 binary,
      bwrap, and busybox fixed-point assertions unchanged.
- [x] I17 Write `protected-exec-audit.json` into proof bundles and summarize the ✅ 0m 2s (started: 2026-04-25T22:42:43Z → completed: 2026-04-25T22:42:45Z)
      audit in `summary.txt`, including the machine-readable execution audit,
      stage0 inventory digest, blocked host command set, declared seed sandbox
      and shell/toolchain artifact records, fallback-event markers, and final
      result. [covers=bootstrap.hosttoolfree.proof.mode]
      Evidence: pueue task 162 passed `cargo test -p crunch --test
      self_hosting proof_bundle -- --nocapture` with 3 tests passed. Proof
      bundle generation now writes `protected-exec-audit.json`, records schema
      `crunch-protected-exec-audit-v1`, no-host-tools mode, optional stage0
      inventory BLAKE3 digest, blocked host command set, stage0/stage2 fallback
      events, stage0/stage2 protected transition records, and result; summary
      output includes the audit digest/path and transition summaries.
- [x] I18 Add a no-host-tools proof failure fixture that injects an undeclared ✅ 0m 6s (started: 2026-04-25T22:45:05Z → completed: 2026-04-25T22:45:11Z)
      protected-phase executable or child exec, verifies the proof fails, and
      verifies the proof bundle identifies the hidden host tool.
      [covers=bootstrap.hosttoolfree.proof.mode]
      Evidence: pueue task 169 passed chained `cargo test -p crunch
      seccomp_supervisor -- --nocapture` and `cargo test -p crunch --test
      self_hosting proof_bundle -- --nocapture`; seccomp coverage passed 5 lib
      and 5 bin tests, including
      `seccomp_supervisor_denies_undeclared_host_bwrap_before_execve`. The
      fixture creates a fake executable named `bwrap`, installs the protected
      exec supervisor with an inventory that only declares the current test
      binary, asserts the attempted host bwrap receives `PermissionDenied`, and
      asserts the audit event is `policy_decision=denied` for `execve` with no
      inventory entry before execution.
- [x] I19 Update README and `docs/bootstrap-stage0-inventory.md` with the ✅ <1m (completed: 2026-04-25T22:46:38Z)
      protected-phase boundary, exact allowed kernel-interface list, inventory
      schema/path, predeclared seed-artifact rules, seccomp supervisor behavior,
      and no-host-tools proof claim.
      [covers=bootstrap.hosttoolfree.exec.boundary,bootstrap.hosttoolfree.proof.mode]
      Evidence: `rg -n -- '--no-host-tools|protected-exec-audit|seccomp|bootstrap/stage0-inventory.ncl|sandbox-entry' README.md docs/bootstrap-stage0-inventory.md` found the updated operator docs; `openspec validate host-tool-free-first-bootstrap --strict` passed; `git diff --check` passed. Docs now describe the no-host-tools helper mode, protected phase start/end, allowed direct Linux interfaces, Nickel inventory path/schema, seed roles, fail-closed seccomp behavior, proof audit bundle file, and bounded proof claim.

## Phase 4: V4 proof correctness hardening

- [x] I20 Resolve protected seccomp exec paths in the tracee execution ✅ 9m 45s (started: 2026-04-25T23:30:54Z → completed: 2026-04-25T23:40:39Z)
      namespace before digesting: record the tracee path and resolved host path,
      hash the resolved executable bytes, and fail closed for unsupported
      `execveat` dirfd/path forms, relative paths, unreadable paths, or other
      ambiguous resolution cases. [covers=bootstrap.hosttoolfree.exec.boundary]
      Evidence: `cargo test -p crunch seccomp_supervisor -- --nocapture`
      passed after the change with 9 lib tests and 9 bin tests, including
      tracee-root join coverage for `/bin/sh`, symlink resolution before
      digesting, relative path denial, unreadable path denial, and unsupported
      non-`AT_FDCWD` `execveat` dirfd denial.
- [x] I21 Thread actual `ProtectedSeccompAuditEvent` records from the stage0 ✅ 3m 59s (started: 2026-04-25T23:41:07Z → completed: 2026-04-25T23:45:06Z)
      supervisor into `SelfBuildReport` and `protected-exec-audit.json`, and make
      the audit result derived from the observed proof outcome rather than a
      hard-coded success string. [covers=bootstrap.hosttoolfree.exec.boundary,bootstrap.hosttoolfree.proof.mode]
      Evidence: pueue task 14 passed
      `cargo test -p crunch report_format_roundtrip_with_protected_transition -- --nocapture`
      and `cargo test -p crunch --test self_hosting proof_bundle -- --nocapture`.
      The report roundtrip preserves protected seccomp event JSON, and proof
      bundle coverage writes `stage0_seccomp_events`, tracee/resolved path
      fields, and a derived `fixed-point-mismatch` result instead of a hard-coded
      success string in the mismatch fixture.
- [x] I22 Copy the concrete no-host-tools `stage0-inventory.ncl` into the proof ✅ 2m 15s (started: 2026-04-25T23:45:38Z → completed: 2026-04-25T23:47:53Z)
      bundle, hash the copied file, and include declared seed artifact records
      (id, role, phase, path, digest, provenance category/text, owner, required
      flag) in `protected-exec-audit.json` and `summary.txt`.
      [covers=bootstrap.hosttoolfree.proof.mode,bootstrap.hosttoolfree.sandbox.entrypoint]
      Evidence: pueue task 18 passed
      `cargo test -p crunch --test self_hosting proof_bundle -- --nocapture`
      with 3 tests. The proof-bundle fixture now runs in no-host-tools mode,
      copies `stage0-prerequisites/stage0-inventory.ncl`, hashes the copied
      file, serializes `declared_seed_artifacts` with id/role/phase/path/digest/
      provenance/owner/required fields, and mirrors the inventory copy plus
      artifact records in `summary.txt`.
- [ ] I23 Add an operator-facing inventory generation/preflight path for V4 that
      consumes only explicit seed paths, rejects discovery from `PATH` or
      `/nix/store`, reports static-vs-dynamic seed closure risk, and emits the
      concrete inventory path used by the full proof.
      [covers=bootstrap.hosttoolfree.proof.mode,bootstrap.hosttoolfree.sandbox.entrypoint]
- [ ] I24 Tighten protected source artifact extraction rules to the documented
      `key=value` grammar (`format`, `strip-components`, `root`) and reject
      malformed or ambiguous rules before fetch or extraction.
      [covers=bootstrap.hosttoolfree.exec.boundary]
- [ ] I25 Implement verified-output promotion for protected-phase fetched or
      built outputs: after crunch validates a protected source/hash/extraction
      contract, enumerate permitted executable files, compute BLAKE3 digests,
      append promotion audit records including the source entry and accepted
      extraction rules, and extend the supervisor policy before any child process
      can execute those paths. [covers=bootstrap.hosttoolfree.exec.boundary,bootstrap.hosttoolfree.sandbox.entrypoint]

## Validation

- [x] V1 Run `openspec validate host-tool-free-first-bootstrap --strict` and ✅ 0m 2s (started: 2026-04-25T22:47:01Z → completed: 2026-04-25T22:47:03Z)
      record the result. [covers=bootstrap.hosttoolfree.exec.boundary,bootstrap.hosttoolfree.sandbox.entrypoint,bootstrap.hosttoolfree.proof.mode]
      Evidence: pueue task 174 printed `Change 'host-tool-free-first-bootstrap' is valid`.
- [x] V2 Run fake-PATH and inventory validation tests proving `git`, `tar`, ✅ 0m 2s (started: 2026-04-25T22:47:01Z → completed: 2026-04-25T22:47:03Z)
      `cp`, `sh`, `cargo`, `bwrap`, and Nix commands are not invoked in the
      protected phase, declared seed sandbox/shell/toolchain acceptance and
      reporting succeeds, declared seed digest mismatches are rejected, missing
      required seed artifacts fail closed with diagnostics, non-executable
      source URLs and extraction rules are validated, provenance categories are
      enforced, and protected-phase behavior stays within the documented
      kernel-interface allowlist. [covers=bootstrap.hosttoolfree.exec.boundary,bootstrap.hosttoolfree.sandbox.entrypoint]
      Evidence: pueue task 174 passed `cargo test -p crunch protected_exec --
      --nocapture`; the focused filter passed 23 lib tests and 23 bin tests.
- [x] V3 Run seccomp supervisor tests proving child `execve`/`execveat` events ✅ 0m 2s (started: 2026-04-25T22:47:01Z → completed: 2026-04-25T22:47:03Z)
      are denied before execution when undeclared or mismatched, and proving
      supervisor-unavailable/filter-inheritance plus current raw path parse
      failures are fail-closed; namespace-aware tracee-root resolution remains
      covered by I20/V4a. [covers=bootstrap.hosttoolfree.exec.boundary,bootstrap.hosttoolfree.sandbox.entrypoint]
      Evidence: pueue task 174 passed `cargo test -p crunch seccomp_supervisor
      -- --nocapture`; the focused filter passed 5 lib tests and 5 bin tests,
      including mismatch denial, undeclared host-bwrap denial, execveat allow,
      and unsupported audit-architecture fail-closed coverage.
- [ ] V4a Run focused namespace-resolution tests for the seccomp supervisor:
      sandbox `/bin/sh` resolves through the tracee root, relative paths fail
      closed, unreadable paths fail closed, and unsupported `execveat` dirfd
      forms fail closed before execution. [covers=bootstrap.hosttoolfree.exec.boundary,bootstrap.hosttoolfree.sandbox.entrypoint]
- [ ] V4b Run proof-bundle audit tests showing actual child-exec event fields,
      copied concrete inventory digest, declared seed records, blocked host
      command set, and derived success/failure result are written to
      `protected-exec-audit.json` and summarized in `summary.txt`.
      [covers=bootstrap.hosttoolfree.proof.mode]
- [ ] V4c Run extraction-rule and verified-output promotion tests showing
      malformed source rules are rejected, accepted rules are carried into
      promotion records, promoted executable digests are added before exec, and
      unpromoted generated-output executables are denied.
      [covers=bootstrap.hosttoolfree.exec.boundary,bootstrap.hosttoolfree.sandbox.entrypoint]
- [ ] V4d Run inventory preflight tests proving the V4 helper consumes only
      explicit seed paths, rejects `PATH` and `/nix/store` discovery, reports
      static-vs-dynamic seed closure risk, and emits the exact concrete
      inventory path passed to the full proof.
      [covers=bootstrap.hosttoolfree.proof.mode,bootstrap.hosttoolfree.sandbox.entrypoint]
- [ ] V4 After I20-I25 are complete, run the full no-host-tools
      self-hosting proof with a concrete bundled stage0 inventory and explicit
      seed-toolchain strategy, then record proof bundle path,
      stage1/stage2/busybox/bwrap fixed-point status, protected execution audit
      summary, bundled stage0 inventory digest, blocked host command set,
      declared seed sandbox/shell/toolchain artifact records, actual child-exec
      event summary, final derived result, and preserved
      `self-build-proof: fallback-event=...` markers.
      [covers=bootstrap.hosttoolfree.proof.mode]
- [ ] V5 Run the hidden-host-tool proof failure fixture and record that the
      proof fails while identifying the injected undeclared host tool in the
      proof bundle. [covers=bootstrap.hosttoolfree.proof.mode]
