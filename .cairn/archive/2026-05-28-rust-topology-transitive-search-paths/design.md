# Design: Transitive native rustc search paths

## Context

`produced_target_artifacts: BTreeMap<String, PathBuf>` is correct for selecting one direct artifact for a package dependency placeholder. It is not sufficient for rustc metadata search paths because rustc may need the exact parent directory of a transitive dependency artifact embedded in an upstream rlib's metadata. Cargo can schedule multiple same-package unit variants, so package-keyed maps can drop a still-needed earlier variant path.

## Design

Keep two concepts separate:

1. Direct binding maps (`package_id -> artifact path`) for replacing `artifact:<package>:<crate>` placeholders.
2. Search path history (`Vec<PathBuf>` or equivalent ordered collection) for every successfully produced target-lib/proc-macro artifact path.

After normal direct binding, append deterministic `-L dependency=<parent>` entries from the full search path history. The append helper deduplicates against already-present search args so repeated calls remain stable.

## Non-goals

This does not solve all same-package variant identity problems in direct dependency placeholders. It fixes the observed transitive metadata search-path loss while preserving existing direct binding behavior.
