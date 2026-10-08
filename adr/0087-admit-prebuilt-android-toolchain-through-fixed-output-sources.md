# ADR 0087: Admit prebuilt Android toolchain through fixed-output sources

## Status

Proposed

## Context

Mantle's source-built bootstrap chain has no JVM or Android SDK tools. APK assembly needs `javac`, `d8`, `aapt2`, `apksigner`, and a platform `android.jar`. Building OpenJDK and the SDK from source is a different, substantially larger project. Mantle already has `builtin:fetchurl`, fixed-output verification, source bundles, and an offline `SourceFetchOverridePlan` handoff.

## Decision Drivers

- Make third-party prebuilt inputs explicit and digest-bound instead of discovering ambient SDK installations.
- Refuse a mismatched source before any consumer tool runs.
- Preserve offline replay without extending network authority into Nickel evaluation.
- Keep provenance and bootstrap claims within what the evidence actually proves.

## Decision

Admit an initial explicitly versioned Linux x86_64 cohort: Temurin JDK 17, Android command-line tools, one build-tools release, and a platform archive containing `android.jar`. Each record declares an exact upstream URL, SHA-256 of the downloaded bytes, component name/version, unpack layout, and platform. A BLAKE3 metadata identity binds the canonical normalized component/version/URL/SHA-256/layout/platform fields, excluding the identity itself. Nickel validates the declared record and emits an existing fixed-output `builtin:fetchurl` derivation. SHA-256 is the fetched-byte pin; BLAKE3 identifies the metadata record, not the bytes directly. A caller that executes a tool must bind its identity and fetch output as a declared derivation input; digest mismatches are rejected before tool execution. Network access belongs only to the fetch effect, never to Nickel validation.

The source bundle/override seam may substitute matched fetches for offline replay; an unmatched request fails closed. The store remains the authority for fetched bytes. Do not build a parallel SDK store, accept an ambient installation, or use a placeholder digest as a pin.

The rejected alternative is source-building OpenJDK and Android SDK now: the existing chain cannot support it without its own bootstrap design and proof. This ADR does not prohibit a future source-built replacement.

## Consequences

Admission proves matching content digests for selected third-party binary archives only. Nickel 1.17 does not compute BLAKE3, so a deterministic external freshness check must recompute the reviewed metadata identities before accepting manifest changes; runtime Nickel binding compares the entire canonical metadata preimage and reviewed recorded digest, not an independently recomputed BLAKE3. It does not establish source provenance, absence of malicious behavior, distribution-license compliance, source-built status, bootstrap-chain integrity, APK correctness, or release eligibility. Tool execution through this adapter requires a sandboxed derivation and declared inputs; offline fixture replay proves only its matched requests and bytes. An ADR alone does not prove an actual tool execution or archive this Cairn.
