# I3 source hardening evidence

Task-ID: I3
Covers: bootstrap.part.flex.2.6.4

- Added source provenance and first-consumer comments beside the `flex_src` fixed-output fetch.
- Removed suppressed object compile failures and fail-open partial object linking.
- Added fail-closed object, link, installed `flex`/`lex`/`flex++`, `libfl.a`, version, and scanner-generation smoke checks.
- Preserved conditional `FlexLexer.h` installation while failing closed if the copy path is taken and the header is not installed.
