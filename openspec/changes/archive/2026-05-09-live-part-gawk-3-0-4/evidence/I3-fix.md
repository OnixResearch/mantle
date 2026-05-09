# I3 source hardening evidence

Task-ID: I3
Covers: bootstrap.part.gawk.3.0.4

- Added source provenance and first-consumer comments beside the `gawk_src` fixed-output fetch.
- Removed suppressed object compile failures and fail-open partial object linking.
- Made the pre-generated parser source requirement explicit and fail-closed.
- Added fail-closed object, link, installed `gawk`/`awk`, and local/installed `awk-ok` smoke checks.
