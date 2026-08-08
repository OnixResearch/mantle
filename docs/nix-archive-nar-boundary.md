# Filesystem and castore NAR boundary

Mantle uses two Nix Archive (NAR) paths. Each path has a different input and a different owner.

## Filesystem observations

The `crunch-nar` crate owns selected NAR observations for host filesystem paths. It uses `nix-archive` version `0.1.0`.

Each request names these values:

- the hash algorithm;
- the case-hack policy;
- the maximum NAR byte count.

Each successful observation records these facts:

- the adapter version;
- the upstream package, version, commit, and Cargo checksum;
- the case-hack policy;
- the hash algorithm;
- the NAR size;
- the digest and disposition.

The adapter preserves Unix filename bytes, symlink-target bytes, executable mode, and canonical directory order. Regular-file payloads stay streamed.

The adapter checks the root identity before and after encoding. A changed root returns an error. This check does not prove an atomic snapshot.

Physical store verification and recursive project hashing use this path. Flat project hashes do not use NAR encoding.

## Castore streams

Adapted Snix owns NAR work that starts from castore nodes or ends in castore services. This work stays asynchronous and streamed.

This boundary includes these paths:

- native store archives;
- Nario payloads;
- HTTP cache imports;
- remote build transfer;
- shared Rust cache transfer;
- PathInfo repair;
- output persistence.

These paths use `write_nar`, `SimpleRenderer`, or `ingest_nar_and_hash`. They do not collect a complete payload for `nix-archive` decode or restore.

## Cutover rule

A production filesystem seam moves only after the required parity matrix agrees. The matrix compares bytes, size, hashes, names, modes, symlinks, order, errors, and selected races.

An unavailable `nix-store --dump` oracle has the disposition `unavailable`. It does not have the disposition `matched`.

A missing case, stale source identity, unsupported platform, or mismatch rejects the cutover. The adapter does not use silent fallback.

## Deferred restore requirements

Production restore is not part of this boundary. A future change must provide all of these controls:

1. Set named payload limits before allocation.
2. Use a fresh staging destination.
3. Remove the staging destination after a failure.
4. Use no-replace publication for the final path.
5. Run post-publication verification against the accepted NAR facts.

A descriptor-relative restore can still leave a partial destination after an error. Therefore, descriptor-relative access alone is not sufficient.

## Non-claims

Parity proves agreement for the selected cases only. It does not prove Nix, Snix, `nix-archive`, Mantle, or the host filesystem correct.

A filesystem observation does not grant PathInfo, store, source, transport, publication, or release authority.
