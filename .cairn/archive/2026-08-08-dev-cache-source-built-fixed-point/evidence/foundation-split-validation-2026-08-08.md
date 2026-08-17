# Dev-cache foundation split validation

## Scope decision

The foundation retains receipt-validated provider adoption, persistent content-addressed store reuse, fast-fail selection, and promoted-path isolation.

`add-dev-cache-cross-run-resume` now owns fresh-directory multi-stage resume, cold-to-cached-to-adopt runtime evidence, and fresh promoted cold-path confirmation.

## Focused validation

Command:

```text
SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo test -p mantle --bin mantle source_built_fixed_point
```

Result:

```text
test result: ok. 51 passed; 0 failed; 1 ignored; 0 measured; 2211 filtered out; finished in 0.31s
```

Command:

```text
SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo test -p mantle --bin mantle self_build_cli
```

Result:

```text
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 2251 filtered out; finished in 0.01s
```

Command:

```text
SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo clippy -p mantle --bin mantle --no-deps -- -D warnings
```

Result:

```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 04s
```

The first leaf-format pass found import-order drift in `src/source_built_fixed_point_shell.rs`. Rustfmt changed imports only.

Commands after the repair:

```text
nix develop -c rustfmt --edition 2021 --check src/source_built_fixed_point_dev_cache.rs src/source_built_fixed_point_shell.rs
git diff --check
```

Result: both commands exited with status 0.

## Lifecycle validation

The repository validator returned:

```text
"valid": true
```

The proposal, design, and tasks gates for `dev-cache-source-built-fixed-point` returned:

```text
"verdict": "PASS"
```

The proposal, design, and tasks gates for `add-dev-cache-cross-run-resume` also returned PASS.

## Non-claims

This evidence does not prove fresh-directory resume, a complete cold-to-cached-to-adopt cycle, or a successful fresh promoted fixed point. The successor package owns those facts.
