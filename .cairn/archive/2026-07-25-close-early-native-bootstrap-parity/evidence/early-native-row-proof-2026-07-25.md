# Early-native row proof — 2026-07-25

## Scope

This evidence closes only the bounded `binutils.tcc` and `gcc.4.0` parity rows.
It does not claim general compiler or binutils correctness, provider or seed
admission, or whole-bootstrap correctness.

## Accepted artifacts

| Row | Logical output | Canonical artifact-attestation BLAKE3 | NAR SHA-256 |
|---|---|---|---|
| `binutils.tcc` | `/mantle/store/rm39hk4lfnaqzhcpazva7n17f58l5syl-binutils-2.30-tcc-source-v1` | `e0a40c67426126069e36508046c41cc6230afb93e90cc1e07f2255f9d62cb5a1` | `4062d1da2398a0a5103cc214b3cf0a6c8bfaf3418717d516e6c2a0e1773084e4` |
| `gcc.4.0` | `/mantle/store/7rfhqyf3dmv0ziwh8j6jl8jr9labmn9z-gcc-4.0.4-musl-cxx-v7` | `d48ce69b37a151cccaf5ee756362806b517c4427481581063a2ddc7bf8f7d0ac` | `a63f40ae6b88edb4b219b1423027d341685bf296384f4619885101631954b96b` |

The durable output envelopes are
`bootstrap/evidence/early-native-binutils-artifact.json` and
`bootstrap/evidence/early-native-gcc40-artifact.json`. The validator
recanonicalizes each attestation and compares its logical path, canonical
BLAKE3, and NAR digest to the row receipt.

## Independent closure and acceptance observations

The row receipts are:

- `bootstrap/evidence/early-native-binutils-row-v2.json`
- `bootstrap/evidence/early-native-gcc40-row-v2.json`

Validation does not accept predecessor digest strings or receipt booleans by
themselves. It independently:

1. hashes every contracted current Nickel source with BLAKE3;
2. loads every envelope under
   `bootstrap/evidence/early-native-predecessors/`, canonicalizes the artifact
   attestation, and compares role, logical path, and digest;
3. loads the row-specific acceptance evidence, compares the positive and
   rejection matrices, and requires the runtime fingerprint to record
   `trust_unsigned=false`, `signing_key_selected=true`, and substitutions
   disabled;
4. scans the final row source for required matrix/bound markers and forbidden
   ambient-host, TinyCC-delegation, fabricated/non-admission markers; and
5. canonicalizes the final output artifact envelope.

The final GCC build removed inherited ambient `PATH`; its declared PATH now
contains only the bounded BusyBox applet directory and declared derivation
outputs. Task `1617` rebuilt the signed, no-substitution GCC root after that
change. Task `1634` verified the resulting artifact attestation and printed:

```text
OK artifact digest=d48ce69b37a151cccaf5ee756362806b517c4427481581063a2ddc7bf8f7d0ac
```

Both strict runtime fingerprints record `substitution_mode="disabled"`,
`signing_key_selected=true`, and `trust_unsigned=false`. The accepted GCC
recipe log is
`sgssv3i19y3f4gkiziaq9amm3n62jwvb-gcc-4.0.4-musl-cxx-v7.drv.log`; it ends
with `GCC 4.0.4 regenerated C/C++ early-native row behavior matrix passed`.
The accepted binutils recipe log is
`66rwlnpidlrgnqwlpxs4jh3wj7jq1jdw-binutils-2.30-tcc-source-v1.drv.log`; it
records `source-built TCC-era binutils matrix passed`.

The embedded Bison patch intentionally preserves upstream tabs and blank
context lines. `.gitattributes` scopes Git whitespace handling for that file.
A diagnostic rewrite attempt (task `1681`) failed and was not used as evidence;
the exact previously receipted source bytes were restored. Task `1688` then
accepted the current Bison root from the signed no-substitution state at
`/mantle/store/4ag6mwv2pmsa0b8wii2zwc4jws5iajbk-bison-2.3-musl`.

## Verification completed

- Task `1647`: `nix develop -c cargo test -p mantle --bin mantle bootstrap_parity -- --nocapture`
  — `88 passed; 0 failed`.
- Task `1656`: `nix develop -c cargo test -p mantle --test bootstrap_parity_cli -- --nocapture`
  — `16 passed; 0 failed`.
- The CLI suite includes positive production-row coverage and negative cases
  for malformed receipts, stale source digests, stale generated-artifact
  status, cross-row substitution, corrupt predecessor attestations, missing
  output evidence, forbidden host discovery, wrapper delegation, and an
  untrusted runtime fingerprint.
