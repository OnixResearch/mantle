# ADR 0102: Vendor wasi-virt with Crane

## Status

Accepted (2026-09-01)

## Context

`nix flake check -L` built `wasi-virt` with `buildRustPackage`. That path fetched
some locked crates through `crates.io/api`, which returned HTTP 403 on the
current Onix builders. The main Mantle workspace already uses Crane. Crane's
registry path fetched the same locked crate content from
`static.crates.io/crates` successfully.

The `wasi-virt` source revision, Cargo lockfile, Rust version, feature set, and
output stay unchanged.

## Decision Drivers

- Keep the existing immutable source and lockfile.
- Avoid an endpoint that rejects the Onix builders.
- Reuse the repository's existing Crane build boundary.
- Do not modify `flake.lock` by hand.
- Keep component Rust 1.90.0 separate from the main workspace toolchain.

## Decision

Create a component-scoped Crane library with the pinned Rust 1.90.0 toolchain.

Use `vendorCargoDeps` with the `wasi-virt` lockfile. Build dependencies with
`buildDepsOnly`, then build the `wasi-virt` package with `buildPackage` and
`--no-default-features`.

Do not change the source revision, Cargo lockfile, package selection, features,
or the published toolchain record.

## Alternatives Considered

### Retry the blocked API endpoint

Rejected. Repeated attempts returned the same HTTP 403 response.

### Prefetch individual missing crates manually

Rejected. Manual store seeding is not a reproducible repository build rule.

### Change the wasi-virt source or lockfile

Rejected. The failure was transport-only and did not require a dependency
change.

### Add a second package manager path

Rejected. Mantle already owns a Crane vendor path for Rust packages.

## Consequences

- The component uses the reachable static crate endpoint through Crane.
- Dependency and package derivations remain separate and cacheable.
- The component toolchain remains Rust 1.90.0 with `wasm32-wasip2` support.
- `nix flake check -L` can progress beyond the former crate-fetch blocker.
