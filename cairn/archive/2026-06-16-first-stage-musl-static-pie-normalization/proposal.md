# first-stage musl static-pie normalization

## Problem

The musl-host Rust source provider now routes plain `cc` through the generated source-root musl wrapper, but the next real run fails when Rust's first-stage `run_rustc` smoke link requests `-static-pie`. The source-root musl seed provides a static `libc.a` that links successfully with `-static` but is not compiled as PIE, so `x86_64-linux-musl-ld` rejects relocations from `libc.a` under `-static-pie`.

## Proposed change

Normalize first-stage source-root musl target wrapper link arguments from `-static-pie` to `-static` before invoking the source-root target GCC. Keep the normalization inside the generated private wrapper so it only applies to first-stage musl target links and does not change generic host aliases or global PATH.

## Success criteria

- The generated first-stage wrapper script rewrites `-static-pie` to `-static` for source-root musl target links.
- Focused tests prove the wrapper script contains the normalization and still exposes only private target aliases.
- Evidence records the real provider rerun frontier and a direct source-root probe proving `-static-pie` fails while `-static` works with the current seed libc.
