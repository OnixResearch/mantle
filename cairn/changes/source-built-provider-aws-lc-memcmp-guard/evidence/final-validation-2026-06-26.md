# Final validation

Date: 2026-06-26
Change: `source-built-provider-aws-lc-memcmp-guard`
Task: V4
Requirement: `r[rust_package_planning.source_built_toolchain_closure.aws_lc_memcmp_guard]`

## Formatting and diff hygiene

Pueue task 58:

```text
nix develop -c rustfmt --check src/main.rs src/source_toolchain_closure.rs src/cargo_free_self_build.rs src/rust_plan.rs && git diff --check
```

Result: completed successfully.

## Full mantle binary unit suite

Pueue task 65:

```text
cargo test -p mantle --bin mantle
```

Environment included the documented nightly/toolchain PATH, OpenSSL `PKG_CONFIG_PATH`, and static `SNIX_BUILD_SANDBOX_SHELL`.

Result:

```text
test result: ok. 868 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.45s
```

## Cairn validation

Pueue task 75:

```text
nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
```

Result:

```json
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 7,
  "valid": true
}
```

## Cairn gates

Pueue task 76:

```text
nix run path:/home/brittonr/git/cairn#cairn -- gate proposal source-built-provider-aws-lc-memcmp-guard --root .
```

Result: `"stage": "proposal"`, `"valid": true`, `"verdict": "PASS"`.

Pueue task 78:

```text
nix run path:/home/brittonr/git/cairn#cairn -- gate design source-built-provider-aws-lc-memcmp-guard --root .
```

Result: `"stage": "design"`, `"valid": true`, `"verdict": "PASS"`.

Pueue task 81:

```text
nix run path:/home/brittonr/git/cairn#cairn -- gate tasks source-built-provider-aws-lc-memcmp-guard --root .
```

Result: `"stage": "tasks"`, `"valid": true`, `"verdict": "PASS"`.

## Provider proof rerun

The real provider-backed proof rerun evidence is recorded in `provider-rerun-stage-receipt-2026-06-25.md` from pueue task 22. It remains blocked, but after AWS-LC succeeds and after the selected receipt-bound compiler route is recorded in the durable stage receipt.

## Post-task-update gate rerun

After marking V4 complete, pueue task 88 reran:

```text
nix run path:/home/brittonr/git/cairn#cairn -- validate --root . && nix run path:/home/brittonr/git/cairn#cairn -- gate tasks source-built-provider-aws-lc-memcmp-guard --root .
```

Result: validation completed successfully and the tasks gate reported `"stage": "tasks"`, `"valid": true`, `"verdict": "PASS"`.
