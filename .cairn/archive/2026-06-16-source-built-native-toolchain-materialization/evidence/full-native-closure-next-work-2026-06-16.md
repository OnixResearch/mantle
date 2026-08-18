# Full native closure next work (2026-06-16)

Task-ID: H1
Covers: r[rust_package_planning.source_built_toolchain_closure.native_materialization]

## Current result

Mantle now has a fail-closed `bootstrap native-toolchain-closure` materializer and fixed-point enforcement now digest-checks declared `crt-object` and `runtime-library` members. The current real provider frontier still fails before writing a manifest because the available source-root provider is target-musl only for this proof and does not provide the required host-native closure surface.

## Missing host-native closure members from current evidence

The current materialization attempt reports these missing members:

- `cc`
- `ld`
- `host-crt1.o`
- `host-libgcc_s.so.1`
- `host-libc.so`

## Required next implementation work

1. Build or import an actually source-built host-compatible native root for the Rust provider host ABI (`x86_64-unknown-linux-gnu` today), with provider metadata under `share/mantle-native-toolchain/provider.json` or a documented equivalent.
2. The host root must expose at least `bin/cc`, `bin/ld`, `lib/crt1.o`, `lib/libgcc_s.so.1`, and `lib/libc.so` at stable paths. If the real libc layout needs more libraries (`libpthread`, `libdl`, `librt`, `libm`, `libutil`), add them as additional runtime-library members and extend enforcement tests.
3. Keep the existing source-root musl provider as the target root only after its target helpers and target runtime members are present and source-built.
4. Rerun `mantle bootstrap native-toolchain-closure --rust-source-provider <provider> --host-root <host-root> --target-root <target-root> --output <manifest>` and require a zero-seed manifest.
5. Rerun `mantle self-build --cargo-free --fixed-point --rust-source-provider <provider> --toolchain-closure <manifest> --target x86_64-unknown-linux-musl` with the proof bundle outside the source root.
6. Retire `not-source-built-toolchain-closure` only when the enforced fixed-point summary reports `source_built_toolchain_closure.claim = true`.

## Non-claims preserved

This change proves fail-closed materialization and stronger enforcement. It does not prove the host-native source-built closure, release reproducibility, or full Cargo compatibility yet.
