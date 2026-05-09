# I3 source hardening evidence

Task-ID: I3
Covers: bootstrap.part.bash.2.05b

- Added source provenance and first-consumer comments beside the `bash_src` fixed-output fetch.
- Removed the object-mode chmod suppression so missing/unwritable object files fail closed.
- Replaced the hidden best-effort `--version || -c echo || true` smoke with required `--version` and `bash -c 'echo bash-ok'` checks.
- Added an installed `bin/sh` output-contract check alongside `bin/bash`.
