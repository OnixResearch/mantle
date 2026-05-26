# Design: Native registry workspace-dependency topology execution

## Receipt boundary

`native_registry_workspace_dependency_topology_execution` is a dedicated execution receipt for the already-planned workspace-inherited vendored-registry fragment. It is intentionally narrower than the general unified topology receipt.

The receipt records:

- schema version and stable receipt hash.
- execution status and deterministic blocker, when blocked.
- bounded claim/non-claim text.
- workspace root and member package id.
- inherited dependency package ids.
- ordered unit execution receipts for producer units and the member consumer.

## Preconditions

Execution requires:

1. ready native package/target planning evidence containing selected workspace dependency inheritance facts.
2. ready unit derivation graph evidence.
3. explicit manifest paths for inherited vendored-registry dependency packages.
4. supported lib producer derivations and a supported lib/bin workspace member consumer derivation.

Unsupported or stale evidence blocks before `rustc`.

## Execution order

1. Locate the single bounded workspace member with selected workspace dependency facts.
2. For each selected inherited dependency, locate its native package facts and supported lib producer derivation.
3. Execute producers first and capture artifact digests.
4. Bind produced artifacts into the member consumer's dependency artifact surfaces.
5. Execute the member consumer.

## Failure model

The execution path returns deterministic blockers instead of falling back to Cargo, `$CARGO_HOME`, registry caches, target directories, network/index access, or resolver repair.

## Test strategy

CLI coverage must include:

- positive explicit execution of a vendored-registry dependency inherited via `[workspace.dependencies]`.
- negative unsupported inheritance fixture blocking before `rustc`.
- guard coverage that normal `--execute-topology` does not emit the dedicated execution receipt.
