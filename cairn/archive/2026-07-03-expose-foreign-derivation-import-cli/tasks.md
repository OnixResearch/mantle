## Implementation

- [x] [serial] I1 Add a `foreign-import` CLI command with `validate` and `plan` subcommands that read graph, package-index, and policy files in the shell. r[foreign_derivation_import.operator_cli_surface]
- [x] [serial] I2 Deserialize inputs into the existing pure core types and keep filesystem access, JSON parsing, and output rendering outside the translation core. r[foreign_derivation_import.cli_file_boundary]
- [x] [serial] I3 Add checked-in Guix-like and Nix-like hello fixture files plus expected receipt/plan snapshots where stable. r[foreign_derivation_import.checked_fixtures]
- [x] [serial] I4 Document CLI usage and receipt non-claims in operator docs or README. r[foreign_derivation_import.operator_cli_surface]

## Verification

- [x] [serial] V1 Positive: validate and plan both checked-in hello fixtures and assert deterministic receipt/plan output. r[foreign_derivation_import.checked_fixtures]
- [x] [serial] V2 Negative: reject malformed JSON, unsupported metadata, stale receipt input, undeclared embedded rewrites, untrusted cache hints, and undeclared sandbox capabilities through the CLI. r[foreign_derivation_import.operator_cli_surface]
- [x] [serial] V3 Boundary: prove CLI consumption does not execute `guix`, `nix`, flake evaluation, Nix expression evaluation, overlays, or package-module evaluation. r[foreign_derivation_import.cli_file_boundary]
- [x] [serial] V4 Run focused CLI tests, `cargo fmt -p mantle --check`, `git diff --check`, Cairn validation, and Cairn proposal/design/tasks gates. r[foreign_derivation_import.operator_cli_surface]
