# Architecture validation (V4)
Observed 2026-10-29 (local run timestamps below). Command observations for one selected source tree. This file records what ran and what it printed; it does not accept I6, V1, V2, V3 or V5, and it does not complete the change. See Non-claims.
## Subject
- Git commit `5fc2ad4b3dfb1ae80ff5060cdd8fb6670230e316`, tree `35d06fef314555b7df2e47d1f15294caa29a47ad` (both from the first retained step).
- Working tree status before and after: empty (clean) in all 2 retained `git status --porcelain` steps.
- Toolchain as printed by the semantic rail: `cargo 1.96.0-nightly (a357df4c2 2026-04-03)`; `rustc 1.96.0-nightly (c75612477 2026-04-07)`; `LLVM version: 22.1.2`.
- Checker source `scripts/check-cli-architecture.rs` BLAKE3 `b296a0c9ea91ad7018aa15799f7d999a416b07cc65ba53f73d8d034c597b3da8`; semantic runner `scripts/no_std_core_checks.py` BLAKE3 `db22a8765c93f310b25029f2e91c3994528ac83c0a24b480b420e194e28d2163`.
- Runner: `run-commands.py` (BLAKE3 `93e431b88e990538ee266521a2ade108881d0b5d51f73b32f94ba461acd67b3d`), request BLAKE3 `0a93d42e15c7a02e9103daecdfca17faaac74d0e79b95cede06f5faefb6b81dd`.
- Retained run directory: `/home/brittonr/.cargo-target/mantle-complete-20261026-9ar4jfgk/thin-cli-architecture-v85`.
- Started 2026-10-29T22:58:58.052743-04:00, finished 2026-10-29T23:09:44.854328-04:00; driver exit 0; selected-subject digests unchanged: True.
## Commands
| # | Label | Exit | Elapsed | stdout BLAKE3 (first 16) | stderr BLAKE3 (first 16) |
|---|---|---|---|---|---|
| 1 | `source-identity-before` | 0 | 0.0s | `05fd3aae0655d553` | `af1349b9f5f9a1a6` |
| 2 | `worktree-status-before` | 0 | 0.3s | `af1349b9f5f9a1a6` | `af1349b9f5f9a1a6` |
| 3 | `architecture-self-test` | 0 | 2.1s | `459254665391e647` | `af1349b9f5f9a1a6` |
| 4 | `architecture-check` | 0 | 16.4s | `cba3e7a3d6eca5a8` | `af1349b9f5f9a1a6` |
| 5 | `architecture-semantic-self-test` | 0 | 2.0s | `0872a5e7f3ede7af` | `af1349b9f5f9a1a6` |
| 6 | `architecture-semantic-check` | 0 | 616.5s | `ad5a57a2646cfd6d` | `af1349b9f5f9a1a6` |
| 7 | `operator-contract-generator-self-test` | 0 | 1.3s | `37b56919070451e8` | `af1349b9f5f9a1a6` |
| 8 | `operator-contract-check` | 0 | 2.0s | `cb8d6b7b58b05d80` | `af1349b9f5f9a1a6` |
| 9 | `application-contract-wasm32` | 0 | 5.4s | `af1349b9f5f9a1a6` | `2feec4dc3871ce52` |
| 10 | `worktree-status-after` | 0 | 0.2s | `af1349b9f5f9a1a6` | `af1349b9f5f9a1a6` |

Exact argv (executed inside the repository development shell through the retained integration payload wrapper):
- `source-identity-before`: `git rev-parse HEAD HEAD^{tree}`
- `worktree-status-before`: `git status --porcelain`
- `architecture-self-test`: `cargo -Zscript scripts/check-cli-architecture.rs --self-test`
- `architecture-check`: `cargo -Zscript scripts/check-cli-architecture.rs`
- `architecture-semantic-self-test`: `cargo -Zscript scripts/check-cli-architecture.rs --semantic --self-test`
- `architecture-semantic-check`: `cargo -Zscript scripts/check-cli-architecture.rs --semantic`
- `operator-contract-generator-self-test`: `cargo run --locked --offline -q -p mantle --bin generate-operator-command-contract -- --self-test`
- `operator-contract-check`: `bash scripts/check-operator-command-contract.sh`
- `application-contract-wasm32`: `cargo check --locked --offline -p mantle-application-contract --target wasm32-unknown-unknown`
- `worktree-status-after`: `git status --porcelain`

