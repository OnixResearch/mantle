# Release filesystem capability boundaries

Mantle release code should treat local filesystem authority as an explicit shell
capability. The release capability boundary uses typed root roles for:

- release evidence roots;
- witness rebuild roots;
- bootstrap roots;
- build artifact roots; and
- store roots.

`release_capability::authorize_release_path` is the pure core that validates a
relative path request against a declared root authority. `ReleaseCapabilityRoot`
is the imperative shell wrapper around `cap-std` for relative reads and writes.
Pure release planning code must not open ambient paths, inspect the environment,
or resolve symlinks itself.

Passing this boundary proves only that a local file access was routed through a
declared capability root and relative path validator. It does not prove release
evidence correctness, build correctness, store correctness, or absence of bugs in
higher-level release policy.
