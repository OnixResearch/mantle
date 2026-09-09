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

## Leg 3: adopt — complete

`dev-adopt-a7841aea` ran from `2026-09-08T01:59:16-04:00` to `15:47:18-04:00` (13h48m), exit 0.

- gates: cached run exit 0 plus both prior report reviews accepted; cache entry present
- flags: `--dev-provider-cache` without `--dev-resume`, so the shell adopted cached providers instead of resuming stage bundles
- `native-provider.adopted.txt` transcript proves the cache adoption path executed; the Rust provider chain rebuilt fresh (1.91.1, 1.92.0, 1.93.1, 1.94.0 final) and the 17-member closure materialized with adoption provenance
- disposition `provider-cache-adopted`; executed exactly `full-source-rust-provider`, `mantle-stage1`, `mantle-stage2`
- `fixed_point: true`; stage1 and stage2 binaries byte-identical `0d2b6ad6b0a14f5e2767a32998979ffbd2cac2551050d0767f00889f01a511cb`; 800 units each; smoke exit 0
- dev-only discipline held: no promoted receipt, no release alias
- report review accepted for mode `adopt`

## V2 verdict

The cold-to-cached-to-adopt dev cycle is complete with exact source, plan, provider, store, stage, report, and alias evidence recorded under this directory. All three dispositions (`cold-executed`, `resume-restored`, `provider-cache-adopted`) were observed with accepted report reviews and unchanged release aliases.

## Leg 4: promoted cold — complete

`promoted-cold-a7841aea` ran `2026-09-08T17:16:57-04:00` to `2026-09-09T10:30:56-04:00` (17h14m), exit 0. The remote process survived loss of the local Pueue connection. No second proof was launched.

The launch used source cohort `a7841aea`, without dev-cache, resume, fast-fail, or checkpoint options. The promoted plan digest equals the dev plan digest. Explicit options, not that shared digest, select cache authority. See `promoted-authority-review.md` and `promoted-launch-a7841aea/`.

Terminal evidence: attempt `complete`, no blocker. `fixed_point: true`. Stage1 and stage2 binaries byte-identical `829d6bbbcf901d50b217e9bad8efd5bcf4f63a50c09b84d9e58236709e75a3b4`; 800 units each, zero failed units, smoke exit 0, cargo marker absent. Promoted receipt `deterministic-build-proof.json` reports `verdict: self-rebuild-match` and strict proof admission `admitted`. The receipt shows clean-namespace-per-run store isolation, no substitution, and no authority violations. Final proof-bundle digest `1178303b…` matches the collected `proof-bundle-blake3.txt`.

No cache-adoption disposition: `native-provider.adopted.txt` and `dev-resume-report.json` are absent. Both aliases `latest` and `latest-source-built-fixed-point` resolve to `promoted-cold-a7841aea`. Staging was published into the final output directory and is gone.

See `promoted-terminal-a7841aea/` for the collected records, receipt digests, and observations.

## V4 verdict

Local verification passed for Rust source `e5cf7fdb`. Focused tests, formatting, first-party Clippy, pinned Cairn validation, Tracey, all three gates, and five relevant Nix checks passed. See `../local-verification-2026-09-08/summary.md` for exact scopes and captured results.

## Overall verdict

Legs 1 through 4 are complete. The dev cycle (cold, cached resume, adopt) and the promoted cold proof all recorded exact source, plan, provider, store, stage, receipt, and alias evidence. Both the dev cycle and the promoted proof reached a byte-identical stage1==stage2 fixed point with strict eligibility admitted. The promoted run emitted no cache-adoption disposition and published both success aliases.

## Non-claims

Report reviews prove report shape and declared expectations only. The promoted proof shows self-rebuild match under its recorded authorities. It does not prove compiler correctness, seed correctness, deployment success, independent rebuild agreement, or bit-for-bit release reproducibility beyond this host and run.

The runtime cohort remains `a7841aea`. Local formatting commit `e5cf7fdb` does not change that recorded authority. Runtime results do not prove the newer source bytes.
