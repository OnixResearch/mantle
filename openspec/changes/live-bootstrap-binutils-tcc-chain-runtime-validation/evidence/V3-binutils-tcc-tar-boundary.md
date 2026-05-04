# V3 binutils-tcc root narrowing and tar boundary

Task-ID: V3
Covers: `bootstrap.binutils.tcc.runtime-validation`

## Result

PARTIAL / BLOCKED: the direct `bootstrap/binutils-tcc.ncl` validation no longer stalls in the broad pre-build conversion path after this slice:

- `crates/crunch-glue` now memoizes completed recursive derivation conversions by the existing pre-order derivation identity, avoiding repeated conversion across diamond-shaped bootstrap dependency graphs.
- `bootstrap/binutils-tcc.ncl` no longer imports/passes `autoconf-2.69`, `automake-1.15.1`, or `libtool-2.2.4` as direct build inputs; the binutils release tarball has a generated `configure` script, and these unused direct inputs pulled in the large autoconf/automake epoch chain before binutils could reach a concrete build boundary.

With those changes, the focused root validation reached a concrete failed dependency instead of running indefinitely with empty logs:

```text
dependency tar-1.12-tcc.drv failed
```

The next dependency blocker for the binutils root is therefore `tar-1.12-tcc.drv`, whose direct validation reaches its own prerequisite boundary at `gzip-1.2.4-tcc.drv`.

## Commands

```sh
nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json \
  --store "$PWD/.crunch-drain/binutils-tcc-slim-store" \
  --state-dir "$PWD/.crunch-drain/binutils-tcc-slim-state" \
  bootstrap validate bootstrap/binutils-tcc.ncl \
  --evidence-dir "$PWD/.crunch-drain/binutils-tcc-slim-evidence" \
  --resume
```

Exit status: `1`

```sh
nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json \
  --store "$PWD/.crunch-drain/binutils-probe-tar-store" \
  --state-dir "$PWD/.crunch-drain/binutils-probe-tar-state" \
  bootstrap validate bootstrap/tar-tcc.ncl \
  --evidence-dir "$PWD/.crunch-drain/binutils-probe-tar-evidence" \
  --resume
```

Exit status: `1`; direct tar evidence reported `dependency gzip-1.2.4-tcc.drv failed`.

## Evidence files

- `evidence/V3-binutils-tcc-tar-boundary-doctor.json`
- `evidence/V3-binutils-tcc-tar-boundary-validation-summary.json`
- `evidence/V3-binutils-tcc-tar-boundary-validation-summary.md`
- `evidence/V3-binutils-tcc-tar-boundary-build.stdout.log`
- `evidence/V3-binutils-tcc-tar-boundary-build.stderr.log`
- `evidence/V3-binutils-tcc-tar-boundary-root-derivation.log`
- `evidence/V3-tar-tcc-gzip-boundary-validation-summary.json`
- `evidence/V3-tar-tcc-gzip-boundary-validation-summary.md`
- `evidence/V3-tar-tcc-gzip-boundary-build.stdout.log`
- `evidence/V3-tar-tcc-gzip-boundary-build.stderr.log`
- `evidence/V3-tar-tcc-gzip-boundary-root-derivation.log`

## Prior runaway symptom

Before this slice, the direct binutils root validation ran for ~78 minutes with empty build logs, a tiny store/state (`4.0K` / `36K`), one hot Tokio worker, and ~36 GiB RSS in the child build process. A narrower `bootstrap/autoconf-2.52.ncl` probe in the same area hit the debug guard:

```text
pending entries exceeded limit (16384); call drain_pending()
```

That supported treating the first blocker as pre-build recursive conversion/evaluation pressure rather than a binutils source compile failure.

## Next boundary

Continue V3 by draining the `tar-tcc` / `gzip-tcc` prerequisite chain before retrying final binutils assembler smoke.
