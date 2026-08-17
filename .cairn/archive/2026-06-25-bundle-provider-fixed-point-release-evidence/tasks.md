# Tasks

- [x] [serial] I1 Extend release evidence manifest data structures to record an optional provider fixed-point proof artifact with relative path, digest, size, and bounded role. r[rust_package_planning.bundle_provider_fixed_point_release_evidence]
- [x] [serial] I2 Add `mantle release create --provider-fixed-point-proof <dir>` packaging that validates the proof before copying it into the release evidence bundle. r[rust_package_planning.bundle_provider_fixed_point_release_evidence]
- [x] [serial] I3 Update `mantle release verify --require-provider-fixed-point-proof` to use the bundle-local proof when no external `--provider-fixed-point-proof` path is supplied, while preserving external override behavior. r[rust_package_planning.bundle_provider_fixed_point_release_evidence]
- [x] [serial] I4 Report provider fixed-point proof source, digest, closure policy, stage binary digest, and bounded non-claims in human and JSON release verification output. r[rust_package_planning.bundle_provider_fixed_point_release_evidence]
- [x] [serial] V1 Add positive and negative tests for release creation with valid proof evidence, required verification from bundled proof, missing proof failure, invalid proof failure, and external override behavior. r[rust_package_planning.bundle_provider_fixed_point_release_evidence]
- [x] [serial] V2 Run focused Rust tests, build, diff check, Cairn validation/gates, then record evidence before archive. r[rust_package_planning.bundle_provider_fixed_point_release_evidence]
