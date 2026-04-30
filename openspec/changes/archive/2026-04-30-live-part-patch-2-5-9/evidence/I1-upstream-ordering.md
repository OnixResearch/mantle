Task-ID: I1
Covers: bootstrap.part.patch.2.5.9

# patch 2.5.9 upstream ordering and output contract

Sources checked:

- `/home/brittonr/git/pi-repos/fosslinux--live-bootstrap/parts.rst`, `patch 2.5.9` section.
- `/home/brittonr/git/pi-repos/fosslinux--live-bootstrap/steps/patch-2.5.9/pass1.kaem`.
- `/home/brittonr/git/pi-repos/fosslinux--live-bootstrap/steps/patch-2.5.9/sources`.

Ordering:

1. `make 3.82` exists.
2. `patch 2.5.9` is built to support more complex source edits in later stages.
3. Later gzip/tar/sed/bzip2 stages can depend on it.

Expected Crunch output contract:

- `bin/patch` executable.
- `patch --version` succeeds.
