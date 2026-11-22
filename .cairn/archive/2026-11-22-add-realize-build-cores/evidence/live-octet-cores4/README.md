# Octet Aeneas pilot on route `cache-inputs-build-roots-v2` with `--build-cores 4`

Status: **completed** — the Aeneas job built (run 1, `build_cores = 4`), was reused (run 2, zero builds and zero
downloads), and Cairn reports the target row `current` for both the built and the reused case. Three recorded
deviations: attempt 1 failed on a full ZFS pool (environment, not the route); the successful build's log is not
retained by Mantle, so the `affinity list` line could not be read back (see *Cores evidence*); the case evidence
deck path had to be relocated into the case's producer output, which required recomputing the Octet decision used by
the Cairn cases (see *Cairn input adaptation*).

## Inputs

| Field | Value |
|---|---|
| Mantle `M` | `/nix/store/0j2489gc0g3nihxfix9864xrki53g19h-mantle-0.1.0/bin/mantle`, `mantle 0.1.0` (checked build of 8ae03535b); BLAKE3 `b578eea0f7b9781dbf1849cb11f1de1d829a8c0e50e5d3d0b99b02e6ee4fcc3a` — **matches** the required value |
| Bridge `BR` | `/datapool/mantle-store/build-or-reuse-20261122/bridge-target-8ae03535b/release/mantle-verification-bridge`; BLAKE3 `e387bc92efba55d0a9f2bc5046a6e475bf6b1c10d95cc4b1e53ff53678beb840` — **matches** the required value |
| Cairn CLI `C` | `/nix/store/32ldglhzd6dfm1xahd743m3br4faid2n-cairn-0.2.1/bin/cairn`, `cairn 0.2.1`; BLAKE3 `bb96dd4d90f71495511867cb6874718fb44cf5f1ea0f4a446afaf5008899f7da` |
| Octet CLI | `/home/brittonr/wt/pilot-v2/octet-pilot/target/release/cargo-octet`, `cargo-octet 0.1.1-rc.1`; source root `/home/brittonr/wt/pilot-v2/octet-rev` (d5b34c2d64b1227417a46df8ed8427f2d10ed3c8) |
| Source configs `SRC` | `/datapool/mantle-store/build-or-reuse-20261122/mantle-src-8ae03535b` |
| Root derivation | `/nix/store/vl7bqxyz2f86x0vlvxipcvryavckb9wj-octet-mantle-aeneas-consumer-job-v1.drv` |
| Producer export | `/home/brittonr/wt/convergence-evidence/realization/producer-export` |
| Filtered cache | `/datapool/mantle-store/build-or-reuse-20261122/pilot-inputs-cache` (593 narinfo, 588 NARs, 5.7 GB), served on `127.0.0.1:18571` |
| Cache URL | `http://127.0.0.1:18571?trusted_public_keys[0]=convergence-pilot-20261119-1:sRZV4nbjXTCkhRjkEIVe5Pg4G2ajYaCobLqS0JJVu2A%3D` |
| Cache-closure policy | `cache-closure-policy.json`, `max_nar_bytes` 34359738368; 157 B, BLAKE3 `69539adfb1bfe41c6c07f94c488c8c9bb4e1869e3e8865be29e1e514f2c4b278` |
| Endpoint binding | `binding/remote-endpoint-binding.json` (0600), 1347 B, BLAKE3 `63840fe2d8bb882d930f08e18dd83d426fed88fb544983f2a514c37126cb987b`; endpoint table `binding/endpoint-state/remote-builders/endpoint-bindings.json` 823 B BLAKE3 `315ba69f729fc3e3a552f69b2e1a470d7e348a7505630463696334880ed04508` |
| Run root `RUN` | `/home/brittonr/wt/pilot-v2-cores4-run` (root filesystem; `store/` 22 GB, `state/` 6.8 GB) |
| Space before realize run 1 | `df -h $RUN` → `/dev/nvme1n1p3 1.8T 1.6T 157G 91% /`; `zfs list -H -o avail datapool` → `72.2G` (15:31): `logs/df-before-run1.txt` |
| Evidence dir `N` | `/home/brittonr/wt/convergence-evidence/build-or-reuse/pilot-v2-cores4/` |

