## Phase 1: NAR hashing infrastructure

- [x] Verify `write_nar` is accessible from crunch-build (check pub visibility)
- [x] Implement `nar_hash()`: call write_nar into AsyncIoBridge-wrapped hasher, return NixHash

## Phase 2: Wire into verify_fod_hash

- [x] Add `directory_service` parameter to `verify_fod_hash`
- [x] Replace the `CAHash::Nar(_)` catch-all arm with calls to `nar_hash()`
- [x] Handle Nar(Md5), Nar(Sha1), Nar(Sha512) — compute NAR hash and compare

## Phase 3: Tests

- [x] Test NAR sha1 match and mismatch (file node)
- [x] Test NAR md5 match and mismatch (file node)
- [x] Test NAR sha512 match and mismatch (file node)
- [x] Test NAR non-sha256 on directory node (multi-file output)
- [x] Cross-check: nar_hash(Sha256) matches calculate_size_and_sha256
