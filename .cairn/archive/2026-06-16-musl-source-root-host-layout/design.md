## Context

The source-root provider already exposes target-prefixed musl C/C++/binutils helpers and musl runtime/startup files. That layout is not a host root for a GNU-host Rust provider, but it is the right host native root for a future musl-host Rust provider.

## Design

### Functional core

Extend the provider-root classifier to return a layout classification, not just success/failure. Source-root musl metadata returns `source-root-musl` when the expected role is `target`, or when the role is `host` and the expected host triple normalizes to `x86_64-linux-musl`. Native provider metadata returns `native-provider`.

### Imperative shell

The member collector uses the layout classification to choose paths:

- `native-provider` host root: `bin/cc`, `bin/ld`, `lib/crt1.o`, `lib/libgcc_s.so.1`, `lib/libc.so`.
- `source-root-musl` host root: `bin/x86_64-linux-musl-gcc`, `bin/x86_64-linux-musl-ld`, `x86_64-linux-musl/lib/crt1.o`, `x86_64-linux-musl/lib/libgcc_s.so.1`, `x86_64-linux-musl/lib/libc.so`.

Target-root collection keeps the existing target-prefixed source-root layout.

### Claim boundary

This change only removes a future false blocker for musl-host Rust providers. It does not make the current GNU-host provider a zero-seed closure and does not retire `not-source-built-toolchain-closure`.
