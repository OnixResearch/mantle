# Design: First-stage musl linker wrapper PATH

## Problem

The first-stage script correctly discovers the source-root musl compiler and writes private wrappers under `$BUILD_DIR/target-linker-bin/`, including `cc`. However, mrustc's generated Rust 1.90 compiler still asks the system linker for `cc` during the musl-host `run_rustc` smoke link. Without the private alias dir at the front of PATH, that plain `cc` resolves to the GNU host compiler and fails to find musl CRT objects and libraries.

## Approach

In `push_first_stage_target_linker_wrapper`, after writing and chmod'ing the private `cc` wrapper, prepend `$target_alias_dir` to PATH and export it. This keeps the alias scoped to the generated first-stage script while ensuring child `rustc` invocations that ask for plain `cc` use the digest-selected musl wrapper.

The wrapper still delegates to the target-prefixed compiler selected from source-root or Nix-wrapper metadata. The script continues to set target-specific Cargo/mrustc environment variables as before.

## Validation

Focused validation checks that generated first-stage scripts contain the PATH prepend immediately after wrapper creation, that source-root alias tests still pass, and that default synthetic materialization still succeeds.

A real musl-host provider attempt failure transcript is recorded as frontier evidence; a follow-up run can now test whether the PATH repair advances past the first-stage link.
