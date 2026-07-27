# Linux header binding repair

## Question

Should the Rust LLVM stage disable `llvm-exegesis`, or should it consume the already authenticated Linux headers?

## Inspected evidence

- v11 failed at 96% in `llvm-exegesis` because `<asm/prctl.h>` was not on the include path.
- The authenticated Linux 6.6 header provider contains `include/asm/prctl.h` and `include/linux/prctl.h`.
- The v4 host-tool receipts already committed the Linux-header tree and attestation digests as dependencies.
- The v4 host-tool manifest did not preserve the header path, tree digest, source ID, or attestation as a typed support input.

## Decision

Use the authenticated Linux-header provider. Do not disable `llvm-exegesis` to hide the missing dependency.

The v5 host-tool manifest now binds the header root, BLAKE3 tree digest, source ID, attestation path, and attestation digest. Manifest observation verifies the complete tree, required headers, and attestation. Generated source-build scripts pass the bound include directory through `CFLAGS`, `CPPFLAGS`, `CXXFLAGS`, and the explicit LLVM `cflags` and `cxxflags` configuration.

The generated script digest and host-tool manifest digest bind this choice into every stage receipt. Ambient include directories and host-header fallback remain disabled.

## Owner

Mantle full-source bootstrap implementation.

## Next action

Complete a fresh provider construction. Treat passing focused tests or reaching the prior LLVM frontier as insufficient until final provider validation and smoke evidence pass.
