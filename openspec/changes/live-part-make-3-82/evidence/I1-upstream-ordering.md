Task-ID: I1
Covers: bootstrap.part.make.3.82

# make 3.82 upstream ordering and output contract

Sources checked:

- `/home/brittonr/git/pi-repos/fosslinux--live-bootstrap/parts.rst`, first `make 3.82` section.
- `/home/brittonr/git/pi-repos/fosslinux--live-bootstrap/steps/make-3.82/pass1.kaem`.
- `/home/brittonr/git/pi-repos/fosslinux--live-bootstrap/steps/make-3.82/sources`.

Ordering:

1. `tinycc 0.9.27` and earlier Mes/stage0 tools exist.
2. `make 3.82` is built to replace hand-written kaem scripts with Makefiles.
3. Later parts use make as the bootstrap build system.

Expected Crunch output contract:

- `bin/make` executable.
- `make --version` works and reports GNU Make 3.82.

Negative space:

- This is the early tcc-built make, not the later GCC-rebuilt make part mentioned in the second `make 3.82` section of `parts.rst`.
