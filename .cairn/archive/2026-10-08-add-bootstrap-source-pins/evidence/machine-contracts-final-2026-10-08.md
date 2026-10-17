# Machine-schema contract check on the rebased branch (2026-10-08 UTC)

Guarded Leviathan run `/home/brittonr/mantle-cairn-finish/logs/add-bootstrap-source-pins/machine-contracts-20261008T013801Z` (mcf-guard.sh; full log retained on Leviathan).
Job script: `/home/brittonr/mantle-cairn-finish/bin/jobs/bsp-machine-contracts.sh`.

## guard.log

```text
phase=preflight start=2026-10-08T01:38:01Z free=615525044224 floor=214748364800 peak_reserve=53687091200 dev=66305
cwd=/home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins head=0f80aa9a4e6e412737df9065ddebcec3afdeb519 CARGO_TARGET_DIR=/home/brittonr/mantle-cairn-finish/targets/add-bootstrap-source-pins TMPDIR=/home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins CARGO_BUILD_JOBS=32
command=nix develop /home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins --command /home/brittonr/mantle-cairn-finish/bin/jobs/bsp-machine-contracts.sh
phase=running launcher=2768926 pgid=2768926
phase=child_exited exit=0 elapsed_s=42 free=604065390592
phase=final exit=0 group_survivors=0 end=2026-10-08T01:38:43Z elapsed_s=42 free=604066820096
```

## exit-code

```text
0
```

## output.log (cargo Compiling/Checking progress lines elided)

```text
head=0f80aa9a4e6e412737df9065ddebcec3afdeb519
## finish/add-bootstrap-source-pins
+ cargo -Zscript scripts/check-machine-schema-contracts.rs --self-test
machine schema contract self-test: PASS
machine_contracts_self_test_exit=0 machine_contracts_self_test_wall_s=3
+ cargo -Zscript scripts/check-machine-schema-contracts.rs
machine schema contract check: PASS (24 contracted, 59 classified)
machine_contracts_check_exit=0 machine_contracts_check_wall_s=4
```
