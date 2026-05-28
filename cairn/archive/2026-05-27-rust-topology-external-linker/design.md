## Context

The rust-plan execution rail invokes `rustc` directly from selected Cargo unit arguments. On this host, rustc injects a target `gcc-ld/ld.lld` wrapper that points to a removed Nix store path. A direct smoke compile with `-C link-self-contained=no` succeeds with the same absolute external clang wrapper.

## Decision

### 1. Add a runtime rustc-argument core

Introduce a pure function that derives execution rustc args from reviewable unit args. It appends `-C link-self-contained=no` only if the unit args do not already contain `link-self-contained` in split (`-C link-self-contained=...`) or joined (`-Clink-self-contained=...`) form.

### 2. Preserve explicit unit choices

If the selected unit already carries any `link-self-contained` setting, Mantle does not override it. That keeps unit-specific toolchain choices deterministic and reviewable.

## Risks

- Runtime execution args differ from the recorded reviewable derivation args. This is acceptable for topology evidence because the rail is not a full Cargo-compatible build product; it is bounded execution evidence for selected units. The injected arg avoids host-rustup wrapper breakage while still using the selected external linker.
