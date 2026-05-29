# Proposal: Preserve real Cargo host unit identity

## Problem

Native target artifact binding now uses selected Cargo unit IDs, but host artifact planning still groups selected host units by `(package_id, target_name, target_kind)`. Two selected proc-macro or custom-build units with the same package/name/kind can overwrite each other before host artifacts are attached to consumers.

That violates the existing `native_real_unit_identity` requirement for dependency and host artifact binding.

## Change

Preserve every selected Cargo host unit ID through host planning. Host artifacts and host units must be created from exact selected Cargo unit entries or exact Cargo dependency edges, not from a coarse package/name/kind map.

## Impact

- **Files**: `src/rust_plan.rs`, focused Rust tests, Cairn evidence.
- **Testing**: duplicate host-unit identity regression, focused host graph tests, dirty and clean topology self-probes.
