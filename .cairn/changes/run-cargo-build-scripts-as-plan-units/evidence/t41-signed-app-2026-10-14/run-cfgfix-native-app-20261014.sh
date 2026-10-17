#!/usr/bin/env bash
set -euo pipefail
umask 077
root=/tmp/mantle-unit-plan-20261013-87lqAtNv
free_bytes=$(df -B1 --output=avail "$root" | sed -n '2p')
home_bytes=$(df -B1 --output=avail /home/brittonr | sed -n '2p')
if (( free_bytes < 53687091200 || home_bytes < 2684354560 )); then
    printf 'cfgfix-app-resource-bound root_free=%s home_free=%s\n' "$free_bytes" "$home_bytes"
    exit 75
fi
test -s "$root/cfgfix-native-producer-verify.log"
test -s "$root/cfgfix-native-app-consumer.ncl"
test -s "$root/cfgfix-native-app-unit-id.txt"
test -x "$root/client-store/d7fc5i7y71rj8cr5jwmaxwjnyvfiybdp-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox"
test ! -e "$root/cfgfix-native-app-report.json"
test ! -e "$root/cfgfix-native-app-trace.log"
setsid -w bash -c '
    set -euo pipefail
    root=/tmp/mantle-unit-plan-20261013-87lqAtNv
    printf "%s\n" "$$" > "$root/owned-proof-cfgfix.pgid"
    cd "$root/source"
    export TMPDIR="$root/tmp" KACHE_CACHE_DIR="$root/kache" CRUNCH_NO_FUSE=1
    export SNIX_BUILD_BWRAP=/nix/store/lqndphylsxqwbwm804n473pb4sqb98sh-bubblewrap-0.11.2/bin/bwrap
    export SNIX_BUILD_SANDBOX_SHELL=/nix/store/d7fc5i7y71rj8cr5jwmaxwjnyvfiybdp-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox
    export PATH="/nix/store/lqndphylsxqwbwm804n473pb4sqb98sh-bubblewrap-0.11.2/bin:/run/wrappers/bin:/run/current-system/sw/bin:$PATH"
    export RUST_LOG="crunch_pipeline::evaluation_stream=info,crunch_build::orchestrate=info,snix_build::buildservice::bwrap=info"
    printf "phase=cfgfix-app-start time=%s no-substitute=true bwrap=%s verified_producer=true selected_unit=%s\n" "$(date -u +%FT%TZ)" "$SNIX_BUILD_BWRAP" "$(cat "$root/cfgfix-native-app-unit-id.txt")" > "$root/cfgfix-native-app-trace.log"
    timeout --foreground -v --signal=TERM --kill-after=10s 43200s "$root/target/debug/mantle" --json --log-level error --nix-compat --store "$root/client-store" --state-dir "$root/client-state" build --no-substitute --trusted-public-keys "mantle-unit-plan-local-1:jlpkELaOmQsXYXwNvRQxPfzO1fqmMfJ4yxpj4hbt2Kw=,mantle-unit-plan-remote-87lq:qXjNX6CsTA+vCkoh+nZDI3psVEfCLnvLhV4ZhfiDnt8=" -I . -I lib "$root/cfgfix-native-app-consumer.ncl" > "$root/cfgfix-native-app-report.json" 2>> "$root/cfgfix-native-app-trace.log"
    printf "phase=cfgfix-app-done time=%s\n" "$(date -u +%FT%TZ)" >> "$root/cfgfix-native-app-trace.log"
'
