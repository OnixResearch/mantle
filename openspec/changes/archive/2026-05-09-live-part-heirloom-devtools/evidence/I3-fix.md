# I3 source hardening evidence

Task-ID: I3
Covers: bootstrap.part.heirloom.devtools

- Added source provenance and first-consumer comments beside `heirloom_src`.
- Made `mk.config` edits fail closed instead of `|| true`.
- Converted manual `yacc`/`lex` compile fallback from best-effort suppressed failures into explicit object/link checks.
- Required build outputs before install and required both installed `yacc` and `lex`.
