## Design

Mantle will open ambient filesystem roots only at the CLI or adapter shell using `cap_std::fs::Dir::open_ambient_dir`. Downstream code receives narrow typed wrappers such as release-evidence, bootstrap, witness-rebuild, build-artifact, and store roots. Those wrappers expose only relative-path operations needed by the existing behavior.

The pure build and provenance cores continue to accept in-memory data, object identities, and validated relative locators. They must not depend on `cap-std`, inspect ambient paths, or open files directly.

The first conversion targets are `src/release_evidence.rs`, `src/witness_rebuild.rs`, `src/bootstrap.rs`, `crates/crunch-store`, and `crates/crunch-build`. The conversion should replace parent-directory and absolute-path checks with capability-relative opens, while preserving stable diagnostics for valid and invalid release evidence.

Tests must cover successful reads/writes under the declared root and negative cases for `../` traversal, absolute path input, missing root authority, and symlink escape attempts. Documentation should state that `cap-std` bounds local filesystem authority only; it does not prove build correctness, artifact truth, release eligibility, or external evidence semantics.
