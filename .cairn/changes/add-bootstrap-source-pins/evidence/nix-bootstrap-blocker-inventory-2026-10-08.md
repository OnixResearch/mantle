# Nix check bootstrap-blocker-inventory (pre-rebase tree ac68e8eb, 2026-10-08 UTC)

Guarded Leviathan run `/home/brittonr/mantle-cairn-finish/logs/add-bootstrap-source-pins/nix-blocker-20261008T010250Z` (mcf-guard.sh; full log retained on Leviathan).
Job script: ``.

## guard.log

```text
phase=preflight start=2026-10-08T01:02:50Z free=1033726783488 floor=214748364800 peak_reserve=429496729600 dev=66305
cwd=/home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins head=ac68e8ebed7a7fe8242d7a6ab9a9c73da0a0d86c CARGO_TARGET_DIR=/home/brittonr/mantle-cairn-finish/targets/add-bootstrap-source-pins TMPDIR=/home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins CARGO_BUILD_JOBS=32
command=/home/brittonr/mantle-cairn-finish/bin/jobs/bsp-nix.sh
phase=running launcher=1363564 pgid=1363564
phase=child_exited exit=0 elapsed_s=28 free=1025103757312
phase=final exit=0 group_survivors=0 end=2026-10-08T01:03:18Z elapsed_s=28 free=1025097187328
```

## exit-code

```text
0
```

## output.log (cargo Compiling/Checking progress lines elided)

```text
head=ac68e8ebed7a7fe8242d7a6ab9a9c73da0a0d86c
## finish/add-bootstrap-source-pins
this derivation will be built:
  /nix/store/icd3mn6a0vg5nr2cgpsfja48svz06vkd-bootstrap-blocker-inventory.drv
building '/nix/store/icd3mn6a0vg5nr2cgpsfja48svz06vkd-bootstrap-blocker-inventory.drv'...
bootstrap-blocker-inventory> gcc40 configure preprocess bridge: PASS (classes=3, source_spellings=2, source_bytes_max=65536, invocation_count_max=4096)
bootstrap-blocker-inventory> bootstrap blocker inventory: 0 findings across 0 classes, 470 evidence-backed suppressions, 0 promotion claims, enforce=true
/nix/store/lbfb2m0asfa2ismjd36v2dvysgv3hk83-bootstrap-blocker-inventory
nix_bootstrap_blocker_inventory_exit=0 wall_s=26
```
