# Design: Closed self-build source staging

## Context

Mantle intentionally does not invoke host `cargo vendor` during self-build. The shell stages a fixed top-level source allowlist, validates `Cargo.lock` registry/git entries against the supplied `vendor-deps/` directory and Cargo checksum manifests, then builds inside the bootstrap sandbox.

That design is sound only when the explicit vendor directory is complete and the fixed source allowlist tracks every workspace compile-time input. The current checkout failed both conditions: `cap-fs-ext` was absent from the ignored vendor tree, and `config/action-result-policy/generated/action-result-policy.json` was absent from staged source even though `crunch-store` compiles it with `include_str!`.

## Goals

- Restore one Cargo-authored directory source exactly matching the locked graph.
- Prove metadata resolution without ambient registry/git caches or network access.
- Stage the tracked `config/` policy payload without widening staging to arbitrary checkout roots.
- Preserve positive and negative staging/checksum tests.
- Produce current fixed-point evidence before making a success claim.

## Non-goals

- Committing the large `vendor-deps/` payload to Git or making a fresh clone independently offline.
- Replacing the reduced seed provider, proving compiler correctness, or proving release reproducibility.
- Replacing the fixed allowlist with unrestricted source-tree copying.
- Treating proof preflight, metadata resolution, or stage1 compilation as fixed-point success.

## Decisions

### 1. Regenerate the complete locked vendor directory

**Choice:** Use `cargo vendor --locked` to a fresh sibling directory, then atomically replace the ignored checkout-local `vendor-deps/` input. Validate it with `cargo metadata --offline --locked --config .cargo/vendor-config.toml` under an empty `CARGO_HOME` and `CARGO_NET_OFFLINE=true`.

**Rationale:** Hand-copying only `cap-fs-ext` would not prove that the remaining graph is complete and could simply reveal another missing package. Cargo-authored `.cargo-checksum.json` files preserve the existing self-build validator's authority.

### 2. Keep the fixed staging boundary and add `config/`

**Choice:** Add the top-level `config` root to `STAGED_SOURCE_TOP_LEVEL_ENTRIES`. The test fixture carries the exact action-result generated-policy shape and asserts that it reaches staged source. Existing assertions continue to reject `target/`, arbitrary scratch files, and private `.pi` content.

**Rationale:** `config/` is a current compile-time workspace input. Broad tracked-tree copying would weaken the reviewed fixed source boundary and include lifecycle/operator material that the self-build derivation does not need.

### 3. Use a stable UTF-8 width expression

**Choice:** Replace `char::MAX_LEN_UTF8` with `char::MAX.len_utf8()` in the diagnostic bound assertion.

**Rationale:** The expressions have the same semantic bound, while the latter compiles on the pinned proof nightly without adding an unstable feature gate or a magic numeric literal.

### 4. Preserve Linux no-clobber semantics through the syscall ABI

**Choice:** Route the four root-package `RENAME_NOREPLACE` call sites through one `linux_rename` shell helper that invokes the Linux `renameat2` syscall number directly. Keep path-to-`CStr` validation and caller-specific diagnostics at each boundary. Test both successful movement and an existing-destination race that preserves both files.

**Rationale:** The bootstrap musl target's `libc` Rust bindings expose the syscall number and flag but not the `renameat2` function symbol. Falling back to ordinary rename, check-then-rename, or link/unlink would weaken reviewed no-clobber semantics. A shared safe wrapper isolates the unsafe ABI call and removes four copies.

### 5. Treat only the full proof as self-build success

**Choice:** Offline metadata, checksum validation, proof preflight, bootstrap-tool success, and stage1 compilation are intermediate evidence. The self-build claim requires the canonical ignored proof to finish and its bundle to report stage1/stage2 equality. On this host, `CRUNCH_NO_FUSE=1` is an explicit materialized-input transport selection, not a semantic relaxation.

**Rationale:** Earlier attempts failed first at compiler compatibility, then FUSE setup, omitted source, and target-libc API shape. Recording each frontier prevents partial progress from being promoted into fixed-point evidence.

## Approach registry

| Family | Mechanism | Claim | State | Discriminating evidence |
|---|---|---|---|---|
| Narrow crate copy | Copy cached `cap-fs-ext` and synthesize/check one manifest | Clears the first resolver error | Rejected | Does not establish complete locked closure. |
| Nix source extraction | Reuse a Nix store crate source | Supplies package bytes | Rejected | Does not establish Cargo's complete directory source or all checksum manifests. |
| Cargo-authored full refresh | `cargo vendor --locked` into a fresh tree | Reconstructs the complete locked directory source | Active | Empty-`CARGO_HOME` offline metadata plus self-build checksum validation. |
| Unrestricted source copy | Stage the whole checkout | Avoids future missing roots | Rejected | Widens source authority and admits unrelated/private/generated paths. |
| Fixed allowlist repair | Add only current build-required `config/` | Closes the observed compile-time source gap | Active | Positive policy-file staging and existing negative exclusion tests. |
| Weakened rename fallback | Check destination then call ordinary rename | Compiles on musl | Rejected | Reintroduces a race that can clobber another publisher. |
| Shared syscall shell | Call Linux `SYS_renameat2` with `RENAME_NOREPLACE` | Preserves no-clobber semantics across libc targets | Active | Direct positive/existing-destination tests plus production race fixtures. |

## Validation budget

- **Source budget:** `Cargo.lock`, Cargo vendor output, `.cargo/vendor-config.toml`, `src/self_build.rs`, the compile-time `include_str!`, current proof logs, and accepted bootstrap/proof requirements.
- **Mechanism budget:** seven families above, with three complementary surviving repairs.
- **Round budget:** failing baseline; empty-home offline metadata; focused staging/checksum tests; proof preflight; materialized-input fixed-point proof; focused/full quality; Cairn sync/archive.
- **Allowed terminal outcomes:** validated current fixed point; exact proof blocker with durable diagnostics; exhausted bounded repair; or user decision required for a broader source-distribution model.

## Risks

- `vendor-deps/` remains ignored and can drift again after lock changes; current evidence is checkout-local and time-bound.
- Adding future `include_str!` or build-script source roots requires updating the allowlist and tests.
- The materialized-input route may expose later proof defects hidden by prior FUSE failure.
- A successful fixed point remains seed-assisted and is not independent release reproducibility evidence.
