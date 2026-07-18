# Design: Verified fresh-clone source hydration

## Goal

Turn a transferred, identity-pinned Mantle source bundle into the two explicit checkout-local inputs missing from a fresh clone: the locked Cargo directory source and pinned legacy provider source state. Preserve existing source-bundle validation, Cargo checksum authority, path safety, no-clobber publication, and claim boundaries.

## Functional core and imperative shell

The deterministic core validates the expected manifest BLAKE3, classifies the bootstrap profile records, requires exactly one vendored-Cargo record plus the provider archive and provider manifest records, and returns a hydration plan. It performs no filesystem mutation.

The shell reads the bundle, materializes the vendor record under a same-filesystem staging root, copies only `Cargo.lock` and `.cargo/vendor-config.toml` into that validation root, calls the existing host-tool-free vendor guard, publishes `vendor-deps` with Linux `renameat2(RENAME_NOREPLACE)`, imports and pins the verified source records, and emits a receipt. If source-state persistence fails after publication, the shell removes only the directory it just created before returning failure.

## Decisions

### 1. Reuse the Mantle source-bundle format

**Choice:** Extend the existing `mantle-source-bundle-v1` workflow with a bounded `fresh-clone-inputs` profile rather than introduce a tarball-only format or commit `vendor-deps/` to Git. The profile carries exactly the unpacked legacy provider archive, provider manifest, and vendored Cargo directory source.

**Rationale:** Source bundles already canonicalize file paths, modes, symlinks, payload bytes, BLAKE3 record identities, profile classes, provider metadata, import state, and pins. Reuse keeps one validation authority and allows the same provider archive to feed the existing offline bootstrap override. The named per-file bound increases from 16 MiB to 64 MiB because the pinned provider contains compiler executables up to about 26 MiB; payloads above the new bound still fail before allocation. Relative-path validation rejects actual empty, `.` and `..` components while permitting ordinary Cargo fixture names that contain `..` inside one component. Bootstrap archives may preserve intentional case-distinct kernel headers; post-materialization recanonicalization detects filesystems that collapse them, while portable non-bootstrap records retain the case-collision ban.

### 2. Require an out-of-band manifest identity

**Choice:** Hydration requires `--expected-manifest-blake3`; the bundle's internal digest is validated but is not sufficient by itself.

**Rationale:** An attacker who changes payload bytes can recompute a self-declared digest. The operator must bind the transferred artifact to a separately obtained BLAKE3 identity or a stronger signed publication layer.

### 3. Validate before atomic no-replace publication

**Choice:** Materialize into a temporary checkout-local validation root, run the existing `Cargo.lock`/Cargo-checksum guard there, and publish only with `RENAME_NOREPLACE`.

**Rationale:** The fresh clone never observes a partial vendor tree, malformed bundles do not create output, and an existing `vendor-deps/` is preserved even under a race. Linux is the supported hydration publication platform until another platform offers equivalent no-replace semantics.

### 4. Import and pin provider state in the same operator action

**Choice:** After vendor publication, import every verified record and pin the source-bundle manifest in the selected Mantle state directory. Roll back the newly published vendor directory if persistence fails. The bootstrap command retains the source-fetch plan, including its owned materialization tempdirs, until the asynchronous fetch build completes.

**Rationale:** A successful report should mean both explicit inputs are available: Cargo can resolve from the directory source, and the legacy provider record can satisfy `bootstrap --fetch --offline-source-preflight` without network access.

### 5. Treat imported provider permissions as immutable source data

**Choice:** The reduction shell copies read-only provider directories completely before restoring their source modes, then makes only the private reduction staging directories owner-writable. Existing destination leaves are removed before deterministic replacement.

**Rationale:** The pinned provider is store-like and read-only. Applying read-only directory modes before copying children prevents faithful acquisition; preserving those modes on the source while normalizing only private staging keeps source identity immutable and allows deterministic reduction.

## Rejected alternatives

- **Commit `vendor-deps/`:** adds roughly 811 MiB of generated source and churn to normal Git operations.
- **Rely on Cargo/Nix caches:** does not create a portable, inspectable handoff and cannot prove empty-cache behavior.
- **Use only a Nix fixed-output derivation:** makes Nix closure transfer a prerequisite and duplicates Mantle's existing source-record/pin semantics.
- **Trust only the bundle's internal digest:** provides integrity consistency but no external artifact authority.

## Evidence boundary

A positive rail uses a fresh Git clone fixture, an empty Cargo home, disabled Cargo networking, and fresh Mantle source state. It proves bundle identity verification, no-clobber materialization, Cargo lock/checksum acceptance, provider import/pinning, and offline provider preflight. The full self-hosting proof remains separately required before any stage1/stage2 fixed-point claim.

## Risks / Trade-offs

- The current JSON bundle hex-encodes payloads and is larger than a packed transport; resumable transfer can carry it, but compact chunked bundle encoding remains future work.
- Hydration is Linux-only because silent weakening of atomic no-replace semantics is not accepted.
- The profile describes explicit inputs supplied by its producer. Readiness does not prove that future bootstrap definitions have no additional undeclared source.
