# V78 Rust bootstrap OpenSSL generated-shell blocker

## Outcome

V78 stopped at 2026-08-28T04:38:32-04:00 with exit code 1. It did not publish a provider checkpoint.

V78 passed the prior Cargo and LLVM TableGen boundaries. The Rust 1.91.1 build completed LLVM and reached the stage-two tool closure.

The MRustC stage reconciled 88,038 observed executions with zero denials and 177 promotions. Its reconciliation digest is `c56a1bc040361532929cf21a271ed7c39b71d7726e5853fa7a4123d872391f15`.

The Rust 1.91.1 stage observed 56,302 executions. It matched 56,293, denied nine, and recorded 132 promotions. Its reconciliation digest is `623a74e07e53c0c7b2c2428357963885e05904470c51424bfbb6aada6ab81ffb`.

## Exact blocker

OpenSSL assembly generation invoked `crypto/aes/asm/../../perlasm/x86_64-xlate.pl`. That path requested ambient `/bin/sh`, which resolved to an undeclared Nix Bash. The protected supervisor denied the execution.

The stage also denied eight missing-path `g++` probes. These probes searched the generated target-tool directory and the bounded source-built tool path.

The preserved audit shows all nine denials. The build log records `Permission denied` from the OpenSSL Perl generator and the failed `make build_libs` command.

## Repair

The repair keeps the existing fail-closed execution policy.

- Every source-built Rust bootstrap plan now includes the authenticated OpenSSL `no-asm` rewrite before `x.py`.
- The rewrite covers a bounded set of vendored `openssl-src-300.*` configurations.
- The rewrite is idempotent and rejects missing, conflicting, or unknown source forms.
- The generated target-tool directory now includes a `g++` alias only for the full-source route.
- The `g++` alias uses the receipt-bound shell and the existing protected C++ compiler wrapper.
- No ambient shell, compiler, PATH directory, or fallback executable receives authority.

## Evidence

The directory preserves the V78 wrapper state, launch state, fixed-point plan, native-prefix validation, stage plans, source manifests, scripts, compressed build logs, compressed raw audits, reconciliations, and `attempt-status.json`.

`denied-exec-events.txt` contains the nine denied audit records. `blake3-manifest.txt` records local evidence identities.

## Local validation

The focused Rust-provider suite passed 82 tests. The patch-plan suite passed seven tests. `cargo check -p mantle --bin mantle`, edition-2024 rustfmt, and `git diff --check` passed.

Strict Clippy did not provide acceptance evidence. It stopped on nine pre-existing findings outside the changed files. The exact findings are preserved in `post-repair-validation.log`.

## Non-claims

This repair does not prove the Rust 1.91.1 stage, later Rust stages, checkpoint publication, restored fixed-point equality, or complete action trust. A fresh detached proof must establish those results.
