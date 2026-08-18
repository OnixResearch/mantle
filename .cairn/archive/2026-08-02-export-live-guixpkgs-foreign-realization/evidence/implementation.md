# Implementation evidence

## Selected boundary

The implementation uses GuixPkgs as an external translation boundary. It does
not add a Guix evaluator or Guix narinfo signature parser. Producer-side Nix
exports the checked-in translated graph, realizes the selected root, and signs a
four-member runtime cache.

Mantle consumes only the concrete graph, package index, policy, empty source
bundle, public cache key, NARInfo facts, and NAR bytes. The consumer does not run
Nix, Guix, GuixPkgs, or `guix-transfer`.

## Provenance core changes

The live audit exposed three generic classifier gaps:

1. Store-path shebangs passed a full executable path where the resolver expected
   a store root plus suffix.
2. Every `..` suffix component failed, even when lexical normalization stayed
   inside one store root.
3. Zstd streams were unsupported, including compressed manual pages.

`crates/crunch-store/src/provenance.rs` now:

- splits store-path shebangs into a validated store root and suffix;
- normalizes `.` and `..` suffixes but rejects root escape;
- scans zstd streams with the existing container expansion limit;
- shares bounded single-payload inspection between gzip and zstd;
- records normalized suffixes in deterministic observations.

`crates/crunch-store/Cargo.toml` promotes the existing `zstd` dependency from a
test-only dependency to a runtime dependency.

Positive tests cover valid store shebangs, safe parent normalization, and zstd
payload inspection. Negative tests cover root escape and truncated zstd input.

## Final audit boundary

The fixes removed 44 false findings. One valid finding remains for Guix glibc's
`bin/mtrace`. The file has executable mode, no shebang, and invokes ambient
`perl`. Mantle keeps it unclassified and does not claim `provenance-audited`.

## Documentation and references

README and operator documents now describe the GuixPkgs producer export, proof
signing key, no-frontend consumer boundary, bounded zstd inspection, safe suffix
normalization, and audit non-claim.

The README records the adopted `guix-transfer`, GuixPkgs, and Guix-by-Nix
references. Mantle retains execution, trust, and evidence authority.
