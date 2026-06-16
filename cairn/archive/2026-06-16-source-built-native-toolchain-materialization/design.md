## Context

The previous change made explicit closure promotion depend on enforced zero-seed/all-source-built accounting. The immediate blocker is not promotion logic; it is producing an explicit manifest whose members are real source-built host and target native tools plus sysroot/runtime inputs.

## Design

### Functional core

A pure manifest builder will accept a provider root and candidate member paths as data, validate required roles and aliases, and return either a complete `mantle-source-built-toolchain-closure-v1` manifest model or precise missing-member diagnostics. The core will not read files, hash bytes, inspect permissions, or touch the environment.

### Imperative shell

The shell may inspect a concrete provider/root directory, canonicalize executable/sysroot paths, hash file content with BLAKE3, and call the pure builder. It must refuse to overwrite an existing output manifest unless explicitly allowed by the calling command.

### Required zero-seed member surface

The manifest must include the source-built Rust provider `rustc`, a host-compatible C compiler/linker path for `cc`/linking host units, host runtime/sysroot evidence sufficient for `rustc` to link a host executable, and target-prefixed musl helpers (`x86_64-linux-musl-gcc`, `g++`, `ld`, `ar`, `ranlib`). Generic `cc`, `ld`, and `ld.lld` names remain host-oriented.

### Claim boundary

A generated manifest with missing required members must not be emitted as source-built. A generated zero-seed manifest still does not retire `not-source-built-toolchain-closure` until `mantle self-build --cargo-free --fixed-point --toolchain-closure <manifest>` enforces it and reports `claim = true`.
