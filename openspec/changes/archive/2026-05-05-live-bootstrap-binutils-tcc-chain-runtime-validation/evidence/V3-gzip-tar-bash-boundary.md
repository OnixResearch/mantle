# V3 gzip/tar progress and bash boundary evidence

## Commands

All commands were run from `/home/brittonr/git/crunch/crunch` with `nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json` and repo-local writable `--store` directories under `.crunch-drain`. Validation used the default Crunch state dir (`~/.local/state/crunch`) with `--resume` so previously validated prerequisites could be reused.

## Results

- `bootstrap/gzip-tcc.ncl` passed focused validation. Evidence prefix: `V3-gzip-tcc-passed-*`.
- `bootstrap/tar-tcc.ncl` passed focused validation after gzip. Evidence prefix: `V3-tar-tcc-passed-*`.
- `bootstrap/binutils-tcc.ncl` advanced beyond the prior `tar-1.12-tcc -> gzip-1.2.4-tcc` blocker and now fails because `bash-2.05b-tcc.drv` fails. Evidence prefix: `V3-binutils-tcc-bash-boundary-*`.
- Direct `bootstrap/bash-2.05b-tcc.ncl` validation now bypasses the unused `oyacc` prerequisite and reaches a source/link boundary: full bash link fails, the minimal fallback has no compiled core input objects, and TinyCC reports `no input files`. Evidence prefix: `V3-bash-tcc-boundary-*`.

## Current blocker

The active V3 dependency chain is now narrowed to `binutils-2.30-tcc -> bash-2.05b-tcc`; the earlier `tar-1.12-tcc -> gzip-1.2.4-tcc` blocker is resolved by focused validation.
