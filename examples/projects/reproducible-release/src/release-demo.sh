#!/bin/sh
set -eu

ARGUMENT_COUNT_MAX=1
EXIT_USAGE=64

if [ "$#" -gt "$ARGUMENT_COUNT_MAX" ]; then
    echo 'usage: release-demo [name]' >&2
    exit "$EXIT_USAGE"
fi

name=${1:-World}
printf 'Release hello, %s!\n' "$name"
