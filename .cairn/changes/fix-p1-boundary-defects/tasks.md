# Tasks: Fix the P1 boundary defects

## Phase 1: Secret and store boundaries

- [x] [serial] T1.1 Share bounded private-key file admission between the ordinary and shared Rust loaders; zeroize generated and decoded buffers. r[boundary_hardening.private_key_diagnostics]
- [x] [serial] T1.2 Redact keypair decode errors and reject a public half that does not match the secret half. r[boundary_hardening.private_key_diagnostics]
- [x] [serial] T1.3 Preserve unsigned retained PathInfo during Snix GC and document the build-output scope of the signing invariant. r[boundary_hardening.unsigned_gc]
- [x] [serial] T1.4 Add regressions for redacted parse errors, mismatched halves, loose modes, symlinks, oversized files, and unsigned GC retention. r[boundary_hardening.private_key_diagnostics] r[boundary_hardening.unsigned_gc]

## Phase 2: Worker and sandbox liveness

- [x] [serial] T2.1 Complete the worker task on a cancellable build-service panic with a failed outcome instead of a permanent pending future. r[boundary_hardening.worker_liveness]
- [x] [serial] T2.2 Complete unobserved fetch/sandbox errors the same way while leaving quarantine ownership with the service. r[boundary_hardening.worker_liveness]
- [x] [serial] T2.3 Remove the UTF-8 environment panic; preserve Unix bytes and reject NUL/oversized values with a typed error before spawn. r[boundary_hardening.worker_liveness]
- [x] [serial] T2.4 Drain sandbox stdout/stderr under a per-stream bound and append an explicit truncation marker. r[boundary_hardening.bounded_diagnostics]
- [x] [serial] T2.5 Add panic-shutdown, environment-byte, and capture-bound regressions. r[boundary_hardening.worker_liveness] r[boundary_hardening.bounded_diagnostics]

## Phase 3: Acquisition budgets

- [x] [serial] T3.1 Add separate compressed, decompressed, per-entry, and cumulative budgets with bounded compressed spooling. r[boundary_hardening.bounded_acquisition]
- [x] [serial] T3.2 Bound XZ/LZMA output through a limited writer and sniff unknown suffixes from a two-byte prefix. r[boundary_hardening.bounded_acquisition]
- [x] [serial] T3.3 Enforce the entry count during prefix detection and charge regular entries and hardlink copies against the expansion budget. r[boundary_hardening.bounded_acquisition]
- [x] [serial] T3.4 Probe one byte past the flat-download limit and clean failure outputs. r[boundary_hardening.bounded_acquisition]
- [x] [serial] T3.5 Add exact-limit, over-limit, per-entry, cumulative, and compressed input regressions. r[boundary_hardening.bounded_acquisition]

## Phase 4: Clean-runner evaluation

- [x] [serial] T4.1 Redirect the pinned valence transitive artifact source to the same HTTPS revision with a Cargo patch. r[boundary_hardening.clean_runner_evaluation]
- [x] [serial] T4.2 Confirm `nix develop` evaluates without the SSH artifact source. r[boundary_hardening.clean_runner_evaluation]
- [ ] [serial] T4.3 Report a full `nix flake check` outcome; a later failing check stays tracked separately rather than being hidden. r[boundary_hardening.clean_runner_evaluation]

## Phase 5: Verification and records

- [x] [serial] T5.1 Run the focused regressions and record their exact output. r[boundary_hardening.change_evidence]
- [x] [serial] T5.2 Run the first-party quality gate or scoped equivalents for the touched packages. r[boundary_hardening.change_evidence]
- [x] [serial] T5.3 Update the ADR index and retain the change evidence. r[boundary_hardening.change_evidence]

## Phase 6: Follow-on security boundaries

- [x] [serial] T6.1 Publish first-run signing keys through atomic no-replace temporary-file persistence and converge concurrent callers. r[boundary_hardening.atomic_key_publication]
- [x] [serial] T6.2 Strip setuid, setgid, and sticky bits from regular tar entries and hardlink copies; fail on permission errors. r[boundary_hardening.tar_mode_policy]
- [x] [serial] T6.3 Move sandbox work roots under the state directory with 0700 creation, owner/type/mode admission, and an operator override; apply the same admission to the remote executor. r[boundary_hardening.private_work_roots]
