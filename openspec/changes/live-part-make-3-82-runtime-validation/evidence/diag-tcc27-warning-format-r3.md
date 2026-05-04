# Diagnostic: TinyCC 0.9.27 warning-format corruption reduced input (r3)

Date: 2026-05-03

## Command

```sh
nix shell nixpkgs#bubblewrap -c env \
  SNIX_BUILD_SANDBOX_SHELL=/nix/store/4mdqc2snfiihr6r61ln1rqs4fis6br9b-busybox-static-x86_64-unknown-linux-musl-1.36.1/bin/busybox \
  ~/.cargo-target/debug/crunch --json \
  --store .crunch-drain/diag-tcc27-warning-format-r1-store \
  --state-dir .crunch-drain/diag-tcc27-warning-format-r1-state \
  bootstrap validate bootstrap/diag-tcc27-warning-format.ncl \
  --warmup bootstrap/tinycc.ncl \
  --resume \
  --evidence-dir target/bootstrap-validation/diag-tcc27-warning-format-r3
```

## Result

- Validation status: `passed`.
- Warmup `bootstrap/tinycc.ncl`: passed from the reused validation state.
- Diagnostic output path: `/tmp/hermes-worktrees/crunch/make-warning-repro/.crunch-drain/diag-tcc27-warning-format-r1-store/c8vln8rljmgqh86pm17yd5j415b10skg-diag-tcc27-warning-format`.
- The reduced input compiled with exit `0` and reproduced the literal TinyCC/Mes diagnostics from the Make transcript.

Reduced input:

```c
      int main(void) {
        char *p;
        p = missing_warning_symbol(1, 2, 3);
        return p != 0;
      }
```

Captured `warn.stderr`:

```text
%s:%d: warning: implicit declaration of function '%s'
%s:%d: warning: implicit declaration of function '%s'
%s:%d: warning: assignment makes pointer from integer without a cast
%s:%d: warning: assignment makes pointer from integer without a cast
```

## Interpretation

The post-stdarg Make blocker is not specific to GNU Make source shape: a tiny
implicit-declaration/assignment warning is enough to make TinyCC 0.9.27 on the
current Mes runtime print literal diagnostic format placeholders (`%s:%d`, `%s`)
instead of file/line/symbol values.  The next repair should focus on TinyCC's
diagnostic formatting path or the Mes libc functions it depends on, then rerun
this diagnostic before rerunning `bootstrap/make-tcc.ncl`.

## Files

- `diag-tcc27-warning-format-r3-validation-summary.json`
- `diag-tcc27-warning-format-r3-validation-summary.md`
- `diag-tcc27-warning-format-r3-build.stdout.log`
- `diag-tcc27-warning-format-r3-build.stderr.log`
- `diag-tcc27-warning-format-r3-warmup-tinycc.stdout.log`
- `diag-tcc27-warning-format-r3-warmup-tinycc.stderr.log`
- `diag-tcc27-warning-format-r3-doctor.json`
