Evidence-ID: no-std-functional-core-v4-no-std-boundary-runner
Task-ID: V4
Artifact-Type: verification-note
Covers: functional.core.nostd.boundary.continuously.verified.regression.introduces.std.leak, portability.nostd.core.compiles.without.std.target, portability.nostd.core.dependency.allowlist.catches.std.leak
Reviewer-Role: agent
Verdict: pass
Reviewed-At: 2026-04-22

Validation command:
- `./scripts/check-no-std-core.sh`
- Environment: PATH included `/nix/store/ka4jb1dby54qnd2rv7r4h5r7yl5dyrar-rustup-1.28.2/bin`, `/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin`, the documented clang/mold/pkg-config paths, plus `PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig` and `SNIX_BUILD_SANDBOX_SHELL=/bin/sh`
- Output: `rustup toolchain: nightly-x86_64-unknown-linux-gnu`
- Output: `Change 'no-std-functional-core' is valid`
- Output: `test result: ok. 65 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
- Output: `test result: ok. 77 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
- Output: `test discovery::tests::shell_adapter_keeps_discovery_outside_core ... ok`
- Output: `test refresh_adapter::tests::shell_adapter_keeps_refresh_io_outside_core ... ok`
- Output: `dependency allowlist OK: arrayref, arrayvec, blake3, cfg-if, constant_time_eq, crunch-attestation-core, crunch-project-core, data-encoding, itoa, memchr, serde, serde_core, serde_json, zmij`
- Output: `purity check OK`
- Output: `scope check OK`
- Output: `API shape check OK`
- Output: `ownership check OK`

Result: the rustup-managed umbrella runner now enforces the wasm target prerequisite, executes the required host + `wasm32-unknown-unknown` checks, runs the focused adapter tests, and proves the dependency allowlist / portability / regression rails all pass together.
