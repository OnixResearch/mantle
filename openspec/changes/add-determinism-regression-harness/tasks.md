# Tasks: Add determinism regression harness

## Phase 1: Harness shape

- [ ] Build a harness that reruns selected builds under varied ambient host state
- [ ] Vary at least `HOME`, `PATH`, `USER`, `TZ`, `LANG`, `TMPDIR`, cwd, and umask
- [ ] Compare output digests and hermeticity audit facts across runs

## Phase 2: Coverage

- [ ] Cover at least one normal derivation
- [ ] Cover at least one fetcher-rooted build
- [ ] Cover at least one self-build-friendly path

## Phase 3: Validation

- [ ] Re-read the touched build-pipeline spec against the final harness behavior before implementation starts
- [ ] Run `openspec validate add-determinism-regression-harness`
