## Why

Mantle's deterministic scheduler already accepts ordinal `resource-fit`, `content-locality`, and `transfer-cost` classes, and remote workers advertise basic capabilities and concurrency. Those classes do not yet derive from concrete bounded requirements and inventories for CPU, memory, scratch space, accelerators, or named scarce token pools such as licensed tools. The coordinator therefore cannot prove atomic reservation, prevent overcommit across concurrent assignments, or explain how verified worker content produced a locality class.

Large or expensive builds need resource eligibility and data placement to be explicit facts rather than hand-assigned preference labels. This extends the existing scheduler without replacing its lazy graph or deterministic ordering.

## What Changes

- Add provider-neutral quantified resource requirement and worker-capacity records for CPU, memory bytes, scratch bytes, accelerator classes/counts, and bounded named token pools.
- Separate semantic platform/tool capabilities that affect action identity from scheduling-only quantities and dynamic inventory that must not perturb action identity.
- Add a pure reservation planner and durable fenced worker leases so concurrent assignments cannot overcommit capacities or scarce tokens.
- Derive hard resource eligibility and normalized resource-fit classes from requirement/capacity/reservation facts rather than provider response order.
- Derive content-locality and transfer-cost classes from receiver-verified object presence, demanded bytes, and transfer policy rather than unverified worker claims.
- Prefer compatible workers that minimize declared transfer while retaining deterministic starvation protection and every existing trust, privacy, network, prefix, and capability gate.
- Extend typed Nickel remote-builder policy and bounded scheduling/build evidence.

## Impact

- **Surfaces**: scheduler and distributed cores, coordinator worker registration and assignment, remote transfer probes, `lib/remote-builders.ncl`, `lib/scheduling.ncl`, status/build reports, and recovery state.
- **Dependencies**: consumes accepted deterministic scheduler ordering, fenced attempts, production resumable transfer, and coordinator runtime; does not replace them.
- **Non-claims**: no globally optimal placement, no provider autoscaler, no Slurm/Kubernetes API, no license-server implementation, no trust from resource authorization, and no use of locality to bypass hard eligibility.
- **Validation**: arithmetic/property tests, concurrent lease and restart fixtures, stale-fence negatives, verified-locality fixtures, Nickel parity tests, and Cairn gates.
