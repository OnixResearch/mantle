# I9-I11: Audit command, receipt, and operator boundary

`mantle foreign-import audit` requires an executable plan, complete realization
receipt, typed audit policy, explicit selected roots, existing store state, and
an output path. It rejects an existing output before opening the store.

Admission verifies the plan and realization receipt self-identities. It derives
the selected unit closure, exact expected NAR facts, declared references, and
foreign-to-target path map before scanning.

The command emits `mantle-foreign-provenance-audit-v1`. Its BLAKE3 identity
binds these facts:

- realization receipt and build-report identities;
- plan, path-map, profile-set, and policy identities;
- selected roots and closure paths;
- signed PathInfo and castore observations;
- existing build-correctness reference reports;
- payload classes, limits, findings, and disposition; and
- explicit non-claims.

A passing audit reports `provenance-audited`. A failed audit reports `realized`
as the strongest state. Neither result changes the realization receipt or build
report.

The Nickel policy source is in `config/foreign-provenance-audit/`. The generated
JSON is checked for freshness. Positive and negative Nickel fixtures check exact
fields and nonzero limits.

The trust model, operator guide, README, machine-artifact guide, and compatibility
inventory document the new state and its limits.
