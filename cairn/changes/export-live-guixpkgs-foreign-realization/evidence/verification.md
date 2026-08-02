# Verification evidence

Date: 2026-08-02

The repository used this external Cairn policy:

`/home/brittonr/git/OnixResearch/cairn/cairn-policy/generated/cairn-policy.json`

The repository-local generated policy lacks `nominal_identity_policy`.

## Focused checks

| Command | Result |
|---|---|
| `nix develop -c cargo test -p crunch-store provenance` | pass, 21 tests |
| `nix develop -c cargo test -p crunch-store http_closure` | pass, 26 tests |
| `nix develop -c cargo test -p mantle --bin mantle foreign_derivation_import` | pass |
| `nix develop -c cargo test -p mantle --bin mantle foreign_realization` | pass |
| `nix develop -c cargo test -p mantle --bin mantle foreign_realization_receipt` | pass |
| `nix develop -c cargo test -p mantle --bin mantle foreign_provenance_audit` | pass |
| `nix develop -c cargo test -p mantle --test foreign_import_cli` | pass, 14 tests |

The provenance tests ran before and after the core changes. The final rail also
ran after the independent review added the ambient-Perl negative case.

## Strict checks

| Command | Result |
|---|---|
| `nix develop -c cargo fmt --all -- --check` | pass |
| `nix develop -c cargo check --workspace --all-targets` | pass |
| `nix develop -c cargo clippy -p crunch-store --lib --no-deps -- -D warnings` | pass |
| `nix develop -c cargo clippy -p mantle --bin mantle --test foreign_import_cli --no-deps -- -D warnings` | pass |
| `nix develop -c check-nickel-configs` | pass, including expected negative fixtures |
| `nix develop -c cargo -Zscript scripts/check-foreign-import-trust-model.rs` | pass |
| `nix develop -c cargo -Zscript scripts/check-foreign-import-trust-model.rs --self-test` | pass |
| `nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs --self-test` | pass |
| `git diff --check` | pass |

The full machine-schema check retains 39 unrelated StageX and source-build
inventory findings. It reports no foreign import, cache closure, realization,
or provenance finding. `machine-schema.log` retains the exact baseline output.

## Cairn pre-sync status

Strict validation passed. Proposal, design, and tasks gates returned `PASS` and
`valid: true`.

Pre-sync Tracey reports 269 referenced requirements and 696 accepted
requirements. Repository-wide debt keeps `valid: false`. The new requirement is
not missing. It is dangling only because accepted-spec sync has not run yet.

After sync, Tracey reports 270 referenced requirements and 697 accepted
requirements. The new requirement is neither missing nor dangling. Unrelated
repository debt still keeps `valid: false`. The pre-sync and post-sync JSON files
retain both exact reports.

## Live proof

The producer export contains 1,176 graph units and four runtime closure paths.
The first realization reached `realized` with four `remote-substituted` members.
The reuse run recorded four `local-reuse` members and no builder execution.
The receipt-selected fresh pull admitted four members with zero reuse.

The bounded audit returned status 1 by design. It retained one
`unclassified-executable` finding for Guix glibc's `bin/mtrace`. The strongest
state remains `realized`.

Wrong-key, one-member-limit, tampered-receipt, and missing-member checks returned
status 3. They produced no realization receipt and exported no store path.

See `live-guixpkgs-hello/summary.md` for all retained identities and non-claims.
