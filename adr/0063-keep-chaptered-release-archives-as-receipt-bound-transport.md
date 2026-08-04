# ADR 0063: Keep chaptered release archives as receipt-bound transport

## Status

Accepted

## Context

Mantle release evidence is a verified directory with a canonical manifest. Large bundles can contain source archives, binaries, proof trees, reproducibility reports, and external evidence.

A normal gzip tar archive keeps broad tool compatibility. It does not expose deterministic boundaries for direct manifest or artifact-group access.

`chapter-tgz` creates a normal gzip and tar stream with embedded chapter markers. Standard readers ignore those markers. A chapter-aware reader can seek to one chapter.

Version 0.1.0 is a new upstream release with limited independent use evidence. It requires Rust 1.91 and depends on `flate2` and `tar`.

A local pilot found that chapter access accepted an archive with one missing gzip trailer byte. A complete gzip decoder rejected the same truncation.

## Decision Drivers

- Keep the release directory and manifest as canonical authority.
- Preserve ordinary gzip and tar compatibility.
- Support direct control-chapter access.
- Detect all compressed-byte changes before chapter access.
- Keep planning pure and extraction capability-confined.
- Avoid format changes for existing interoperability surfaces.
- Keep young upstream code behind exact pins and Mantle-owned tests.

## Decision

Mantle will provide an opt-in chaptered release transport directory. The directory contains `release.tgz` and `receipt.json`.

The archive uses `chapter-tgz` version 0.1.0. Cargo locks the crates.io checksum. Mantle records the upstream release commit in lifecycle evidence.

Chapter zero contains `__mantle_release_transport_index__.json` and `manifest.json`. Later chapters group source members, immediate binary children, and other top-level artifact groups.

`crunch-release-core` owns deterministic grouping, canonical index and receipt models, limits, and diagnostics. The Mantle shell owns files, tar headers, compression, digest measurement, extraction, and publication.

The detached receipt binds the complete compressed archive BLAKE3. It also binds archive size, source manifest BLAKE3, index BLAKE3, format versions, chapter count, member count, claim scope, and non-claims.

Inspect and unpack require the receipt. They copy the archive into a private snapshot while measuring the receipt-bound BLAKE3. They then validate the complete bounded gzip stream. Chapter access starts only after both checks pass.

A raw marker-prefix count must match the receipt before `TgzReader::open`. This check bounds chapter-chain allocation under the exact 0.1.0 marker encoding.

Unpack validates archive metadata before output writes. It then extracts through capability-relative no-follow operations into a private sibling stage. Normal release verification must pass before atomic no-replace publication. Pack performs this complete round trip before it publishes the transport.

The verified release directory remains canonical. The archive and receipt provide transport identity and safe materialization evidence only.

Mantle will not apply this format to OCI layers, Android image archives, NARs, upstream source archives, the existing release source tar, or Aspen release exports without separate evidence and decisions.

Parallel decompression remains deferred. A 64 MiB benchmark measured an 8.3 percent median chapter-read improvement. Production activation requires at least 20 percent without weaker validation.

## Alternatives Considered

### Replace the release directory with one archive

Rejected because it would change canonical release evidence, mutation behavior, and existing directory-based verification.

### Trust gzip CRC during chapter access

Rejected because chapter reads do not establish complete gzip trailer validation. The detached BLAKE3 covers all compressed bytes before chapter access. Complete gzip validation also rejects malformed bytes with a rewritten receipt.

### Put the receipt inside the archive

Rejected because an archive cannot contain a non-circular digest of its final compressed bytes.

### Publish archive and receipt as independent files

Rejected because two file renames cannot publish one atomic pair. A containing transport directory supports one no-replace publication step.

### Use one chapter for each file

Rejected because many small chapters add marker and seek overhead. Deterministic role groups preserve useful selection with bounded chapter counts.

### Replace Aspen `tar.zst`

Rejected because Aspen owns an established deterministic export identity. No representative comparison currently supports that format change.

## Consequences

- Operators get pack, inspect, and unpack commands under `mantle release transport`.
- Standard gzip and tar tools can read `release.tgz`.
- Standard extraction includes the reserved transport index.
- Mantle unpack omits the reserved index and recreates the verified release directory.
- Every valid transport needs a detached receipt and complete gzip validation.
- The receipt needs an authenticated parent handoff for adversarial authority.
- Two-pass metadata validation adds read work before extraction.
- The exact dependency pin and local negative tests become part of the release transport contract.

## References

- <https://github.com/dtolnay/chapter-tgz>
- <https://docs.rs/chapter-tgz/0.1.0/chapter_tgz/>
- [ADR 0021](0021-publish-release-bundles-with-an-atomic-no-clobber-commit.md)
- [ADR 0027](0027-distinguish-transfer-content-from-chunk-occurrence.md)
