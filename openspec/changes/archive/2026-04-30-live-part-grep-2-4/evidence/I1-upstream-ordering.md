Task-ID: I1
Covers: bootstrap.part.grep.2.4

# grep 2.4 upstream ordering and output contract

Sources checked:

- `/home/brittonr/git/pi-repos/fosslinux--live-bootstrap/parts.rst`, section `grep 2.4`.
- `/home/brittonr/git/pi-repos/fosslinux--live-bootstrap/steps/grep-2.4/sources`.

Ordering:

1. Final pre-GCC `tcc-musl-v2` and the self-consistent musl are available.
2. `grep 2.4` is built as first post-musl text search tool.
3. Later musl v3 uses grep to regenerate headers that were unavailable in earlier musl passes.

Upstream notes:

- `parts.rst` names grep as required for rebuilding generated musl headers.
- There is no custom `pass*.sh` script in upstream `steps/grep-2.4/`; only the source manifest is present, so Crunch owns the explicit compile/link recipe.

Expected Crunch output contract:

- `bin/grep`
- `bin/egrep` symlink to `grep`
- `bin/fgrep` symlink to `grep`

Negative space:

- This part does not regenerate musl headers itself; it only supplies grep for that downstream musl stage.
