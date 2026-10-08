# ADR 0110: Separate reviewed Android metadata from source identity

## Status

Proposed (2026-10-08). This decision is not accepted. The merged reviewed
API, documentation, identity checks, and Cairn lifecycle boundary still require
verification against the published source cohort before the active Android
admission change can claim completion or archive.

## Context

[ADR 0089](0089-admit-prebuilt-android-toolchain-through-fixed-output-sources.md)
records the published Android `mantle-android-source-v1` identity. Its BLAKE3
preimage is UTF-8 `mantle-android-source-v1\n` followed by a compact JSON
array of component, version, URL, SRI SHA-256, unpack shape, and
`"x86_64-linux"`. It remains the source-admission authority at the top-level
Android API. Independently, the local APK-adapter work recorded metadata about
an exact prebuilt cohort with hex SHA-256 and an archive-kind/root pair. That
local `mantle-android-prebuilt-record-v1` preimage and its historical digests
are different from ADR 0089's preimage. Renumbering its draft ADR 0087 to
0089 would misrepresent two different identities as equivalent.

## Decision drivers

- Preserve the published source-v1 contract and its existing consumers.
- Bind APK execution planning to explicitly reviewed component metadata
  without treating an authored digest as proof of acquired bytes.
- Keep historical reviewed-cohort receipts readable without rewriting their
  source snapshots, identities, or observed results.
- Do not promote a local implementation or an unverified combined tree to an
  accepted source-admission decision.

## Proposed decision

Keep two separately named domains. The top-level Android source-v1
`cohort`, `source`, and `bind` API and its `mantle-android-source-v1` preimage
remain governed by ADR 0089. The APK adapter's independently reviewed
prebuilt metadata belongs under `android.reviewed` (exported as
`mantle.AndroidReviewedSources`) and its reviewed APK composition entry point
under `mantle.mkReviewedApk`. Neither API accepts the other domain's record
identity as its own.

The reviewed `record_identity_blake3` is BLAKE3 of the UTF-8 JSON serialization
of the ordered eight-element array
`["mantle-android-prebuilt-record-v1", component, version, url, sha256_hex,
archive_kind, archive_root, platform]`. The identity field itself is excluded.
This reviewed metadata is an exact-cohort declaration, **not** a source-v1
identity and **not** a substitute for checking the SHA-256 of fetched bytes.
Nickel does not independently calculate BLAKE3: freshness needs an external
check of the canonical preimage. The merged `android.reviewed` binder now
compares the reviewed record's full preimage and digest to its admitted
expectation, then selects one of four explicit reviewed-to-published mappings.
Its selected source-v1 cohort record is checked with the published validator:
component alias, version, URL, archive root and kind, platform, and the
reviewed hex/published SRI SHA-256 pin pair must agree. The binder delegates
the **sole executable fixed-output input** to the published `bind` path and
attaches reviewed metadata only after that binding. Byte authority remains
the fetcher's fixed-output SHA-256 check, not either recorded BLAKE3 string.
The APK plan may bind only to those admitted source facts; a reviewed string
alone does not admit a source or permit execution.

This bridge is implemented in `lib/android.ncl`, but its positive and drifted
record fixtures and final merged-source checks had not been exercised when
this decision was proposed. It is not yet acceptance evidence.

This proposal does not change ADR 0089, authorize a different upstream source
cohort, or accept the distinct reviewed schema merely because both records
refer to the same upstream archive. Evidence from an earlier `android.bind_tool`
call remains historical to that source snapshot; the merged named API needs
its own bounded verification before the associated Cairn trust-decision task
can be checked.

The reviewed-to-source-v1 bridge governs Nickel's `android.reviewed`
authoring/binding route. The canonical Rust CLI `prepare_apk` route separately
evaluates the published `android.cohort` before admission. This decision does
**not** claim that the generic Rust `PreparedApk::bind` API runs the Nickel
bridge: direct callers must supply an independently approved `Toolchain` and
the Rust binder checks the declared archive SHA-256 bytes against it. Neither
route can borrow this proposal's unverified status as evidence for the other.

## Rejected alternatives

- Rename the old reviewed metadata digest to source-v1 without re-encoding the
  preimage: the bytes and authority claims would disagree.
- Export reviewed metadata through the top-level source-v1 API: a caller could
  confuse a plan declaration with source admission.
- Treat an author-supplied BLAKE3 string or offline bundle presence as proof
  of upstream bytes, executable correctness, or signed APK behavior.
- Build OpenJDK and the Android SDK from source as a way to avoid distinguishing
  these identity domains now: that separate bootstrap project has no admitted
  implementation and cannot retroactively qualify the reviewed binary cohort.

## Consequences and non-claims

A passing reviewed-metadata freshness check would show only that the declared
metadata matches its reviewed preimage. A fixed-output SHA-256 check establishes
only the fetched bytes for that request. Neither proves upstream provenance,
license compliance, benign tool behavior, APK correctness, reproducibility, or
release eligibility. Acceptance of this ADR requires inspecting the merged
named API and exercising positive and drifted-record negative bindings, the
canonical ADR 0089 cohort cross-check, and the independently verified
fixed-output source bytes; until then the active change's ADR task and archive
gate remain open.