## Verbatim output
### `architecture-self-test` (exit 0)
```
cli-architecture self-test: PASS
```
### `architecture-check` (exit 0)
```
cli architecture: PASS
```
### `architecture-semantic-self-test` (exit 0)
```
cargo 1.96.0-nightly (a357df4c2 2026-04-03)
release: 1.96.0-nightly
commit-hash: a357df4c26fc14514e66aae2a269456b5545c7db
commit-date: 2026-04-03
host: x86_64-unknown-linux-gnu
libgit2: 1.9.2 (sys:0.20.4 vendored)
libcurl: 8.19.0-DEV (sys:0.4.87+curl-8.19.0 vendored ssl:OpenSSL/3.5.4)
ssl: OpenSSL 3.5.4 30 Sep 2025
os: NixOS 26.11.0 [64-bit]
rustc 1.96.0-nightly (c75612477 2026-04-07)
binary: rustc
commit-hash: c756124775121dea0e640652c5ee3c89e3dd0eb4
commit-date: 2026-04-07
host: x86_64-unknown-linux-gnu
release: 1.96.0-nightly
LLVM version: 22.1.2
architecture compiler fixture: bounded chunked DefId capture and complete counts: accepted/rejected
architecture compiler fixture: pure: accepted
architecture compiler fixture: pure-local-mantle: accepted
architecture compiler fixture: pure-reexport: accepted
architecture compiler fixture: cross-module-alias: rejected
architecture compiler fixture: macro-process: rejected
architecture compiler fixture: macro-output: rejected
architecture compiler fixture: cross-crate-reexport: rejected
architecture compiler fixture: renamed transitive normal dependency: rejected
architecture compiler fixture: build-only and proc-macro host dependencies: accepted
cli architecture compiler self-test: PASS
```
### `architecture-semantic-check` (exit 0)
```
cargo 1.96.0-nightly (a357df4c2 2026-04-03)
release: 1.96.0-nightly
commit-hash: a357df4c26fc14514e66aae2a269456b5545c7db
commit-date: 2026-04-03
host: x86_64-unknown-linux-gnu
libgit2: 1.9.2 (sys:0.20.4 vendored)
libcurl: 8.19.0-DEV (sys:0.4.87+curl-8.19.0 vendored ssl:OpenSSL/3.5.4)
ssl: OpenSSL 3.5.4 30 Sep 2025
os: NixOS 26.11.0 [64-bit]
rustc 1.96.0-nightly (c75612477 2026-04-07)
binary: rustc
commit-hash: c756124775121dea0e640652c5ee3c89e3dd0eb4
commit-date: 2026-04-07
host: x86_64-unknown-linux-gnu
release: 1.96.0-nightly
LLVM version: 22.1.2
architecture compiler owner: /home/brittonr/.cargo-target/mantle-complete-20261026-9ar4jfgk/worktree/crates/crunch-action-result-core/Cargo.toml: macro-expanded HIR + typed HIR: PASS
architecture compiler owner: /home/brittonr/.cargo-target/mantle-complete-20261026-9ar4jfgk/worktree/crates/crunch-android-core/Cargo.toml: macro-expanded HIR + typed HIR: PASS
architecture compiler owner: /home/brittonr/.cargo-target/mantle-complete-20261026-9ar4jfgk/worktree/crates/crunch-attestation-core/Cargo.toml: macro-expanded HIR + typed HIR: PASS
architecture compiler owner: /home/brittonr/.cargo-target/mantle-complete-20261026-9ar4jfgk/worktree/crates/crunch-bootstrap-core/Cargo.toml: macro-expanded HIR + typed HIR: PASS
architecture compiler owner: /home/brittonr/.cargo-target/mantle-complete-20261026-9ar4jfgk/worktree/crates/crunch-cmake-core/Cargo.toml: macro-expanded HIR + typed HIR: PASS
architecture compiler owner: /home/brittonr/.cargo-target/mantle-complete-20261026-9ar4jfgk/worktree/crates/crunch-composition-core/Cargo.toml: macro-expanded HIR + typed HIR: PASS
architecture compiler owner: /home/brittonr/.cargo-target/mantle-complete-20261026-9ar4jfgk/worktree/crates/crunch-delta-core/Cargo.toml: macro-expanded HIR + typed HIR: PASS
architecture compiler owner: /home/brittonr/.cargo-target/mantle-complete-20261026-9ar4jfgk/worktree/crates/crunch-esp-idf-core/Cargo.toml: macro-expanded HIR + typed HIR: PASS
architecture compiler owner: /home/brittonr/.cargo-target/mantle-complete-20261026-9ar4jfgk/worktree/crates/crunch-eval-budget-core/Cargo.toml: macro-expanded HIR + typed HIR: PASS
architecture compiler owner: /home/brittonr/.cargo-target/mantle-complete-20261026-9ar4jfgk/worktree/crates/crunch-evaluation-stream-core/Cargo.toml: macro-expanded HIR + typed HIR: PASS
architecture compiler owner: /home/brittonr/.cargo-target/mantle-complete-20261026-9ar4jfgk/worktree/crates/crunch-fpga-core/Cargo.toml: macro-expanded HIR + typed HIR: PASS
architecture compiler owner: /home/brittonr/.cargo-target/mantle-complete-20261026-9ar4jfgk/worktree/crates/crunch-gc-core/Cargo.toml: macro-expanded HIR + typed HIR: PASS
architecture compiler owner: /home/brittonr/.cargo-target/mantle-complete-20261026-9ar4jfgk/worktree/crates/crunch-gpu-core/Cargo.toml: macro-expanded HIR + typed HIR: PASS
architecture compiler owner: /home/brittonr/.cargo-target/mantle-complete-20261026-9ar4jfgk/worktree/crates/crunch-hardware-simulation-core/Cargo.toml: macro-expanded HIR + typed HIR: PASS
architecture compiler owner: /home/brittonr/.cargo-target/mantle-complete-20261026-9ar4jfgk/worktree/crates/crunch-kernelscript-core/Cargo.toml: macro-expanded HIR + typed HIR: PASS
architecture compiler owner: /home/brittonr/.cargo-target/mantle-complete-20261026-9ar4jfgk/worktree/crates/crunch-model-conversion-core/Cargo.toml: macro-expanded HIR + typed HIR: PASS
architecture compiler owner: /home/brittonr/.cargo-target/mantle-complete-20261026-9ar4jfgk/worktree/crates/crunch-nix-gateway-core/Cargo.toml: macro-expanded HIR + typed HIR: PASS
architecture compiler owner: /home/brittonr/.cargo-target/mantle-complete-20261026-9ar4jfgk/worktree/crates/crunch-overlay-core/Cargo.toml: macro-expanded HIR + typed HIR: PASS
architecture compiler owner: /home/brittonr/.cargo-target/mantle-complete-20261026-9ar4jfgk/worktree/crates/crunch-pnpm-core/Cargo.toml: macro-expanded HIR + typed HIR: PASS
architecture compiler owner: /home/brittonr/.cargo-target/mantle-complete-20261026-9ar4jfgk/worktree/crates/crunch-project-core/Cargo.toml: macro-expanded HIR + typed HIR: PASS
architecture compiler owner: /home/brittonr/.cargo-target/mantle-complete-20261026-9ar4jfgk/worktree/crates/crunch-python-core/Cargo.toml: macro-expanded HIR + typed HIR: PASS
architecture compiler owner: /home/brittonr/.cargo-target/mantle-complete-20261026-9ar4jfgk/worktree/crates/crunch-release-core/Cargo.toml: macro-expanded HIR + typed HIR: PASS
architecture compiler owner: /home/brittonr/.cargo-target/mantle-complete-20261026-9ar4jfgk/worktree/crates/crunch-remote-core/Cargo.toml: macro-expanded HIR + typed HIR: PASS
architecture compiler owner: /home/brittonr/.cargo-target/mantle-complete-20261026-9ar4jfgk/worktree/crates/crunch-repair-core/Cargo.toml: macro-expanded HIR + typed HIR: PASS
architecture compiler owner: /home/brittonr/.cargo-target/mantle-complete-20261026-9ar4jfgk/worktree/crates/crunch-rust-cache-core/Cargo.toml: macro-expanded HIR + typed HIR: PASS
architecture compiler owner: /home/brittonr/.cargo-target/mantle-complete-20261026-9ar4jfgk/worktree/crates/crunch-service-readiness-core/Cargo.toml: macro-expanded HIR + typed HIR: PASS
architecture compiler owner: /home/brittonr/.cargo-target/mantle-complete-20261026-9ar4jfgk/worktree/crates/crunch-shell-core/Cargo.toml: macro-expanded HIR + typed HIR: PASS
architecture compiler owner: /home/brittonr/.cargo-target/mantle-complete-20261026-9ar4jfgk/worktree/crates/crunch-source-core/Cargo.toml: macro-expanded HIR + typed HIR: PASS
architecture compiler owner: /home/brittonr/.cargo-target/mantle-complete-20261026-9ar4jfgk/worktree/crates/crunch-spacewasm-core/Cargo.toml: macro-expanded HIR + typed HIR: PASS
architecture compiler owner: /home/brittonr/.cargo-target/mantle-complete-20261026-9ar4jfgk/worktree/crates/crunch-test-core/Cargo.toml: macro-expanded HIR + typed HIR: PASS
architecture compiler owner: /home/brittonr/.cargo-target/mantle-complete-20261026-9ar4jfgk/worktree/crates/crunch-verification-bridge-core/Cargo.toml: macro-expanded HIR + typed HIR: PASS
architecture compiler owner: /home/brittonr/.cargo-target/mantle-complete-20261026-9ar4jfgk/worktree/crates/crunch-wasm-component-core/Cargo.toml: macro-expanded HIR + typed HIR: PASS
architecture compiler owner: /home/brittonr/.cargo-target/mantle-complete-20261026-9ar4jfgk/worktree/crates/crunch-watch-core/Cargo.toml: macro-expanded HIR + typed HIR: PASS
architecture compiler owner: /home/brittonr/.cargo-target/mantle-complete-20261026-9ar4jfgk/worktree/crates/crunch-zephyr-core/Cargo.toml: macro-expanded HIR + typed HIR: PASS
architecture compiler owner: /home/brittonr/.cargo-target/mantle-complete-20261026-9ar4jfgk/worktree/crates/mantle-application-contract/Cargo.toml: macro-expanded HIR + typed HIR: PASS
architecture compiler owner: /home/brittonr/.cargo-target/mantle-complete-20261026-9ar4jfgk/worktree/crates/mantle-cargo-build-bundle-core/Cargo.toml: macro-expanded HIR + typed HIR: PASS
architecture compiler owner: /home/brittonr/.cargo-target/mantle-complete-20261026-9ar4jfgk/worktree/crates/mantle-causal-trace-core/Cargo.toml: macro-expanded HIR + typed HIR: PASS
architecture compiler owner: /home/brittonr/.cargo-target/mantle-complete-20261026-9ar4jfgk/worktree/crates/mantle-lock-vendor-core/Cargo.toml: macro-expanded HIR + typed HIR: PASS
architecture compiler owner: /home/brittonr/.cargo-target/mantle-complete-20261026-9ar4jfgk/worktree/crates/mantle-portable-client-core/Cargo.toml: macro-expanded HIR + typed HIR: PASS
architecture compiler owner: /home/brittonr/.cargo-target/mantle-complete-20261026-9ar4jfgk/worktree/crates/mantle-rust-differential-core/Cargo.toml: macro-expanded HIR + typed HIR: PASS
architecture compiler owner: /home/brittonr/.cargo-target/mantle-complete-20261026-9ar4jfgk/worktree/crates/mantle-rust-plan-core/Cargo.toml: macro-expanded HIR + typed HIR: PASS
architecture compiler owner: /home/brittonr/.cargo-target/mantle-complete-20261026-9ar4jfgk/worktree/crates/mantlepkgs-core/Cargo.toml: macro-expanded HIR + typed HIR: PASS
cli architecture compiler and resolved runtime dependencies: PASS
```
### `operator-contract-generator-self-test` (exit 0)
```
operator command contract generator self-test: PASS
```
### `operator-contract-check` (exit 0)
```
operator command contract generator self-test: PASS
operator command contract: PASS (commands=211)
```
### `application-contract-wasm32` (exit 0; stdout empty; stderr verbatim)
```
warning: /home/brittonr/.cargo-target/mantle-complete-20261026-9ar4jfgk/worktree/Cargo.toml: file `/home/brittonr/.cargo-target/mantle-complete-20261026-9ar4jfgk/worktree/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling proc-macro2 v1.0.106
   Compiling unicode-ident v1.0.24
   Compiling quote v1.0.45
   Compiling shlex v1.3.0
   Compiling find-msvc-tools v0.1.9
   Compiling serde_core v1.0.228
   Compiling cc v1.2.59
   Compiling zmij v1.0.21
   Compiling syn v2.0.117
   Compiling serde v1.0.228
   Compiling serde_json v1.0.149
   Compiling blake3 v1.8.5
    Checking cfg-if v1.0.4
    Checking arrayvec v0.7.6
    Checking constant_time_eq v0.4.2
    Checking itoa v1.0.18
    Checking memchr v2.8.0
    Checking arrayref v0.3.9
   Compiling serde_derive v1.0.228
    Checking mantle-rust-plan-core v0.1.0 (/home/brittonr/.cargo-target/mantle-complete-20261026-9ar4jfgk/worktree/crates/mantle-rust-plan-core)
    Checking mantle-application-contract v0.1.0 (/home/brittonr/.cargo-target/mantle-complete-20261026-9ar4jfgk/worktree/crates/mantle-application-contract)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 5.39s
