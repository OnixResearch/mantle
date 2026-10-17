#!/usr/bin/env bash
set -euo pipefail
umask 077
root=/tmp/mantle-unit-plan-20261013-87lqAtNv
test -s "$root/cfgfix-native-app-verify.log"
test -s "$root/variant-native-provider-verify.log"
test -s "$root/variant-native-config-verify.log"
test ! -e "$root/variant-native-app.log"
test ! -e "$root/variant-native-producer-trace.log"
test -s "$root/variant-native-plan.ncl"
test -s "$root/variant-native-plan-eval.json"
free_bytes=$(df -B1 --output=avail "$root" | sed -n '2p')
if (( free_bytes < 53687091200 )); then printf 'variant-app-resource-bound free=%s floor=53687091200\n' "$free_bytes"; exit 75; fi
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
    export RUST_LOG="crunch_build::orchestrate=info"
    printf "phase=variant-producer-start time=%s no-substitute=true source=%s\n" "$(date -u +%FT%TZ)" "$root/variant-native-plan.ncl" > "$root/variant-native-producer-trace.log"
    printf "variant-app-native-root-source=%s sandbox-shell=%s no-substitute=true\n" "$root/variant-native-plan.ncl" "$SNIX_BUILD_SANDBOX_SHELL"
    timeout --foreground -v --signal=TERM --kill-after=10s 64800s "$root/target/debug/mantle" --json --nix-compat --store "$root/client-store" --state-dir "$root/client-state" build --no-substitute --trusted-public-keys "$old_public,$new_public" -I . -I lib "$root/variant-native-plan.ncl" 2>> "$root/variant-native-producer-trace.log"
    printf "phase=variant-producer-done time=%s\n" "$(date -u +%FT%TZ)" >> "$root/variant-native-producer-trace.log"
' > "$root/variant-native-app.log" 2>&1
