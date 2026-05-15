## Why

Deterministic release eligibility now requires typed sandbox isolation evidence, but `mantle release reproduce --deterministic-proof-runs` only emits the deterministic build proof receipt. Operators need the release workflow to generate the required isolation evidence artifact from the actual proof sandbox profile used for the repeated runs.

## What Changes

- Emit a canonical deterministic sandbox isolation evidence JSON artifact beside `deterministic-build-proof.json` whenever deterministic proof runs are requested.
- Bind the evidence to the supported `mantle-proof-sandbox-v1` profile family and the concrete generated proof sandbox profiles.
- Include the evidence artifact path and digest in release reproduce output.
- Keep deterministic promotion fail-closed: missing or malformed evidence remains invalid.

## Capabilities

### Modified Capabilities
- `release-verification-tech`: deterministic proof generation produces the sandbox isolation evidence required by deterministic release promotion.

## Impact

- **Files**: release reproducibility orchestration, release CLI output, release-core determinism helpers, CLI tests.
- **APIs**: additive public helper(s) for canonical deterministic sandbox isolation evidence bytes/digest if needed by the CLI.
- **Testing**: CLI test proves deterministic reproduction writes the evidence artifact with required checks and digest; core tests cover canonical evidence ordering/digest validation.
