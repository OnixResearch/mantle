#!/bin/sh
set -eu
if [ "$1" = fail ]; then
  exit 43
fi
echo "$1" > "$out"
