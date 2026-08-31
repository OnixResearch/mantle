# ADR 0098: Bind the GCC subprogram prefix

## Status

Accepted (2026-08-31)

## Context

V95 passed guard-path composition and began protected Rust stage1 execution.
The GCC driver then requested this executable path:

```text
<native-provider>/bin/../libexec/gcc/x86_64-unknown-linux-musl/10.5.0/cc1
```

The target was already a validated native artifact, but the lexical path
contained a parent component. Ptrace correctly rejected it before execution.
Parent-component rejection prevents alternate spellings from bypassing exact
path authority and must remain unchanged.

The native binding already carries exact BLAKE3 identities for `cc1`,
`cc1plus`, `collect2`, and `lto-wrapper`.

## Decision Drivers

- Preserve lexical parent-component rejection.
- Select compiler internals only from validated native bindings.
- Cover C, C++, and preprocessor drivers.
- Keep GCC-generated exec paths absolute and normalized.
- Reject incomplete, split-directory, unknown, or changed internal tools.
- Do not add ambient compiler search paths.

## Decision

Derive one GCC subprogram directory from native artifacts whose role is
`compiler-internal`.

Require exactly this closed name set:

```text
cc1
cc1plus
collect2
lto-wrapper
```

Canonicalize each path, recompute its BLAKE3 digest, compare it with the binding,
and require all four files to share one directory.

Derive the bound compiler-driver set from validated C compiler, C++ compiler,
and preprocessor artifacts. For each guarded alias that targets one of those
drivers, append this final option:

```text
-B<canonical compiler-subprogram directory>/
```

The existing receipt-bound alias still supplies normalized CRT, unwind, and
linker-runtime arguments. The final `-B` changes only GCC subprogram selection.

Apply the same bound artifacts to compatibility probes and fixed-point stage
aliases.

## Alternatives Considered

### Permit parent components in ptrace

Rejected. This creates multiple lexical identities for one executable and
weakens the existing deny-before-exec rule.

### Canonicalize the path inside ptrace

Rejected. The kernel would still receive the noncanonical request, and policy
would no longer match the exact path used at syscall entry.

### Set ambient `GCC_EXEC_PREFIX` or `COMPILER_PATH`

Rejected. Environment search is broader and easier to inherit or override.

### Bind only `cc1`

Rejected. C++, linking, and LTO use the companion internal executables from the
same validated artifact family.

## Consequences

- GCC requests canonical internal executable paths.
- The selected directory and every executable remain receipt-bound.
- C, C++, and preprocessing routes share the same normalization.
- Missing or changed compiler internals fail before alias publication.
- Ptrace parent-component rejection remains unchanged.
- V95 remains failed evidence. A fresh promoted proof must validate this route.
