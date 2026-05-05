# GCC 4.0 c-parse auto-host 83/84 boundary

Focused diagnostic target: `bootstrap/diag-gcc40-c-parse-boundary.ncl`.

This slice further narrows the generated `auto-host.h` define-prefix boundary
for the deterministic TinyCC segmentation fault in `gcc/c-parse.c` header/config
setup. The previous committed boundary was first `80` generated `#define`s
followed by `system.h` passing, and first `90` generated `#define`s followed by
`system.h` segfaulting.

The new diagnostic probes every prefix from `81` through `89` under the same
real c-parse compile flags and the verified `ansidecl.h` inline repair.

Result:

```text
diag-cparse: cparse_inc_config rc=0
diag-cparse: autohost_defines_10_system rc=0
diag-cparse: autohost_defines_25_system rc=0
diag-cparse: autohost_defines_50_system rc=0
diag-cparse: autohost_defines_60_system rc=0
diag-cparse: autohost_defines_70_system rc=0
diag-cparse: autohost_defines_80_system rc=0
diag-cparse: autohost_defines_81_system rc=0
diag-cparse: autohost_defines_82_system rc=0
diag-cparse: autohost_defines_83_system rc=0
diag-cparse: autohost_defines_84_system rc=139
diag-cparse: autohost_defines_85_system rc=139
diag-cparse: autohost_defines_86_system rc=139
diag-cparse: autohost_defines_87_system rc=139
diag-cparse: autohost_defines_88_system rc=139
diag-cparse: autohost_defines_89_system rc=139
diag-cparse: autohost_defines_90_system rc=139
diag-cparse: autohost_defines_95_system rc=139
diag-cparse: autohost_defines_100_system rc=139
diag-cparse: autohost_defines_200_system rc=139
diag-cparse: autohost_defines_400_system rc=139
diag-cparse: cparse_manual_config_system rc=139
diag-cparse: cparse_autohost_system rc=139
diag-cparse: cparse_ansidecl_system rc=0
diag-cparse: cparse_inc_system_only rc=0
diag-cparse: cparse_inc_system rc=139
diag-cparse: cparse_full rc=139
```

Interpretation:

- `autohost_defines_83_system rc=0` still passes.
- `autohost_defines_84_system rc=139` is the first observed segfaulting prefix.
- The remaining full configured probes still crash (`cparse_inc_system`,
  `cparse_autohost_system`, `cparse_manual_config_system`, and `cparse_full`).
- The repaired `ansidecl.h + system.h` control remains passing
  (`cparse_ansidecl_system rc=0`), so this slice keeps focus on generated
  `auto-host.h` macro context.

This is evidence-only narrowing; it does not claim GCC frontend completion.
