# ADR 0089: Resolve native bindings through the validated closure

## Status

Accepted (2026-08-31)

## Context

The full-source Rust binding records native artifact paths relative to its
native-provider root. This form keeps the binding valid after an isolated
provider copy moves.

V86 built and validated the complete Rust provider. It then loaded the fresh
binding before the first Mantle stage.

The loader treated `bin/ar` as a process-relative path. It rejected the path as
unavailable, although the isolated native provider still contained the exact
executable.

The validated toolchain closure already records absolute paths for the isolated
native compiler, linker, sysroot, helpers, and runtime files.

## Decision Drivers

- Keep the canonical full-source binding location-independent.
- Bind execution to the isolated native provider, not its preserved origin.
- Reject absolute, empty, or escaping receipt paths.
- Recheck every executable mode and BLAKE3 identity after path resolution.
- Do not widen ambient path discovery or fallback authority.
- Keep failed proof evidence immutable.

## Decision

Load and validate the toolchain closure before loading the Rust provider.

Derive one native-provider root from the closure's receipt-bound C compiler.
The existing closure-root validation must also confirm the native linker uses
that root and the Rust provider uses a different root.

For each native artifact in the Rust binding:

1. Require a nonempty relative path.
2. Permit only normal path components.
3. Join the path to the validated native-provider root.
4. Require the joined path to exist.
5. Require executable permission for executable roles.
6. Require the exact recorded BLAKE3 digest.

The runtime copy of each binding uses the resolved absolute path. The canonical
binding receipt remains unchanged.

If a full-source binding exists without an explicit validated closure, loading
fails before execution.

## Alternatives Considered

### Store absolute native paths in the binding

Rejected. Absolute paths would bind the receipt to one scratch or import
location and prevent safe provider relocation.

### Resolve native paths under the Rust-provider root

Rejected. These files belong to the separately validated native provider.

### Search the filesystem for matching tools

Rejected. Name or digest search would add ambient discovery and could select a
provider outside the validated closure.

### Skip native artifact validation

Rejected. Later action plans need exact executable and runtime authority.

## Consequences

- Relocated native-provider bindings remain portable and fail closed.
- Runtime authority stays bound to the isolated closure paths and exact bytes.
- Provider loading now depends on closure validation when a full-source binding
  is present.
- V86 remains failed evidence. A fresh promoted proof must verify this repair.
