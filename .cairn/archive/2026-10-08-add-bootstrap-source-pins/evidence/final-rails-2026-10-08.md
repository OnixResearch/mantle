# Final pre-archive rails (2026-10-08 UTC)

Head `f74f81c30406a821672196db0c90886fe2f88619` on published main `30a5d2a85f7c7bb6683f69eb4fe449971bdb29b2`. Job `bin/jobs/bsp-final.sh` under `mcf-guard.sh --devshell`; log `/home/brittonr/mantle-cairn-finish/logs/add-bootstrap-source-pins/final-20261008T052743Z/output.log`.

```text
head=f74f81c30406a821672196db0c90886fe2f88619
+ cargo test -p crunch-project-core --lib bootstrap_pins
core_pins_exit=0 core_pins_wall_s=3
+ cargo test -p mantle --bin mantle bootstrap_pin_cmd::tests::
cli_pins_exit=0 cli_pins_wall_s=37
+ ./scripts/check-operator-command-contract.sh
operator_contract_exit=0 operator_contract_wall_s=96
+ cargo -Zscript scripts/check-machine-schema-contracts.rs
machine_contracts_exit=0 machine_contracts_wall_s=3
+ git diff --check main
diff_check_exit=0 diff_check_wall_s=0
+ nix build --no-link --print-out-paths -L .#checks.x86_64-linux.bootstrap-blocker-inventory
nix_bootstrap_blocker_inventory_exit=0 nix_bootstrap_blocker_inventory_wall_s=13
+ /home/brittonr/mantle-cairn-finish/cairn-result/bin/cairn validate --root .
cairn_validate_exit=0 cairn_validate_wall_s=0
+ /home/brittonr/mantle-cairn-finish/cairn-result/bin/cairn gate proposal add-bootstrap-source-pins --root .
cairn_gate_proposal_exit=0 cairn_gate_proposal_wall_s=0
+ /home/brittonr/mantle-cairn-finish/cairn-result/bin/cairn gate design add-bootstrap-source-pins --root .
cairn_gate_design_exit=0 cairn_gate_design_wall_s=0
+ /home/brittonr/mantle-cairn-finish/cairn-result/bin/cairn gate tasks add-bootstrap-source-pins --root .
cairn_gate_tasks_exit=0 cairn_gate_tasks_wall_s=0
+ /home/brittonr/mantle-cairn-finish/cairn-result/bin/cairn sync add-bootstrap-source-pins --root .
cairn_sync_dry_exit=0 cairn_sync_dry_wall_s=0
+ env CAIRN_ARCHIVE_DATE=2026-10-08 /home/brittonr/mantle-cairn-finish/cairn-result/bin/cairn archive add-bootstrap-source-pins --root .
cairn_archive_dry_exit=0 cairn_archive_dry_wall_s=0
phase=final exit=0 group_survivors=0 end=2026-10-08T05:30:42Z elapsed_s=179 free=931371806720 max_drop=4026101760
```
