# Broad validation evidence

Date: 2026-08-08

This record covers the accepted durable-publication adoption on the merged Mantle history. Full-source bootstrap and fixed-point repairs are outside this change.

## Source closure

The baseline Nix package build failed before tests because the filtered source omitted two required files:

```text
error: couldn't read `tests/../fixtures/content-bound-requirements/mantle-registry.json`: No such file or directory (os error 2)
error: couldn't read `tests/../fixtures/content-bound-requirements/mantle-requirement-ref.json`: No such file or directory (os error 2)
```

`flake.nix` now has one named `sourceFilter`. It includes only `fixtures/content-bound-requirements` from the otherwise excluded fixture tree. The `content-bound-requirement-source-closure` check proves both directions:

- The filtered source compiles and runs the positive content-bound requirement tests.
- The same build with that exact fixture root removed fails with the missing-file diagnostic above.

The final command passed:

```text
nix build .#checks.x86_64-linux.content-bound-requirement-source-closure -L --option secret-key-files ''
```

After this repair, `nix build .#mantle` reached 154 library tests. It then failed at the independent test `protected_exec_seccomp::linux::tests::seccomp_listeners_require_distinct_fresh_worker_threads`. The nested test reported `No such file or directory`. This change does not alter `src/protected_exec_seccomp.rs`.

## Bootstrap blocker inventory

Report-only mode and its self-test passed. The final report contains:

```text
Actionable findings: 115
Marker classes present: 3
Evidence-backed suppressions: 355
Promotion claims: 0
bridge-output: 72
compiler-runtime-crash-boundary: 40
placeholder-deferred: 3
```

Enforcement mode failed as required while these findings remain:

```text
bootstrap blocker inventory: 115 findings across 3 classes, 355 evidence-backed suppressions, 0 promotion claims, enforce=true
FAIL: bootstrap blocker inventory is not clean; expected 0 findings and 0 promotion claims
```

No marker was hidden, reclassified as clean, or converted into a promotion claim.

## Rust quality ownership

The first-party Clippy command passed after changing the Mantle-owned assertion in `src/rust_plan.rs` to its direct negative form:

```text
cargo clippy --workspace --all-targets --no-deps \
  --exclude fuse-backend-rs --exclude nix-compat --exclude nix-compat-derive \
  --exclude snix-build --exclude snix-castore --exclude snix-store \
  --exclude snix-tracing -- -D warnings
Finished `dev` profile
```

The focused positive test also passed:

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 2302 filtered out
```

The separate vendored `fuse-backend-rs` audit failed with 36 Clippy errors. The first errors were `useless conversion to the same type: u64` in `vendor/fuse-backend-rs/src/api/vfs/sync_io.rs`. This is a visible vendored blocker, not a first-party success claim.

## Octet

Canonical Octet revision `d87153a1bbfe4c3469b2dee6fb5512eafa812d88` completed the normal `cargo-octet check` command. The summary was `Status: warning-only`, with 872 findings. The historical hard failure `invalid Cargo diagnostic path: capability locator ... contains ..` did not occur. This record does not claim an Octet warning-free result.

## Broad matrix

The final commands produced these results:

- Durable-publication Nickel typecheck and negative fixture export: passed. The fixture returned `{ "tests": true }`.
- Durable-publication adoption Nix check: passed.
- Content-bound requirement evidence Nix check: passed.
- Workspace library and test suite: `2232 passed; 3 failed; 68 ignored` for the root binary target. The three failures were the existing `binutils_tcc`, `gcc40`, and `gcc47` bootstrap parity receipt-drift tests.
- First-party Tiger Style: failed in unchanged `crunch-overlay-core` and `crunch-gc-core` surfaces. The first finding was missing assertion density in `revalidate_overlay`.
- Root package formatting: failed only on unchanged `src/source_built_fixed_point_shell.rs`.
- `git diff --exit-code origin/main` over the bootstrap-parity, Tiger Style, formatting, seccomp, and vendored-Clippy blocker surfaces passed. The candidate does not change those surfaces.

## Broad Nix result

The final command evaluated 28 checks and stopped at the truthful bootstrap inventory enforcement:

```text
nix flake check path:$PWD -L --option secret-key-files ''
bootstrap blocker inventory: 115 findings across 3 classes, 355 evidence-backed suppressions, 0 promotion claims, enforce=true
FAIL: bootstrap blocker inventory is not clean; expected 0 findings and 0 promotion claims
```

This is the next exact independent blocker after the focused durable-publication and source-closure checks passed.

## Cairn and traceability

The legacy-layout Cairn revision `e5ee2a61d8561d8fb47f42012b5d23211f847e7e` produced:

- strict validation: valid, with no findings or issues;
- proposal, design, and tasks gates: passed;
- Tracey coverage: `155/155`, verdict `pass`.

## Non-claims

This evidence does not claim broad Nix success, workspace test success, vendored Clippy success, Tiger Style success, formatting success, bootstrap blocker closure, full-source completion, fixed-point success, or release readiness. It proves that the durable-publication source-closure defect is repaired, the focused adoption checks pass, and all observed broad blockers remain explicit.
