# Nickel 1.17 cohort validation

- **Question:** Does Mantle use one exact Nickel 1.17 cohort across embedded evaluation, command-line validation, ignored self-build vendor inputs, and checked evidence?
- **Inspected evidence:** The pre-change evaluator test compiled `nickel-lang` 2.0.0 and `nickel-lang-core` 0.16.1, then passed 82 tests. The generated Nix lock binds source commit `1320a983e6c3d1e2fb53dd2464b084b4903b1426` and its NAR SHA-256. The generated Cargo lock binds `nickel-lang` 2.2.0, core 0.18.0, parser 0.3.0, and vector 0.2.0. The repository-owned importer recreated 839 locked vendor package directories. The exact four-package Nickel vendor manifest passed BLAKE3, checksum, license, offline-metadata, stale-manifest, mixed-cohort, floating-pin, boundary-escape, and weakened-non-claim checks. The pinned CLI reports `nickel 1.17.0 (rev 1320a98)`. Post-change checks passed 84 evaluator tests, 14 budget tests, 7 example tests, 40 pipeline tests, focused Clippy, formatting, Nickel configuration checks, the dedicated Nix cohort check, `nix flake check --no-build -L`, and the repository Tiger Style check. The first full check exposed stale durable-publication source bindings for the intentionally changed Cargo and flake bytes; the Nickel source of truth, exported JSON, validator constants, and receipt BLAKE3 were refreshed together, and the focused check then passed.
- **Decision:** Accept the cohort update. Upstream API compatibility required no behavior adapter change. Mantle-owned evaluation, build, store, scheduler, evidence, and release boundaries remain in place. The committed-source full check completed the cohort, durable-publication, dependency, formatting, and Tiger Style paths before reaching the unchanged remote `rust-src` fixed-output mismatch.
- **Owner:** Mantle maintainers.
- **Next action:** Synchronize the accepted Nickel requirements, archive the change, push its branch, and integrate it into `origin/main`.

## Non-claims

The current source differs from historical V98 proof input. This change does not claim a current fixed point, evaluator correctness, derivation correctness, compiler correctness, build hermeticity, or release eligibility.

The optional `cargo-deny` diagnostic still reports pre-existing repository-wide source-policy and advisory failures. It is preserved as a non-gating diagnostic and is not used as cohort acceptance evidence.