- Task `1694`: `nix develop -c ./scripts/check-gcc40-configure-bridge.rs --self-test`
  — `PASS (classes=3, source_spellings=2, source_bytes_max=65536, invocation_count_max=4096)`.
- Task `1697`: direct nightly Cargo-script execution of
  `scripts/check-bootstrap-source-pins.rs` exited successfully.
- Task `1687`: `git diff --check` exited successfully.
- Task `1720`: `nix develop -c cargo test -p mantle --test bootstrap_eval -- --nocapture`
  completed all 30 tests successfully before the chained formatting leg reported
  one stale formatting diff; task `1729` applied formatting and the subsequent
  `cargo fmt --check -p mantle -v` leg passed.
- Task `1745`: focused first-party
  `cargo clippy -p mantle --bin mantle --test bootstrap_parity_cli --no-deps -- -D warnings`
  passed. The visible `snix-castore` dead-code warning is dependency output and
  was outside the `--no-deps` first-party lint surface.
- Task `1742`: canonical Cairn validation recorded `valid: true`; proposal,
  design, and tasks gates each recorded `valid: true` and `verdict: "PASS"`.

## Remaining non-claims and downstream blockers

The configure preprocessing bridge remains confined to the audited
`conftest.c` authority and is not a provider preprocessor. Intermediate
same-version bootstrap artifacts remain explicit predecessors rather than
being retroactively promoted. GCC 4.7, GCC 10, full-musl binutils, full-source
Rust qualification, StageX lineage, deterministic fixed-point proof, and
whole-bootstrap promotion remain separate downstream work.

## Post-archive validation transcript

