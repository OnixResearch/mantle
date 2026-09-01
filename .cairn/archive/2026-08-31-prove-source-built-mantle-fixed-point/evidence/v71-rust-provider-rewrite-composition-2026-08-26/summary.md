# V71 run-rustc rewrite composition

## Result

V71 revalidated the V61 native prefix, completed the 340-unit translated Rust compiler closure, and completed all 393 translated-Cargo units. The serialized Cargo repair remained effective.

The run then failed before `run_rustc` execution. Action reconciliation recorded 67,221 observed events, 67,221 matches, zero denials, and 105 promotions. Its BLAKE3 is `961be60ce15534fbe0581c675b105b8b365ebf56b27dd2b2d5a49bc3a118e705`.

## Cause

The protected executable rewrite correctly changed the Cargo recipe from `$(BINDIR_S)cargo` to `$(abspath $(BINDIR_S)cargo)` before the host phase. The later all-static feature rewrite still accepted only the original relative recipe. It therefore rejected the already-protected line before executing `run_rustc`.

This was a composition defect between two fail-closed source rewrites. It was not an execution-authority denial or a compiler failure.

## Repair

The all-static rewrite now recognizes both reviewed input forms:

- the original relative Cargo recipe;
- the protected absolute Cargo recipe.

Each input maps to its corresponding `--features all-static` form. Already-normalized relative and protected forms remain idempotent. Unknown protected Cargo recipes still fail closed.

Focused positive and negative tests execute the two generated shell rewrites in their production order.

## Validation

The current validation transcript is `post-repair-validation.log`.

- Rust-provider tests: 70 passed, zero failed.
- `cargo check --bin mantle`: passed.
- Edition-2024 rustfmt check: passed.
- `git diff --check`: passed.
