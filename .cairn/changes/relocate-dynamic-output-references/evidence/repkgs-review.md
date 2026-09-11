# Evidence: repkgs source review

## Source

- Repository: https://github.com/mic92/repkgs
- Reviewed revision: `1cd7b8b` (2026-09-10), local clone at
  `/tmp/repkgs-review` during the 2026-09-10 session.

## Mechanism mapped by this change

- `pkgs/ji/jig/src/fixup_mode.cc`: in-place NEEDED/RUNPATH relativization;
  `$ORIGIN/../../<hash>-<name>/lib` entries keep the dependency hash literal so
  Nix reference scanning keeps working; symbol-tail overlap in merged dynstr
  is detected and refused, never rewritten.
- `pkgs/ji/jig/src/driver.cc` `RunpathList`: link-time RUNPATH padding slack
  per store entry, so the post-link rewrite never grows a section.
- `builder/launchers.nu` and `pkgs/cr/crt-interp/src/crt_interp.c`: launcher
  records replacing shebang patching and `makeWrapper`; a 300-byte
  relative-interpreter stub (not adopted in Mantle's first slice).

## Adaptation boundary

Mantle reuses the mechanism shape, not the code: its own bounded ELF core
beside `elf_local_symbol_core.rs`, its own scanner-parity requirement, and its
GCC shared-runtime family as the first adoption target.
