#!/bin/sh
set -eu
self_dir=${0%/*}
case "$self_dir" in
/*) ;;
*) self_dir="$(pwd -P)/$self_dir" ;;
esac
root_dir=$(cd "$self_dir/.." && pwd -P)
runtime_dir="$root_dir/lib/mantle-runtime"
loader="$root_dir/lib/mantle-runtime/libc.so"
ld_path="$runtime_dir:$root_dir/lib:$root_dir/lib/rustlib/x86_64-unknown-linux-musl/lib"
if [ "${LD_LIBRARY_PATH+x}" = x ] && [ -n "${LD_LIBRARY_PATH}" ]; then
ld_path="$ld_path:$LD_LIBRARY_PATH"
fi
if [ ! -x "$loader" ]; then
printf '%s\n' "provider dynamic runtime loader missing: $loader" >&2
exit 1
fi
has_sysroot=false
for arg in "$@"; do
if [ "$arg" = "--sysroot" ]; then has_sysroot=true; break; fi
case "$arg" in --sysroot=*) has_sysroot=true; break ;; esac
done
if [ "$has_sysroot" = false ]; then set -- --sysroot "$root_dir" "$@"; fi
LD_LIBRARY_PATH="$ld_path" exec "$loader" "$self_dir/rustc.dynamic" "$@"
