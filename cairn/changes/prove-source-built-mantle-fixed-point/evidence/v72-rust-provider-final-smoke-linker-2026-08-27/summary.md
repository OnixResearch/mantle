# V72 final Rust smoke linker authority

## Result

V72 revalidated the V61 native prefix, completed the 340-unit first Rust compiler closure, completed all 393 serialized translated-Cargo units, and built Cargo through the protected `run_rustc` path. This proves that the V71 rewrite-composition repair crossed its target boundary.

The final `run_rustc` smoke failed closed. Reconciliation recorded 88,014 observed events, 88,009 matches, five denials, and 172 promotions. Its BLAKE3 is `9294d9f53c44c0d89d6ece258481e537801986a457a4d8651006348005a1065e`.

## Cause

The final hello-world recipe executed `output/prefix/bin/rustc` through the relative `$(BINDIR)rustc` spelling. That compiler then probed two nonexistent sysroot linker paths twice before it fell back to the receipt-bound target linker wrapper.

The audit therefore contains:

- one denied relative final-rustc execution;
- four denied nonexistent `cc` probes;
- a later allowed execution of the receipt-bound target linker wrapper.

Fallback success does not cancel denied action evidence.

## Repair

The source rewrite now requires the exact reviewed final-smoke recipe and changes it to:

- execute `$(abspath $(BINDIR)rustc)`;
- pass `-C linker=$(MANTLE_TARGET_LINKER)` explicitly.

The host `run_rustc` Make call binds `MANTLE_TARGET_LINKER` to the absolute receipt-bound `target-linker-bin/cc` path. Unknown or missing final-smoke recipes fail before protected execution.

Positive coverage checks the exact protected recipe. Negative coverage rejects an unknown final-smoke recipe.

## Validation

The current validation transcript is `post-repair-validation.log`.

- Rust-provider tests: 71 passed, zero failed.
- `cargo check --bin mantle`: passed.
- Edition-2024 rustfmt check: passed.
- `git diff --check`: passed.
