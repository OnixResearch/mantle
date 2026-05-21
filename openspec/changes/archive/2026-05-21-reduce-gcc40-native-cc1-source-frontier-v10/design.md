# Design

Keep the v10 probe narrow and source-frontier-only:

1. Reuse the existing compact diagnostic derivation and focused `make -j1 -C gcc ... c-parse.o` command.
2. After capturing `/tmp/gcc40-cparse-make.log`, emit:
   - `cparse_make_include_flood=present|missing`
   - `cparse_make_include_flood_lines=<count>`
3. Fail closed only through the receipt/checker path, not by making the diagnostic succeed. The diagnostic may still exit with the native make return code.
4. Require the new markers and observed-frontier wording in parity validation.

This improves the active failure description from a generic make tail to a specific recursive include flood signature while retaining all v9 evidence and non-claims.
