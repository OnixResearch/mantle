#!/usr/bin/env bash
# Detached, standalone after the current release/static binaries are built: fresh signed-cache client.
set -euo pipefail
R=$HOME/mantle-cairn-finish
WT=$R/wt/rust-efficiency-20261011
H=$WT/scripts/rust-unit-plan-build-script-replay
B=${RCBS_B:-$R/tmp/rust-efficiency-20261011/rcbs-unit}
STORE=${RCBS_STORE:-$R/stores/rust-efficiency-20261011/rcbs-client}
MANTLE=$R/targets/rust-efficiency-20261011/release/mantle
PYTHON=/nix/store/3n4qphl9s728sz8frmpqqrv9b1m87g68-python3-3.14.7/bin/python3
test -x "$PYTHON"
export SNIX_BUILD_BWRAP=/nix/store/vvxfl9d1b9a944ca7miswhnxg06cs6jl-mantle-wasm-component-toolchain-v1/bin/bwrap
export SNIX_BUILD_SANDBOX_SHELL=/nix/store/h75l4bwd606k94l22ajqdb98vfar5x66-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox
[[ ! -e $B && ! -e $STORE ]] || { echo 'proof scratch/client exists; refusing unchecked reuse' >&2; exit 1; }
# A host-reachable loopback listener distinguishes sandbox denial from an unreachable public IP.
coproc NETWORK_PROBE {
  exec "$PYTHON" -u -c '
import socket
listener = socket.socket()
listener.bind(("127.0.0.1", 0))
listener.listen(8)
print(listener.getsockname()[1], flush=True)
while True:
    connection, _ = listener.accept()
    connection.close()
'
}
probe_pid=$NETWORK_PROBE_PID
trap 'kill "$probe_pid" 2>/dev/null || true; wait "$probe_pid" 2>/dev/null || true' EXIT
IFS= read -r RCBS_NETWORK_PORT <&"${NETWORK_PROBE[0]}"
[[ $RCBS_NETWORK_PORT =~ ^[0-9]+$ ]] || { echo 'invalid loopback probe port' >&2; exit 1; }
"$PYTHON" -c 'import socket, sys; socket.create_connection(("127.0.0.1", int(sys.argv[1])), timeout=2).close()' "$RCBS_NETWORK_PORT"
export RCBS_NETWORK_PORT
cd "$WT"
"$H/prepare.sh"
p() { jq -er --arg key "$1" '.[$key]' "$B/paths.json"; }
mapfile -t SA < <(printf '%s\n' --nix-compat --store "$STORE/store" --state-dir "$STORE/state")
mkdir -p "$STORE/store" "$STORE/state"
"$MANTLE" "${SA[@]}" store pull --from "$B/input-cache" --trusted-public-keys "$(p public_key)" --all > "$B/input-pull.stdout" 2> "$B/input-pull.stderr"
# libbz2 is rebuilt twice by *this* Mantle client; no historical library output enters the cache.
lib_goal() {
  local tag=$1 note=$2
  cat > "$B/ncl/lib-$tag.ncl" <<EOF
let recipe = import "$H/libbz2.ncl" in
recipe { cc_toolchain = "$(p linker)", cflags_note = "$note" }
EOF
  build "lib-$tag" "$B/ncl/lib-$tag.ncl" success
  local out
  out=$(jq -er '.outcomes[] | select(.label == "libbz2-1.0.8") | .outputs[] | select(.name == "out") | .path' "$B/runs/lib-$tag/report.json")
  test -s "$out/lib/libbz2.a" && test -s "$out/lib/pkgconfig/bzip2.pc"
  printf '%s\n' "$out"
}
build() {
  local tag=$1 file=$2 expectation=$3 rc=0
  local dir=$B/runs/$tag
  mkdir "$dir" "$dir/logs"
  cp "$file" "$dir/build.ncl"
  printf 'mantle build %s\n' "$tag" >&2
  CRUNCH_LOG_DIR="$dir/logs" "$MANTLE" --json "${SA[@]}" build --no-substitute -I "$WT/lib" "$file" > "$dir/report.json" 2> "$dir/stderr" || rc=$?
  echo "$rc" > "$dir/exit"
  jq -e '.schema == "crunch-build-report-v1" and (.counts.failed_total >= 0)' "$dir/report.json" >/dev/null
  if [[ $expectation == success ]]; then
    [[ $rc == 0 ]] && jq -e '.counts.failed_total == 0' "$dir/report.json" >/dev/null
  else
    [[ $rc != 0 ]] && jq -e '.counts.failed_total > 0' "$dir/report.json" >/dev/null
  fi
}
# Preserve exact Cargo registry package identity (including +1.0.8) across config and Nickel roles.
PKG_ID='registry+https://github.com/rust-lang/crates.io-index#bzip2-sys@0.1.13+1.0.8'
config() { # tag workspace native-JSON -> immutable signed producer config
  local tag=$1 workspace=$2 native=$3 path
  jq -cn --arg workspace "$workspace" --arg vendor "$(p vendor)" --arg planner "$(p planner)" \
    --arg toolchain "$(p toolchain)" --arg compiler "$(p compiler)" --arg linker "$(p linker)" --arg helper "$(p helper)" \
    --argjson native "$(cat "$native")" '{workspace:$workspace,vendor:$vendor,planner:$planner,
      cargo:($toolchain + "/bin/cargo"),rustc:($compiler + "/bin/rustc"),linker:($linker + "/bin/cc"),runtime_inputs:[$compiler],
      toolchain:$toolchain,helper:$helper,store_prefix:"/nix/store",system:"x86_64-linux",
      host_triple:"x86_64-unknown-linux-gnu",target_triple:"x86_64-unknown-linux-gnu",
      profile:"dev",native_inputs:$native}' > "$B/stage/$tag-config.json"
  path=$(nix-store --add "$B/stage/$tag-config.json")
  nix copy --offline --to "file://$B/input-cache?compression=none&secret-key=$B/keys/secret" "$path"
  "$MANTLE" "${SA[@]}" store pull --from "$B/input-cache" --trusted-public-keys "$(p public_key)" --all > "$B/runs/$tag-config-pull.stdout" 2> "$B/runs/$tag-config-pull.stderr"
  printf '%s\n' "$path"
}
goal() { # tag workspace native_json, identical JSON read by producer and Nickel
  local tag=$1 ws=$2 native=$3 cfg
  cfg=$(config "$tag" "$ws" "$native")
  cat > "$B/ncl/$tag.ncl" <<EOF
let cargo_unit_plan = import "$WT/lib/cargo_unit_plan.ncl" in
cargo_unit_plan {
  goal_name = "rcbs-$tag", producer = "$(p producer)", config = "$cfg",
  workspace = "$ws", planner = "$(p planner)", toolchain = "$(p toolchain)",
  linker = "$(p linker)", helper = "$(p helper)", vendor_sources = ["$(p vendor)"],
  runtime_inputs = ["$(p compiler)"],
  native_inputs = import "$native",
}
EOF
}
plan() { # success: check accepted plan against actual content and capture machine-readable plan artifact
  local tag=$1 report=$B/runs/$1/report.json logical physical
  jq -e '.native_dynamic_plans | length == 1 and .[0].scheduler_action == "registered-roots" and
    (.[0].canonical_plan_digest | type == "string" and length == 64) and (.[0].accepted_unit_ids | length > 0)' "$report" >/dev/null
  logical=$(jq -er '.native_dynamic_plans[0].plan_artifact_path' "$report")
  physical=$STORE/store/${logical##*/}
  test -s "$physical"
  jq -e --slurpfile report "$report" '([.units[].id] | sort) == ([$report[0].native_dynamic_plans[0].accepted_unit_ids[]] | sort)' "$physical" >/dev/null
  cp "$physical" "$B/runs/$tag/accepted-plan.json"
}
lib_a=$(lib_goal a 11)
lib_b=$(lib_goal b 22)
test "$lib_a" != "$lib_b"
for tag in a b; do
  if [[ $tag == a ]]; then lib=$lib_a; else lib=$lib_b; fi
  jq -cn --arg pkg "$PKG_ID" --arg lib "/nix/store/${lib##*/}" --arg pkg_config "$(p pkg_config)" \
    '{($pkg):{inputs:{bzip2:$lib,"pkg-config":$pkg_config},env:{PKG_CONFIG_PATH:"{{role:bzip2}}/lib/pkgconfig"},path:["{{role:pkg-config}}/bin"]}}' > "$B/ncl/bz-$tag-native.json"
  goal "bz-$tag" "$(p bz)" "$B/ncl/bz-$tag-native.json"
done
printf '{}\n' > "$B/ncl/empty-native.json"
for tag in graphs links nonzero host network; do
  goal "$tag" "$(p "$tag")" "$B/ncl/empty-native.json"
done
# Offline Cargo is a separate functional oracle, not sandbox evidence; uses same original immutable sources.
# Parity runs on the host, where Mantle's logical /nix/store paths are not
# exported; rebase the one exact library root, including literal -L/-I flags,
# to the same physical client output without altering the native sandbox input.
mkdir -p "$B/offline/pkgconfig"
sed "s|/nix/store/${lib_a##*/}|$lib_a|g" \
  "$lib_a/lib/pkgconfig/bzip2.pc" > "$B/offline/pkgconfig/bzip2.pc"
grep -Fxq "Libs: -L$lib_a/lib -lbz2" "$B/offline/pkgconfig/bzip2.pc"
grep -Fxq "Cflags: -I$lib_a/include" "$B/offline/pkgconfig/bzip2.pc"
for tag in graphs links bz; do
  if [[ $tag == graphs ]]; then ws=$(p graphs); elif [[ $tag == links ]]; then ws=$(p links); else ws=$(p bz); fi
  cp -r "$ws" "$B/offline/$tag"
  chmod -R u+w "$B/offline/$tag"
  mkdir -p "$B/offline/$tag/.cargo"
  printf '[source.crates-io]\nreplace-with = "fixture-vendor"\n[source.fixture-vendor]\ndirectory = "%s"\n[net]\noffline = true\n' "$(p vendor)" > "$B/offline/$tag/.cargo/config.toml"
  (cd "$B/offline/$tag" && env CARGO_HOME="$B/offline/cargo-home" CARGO_TARGET_DIR="$B/offline/$tag-target" \
    PKG_CONFIG_PATH="$B/offline/pkgconfig" PATH="$(p pkg_config)/bin:$(p linker)/bin:$(p toolchain)/bin:$PATH" \
    "$(p toolchain)/bin/cargo" build --locked --offline --message-format=json) > "$B/offline/$tag-build.jsonl" 2> "$B/offline/$tag-build.stderr"
  exe=$(jq -r 'select(.reason == "compiler-artifact" and .executable != null) | .executable' "$B/offline/$tag-build.jsonl" | sed -n '$p')
  test -x "$exe"
  "$exe" > "$B/offline/$tag.stdout"
done
build graphs "$B/ncl/graphs.ncl" success; plan graphs
build links "$B/ncl/links.ncl" success; plan links
build bz-a "$B/ncl/bz-a.ncl" success; plan bz-a
build bz-b "$B/ncl/bz-b.ncl" success; plan bz-b
# A native role change alters the script and dependent root; unchanged units must not rebuild.
"$H/check.sh" positive "$B"
build nonzero "$B/ncl/nonzero.ncl" failure
plan nonzero
grep -R -q 'build-script-nonzero-exit' "$B/runs/nonzero/logs"
build host "$B/ncl/host.ncl" success; plan host
build network "$B/ncl/network.ncl" success; plan network
"$H/check.sh" sandbox "$B"
# A missing declaration cannot access the previously built native library in the sandbox.
goal bz-undeclared "$(p bz)" "$B/ncl/empty-native.json"
build bz-undeclared "$B/ncl/bz-undeclared.ncl" failure
plan bz-undeclared
# The Nickel validator rejects an undeclared role before producer execution.
jq -cn --arg pkg "$PKG_ID" --arg pkg_config "$(p pkg_config)" \
  '{($pkg):{inputs:{"pkg-config":$pkg_config},env:{PKG_CONFIG_PATH:"{{role:absent}}/lib/pkgconfig"},path:["{{role:pkg-config}}/bin"]}}' > "$B/ncl/wrong-role-native.json"
goal bz-wrong-role "$(p bz)" "$B/ncl/wrong-role-native.json"
build bz-wrong-role "$B/ncl/bz-wrong-role.ncl" failure
jq -e '(.native_dynamic_plans | length) == 0 and (.failed | any(.[]; .message | contains("native-input-role-undeclared")))' "$B/runs/bz-wrong-role/report.json" >/dev/null
# A host path presented as a native role is rejected by the Nickel StorePath contract.
jq -cn --arg pkg "$PKG_ID" '{($pkg):{inputs:{bzip2:"/etc"},env:{PKG_CONFIG_PATH:"{{role:bzip2}}/lib/pkgconfig"},path:[]}}' > "$B/ncl/host-role-native.json"
goal bz-host-role "$(p bz)" "$B/ncl/host-role-native.json"
build bz-host-role "$B/ncl/bz-host-role.ncl" failure
"$H/check.sh" negatives "$B"
printf 'native build-script replay PASS: %s\n' "$B/summary.json"
