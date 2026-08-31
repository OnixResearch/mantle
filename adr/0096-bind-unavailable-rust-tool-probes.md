# ADR 0096: Bind unavailable Rust tool probes

## Status

Accepted (2026-08-31)

## Context

V93 proved that ADR 0095 removed rustc's absent sysroot-linker probe. Protected
stage1 then stopped at this missing executable:

```text
<cargo-guard-bin>/emcc
```

The selected action was probing for an optional compiler. Mantle's earlier
source-built Rust bootstrap already handled the same bounded probe family with
reviewed exit-127 shims for `emcc`, `pkg-config`, `pkgconf`, and `git`.

Ptrace must remain fail closed when it cannot open an exec target. Missing-path
exceptions would weaken path identity and create a race with later file
publication.

## Decision Drivers

- Preserve deny-before-exec and fail-closed path observation.
- Give optional discovery deterministic unavailable results.
- Reuse the bounded probe family already observed in Rust bootstrap.
- Do not replace real receipt-bound tools.
- Bind exact shim bytes into each Rust action authority.
- Reject stale or modified shims before execution.

## Decision

For cargo-free fixed-point stage execution, reserve every alias supplied by the
validated toolchain closure or source-built host-tool bindings.

For each absent name in this closed set:

```text
emcc
pkg-config
pkgconf
git
```

write an executable script under the one-entry guarded `PATH`. The script uses
the selected source-built BusyBox shell and exits with status 127. Do not write
a shim when a validated provider owns the alias.

Classify each generated shim as a `CompilerPolicyAdapter`. Add its absolute
path, BLAKE3 identity, and producer identity to fixed Rust action authority.
Before authority construction, require the file bytes to equal the generated
script exactly. Reject missing, modified, or ambiguous aliases.

Do not change ptrace handling for missing paths. Do not grant network, package
metadata, compiler, or version-control capability through these shims.

## Alternatives Considered

### Return `ENOENT` from ptrace

Rejected. The supervisor cannot prove that a path remains absent until kernel
execution without weakening its current fail-closed model.

### Add only `emcc`

Rejected. Prior Rust-bootstrap evidence identified the closed companion probe
family. Adding one name per failed proof would repeat known failures.

### Put ambient tools on `PATH`

Rejected. That would add undeclared compiler, package, network, and Git
authority.

### Replace a declared tool with an unavailable shim

Rejected. A validated provider remains authoritative for its own alias.

## Consequences

- Optional tool discovery receives a stable exit-127 result.
- Ptrace observes an existing, hashed, authorized executable.
- The adapters provide no underlying tool capability.
- Real provider aliases take precedence.
- A changed adapter fails before protected action execution.
- V93 remains failed evidence. A fresh promoted proof must validate this rule.
