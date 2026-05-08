# V1 source-pin validation evidence

Task-ID: V1
Covers: bootstrap.part.autoconf.2.61

- Command: `CARGO_TARGET_DIR=target cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/autoconf-2.61.ncl`
- Exit status: 0
- Transcript: `evidence/V1-source-pin-recheck.log`
