# ADR 0090: Bind Rust sysroot relocation to the binding rewrite

## Status

Accepted (2026-08-31)

## Context

V86 published a promoted checkpoint with 17 payloads. Its Rust-provider payload
contains the full-source binding from the original proof root.

V87 restored that immutable payload into a fresh root. The restore path then
rewrote the binding's Rust, host-tool, receipt, and support-input paths.

This canonical rewrite changed one file inside the Rust sysroot. The sysroot
directory BLAKE3 therefore changed from
`69740316b19be878af5b1788afa0181891489bf303e7b8791f92115aed1be96e` to
`6fb5ada63ae8d107e29083f9707e08f6f1b291e6258cd5e3343ac19352091db0`.

The strict closure relocation comparator rejected that digest change, although
all other Rust-provider content remained equal. Its existing policy permitted
path relocation but no content relocation.

## Decision Drivers

- Keep the published checkpoint immutable.
- Preserve full Rust sysroot identity before and after relocation.
- Permit no unobserved member or tree changes.
- Keep all non-sysroot closure authority checks strict.
- Preserve provider-relative native artifact paths.
- Link the closure decision to the canonical binding relocation report.
- Do not repeat provider execution.

## Decision

Measure the Rust-provider directory with the closure's directory BLAKE3
algorithm immediately before the binding rewrite. Measure it again immediately
after the write.

The binding relocation report records both tree digests and both binding-file
digests. The closure relocation report binds that report by path and BLAKE3.

The functional core accepts a Rust sysroot digest change only when all these
conditions hold:

1. The changed member is exactly the `host-sysroot` member.
2. Its origin digest equals the observed pre-rewrite tree digest.
3. Its relocated digest equals the observed post-rewrite tree digest.
4. The two observed digests differ.
5. Its role, name, trust, source, build receipt, and relative path remain equal.
6. Every other closure member passes the existing strict comparator.

The strict comparator remains unchanged for callers without explicit binding
relocation evidence.

Checkpoint restore validates each native artifact under the relocated native
root, but keeps its binding path provider-relative. The cargo-free loader later
resolves that path through the validated closure, as required by ADR 0089.

## Alternatives Considered

### Ignore the host sysroot digest

Rejected. That would remove the identity check for the complete Rust provider.

### Accept any host sysroot digest change

Rejected. The core requires the exact before-and-after observations from the
single canonical binding rewrite.

### Exclude the binding from every sysroot digest

Rejected. This would weaken existing provider identity outside checkpoint
relocation.

### Move the runtime binding outside the Rust provider

Rejected for this repair. It would require a broader loader and authority
interface while the existing relocation already owns a bounded canonical
rewrite.

### Rebuild the Rust provider for each proof root

Rejected. The promoted checkpoint exists to reuse verified provider execution
without repeating it.

## Consequences

- Restored checkpoints retain their immutable published payload identity.
- The fresh runtime sysroot receives a new, explicitly observed tree identity.
- Closure validation permits one named transformation and rejects all others.
- Native artifact paths remain portable across checkpoint roots.
- V87 remains failed evidence. A fresh promoted run must verify this decision.
