# V86 fresh-binding native-root failure

## Verdict

V86 failed after the complete Rust provider and its 17-member toolchain closure
were materialized. The failure was a loader path-resolution error.

This attempt does not prove the final Mantle fixed point, checkpoint
publication, stage equality, or complete trust.

## Observed evidence

- The proof used strict hermeticity and disabled substitution.
- The reused native provider passed origin and isolated-copy revalidation.
- All five Rust-provider actions completed locally.
- The aggregate reconciled 391,207 observed events to 391,207 matched events.
- The aggregate had zero unknown, missing, fallback, remote, cache-only, or
  authority-violation events.
- The Rust provider produced Rust 1.94.0 and its final build receipt.
- The closure contained 17 members with policy BLAKE3
  `2af733999525e8e907c206be5a82bfde27e36810c4923a55cea1b14fcfc57256`.
- The first Mantle stage did not start.
- `attempt-status.json` records `full-source native artifact is unavailable:
  bin/ar`.

## Root cause

`full-source-binding.json` correctly records `bin/ar` relative to the admitted
native-provider root. The binding keeps native artifact paths relative so an
isolated provider copy can move.

`source-built-toolchain-closure.json` records the isolated native compiler and
linker with absolute paths. Both paths share the admitted native-provider root.

`load_bound_rust_execution_authority` passed the relative binding path directly
to a validator that requires an absolute existing path. The validator therefore
checked process-relative `bin/ar` instead of the isolated provider file.

`native-artifact-path-inspection.txt` proves that both the preserved origin and
the isolated copy still contain executable regular files. Both files have mode
`0555`, size 1,383,896 bytes, and BLAKE3
`3a90141c6ca90962c0a0334b14b8c1d2b5c4365ea2e6f01efeaa0e8d9485c5ce`.
That digest equals the binding digest.

The failure did not show deletion, mutation, an action-authority violation, or
an incomplete Rust-provider build.

## Decision

ADR 0089 requires the loader to validate the closure first. It derives the
native-provider root from the receipt-bound C compiler.

The loader accepts only nonempty relative binding paths with normal path
components. It joins each path to that root, then rechecks the file type,
executable mode, and exact BLAKE3 digest.

The canonical binding remains unchanged. Ambient search, fallback, origin-path
selection, and weakened digest checks remain forbidden.

## Validation

`post-repair-validation.log` records the focused positive and negative tests.
It also records the complete `cargo_free_self_build` and
`source_toolchain_closure` module test results.

## Preserved evidence

This directory contains:

- detached profile and proof launch records;
- source-profile readiness and verification logs;
- the proof plan, closure, and failed attempt status;
- origin and isolated native-provider revalidation reports;
- the complete Rust-provider action plans and reconciliations;
- compressed raw aggregate and per-stage execution audits;
- per-stage build plans, reports, candidate reports, and compressed logs;
- the final Rust-provider build receipt and full-source binding;
- binary transfer parity scripts and records;
- the V84/V85 cleanup receipt;
- the exact native artifact inspection; and
- the profile, proof, and watcher scripts.

## Owner and next action

The Mantle source-built fixed-point change owns the repair. Build and transfer a
new release binary, refresh a Ready profile, and run a fresh promoted proof.
Keep V86 until the fresh run no longer needs its Rust-provider diagnostics.
