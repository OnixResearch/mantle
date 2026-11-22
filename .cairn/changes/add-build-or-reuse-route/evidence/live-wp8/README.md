# WP8 Terrane job on Mantle route `cache-inputs-build-roots-v2` (built, then reused)

Repeat of Mantle's E2 qualification (`add-cache-inputs-build-roots-route`, E2 on `5263dabfc`) of the WP8
`trellis-bounds-verus-job` (bounds.clamp, Verus) on the landed build-or-reuse route `cache-inputs-build-roots-v2`.

| Field | Value |
|---|---|
| Mantle (checked, `6594e7fd1`) | `/nix/store/5z292f6b7rgb3wr2a9sczkrzaz8x1bii-mantle-0.1.0/bin/mantle`, BLAKE3 `878bdea3cf7945cd6c39e623233810c563fc769ad335c0fbca6cf15f4b83ea76` |
| Bridge (same revision) | `/datapool/mantle-store/build-or-reuse-20261122/bridge-target-6594e7fd1/release/mantle-verification-bridge`, BLAKE3 `beebf997964319975f00405e6533e4ea8af70958f9ac80bc8546c65c451b52a5` |
| Producer export | `/home/brittonr/wt/convergence-evidence/wp8/producer-export` (Terrane `e8a961af3dbeb6f7a0f7dfbf82100bfe2abcb482`; graph inputs `wp8/mantle-graph-2/`) |
| Root node / target derivation | `nix:cvgjzlyd2nn1ihbq4z4s5ns6slirx7is-trellis-bounds-verus-job.drv` → `/nix/store/clnjhb7wy6fgrakdrjdz23ki8ahzap8c-trellis-bounds-verus-job.drv` |
| Root output | `/nix/store/5hqkrl54nhfa44nmmaf80phvyi5kvjz1-trellis-bounds-verus-job` (NAR `sha256-zUn6DMJIENyxoINDZMGptW0FLphtmE0ydx9mfM1ft1o=`, 9,867,968 B) |
| Filtered input cache | `/datapool/mantle-store/wp8-20261121/cache-build-roots` (7,375 narinfo, root absent), served on `127.0.0.1:18573`; trusted key `wp8-20261121-1:2Iu1BO2fKZfbeBLGI0Uy4NzD7UzvKQPDWCXnCz9vYbc=` |
| Cache-closure policy | `cache-closure-policy.json` (file BLAKE3 `9ec692ba…`, canonical policy digest `fa213f71c53eba0bcbc151d17f26e65d1f8babc9899284856f5c2d013551f262`, equal to E2's) |
| Endpoint binding | `binding/remote-endpoint-binding.json` (mode 0600); endpoint table copied from `/datapool/mantle-store/wp8-20261121/endpoint-state` into `binding/endpoint-state` |
| Builder key | local key auto-generated in the fresh state dir, name `crunch-britton-desktop-1`, public key `FlA62qnSo8C7U6HsFdOrOGUZU0mVVvkR37lzfJYazfg=`, fingerprint `3da127ef3efcf4a6a90314496976fc3a4d3972ad7e0f8f67e819e593495e2ccd` |
| Fresh run root | `/datapool/mantle-store/build-or-reuse-20261122/wp8-v2/` (`plan-state/`, `state/`, `store/`; did not exist before 11:18) |
| Transactions | `run-transcript.sh` (copy of `realization/run-transcript.sh`, output dir changed); cache-backed commands via `evidence-cmd.sh`, which starts the loopback cache, records the command, and stops the server (`logs/`) |

## Steps

| # | Step | Command (transcript) | Exit | Wall | Compared files / result |
|---|---|---|---|---|---|
| 1 | plan (route v2) | `mantle --json foreign-import plan --graph wp8/mantle-graph-2/nixpkgs.graph.json --package-index wp8/mantle-graph-2/nixpkgs.index.json --policy mantle-src-6594e7fd1/config/foreign-cache-closure/generated/preserve-nix.json --execution-profile …/nix.json --package trellis-bounds-verus-job --system x86_64-linux --nix-compat --realization-route cache-inputs-build-roots-v2 --plan-out graph/plan.json --receipt-out graph/import-receipt.json --state-dir …/wp8-v2/plan-state` (`01-plan`) | 0 | 75.39 s | `plan_identity 50bb88031d9fe0910a9439a1c35bc028df7c34d9e683ba0a7fc6c9aefd44a069`, route `cache-inputs-build-roots-v2`; `graph/plan.json` 73,715,736 B BLAKE3 `e8c79ed6…e49d2d`; `graph/import-receipt.json` 1,639,781 B `ce226ac8…84d35`. Structural diff against E2's `wp8/build-roots/plan.json` (`compare/plan-diff-vs-e2.txt`): exactly 3 differences — `realization_route`, `plan_identity.value`, and one added `non_claims` entry. Root/target/output identical. |
| 2 | prepare-sources | `mantle --json foreign-import prepare-sources --plan graph/plan.json --out graph/source-bundle.json --state-dir …/plan-state` (`02-prepare-sources`) | 0 | 29.50 s | `graph/source-bundle.json` 312 B BLAKE3 `b8abb366…7ab8030`, manifest `8ed1103b5de3054ee13ea391af805e276e3cc3b6ceaa50b2149193adb1ff1777` (= E2), roots/records empty |
| 3 | realize run 1 | `env PATH=/var/empty SNIX_BUILD_BWRAP=…/bwrap mantle --json --state-dir …/wp8-v2/state --store …/wp8-v2/store --store-prefix /nix/store --remote-binding-config binding/…json foreign-import realize --plan … --import-receipt … --source-bundle … --source-bundle-blake3 8ed1103b… --execution-profile …/nix.json --cache-closure-policy cache-closure-policy.json --receipt-out receipts/realization-receipt-run1.json --substitute` (`03-realize-run1`) | 0 | 1526.28 s | Receipt v2, `status complete`, `strongest_state realized`, `receipt_blake3 14df9a9ae3f791847eb342c78f360051133f026c3362daedb8781b41f3f9d137` (4,117,555 B), root `disposition: built`, signer role `builder`, `built.input_closure_blake3 ba46016f…`, `trust_policy_blake3 e8dc58ff…`; 7,524 units (1 `built`, 264 `substituted`, 12 `already-present`, 7,247 `not-required-cache-only`); index `state/foreign-realization/built-roots/clnjhb7w….drv.json` 4,117,555 B, byte-identical to the receipt. Server: 325 NAR + 326 narinfo requests, **0** `/action-results`, root narinfo probe `GET /5hqkrl54….narinfo` → **404**. |
| 4 | NAR + tree compare | `nix-store --dump <Mantle-built root> > compare/mantle-built-root.nar; sha256sum; cmp…; diff -r /nix/store/5hqkrl54… <Mantle-built root>` | 0 | — | `compare/mantle-built-root.nar` 9,867,968 B sha256 `cd49fa0cc24810dcb1a0834364c1a9b56d052e986d984d32771f667ccd5fb75a` (= `sha256-zUn6DMJI…`, = Nix-built NAR, BLAKE3 `6a1c6095…76e7fe50`); `cmp` identical; `diff -r` exit 0, `compare/diff.txt` 0 B |
| 5 | store info + verify | `mantle --json --state-dir …/state --store …/store --store-prefix /nix/store store info 5hqkrl54…` (`05-store-info`); `… store verify 5hqkrl54… --trusted-public-keys crunch-britton-desktop-1:FlA…=` (`06-store-verify`) | 0 / 0 | 0.50 s / 0.32 s | `store info`: `nar_sha256 cd49fa0cc2…`, `nar_size 9867968`, deriver `clnjhb7w…`, references `[5hqkrl54…, p72j1v14…]`, signature `crunch-britton-desktop-1:…`; `store verify`: `OK 5hqkrl54… trusted_signatures=1/1`, `1 checked, 0 mismatches` |
| 6 | bridge `trellis-job` + replay | `bridge trellis-job <Mantle-built root> wp8/producer-export e8a961af… handoff/built` (`08-bridge-trellis-job`); `env PATH=/var/empty bridge trellis-job … handoff/built-replay` (`09-bridge-replay`) | 0 / 0 | 0.81 s / 1.89 s | handoff 24 MB; receipt `b3:e1186ca7…`, association `b3:9fcafc24…`, producer job `b3:387008d0…`, tool graph `b3:8bc04fed…`, external report `b3:5c2d65ad…`, `proof_acceptance: not-evaluated-by-mantle` — every identity equals E2's; `mantle-observation.json` byte-identical between run and replay |
| 7 | realize run 2 (reuse) | same as step 3 with `--receipt-out receipts/realization-receipt-run2.json` (`04-realize-run2`) | 0 | 857.16 s | Receipt v2 `complete`/`realized`, `receipt_blake3 27bfd5e01884efa995dcd9d5fdec81886e2621b25cab19256daa2350219d7704`, root `disposition: reused`, `reused {original_receipt_blake3: 14df9a9a…, source: "local", builds: 0, downloads: 0}` = R1's `receipt_blake3`; `built: null`; same signer/NAR. Run-2 server log contains **only the `Serving HTTP …` header** — zero HTTP requests |
| 8 | negative: fresh signing key | step 3 plus `--signing-key negative/wp8-v2-negative-1.sec` and `--receipt-out receipts/negative-signer-mismatch.json` (`10-negative-signer-mismatch`) | 3 | 70.31 s | `refusing to reuse present foreign root 5hqkrl54…: present root cannot be reused: signer-mismatch`; no receipt written; no store mutation |
| 9 | negative: retired key trusted | step 8 plus `--trusted-builder-public-key crunch-britton-desktop-1:FlA62qnSo8C7U6HsFdOrOGUZU0mVVvkR37lzfJYazfg=` (`11-negative-retired-key-trusted`) | 0 | 573.14 s | Receipt v2 `complete`/`realized`, `receipt_blake3 9ef6cce692a7805fbbfa58963da9bfff11f11c87adc5153808c40798012bde33`, root `reused` with `original_receipt_blake3 14df9a9a…`, `source: "local"`, `builds: 0`, `downloads: 0`, `built: null`; the root entry keeps the original `signer {crunch-britton-desktop-1, 3da127ef…, builder}` (reuse mints no signature); server log header only; 7,524 units = 264 `substituted` + 7,260 `already-present` |

Records (size, BLAKE3):

- `transcripts/01-plan.txt` 77,124,888 B `a21fd27325b8351a60a4fa9f5b908bf4454303ef80553a89d686aafabaf2b4f0`
- `transcripts/02-prepare-sources.txt` 715 B `2e1eca3072a7b4e920baba9852ded49d70b342c89f385856028772e4626f1e1d`
- `transcripts/03-realize-run1.txt` 3,434,417 B `52f87a12b77816b65f408ff0d336b008470b969efb7adc00570106dd77fa2237`
- `transcripts/04-realize-run2.txt` 3,242,148 B `e75de1a119198ead6cc5d6936bdcd223b9b613f83b75c347d02e338a92f11c31`
- `transcripts/05-store-info.txt` 2,360 B `2f96bb1c0eea84fd2d2188248e65d434f1a9a380f34b26f88c033bbd70368c9f`
- `transcripts/06-store-verify.txt` 704 B `7d1a9807be856cea569215fb9aa5eae81ffffa553a86d63399fe919f7e510bd4`
- `transcripts/07-negative-key.txt` 478 B `551e8eea500b014d23bfcf4dc15aa739c73c91c10cf32c6eb5fed89286f20046`
- `transcripts/08-bridge-trellis-job.txt` 2,226 B `601ab0994e2967a6576824c3211d470d5a8e8f9c0caf9846e7a025344c7ae2e1`
- `transcripts/09-bridge-replay.txt` 2,257 B `e8724279ce1601856b954f8d4c4e992f66333e4a8c3ff63f5cdcd84a8bdd360d`
- `transcripts/10-negative-signer-mismatch.txt` 1,754 B `5fbe3269d4dbdf363f5538bd86c9da46332ee7c9ab9014af30864c36742f4826`
- `transcripts/11-negative-retired-key-trusted.txt` 3,242,373 B `95e77b5891ec2e4bc66b61803c97d418e10b178858695c655d60478b92e2860c`
- `receipts/negative-retired-key-trusted.json` 3,890,808 B `61c41b884f1c8e7da67f3ffbd1ec078b23c4a56b32c11d58713a393f5ada0731`
- `handoff/built/` 24,628,537 B; `mantle-observation.json` 1,678 B `74d931c79c1f65bc5ab106235a2c563b2db5e316010e4162ba96de31b939d40e`; `handoff/built` and `handoff/built-replay` are tree-identical (`diff -r` exit 0)
- `receipts/realization-receipt-run1.json` 4,117,555 B `810200235386b0ab4536a01738864b8b9270b0c47024248f47b27c9aa55654e1`
- `receipts/realization-receipt-run2.json` 3,890,808 B `54060102c5bca5c63ba81ccea8d358d6b212a6d4c6af681789572dbc4429690f`
- `compare/mantle-built-root.nar` = `compare/nix-built-root.nar` 9,867,968 B `6a1c6095606e9aec2854f127e7e070bf6b2afd4aab9df82fe74313cb76e7fe50`
- `compare/diff.txt` 0 B; `compare/plan-diff-vs-e2.txt` 310 B `feadeb0800bf75a5b04852c54da1b8bc2d76e22080f87d0a5ddc2a86c7381159`
- `logs/server-03-realize-run1.log` 73,185 B `c10b7e820bac978d586a34f9e453ba238b76f36df60ddba53c0611104d9bdd40`
- `logs/server-04-realize-run2.log` 67 B `45239b2f6047230eaedb3f3f9befe2beb93eb3a9258e3c4640e3a3f064599794`
- `logs/server-10-negative-signer-mismatch.log` 67 B `45239b2f…`; `logs/server-11-negative-retired-key-trusted.log` 67 B `45239b2f…`
- `binding/remote-endpoint-binding.json` 1,216 B `205d93d0e98d155d2e9f04363c5278ce9ca1c33e3b36223b7152e04f9fc6af74`; `run-transcript.sh` 694 B `250f809f…`; `evidence-cmd.sh` 588 B `066fcf5d…`
- `builder-public-key.txt` 69 B `f4c5233fa70223136237422091ad145a054c9d29edaf66fabfa19c4027f0fb43`; `negative/wp8-v2-negative-1.pub` 62 B `a53fbdd51173eaf3c1bb72098bce8f576d3d684ba87f9d11f324340e42ef1e3f` (secret key kept mode 0600, not hashed)
- `graph/plan.json` `e8c79ed64bf35f0d64ec23bd81d43fc24569d631a68a48c862ece5e99ea49d2d`; `graph/import-receipt.json` `ce226ac82370bf5bbe0d7941f5c316c676b0d4b614d49ea2faa6a651f8a84d35`; `graph/source-bundle.json` `b8abb366a886bb79314411a3d479a97fe8481173334c238333b9118fe7ab8030`; `cache-closure-policy.json` `9ec692ba1bf2259eb83f96707465cf2160c1a848314b6b32fd92dde75285523b`

## Observations

- Route v2 reproduces E2's build byte-for-byte (`diff -r` empty, NAR identical) and then reuses it with no build, no
  NAR download, and no HTTP request at all.
- Both reuse runs classify from the indexed `built` receipt and recompute the NAR from present bytes; they spend ~14 min
  before publishing, consistent with a single-core closure/digest audit rather than a build (no sandbox process runs in
  run 2 and the cache is never contacted).
- The signer rule holds both ways: a different local key gives `signer-mismatch` with no receipt; adding the original
  builder public key as `--trusted-builder-public-key` restores `reused`. The reused entry keeps the original
  `signer` identity (reuse mints no signature), so trust is carried by the original `built` receipt plus the
  PathInfo signature over the present bytes.

## Cairn convergence on the route-v2 receipts (landed Cairn 2be6695)

CLI: `/nix/store/32ldglhzd6dfm1xahd743m3br4faid2n-cairn-0.2.1/bin/cairn` (BLAKE3 `bb96dd4d…`). Cases are copies of Terrane main `91333c2`'s `proof-rails/verus-bounds/convergence` with the v2 realization receipt, the v2 store info (`cairn/store-info.json`, from `mantle store info` on the v2 store), the v2 handoff (`handoff/built`), `trusted_builder_keys = [3da127ef…]`, and for the reused case `original_realization_receipt` = R1. Root: Terrane checkout at `e8a961af3dbeb6f7a0f7dfbf82100bfe2abcb482`. Command: `cairn convergence inspect --case <case>/case.json --root <checkout>`.

| case | exit | bounds.clamp | issues | bound | inspect receipt BLAKE3 |
|---|---|---|---|---|---|
| built (R1) | 0 | current | none | root_disposition built, route v2 | `90987e9b76f6a381…` |
| reused (R2 + original R1) | 0 | current | none | root_disposition reused, original_receipt_blake3 `14df9a9a…` (= R1) | `b22c55b7832590ce…` |
| negative: tampered original receipt_blake3 | 0 | stale | convergence.reuse_link_invalid | | `70d4c47a2e84d2c6…` |
| negative: built, trusted_builder_keys [] | 0 | unrealized | convergence.untrusted_signer | | `b7950a96cce4e74f…` |
| negative: reused, trusted_builder_keys [] | 0 | unrealized | convergence.untrusted_signer | | `a28139984ccbcbee…` |
| negative: reused, decision aggregate failed | 0 | failed | convergence.aggregate_failed | | `d270322054646f4f…` |

`targets_current` is true for both positive cases; `complete` is false because the other Terrane requirements are untargeted, as on main.

## Files kept outside the repository (over 5 MB)

| file | bytes | BLAKE3 |
|---|---|---|
| `compare/mantle-built-root.nar` | 9867968 | `6a1c6095606e9aec2854f127e7e070bf6b2afd4aab9df82fe74313cb76e7fe50` |
| `compare/nix-built-root.nar` | 9867968 | `6a1c6095606e9aec2854f127e7e070bf6b2afd4aab9df82fe74313cb76e7fe50` |
| `graph/plan.json` | 73715736 | `e8c79ed64bf35f0d64ec23bd81d43fc24569d631a68a48c862ece5e99ea49d2d` |
| `handoff/built-replay/tool-closure.derivation-json.json` | 24295507 | `8bc04fed7cf257b27492382e29a8580820d564e900783e7650149f1e1d8928e8` |
| `handoff/built/tool-closure.derivation-json.json` | 24295507 | `8bc04fed7cf257b27492382e29a8580820d564e900783e7650149f1e1d8928e8` |
| `transcripts/01-plan.txt` | 77124888 | `a21fd27325b8351a60a4fa9f5b908bf4454303ef80553a89d686aafabaf2b4f0` |

The Cairn case directories under `/home/brittonr/wt/convergence-evidence/build-or-reuse/wp8-v2/cairn/` hold copies of the handoff and producer output; their inspect receipts are retained here.
