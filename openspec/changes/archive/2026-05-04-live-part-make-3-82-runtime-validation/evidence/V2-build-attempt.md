# V2 build attempt: bootstrap/make-tcc.ncl

Task-ID: V2
Covers: bootstrap.part.make.3.82.runtime-validation

## Command

```bash
nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json \
  --store "$PWD/.crunch-drain/make-store" \
  --state-dir "$PWD/.crunch-drain/make-state" \
  bootstrap validate bootstrap/make-tcc.ncl \
  --evidence-dir openspec/changes/live-part-make-3-82-runtime-validation/evidence \
  --resume
```

## Result

Failure class: `incomplete-hung-validation-run`.

The validation runner wrote `doctor.json` and passed the build-profile preflight. It spawned the child command:

```text
/home/brittonr/git/crunch/crunch/target/debug/crunch --json \
  --store /home/brittonr/git/crunch/crunch/.crunch-drain/make-store \
  --store-prefix /crunch/store \
  --state-dir /home/brittonr/git/crunch/crunch/.crunch-drain/make-state \
  build bootstrap/make-tcc.ncl --no-substitute \
  --import-path /home/brittonr/git/crunch/crunch/lib
```

The child remained running for about 11 minutes with no `validation-summary.json`, `build.stdout.log`, or `build.stderr.log` emitted by the parent runner. State growth reached about 51 MiB, while the export store remained empty.

After killing the stuck validation process, `crunch store list` against the same state/store failed with:

```text
error: opening PathInfo database /home/brittonr/git/crunch/crunch/.crunch-drain/make-state/pathinfo.redb: redb database error: Database repair aborted.
```

No make output path was produced, and no runtime smoke can be claimed from this attempt.

## Evidence files

- `doctor.json`: preflight passed.
- This file records the failed/hung build attempt and post-kill state inspection.
