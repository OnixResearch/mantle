# ADR 0079: Stable SpaceWasm evidence uses the libtest JSON harness grammar

## Status

Accepted (in the `stabilize-spacewasm-bundle-evidence` change).

## Context

The SpaceWasm reference producer hashes raw Cargo/libtest text output into
receipts and bundle members. Compilation order, test completion order, and
elapsed times change those hashes across identical derivations, so the
bundle is not reproducible (see the retained same-derivation rebuild
failure in the change baseline).

The pinned toolchain used by the SpaceWasm reference Nix lane is a stable
Rust (`rust-bin.stable`), so libtest's JSON format needs the explicit
`RUSTC_BOOTSTRAP=1` environment plus `-Z unstable-options`. This is a
bounded, pinned exception: the flag is part of the recorded command
identity, the env var is set only for the two test producers, and any
toolchain change that alters the grammar requires a new encoding version
and a reviewed contract migration.

## Decision

The stable report consumes a closed, versioned structured grammar: libtest
JSON harness lines (`type: test` and `type: suite` objects only, with
`deny_unknown_fields` admission and bounded counts). The stable identity
covers the schema version, suite, command identity, canonical
(name-ordered) test records with their outcomes, and the encoding version.
Durations, order, and all other presentation fields are excluded.

A text-output adapter with scraped summary lines is rejected: human text
has no closed grammar, binds incidental formatting, and cannot prove that
"admitted facts" are anything more than the final summary. The structured
format makes unknown or incomplete captures explicit admission failures.

Raw stdout/stderr captures remain run evidence, archived separately by the
shell and bound to the stable identity one-way. They never enter stable
identity computation.

## Consequences

- The producer's test invocations gain `-- --format json -Z unstable-options`.
- Consumers verify stable identity over canonical facts; raw logs remain
  available for diagnosis but are no longer bundle identity inputs.
- A toolchain change that alters the libtest JSON grammar requires a new
  encoding version and a reviewed contract migration.
- The pinned upstream source must support the nightly `-Z unstable-options`
  flag; it is built with the pinned nightly toolchain, so this holds.
