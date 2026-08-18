# Design: Native rlib crate-type derivation planning

Cargo unit graph target `kind` arrays can contain crate-type-oriented values such as `rlib` and `cdylib` instead of the broader `lib` class. Mantle's direct `rustc` rail already produces rlib-style dependency artifacts for library consumers.

The bounded rule is:

- If a target kind array contains `custom-build`, classify it as host `custom-build`.
- Else if it contains `proc-macro`, classify it as host `proc-macro`.
- Else if it contains `lib` or `rlib`, classify it as target `lib`.
- Else if it contains `bin`, classify it as target `bin`.
- Otherwise emit the existing deterministic unsupported-target blocker.

This supports mixed `[cdylib, rlib]` targets only through their rlib/library facet. It does not claim cdylib output production.
