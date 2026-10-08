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
- BLAKE3 metadata record identity over a canonical ordered JSON array of schema, component, version, URL, SHA-256, archive type, root, and platform (excluding the identity itself);
- unpack shape (single archive, nested directory layout);
- declared platform (linux, x86_64).

The initial cohort is an implementation decision recorded in the manifest, not in requirements: Temurin JDK 17 (an LTS release), Android SDK command-line tools, one pinned build-tools release, and one platform `android.jar`. Requirements pin the record shape, not the versions.

### Toolchain identity binding

A derivation that executes a prebuilt tool MUST reference an admitted identity record as a declared input. The reviewed-cohort binding helper chooses the expected reviewed record from the checked-in manifest and compares the entire canonical metadata preimage and digest to the admitted reviewed record. Its finite four-component bridge then checks the alias, version, URL, archive root and kind, platform, and explicit hex/SRI SHA-256 pin pair against a validated published ADR 0089 source-v1 cohort record. The published `bind` path supplies the sole executable fixed-output fetch as a declared input; reviewed metadata is attached afterward. A separate freshness rail recomputes the BLAKE3 identity of every reviewed record; Nickel 1.17 cannot compute BLAKE3 itself, so this rail MUST be run before relying on a changed manifest. The fetcher's SHA-256 independently rejects drift of acquired bytes before the consumer runs. The BLAKE3 comparison at Nickel lowering is a reviewed metadata binding, not an independent runtime BLAKE3 recomputation. This merged bridge is implemented but its fixtures and fixed-output boundary have not yet passed integration verification.

This keeps the store as the authority for bytes. The manifest is an index, not a second store.

### Offline replay

Admitted records use the same fetch request shape as existing bootstrap sources: kind and URL matching against `SourceFetchOverridePlan`. After one connected fetch, a proof run exports the records through source-bundle export and replays them offline. Unmatched kind or URL requests fail before live acquisition, exactly as bootstrap fetches do today.

### Trust boundary and non-claims

The admitted components are prebuilt third-party binaries. Admission proves content digest match only. It does not prove:

- absence of malicious behavior in the binaries;
- source provenance of the binaries;
- license compliance for downstream distribution;
- any source-built or bootstrap-chain claim.

Execution stays inside bwrap sandbox derivations with only declared inputs mounted. ADR 0089 records the published source-v1 fixed-output decision and rejects source-building OpenJDK now; proposed ADR 0110 records the reviewed prebuilt-record-v1 metadata identity with its separate preimage, not an alias for ADR 0089's source-v1 identity. ADR 0079 is allocated to SpaceWasm evidence.

## Failure and abuse controls

- Missing or placeholder digest: manifest contract rejects the record.
- Digest drift between manifest and fetched bytes: fixed-output SHA-256 verification fails the build. A copied BLAKE3 string alongside changed metadata is also rejected by the complete canonical-record comparison; the freshness rail checks the reviewed record's digest.
- Undeclared tool execution: the Android binding API does not return an execution path without the reviewed identity and fixed-output fetch input. This does not ban separately authored arbitrary derivations elsewhere in Mantle.
- Network dependence in proofs: unmatched override requests fail closed instead of silently fetching.

## Testing

- Nickel contract tests for the manifest: complete record accepted; missing URL, missing SHA-256, empty version, unsupported platform, and duplicate component rejected.
- Identity binding unit tests: matching identity resolves; drifted digest is rejected before execution.
- Offline replay test using the existing `file://` fixture pattern.
