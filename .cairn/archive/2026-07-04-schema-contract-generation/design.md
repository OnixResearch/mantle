# Design: Schema contract generation rail

## Architecture

The rail has three deterministic stages.

1. Schema inventory: identify stable JSON surfaces and the checked-in JSON Schema or schema-like Rust owner for each.
2. Contract generation/check: generate Nickel contracts from JSON Schema into checked-in files or compare against checked-in snapshots. Conversion is a dev/release check, not a runtime dependency.
3. Fixture validation: run valid and invalid example payloads through the generated contract and record whether the failure is a precise record-contract rejection or an opaque predicate rejection.

## Contract handling

Generated contracts should prefer lazy, inspectable record contracts when the schema permits it. When conversion must use a strict predicate contract, the rail should label the contract as opaque for LSP/doc purposes while still validating pass/fail behavior. Unsupported schema dialects or keywords become explicit blockers.

## Evidence and non-claims

A passing schema-contract rail proves only that the selected fixtures match or fail the selected schema contract. It does not prove the command that emits the JSON is correct unless a separate test runs that command and validates its output against the contract.

## Validation strategy

Positive tests should validate representative payloads for each selected surface. Negative tests should cover missing required fields, type mismatches, malformed enum values, unknown schema versions where rejected, oversized bounded fields, and stale generated-contract snapshots. Snapshot tests should make generated contract drift reviewable.
