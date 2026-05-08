# V3 GCC 4.0 c-parse boundary host-leakage scan

Task-ID: V3
Covers: bootstrap.gcc40.runtime-validation

## Scope

Scanned the focused `bootstrap/gcc-4.0.ncl` validation evidence captured for the current deterministic boundary:

- `evidence/V2-gcc40-gcc-c-parse-boundary-build.stdout.log`
- `evidence/V2-gcc40-gcc-c-parse-boundary-build.stderr.log`
- `evidence/V2-gcc40-gcc-c-parse-boundary-wrapper.stdout.log`
- `evidence/V2-gcc40-gcc-c-parse-boundary-wrapper.stderr.log`
- `evidence/V2-gcc40-gcc-c-parse-boundary-drv.log`

The build still fails at `gcc/c-parse.c -> TinyCC Segmentation fault`; this scan is only a leakage audit for the captured failed build boundary, not proof that GCC 4.0 builds or runs.

## Command

```sh
python - <<'PY'
from pathlib import Path
base = Path('openspec/changes/live-bootstrap-gcc-4-0-runtime-validation/evidence')
files = sorted(base.glob('V2-gcc40-gcc-c-parse-boundary*log'))
needles = ['/nix/store', '/usr/bin', '/usr/lib', '/lib/x86_64', '/home/', '/bin/', '/tmp/', '/crunch/store']
for f in files:
    txt = f.read_text(errors='ignore')
    print(f.name)
    for n in needles:
        c = txt.count(n)
        if c:
            print(' ', n, c)
PY
```

## Result

Exit status: 0.

Transcript summary:

```text
V2-gcc40-gcc-c-parse-boundary-build.stderr.log
 size 0
V2-gcc40-gcc-c-parse-boundary-build.stdout.log
  /home/ 3
  /bin/ 31
  /tmp/ 255
  /crunch/store 14
 size 45773
V2-gcc40-gcc-c-parse-boundary-drv.log
  /bin/ 31
  /tmp/ 255
  /crunch/store 12
 size 44176
V2-gcc40-gcc-c-parse-boundary-wrapper.stderr.log
 size 0
V2-gcc40-gcc-c-parse-boundary-wrapper.stdout.log
  /home/ 8
  /bin/ 1
  /crunch/store 1
 size 1284
```

Interpretation:

- No `/nix/store`, `/usr/bin`, `/usr/lib`, or `/lib/x86_64` host-tool/libc paths appear in the captured build/driver logs.
- `/crunch/store` references are declared bootstrap inputs/outputs.
- `/tmp` references are sandbox-local build paths such as `/tmp/gcc-build`.
- `/bin/` references are sandbox paths (`/bin/sh`, `/bin/busybox`, generated `/tmp/gcc-tools` wrappers, and GCC build scripts invoking `/bin/sh`).
- `/home/` references occur in validation-wrapper metadata only: local evidence/store/state paths and the saved Crunch driver log path. They do not appear as compiler, libc, shell, or legacy-provider inputs inside the derivation transcript.

Conclusion: the current failed `c-parse.c` boundary transcript has no evidence of undeclared host compiler, host libc, host shell, Nix, or legacy-provider leakage. V4 remains open because no GCC 4.0 output exists for C/C++ smoke tests.
