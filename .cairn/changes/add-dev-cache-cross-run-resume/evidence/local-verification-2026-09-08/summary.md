# Local verification: 2026-09-08

## Result and scope

V4 passed for the local Rust source at `e5cf7fdb3c581635a6e1ab9fec3a021abcc9b959`. The captured checks cover the resume implementation, its runtime repairs, and source-fixture staging.

V3 remains open. The promoted proof uses source cohort `a7841aea055bf643cc473164b98a2442014f080b` on Leviathan. These local checks do not prove that remote run or cross-cohort binary equality.

The evidence-only commit `7673661e` did not change the tested Rust source, Cargo inputs, Nix inputs, or policy. Final lifecycle checks cover the updated evidence and task record.

## Environment and evidence

The focused checks used the repository development shell and an isolated target directory. `raw/focused-subject.txt` records Rust `1.96.0-nightly` and the exact compiler revision. `raw/devshell-tools.txt` records resolved tools.

Cairn came from Mantle's pinned revision `695124d459574ba7aeba6097310d237f393c243c`, resolved through `nix develop`. The newer sibling checkout is not the policy tool for this change.

Each check has an argument list, exit status, and complete compressed output under `raw/`. The collector used `gzip -n --best`. It did not change the log text. Each original log passed a named two-MiB capture bound. `BLAKE3SUMS` binds the stored files.

The runner files record the exact commands, resource bounds, shell environment, and result handling. They capture each command's exit status directly, without a result-filtering pipeline.

## Focused checks

Run `raw/focused-checks.command.sh` from the worktree through `nix develop -c sh`. Pueue task `4562` completed successfully. Its `focused-result.txt` records `failed_checks=0`.

| Check | Passed | Failed | Ignored |
|---|---:|---:|---:|
| `crunch-dev-resume-core --all-targets` | 15 | 0 | 0 |
| `source_built_fixed_point` | 108 | 0 | 3 |
| `dev_resume` | 9 | 0 | 0 |
| `protected_exec_ptrace` | 15 | 0 | 0 |
| `stagex_transition` | 43 | 0 | 3 |
| `full_source_provider` | 14 | 0 | 0 |
| `copy_selected_source_tree` | 1 | 0 | 0 |
| `source_built_mantle_source` | 5 | 0 | 0 |

Filters overlap. These counts do not describe unique tests. The ignored tests did not run.

The fixed-point check reported:

```text
test result: ok. 108 passed; 0 failed; 3 ignored; 0 measured; 2580 filtered out; finished in 109.14s
```

Positive cases cover fresh-directory restoration and preserved source authority. Negative cases cover stale or modified authority, partial bundles, unknown fields, wrong source profiles, and promoted-path resume rejection.

The same shell passed these checks:

```sh
rustfmt --edition 2024 --check src/self_build.rs src/source_bundle.rs src/full_source_provider.rs src/protected_exec_ptrace.rs src/source_built_fixed_point_shell.rs src/stagex_transition.rs
cargo fmt --check -p crunch-dev-resume-core
cargo clippy --locked --offline -p mantle -p crunch-dev-resume-core --all-targets --no-deps -- -D warnings
git diff --check
```

Clippy reported `Finished` with exit code 0. This is first-party lint evidence for the named packages, not a complete workspace Clippy claim.

Task `4576` ran the two staging filters after the main test bundle. Its status file and output are stored under `raw/staging-fixtures.*`.

## Repository checks

`raw/repository-gates.command.sh` records the pinned Cairn path and exact commands. Cairn validation and proposal, design, and tasks gates passed. Tracey reported:

```text
traceability coverage ok: 157/157 referenced (profile mantle-default)
```

The equivalent development-shell commands are:

```sh
nix develop -c cairn validate --root .
nix develop -c cairn tracey coverage --root .
nix develop -c cairn gate proposal add-dev-cache-cross-run-resume --root .
nix develop -c cairn gate design add-dev-cache-cross-run-resume --root .
nix develop -c cairn gate tasks add-dev-cache-cross-run-resume --root .
```

All five relevant Nix checks built successfully:

| `checks.x86_64-linux` attribute | Decisive output |
|---|---|
| `dev-resume-architecture` | `findings=0 negative-fixtures=9` |
| `dev-resume-core` | 15 passed, 0 failed |
| `dev-resume-core-wasm` | Release-profile build finished, exit 0 |
| `dev-resume-stage-publication` | 3 passed, 0 failed |
| `dev-resume-integration` | 108 passed, 0 failed, 3 ignored |

Each used `nix build --no-link -L --max-jobs 2 --cores 4` with its full check attribute. The argument files preserve each invocation.

### Flake evaluation recovery

The first `nix flake check --no-build --no-eval-cache` failed on an unrealized SpaceWasm bundler source. The following prerequisite command realized that source:

```sh
nix build --dry-run --no-link --no-eval-cache .#spacewasm-reference-bundler
```

The unchanged flake evaluation then passed at `2026-09-08T18:22:32-04:00`. See `raw/flake-prerequisite.*` and `raw/flake-eval-retry.*`. No source, lockfile, pin, generated policy, or shared Nix state was edited to bypass the error.

The original repository runner still records `failed_checks=1`. That is the preserved first evaluation failure, not a failure of the five Nix checks. The successful retry has its own status and complete output. Pueue no longer retained task `4566` when its final log was requested; the per-command files remain available.

Flake evaluation is not a complete `nix flake check` build. It checked the host system only. The five named checks provide the scoped build evidence.

## Owner and next action

Owner: Mantle maintainers and the operator for `add-dev-cache-cross-run-resume`.

Immediate result: checked local resume, provider-admission, ptrace, staging, and publication behavior. Durable contribution: reproducible check commands and bound evidence for later regression review.

V3 stays open until the existing promoted run terminates and its runtime receipts pass review. Observer task `4587` can collect terminal evidence. It cannot launch a proof or accept its result.
