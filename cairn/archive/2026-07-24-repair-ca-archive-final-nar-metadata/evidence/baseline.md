# Baseline: CA archive final NAR mismatch

## Exact reproduction

Retained state:

- state: `/home/brittonr/.cache/mantle-full-source-20260718/gcc40-state`
- physical store: `/home/brittonr/.cache/mantle-full-source-20260718/gcc40-store`
- logical path: `/mantle/store/57dqxg2kjddkvjmwkvkr54nifqfs38l0-bison-2.3-gcc-v6`

The signed PathInfo and artifact attestation record:

```text
nar_size=972064
nar_sha256=fff5e607c24805da4403a7619a5a967b61d0ea3fca268103773a47bde813f3eb
ca=Nar(Sha256(fff5e607c24805da4403a7619a5a967b61d0ea3fca268103773a47bde813f3eb))
node=Directory(blake3-vZIydtZX2lLdWghEdr4jGYjBV7XmIZFDI/CyLIM4NuU=)
```

Exporting that path and importing into fresh state with the explicit unsigned trust escape hatch reproduced the pending blocker exactly:

```text
archive import: store: ingesting archive payload for 57dqxg2kjddkvjmwkvkr54nifqfs38l0-bison-2.3-gcc-v6: Hash mismatch, expected: sha256-//XmB8JIBdpEA6dhmlqWe2HQ6j/KJoEDdzpHvegT8+s=, got: sha256-xLxyS25fkgR8xX5tnaCacetsRhEPrAQlTrqHBi8cZzg=.
```

The actual rendered final NAR SHA-256 is `c4bc724b6e5f92047cc57e6d9da09a71eb6c46110fac04254eba87062f1c6738`.

## Source diagnosis

`crates/crunch-build/src/orchestrate.rs::compute_ca_intermediates` hashes marker-normalized nodes and derives CA paths. `finalize_ca_outputs` then rewrites markers to final paths but passes `intermediate.nar_size` and `intermediate.nar_sha256` to `persist_and_export_output` with the rewritten final node. The same intermediate hash becomes `PathInfo.ca`.

`crates/crunch-store/src/archive.rs::import_missing_path` passes that CA field to `ingest_nar_and_hash`, which compares the final NAR bytes to the marker-normalized hash before the archive's own final NAR and node checks can run.

## Baseline tests

Before modifying builder or archive core logic, task `249` ran:

```text
nix develop -c cargo test -p crunch-store archive_
nix develop -c cargo test -p crunch-build ca_
```

The command completed successfully; the visible `crunch-build` summary reported `20 passed; 0 failed` for the selected CA tests. Existing archive tests cover ordinary non-CA round trips but do not construct a marker-normalized CA field distinct from final NAR identity.

## Non-claim

This baseline identifies stale final-NAR metadata and identity conflation in one retained Mantle CA output. It does not prove compiler correctness, archive compatibility with external nario formats, or that every historical PathInfo is affected. Existing signed stale metadata cannot be silently repaired without invalidating its signature.
