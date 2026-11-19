# CI alignment evidence

## Flake evaluation

```text
nix eval .#checks.x86_64-linux.clippy.drvPath
"/nix/store/p6yyjbr9hmfsk0xx1zjjgzwwbs2lqhsa-mantle-clippy-0.1.0.drv"
```

The check evaluates with `--workspace --all-targets` and the canonical vendored
exclusions.

## Local all-target Clippy

The scoped first-party runs during this work used
`cargo clippy -p crunch-build -p crunch-store -p crunch-pipeline -p mantle
--all-targets --no-deps -- -D warnings` and completed with no warnings.

## Workflow

`.github/workflows/flake-check.yml` now triggers on `push`, `pull_request`, and
`workflow_dispatch`, keeps `permissions: contents: read`, and records
`rustc -Vv`. Fork PRs receive the same read-only check without secrets.
