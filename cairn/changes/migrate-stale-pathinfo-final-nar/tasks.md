## Phase 1: Contract and pure core

- [ ] [serial] I1 Record the retained stale signed PathInfo baseline and define exact-path, dry-run, replacement-signature, CA-identity, sidecar-preservation, and non-claim boundaries. r[store_transports.pathinfo_final_nar_migration]
- [ ] [serial] I2 Extract shared CA path-identity validation and implement the bounded pure final-NAR repair planner with positive and negative unit tests. r[store_transports.pathinfo_final_nar_migration]

## Phase 2: Store and CLI shell

- [ ] [serial] I3 Implement exact PathInfo lookup, complete-castore measurement, staged artifact-attestation refresh, replacement signing, persistence, rollback diagnostics, and post-write verification. r[store_transports.pathinfo_final_nar_migration]
- [ ] [serial] I4 Add the dry-run-by-default `mantle store repair-final-nar` human/JSON CLI, mutation locking, side-effect-free dry run, and operator documentation. r[store_transports.pathinfo_final_nar_migration]

## Phase 3: Evidence and lifecycle

- [ ] [serial] V1 Add store-level and CLI positive/negative tests, including current no-op, stale signed repair, unsigned stale rejection, missing/incomplete content, invalid CA, provenance-preserving sidecar refresh, and no mutation on dry run or preflight failure. r[store_transports.pathinfo_final_nar_migration]
- [ ] [serial] V2 Run focused tests, strict Clippy, Tiger Style, first-party quality, machine-contract/blocker checks, Cairn, Tracey, and full Nix validation; exercise the retained Bison state when the local fixture remains available. r[store_transports.pathinfo_final_nar_migration]
- [ ] [serial] V3 Sync and inspect accepted `store-transports` requirements, archive the completed change, append exact post-archive validation, and commit lifecycle evidence. r[store_transports.pathinfo_final_nar_migration]
