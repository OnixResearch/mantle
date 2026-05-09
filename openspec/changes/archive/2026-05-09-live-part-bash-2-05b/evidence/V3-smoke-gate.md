# V3 bash 2.05b smoke gate evidence

Task-ID: V3
Covers: bootstrap.part.bash.2.05b

The smoke contract for this part is conditional on a produced `bash-2.05b-tcc` output. Because V2 has no output path while prerequisites remain blocked or gated, no installed Bash smoke success is claimed.

The hardened derivation now fails closed unless `./bash --version`, `./bash -c 'echo bash-ok'`, installed `bin/bash`, and installed `bin/sh` all satisfy the local contract during a real build.
