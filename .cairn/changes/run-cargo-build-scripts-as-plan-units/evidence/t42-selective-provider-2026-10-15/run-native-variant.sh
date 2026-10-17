#!/usr/bin/env bash
set -euo pipefail
umask 077
root=/tmp/mantle-unit-plan-20261013-87lqAtNv
test -s "$root/native-answer-variant.ncl"
test -s "$root/cfgfix-native-app-verify.log"
test ! -e "$root/native-variant.log"
free_bytes=$(df -B1 --output=avail "$root" | sed -n '2p')
if (( free_bytes < 53687091200 )); then printf 'native-variant-resource-bound free=%s floor=53687091200\n' "$free_bytes"; exit 75; fi
setsid -w bash -c '
    set -euo pipefail
    root=/tmp/mantle-unit-plan-20261013-87lqAtNv
    old_public=mantle-unit-plan-local-1:jlpkELaOmQsXYXwNvRQxPfzO1fqmMfJ4yxpj4hbt2Kw=
    new_public=mantle-unit-plan-remote-87lq:qXjNX6CsTA+vCkoh+nZDI3psVEfCLnvLhV4ZhfiDnt8=
    printf "%s\n" "$$" > "$root/owned-proof-cfgfix.pgid"
    cd "$root/source"
    export TMPDIR="$root/tmp" KACHE_CACHE_DIR="$root/kache" CRUNCH_NO_FUSE=1
    export SNIX_BUILD_BWRAP=/nix/store/lqndphylsxqwbwm804n473pb4sqb98sh-bubblewrap-0.11.2/bin/bwrap
    export SNIX_BUILD_SANDBOX_SHELL=/nix/store/d7fc5i7y71rj8cr5jwmaxwjnyvfiybdp-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox
    export PATH="/nix/store/lqndphylsxqwbwm804n473pb4sqb98sh-bubblewrap-0.11.2/bin:/run/wrappers/bin:/run/current-system/sw/bin:$PATH"
    printf "variant-native-provider-static-shell=%s no-substitute=true\n" "$SNIX_BUILD_SANDBOX_SHELL"
    timeout --foreground -v --signal=TERM --kill-after=10s 7200s "$root/target/debug/mantle" --json --nix-compat --store "$root/client-store" --state-dir "$root/client-state" build --no-substitute --trusted-public-keys "$old_public,$new_public" -I . -I lib "$root/native-answer-variant.ncl"
' > "$root/native-variant.log" 2>&1
