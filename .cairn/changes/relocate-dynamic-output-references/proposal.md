# Proposal: Relocate dynamic output references

## Why

Mantle supports configurable logical store prefixes (`/mantle/store`,
`/crunch/store`, `/nix/store`), but outputs still embed absolute references:
dynamic libraries through NEEDED/RUNPATH, wrappers through generated shell, and
prefix-sensitive smoke paths. The repository record shows the cost: wrapper
relocation fixes recipe by recipe (GCC pass4 v6→v7), a compiler pinned to one
logical prefix, and overlay fingerprints still tied to `/nix/store`.

The reviewed external reference (`mic92/repkgs`, commit `1cd7b8b`, see
`evidence/repkgs-review.md`) proves the mechanism: dependency references are
made relative to the output itself, with the dependency hash kept literal so
reference scanning and GC keep working, and rewrites happen in place without
section growth.

## What Changes

- Define a relocatable-reference contract for dynamic ELF outputs: dependency
  libraries are referenced `$ORIGIN`-relative with the dependency hash literal,
  so closure and reference scanning stay unchanged.
  r[mantle.relocatable_outputs.origin_relative_needed]
- Reserve rewrite capacity at link time (RUNPATH padding) and define a bounded
  in-place rewrite that refuses unsafe ELF shapes instead of corrupting them.
  r[mantle.relocatable_outputs.bounded_fixup_admission]
- Replace env-capturing generated wrapper scripts with a bounded launcher-record
  form: one static launcher per output plus a typed record.
  r[mantle.relocatable_outputs.record_launchers]
- Make prefix independence checkable: a relativized output must be usable under
  any configured logical prefix without rebuilding.
  r[mantle.relocatable_outputs.prefix_independence]
- Preserve scanner parity as a hard requirement: relativized outputs produce
  the same reference set as absolute outputs.
  r[mantle.relocatable_outputs.reference_scanning_parity]

## Impact

- **Immediate consumer**: the dynamic milestone outputs — GCC 10 shared C and
  C++ runtimes and their consumers — and the store-prefix migration surface.
- **Immediate outcome**: dynamic outputs stop embedding absolute logical
  prefixes, so prefix changes no longer require recipe-by-recipe repair.
- **Durable capability**: outputs that run from any path, which is also the
  precondition for content-addressed outputs without a rewriting pass.
- **Maintenance owner**: Mantle ELF-tooling owner, covering
  `src/elf_local_symbol_core.rs`, `src/stagex_archive_core.rs`, and the new
  fixup shell.
- **Repeatability evidence**: byte-level fixup fixtures, scanner parity
  fixtures, relocation runs under a second logical prefix, and negative
  controls for every refused ELF shape.
- **Compatibility**: static musl chain outputs are untouched; relativization is
  opt-in per derivation family until proven.

## Scope

The change covers the relativization contract, the link-time padding seam in
the affected bootstrap link steps, the bounded in-place fixup, the launcher
record format, scanner parity, and adoption for the GCC shared-runtime family.

## Out of Scope

- PE/Mach-O formats; Linux ELF only.
- The static musl bootstrap chain, which needs no dynamic relocation.
- Compiler correctness, full upstream-binary relocation (a later extension for
  fetched toolchains), and release eligibility.
- Changing reference-scanner semantics themselves.

## Success Criteria

- A relativized dynamic output runs from its build path and from a copy at
  another path, under at least two configured logical prefixes, without
  rebuilding.
- The reference scanner derives the same reference set from the relativized
  output as from the absolute one.
- Every unprovable ELF shape is refused with a typed error; no output is ever
  silently corrupted.
- Wrapper outputs carry launcher records instead of generated env-capturing
  shell.
