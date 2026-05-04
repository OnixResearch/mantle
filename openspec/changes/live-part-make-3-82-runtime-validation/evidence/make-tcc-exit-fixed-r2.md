# GNU Make 3.82 exits cleanly after version smoke

Task-ID: V3
Covers: bootstrap.part.make.3.82.runtime-validation

## Result

`bootstrap/make-tcc.ncl` now builds and installs an output after `./make --version` exits 0. The previous post-version exit-60 failure was traced to TinyCC's rebuilt Mes `_exit` inline assembly clobbering the requested exit code with syscall number 60.

Validation command:

```sh
crunch --json --store .crunch-drain/diag-tcc27-exit-semantics-fixed-r1-store \
  --state-dir .crunch-drain/diag-tcc27-exit-semantics-fixed-r1-state \
  bootstrap validate bootstrap/make-tcc.ncl \
  --warmup bootstrap/tinycc.ncl --resume \
  --evidence-dir target/bootstrap-validation/make-tcc-exit-fixed-r2
```

Output path: `/tmp/hermes-worktrees/crunch/make-exit60-diagnostic/.crunch-drain/diag-tcc27-exit-semantics-fixed-r1-store/24na3y8ni0rq2l7gaf3syrq5nq4nj6ns-make-3.82-tcc`

Boundary: V3 remains incomplete because adding a simple Makefile recipe smoke still segfaulted after command dispatch; that is the next blocker. The version-only runtime boundary is now proven.

Evidence files: `make-tcc-exit-fixed-r2-*.log`, `make-tcc-exit-fixed-r2-validation-summary.json`, and `make-tcc-exit-fixed-r2.derivation.log`.
