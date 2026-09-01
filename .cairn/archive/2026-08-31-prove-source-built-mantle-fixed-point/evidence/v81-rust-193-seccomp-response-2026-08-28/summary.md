# V81 Rust 1.93.1 seccomp response blocker

## Outcome

V81 cleared the OpenSSL source and Cargo checksum boundary. Rust 1.91.1 and Rust 1.92.0 completed with full action reconciliation. Rust 1.93.1 stopped during its LLVM build.

Stage reconciliation results:

- MRustC to Rust 1.90.0: 88,038 observed, 88,038 matched, zero denied.
- Rust 1.91.1: 75,843 observed, 75,843 matched, zero denied.
- Rust 1.92.0: 76,071 observed, 76,071 matched, zero denied.
- Rust 1.93.1: 22,192 observed, 22,192 matched, zero denied, then `stage-execution-failed`.

## Exact blocker

During the Rust 1.93.1 LLVM build, concurrent processes received `Permission denied` while executing the bound CMake and assembler paths.

After the stage stopped, both paths had mode `0555` and executed successfully outside the supervisor. The raw audit classified the final CMake and assembler requests as `allowed` with the expected BLAKE3 identities.

This evidence rules out missing-path authority, digest mismatch, file mode, and persistent filesystem execution failure. It identifies a gap between policy classification and the kernel notification response.

## Repair

The protected seccomp supervisor now binds audit meaning to the result of `SECCOMP_IOCTL_NOTIF_SEND`.

- `EINTR` responses retry up to a named fixed bound.
- An audit event remains `allowed` only when the kernel accepts the response.
- A failed response becomes a denied audit event with the exact ioctl error.
- A failed response discards any automatic promotion from that notification.
- The response retry loop fails closed for all non-`EINTR` errors.

Positive and negative tests cover response-result binding. A subprocess stress test runs 512 allowed executions across four concurrent workers and requires every execution and audit event to succeed.

## Validation

The response-result tests passed. The concurrent stress test passed 512 executions across four workers. `cargo check -p mantle --bin mantle`, edition-2024 rustfmt, and `git diff --check` passed.

The full serialized seccomp suite passed 21 of 22 tests. The deep-descendant test had the same pre-change and post-change failure because its adopted child did not exit within 30 seconds. This existing failure is separate from notification response handling.

## Evidence

The directory preserves the plans, source manifests, compressed raw audits, reconciliations, compressed logs, launch records, exact `EACCES` lines, post-failure direct execution, and final allowed audit records.

## Non-claims

V81 does not prove Rust 1.93.1, later Rust stages, checkpoint publication, fixed-point equality, final receipt verification, or complete action trust. A fresh detached proof must establish those results.
