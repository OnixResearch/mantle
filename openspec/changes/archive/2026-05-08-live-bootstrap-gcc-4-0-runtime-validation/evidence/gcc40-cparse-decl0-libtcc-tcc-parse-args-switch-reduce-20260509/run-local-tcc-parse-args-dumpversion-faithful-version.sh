#!/usr/bin/env bash
set -euo pipefail

ROOT=${ROOT:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}
EVIDENCE_DIR="$ROOT/openspec/changes/archive/2026-05-08-live-bootstrap-gcc-4-0-runtime-validation/evidence/gcc40-cparse-decl0-libtcc-tcc-parse-args-switch-reduce-20260509"
WORK=${WORK:-/tmp/libtcc-tcc-version-faithful}
CACHE=${CACHE:-/tmp/crunch-tcc-version-faithful-cache}
RESTORE=${RESTORE:-/tmp/crunch-tcc-version-faithful-restore}
STATE_DIR=${STATE_DIR:-/home/brittonr/.local/state/crunch}
CRUNCH=${CRUNCH:-$ROOT/target/debug/crunch}

SOURCE_FRAGMENT=4jp67gx0k6h76np8wazalwlg5wyg5g3q-tcc-0.9.27-src
TCC_FRAGMENT=wl2qc53zbwka1qkq5kn1vk5lqzn6zh8l-tcc-0.9.27-musl-v2
MUSL_FRAGMENT=p1m8x9l2kbprlqbwy8xxahqjrkrb7v8i-musl-1.1.24-tcc-musl

restore_fragment() {
  local fragment=$1
  local out="$RESTORE/$fragment"
  if [ -e "$out" ]; then
    printf '%s\n' "$out"
    return 0
  fi
  mkdir -p "$CACHE" "$RESTORE"
  "$CRUNCH" store push --state-dir "$STATE_DIR" --store "$ROOT/.crunch-drain/store" --to "$CACHE" --trust-unsigned "$fragment" >/tmp/crunch-tcc-version-faithful-push.log
  local narinfo="$CACHE/${fragment%%-*}.narinfo"
  local url
  url=$(awk '/^URL:/{print $2}' "$narinfo")
  nix-store --restore "$out" < "$CACHE/$url"
  printf '%s\n' "$out"
}

TCC_SRC_PATH=$(restore_fragment "$SOURCE_FRAGMENT")
TCC_PATH=$(restore_fragment "$TCC_FRAGMENT")
MUSL_PATH=$(restore_fragment "$MUSL_FRAGMENT")

printf 'diag-libtcc-tcc-version-faithful: mode=crunch-cache-restored\n'
printf 'diag-libtcc-tcc-version-faithful: tcc=%s\n' "$TCC_PATH/bin/tcc"
printf 'diag-libtcc-tcc-version-faithful: musl_include=%s/include\n' "$MUSL_PATH"
printf 'diag-libtcc-tcc-version-faithful: source=%s\n' "$TCC_SRC_PATH"
printf 'diag-libtcc-tcc-version-faithful: define=-DTCC_VERSION="0.9.27-decl0-diag"\n'

rm -rf "$WORK"
mkdir -p "$WORK"

patch_and_run() {
  local source_script=$1
  local patched=$2
  python3 - "$source_script" "$patched" "$TCC_PATH" "$MUSL_PATH" "$TCC_SRC_PATH" <<'PY'
from pathlib import Path
import sys
src, dst, tcc, musl, tcc_src = map(Path, sys.argv[1:])
text = src.read_text()
text = text.replace('TCC="/crunch/store/b557z7jmjqnlz118c8zvwdjvkv4v5qb3-tcc-0.9.27-musl-v2"', f'TCC="{tcc}"')
text = text.replace('MUSL="/crunch/store/g38fpdz6wrs7n2p5f3afqbcb3w4s2m64-musl-1.1.24-tcc-musl"', f'MUSL="{musl}"')
text = text.replace('TCC_SRC="/crunch/store/4jp67gx0k6h76np8wazalwlg5wyg5g3q-tcc-0.9.27-src"', f'TCC_SRC="{tcc_src}"')
old = "-D CONFIG_USE_LIBGCC=1'"
new = "-D CONFIG_USE_LIBGCC=1 -DTCC_VERSION=\"0.9.27-decl0-diag\"'"
if old not in text:
    raise SystemExit(f'missing flags_common marker in {src}')
text = text.replace(old, new, 1)
dst.write_text(text)
dst.chmod(0o755)
PY
  printf 'diag-libtcc-tcc-version-faithful: running=%s\n' "${source_script##*/}"
  (cd "$WORK" && bash "$patched")
}

patch_and_run "$EVIDENCE_DIR/run-local-tcc-parse-args-dumpversion-printf-shape.sh" "$WORK/faithful-printf-shape.sh"
patch_and_run "$EVIDENCE_DIR/run-local-tcc-parse-args-dumpversion-macro-shape.sh" "$WORK/faithful-macro-shape.sh"
