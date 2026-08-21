# ADR 0081: Expose remapped Rust manifests through the compiler working directory

## Status

Accepted (2026-08-21)

## Context

Cargo-free Rust plans replace checkout paths with stable release prefixes.
Before V31, normal Rust units received a deterministic
`CARGO_MANIFEST_DIR` such as `/mantle/release/source/vendor-deps/crossterm`.
That path existed only as an identity. It was not mounted on the host.

The V31 preserved-provider replay passed vendor planning and executed 683
stage1 units. `crossterm` then expanded `document_features!()`. That proc macro
reads `CARGO_MANIFEST_DIR/Cargo.toml` during compilation. Rustc failed because
the deterministic identity path was not readable.

Using the physical checkout path directly is also unsafe for deterministic unit
identity. A focused Rust test showed that `--remap-path-prefix` remaps source
paths but does not rewrite strings produced by `env!("CARGO_MANIFEST_DIR")`.
Two otherwise equal rlibs retained different physical directory strings.

## Decision Drivers

- Give compile-time macros read access to the admitted package manifest.
- Keep `CARGO_MANIFEST_DIR` absolute, stable, and independent of the run root.
- Do not create a global mutable symlink or require a new mount authority.
- Keep the selected source root as the compiler working directory.
- Preserve existing custom-build execution behavior.

## Decision

For deterministic, non-custom Rust units, Mantle sets
`CARGO_MANIFEST_DIR` to `/proc/self/cwd/<package-relative-path>`. The Rust
compiler already runs with the admitted physical source root as its working
directory. Therefore, this absolute procfs path resolves to the selected package
without exposing the physical run root in the planned environment.

The package-relative suffix comes from lexical admission beneath the selected
workspace root. A package outside that root does not receive this lowering.

Custom-build compiler units keep their existing physical manifest path. Their
executed build-script child also receives the physical package root and runs
from that root. This ADR does not change build-script authority.

The stable `/mantle/release/source` prefix remains the rustc source-path remap.
It still controls diagnostics and source path identity. The procfs path only
provides compile-time manifest access.

## Alternatives Considered

### Pass the physical package path

Rejected. Rustc does not remap arbitrary `env!` output. Physical run roots can
remain in artifacts and make deterministic cache identity unsound.

### Keep the logical release path and ignore proc-macro reads

Rejected. Cargo packages can read their manifest during macro expansion. A
nonexistent path is not Cargo-compatible enough for the selected closure.

### Bind-mount the source tree at `/mantle/release/source`

Rejected for this repair. It would add mount orchestration and executable
authority to every native Rust unit.

### Use a shared stable symlink

Rejected. A shared mutable path introduces races between concurrent plans and
moves source selection outside the unit's explicit authority.

## Consequences

- Compile-time manifest readers can access their admitted package data.
- Planned manifest paths remain stable across proof roots.
- This deterministic lowering depends on Linux procfs and the explicit rustc
  working directory.
- Non-deterministic planning continues to use the physical package path.
- Custom-build behavior remains unchanged.
- This decision does not authorize other procfs reads, ambient source
discovery, network access, or Cargo invocation.
