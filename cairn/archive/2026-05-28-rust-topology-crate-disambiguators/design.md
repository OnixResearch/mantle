# Design

## Functional core

Add a pure helper that computes a rustc metadata disambiguator from unit identity:

- package ID
- target name
- target kind
- execution mode
- source digest
- selected feature set
- crate types

The helper serializes these fields canonically and returns a short lowercase BLAKE3 hex prefix suitable for rustc `-C metadata=...`.

## Imperative shell

`native_unit_derivation(...)` and `native_host_unit_derivation(...)` append the codegen option before dependency binding. Existing execution still invokes rustc directly and still resolves produced artifacts from receipts. Output paths remain `lib<crate>.rlib` / proc-macro `.so` in per-unit directories, so existing artifact discovery continues to work.

## Risk

Changing metadata changes every produced rlib hash and invalidates old execution receipts. This is desired: old artifacts lack disambiguators and are not safe for same-name dependency graphs.
