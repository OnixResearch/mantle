## Design

Keep the fix in the pure registry dependency resolver. `native_dependency_source(...)` already accepts the dependency key plus manifest value and returns the selected vendored manifest path. The resolver should filter registry candidates by the manifest package name and then apply the declared version constraint when needed.

Bounded behavior:

- If a declared version requirement identifies exactly one vendored source by exact or dotted-prefix match, select that source.
- If an exact version requirement such as `=2.0.18` is present but absent from vendored sources, return a deterministic blocker.
- If no bounded version match is decisive, keep the previous deterministic same-name fallback instead of reimplementing Cargo's full semver resolver.

This fixes the observed multi-version proc-macro mismatch while avoiding broad Cargo resolver reimplementation.