```text
waiting for another Nix process to finish fetching input 'path:/home/brittonr/git/OnixResearch/cairn'...
error (ignored): SQLite database '/home/brittonr/.cache/nix/eval-cache-v6/3d39bbe44046dc3b8d1be188d3645bcc20981381d9f3b0348077f77d0e17b96e.sqlite' is busy
this derivation will be built:
  /nix/store/k9vg9dhim6y901w9piwlyabdqscf0h77-cairn-0.1.0.drv
{
  "change_issues": [],
  "changes": 5,
  "cross_repo_dependencies": [],
  "cross_repo_evidence_issues": [],
  "findings": [],
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "spec_findings": [],
  "spec_issues": [],
  "spec_substance": [
    {
      "path": "./cairn/changes/bind-full-source-rust-provider/specs/bootstrap-inventory/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 4,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "./cairn/changes/bind-full-source-rust-provider/specs/rust-package-planning/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 3,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "./cairn/changes/close-final-native-toolchain-parity/specs/bootstrap-inventory/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 4,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "./cairn/changes/materialize-stagex-lineage-provider/specs/bootstrap-inventory/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 5,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "./cairn/changes/promote-full-bootstrap-parity/specs/bootstrap-inventory/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 5,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "./cairn/changes/prove-source-built-mantle-fixed-point/specs/bootstrap-inventory/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 5,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "./cairn/specs/artifact-auth-adoption/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 13,
      "substantive_requirement_blocks": 8
    },
    {
      "path": "./cairn/specs/artifact-auth-operational-receipt/spec.md",
      "requirement_blocks": 4,
      "scenario_blocks": 7,
      "substantive_requirement_blocks": 4
    },
    {
      "path": "./cairn/specs/artifact-auth-shell-verification/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 5,
      "substantive_requirement_blocks": 5
    },
    {
      "path": "./cairn/specs/bootstrap-inventory/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 40,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/build-correctness/spec.md",
      "requirement_blocks": 25,
      "scenario_blocks": 59,
      "substantive_requirement_blocks": 25
    },
    {
      "path": "./cairn/specs/build-scheduling/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 16,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "./cairn/specs/build-tool-boundary/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 23,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/cache-substitution/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 16,
      "substantive_requirement_blocks": 5
    },
    {
      "path": "./cairn/specs/examples/spec.md",
      "requirement_blocks": 11,
      "scenario_blocks": 24,
      "substantive_requirement_blocks": 11
    },
    {
      "path": "./cairn/specs/external-batch-dispatchers/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 18,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/flake-source-inventory/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 1,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "./cairn/specs/foreign-derivation-import/spec.md",
      "requirement_blocks": 22,
      "scenario_blocks": 52,
      "substantive_requirement_blocks": 22
    },
    {
      "path": "./cairn/specs/gcc40-bridge/spec.md",
      "requirement_blocks": 2,
      "scenario_blocks": 6,
      "substantive_requirement_blocks": 2
    },
    {
      "path": "./cairn/specs/hardware-simulation-builds/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 17,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/i386-tinycc27/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 4,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "./cairn/specs/kani-toolchain-evidence/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 11,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "./cairn/specs/kernel-bundle-oci/spec.md",
      "requirement_blocks": 14,
      "scenario_blocks": 30,
      "substantive_requirement_blocks": 14
    },
    {
      "path": "./cairn/specs/kernelscript-experiment/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 14,
      "substantive_requirement_blocks": 8
    },
    {
      "path": "./cairn/specs/machine-artifact-contracts/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 9,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/nickel-export-infrastructure/spec.md",
      "requirement_blocks": 6,
      "scenario_blocks": 12,
      "substantive_requirement_blocks": 6
    },
    {
      "path": "./cairn/specs/operator-diagnostics/spec.md",
      "requirement_blocks": 13,
      "scenario_blocks": 32,
      "substantive_requirement_blocks": 13
    },
    {
      "path": "./cairn/specs/portable-build-receipts/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 14,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "./cairn/specs/project-workflows/spec.md",
      "requirement_blocks": 24,
      "scenario_blocks": 80,
      "substantive_requirement_blocks": 24
    },
    {
      "path": "./cairn/specs/realization-routing/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 21,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/release-provenance/spec.md",
      "requirement_blocks": 66,
      "scenario_blocks": 93,
      "substantive_requirement_blocks": 66
    },
    {
      "path": "./cairn/specs/remote-builds/spec.md",
      "requirement_blocks": 35,
      "scenario_blocks": 109,
      "substantive_requirement_blocks": 35
    },
    {
      "path": "./cairn/specs/rust-package-planning/spec.md",
      "requirement_blocks": 129,
      "scenario_blocks": 447,
      "substantive_requirement_blocks": 129
    },
    {
      "path": "./cairn/specs/source-transports/spec.md",
      "requirement_blocks": 12,
      "scenario_blocks": 25,
      "substantive_requirement_blocks": 12
    },
    {
      "path": "./cairn/specs/spacewasm-reference-materialization/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 16,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/store-transports/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 26,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/verification-evidence/spec.md",
      "requirement_blocks": 58,
      "scenario_blocks": 164,
      "substantive_requirement_blocks": 58
    },
    {
      "path": "./cairn/specs/wasm-component-builds/spec.md",
      "requirement_blocks": 14,
      "scenario_blocks": 25,
      "substantive_requirement_blocks": 14
    }
  ],
  "specs_validated": 38,
  "substance": [
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/bind-full-source-rust-provider/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 12,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/bind-full-source-rust-provider/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 10,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/bind-full-source-rust-provider/specs/bootstrap-inventory/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 4,
      "substantive_lines": 17,
      "substantive_requirement_blocks": 1,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/bind-full-source-rust-provider/specs/rust-package-planning/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 3,
      "substantive_lines": 13,
      "substantive_requirement_blocks": 1,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 7,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/bind-full-source-rust-provider/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 7,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 7,
      "task_done": 2,
      "task_in_progress": 0,
      "task_todo": 5
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/close-final-native-toolchain-parity/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 12,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/close-final-native-toolchain-parity/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 9,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/close-final-native-toolchain-parity/specs/bootstrap-inventory/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 4,
      "substantive_lines": 17,
      "substantive_requirement_blocks": 1,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 8,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/close-final-native-toolchain-parity/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 8,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 8,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 8
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/materialize-stagex-lineage-provider/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 12,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/materialize-stagex-lineage-provider/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 10,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/materialize-stagex-lineage-provider/specs/bootstrap-inventory/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 5,
      "substantive_lines": 21,
      "substantive_requirement_blocks": 1,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 8,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/materialize-stagex-lineage-provider/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 8,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 8,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 8
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/promote-full-bootstrap-parity/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 14,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/promote-full-bootstrap-parity/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 12,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/promote-full-bootstrap-parity/specs/bootstrap-inventory/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 5,
      "substantive_lines": 21,
      "substantive_requirement_blocks": 1,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 9,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/promote-full-bootstrap-parity/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 9,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 9,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 9
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/prove-source-built-mantle-fixed-point/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 14,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/prove-source-built-mantle-fixed-point/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 10,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/prove-source-built-mantle-fixed-point/specs/bootstrap-inventory/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 5,
      "substantive_lines": 21,
      "substantive_requirement_blocks": 1,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 8,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/prove-source-built-mantle-fixed-point/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 8,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 8,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 8
    }
  ],
  "substance_findings": [],
  "substance_issues": [],
  "valid": true
}

```
