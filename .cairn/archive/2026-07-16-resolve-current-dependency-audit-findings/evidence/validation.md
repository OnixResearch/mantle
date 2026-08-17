# Dependency audit repair validation

## Oracle checkpoint

- **Question:** Does Mantle resolve the current actionable advisory, source, and license findings without adding a vulnerability waiver, floating a Git dependency, or weakening fail-closed audit defaults?
- **Inspected evidence:** generated lock movement, checked-policy cargo-deny output, locked compile/tests, exact Nickel export pin checks and self-test, dependency-evidence checker self-test, temporary negative source/license policies, first-party quality output, Cairn validation/gates, and Tracey status.
- **Decision:** The bounded dependency-audit repair passes. `crossbeam-epoch` moved to the compatible fixed release, only the already-mandated Git repository and exact SPDX expression were admitted, and positive plus negative checks preserve the stated boundaries.
- **Owner:** Mantle dependency and release-evidence maintainers.
- **Next action:** Preserve the four existing upstream-blocked advisory waivers until their documented unblock conditions occur; independently repair the existing release-provenance Tracey gaps.

## Approach-family registry

| Family | Mechanism | State | Evidence |
|---|---|---|---|
| dependency movement | generate the smallest compatible lock update | validated | `crossbeam-epoch 0.9.18 -> 0.9.20`; no advisory waiver |
| advisory suppression | add `RUSTSEC-2026-0204` to policy ignores | rejected | a compatible fixed version exists, so suppression would preserve avoidable vulnerable code |
| exact source admission | allow only the accepted Nickel export repository while retaining independent revision checks | validated | checked-policy audit plus pin verification/self-test |
| broad source admission | weaken `unknown-git` or permit floating sources | falsified by contract | `unknown-git = "deny"` remains and pin drift self-test passes |
| exact license admission | allow the declared compound SPDX expression | validated | checked-policy audit passes; omission fixture fails on `winx` |
| crate/license bypass | disable or broadly except license checks | rejected | confidence and per-expression policy remain active |

## Implementation result

The generated `Cargo.lock` entry is:

```text
name = "crossbeam-epoch"
version = "0.9.20"
```

No `0.9.18` entry and no `RUSTSEC-2026-0204` ignore remain. `deny.toml` retains:

```text
unknown-registry = "deny"
unknown-git = "deny"
confidence-threshold = 0.8
```

It adds only:

```text
Apache-2.0 WITH LLVM-exception
https://github.com/OnixResearch/nickel-export
```

## Positive checked-policy audit

Final command:

```text
SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo-deny check --config deny.toml
```

Final result, pueue task `70`:

```text
advisories ok, bans ok, licenses ok, sources ok
```

## Locked consumer checks

Pueue task `47` ran:

```text
nix develop -c cargo check -p mantle -p crunch-store -p snix-castore --locked
nix develop -c cargo test -p crunch-attestation-core -p crunch-store --lib --locked
```

Both commands passed; the visible `crunch-store` result was:

```text
test result: ok. 219 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Pueue task `55` ran the ordinary first-party quality gate and completed successfully:

```text
nix develop -c ./scripts/check-first-party-quality.sh
```

Its rustfmt, strict first-party Clippy, and serialized first-party workspace test legs all passed.

## Source and evidence negative checks

Pueue task `48` ran:

```text
nix develop -c cargo -Zscript scripts/check-nickel-export-core-pin.rs --root .
nix develop -c cargo -Zscript scripts/check-nickel-export-core-pin.rs --self-test
nix develop -c cargo -Zscript scripts/check-dependency-audit-evidence.rs --self-test
```

Results:

```text
nickel-export-core pin verified: repository=https://github.com/OnixResearch/nickel-export revision=257fafc1c746f1faf156207043a4c826bfb16d49
nickel-export-core pin self-test passed
dependency audit evidence checker self-test passed
```

Pueue task `50` removed each new policy admission independently in temporary policy files and required cargo-deny to fail on the corresponding class:

```text
negative license policy: rejected as expected
negative source policy: rejected as expected
```

The license-negative output identified `winx 0.36.4` and ended with `licenses FAILED`; the source-negative output identified the unapproved Git source and ended with `sources FAILED`.

## Lifecycle and Tracey status

Pueue task `79` passed `git diff --check`, dependency-evidence validation, Cairn validation, and proposal/design/tasks gates. Cairn validated 28 accepted specs plus this active change with no issues.

Pueue task `81` ran `cairn tracey coverage --root . --json`. The existing `mantle-default` release-provenance profile remains non-green at `140/145` referenced, with no dangling references and these five missing IDs:

```text
mantle.release_provenance.cairn_evidence_handoff.bypass_protection
mantle.release_provenance.cairn_evidence_handoff.cross_repo_dependency
mantle.release_provenance.cairn_evidence_handoff.flake_check_ci
mantle.release_provenance.cairn_evidence_handoff.measured_inputs
mantle.release_provenance.cairn_evidence_handoff.production_wiring
```

That profile scans only `cairn/specs/release-provenance/spec.md` and `tools/release_provenance_tracey_refs.rs`; it does not evaluate this change's verification-evidence requirement. The bounded blocker is retained rather than relabeled as audit-repair success.

## Offline vendor-material boundary

The ignored local `vendor-deps/crossbeam-epoch` snapshot was regenerated from Cargo's locked source and now contains `0.9.20`. The broader offline metadata probe remains blocked before reaching that crate because this checkout's ignored vendor payload lacks pre-existing package `cap-fs-ext`:

```text
error: no matching package named `cap-fs-ext` found
location searched: directory source `vendor-deps`
```

This is not promoted to an offline self-build claim. The checked-in lock, policy, regular locked builds, and audit are validated; complete offline vendor-bundle materialization remains a separate prerequisite owned by the existing self-build workflow.

## Portfolio budget and terminal condition

The review used three mechanism families, one targeted implementation round, one adversarial negative-policy round, and repository-local evidence only. The terminal condition is **validated** for the bounded audit repair. The local VibeThinker response was unusable and was not treated as evidence.

## Non-claims

Passing these checks does not prove dependency correctness, permanent absence of future advisories, legal suitability for every downstream distribution, trust in arbitrary commits from an admitted repository, completeness of ignored offline vendor material, or release eligibility.

## Post-archive Cairn validation

```text
$ nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 0,
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
      "path": "./cairn/specs/bootstrap-inventory/spec.md",
      "requirement_blocks": 3,
      "scenario_blocks": 10,
      "substantive_requirement_blocks": 3
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
      "requirement_blocks": 10,
      "scenario_blocks": 21,
      "substantive_requirement_blocks": 10
    },
    {
      "path": "./cairn/specs/external-batch-dispatchers/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 18,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/foreign-derivation-import/spec.md",
      "requirement_blocks": 22,
      "scenario_blocks": 52,
      "substantive_requirement_blocks": 22
    },
    {
      "path": "./cairn/specs/gcc40-bridge/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 3,
      "substantive_requirement_blocks": 1
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
      "requirement_blocks": 8,
      "scenario_blocks": 15,
      "substantive_requirement_blocks": 8
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
      "requirement_blocks": 8,
      "scenario_blocks": 19,
      "substantive_requirement_blocks": 8
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
  "specs_validated": 28,
  "substance": [],
  "substance_findings": [],
  "substance_issues": [],
  "valid": true
}
```
