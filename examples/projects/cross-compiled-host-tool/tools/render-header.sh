#!/bin/sh
set -eu
if [ "$#" -ne 2 ]; then
  echo 'usage: render-header.sh <target-triple> <output>' >&2
  exit 64
fi
target_triple="$1"
output="$2"
case "$target_triple" in
  *[!A-Za-z0-9_.-]*|'')
    echo 'target triple contains unsupported characters' >&2
    exit 65
    ;;
esac
cat > "$output" <<EOF
#ifndef BUILD_TARGET_H
#define BUILD_TARGET_H
#define BUILD_TARGET_TRIPLE "$target_triple"
#define BUILD_TARGET_ROLE "target"
#endif
EOF
