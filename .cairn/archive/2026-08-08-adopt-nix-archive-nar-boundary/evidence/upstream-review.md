# Upstream review: cachix/nix-archive

## Review question

Can Mantle use `cachix/nix-archive` without weakening castore streaming, trust, mutation, or evidence boundaries?

## Inspected evidence

- Repository: `https://github.com/cachix/nix-archive`
- Reviewed source: published `nix-archive` crate `0.1.0`
- Source commit from packaged `.cargo_vcs_info.json`: `14362ab589daa4869bda744d4fbe26a1914b5491`
- Cargo registry checksum from `Cargo.lock`: `70e73d0af2e2dce844911f162414cb04cda4bca5a4847328a71034b244a6acf1`
- License in the reviewed `Cargo.toml`: `Apache-2.0`
- Retrieval date: `2026-08-07`
- Reviewed public API descriptions: `decode_events`, `decode`, `encode_tree`, `hash_tree`, `encode_path`, `hash_path`, `restore_path`, and explicit case-hack variants.
- Reviewed dependency classes: `libc`, `rustix`, `sha2`, and `thiserror`.

## Observed strengths

- Filesystem encoding preserves raw Unix path and symlink bytes.
- Filesystem encoding streams regular-file payloads to the writer.
- Canonical directory order and executable mode are part of NAR encoding.
- Filesystem traversal and restoration use descriptor-relative operations against path substitution.
- Borrowed tree encoding and hashing avoid heap allocation when the caller does not allocate.
- The upstream suite reports goldens, Nix differential tests, truncation tests, generated round trips, race regressions, and allocation counts.

## Observed limits

- The package is Unix-only.
- Borrowed decode accepts a complete NAR byte slice.
- Restore accepts a complete NAR byte slice.
- Restore is not transactional and can leave a partial destination after failure.
- The public API does not expose Mantle's async blob service, directory service, or castore node model.
- Upstream tests are evidence for the reviewed source. They do not prove Mantle integration behavior.
- Descriptor-relative traversal does not make a changing tree into one atomic snapshot.

## Approach review

### Family: wholesale replacement

- **Mechanism:** Replace Snix NAR rendering and ingest with `nix-archive`.
- **Claim:** One NAR implementation would own all Mantle paths.
- **Blocker:** The APIs do not provide asynchronous service-backed castore rendering or streaming ingest.
- **State:** Falsified for this change.

### Family: oracle-only use

- **Mechanism:** Keep production unchanged and use `nix-archive` only in tests.
- **Claim:** A second implementation would improve compatibility evidence.
- **Blocker:** This does not integrate the reviewed filesystem encoder into a useful production seam.
- **State:** Valid but too weak for the requested integration.

### Family: split boundary

- **Mechanism:** Use `nix-archive` for filesystem NAR encoding and hashing. Keep Snix for castore rendering and ingest.
- **Claim:** Mantle can remove temporary castore round trips without weakening streamed store paths.
- **Blocker:** Each moved seam needs current parity and offline dependency evidence.
- **State:** Selected.

### Family: immediate restore adoption

- **Mechanism:** Restore imported NAR bytes directly to final output paths.
- **Claim:** Descriptor-relative creation would harden materialization.
- **Blocker:** The API needs a complete byte slice and can leave partial output after failure.
- **State:** Deferred.

## Decision

Adopt the split boundary. Use one Mantle adapter for filesystem NAR work. Keep Snix for castore NAR work.

Do not use production decode or restore in this change. A later restore proposal must add bounded staging and no-replace publication.

## Owner

Mantle store and project maintainers own the adapter, parity gate, dependency review, and production cutover.

## Next action

Run the recorded baseline tests before dependency or production changes.

## Non-claims

This review does not prove `nix-archive`, Snix, Nix, Mantle, the host filesystem, or generated fixtures correct. It does not grant store, PathInfo, source, or release authority.
