## Why

Mantle reads and writes release evidence, witness rebuild inputs, bootstrap artifacts, and store/build metadata under operator-selected roots. Those shell edges currently rely on ambient filesystem paths plus hand-rolled path checks, which makes traversal, absolute-path, and symlink-escape behavior harder to review.

## What Changes

- Adopt `cap-std` at Mantle filesystem shell boundaries that operate under an explicit artifact, evidence, build, or store root.
- Introduce typed root wrappers around `cap_std::fs::Dir` and pass those wrappers into release-evidence, witness-rebuild, bootstrap, crunch-store, and crunch-build adapters.
- Keep Mantle pure planning, provenance models, and build decision cores independent of `cap-std`.
- Add positive and negative fixtures for valid relative paths, `../` traversal, absolute paths, and symlink escapes.

## Impact

- Filesystem authority becomes explicit at the boundary where Mantle opens an ambient root.
- Existing release and build behavior should remain stable for valid inputs.
- Invalid path shapes fail earlier with deterministic diagnostics and no broadened release-evidence claims.
