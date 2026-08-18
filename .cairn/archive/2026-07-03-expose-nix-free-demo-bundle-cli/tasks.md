## Implementation

- [x] [serial] I1 Add a CLI command or subcommand that reads a demo bundle machine summary and calls the pure validator. r[verification_evidence.nix_free_demo_bundle_cli]
- [x] [serial] I2 Add JSON and human output that report claimability, profile, fixed-point verdict, and stable diagnostics without leaking unrelated logs into JSON stdout. r[verification_evidence.nix_free_demo_bundle_cli]
- [x] [serial] I3 Add README rendering from the machine summary and validation result through a CLI flag or subcommand. r[verification_evidence.nix_free_demo_bundle_cli]
- [x] [serial] I4 Document the command in help/README wording with the current bounded non-claims. r[verification_evidence.nix_free_demo_bundle_cli]

## Verification

- [x] [serial] V1 Positive: validate a fixture bundle with matching stage digests and all guard denials through the CLI and assert claimable output. r[verification_evidence.nix_free_demo_bundle_cli]
- [x] [serial] V2 Negative: remove fixed-point digest evidence and assert non-zero or non-claimable CLI output with `missing-fixed-point-evidence`. r[verification_evidence.nix_free_demo_bundle_cli]
- [x] [serial] V3 Negative: remove one required guard denial and assert `missing-guard-evidence` without Nix-free claim wording. r[verification_evidence.nix_free_demo_bundle_cli]
- [x] [serial] V4 Run CLI JSON-output tests, README-rendering tests, `cargo fmt -p mantle --check`, `git diff --check`, Cairn validation, and Cairn proposal/design/tasks gates. r[verification_evidence.nix_free_demo_bundle_cli]
