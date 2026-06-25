# Tasks

- [ ] [serial] I1 Extend release evidence manifest data structures to record optional deterministic build proof and sandbox isolation evidence artifacts with relative path, BLAKE3 digest, size, and bounded role. r[rust_package_planning.bundle_deterministic_release_proof]
- [ ] [serial] I2 Update `mantle release reproduce` so requested deterministic proof runs attach proof artifacts to the release bundle and keep manifest evidence consistent with written files. r[rust_package_planning.bundle_deterministic_release_proof]
- [ ] [serial] I3 Update `mantle release verify --require-deterministic-release` to use bundle-local deterministic proof artifacts when explicit sidecar paths are absent. r[rust_package_planning.bundle_deterministic_release_proof]
- [ ] [serial] I4 Preserve explicit external deterministic proof paths as overrides and report whether verification used bundled or external proof evidence. r[rust_package_planning.bundle_deterministic_release_proof]
- [ ] [serial] V1 Add positive and negative tests for bundled deterministic proof verification, missing proof blockers, corrupted proof blockers, mismatched artifact digest blockers, and external override behavior. r[rust_package_planning.bundle_deterministic_release_proof]
- [ ] [serial] V2 Run focused Rust tests, `cargo build -p mantle --bin mantle`, `git diff --check`, Cairn validation/gates, and record evidence before archive. r[rust_package_planning.bundle_deterministic_release_proof]
