# Baseline AWS-LC memcmp guard blocker — 2026-06-25

Task-ID: I1, I2
Covers: r[rust_package_planning.source_built_toolchain_closure.aws_lc_memcmp_guard]

## Current blocker

Command evidence: pueue task 15 inspected the fresh current-code provider-backed fixed-point attempt from `mantle-source-built-rust-provider-fixed-point-release-bound-2026-06-25`.

```text
receipt=/home/brittonr/git/mantle-source-built-rust-provider-fixed-point-release-bound-2026-06-25/stage1/receipt.json
manifest=target/source-built-rust-provider-fixed-point-2026-06-25/native-toolchain-closure.json

-- topology blocker --
180051:      "class": "build-script-run-failed",
180052:      "message": "\nthread 'main' (2624845) panicked at /home/brittonr/git/mantle/vendor-deps/aws-lc-sys/builder/cc_builder.rs:729:9:\n### COMPILER BUG DETECTED ###\nYour compiler (cc) is not supported due to a memcmp related bug reported in https://gcc.gnu.org/bugzilla/show_bug.cgi?id=95189. We strongly recommend against using this compiler. \nEXECUTED: true\nERROR: \nOUTPUT: \n\nnote: run with `RUST_BACKTRACE=1` environment variable to display a backtrace"
180052:      "message": "\nthread 'main' (2624845) panicked at /home/brittonr/git/mantle/vendor-deps/aws-lc-sys/builder/cc_builder.rs:729:9:\n### COMPILER BUG DETECTED ###\nYour compiler (cc) is not supported due to a memcmp related bug reported in https://gcc.gnu.org/bugzilla/show_bug.cgi?id=95189. We strongly recommend against using this compiler. \nEXECUTED: true\nERROR: \nOUTPUT: \n\nnote: run with `RUST_BACKTRACE=1` environment variable to display a backtrace"
180097:    "execution_status": "blocked",

-- closure compiler members --
39:      "role": "c-compiler",
40:      "name": "cc",
142:      "name": "x86_64-linux-musl-gcc",
```

## Initial diagnosis

- The active source-built native closure exposes `cc` and `x86_64-linux-musl-gcc` from the same source-root musl GCC toolchain.
- The inspected closure output did not list `clang` or `x86_64-linux-musl-clang` compiler members.
- AWS-LC's guard lives in `vendor-deps/aws-lc-sys/builder/cc_builder.rs::memcmp_check()` and runs when `HOST == TARGET`; it compiles and executes `memcmp_invalid_stripped_check` and treats a failing executable as GCC PR95189 compiler-risk evidence.

## Next implementation focus

Mantle should not bypass the guard by spoofing `HOST`/`TARGET` or falling back to an undeclared host compiler. The next step is to implement deterministic handling that either selects a receipt-bound compiler route that passes the guard or reports a stable unsupported-compiler blocker for provider fixed-point evidence.
