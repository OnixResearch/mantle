# Tasks: Raise foreign-import graph bounds to the producer contract

## Phase 1: Admission bounds

- [x] [serial] T1.1 Raise `MAX_GRAPH_NODES` to 32,768 and tie it to the bridge producer contract with a dev-dependency assertion. r[foreign_derivation_import.producer_graph_bounds]
- [x] [serial] T1.2 Raise `MAX_SOURCE_PAYLOADS` to 16,384 and fail lowering explicitly with `source-payload-limit-exceeded`. r[foreign_derivation_import.producer_graph_bounds]
- [x] [serial] T1.3 Derive `MAX_HASH_DOMAIN_RECORDS` from the node bound and `MAX_FOREIGN_READ_CALLS` from the source-payload bound plus fixed reads. r[foreign_derivation_import.producer_graph_bounds]
- [x] [serial] T1.4 Replace per-node combined-map rebuilds with one incremental exact-path index in graph compilation. r[foreign_derivation_import.producer_graph_bounds]
- [x] [serial] T1.5 Raise `MAX_GRAPH_EDGES` to 4,096 and foreign-import `MAX_FIELD_BYTES` to 256 KiB from the frozen export's measured surface. r[foreign_derivation_import.producer_graph_bounds]
- [x] [serial] T1.6 Raise `crunch-build::goal::MAX_GOALS` to 32,768 and derive `registry::MAX_ENTRIES` from it. r[foreign_derivation_import.producer_graph_bounds]

## Phase 2: Verification

- [x] [serial] T2.1 Add positive and negative bound tests (4,096/32,768/32,769 derivations; 16,384/16,385 source payloads). r[foreign_derivation_import.producer_graph_bounds]
- [x] [serial] T2.2 Run the focused `mantle` build/test/clippy/fmt and no-std gates and record the transcript. r[foreign_derivation_import.producer_graph_bounds]
- [x] [serial] T2.3 Record ADR 0166 and validate the Cairn change (proposal/design/tasks gates). r[foreign_derivation_import.producer_graph_bounds]
- [x] [serial] T2.4 Run the real frozen-export `produce-nix` and `plan` check and record the verdict. r[foreign_derivation_import.producer_graph_bounds]
- [x] [serial] T3.1 Fix v4 fixed-output metadata: SRI algorithm, `method` recursion, and explicit conflicts. r[foreign_derivation_import.producer_graph_bounds]
- [x] [serial] T3.2 Re-run the real frozen-export plan and prepare-sources check; record the verdict. r[foreign_derivation_import.producer_graph_bounds]
