# V1 source pin recheck evidence

Task-ID: V1
Covers: bootstrap.part.seed.full

Command:

```sh
cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/seed-full.ncl > openspec/changes/live-part-seed-full/evidence/V1-source-pin-recheck.log 2>&1
```

Result: pass, exit code 0. `seed-full.ncl` has no direct source fetches; source pins are inherited from declared predecessor derivations.
