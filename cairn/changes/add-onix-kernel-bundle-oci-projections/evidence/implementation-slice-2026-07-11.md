# Kernel-bundle OCI implementation slice — 2026-07-11

## Scope completed

The implementation now provides the non-registry portions of the accepted
change:

- pure `mantle-oci-projection-v1` admission, canonicalization, digest-role,
  archive, OCI graph, import-state, and report logic;
- exact export-only source-admission bundles whose domain-separated BLAKE3
  reductions cover stable admission fields, explicitly omit export-only build
  roots, and must reproduce the sealed path-free projection before CAS reads;
- a thin CAS/filesystem shell with preflight availability checks, exact object
  identity revalidation, deterministic layer construction, staged self-checked
  OCI export, Linux no-replace atomic publication, descriptor-first import,
  exact post-import CAS verification, and report-last admission;
- fail-closed filesystem hardening: no-follow file handles, post-open type and
  byte-bound checks, an exact layout-root allowlist, layer-count rejection
  before descriptor preparation, and no racy non-Linux publication fallback;
- `mantle artifact oci-export` and `mantle artifact oci-import` CLI wiring;
- exact/canonical/gzip, KBI annotation, BLAKE3/SHA-256, traversal, special-file,
  hard-link, no-follow-open, unknown-root-entry, excessive-layer-count,
  collision, duplicate-descriptor, malformed-media, archive-bound, blob-tamper,
  overclaim, interruption, admitted, and compatibility-only tests;
- full/minimal Mantle golden projection/export/import fixtures and exact reviewed
  Onix snapshots from accepted OnixOS commit `8a99461`;
- `docs/kernel-bundle-oci.md`, CLI documentation, digest-role/non-claim tables,
  and Tracey implementation/verification references for the completed
  requirements.

The export report inside the layout carries the complete sealed path-free
projection. Import recomputes that projection BLAKE3, verifies its source
admission reductions and expected external layer SHA-256 values, verifies all
OCI descriptors and rootfs diff IDs, and only then classifies the layout as
`admitted`. A safe external layout without that report remains
`compatibility-only`.

## Focused evidence

All commands ran from the Mantle repository in its Nix development shell.

| Command | Result |
|---|---|
| `cargo test -p mantle --lib oci_projection` | PASS — 19 passed |
| `cargo test -p mantle --bin mantle oci_projection` | PASS — 28 passed |
| `cargo test -p mantle --test kernel_bundle_oci_cli` | PASS — 2 passed |
| golden fixture test with `MANTLE_UPDATE_KERNEL_BUNDLE_OCI_FIXTURES` unset | PASS — checked committed bytes without rewrite mode |
| `cargo check -p mantle --bin mantle` | PASS |
| `cargo clippy -p mantle --lib --no-deps -- -A clippy::result_large_err -A clippy::collapsible_if -D warnings` | PASS; the two allowances are pre-existing `build_correctness.rs` diagnostics |
| `cargo clippy -p mantle --bin mantle --no-deps` plus an `oci_projection` diagnostic scan | PASS at the repository's 440-warning baseline; no OCI-path diagnostics |
| targeted `rustfmt --check` over the four OCI Rust files | PASS |
| root-package `cargo fmt --check -p mantle -v` | BLOCKED outside this slice by pre-existing formatting drift in `tests/trust_policy_offline_rail.rs` |
| `git diff --quiet -- Cargo.toml Cargo.lock` | PASS — no dependency changes |

The initial direct `cargo` baseline could not run because `cargo` is not on the
ambient PATH. The Nix-shell baseline command completed before the change; its
selector matched zero tests. All post-change evidence above uses the supported
Nix shell.

Cairn validation with the canonical policy also passed:

- repository validation: `valid: true`, no issues;
- proposal gate: `PASS`;
- design gate: `PASS`;
- tasks gate: `PASS`.

The commands used
`nix run path:/home/brittonr/git/OnixResearch/cairn#cairn` and
`/home/brittonr/git/OnixResearch/cairn/cairn-policy/generated/cairn-policy.json`
because Mantle's local generated policy remains older than the canonical Cairn
binary.

## Resolved registry dependency

The stable report-registration dependency is now implemented through
`mantle.expand-machine-artifact-contract-registry`; no OCI-specific checker
branch was added. The generic rail owns both `oci.export-report` and
`oci.import-report`, their exact Rust DTO roots, schemas, generated Nickel
contracts, version policy, reviewed golden fixtures, categorized negative
fixtures, BLAKE3 freshness, consumer policy, and non-claims.

Integration caught and repaired concurrent `BuildJsonReport` contract drift,
then passed with 15 contracted surfaces and 44 classified families. Adversarial
review also tightened nested skipped `Option` fields to optional non-null schema
properties and repaired Nickel recursive-record shadowing of the new SHA-256
and Mantle-reference raw predicates.

```text
$ nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs --generate
machine schema contract generation: PASS (15 contracted, 44 classified)

$ nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs --self-test
machine schema contract self-test: PASS

$ nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs
machine schema contract check: PASS (15 contracted, 44 classified)

$ nix develop -c env CARGO_TARGET_DIR=/tmp/mantle-main-registry-target cargo test -q -p mantle --test machine_schema_contracts -- --nocapture
running 4 tests
....
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.59s

$ nix develop -c env CARGO_TARGET_DIR=/tmp/mantle-main-registry-target cargo test -q -p mantle --bin mantle oci_projection::tests::golden_reports_cover_full_minimal_and_both_import_states -- --exact --nocapture
running 1 test
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1360 filtered out; finished in 0.01s

$ nix develop -c env CARGO_TARGET_DIR=/tmp/mantle-main-registry-target cargo check -q -p mantle --bin mantle
(exit 0)
```

## Final current-tree lifecycle review

```text
$ nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root . --policy cairn-policy/generated/cairn-policy.json
{
  "change_issues": [],
  "changes": 14,
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "spec_issues": [],
  "specs_validated": 39,
  "valid": true
}

$ nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate proposal add-onix-kernel-bundle-oci-projections --root . --policy cairn-policy/generated/cairn-policy.json
input_hash=ee435ad3f753a35a5fc007f48e2b5f0e9960fae01da3a388df2a1adcfe80fe4e
receipt_hash=f7ee3f06665ca4c709f0a7a651743d5da40f33659a528e3a005a75bd38f8a05d
issues=[] valid=true verdict=PASS

$ nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate design add-onix-kernel-bundle-oci-projections --root . --policy cairn-policy/generated/cairn-policy.json
input_hash=e9f2fd1ee22b00eaa1d8adfe0e41d21b28e6b109e1923c78d2899ed798a71a45
receipt_hash=cd61ebbd4f0372e789956b238f8a9b674945a04ec87dd839057137c345b70062
issues=[] valid=true verdict=PASS

$ nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate tasks add-onix-kernel-bundle-oci-projections --root . --policy cairn-policy/generated/cairn-policy.json
input_hash=5c477973d2cc472fd3de51fc6f25d7385345008b71c440b6f80d3155163e5054
receipt_hash=8e864e6283f14965e6f07a040786513dcb58fd0169a3221a8da8321d8cafaf21
issues=[] valid=true verdict=PASS
```

All tasks are now checked. Sync and archive remain separate recorded lifecycle
mutations and do not strengthen the bounded implementation claim.

## Claim boundary

This evidence proves the bounded local projection and import/export behavior
covered above. It does not prove validator authenticity, attestation truth,
signature trust, registry publication, bootability, kernel or hardware
compatibility, module/BPF safety, deployability, release eligibility, or
authority to mutate an Onix target.
