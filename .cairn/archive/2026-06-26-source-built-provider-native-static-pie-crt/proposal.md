## Why

The provider-backed Cargo-free fixed-point proof now advances past local warning hygiene and blocks while linking the root package binary with the receipt-bound source-root musl C compiler. The failed unit uses the guarded `cc` alias, but the alias only downgrades `-static-pie` to `-static`; it does not replace Rust's PIE startup object with the manifest-declared non-PIE `crt1.o`.

The source-root musl seed libc in the declared closure contains non-PIE static objects. Mantle should keep topology links receipt-bound and fail closed, while normalizing unsupported static-PIE CRT inputs through private alias materialization rather than leaking ambient host toolchain behavior.

## What changes

- Extend the receipt-bound C compiler alias runtime to materialize the manifest-declared non-PIE CRT object.
- Map `rcrt1.o` inputs to the private runtime `crt1.o` whenever the alias downgrades static-PIE links to static links.
- Add a private `-no-pie` linker flag when that downgrade occurs so GCC defaults cannot keep the final link in PIE mode.
- Rewrite readable linker response files under the alias runtime directory so Rust's long-link `@file` form receives the same CRT/static-PIE normalization.
- Keep normalization private to the receipt-bound alias directory.
- Rerun focused tests and the provider-backed proof, recording fixed-point success or the next deterministic blocker.

## Scope

- **In scope**: `src/cargo_free_self_build.rs`, Cairn spec/evidence for provider-backed native static-PIE CRT normalization.
- **Out of scope**: replacing the source-root musl seed libc, claiming full fixed-point success without a rerun, changing first-stage Rust provider wrappers, or adding ambient host compiler fallback.
