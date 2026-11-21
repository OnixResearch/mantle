# Stage-2 addendum: corrected root 5hqkrl54 (2026-11-21)

The archived stage-2 evidence above covers the superseded 3fc1649f output; it is kept
unchanged as history. Cairn required the proof card to list the producer receipt and the
3fc1649f cards did not, so Terrane produced revision e8a961af with root drv
/nix/store/cvgjzlyd2nn1ihbq4z4s5ns6slirx7is-trellis-bounds-verus-job.drv and output
/nix/store/5hqkrl54nhfa44nmmaf80phvyi5kvjz1-trellis-bounds-verus-job.

Rerun with the same T2 executables (mantle 27fb1497..., bridge edbc2751...):
- preserve plan/realize exit 0; receipt status complete, strongest_state realized,
  receipt_blake3 86a98c78b2385f8d99c6e45e2c6413b819745a75c67be7ca2628abccf336f16b,
  plan_blake3 8875c84b..., import_receipt_blake3 f719f54d..., build_report_blake3 034077a3...;
  root /nix/store/5hqkrl54nhfa44nmmaf80phvyi5kvjz1-trellis-bounds-verus-job.
- NAR sha256 sha256-zUn6DMJIENyxoINDZMGptW0FLphtmE0ydx9mfM1ft1o= equals the Nix narHash; diff -r exit 0.
- handoff: receipt b3:e1186ca7..., association b3:9fcafc24..., producer job b3:387008d0...,
  tool graph b3:8bc04fed..., report b3:5c2d65ad..., proof_acceptance not-evaluated-by-mantle;
  PATH=/var/empty replay exit 0 with identical association and report.
- negatives: tampered NAR and different signer each fail store verify (exit 1).
- route non-claim unchanged: built by Nix, preserved through the cache-only route;
  the real build route remains unsupported at T2 (stage2-real-build-unsupported.md).

Full transcripts: /home/brittonr/wt/convergence-evidence/wp8/transcripts/

## Checked-build identity comparison (orchestrator, 2026-11-21T09:38:44Z)
Mantle main is now 4b05f9f677c8236b28aaaeca2af902063610b252 (F_final: the eight check-phase repairs on ead7b8dac). Its canonical clean-export `#default` build PASSED its check phase:
/nix/store/ahqnikfq18p6ycp4krx58cqgr5inm2sd-mantle-0.1.0 (transcript convergence-evidence/mantle-clean-4b05f9f677c8.log).
F_final bin/mantle b3 9c2225afe460f7d45854072ca05d8451507385070f36a66602edf542b51cc87f.
T2 (54cb49885, doCheck = false) bin/mantle b3 27fb14972a47976e7dcefc61a6357413d54f4446d6d42ae2fe9f117e366164dc.
They DIFFER. T2 also carries the verification-bridge commits and a different source tree, and builds are not shown to be bit-reproducible across trees, so the passing F_final check phase does NOT confirm T2's executable. The WP8 non-claim stands: T2's executables were built without the check phase.
