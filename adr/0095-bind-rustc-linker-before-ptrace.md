# ADR 0095: Bind the rustc linker before ptrace

## Status

Accepted (2026-08-31)

## Context

V92 reached protected Rust unit execution. The ptrace supervisor stopped when
rustc attempted to execute this absent path:

```text
<rust-sysroot>/lib/rustlib/x86_64-unknown-linux-musl/bin/cc
```

A no-supervision `strace` showed normal `execvp` behavior. The process first
tried that sysroot candidate, received `ENOENT`, then executed the receipt-bound
`cc` from `PATH` and completed successfully.

The supervisor intentionally fails closed when it cannot open an exec target.
Changing it to ignore missing paths would weaken the path-observation rule and
complicate action reconciliation.

## Decision Drivers

- Keep ptrace fail-closed path observation unchanged.
- Avoid unavailable exec candidates before supervision.
- Use the existing receipt-bound C compiler alias.
- Keep linker selection absolute and reviewable.
- Preserve the same linker for compatibility and stage execution.
- Add every generated executable path to protected action authority.

## Decision

Create the rustc runtime wrapper only after its receipt-bound toolchain aliases
exist.

Select the exact `cc` alias for that wrapper location:

- fixed-point compatibility uses `toolchain/receipt-bound-path/cc`;
- ordinary cargo-free execution uses its guarded `cc` sibling.

Require that alias to be executable. Append this final rustc codegen option:

```text
-C linker=<absolute receipt-bound cc>
```

The final option overrides rustc's default sysroot linker candidate. The
wrapper then launches rustc through the binding-owned loader and runtime from
ADR 0091.

When the selected rustc is this generated wrapper, derive its exact linker alias
from the wrapper location. Add that alias to fixed Rust child-action authority
as a C compiler. Require exactly one sibling or nested alias; reject zero or
multiple aliases.

Do not change ptrace handling for `ENOENT`, unreadable paths, relative paths, or
hash failures.

## Alternatives Considered

### Let ptrace return `ENOENT` and continue

Rejected. This weakens fail-closed path observation and adds non-execution event
semantics to every action reconciliation.

### Materialize `cc` inside the restored Rust sysroot

Rejected. That would mutate the immutable checkpoint payload and broaden the
bound sysroot relocation.

### Use `-C linker=cc`

Rejected. PATH is receipt-bound, but the action plan should still name the
absolute executable before execution.

### Permit the missing sysroot candidate

Rejected. A nonexistent path has no bytes or executable authority to admit.

## Consequences

- rustc does not issue the absent sysroot-linker `execve`.
- Compatibility and stage execution use one exact linker alias.
- The alias is included in protected fixed-executable authority.
- The ptrace fail-closed contract remains unchanged.
- V92 remains failed evidence. A fresh promoted proof must verify this route.
