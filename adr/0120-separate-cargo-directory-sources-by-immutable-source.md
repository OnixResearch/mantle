# ADR 0120: Separate Cargo directory sources by immutable source

- Status: Accepted
- Date: 2026-09-04

## Context

Mantle directly pins Artifact Auth revision `c932138d880ddf4c2967f4c024b489b5c0022bf1`.
Valence revision `e40c76b4d2070a29636e00c85c0dff93f03dba2f` pins Artifact revision
`e41340bec587b6d049b5cc518ec7db925dde84be`.

Both revisions publish `artifact-auth-core` version `0.1.0`. Cargo rejects one
directory source that contains the same package name and version from two
sources. Mantle must not change either accepted source revision to hide this
conflict.

## Decision

Keep both immutable source revisions. Route them to separate Cargo directory
sources in `.cargo/vendor-config.toml`.

The primary `vendor-deps/` source contains registry packages, other Git
packages, and the Artifact revision used by Valence. The nested
`vendor-deps/.artifact-auth-c932/` source contains Mantle's directly accepted
Artifact Auth packages.

The host-tool-free vendor guard parses the Cargo source map. It requires each
directory source to stay under `vendor-deps/`. It maps every locked source to
exactly one directory root and validates each package and checksum in that
root. The native Rust planner also reads `.cargo/vendor-config.toml` so both
roots are explicit planning inputs.

The native planner matches Cargo dependency references without the resolved
Git fragment against the full locked package source. Package keys retain the
resolved commit. Explicit vendor routes select one directory for that source.
A missing payload cannot fall back to another directory. Without an explicit
route, a package lookup must find exactly one matching directory source.

Relative normal and build dependencies inside a Git package use captured
package facts from the same full Git source identity. They do not depend on
the original sibling directory spelling after Cargo creates versioned vendor
directories. Missing, ambiguous, cross-revision, and incompatible-version
candidates reject. Absolute Git dependency paths also reject.

The bounded config reader accepts direct directory sources and one replacement
alias within the same file. It rejects conflicting routes, missing aliases,
config files above 65,536 bytes, and more than 1,024 source routes.

## Consequences

- Mantle preserves both accepted revisions without a patch, mutable reference,
  ambient Cargo cache, or source-identity substitution.
- A duplicate package name and version is accepted only when Cargo routes the
  two immutable sources to different checked directory roots.
- Missing mappings, one source mapped to multiple roots, two duplicate packages
  in one root, path escapes, symlink roots, stale files, and checksum changes
  fail before self-build.
- `vendor-deps/` remains one source-bundle and hydration payload. The nested
  root does not add a second publication authority.
