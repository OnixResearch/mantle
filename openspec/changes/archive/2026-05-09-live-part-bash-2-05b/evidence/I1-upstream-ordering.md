# I1 upstream ordering and output contract evidence

Task-ID: I1
Covers: bootstrap.part.bash.2.05b

- Upstream part: fosslinux/live-bootstrap `bash 2.05b`.
- Crunch derivation: `bootstrap/bash-2.05b-tcc.ncl`.
- Direct predecessor imports observed in derivation: `tinycc-0.9.27`, `stage0-posix`.
- Output contract after hardening: `bin/bash` must be executable, `bin/sh` must resolve to the installed shell, and an in-build `bash -c 'echo bash-ok'` smoke must pass before install.
