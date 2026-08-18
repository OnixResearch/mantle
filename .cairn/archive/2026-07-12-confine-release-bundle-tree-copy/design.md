## Context

`collect_paths_sorted` appends each child and uses `Path::is_dir` to decide recursion. That query follows symlinks. For a directory symlink, the collected list therefore contains the symlink and paths beneath its target. `copy_tree_entry` recreates the symlink first; later destination joins for the collected descendants traverse the newly created symlink and write outside the bundle root.

The fix must cover discovery, planning, mutation, and hashing. Merely changing one metadata call leaves destination traversal and source type-of-check/type-of-use races unaddressed.

## Decisions

### Discover entries without following links

The shell enumerates one directory handle at a time and records each child's relative path, no-follow file type, mode, and symlink target when applicable. It never invokes recursive directory traversal for a symlink, even when the target currently names a directory. Unsupported special files are observations that cause planning to fail.

### Validate a pure tree-copy plan

A pure planner sorts normalized observations and validates path components, uniqueness, parent relationships, entry/depth limits, supported file kinds, and symlink policy. The resulting plan contains explicit create-directory, copy-file, and create-symlink operations in an order that cannot create a traversable destination parent before later writes.

The planner has no filesystem authority. The shell owns enumeration, open handles, byte copying, metadata revalidation, and error plumbing.

### Limit supported symlinks

A supported symlink target is relative, normalizes from the symlink's parent to a path within the planned source root, and names another planned entry. Symlinks are copied as links and are never followed during traversal, hashing, copying, or verification. Absolute targets, parent escapes, targets outside the plan, and type drift fail closed.

### Keep destination operations beneath one capability root

The shell creates or opens a trusted destination root and performs every operation relative to that capability. It rejects a symlink root, symlinked destination parent, or any existing path component that would be followed. Before each operation it confirms that the source entry still has the planned no-follow kind and that destination parents are real directories owned by the operation.

File and directory hashing consumes the same no-follow entry model, so copied symlink targets are hashed as target text rather than traversed content.

### Prove non-escape with sentinels

The regression suite constructs the audited exploit shape: a source directory symlink points to a tree containing descendants, while the copied destination symlink points outside the destination root. The test asserts deterministic failure and byte-for-byte preservation of an external sentinel. Additional cases cover absolute and `..` targets, a pre-existing destination symlink, source type swaps, special files, and excessive entry/depth observations.

## Risks / Trade-offs

- Existing proof trees with escaping, dangling, or platform-unsupported symlinks will be rejected.
- Race-resistant capability-relative operations are more explicit than `std::fs` path joins but are required to make confinement an execution property rather than a lexical check.
- This change secures local tree handling; it does not establish trust in artifact contents.
