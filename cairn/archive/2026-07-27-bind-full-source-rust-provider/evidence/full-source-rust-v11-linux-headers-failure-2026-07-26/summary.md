# Full-source Rust v11 failure

## Question

Did v11 publish a complete full-source Rust provider, or did it fail closed before publication?

## Inspected evidence

- `status.txt` contains `1`.
- `rustc-stage1-build.log.zst` records the Rust 1.91.1 LLVM build failure at 96%.
- The exact compiler diagnostic is `fatal error: asm/prctl.h: No such file or directory`.
- The planned output `/home/brittonr/.cargo-target/mantle-full-source-rust-provider-detached-v11-20260726` did not exist after the driver stopped.
- `driver.sh`, `driver.log.zst`, `status.txt`, and `rustc-stage1-build.log.zst` are preserved in this directory.
- `blake3sums.txt` records the preserved-file identities.

## Decision

v11 did not produce a valid provider. It failed closed before publication. Its scratch state is not eligible for resume or completion evidence.

## Owner

Mantle full-source bootstrap implementation.

## Next action

Bind the authenticated Linux 6.6 header root in the host-tool manifest and generated LLVM configuration. Then start a fresh provider construction with new scratch and output paths.
