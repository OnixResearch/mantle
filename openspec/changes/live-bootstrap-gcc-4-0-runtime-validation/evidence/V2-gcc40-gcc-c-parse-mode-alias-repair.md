# V2 gcc-4.0 `c-parse.c` mode-bound alias repair

## Change

Materialized the generated `insn-modes.h` mode-class bound aliases used by
`tree.h`'s complex builtin enum arithmetic:

```c
#define MIN_MODE_COMPLEX_FLOAT SFmode
#define MAX_MODE_COMPLEX_FLOAT TFmode
```

The repair was applied to both the production `bootstrap/gcc-4.0.ncl` wrapper
and the focused `bootstrap/diag-gcc40-c-parse-boundary.ncl` diagnostic wrapper.
This matches the prior cross-enum evidence where explicit generated-context
aliases to `SFmode` / `TFmode` compiled successfully, while unresolved
`MIN_MODE_COMPLEX_FLOAT` / `MAX_MODE_COMPLEX_FLOAT` identifiers reproduced the
TinyCC/Mes `rc=139` crash class.

## Verification

Commands run from `/home/brittonr/git/crunch/crunch`:

```sh
./target/debug/crunch eval bootstrap/gcc-4.0.ncl >/tmp/gcc40.eval.json
./target/debug/crunch eval bootstrap/diag-gcc40-c-parse-boundary.ncl >/tmp/diag-gcc40.eval.json
```

Result:

```text
/tmp/gcc40.eval.json gcc-4.0.4
/tmp/diag-gcc40.eval.json diag-gcc40-c-parse-boundary
```

Static alias/materialization check and host C enum arithmetic smoke:

```sh
python3 - <<'PY'
from pathlib import Path
for p in ['bootstrap/gcc-4.0.ncl','bootstrap/diag-gcc40-c-parse-boundary.ncl']:
    text=Path(p).read_text()
    assert '#define MIN_MODE_COMPLEX_FLOAT SFmode' in text, p
    assert '#define MAX_MODE_COMPLEX_FLOAT TFmode' in text, p
print('mode alias materialization present in both gcc-4.0 and diagnostic genmodes fallback')
PY
cat > /tmp/gcc40-mode-alias-smoke.c <<'EOF'
enum machine_mode { VOIDmode, BLKmode, CCmode, BImode, QImode, HImode, SImode, DImode, TImode, SFmode, DFmode, XFmode, TFmode, MAX_MACHINE_MODE, NUM_MACHINE_MODES = MAX_MACHINE_MODE };
#define MIN_MODE_COMPLEX_FLOAT SFmode
#define MAX_MODE_COMPLEX_FLOAT TFmode
enum built_in_function {
  BUILT_IN_COMPLEX_MUL_MIN,
  BUILT_IN_COMPLEX_MUL_MAX = BUILT_IN_COMPLEX_MUL_MIN + MAX_MODE_COMPLEX_FLOAT - MIN_MODE_COMPLEX_FLOAT,
  END_BUILTINS
};
_Static_assert(BUILT_IN_COMPLEX_MUL_MAX == 3, "expected TFmode-SFmode range");
int main(void) { return BUILT_IN_COMPLEX_MUL_MAX == 3 ? 0 : 1; }
EOF
gcc -std=c11 -Wall -Wextra -c /tmp/gcc40-mode-alias-smoke.c -o /tmp/gcc40-mode-alias-smoke.o
```

Result:

```text
mode alias materialization present in both gcc-4.0 and diagnostic genmodes fallback
host gcc alias smoke passed
```

A fresh full `crunch build --no-substitute bootstrap/diag-gcc40-c-parse-boundary.ncl`
was attempted with an isolated store/state, but did not reach the diagnostic
`diag-cparse:` phase within the local foreground/background budget because the
fresh state was still rebuilding Mes prerequisite outputs. The killed prerequisite
attempt produced no `diag-cparse:` lines, so it is not used as boundary evidence.

Completed UTC: 2026-05-06T22:14:37Z
