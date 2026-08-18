# Design: Native registry patch-source topology execution

## Receipt boundary

`native_registry_patch_source_topology_execution` is a dedicated execution receipt for the already-planned local `[patch.crates-io]` fragment. It is intentionally narrower than the general unified topology receipt.

The receipt records:

- schema version and stable receipt hash.
- execution status and deterministic blocker, when blocked.
- bounded claim/non-claim text.
- consumer package id.
- patch source package ids.
- ordered unit execution receipts for producer and consumer.

## Preconditions

Execution requires:

1. ready native registry source planning containing exactly one `patch-path` source.
2. ready native package/target planning evidence.
3. ready unit derivation graph evidence.
4. native package facts for the patch source and exactly one supported consumer.
5. supported lib producer derivation and supported lib/bin consumer derivation.

Unsupported or stale evidence blocks before `rustc`.

## Execution order

1. Locate the single local patch source from native registry source facts.
2. Locate its native package facts and the single consumer that depends on the patch manifest.
3. Execute the patch source producer first and capture artifact digests.
4. Bind the produced artifact into the consumer dependency artifact surfaces.
5. Execute the consumer.

## Failure model

The execution path returns deterministic blockers instead of falling back to Cargo, `$CARGO_HOME`, registry caches, target directories, network/index access, or resolver repair.

## Test strategy

CLI coverage must include:

- positive explicit execution of a local `[patch.crates-io]` source replacement.
- negative unsupported patch fixture blocking before `rustc`.
- guard coverage that normal `--execute-topology` does not emit the dedicated execution receipt.
