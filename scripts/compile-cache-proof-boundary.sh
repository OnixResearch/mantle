#!/usr/bin/env bash

# r[impl mantle.cc_compile_cache.proof_exclusion]
# Call before tool discovery, scratch creation, or destructive release options.
require_compile_cache_exclusion() {
  local binding
  # Read the raw environment: shell variables omit names Bash cannot import.
  # NUL framing preserves newlines and non-UTF-8 bytes; never print values.
  if ! env -0 | while IFS= read -r -d '' binding; do
    case "$binding" in
      MANTLE_CC_CACHE_*=*|MANTLE_COMPILE_*=*)
        printf 'error: compiler-cache-proof-inherited-runtime-binding: %q\n' "${binding%%=*}" >&2
        return 1
        ;;
    esac
  done; then
    return 1
  fi
  printf 'compiler cache mode: excluded; cache receipts are not proof\n' >&2
}
