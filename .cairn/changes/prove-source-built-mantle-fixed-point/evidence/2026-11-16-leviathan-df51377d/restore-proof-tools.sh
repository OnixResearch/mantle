#!/usr/bin/env bash
set -euo pipefail
ROOT=/home/brittonr/sbfp-20261113-berdKH
B3=/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum
SHELL_ROOT=/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0
BWRAP_ROOT=/nix/store/3hqwbwxij9lxhyrf2amsm2xg24xszmmx-bubblewrap-0.12.0
test ! -e "$ROOT/restore-proof-tools-df51377d-unit-plans.log"
exec > "$ROOT/restore-proof-tools-df51377d-unit-plans.log" 2>&1
trap 'status=$?; printf "restore_exact_proof_tools_exit=%s\n" "$status"; exit "$status"' EXIT
set -x
/run/current-system/sw/bin/nix-store --option builders '' --add-root "$ROOT/proof-tool-static-shell-df51377d-unit-plans" --indirect --realise "$SHELL_ROOT"
/run/current-system/sw/bin/nix-store --option builders '' --add-root "$ROOT/proof-tool-bwrap-df51377d-unit-plans" --indirect --realise "$BWRAP_ROOT"
test "$("$B3" --no-names "$SHELL_ROOT/bin/busybox")" = ac277a3b87ae477a75067b89de22865222d1d01e1cb7f91ca5f5eeca8e8ef6bc
test "$("$B3" --no-names "$BWRAP_ROOT/bin/bwrap")" = 9979941d82a18db70c4fa1bed4cd3d33504c1d8b35b9f08b6784e494abbdcece
printf 'exact_original_proof_tools_rooted=1\n'
