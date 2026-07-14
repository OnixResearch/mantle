# Pre-change Nickel export baseline

Captured before changing `src/nickel_export.rs` core logic on branch
`drain/adopt-standalone-nickel-export-core` at
`fa739618e88e19522484fd1f3831f1a3b5046dab`.

## Existing focused tests

Command:

```text
nix develop -c cargo test -p mantle --bin mantle nickel_export -- --nocapture
```

Result:

```text
running 4 tests
test nickel_export::tests::normalize_relative_path_removes_current_dir_without_escaping ... ok
test nickel_export::tests::export_receipt_identity_changes_with_evaluator_descriptor ... ok
test nickel_export::tests::export_request_rejects_absolute_escape_and_unsupported_format ... ok
test machine_contract_producer_tests::nickel_export_reports_serialize_to_registered_positive_fixtures ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 1485 filtered out
```

## Existing CLI fixture

Fixture files are under `evidence/baseline-fixture/`.

Commands and exit statuses:

```text
mantle --json export main.ncl --dep dependency.ncl    # 0
mantle --json export ../outside.ncl                   # 3
mantle --json export invalid.ncl                      # 3
```

The positive legacy receipt recorded:

```text
receipt_digest_blake3=761c8bd5ab8ba77252abba7e51fbb89325328804b9ef9e1a6f6122cb10dd34cc
output_digest_blake3=81f6fd0ef2a83132dc404a233a4db54ee6590314442017bffeed9ba24b760686
source_digest_blake3=91a946e72efa7d1a01984363dd999711933bda6d94942f32c31a3fd17b61a2f5
dependency_digest_blake3=d53962da07af23ef1f42e0a7a947e5390bfc8a2572bff17c4c3058c3510a0f4a
schema=mantle-nickel-export-receipt-v1
evaluator.identity=mantle-embedded-crunch-eval
evaluator.version=0.1.0
```

The path-escape case returned `failure_class=validation`, diagnostic class
`unsafe-source-path`, and no receipt. The invalid Nickel case returned
`failure_class=eval`, diagnostic class `eval`, and no receipt.

The registered positive machine fixtures before the cutover were:

- `schemas/machine-contracts/fixtures/nickel-export-receipt.valid.json`
- `schemas/machine-contracts/fixtures/nickel-export-report.valid.json`

The registered adversarial fixture sets were:

- `schemas/machine-contracts/fixtures/nickel-export-receipt.negatives.json`
- `schemas/machine-contracts/fixtures/nickel-export-report.negatives.json`

## Legacy authority boundary

Before the cutover, `src/nickel_export.rs` owned all of these concerns:

- lexical request/path normalization;
- filesystem reads of the root source and declared dependencies;
- direct `crunch_eval::evaluate_to_json` invocation and evaluator diagnostics;
- BLAKE3 source, dependency, output, and receipt hashing;
- `mantle-nickel-export-receipt-v1` construction;
- destination directory creation and output writes;
- human/JSON rendering and exit status.

The standalone core may replace only the evaluator-neutral normalization,
admission, exact-byte identity, canonical receipt, and Mantle v1 projection.
Mantle must retain `crunch-eval`, filesystem/root and symlink admission,
destination writes, rendering, build evidence, and release policy.

## Pre-change lifecycle blocker

The proposal/design/tasks gate attempt through the canonical Cairn Nix package
stopped before any gate ran because the host Nix configuration tried to open
`/run/secrets/vars/nix-signing-key/key`, which was absent. This is an
environmental baseline blocker, not a passing gate. Final validation must retry
with the secret-key setting neutralized or use another reviewed local Cairn
binary without weakening gate policy.
