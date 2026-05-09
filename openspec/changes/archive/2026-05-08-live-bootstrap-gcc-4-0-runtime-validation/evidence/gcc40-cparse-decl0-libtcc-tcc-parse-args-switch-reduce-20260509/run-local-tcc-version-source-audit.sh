#!/usr/bin/env bash
set -euo pipefail

ROOT=${ROOT:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}
EVIDENCE_DIR="$ROOT/openspec/changes/archive/2026-05-08-live-bootstrap-gcc-4-0-runtime-validation/evidence/gcc40-cparse-decl0-libtcc-tcc-parse-args-switch-reduce-20260509"
SCRIPTS=(
  "$EVIDENCE_DIR/run-local-tcc-parse-args-dumpversion-printf-shape.sh"
  "$EVIDENCE_DIR/run-local-tcc-parse-args-dumpversion-macro-shape.sh"
)
BOOTSTRAP_DIAG="$ROOT/bootstrap/diag-gcc40-c-parse-boundary.ncl"
BOOTSTRAP_TCC="$ROOT/bootstrap/tcc-musl-v2.ncl"

printf 'diag-libtcc-tcc-version-source-audit: scripts=%s\n' "${SCRIPTS[*]}"
for script in "${SCRIPTS[@]}"; do
  name=${script##*/}
  printf 'diag-libtcc-tcc-version-source-audit: script=%s\n' "$name"
  if grep -n '^: > config\.h$' "$script"; then
    printf 'diag-libtcc-tcc-version-source-audit: %s clears generated config.h\n' "$name"
  else
    printf 'diag-libtcc-tcc-version-source-audit: %s does not clear config.h as expected\n' "$name"
  fi
  flags_line=$(grep -n "^flags_common=" "$script" || true)
  printf 'diag-libtcc-tcc-version-source-audit: %s flags_common=%s\n' "$name" "$flags_line"
  if grep -q "^flags_common=.*TCC_VERSION" "$script"; then
    printf 'diag-libtcc-tcc-version-source-audit: %s has TCC_VERSION in flags_common\n' "$name"
  else
    printf 'diag-libtcc-tcc-version-source-audit: %s lacks TCC_VERSION in flags_common\n' "$name"
  fi
  if grep -q "compile_variant .*'-D TCC_VERSION" "$script"; then
    printf 'diag-libtcc-tcc-version-source-audit: %s passes TCC_VERSION through extra flags\n' "$name"
  else
    printf 'diag-libtcc-tcc-version-source-audit: %s does not pass TCC_VERSION through extra flags\n' "$name"
  fi
  grep -n '#define TCC_VERSION' "$script" || true
  grep -n '#define CRUNCH_LOCAL_VERSION' "$script" || true
  grep -n 'static .*TCC_VERSION' "$script" || true
  printf '\n'
done

printf 'diag-libtcc-tcc-version-source-audit: crunch derivation definitions\n'
grep -n 'TCC_VERSION=' "$BOOTSTRAP_DIAG" | tail -5 || true
grep -n 'TCC_VERSION=' "$BOOTSTRAP_TCC" | tail -5 || true

printf 'diag-libtcc-tcc-version-source-audit: conclusion=focused replay clears config.h and omits command-line TCC_VERSION; direct TCC_VERSION uses are unresolved identifier tokens, while the successful local spelling is #define TCC_VERSION "0.9.27".\n'
