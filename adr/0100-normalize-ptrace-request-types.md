# ADR 0100: Normalize ptrace request types at the libc boundary

## Status

Accepted (2026-08-31)

## Context

V97 removed the false missing-action suffix and reached compilation of Mantle
through the restored musl Rust toolchain. That compiler rejected every direct
`libc::ptrace` request constant at the first call:

```text
expected i32, found u32
```

The vendored musl `libc` declaration uses `c_int` for the request. The host GNU
libc declaration accepts `c_uint`. Mantle's request constants use `c_uint`, so
host validation did not expose the musl type mismatch.

The numeric Linux ptrace request values fit both ABI parameter types. The policy,
register handling, syscall filtering, and deny-before-exec behavior do not
change.

V97's reconciliation error originally hid this earlier topology blocker because
the main shell returned only the reconciliation failure when topology returned
a blocked receipt.

## Decision Drivers

- Compile against both GNU and musl libc declarations.
- Keep one reviewed numeric request constant set.
- Perform conversion only at the FFI call boundary.
- Preserve every ptrace failure as fail closed.
- Preserve the primary topology blocker when reconciliation also fails.
- Test the exact musl signature with positive and negative fixtures.

## Decision

Pass each ptrace request constant as `REQUEST as _` in the direct
`libc::ptrace` call.

Rust infers the request parameter type from the selected libc declaration:
`c_uint` on the current GNU host and `c_int` in the restored musl toolchain.
Apply this conversion to `SEIZE`, `GETREGS`, `SETREGS`, `CONT`, and
`GETEVENTMSG` calls.

Do not cast request values earlier, change their constants, or use unchecked
integer arithmetic outside the FFI edge.

When topology returns a blocked receipt and action reconciliation also fails,
include the topology blocker's class and message in the returned error before
the reconciliation detail.

## Alternatives Considered

### Change all request constants to `c_int`

Rejected. That reverses the mismatch on libc declarations that use `c_uint`.

### Add target-specific constant types

Rejected. The numeric policy is identical. Target branches would duplicate the
request table.

### Suppress the restored-toolchain type error

Rejected. This is a real ABI declaration mismatch at compile time.

### Return only reconciliation failure

Rejected. Reconciliation often reports the unexecuted suffix, while the topology
blocker identifies the first causal failure.

## Consequences

- The ptrace module type-checks against GNU and musl request signatures.
- Numeric request values and supervision semantics remain unchanged.
- V97 replay diagnostics can report the causal compiler error before the action
  suffix.
- The restored-rustc fixture accepts the cast form and rejects the uncast form
  with `E0308`.
- V97 remains failed evidence. A fresh promoted proof must validate full module
  compilation and runtime supervision.
