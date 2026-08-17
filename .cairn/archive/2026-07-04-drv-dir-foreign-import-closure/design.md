## Decisions

### 1. Directory inputs are explicit bundles, not host-store discovery

**Choice:** `--drv-dir` reads direct child files whose names end in `.drv` and maps each basename to `/nix/store/<basename>`. It does not walk `/nix/store`, call `nix-store`, recurse through arbitrary host paths, or infer closure members outside the provided directory.

**Rationale:** A bundle directory is portable and deterministic. Host store discovery would reintroduce the coupling the foreign import boundary is avoiding.

### 2. Closure selection is pure and root-reachable

**Choice:** After the CLI shell reads and parses files, a pure helper selects the root-reachable derivation closure by following parsed `inputDrvs`. The emitted graph contains the reachable closure only; unrelated `.drv` files in the bundle do not perturb graph identity.

**Rationale:** Bundle directories often contain more files than one selected root needs. Root-reachable filtering keeps artifacts stable for a selected package and keeps closure completeness checks in testable core logic.

### 3. Missing inputs fail before artifact writes

**Choice:** If a reachable derivation references an input `.drv` not present in the parsed bundle, artifact production returns a deterministic diagnostic and writes no graph/index files.

**Rationale:** Partial graphs are dangerous: they can hide dependencies and produce receipts that look complete. The failure must happen before any artifact side effects.

## Risks / Trade-offs

- Directory mode assumes filenames are the logical Nix store basenames. Bundles with renamed files should continue using explicit `--drv logical=file` mappings.
- The directory reader is intentionally non-recursive for now; recursive bundle formats can be added later with a manifest if needed.
