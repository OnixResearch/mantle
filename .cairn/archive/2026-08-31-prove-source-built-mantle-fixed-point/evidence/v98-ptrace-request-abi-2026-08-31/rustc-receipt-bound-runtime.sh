#!/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v98-ptrace-request-abi-20260831.source-built-fixed-point-staging-3680890/rust-host-tools/busybox/bin/sh
set -eu
has_sysroot=false
for arg in "$@"; do
  if [ "$arg" = "--sysroot" ]; then has_sysroot=true; break; fi
  case "$arg" in --sysroot=*) has_sysroot=true; break ;; esac
done
if [ "$has_sysroot" = false ]; then set -- --sysroot '/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v98-ptrace-request-abi-20260831.source-built-fixed-point-staging-3680890/rust-provider' "$@"; fi
set -- "$@" -C 'linker=/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v98-ptrace-request-abi-20260831.source-built-fixed-point-staging-3680890/cargo-free-fixed-point/toolchain/receipt-bound-path/cc'
LD_LIBRARY_PATH='/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v98-ptrace-request-abi-20260831.source-built-fixed-point-staging-3680890/rust-provider/lib/mantle-runtime:/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v98-ptrace-request-abi-20260831.source-built-fixed-point-staging-3680890/native-store/kcvijyh0sibqcc4g9sqk694xyslzhpsx-full-source-seed-toolchain/x86_64-linux-musl/lib:/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v98-ptrace-request-abi-20260831.source-built-fixed-point-staging-3680890/rust-provider/lib:/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v98-ptrace-request-abi-20260831.source-built-fixed-point-staging-3680890/rust-provider/lib/rustlib/x86_64-unknown-linux-musl/lib'
export LD_LIBRARY_PATH
exec '/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v98-ptrace-request-abi-20260831.source-built-fixed-point-staging-3680890/native-store/kcvijyh0sibqcc4g9sqk694xyslzhpsx-full-source-seed-toolchain/x86_64-linux-musl/lib/ld-musl-x86_64.so.1' '/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v98-ptrace-request-abi-20260831.source-built-fixed-point-staging-3680890/rust-provider/bin/rustc.dynamic' "$@"