```

## Fixture inventory
### Lexical self-test (source enumeration)
The lexical `--self-test` prints one line (`cli-architecture self-test: PASS`). Its fixtures are enumerated here from `scripts/check-cli-architecture.rs` (`fn self_test`), not from printed output. Each rejected authority shape is asserted to be rejected in both the core scope and the contract scope; every comment inside `self_test` is reproduced with its line number so each boundary fixture group is attributable:
- Rejected authority shapes (36): `CLI parser`, `CLI shell`, `bare crate import`, `grouped crate import`, `bare extern crate`, `renamed shell crate`, `qualified shell crate`, `nested shell module`, `shell macro`, `Snix`, `filesystem`, `platform filesystem`, `process`, `thread`, `async runtime`, `alternate runtime`, `environment`, `clock`, `monotonic clock`, `random`, `OS random`, `system random`, `network`, `HTTP client`, `provider`, `store service`, `blob service`, `presentation adapter`, `operator rendering`, `operator stream`, `nested import sibling`, `module alias`, `chained grouped alias`, `extern crate alias`, `re-exported module alias`, `production after tests`.
- `self_test` comments, in source order:
  - L868: // Each authority family is rejected in both protected production scopes.
  - L869: // These are validator inputs, not diagnostic wording or copied-source tests.
  - L920: // Pure allocations/serialization, explicit values, provider/store facts,
  - L921: // formatting, and test-only host helpers remain legal in both scopes.
  - L965: // The same implementation remains permitted in an adapter and integration
  - L966: // test target: neither belongs to a core's production src authority scope.
  - L1015: // Test-module text is excluded from every boundary.
  - L1026: // Core error ownership: a core that names the CLI error type is rejected.
  - L1109: // Command-root classification: an unclassified CLI root is rejected and a
  - L1110: // classified one is accepted.
  - L1140: // Untyped core error returns: a core function returning a bare String error
  - L1141: // is rejected, and a typed one is accepted.

### Semantic self-test (observed output above)
| Fixture | Observed |
|---|---|
| pure | accepted |
| pure-local-mantle | accepted |
| pure-reexport | accepted |
| cross-module-alias | rejected |
| macro-process | rejected |
| macro-output | rejected |
| cross-crate-reexport | rejected |
| renamed transitive normal dependency | rejected |
| build-only and proc-macro host dependencies | accepted |

### Coverage of the authority classes named by V2
Each class below is rejected by the listed lexical self-test families (names asserted to exist in the script by this renderer):

| V2 class | rejecting self-test families |
|---|---|
| CLI | CLI parser, CLI shell, bare crate import, grouped crate import, bare extern crate, renamed shell crate, qualified shell crate, nested shell module, shell macro |
| Snix | Snix |
| filesystem | filesystem, platform filesystem |
| process | process, thread |
| async-runtime | async runtime, alternate runtime |
| environment | environment |
| clock | clock, monotonic clock |
| random | random, OS random, system random |
| network | network, HTTP client |
| provider | provider |
| store-service | store service, blob service |
| rendering | presentation adapter, operator rendering, operator stream |
| alias/evasion shapes | nested import sibling, module alias, chained grouped alias, extern crate alias, re-exported module alias, production after tests |

## Semantic rail owners
42 first-party core/contract owners printed `macro-expanded HIR + typed HIR: PASS`, followed by `cli architecture compiler and resolved runtime dependencies: PASS`. The complete owner list with absolute paths is in the verbatim output above.

## Non-claims
- The lexical checker reads source text; it does not prove arbitrary semantic or domain correctness.
- The semantic rail covers the first-party core and contract owners it enumerates; it is not proof that every operation classifies an observation (I6) or that command tests are adequate (V1).
- Passing fixtures show these rules reject the listed shapes; they do not enumerate all possible authority leaks.
- The operator-contract and wasm32 checks are compile/freshness observations, not behavioral parity.
- No flake check, Cairn gate, strict Clippy or formatting result is claimed by this file (V5).
- No accepted-spec sync, archive, release eligibility or successor publication follows.
