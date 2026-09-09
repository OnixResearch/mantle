# Change: Complete store capability migration

## Why

Mantle has narrow store capability views, but several application shells still receive the broad `StoreHandle` and access raw PathInfo, directory, and blob services. This keeps writable store authority reachable outside the store shell and weakens compiler-enforced ownership.

Output admission also invokes configured publishers directly. The caller cannot review a publication effect plan before external publication starts, and a publisher warning is not represented as a typed execution observation.

## What Changes

- Replace broad `StoreHandle` use outside `crunch-store` and its composition boundary with operation-specific capability values.
- Keep raw Snix PathInfo, directory, blob, signing, overlay, HTTP, and mutable session services private to the store shell.
- Add Mantle-owned request, observation, result, and error types for transfer, output admission, archive access, attestation lookup, and administration.
- Separate admitted output persistence from publication planning and publisher execution.
- Return a typed publication effect plan after successful output admission.
- Record publisher execution observations without changing the admitted output result.
- Add compiler, dependency, and negative source guards that reject broad-handle or raw-service escape.
- Preserve accepted store behavior, PathInfo bytes, signatures, logical paths, reports, and CLI output.

## Non-Goals

- Replacing Snix inside the `crunch-store` adapter.
- Changing store schemas, PathInfo formats, signatures, logical prefixes, or trust policy.
- Creating one generic store trait that combines unrelated capabilities.
- Treating an effect plan as proof that publication occurred.
- Changing cache, substitution, overlay, garbage-collection, repair, or archive semantics.

## Dependencies

- ADR 0058 defines the accepted capability-limited store boundary.
- The accepted `store-lifecycle` requirements define store core and shell ownership.
- Active changes that add store operations must consume the new capability views instead of extending `StoreHandle` reach.

## Impact

- **Affected spec:** `store-lifecycle`
- **Affected code:** `crunch-store`, `crunch-pipeline`, `src/store_cmd.rs`, `src/remote_build.rs`, `src/remote_transfer.rs`, and store-backed cache or archive adapters
- **Compatibility:** accepted wire bytes, database state, reports, and command behavior remain unchanged
- **Testing:** positive capability tests, negative compile-fail tests, publication failure tests, dependency guards, focused store tests, and Cairn gates
