# Tasks: adopt chaptered release transport

## Phase 1: Dependency and pure core

- [x] [serial] I1 Pin `chapter-tgz` 0.1.0 in the Mantle shell, regenerate Cargo lock data, record the source identity, and add the upstream repository to README references. r[mantle.release_provenance.chapter_transport.optional]
- [x] [serial] I2 Add pure bounded transport entry, chapter plan, index, receipt, and validation models to `crunch-release-core`. r[mantle.release_provenance.chapter_transport.plan]
- [x] [parallel] I3 Add positive and negative core tests for stable grouping, input-order independence, canonical bytes, missing manifest, duplicate and reserved paths, unsupported kinds, invalid links, and exhausted limits. r[mantle.release_provenance.chapter_transport.plan]

## Phase 2: Pack and inspect shell

- [x] [serial] I4 Add no-follow source observation and deterministic tar-entry emission through the existing release capability shell. r[mantle.release_provenance.chapter_transport.pack]
- [x] [serial] I5 Implement staged no-clobber chaptered archive and detached receipt publication after normal release verification. r[mantle.release_provenance.chapter_transport.pack]
- [x] [serial] I6 Implement receipt-first archive measurement and chapter-zero inspection with complete bounded metadata comparison. r[mantle.release_provenance.chapter_transport.receipt] r[mantle.release_provenance.chapter_transport.inspect]
- [x] [parallel] I7 Add deterministic, standard gzip/tar, random-access, source-drift, truncation, tamper, malformed-index, and legacy-tgz fixtures. r[mantle.release_provenance.chapter_transport.pack] r[mantle.release_provenance.chapter_transport.inspect]

## Phase 3: Safe unpack and CLI

- [x] [serial] I8 Implement two-pass capability-confined unpack into a private sibling stage with transport-index removal and normal release verification. r[mantle.release_provenance.chapter_transport.unpack]
- [x] [serial] I9 Publish unpacked directories with atomic no-replace rename and bounded cleanup on failure. r[mantle.release_provenance.chapter_transport.unpack]
- [x] [parallel] I10 Add positive round-trip and internal-link tests plus negative traversal, absolute path, duplicate, special type, link escape, stale metadata, oversized payload, and competing destination tests. r[mantle.release_provenance.chapter_transport.unpack]
- [x] [serial] I11 Add `mantle release transport pack|inspect|unpack` CLI parsing, dispatch, human output, and JSON output. r[mantle.release_provenance.chapter_transport.optional]

## Phase 4: Documentation and evidence

- [x] [serial] I12 Add an ADR and operator documentation for format scope, receipt authentication, standard-reader behavior, safe extraction, fixed-format exclusions, dependency maturity, and non-claims. r[mantle.release_provenance.chapter_transport.validation]
- [x] [serial] I13 Add Tracey implementation and test references for every chapter-transport requirement. r[mantle.release_provenance.chapter_transport.validation]
- [x] [parallel] I14 Record the baseline, implementation evidence, dependency identity, upstream review, known limits, and exact validation outputs. r[mantle.release_provenance.chapter_transport.validation]

## Phase 5: Verification and lifecycle

- [x] [serial] V1 Run focused `crunch-release-core`, release transport, release evidence, and CLI tests with positive and negative fixtures. Save exact output in `evidence/verification.md`. r[mantle.release_provenance.chapter_transport.validation]
- [x] [serial] V2 Run Rustfmt, focused first-party Clippy, Tiger Style, locked offline metadata, standard-reader compatibility, and `git diff --check`. r[mantle.release_provenance.chapter_transport.validation]
- [ ] [serial] V3 Run Cairn validation, proposal, design, and tasks gates, Tracey coverage, sync, archive, post-archive validation, and the relevant Nix checks. Record exact pass or bounded blocker output. r[mantle.release_provenance.chapter_transport.validation]
