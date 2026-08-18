# Proposal: Audit realized foreign outputs from castore facts

## Why

A successful foreign build proves that Mantle observed outputs and admitted store facts. It does not prove that every executable payload uses translated dependencies.

Builder scripts, ELF files, shebangs, symlinks, archives, and initrds can retain foreign store paths. The current build-correctness validator checks supplied reference observations, but no production scanner generates those observations from castore.

Mantle needs a bounded, castore-backed provenance audit. It must detect untranslated references and unclassified executable payloads without promoting the result into package correctness.

## What Changes

- Add a pure classifier for executable, script, symlink, archive, initrd, and data payload facts.
- Add a thin castore walker that generates bounded observations from realized closure objects.
- Resolve shebangs and executable dependency references against admitted closure identities.
- Reject or report untranslated foreign store paths, missing targets, path escapes, malformed payloads, and unclassified executable content.
- Emit `mantle-foreign-provenance-audit-v1` with scanned identities, findings, limits, disposition, and non-claims.
- Add an audited state that remains separate from import admission and realization success.

## Dependencies

- `realize-foreign-derivation-adapter`.

## Non-Goals

- Proving executable behavior, compiler correctness, source correctness, or package correctness.
- Proving complete ELF dynamic-link behavior on every platform.
- Proving archives or initrds are safe to execute or boot.
- Mutating, repairing, deleting, or quarantining output content.
- Claiming OS activation, VM boot, deployment safety, or bootstrap parity.

## Impact

- **Files**: a pure provenance classifier, `crates/crunch-store/` castore traversal, `src/build_correctness.rs`, foreign realization reports, CLI tests, trust-model documentation, and lifecycle evidence.
- **Testing**: bounded traversal, ELF and script classification, shebang resolution, symlink handling, archive recursion, untranslated references, malformed payloads, incomplete closure, deterministic findings, and receipt non-claims.
