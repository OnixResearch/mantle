# Design: Prebuilt Android toolchain source admission

## Context

APK tooling needs a JVM and Android SDK binaries that Mantle cannot source-build today. The honest boundary is the same one Nix uses: fetch pinned prebuilt artifacts as fixed-output derivations, confine their execution to sandbox derivations, and record that they are not source-built.

Mantle's existing fetch path makes this a thin admission change: `builtin:fetchurl` derivations already flow through `FetchBuildService`, fixed-output verification already rejects digest drift, and `SourceFetchOverridePlan` already supports offline replay of matched kind and URL requests.

Prior art in the Nix family confirms the shape. Robotnix builds full Android (AOSP) images with Nix and admits prebuilt SDK components as pinned fixed outputs; its signing path keeps keys explicit rather than ambient. Its current alpha state after a long unmaintained period is itself evidence for pinned-manifest admission with explicit identities instead of floating SDK discovery. Robotnix's OS-level scope stays outside this adapter family.

## Architecture

```text
typed Nickel source manifest (lib/android/sources.ncl)
  -> fixed-output fetch derivation per component
  -> toolchain identity record (component, version, digests)
  -> derivation declares identity as an input
  -> sandbox execution sees only admitted bytes
```

### Source manifest

The manifest is a typed Nickel contract in `lib/android/sources.ncl`. Each component record carries:

- component name and exact version;
- upstream URL;
- SHA-256 (the fetch interop requirement);
- BLAKE3 record identity for Mantle-owned artifact identity;
- unpack shape (single archive, nested directory layout);
- declared platform (linux, x86_64).

The initial cohort is an implementation decision recorded in the manifest, not in requirements: Temurin JDK 17 (an LTS release), Android SDK command-line tools, one pinned build-tools release, and one platform `android.jar`. Requirements pin the record shape, not the versions.

### Toolchain identity binding

The source module exposes a fail-closed helper requiring the reviewed identity, the observed archive digest, and a native sandbox derivation; it adds the fixed-output fetch derivation as an input. The downstream APK adapter MUST use this helper at its actual execution/lowering boundary. Admission does not intercept generic Nickel derivations or establish a universal enforcement claim before that consumer exists.

This keeps the store as the authority for bytes. The manifest is an index, not a second store.

### Offline replay

Admitted records use the same fetch request shape as existing bootstrap sources: kind and URL matching against `SourceFetchOverridePlan`. After one connected fetch, a proof run exports the records through source-bundle export and replays them offline. Unmatched kind or URL requests fail before live acquisition, exactly as bootstrap fetches do today.

### Trust boundary and non-claims

The admitted components are prebuilt third-party binaries. Admission proves content digest match only. It does not prove:

- absence of malicious behavior in the binaries;
- source provenance of the binaries;
- license compliance for downstream distribution;
- any source-built or bootstrap-chain claim.

Execution through the downstream adapter stays inside bwrap sandbox derivations with only declared inputs mounted. ADR 0089 records this decision and the rejected alternative (source-building OpenJDK now).

## Failure and abuse controls

- Missing or placeholder digest: manifest contract rejects the record.
- Digest drift between manifest and fetched bytes: fixed-output verification fails the build.
- Undeclared tool execution: the binding helper refuses a missing identity; universal use is an open obligation for the downstream APK adapter, not a property of arbitrary generic derivations.
- Network dependence in proofs: unmatched override requests fail closed instead of silently fetching.

## Testing

- Nickel contract tests for the manifest: complete record accepted; missing URL, missing SHA-256, empty version, unsupported platform, and duplicate component rejected.
- Identity binding unit tests: matching identity resolves; drifted digest is rejected before execution.
- Offline replay test using the existing `file://` fixture pattern.
