#!/nix/store/sfvyavxai6qvzmv9p9x6mp4wwdz4v41m-bash-interactive-5.3p9/bin/bash
set -euo pipefail
expected='cargo-build-script-fixture: proc-macro2 serde-derive libc native-sys = 42'
actual="$("$MANTLE_UNIT_APP/lib/mantle_cargo_build_script_fixture")"
if [[ "$actual" != "$expected" ]]; then
  printf 'unexpected Rust fixture result: %s\n' "$actual" >&2
  exit 1
fi
mkdir -p "$out"
printf '%s\n' "$actual" > "$out/result.txt"
