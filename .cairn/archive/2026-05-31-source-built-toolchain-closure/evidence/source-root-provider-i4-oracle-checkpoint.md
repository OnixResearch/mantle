# I4 source-root provider completion oracle checkpoint

Task-ID: I4-oracle
Covers: rust_package_planning.source_built_toolchain_closure
Date: 2026-05-31
Owner: agent; completion decision requires maintainer acceptance of a real materialization transcript

## Question

Does the current source-root provider integration evidence satisfy I4: “Materialize or integrate a real normalized source-root/seed provider; do not mark this task complete with placeholder providers or host-tool stubs”?

## Inspected evidence

- `cairn/changes/source-built-toolchain-closure/tasks.md` had I4 marked complete.
- `cairn/changes/source-built-toolchain-closure/evidence/source-root-provider-integration-validation.md` records module wiring, seed NCL generation, emitted-role validation, and fail-closed patch handling.
- The strongest CLI-level materializer coverage is negative: `bootstrap_source_root_provider_reaches_materializer_and_fails_closed_on_patches`.
- The evidence file itself says the next action is to add or select real source-root manifest inputs and run `mantle bootstrap --source-root <manifest>` with a writable store.
- No transcript currently records a successful provider materialization from real source inputs into a writable store.

## Decision

No. The current evidence proves integration progress, not I4 completion. I4 must remain unchecked until a positive real source-root manifest/provider materialization run is recorded. Negative-only materializer reachability and seed-generation unit tests are not enough for the “real normalized source-root/seed provider” completion claim.

## Next action

Keep the integration commit as progress. Select or create the real source-root manifest inputs, run `mantle bootstrap --source-root <manifest> --store <writable-store> -o <seed.ncl>`, record the provider output path, manifest digest, output digest, emitted role trace, and generated seed contract, then reconsider checking I4.

## Validation after repair

Command:

```text
/home/brittonr/.cargo-target/debug/cairn validate --root .
/home/brittonr/.cargo-target/debug/cairn gate tasks source-built-toolchain-closure --root .
```

Output:

```json
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 4,
  "valid": true
}
{
  "change": "source-built-toolchain-closure",
  "input_hash": "c564281755873a7666b2a5b8fb5de6464df4dbee01509465dfb269c761cf9a42",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "2d3b51da9580183754e7622e17b4fb91868e7bb93f670234a0b07dacaf3d0a01",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```
