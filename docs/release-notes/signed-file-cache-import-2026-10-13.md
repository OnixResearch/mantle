# Signed Nix file-cache import: compressed NARs

`mantle store pull --from <directory>` now decodes the compression declared by
a signed narinfo before ingesting its NAR. File-cache pulls use the same gzip,
bzip2, xz, zstd (including multiple frames), and uncompressed transport
normalization as HTTP pulls. The imported NAR is checked against the signed
**uncompressed** `NarHash`; invalid compressed payloads and valid NARs with a
mismatched content hash remain unadmitted. Import still requires an accepted
trusted public key unless the operator explicitly requests `--trust-unsigned`.

This fixes previously reported `skipped_hash_mismatch` for genuine Nix
`file://` caches with `compression=zstd` where the compressed bytes were
incorrectly passed to the NAR parser. This change does not establish source
provenance, reproducibility, or sandbox execution correctness.
