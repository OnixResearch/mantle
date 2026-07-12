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

## Deliberate blocker

The stable export/import report registration task remains unchecked behind
`mantle.expand-machine-artifact-contract-registry`. That active change still
owns the data-driven inventory, exact schema authority, generated Nickel
contracts, fixture classification, BLAKE3 freshness, version policy, and
consumer-policy rail. This change does not duplicate or preempt that work.

Consequently this package is intentionally **not** synced into accepted specs
or archived, and downstream ChaosControl implementation remains blocked on the
accepted Mantle package. Once the registry dependency lands, register both
report families, run its positive/negative schema-contract rail, rerun the
focused and Cairn checks, then sync/archive this change.

## Claim boundary

This evidence proves the bounded local projection and import/export behavior
covered above. It does not prove validator authenticity, attestation truth,
signature trust, registry publication, bootability, kernel or hardware
compatibility, module/BPF safety, deployability, release eligibility, or
authority to mutate an Onix target.
