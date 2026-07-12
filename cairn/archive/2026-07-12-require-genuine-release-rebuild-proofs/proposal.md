## Why

Mantle's production deterministic-release rail currently writes a helper into the release bundle that copies the published stage2 binary into each purported rebuild output. The proof sandbox exposes the release bundle read-only, so two matching runs can establish only that the same published bytes were copied twice, not that recorded source and toolchain inputs rebuilt the artifact. The recorded command identity also names paths and arguments without binding the command or recipe bytes.

A promoting `self-rebuild-match` or deterministic-release verdict must require a real rebuild whose process cannot read the published target bytes and whose recipe, source closure, tools, and policy are content-bound.

## What Changes

- Replace the copy-based real-release helper with a reviewed rebuild recipe that consumes an explicit source and toolchain closure.
- Separate rebuild authority from comparison authority so proof runs cannot read the published target artifact or content-identical aliases from the release/proof bundle.
- Bind canonical recipe bytes, executable/tool identities, arguments, source closure, provider identity, sandbox policy, and normalization policy with BLAKE3 in a versioned deterministic proof contract.
- Make deterministic-release admission fail closed for missing or stale rebuild identities, target-byte access, copy-only workflows, and legacy receipts that cannot prove the stronger contract.
- Add positive genuine-rebuild fixtures and negative copy, alias, recipe-drift, tool-drift, undeclared-input, and prior-output-reuse fixtures.

## Impact

- **Files**: real release determinism orchestration, release reproducibility shell/core boundaries, deterministic receipt schema and validation, receipt checker/summary scripts, fixtures, tests, and operator documentation.
- **Compatibility**: existing proof receipts without content-bound rebuild inputs remain inspectable but cannot promote a deterministic-release claim.
- **Related work**: this complements `enforce-hermetic-release-handoff`; that change owns strict execution-policy selection, while this change owns genuine rebuild authority and identity.
- **Claims**: passing evidence remains a bounded repeated rebuild of named artifacts from recorded inputs, not compiler correctness, full bootstrap correctness, cross-platform reproducibility, or global determinism.
