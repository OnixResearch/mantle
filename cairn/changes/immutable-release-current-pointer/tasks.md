## Release object and pointer

- [ ] [serial] Add the immutable, content-addressed release-object contract to the core. r[mantle.release.object]
- [ ] [serial] Add the single current-pointer contract and the rollback-by-identity path. r[mantle.release.pointer]
- [ ] [serial] Bind release evidence to the exact object identity and pointer value. r[mantle.release.evidence]
- [ ] [serial] Document that the caller owns distribution, deployment, retention, and deletion. r[mantle.release.boundary]

## Verification

- [ ] [parallel] Add positive cases for an immutable object, a matching identity, a valid pointer switch, and a rollback. r[mantle.release.verification]
- [ ] [parallel] Add negative cases for a mutated published object, an identity mismatch, and a pointer to a missing object. r[mantle.release.verification]
- [ ] [parallel] Add boundary cases that reject an overwrite and a deployment or readiness claim. r[mantle.release.verification]
- [ ] [serial] Run package, workspace, Clippy, Cairn, and Nix checks, then document non-claims. r[mantle.release.verification]
