# make-tcc POSIX/wait recipe smoke repair r1

Task-ID: V3
Covers: bootstrap.part.make.3.82.runtime-validation

## Command

```sh
export PATH=/nix/store/dk9qhjgg469lv6mriys7v4c59igarmvx-bubblewrap-0.11.1/bin:$PATH
/home/brittonr/.cargo-target/debug/crunch --json \
  --store $PWD/.crunch-drain/make-tcc-posix-wait-smoke-r1-store \
  --state-dir $PWD/.crunch-drain/make-tcc-posix-wait-smoke-r1-state \
  bootstrap validate bootstrap/make-tcc.ncl \
  --warmup bootstrap/tinycc.ncl \
  --resume \
  --evidence-dir $PWD/target/bootstrap-validation/make-tcc-posix-wait-smoke-r1
```

## Result

- Status: `passed`
- Doctor OK: `True`
- Warmup: `bootstrap/tinycc.ncl` exit `0`
- Build exit code: `0`
- Built derivation: `/crunch/store/h32bq1xh6qg5khx0g1w3k19wzfzchdb4-make-3.82-tcc.drv`
- Output path: `/home/brittonr/git/crunch/crunch/.crunch-drain/make-tcc-posix-wait-smoke-r1-store/mh5rl7i5sac8z0frdravpjf0z2x0qffk-make-3.82-tcc`
- Logical output path: `/crunch/store/mh5rl7i5sac8z0frdravpjf0z2x0qffk-make-3.82-tcc`
- Host leakage scan: no coarse host-path needles in captured build output.
- Substitute/fallback status: validation used the explicit local `.crunch-drain/make-tcc-posix-wait-smoke-r1-store` and `.crunch-drain/make-tcc-posix-wait-smoke-r1-state` with warmup `bootstrap/tinycc.ncl`; no provider fallback, placeholder output, or substitute path is reported in the captured validation summary.

## Diagnostic conclusion

The previous `make-tcc-exit-fixed-r2` state proved `make --version` exited 0,
but a simple Makefile recipe segfaulted after dispatching `/bin/busybox echo`.
The repair compiles GNU Make with configure-equivalent POSIX/wait/fcntl feature
macros (`HAVE_SYS_WAIT_H`, `HAVE_WAITPID`, `POSIX`, `HAVE_FCNTL_H`) in addition
to the earlier standard header macros, so x86_64 Mes libc prototypes and wait
status handling are selected instead of old fallback declarations.

The in-derivation smoke now runs:

```make
all:
	/bin/busybox echo smoke-ok > smoke/out
```

and then asserts `grep smoke-ok smoke/out` before installation.

## Key derivation log excerpt

```text
# crunch build log
# derivation: make-3.82-tcc
# drv_path: h32bq1xh6qg5khx0g1w3k19wzfzchdb4-make-3.82-tcc.drv
# status: success
# timestamp: 1777864637

GNU Make 3.82
Built for unknown
Copyright (C) 2010  Free Software Foundation, Inc.
License GPLv3+: GNU GPL version 3 or later <http://gnu.org/licenses/gpl.html>
This is free software: you are free to change and redistribute it.
There is NO WARRANTY, to the extent permitted by law.
/bin/busybox echo smoke-ok > smoke/out
smoke-ok

```

Machine-readable summary: `make-tcc-posix-wait-smoke-r1.validation-summary.json`.
Full derivation log: `make-tcc-posix-wait-smoke-r1.derivation.log`.
