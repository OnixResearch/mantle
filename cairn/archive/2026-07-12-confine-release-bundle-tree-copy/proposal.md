## Why

Release bundle tree collection classifies children with `Path::is_dir`, which follows directory symlinks. The copier first recreates the symlink in the destination and then copies the recursively collected descendants through that symlink. A proof-bundle input can therefore make release creation write outside the declared bundle destination.

This violates Mantle's accepted release capability boundary and turns untrusted release input shape into ambient write authority.

## What Changes

- Traverse release input trees with no-follow metadata and never recurse through symlinks.
- Build a pure, deterministic copy plan from normalized entry observations before filesystem mutation.
- Preserve only supported in-tree relative symlinks and reject absolute, parent-escaping, dangling-policy, special-file, and type-drift cases deterministically.
- Execute destination mutations through a capability-relative root without following destination symlinks or traversing an unverified parent.
- Add positive nested-tree and internal-symlink fixtures plus negative source-symlink, destination-symlink, traversal, type-swap, and external-write regression fixtures.

## Impact

- **Files**: release evidence tree traversal, hashing/copy helpers, capability-root shell code, fixtures, tests, and diagnostics.
- **Compatibility**: proof trees containing escaping or otherwise unsupported symlinks are rejected instead of copied.
- **Security**: every accepted release bundle write remains confined to the declared destination root.
- **Claims**: safe copying proves bounded path and byte handling only; it does not validate the semantics of copied proof or release artifacts.
