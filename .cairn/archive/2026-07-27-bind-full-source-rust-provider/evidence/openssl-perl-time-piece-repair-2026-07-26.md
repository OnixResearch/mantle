# OpenSSL Perl Time::Piece repair

## Question

Why did Rust 1.92 fail after Rust 1.91 passed, and what source-built repair is required?

## Inspected evidence

- v13 completed the mrustc stage and Rust 1.91.1.
- Rust 1.92.0 completed LLVM and reached the stage2 Cargo tool build.
- Its vendored OpenSSL `Makefile.in` contains `use Time::Piece;` at line 37.
- The receipt-bound Perl v4 did not install `Time/Piece.pm` or `Time/Seconds.pm`.
- The authenticated Perl 5.10.1 source contains both modules under `ext/Time-Piece/`.

## Decision

Build Perl v5 from the same authenticated source and admitted native provider. Statically link `Time/Piece`, install both Perl modules, and add direct and copied-tree runtime probes for `Time::Piece->gmtime(0)` and `ONE_DAY`.

The new provider is `/home/brittonr/.cache/mantle-full-source-20260718/gcc40-store/jg59by00lr0nsaparmgldp02hqpc1y5z-perl-5.10.1-full-source-gcc10-v5`. An independent runtime probe printed `full-source-perl-time-piece-v5-ok`.

Its canonical artifact attestation is `full-source-rust-host-perl-v5-attestation-2026-07-26.json`. The v6 host-tool manifest binds the new executable and construction receipt. No ambient Perl module path or dynamic extension fallback is allowed.

## Owner

Mantle full-source bootstrap implementation.

## Next action

Run the full provider construction from fresh scratch. Do not resume v13 state or treat the successful Perl probe as Rust-provider completion.
