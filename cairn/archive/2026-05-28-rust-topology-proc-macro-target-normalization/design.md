# Design

## Functional core

`host_unit_key(package_id, target_name, target_kind)` is the pure matching core for selected host units. Extend it so:

- `custom-build` continues to normalize to `build-script-build`.
- `proc-macro` normalizes `target_name` through `rust_crate_name(...)`.
- Other host key kinds keep their literal target name.

This keeps selection deterministic and bounded to Cargo-selected unit-graph keys.

## Imperative shell

No new shell behavior. Existing rust-plan orchestration and topology execution consume the corrected host graph.

## Risk

Normalizing proc-macro names could accidentally hide a mismatch between two same-package proc-macro targets whose names differ only by hyphen/underscore. Cargo crate-name rules collapse those names for rustc anyway; treat the normalized crate name as the artifact identity used by `--extern`.
