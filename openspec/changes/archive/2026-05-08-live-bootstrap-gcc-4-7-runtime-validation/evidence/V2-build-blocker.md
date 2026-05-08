# V2 gcc-4.7 build status

Task-ID: V2
Covers: bootstrap.gcc47.runtime-validation
Captured: 2026-05-08T21:09:49Z

## Build gate

`bootstrap/gcc-4.7.ncl` was inspected instead of launching a long build against a known-missing prerequisite. The derivation requires:

- `GCC4=$(find_input gcc-4.0.4)`
- `CC="$GCC4/bin/gcc"`
- `CXX="$GCC4/bin/g++"`
- `PATH="$GCC4/bin:..."`

The prerequisite gcc-4.0 runtime-validation archive records that no gcc-4.0.4 output exists at the current TinyCC/Mes `libtcc.c` boundary. Launching gcc-4.7 from this state would only re-materialize that predecessor blocker rather than produce gcc-4.7 transition evidence.

Required transcript field status:

| Field | Status |
|---|---|
| command | deferred: `bootstrap/gcc-4.7.ncl` build is gated on gcc-4.0 output |
| provider | blocked: no validated `gcc-4.0.4` provider |
| exit status | not run; prerequisite gate failed before target build |
| output path | none |
| failure class | prerequisite compiler absent / gcc-4.0 negative runtime boundary |
| fallback status | no gcc-4.7 fallback accepted |
| placeholder rejection | preserved: do not substitute host GCC/Nix/system compiler for `$GCC4` |

No gcc-4.7 build success is claimed in this evidence.