Every Mantle command ran through the staged `evidence-cmd.sh` (loopback cache + `run-transcript.sh`); Octet and Cairn
ran through `run-transcript.sh`. Every realize, `store info` and `store verify` command used
`/usr/bin/env … TMPDIR=$RUN/tmp … --json --log-level warn`. Staged files were not edited.

## Steps

| # | Step | Command (transcript) | Exit | Wall | Compared files / result |
|---|---|---|---|---|---|
| 1 | produce-nix | `$M --json foreign-import produce-nix --derivation-json …/producer-export/tool-closure.derivation-json.json --root-derivation …vl7bq….drv --producer-identity octet@d5b34c2d… --producer-revision d5b34c2d… --package octet-mantle-aeneas-consumer-job --system x86_64-linux --cache-url "$CACHE_URL" --out-dir graph --state-dir $RUN/plan-state` (`01-produce-nix`) | 0 | 1.48 s | `verdict accepted`, no diagnostics. `graph/nixpkgs.graph.json` 64 020 308 B BLAKE3 `e7477a59…08732cda` and `graph/nixpkgs.index.json` 493 B `64375405…3b81d8ec` **identical to pilot-v2** |
| 2 | plan | `$M --json foreign-import plan --graph … --package-index … --policy $SRC/…/preserve-nix.json --execution-profile $SRC/…/nix.json --package octet-mantle-aeneas-consumer-job --system x86_64-linux --nix-compat --realization-route cache-inputs-build-roots-v2 --plan-out graph/plan.json --receipt-out graph/import-receipt.json --state-dir $RUN/plan-state` (`02-plan`) | 0 | 107.72 s | plan identity `c98afa470f6bda9b112351fe2b5d46ed406769fc7e2c9a730320e149f03e91c9` (= pilot-v2); `plan.json` 115 981 614 B `fde2edc9…41d258dc` (= pilot-v2); `import-receipt.json` 2 516 922 B `938bf9f6…a00524e` (= pilot-v2) |
| 3 | prepare-sources | `$M --json foreign-import prepare-sources --plan graph/plan.json --out graph/source-bundle.json --state-dir $RUN/plan-state` (`03-prepare-sources`) | 0 | 62.52 s | bundle 312 B `b8abb366…7ab8030`; manifest `8ed1103b5de3054ee13ea391af805e276e3cc3b6ceaa50b2149193adb1ff1777` |
| 4a | realize run 1, attempt 1 (pre-relaunch) | realize on the old `/datapool/.../pilot-v2-cores4` root, `--build-cores 4`, no `TMPDIR`/`--log-level` (`04-realize-run1`, `logs/server-04-realize-run1.log`) | **3** | **714.47 s** | failed during hydration before any build: `http-closure-content-admission-failed: plshvwcnl…-rust-docs-1.96.0-nightly-2026-03-21-x86_64-unknown-linux-gnu`; pool 100 % full (≈2.7–6.5 GB free, closure ≈20.9 GiB); 403 paths written, no receipt, no root, no affinity line. Transcript/log retained; run root deleted by the orchestrator |
| 4b | realize run 1, attempt 2 | `/usr/bin/env PATH=/var/empty SNIX_BUILD_BWRAP=…/bwrap TMPDIR=$RUN/tmp $M --json --log-level warn --state-dir $RUN/state --store $RUN/store --store-prefix /nix/store --remote-binding-config binding/… foreign-import realize --plan … --import-receipt … --source-bundle … --source-bundle-blake3 8ed1103b… --execution-profile …/nix.json --cache-closure-policy cache-closure-policy.json --receipt-out receipts/realization-receipt-run1.json --substitute --build-cores 4` (`04-realize-run1-attempt2`, `logs/server-04-realize-run1-attempt2.log`) | **0** | **4178.19 s** | Receipt v2 `complete`/`realized`, `receipt_blake3 8e3edd43a3de0d9dfb14c9c8ffca68413b025e5d39921a0b314d53cb12fd8ace` (6 543 173 B); root `zjq6whhf2c641vkz89kgwwpbajjq5cbq-octet-mantle-aeneas-consumer-job-v1` `disposition: built`, `nar_sha256 922b7ec9…fc2946`, `nar_size 577752`, signer `crunch-britton-desktop-1` / `35e4d2ac…7b43ee` (builder), `built.build_cores = 4`, `input_closure_blake3 698061e7…4434be`; units: 1 `built`, 520 `substituted`, 12 `already-present`, 11 032 `not-required-cache-only`; server: 594 narinfo + 593 NAR GETs. **No `affinity list` line is retained** (see *Cores evidence*) |
| 5 | realize run 2 (reuse) | same command, `--receipt-out receipts/realization-receipt-run2.json` (`05-realize-run2`, `logs/server-05-realize-run2.log`) | 0 | 238.08 s | Receipt v2 `complete`/`realized`, `receipt_blake3 480e81e13165afae3328841166b8b4249ddf9d02c243b2a113c9bdf9db7b3db7` (6 024 263 B); root `disposition: reused`, `reused {original_receipt_blake3: 8e3edd43…8ace (= R1), source: "local", builds: 0, downloads: 0}`, `built: null`, same signer/NAR. Server log contains **only** the `Serving HTTP …` header — **zero HTTP requests, zero NAR downloads**; units 520 `substituted` + 11 045 `already-present` |
| 6 | NAR + tree compare | `nix-store --dump <Mantle root> > compare/mantle-built-root.nar; nix-store --dump <Nix root> > compare/nix-built-root.nar; cmp; diff -r <Nix root> <Mantle root> > compare/diff.txt` (`06-compare`; Nix baseline separately `06-nix-built-nar`) | 0 | 0.36 s / 1.52 s | Mantle NAR 577 752 B sha256 `922b7ec9cb2a4837125dff0346441e76432ff77def80ebd036320fe558fc2946` (= receipt), BLAKE3 `6d0e3aac…579a2`. Nix NAR 580 056 B sha256 `2a12b198623393a6887de8051200fa034c3535e842c8ed7e15250ac0bba284be` (**matches** Nix's value), BLAKE3 `eca2a910…f7f90f54`. `cmp` exit 1; `diff -r` exit 1, 394 378 B in `compare/diff.txt`: 11 differing files (see *Determinism finding*). Compared Nix NAR sha256: equal to `2a12b198…`; the two roots are **not** byte-identical |
| 7 | store info | `/usr/bin/env TMPDIR=$RUN/tmp $M --json --log-level warn … store info zjq6whhf… \| tee cairn/store-info.json` (`07-store-info`) | 0 | 0.05 s | `schema mantle-store-info-v3`; `nar_sha256 922b7ec9…`, `nar_size 577752`, deriver `szzqg5vg…`, references `[zjq6whhf…, 3lh4savp…, 3r2zm018…, 5rm1x1c5…]`, signature `crunch-britton-desktop-1:3RRR72o8…LBg==`; `cairn/store-info.json` 2 073 B BLAKE3 `6054569a…dbfd8` (= Cairn's `store_info_blake3`) |
| 8 | store verify | `/usr/bin/env TMPDIR=$RUN/tmp $M --json --log-level warn … store verify zjq6whhf…` (`08-store-verify`; a first attempt passing the PathInfo *signature* as `--trusted-public-keys` is kept as `08-store-verify-attempt1`, exit 3 `Invalid pubkey len`) | 0 | 0.04 s | `OK zjq6whhf2c641vkz89kgwwpbajjq5cbq-octet-mantle-aeneas-consumer-job-v1 trusted_signatures=1/1`; `1 checked, 0 mismatches` |
| 9 | Octet decision | `cargo-octet requirements check --contract standards/requirements/cairn-pilot/contract.json --evidence standards/requirements/fixtures/cairn-pilot/evidence.json --source-root …/octet-rev --output cairn/decision.json --output-format json` (`11-octet-check`), then `requirements verify … --decision cairn/decision.json` (`12-octet-verify`) | 0 / 0 | 0.12 s / 0.02 s | `decision_blake3 = 906cc950af8318c420b9c8188f325aede254fe2c54360b4881c5c87f2d4f9a45` (**= e2e step 5**); `evidence_blake3 5c620f8c…`; verify `status passed`, recorded = recomputed, no diagnostics |
| 10 | Cairn convergence | `cairn convergence inspect --case cairn/case-built/case.json --root …/octet-rev` (`13-cairn-built`), same for case-reused (`14-cairn-reused`), then 3 negatives (`15-…`, `16-…`, `17-…`) | 0 each | 0.16 / 0.22 / 0.20–0.23 s | case B row `octet.aeneas_consumer_contract.compatibility` **`current`**, issues `[]`, `bound.root_disposition = built`, `bound.realization_route = cache-inputs-build-roots-v2`, `bound.signer = crunch-britton-desktop-1`, `bound.target_path = zjq6whhf…`; case U **`current`**, `bound.root_disposition = reused`, `bound.original_receipt_blake3 = 8e3edd43…8ace` (= R1). Negatives: tampered original receipt_blake3 → `stale` + `convergence.reuse_link_invalid`; built and reused with `trusted_builder_keys: []` → `unrealized` + `convergence.untrusted_signer`. `complete = false` for both (other Octet requirements untargeted, as in wp8-v2) |
| 11 | README | this file | — | — | inputs, steps, records, limits |

### Cores evidence (why there is no `affinity list` line)

Mantle 8ae03535b retains the build log only for a failed realization (attempt 1 of pilot-v2 embedded it in the
receipt's failure field); for a successful realize the log is dropped: `$RUN/state/logs` is never created and
`mantle log --list` (and `mantle log zjq6whhf…`) exits 3 with `log directory …/state/logs does not exist (no builds
yet?)`, and the receipt has no log field. The sandbox work tree keeps only the declared inputs
(`…/scratches/build/env-vars` etc.), no captured stdout. Because attempt 2 completed, the affinity line cannot be read
back, and re-running the realize would not rebuild.

Observed evidence that the build was given four cores:

- receipt `roots[0].built.build_cores = 4` (Mantle's recorded `NIX_BUILD_CORES` for this build);
- sandbox environment `…/state/build-work/sandbox-remote/eeba4eba-…/scratches/build/env-vars`:
  `declare -x NIX_BUILD_CORES="4"`;
- wall time 4 178 s for a job that at one core failed the producer's 45-minute rail (pilot-v2: 5 992 s, stage
  `prove` timed out) and that Nix built at four cores in 36:28;
- the proof log (`artifacts/proofs/…stdout`) shows concurrent Lean module builds (e.g. `[7/204]` … `[17/223]`).

### Determinism finding (step 6)

`diff -r` of the Nix-built and Mantle-built roots reports 11 differing files, all of which embed build timing or
progress: `artifacts/{evidence-log-manifest.json, summary.txt, provenance.jsonl}` and their root copies
(`evidence-log-manifest.json`, `summary.txt`, `provenance.jsonl`), `producer-artifact-verification.json`,
`valence-provenance.json`, and `proofs/…{stdout,stderr}` + `artifacts/proofs/…{stdout,stderr}` (e.g. `Total execution
time: 0.263448` vs `0.360763` seconds, `Finished dev profile … in 0.54s` vs `2.95s`, different module-progress
counts). Proof artifacts that matter (`proof-cards.json`, the proof receipt, `llbc`, the Aeneas/Lean JSON records)
are byte-identical. Per the spec this is recorded as a determinism finding; the Mantle-built root was used for every
later step.

## Cairn input adaptation

`e2e/step6/reproduced/case.json` carries the Octet fixture evidence verbatim, and that fixture names the deck by the
absolute e2e store path `/datapool/mantle-store/convergence-pilot-20261119/store/zjq6whhf…/artifacts/proof-cards.json`.
Cairn requires the deck locator to lie under the case's `producer_output_locator`
(`crates/cairn-cli/src/convergence.rs:286-352`, `deck_relative_path`), and the spec requires that locator to be the
`$RUN/store` path, so the unmodified fixture is rejected with `convergence.proof_deck_outside_output` (kept as
`13-cairn-built-attempt1` / `14-cairn-reused-attempt1`). Switching the deck to a relative path (as wp8-v2's fixture
does) also fails, because Octet evaluates the evidence and needs the deck to exist at its stated absolute path.

Adaptation applied only to the two case copies of `evidence.json`: `proof_decks[0].deck` =
`/home/brittonr/wt/pilot-v2-cores4-run/store/zjq6whhf…/artifacts/proof-cards.json` and `artifact_root` = the same
`$RUN/store` root. That evidence is truthful (the deck is read from the Mantle-built root) and Octet then passes:
case evidence BLAKE3 `ab8b12d7ca3f64bb4f50770a09c1346b8a44fb6fad958d446658d63ed316e6de`, case decision
`8abe489a539df45e4cc7408e9ad3c8d215139846a2b0112105e7a6345923c383` (`cairn/decision-case.json`, aggregate
`passed`). The unmodified fixture decision `906cc950…` remains recorded at `cairn/decision.json` for the step-9
identity check. Cairn reads deck bytes from the case's `producer-output/` copy; that copy is byte-identical to the
run root's proof cards.

Case layout (per case): `case.json`, `contract.json`, `evidence.json`, `generated.json`,
`requirement-registry.json`, `targets.json`, `decision.json`, `realization-receipt.json`, `store-info.json`,
`handoff/` (copy of `handoff/built`), `producer-output/zjq6whhf…` (copy of the Mantle-built root). `producer_output_locator`
is `/home/brittonr/wt/pilot-v2-cores4-run/store/zjq6whhf…`; `trusted_signers` `["crunch-britton-desktop-1"]`;
`trusted_builder_keys` `["35e4d2ac280150a776e615c85d23c8c4cea964a054afb59f3554334b197b43ee"]`. Case U adds
`original_realization_receipt` = R1. Negatives are copies of those cases (tampered original `receipt_blake3`;
`trusted_builder_keys: []`).

## Retained records (size, BLAKE3)

Transcripts (`transcripts/`):

- `01-produce-nix.txt` 1 635 B `c38bfdaf…209052`
- `02-plan.txt` 121 290 373 B `10677707…3ad8330`
- `03-prepare-sources.txt` 840 B `396c3c62…178c2f7`
- `04-realize-run1.txt` 1 774 B `1d91d7df…54413e` (attempt 1, failure)
- `04-realize-run1-attempt2.txt` 5 455 857 B `e0253d32…b8771c`
- `05-realize-run2.txt` 5 013 780 B `ba2f3b8e…958274`
- `06-nix-built-nar.txt` 812 B `16deb0bb…314100`
- `06-compare.txt` 4 182 B `b9d0b328…992d2d`
- `07-store-info.txt` 2 708 B `c7a68d09…f41eb`
- `08-store-verify-attempt1.txt` 879 B `1fdd0cff…b61a415c`
- `08-store-verify.txt` 655 B `b99e3fd5…c12f6a81`
- `09-bridge.txt` 3 997 B `42db752d…ee4bcc`
- `10-bridge-replay.txt` 4 034 B `646ba590…d513c18`
- `11-octet-check.txt` 2 844 B `00836f30…1eb7a0`
- `12-octet-verify.txt` 1 524 B `02cf7f3f…53fa6ab`
- `13-cairn-built-attempt1.txt` 1 086 B `3f387b65…c52ab8` (deck-outside-output refusal)
- `13-cairn-built.txt` 232 362 B `6171e556…4794927`
- `14-cairn-reused-attempt1.txt` 1 090 B `02a7a49d…8e058d`
- `14-cairn-reused.txt` 232 428 B `fe2c7130…bc4f6d81`
- `15-neg-reused-tampered-original.txt` 231 686 B `6d4cf687…de86a92`
- `16-neg-built-untrusted-builder.txt` 231 686 B `17e35cd5…34c083ea`
- `17-neg-reused-untrusted-builder.txt` 231 689 B `0c0cafd2…b3b4add42`

Full BLAKE3 values, in the order above:
`c38bfdafcb38218fec6017cce8a268607eaeaf77c58fe568d32ca3ed87209052`,
`10677707e05abc922a4d625599f0d2f8feb7987012d67fd54cd8a3b8d3ad8330`,
`396c3c6235aee63246187f030b6b08f9df4095baa673361fa50f232d1178c2f7`,
`1d91d7dff51fc6ac939e42b5a5ac1bb88f67edaff9d4de17cd43f3a72b54413e`,
`e0253d3278d3cdd1740fa303be1e837804331e1a7fb77c155f2864ab07b8771c`,
`ba2f3b8ec14d8270881586f2ff951aa1c01b6b5f2ca2f5c5aa7573240f958274`,
`16deb0bba04406aef2ae449bfd134c834225e07f942e39a45bf54ce557a23100`,
`b9d0b32831f8135496ba368f037da71edcaede48c9ae9ecfce56bfe402992d2d`,
`c7a68d096d953d1379e1f34d0b4eab6ef60c100e1d58801f9da76dbf0e1f41eb`,
`1fdd0cff67ba086bbe91a8c94730f8a6870b07e9801fee04fa539069b61a415c`,
`b99e3fd50d0a53116c0ba1d66eec823dca234c1569566ae7072a82eec12f6a81`,
`42db752dcc062c3ac1f789cd4e8b6ba808ffaf1c7b27737e3740b32006ee4bcc`,
`646ba5906df7a2da6ab11570812b9bfb8461532f2dc752f46f8494488d513c18`,
`00836f30705f488fdc92ab74b1e63f964b818d36bde23ad4859be929c01eb7a0`,
`02cf7f3fde6fbd4f4ddc639305dc01e5912b30bf6ae2755378a87a3de53fa6ab`,
`3f387b65d6d70f1b86534e8a05f8c4e3e343242c1341c11ba880e48882c52ab8`,
`6171e556c700f64258533802e05454ec87701f8ed120d1046eee5cd0c4794927`,
`02a7a49d509c4e3ad7fc41d10f0f76c567dee87a763013f43a2492f68b8e058d`,
`fe2c7130ac53a0df4804c58c5abc229ac2cc6e41a5e67a3da292eeb0bc4f6d81`,
`6d4cf68705227e31e2202d4b5dbfad042520cddbffbce492cbf57435ede86a92`,
`17e35cd5d1242a39bfbdb92206c9c4d5b1842422366417f3f3a6c29134c083ea`,
`0c0cafd2d8dc9c009ccd42027a6bc9db8038d0a7908fb842418fa1023b4add42`.

Logs (`logs/`):

- `df-before-run1.txt` 244 B `5a3b9928…0ceee94d`
- `server-01-produce-nix.log`, `server-02-plan.log`, `server-03-prepare-sources.log`,
  `server-05-realize-run2.log` each 67 B `4a58da4e515d4cbad000f201c8bc5e098d1c561ed68a777584b1f746c7f9cd71` (header only)
- `server-04-realize-run1.log` 109 714 B `aef62035…bd067d94` (attempt 1: 594 narinfo + 404 NAR GETs)
- `server-04-realize-run1-attempt2.log` 133 217 B `cd485e00…16ceb9e3` (attempt 2: 594 narinfo + 593 NAR GETs)

Receipts (`receipts/`):

- `realization-receipt-run1.json` 6 543 173 B `8b4a78809e270ece7632a462fa4dd477a449f341611bd5cd781e241abe6fae8b`
  (`receipt_blake3 8e3edd43a3de0d9dfb14c9c8ffca68413b025e5d39921a0b314d53cb12fd8ace`)
- `realization-receipt-run2.json` 6 024 263 B `93c75b1d8e86dd1b7bbd647271e816fc7e96d658a5514df68663551c2c9c1047`
  (`receipt_blake3 480e81e13165afae3328841166b8b4249ddf9d02c243b2a113c9bdf9db7b3db7`)

Plan/source artifacts (`graph/`):

- `nixpkgs.graph.json` 64 020 308 B `e7477a59f6e134a126b91c195d4fd0ef73df9c5ea6e943d2a2f3340b08732cda`
- `nixpkgs.index.json` 493 B `64375405dde98af9bdbee705ac5b0faec541c7f215767d75082228953b81d8ec`
- `plan.json` 115 981 614 B `fde2edc93bb0d4ce4149751366dcd56f8f4f4cdbe34b9d5664ac6b6a41d258dc`
- `import-receipt.json` 2 516 922 B `938bf9f64c66c4fa16f8b4b794ff84b94343a25aa85fcb3d273f9a044a00524e`
- `source-bundle.json` 312 B `b8abb366a886bb79314411a3d479a97fe8481173334c238333b9118fe7ab8030`

Compare (`compare/`):

- `nix-built-root.nar` 580 056 B `eca2a9106906a91a36bb2dc79e3d5fef48beadcb55a9ed9a9c2110fec7f90f54`,
  sha256 `2a12b198623393a6887de8051200fa034c3535e842c8ed7e15250ac0bba284be`
- `mantle-built-root.nar` 577 752 B `6d0e3aac49cefb635382383c0b9368f24c433690db5fad80db4cb4e1b02579a2`,
  sha256 `922b7ec9cb2a4837125dff0346441e76432ff77def80ebd036320fe558fc2946`
- `diff.txt` 394 378 B `857190b9895561ebabfc40751457d9fda19505632ddc7a7145716c419709faaa`
- `handoff-built-vs-replay.txt` 0 B `af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262`
- `handoff-built-vs-realization.txt` 2 895 B `7dfb20d9ff9db1ef437e0c4a77d8fab080c55699c91b9643b4ba6baa30f912ad`

Handoff (`handoff/`): `built/` and `replay/` 38 MB each, tree-identical (`diff -r` empty). Identities in
`built/mantle-observation.json` (3 381 B, `6cb01715…4b85d2`): receipt `b3:84d2e777…420e`, producer job
`b3:a14ec403…718`, tool graph `b3:d0ebfcba…7808`, association `b3:fa46849e…7b1`, external report
`b3:39206064…82f8`. `job.json` (3 655 B) `a14ec403025db0330379bad1e1904fae23835ccd1ab95017a80e3e5ca715b318`
(= producer job identity). Compared with `realization/handoff`: 3 files differ (`mantle-observation.json`,
`producer-artifact-verification.json`, `summary.txt`); receipt/producer-job/tool-graph identities are equal, the
association and external-report identities differ because both hash the nondeterministic proof stdout captured in
the output (same root cause as the step-6 finding).

Cairn (`cairn/`):

- `decision.json` 2 330 B `ca047f54add015d84784694c8ccdaa18c74c1f58bf3a835d0eb2df5c7dec91c7` (`decision_blake3 906cc950…`)
- `decision-case.json` 2 330 B `9206d9082460878536d2bce5bcec55e0ee1b09acf5c709ab4866c1f483c265de` (`decision_blake3 8abe489a…`)
- `store-info.json` 2 073 B `6054569af88a5ed0cafc9aa8c2bfb2431bd751ac702f6a4be95ba8c1749dbfd8`
- `inspect-built.json` 231 810 B `4ab1718d855b8d89e7cd38d248130b1455ee5914e54f85d08ef3fe74bbadd125`
- `inspect-reused.json` 231 873 B `42eadf99f745f17d9c5b33e8cca3d54c0ac1fba8cad8d30d7ac3e88a96c7370e`
- `negatives/reused-tampered-original.json` 231 072 B `15733a2010159ece755f7a48f735eaad647244184d82f01f2ee7b0c270ad27c3`
- `negatives/built-untrusted-builder.json` 231 075 B `6c64a1720dec2914a10b26aa5c94cc53b424be5393129e248d32c1b25867f34c`
- `negatives/reused-untrusted-builder.json` 231 075 B `531f10c10977a1ef8494c80269bee41118d712356535bade2200d53b7d21af24`
- case directories: `case-built` 45 MB, `case-reused` 51 MB, negatives 45/50/51 MB (each holds a copy of the
  producer output, the handoff and the receipts; case `case.json`/`evidence.json`/`decision.json` as described above)
- `probe/evidence-absolute.json` (fixture evidence kept unmodified), `probe/decision-adapted.json` (the relative-deck
  experiment that Octet rejects)

Staged inputs: `cache-closure-policy.json` 157 B `69539adf…2c4b278`; `binding/remote-endpoint-binding.json` 1 347 B
`63840fe2…cb987b`; `binding/endpoint-state/remote-builders/endpoint-bindings.json` 823 B `315ba69f…d04508`;
`evidence-cmd.sh` 609 B `36d282a2…aec315`; `run-transcript.sh` 703 B `9d0de298…388b9`.

## What passed, what differed, what was not run

- Passed: binary identities; byte-identical graph/index/plan/source-bundle/import-receipt against pilot-v2;
  `--build-cores 4` realization (`built`, `build_cores 4`, signer `crunch-britton-desktop-1`); reuse with zero builds
  and zero NAR downloads linked to R1; store info/verify; bridge handoff reproduced exactly under `PATH=/var/empty`
  and stable against `realization/handoff` in every identity except the two derived from timing-bearing proof logs;
  Octet decision `906cc950…`; Cairn `current`/`built` and `current`/`reused` with the expected `bound` fields and all
  three negatives.
- Differed: the two built trees are not byte-identical (11 timing/progress files, 2 304 NAR bytes) — a determinism
  finding, not a reuse failure; the case evidence deck path had to be relocated and the case decision recomputed
  (documented above); the `affinity list` line was not observable because Mantle retains no successful build log.
- Not run: nothing in steps 1–11; no repository edits, no negative signer-key experiment beyond the three Cairn
  negatives specified.

## Orchestrator observations (live, not from retained logs)

- CPU affinity during the build, sampled with `taskset -cp` over the descendants of the realize process at 16:05:27:
  33 processes on `0-3` (lake, lean, cargo-octet and their workers) and 2 on `0-31` (the outer bwrap processes).
  This stands in for the `affinity list` build-log line that Mantle does not retain for a successful realize.
- Lean progress (`.olean` files under the `lake` working directory, with `lake` elapsed seconds): 427 at 438 s,
  992 at 864 s, 1 295 at 1 174 s, 1 458 at 1 483 s, 1 510 at 1 794 s; `lake` exited before 16:35, about 34 minutes
  after it started, under the 45-minute stage budget, while the host load average ranged from 38 to 195 on 32 CPUs.
- Before the sandbox started, the realize process spent about 19 minutes (15:40–15:59) reading 72 GiB with no child
  processes, about three times the 21.7 GB input closure. After the output was written (16:33:27) it was idle for about
  6 minutes (no CPU, no cache requests, threads in futex and epoll waits) before writing the receipt at 16:41. Both
  are performance observations, not correctness failures.
- At 15:33:31, on the fresh state directory, redb logged `not shutdown cleanly. Repairing` for `directories.redb` and
  `pathinfo.redb`.
- The decision identity changed only because Octet evidence names the proof deck by absolute host path
  (`artifact_root`, `deck`); `artifacts/proof-cards.json` is byte-identical in the Nix-built and Mantle-built roots.
- Attempt 1's cause was established from the pool, not from a log: `zfs list` showed `datapool` with 6.3 GB
  available at 15:27, the cached NAR's sha256 equals its narinfo `FileHash`, and `crates/crunch-store` is unchanged
  between 6594e7fd1 (whose pilot-v2 run hydrated the same NAR) and 8ae03535b.

## Files kept outside the repository (5 MB or larger)

Retained under `/home/brittonr/wt/convergence-evidence/build-or-reuse/pilot-v2-cores4/`; the Cairn case directories there also hold full copies of the handoff and producer output.

| file | bytes | BLAKE3 |
|---|---|---|
| `graph/nixpkgs.graph.json` | 64020308 | `e7477a59f6e134a126b91c195d4fd0ef73df9c5ea6e943d2a2f3340b08732cda` |
| `graph/plan.json` | 115981614 | `fde2edc93bb0d4ce4149751366dcd56f8f4f4cdbe34b9d5664ac6b6a41d258dc` |
| `handoff/built/tool-closure.derivation-json.json` | 38785189 | `d0ebfcbae2f8b44866a45df068cec42cf5be5b6379ff72cb91bddbe919697808` |
| `handoff/replay/tool-closure.derivation-json.json` | 38785189 | `d0ebfcbae2f8b44866a45df068cec42cf5be5b6379ff72cb91bddbe919697808` |
| `receipts/realization-receipt-run1.json` | 6543173 | `8b4a78809e270ece7632a462fa4dd477a449f341611bd5cd781e241abe6fae8b` |
| `receipts/realization-receipt-run2.json` | 6024263 | `93c75b1d8e86dd1b7bbd647271e816fc7e96d658a5514df68663551c2c9c1047` |
| `transcripts/02-plan.txt` | 121290373 | `10677707e05abc922a4d625599f0d2f8feb7987012d67fd54cd8a3b8d3ad8330` |
| `transcripts/04-realize-run1-attempt2.txt` | 5455857 | `e0253d3278d3cdd1740fa303be1e837804331e1a7fb77c155f2864ab07b8771c` |
