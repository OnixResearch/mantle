# ADR 0089: Admit prebuilt Android tools through pinned fixed-output sources

## Status

Proposed (in `adopt-android-prebuilt-toolchain-sources`).

## Context

Mantle's source-built bootstrap chain has neither a JVM/JDK nor Android SDK
binaries. APK production needs `javac`, `d8`, `aapt2`, and `apksigner` plus a
platform `android.jar`. Treating host-installed tools or floating SDK discovery
as build inputs would make their identities invisible to Mantle and compromise
replay. The accepted ADR 0079 already governs SpaceWasm evidence; this
Android decision deliberately uses the separately reserved number 0089.

## Decision

Admit one explicit Linux/x86_64 cohort: Eclipse Temurin JDK 17.0.17+10,
Android command-line tools 19.0 (archive 13114758), build-tools 35.0.0,
and Android API 35 platform revision 2. Each record contains its official
upstream URL, the measured SHA-256 of the downloaded archive, the unpack
root, and a BLAKE3 identity over normalized record fields. The manifest
rejects incomplete, placeholder, duplicate, out-of-cohort, and drifted
records before lowering them to single-output `builtin:fetchurl` flat
SHA-256 derivations. The SHA-256 fixed-output fetch service checks the actual
bytes; BLAKE3 binds the declaration, not executable behavior.

Canonical record bytes are UTF-8 `mantle-android-source-v1\n` followed by a
compact JSON array of `[component, version, url, sha256, unpack_shape,
"x86_64-linux"]` without a final newline. Field order in the input Nickel
record cannot change these bytes. Compare BLAKE3 against the reviewed cohort
identities rather than accepting a caller-supplied digest on faith.

An owned Android execution derivation must use the source module's identity
binding helper, declare the verified fetch derivation in its inputs, pass an
observed SHA-256, and carry the component, pinned SHA-256, and record BLAKE3
in its build environment. Execution remains inside a native sandbox. The
fixed-output finish path, not the caller's observed-digest assertion, is the
authority for resolved source bytes. The later APK adapter must connect this
helper to its actual execution/lowering path. The source admission change
alone does not intercept generic builders or assert universal enforcement
for arbitrary user-provided derivations.

The same fixed `builtin:fetchurl` URL and file-kind request participates in
existing `SourceFetchOverridePlan` offline replay. A missing kind/URL mapping
must fail rather than contact the network. No SDK binary may be run during
source fetch.

## Rejected alternative

Source-build OpenJDK and the Android SDK immediately. This needs a separate
bootstrap project far beyond the current GCC/binutils/musl chain and cannot be
substituted with an unreviewed host JVM. Do not imply that fixed-output
admission advances the source-built chain.

## Consequences and non-claims

- These inputs are third-party prebuilt binaries, **not source-built** and
  without a bootstrap-chain provenance claim.
- A content digest match does not prove binary correctness, safety,
  non-malicious behavior, reproducible upstream builds, license compliance,
  or upstream source provenance.
- Offline replay proves the declared bytes and source mappings for the
  reviewed cohort, not authenticity of the upstream publishers or APK quality.
- Tool execution, when added by the downstream APK adapter, belongs only in
  sandbox derivations with declared source inputs; neither admission nor a
  generic Nickel import itself executes an Android binary.
- The ADR index is maintained by the integration owner in one shared edit.
