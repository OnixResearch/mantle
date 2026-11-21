# WP8 E real-build route: stopped diagnostics

Date: 2026-11-21. Executables: clean export of HEAD `54cb49885` built as the flake's
`crunch.overrideAttrs { doCheck = false; }` (`/nix/store/6fq4isdp8jhsswq0i9s4bfvrrrji2qs3-mantle-0.1.0`).
Producer export: revision `a143be37c7e56f2d3d66ce9d947f025585d66064`, root
`/nix/store/bgvhgd03ziqwani101savwl9iblrl24q-trellis-bounds-verus-job.drv`.

## What succeeded

- `foreign-import produce-nix`: accepted; 7,524 derivations became
  `mantle-graph/nixpkgs.graph.json` + `nixpkgs.index.json` (`transcripts/wp8-produce-nix.txt`).
- `foreign-import plan` with the recompute-blake3 translation policy: accepted; target prefix
  `/mantle/store`; 630 source requirements (`transcripts/wp8-plan.txt`, `plan.json`,
  `import-receipt.json`).
- `foreign-import prepare-sources` over the export's 630 source payloads: accepted;
  `manifest_blake3 3fe8262c3220924d58b088488f33b13e70a486a076832c499305318f2b246c79`
  (`transcripts/wp8-prepare-sources.txt`).

## Route blockers observed

1. `--cache-closure-policy` is rejected for a real-build plan:
   `cache closure policy is only valid for a cache-only foreign plan`. The refreshed,
   signed, verified 7,373-path cache policy therefore cannot be applied to this route.
2. `--remote-binding-config` rejects non-canonical source URLs (`source-invalid`), then
   admits only with canonical trailing-slash forms and a binding `store_prefix` equal to
   `--store-prefix` (`/mantle/store`). Both fixes are applied and admission passes.
3. With admission passing, Mantle opens the store and schedules the full closure. The
   loopback cache received only `/action-results/v1/indexes/<digest>.json` requests (all
   404) and **no `.narinfo` or `/nar/` requests** (`transcripts/wp8-cache-server.log`).
   Mantle's substitution protocol asks for Mantle action results keyed by the recomputed
   target paths; the Nix binary cache cannot answer them.
4. Mantle then attempts to build the fixed-output bootstrap fetch itself. Exact receipt
   diagnostics:
   - attempt 1 (`realization-receipt-attempt1.json`):
     `origin /mantle/store/c82d14...-stage0-posix-1.9.1-source.drv: store error: build: bubblewrap executable 'bwrap' not found`
   - attempt 2 with `SNIX_BUILD_BWRAP` set (`realization-receipt.json`):
     `origin /mantle/store/c82d14...-stage0-posix-1.9.1-source.drv: store error: build: nonzero exit code: exit status: 1`
     followed by the fetchurl failure:
     `Neither your store nor your substituters seems to have: /1rz4g4znpzjwh1xymhjpm42vipw92pr73vdgl6xs1hycac8kf2n9`
     (the minimal bootstrap source is not in the cache; the FOD fetch ran without the
     needed source/network).
   `completed_unit_count: 0` in both receipts.

## Conclusion

The route as specified ("dependencies hydrate from the cache, Mantle's worker builds the
root") does not hold for this plan: a recompute-blake3 plan addresses dependencies by
recomputed `/mantle/store` paths, while the loopback cache is a Nix binary cache keyed by
`/nix/store` paths, and Mantle's real-build scheduler consults action-result indexes and
target-path narinfo only. No narinfo lookup is attempted, so the 46 GiB input cache is
never read. The first dependency is a fixed-output bootstrap fetch that cannot be built
offline.

Escalated to the orchestrator per E step 3; no preserve fallback was attempted.
