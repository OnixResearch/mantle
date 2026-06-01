# Tasks

## Catalog and inventory

- [ ] [serial] Add `examples/catalog.ncl` with typed entries for every checked-in user-facing example, including support tier, capabilities, network behavior, and validation rail. r[examples.support_catalog]
- [ ] [serial] Add a pure inventory checker/test that reads the catalog and checked-in example paths, then rejects missing paths, duplicate ids, duplicate paths, unsupported tiers, and silent skips. r[examples.support_catalog]
- [ ] [serial] Classify generated seed-dependent, heavyweight, benchmark, and real-network examples explicitly so fast validation knows why each is included or skipped. r[examples.support_catalog]

## Documentation drift

- [ ] [serial] Update `examples/README.md` and the root README examples section from the catalog inventory, preserving exact commands and capability notes. r[examples.documentation_drift]
- [ ] [serial] Add positive and negative README drift tests for missing catalog entries, stale links, omitted supported examples, invalid command snippets, and stale Crunch branding outside exact compatibility identifiers. r[examples.documentation_drift]

## Verification

- [ ] [serial] Run the examples inventory tests and record the command output as durable evidence before marking this change complete. r[examples.support_catalog] r[examples.documentation_drift]
- [ ] [serial] Run `cairn validate --root .` and the tasks gate, then archive only after completed tasks cite durable evidence. r[examples.support_catalog] r[examples.documentation_drift]
