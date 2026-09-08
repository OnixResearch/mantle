# Runtime successor summary

## Cohort

Source commit `a7841aea055bf643cc473164b98a2442014f080b`. Binary `mantle-a7841aea-release` BLAKE3 `4819141a…`. Profile `source-built-fixed-point-sources-a7841aea.json`, semantic digest `d507f1dd…`, file digest `6b643b28…`. Dev plan `ddb30a9b…`. Host `leviathan`. See `runtime-cohort-a7841aea.ncl` and `successor-authority.md`.

## Leg 1: cold — complete

`dev-cold-a7841aea` ran from `2026-09-07T07:46:37-04:00` to `2026-09-08T01:12:21-04:00` (17h26m), exit 0.

- attempt status `complete`, blocker null
- disposition `cold-executed`; all six stages executed, none restored
- stage1 and stage2 binaries byte-identical: `d00466cc45777412cb070d08778471eba2cf58cff1440929a60f1ff6956ab0fe`; 800 units each; smoke exit 0
- `fixed_point: true`; strict eligibility `admitted`; cargo marker absent in both stages
- the Rust provider chain built mrustc → 1.90 → 1.91.1 (1:43:20) → 1.92.0 → 1.93.1 (1:42:34) → 1.94.0 with a candidate smoke
- dev cache published one entry: plan `ddb30a9b…`, cache key `13d9addc…`, stagex `e1039a3c…`, native `63d9bc23…`
- cold report review accepted (report-only scope)

## Leg 2: cached resume — complete

`dev-cached-a7841aea` ran from `01:22:40` to `01:51:56` (29m16s), exit 0.

- restored all six stages cross-run into a fresh staging directory; executed none
- disposition `resume-restored`; selected bundle `720a396f…` (mantle-stage2)
- dev-only discipline held: no promoted receipt, no release alias
- report review accepted for mode `resume`, completed stage `mantle-stage2`

## Leg 3: adopt — running

`dev-adopt-a7841aea` started `2026-09-08T01:59:16-04:00`, wrapper PID 3783718.

- gates: cached run exit 0 plus both report reviews accepted; cache entry present
- flags: `--dev-provider-cache` without `--dev-resume`, so the shell adopts cached providers instead of resuming stage bundles
- `native-provider.adopted.txt` transcript proves the cache adoption path executed
- the Rust provider chain rebuilds fresh: 1.91.1 and 1.92.0 stage-1 products ready; 1.93.1 in progress; then 1.94.0 final, stage1, stage2
- expected executed set: `full-source-rust-provider`, `mantle-stage1`, `mantle-stage2`

## Non-claims

Report reviews prove report shape and declared expectations only. The dev cycle does not satisfy a promoted fixed-point proof. V3 (promoted cold, empty authority, no dev state) and V4 gates remain open.
