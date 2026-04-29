Task-ID: V4
Covers: bootstrap.part.mes.0.27

# Host leakage scan

Commands:

```sh
awk 'found {print} /^output:/ {found=1; print}' \
  evidence/V2-build-full.log > evidence/V2-build-transcript-only.log

rg -n '(/bin/[A-Za-z0-9_.+-]+|/usr/|/home/|/nix/store|/run/)' \
  bootstrap/mes.ncl \
  evidence/V2-build-transcript-only.log \
  > evidence/V4-host-leakage-focused.txt || true

rg -n '(/home/|/nix/store|/usr/bin|/usr/lib|/usr/include|\b(gcc|clang|cc|make|tar|curl|wget|git)\b)' \
  bootstrap/mes.ncl \
  evidence/V2-build-transcript-only.log \
  evidence/V3-smoke-full.log \
  > evidence/V4-host-leakage-raw.txt || true
```

Result: PASS.

Reviewed files:

- `bootstrap/mes.ncl`
- `evidence/V2-build-transcript-only.log`
- `evidence/V3-smoke-full.log`

Focused scan findings are expected / benign:

- Declared sandbox boundary: `builder = "/bin/sh"` and `BB=/bin/busybox`.
- In-sandbox shell use: generated wrappers and `/bin/sh "$MES_PREFIX/build-aux/build-lib.sh"`.
- Output-internal paths: `$out/bin/mes-m2`, `$out/bin/mescc.scm`.
- Final host-visible output path in the pueue transcript, not a sandbox input.

Raw scan findings are expected / benign:

- Fixed source URLs in fetch blocks.
- Comments describing gcc-flavored Mes sources.
- Smoke harness command paths to local Rust/Cargo tooling; the smoke runs after the build and is not part of the derivation.

No undeclared host compiler, host libc, host `/usr` path, host shell, archive tool, network tool, or checkout path is referenced by the derivation body or build transcript.
