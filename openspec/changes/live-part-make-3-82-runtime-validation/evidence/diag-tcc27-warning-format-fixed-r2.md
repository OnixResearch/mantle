# Diagnostic: TinyCC 0.9.27 warning-format repair (fixed r2)

Date: 2026-05-04

## Change

`bootstrap/tinycc.ncl` now keeps deterministic bootstrap path construction on direct string-copy paths, but replaces TinyCC's diagnostic append formatter with a small direct `va_arg` formatter instead of calling Mes `vsnprintf` on a forwarded TinyCC x86_64 `va_list`.

`bootstrap/diag-tcc27-warning-format.ncl` was converted from a reproducer of literal placeholders into a regression guard that rejects `%s:%d` placeholders and requires concrete source location/symbol warnings.

## Validation

Command used `crunch bootstrap validate bootstrap/diag-tcc27-warning-format.ncl --warmup bootstrap/tinycc.ncl --resume` with a fresh local store/state under `.crunch-drain/diag-tcc27-warning-format-fixed-r2-*`.

Result: `passed`; `bootstrap/tinycc.ncl` warmup passed; diagnostic build passed.

Captured `warn.stderr`:

```text
warn.c:3: warning: implicit declaration of function 'missing_warning_symbol'
warn.c:3: warning: implicit declaration of function 'missing_warning_symbol'
warn.c:3: warning: assignment makes pointer from integer without a cast
warn.c:3: warning: assignment makes pointer from integer without a cast
```

Evidence files:

- `diag-tcc27-warning-format-fixed-r2-validation-summary.json`
- `diag-tcc27-warning-format-fixed-r2-validation-summary.md`
- `diag-tcc27-warning-format-fixed-r2-build.stdout.log`
- `diag-tcc27-warning-format-fixed-r2-build.stderr.log`
- `diag-tcc27-warning-format-fixed-r2-warmup-tinycc.stdout.log`
- `diag-tcc27-warning-format-fixed-r2-warmup-tinycc.stderr.log`
- `diag-tcc27-warning-format-fixed-r2-doctor.json`
