## Phase 1: NAR hashing infrastructure

- [ ] Verify `write_nar` is accessible from crunch-build (check pub visibility)
- [ ] Implement `HashWriter<D: Digest>` adapter: wraps `tokio::io::AsyncWrite`, feeds bytes to hasher
- [ ] Implement `nar_hash()`: call write_nar into HashWriter, return NixHash

## Phase 2: Wire into verify_fod_hash

- [ ] Replace the `CAHash::Nar(_)` catch-all arm with calls to `nar_hash()`
- [ ] Handle Nar(Md5), Nar(Sha1), Nar(Sha512) — compute NAR hash and compare

## Phase 3: Tests

- [ ] Test NAR sha1 match and mismatch (file node)
- [ ] Test NAR md5 match and mismatch (file node)
- [ ] Test NAR sha512 match and mismatch (file node)
- [ ] Test NAR non-sha256 on directory node (multi-file output)
