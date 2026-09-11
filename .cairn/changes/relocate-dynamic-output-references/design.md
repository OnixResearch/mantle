# Design: Relocate dynamic output references

## Goal and scope

Dynamic outputs stop embedding absolute logical store prefixes. References
become origin-relative with the dependency hash literal, wrappers become
launcher records, and prefix independence becomes a check, not a hope. Static
musl chain outputs are out of scope; the first family is the GCC shared
runtimes.

Planning success means a native change package with requirements, ownership,
and positive and negative tasks. Gate success proves package structure only.

## Current behavior

Outputs embed absolute paths in NEEDED, RUNPATH, PT_INTERP, generated wrapper
shell, and configured prefixes. Prefix changes require per-recipe repair; the
record shows wrapper relocation fixes, a prefix-pinned compiler, and overlay
fingerprints tied to `/nix/store`. Mantle already owns bounded ELF rewriting
code: `src/elf_local_symbol_core.rs` (fixed-width local-symbol
canonicalization) and `src/stagex_archive_core.rs` (archive rebuild). The
reference scanner and closure resolution are proven surfaces.

The external reference implements the same mechanism (`reloc-fixup`,
`crt-interp`, `launch` records; `evidence/repkgs-review.md`), including the
in-place-rewrite discipline and the symbol-tail overlap refusal.

## Approach review

| Family | Mechanism | Disposition | Required check |
| --- | --- | --- | --- |
| patchelf-style grow-and-move | Rebuild dynamic section with new strings | Rejected: changes file layout, weakens identity discipline | Byte-identity fixtures |
| In-place rewrite with link-time padding | Driver reserves capacity; fixup rewrites in place | Selected for link-owned outputs | Padded-rewrite and refusal fixtures |
| Pure post-hoc rewrite without padding | Rewrite only when capacity exists | Fallback for adopted-but-not-relinked outputs | Refusal path must be proven typed |
| Launcher records | Static launcher plus typed record replaces wrapper shell | Selected for wrapper outputs | Record admission and negative controls |

## Contract and component ownership

- Pure core: fixup planning and admission over an ELF image and a declared
  dependency map — new bounded module beside `elf_local_symbol_core.rs`; no
  I/O, no clock, no process.
- Shell: link-flag injection in the affected bootstrap link steps, the fixup
  execution, launcher installation, and the relocation check wiring.
- Policy: opt-in fields on the builder-layer contract; families adopt
  explicitly.
- The reference scanner is consumed, not modified; parity is a required test.

## Decisions

### Decision: Hash-literal relative entries

**Choice:** Relative entries keep the full `<hash>-<name>` directory literal.

**Rationale:** Reference scanning, closure resolution, and GC keep working
without scanner changes; this is the single property that makes relativization
cheap to adopt.

### Decision: No interp stub in the first slice

**Choice:** The first slice targets NEEDED and RUNPATH only; the
relative-interpreter stub for executables is a later requirement.

**Rationale:** Mantle's dynamic milestone currently links through its own
loader paths; the stub is valuable but separable, and each ELF surgery should
land with its own negative controls.

## Risks / Trade-offs

- NEEDED-as-path changes loader behavior from search to direct open; dlopen
  through RUNPATH keeps search semantics for the retained entries.
- The fixup is another ELF writer; bounded admission and refusal fixtures are
  mandatory, mirroring the local-symbol canonicalizer discipline.
- Launcher records add one static binary per output family; the first family
  builds it from the existing toolchain.
