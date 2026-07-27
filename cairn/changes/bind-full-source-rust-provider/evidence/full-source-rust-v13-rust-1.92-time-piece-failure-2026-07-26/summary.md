# Full-source Rust v13 failure

## Question

Did the direct Linux-header receipt repair produce the complete Rust provider?

## Inspected evidence

- `status.txt` contains `1`.
- The mrustc first stage completed with direct `linux-headers-blake3` stage identity.
- Rust 1.91.1 completed after LLVM consumed the authenticated Linux-header include path.
- Rust 1.92.0 completed LLVM and reached the stage2 Cargo tool build.
- The Rust 1.92.0 OpenSSL build failed because the receipt-bound Perl could not load `Time/Piece.pm`.
- The exact diagnostic is `Can't locate Time/Piece.pm in @INC` at `Makefile.in line 37`.
- The planned provider output did not exist after the driver stopped.
- `blake3sums.txt` records the preserved driver, plans, scripts, successful prior-stage receipts, logs, and failure log.

## Decision

v13 did not produce a valid provider. It failed closed before publication. The completed prior stages are evidence of frontier movement, not resumable state and not provider completion.

## Owner

Mantle full-source bootstrap implementation.

## Next action

Build a fresh authenticated Perl host tool that statically links and installs `Time::Piece` and `Time::Seconds`. Add direct runtime probes, regenerate the host-tool manifest, and start a fresh provider construction.
