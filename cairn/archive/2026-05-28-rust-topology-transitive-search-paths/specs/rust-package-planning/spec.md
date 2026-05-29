## ADDED Requirements

### Requirement: Native topology preserves transitive rustc dependency search paths

r[rust_package_planning.native_transitive_search_paths] Native Rust topology execution MUST preserve every produced target-library and proc-macro artifact directory as a rustc `-L dependency` search path until downstream units that may load transitive metadata have executed, and MUST NOT reduce that search set to one directory per package ID.

#### Scenario: duplicate package variants keep both search directories

GIVEN native topology execution has produced two library artifacts for the same package ID from distinct unit variants
WHEN a later Rust unit is prepared for direct rustc execution
THEN its rustc arguments MUST include `-L dependency` entries for both produced artifact parent directories.

#### Scenario: repeated search paths are deduplicated

GIVEN a Rust unit already contains a `-L dependency` entry for a produced artifact directory
WHEN native topology execution appends full produced search-path history
THEN the rustc arguments MUST NOT contain duplicate `-L dependency` pairs for that directory.
