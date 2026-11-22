#!/bin/sh
set -eu
case "$1" in
  child)
    printf '%s\n' child > "$out"
    ;;
  parent)
    IFS= read -r value < "$child"
    printf '%s\n' "$value" > "$out"
    printf '%s\n' "parent builder ran"
    ;;
  parent-cores)
    printf '%s\n' "${NIX_BUILD_CORES:-unset}" > "$out"
    ;;
  *)
    printf '%s\n' "unsupported fixture action: $1" >&2
    exit 1
    ;;
esac
