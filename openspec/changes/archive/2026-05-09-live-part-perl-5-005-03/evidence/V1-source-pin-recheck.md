# V1 source pin recheck evidence

Task-ID: V1
Covers: bootstrap.part.perl.5.005.03

Command:

```sh
cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/perl-5.005_03-musl.ncl > openspec/changes/live-part-perl-5-005-03/evidence/V1-source-pin-recheck.log 2>&1
```

Result: pass, exit code 0.
