# Machine-schema contract inventory: bootstrap.source-pin-plans family and regenerated freshness (2026-10-08 UTC)

Guarded Leviathan run `/home/brittonr/mantle-cairn-finish/logs/add-bootstrap-source-pins/machine-contracts-generate-20261008T011204Z` (mcf-guard.sh; full log retained on Leviathan).
Job script: `/home/brittonr/mantle-cairn-finish/bin/jobs/bsp-machine-contracts-generate.sh`.

## guard.log

```text
phase=preflight start=2026-10-08T01:12:04Z free=880826363904 floor=214748364800 peak_reserve=429496729600 dev=66305
cwd=/home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins head=92eea6adf6010635b0e84341b3e3e41ecfa3a33a CARGO_TARGET_DIR=/home/brittonr/mantle-cairn-finish/targets/add-bootstrap-source-pins TMPDIR=/home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins CARGO_BUILD_JOBS=32
command=nix develop /home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins --command /home/brittonr/mantle-cairn-finish/bin/jobs/bsp-machine-contracts-generate.sh
phase=running launcher=1919145 pgid=1919145
phase=child_exited exit=0 elapsed_s=58 free=881800310784
phase=final exit=0 group_survivors=0 end=2026-10-08T01:13:02Z elapsed_s=58 free=881788055552
```

## exit-code

```text
0
```

## output.log (cargo Compiling/Checking progress lines elided)

```text
warning: Git tree '/home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins' is dirty
head=92eea6adf6010635b0e84341b3e3e41ecfa3a33a
+ cargo -Zscript scripts/check-machine-schema-contracts.rs --generate
machine schema contract generation: PASS (24 contracted, 59 classified)
machine_contracts_generate_exit=0 machine_contracts_generate_wall_s=4
 M schemas/machine-contracts/inventory.ncl
 M src/bootstrap_pin_cmd.rs
 schemas/machine-contracts/inventory.ncl | 62 +++++++++++++++++++++++++++++++--
 src/bootstrap_pin_cmd.rs                |  1 +
 2 files changed, 61 insertions(+), 2 deletions(-)
+ cargo -Zscript scripts/check-machine-schema-contracts.rs
machine schema contract check: PASS (24 contracted, 59 classified)
machine_contracts_check_exit=0 machine_contracts_check_wall_s=3
```
