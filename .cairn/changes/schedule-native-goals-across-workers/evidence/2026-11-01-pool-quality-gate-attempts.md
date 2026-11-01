# Ordinary quality-gate attempts for pool work (2026-11-01)

Scope: record what `./scripts/check-first-party-quality.sh` did and did not
prove for the pool changes in this environment. No green full-gate run was
obtained; the focused suites and the one real failure it found are recorded.

## Runs

| Run | Environment | Outcome |
| --- | --- | --- |
| 1 | devshell env as-is | `crunch-build` fetcher tests failed: `which`/`git` absent from the constructed PATH; `crunch-eval` stdlib tests failed with `HOME=/homeless-shelter`; incremental tests failed with `incremental configuration budget exhausted` (ambient env is 87 KiB, budget is 64 KiB) |
| 2 | + `git`/`which`, `HOME`, trimmed `NIX_*` | reached the `mantle` bin suite: 2798 passed, 1 failed — `nix_derivation_payload_rejects_drift_and_malformed_json` expected `remote-nix-derivation-json-invalid`, which the tolerant pool parser had replaced. Fixed in `dbfa0257f`; focused test passes |
| 3 | same as 2, detached | stopped at the 3600s tool cap while running individual bin legs; large suites passed (788, 462, 241 tests) |
| 4 | same as 2, detached | failed on two `crunch-rustc-wrapper` timing/socket tests (`occupied_authority_times_out_and_removes_waiter`, `crunch_rust_cache_core_adoption_compiles_then_reuses_local_castore_result`: `read-protocol-frame-prefix` `WouldBlock`); both pass when rerun serialized with the full toolchain env (`test result: ok. 2 passed`) |

## Focused results that stand

- `cargo test -p mantle --test remote_transfer_production`: 18 passed.
- `cargo test -p mantle --bin mantle -- nix_derivation_payload_rejects_drift_and_malformed_json`: 1 passed after `dbfa0257f`.
- strict Clippy and `cargo fmt --check` for `mantle`, `crunch-build`, `crunch-remote-core`: exit 0.
- `cargo test -p crunch-eval --lib` with the trimmed env: 91 passed.
- `cargo test -p crunch-build --lib` with `git`/`which` on PATH: 788 passed.
- `cargo test -p crunch-rustc-wrapper --lib` (the two gate failures): 2 passed serialized.

## Non-claims

- No uninterrupted green run of the ordinary gate is claimed. The environment
  forces a trade-off: the untrimmed devshell env breaks the incremental
  fingerprint budget, while trimming `NIX_CFLAGS*` breaks C rebuilds
  (`aws-lc-sys` cannot find `openssl/boringssl_prefix_symbols.h`).
- The two rustc-wrapper failures are attributed to timing/resource contention
  because they pass serialized, but no root cause was proven.
