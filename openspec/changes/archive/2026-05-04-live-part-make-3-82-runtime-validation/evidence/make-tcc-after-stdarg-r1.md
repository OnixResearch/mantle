# Make 3.82 rerun after production TinyCC stdarg repair (2026-05-03)

Task-ID: V2/V3 follow-up evidence for `bootstrap.part.make.3.82.runtime-validation`

## Command

```sh
nix shell nixpkgs#bubblewrap -c env \
  SNIX_BUILD_SANDBOX_SHELL=/nix/store/4mdqc2snfiihr6r61ln1rqs4fis6br9b-busybox-static-x86_64-unknown-linux-musl-1.36.1/bin/busybox \
  ~/.cargo-target/debug/crunch --json \
  --store "$PWD/.crunch-drain/make-tcc-after-stdarg-r1-store" \
  --state-dir "$PWD/.crunch-drain/make-tcc-after-stdarg-r1-state" \
  bootstrap validate bootstrap/make-tcc.ncl \
  --warmup bootstrap/tinycc.ncl \
  --resume \
  --evidence-dir "$PWD/target/bootstrap-validation/make-tcc-after-stdarg-r1"
```

## Result

Exit status: `1` (`build-failed`).

The prerequisite warmup `bootstrap/tinycc.ncl` passed (`exit_code: 0`) using commit
`baa461376eac2efe326d0ba4837296c6cb7db934`, which contains the production
TinyCC x86_64 stdarg/Mes runtime repair. The follow-up `bootstrap/make-tcc.ncl`
validation still failed in the `make-3.82-tcc` builder with exit status `60`.

Important change from prior Make evidence: the build now reaches a GNU Make 3.82
binary far enough to print `GNU Make 3.82` / `Built for unknown` before failing,
but the builder transcript is dominated by TinyCC/Mes corrupted literal warning
format strings such as `%s:%d: warning: implicit declaration of function '%s'`
and `%s:%d: warning: assignment makes pointer from integer without a cast`.

## Evidence artifacts

- `make-tcc-after-stdarg-r1-validation-summary.json`
- `make-tcc-after-stdarg-r1-validation-summary.md`
- `make-tcc-after-stdarg-r1-build.stdout.log`
- `make-tcc-after-stdarg-r1-build.stderr.log`
- `make-tcc-after-stdarg-r1-warmup-tinycc.stdout.log`
- `make-tcc-after-stdarg-r1-warmup-tinycc.stderr.log`
- `make-tcc-after-stdarg-r1-doctor.json`
- `make-tcc-after-stdarg-r1.derivation.log`

## Interpretation

The TinyCC stdarg repair unblocked the focused static-runtime diagnostic and the
Make build advances beyond the earlier immediate startup segfault class, but V3
remains blocked: no accepted GNU Make output path exists yet, and version/simple
Makefile smoke tests cannot be marked complete until the `make-3.82-tcc` builder
exits successfully and the produced binary runs both smokes.

Next high-ROI repair target: reduce the remaining TinyCC/Mes diagnostic/varargs
format-string corruption against the Make compile path, ideally with a smaller
Make-source or TinyCC warning-format reproduction before layering more GNU Make
source edits.
