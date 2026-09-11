# Evidence: repkgs source review

## Source

- Repository: https://github.com/mic92/repkgs
- Reviewed revision: `1cd7b8b` (2026-09-10), local clone at
  `/tmp/repkgs-review` during the 2026-09-10 session.
- Local review retained: 116 packages, 50 commits at review time.

## Mechanism mapped by this change

- `builder/finish.nu`: the shared finish step — version check in an empty
  environment under an rtld-audit module, absolute-reference warnings with a
  cross `--deny` leak gate, relocation rerun from a copied output, `.la`
  removal, precompiled-header rejection.
- `nix/package.nix`: finish is part of every generated build; gates are spec
  fields with defaults and explicit opt-outs.

## Adaptation boundary

Mantle keeps its own shell and Nickel policy; no Nushell. The leak gate is
report-and-deny only in Mantle; reference rewriting belongs to
`relocate-dynamic-output-references`. Nothing from repkgs is copied; the
reviewed mechanisms inform Mantle-owned requirements.
