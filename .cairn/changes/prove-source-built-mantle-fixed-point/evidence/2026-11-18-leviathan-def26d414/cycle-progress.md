# Protected cycle from def26d414 (in progress)

Published source `def26d4143ff29e3ef27b63ca092e457b657ec35` carries the cold1736
templated compiler-action admission repair (ADR 0161). This cycle replaces the
70db0868 cohort; no earlier archive, profile, program record, report, review or
alias is reused.

## Captured results

| Step | Observation |
|---|---|
| Deterministic code archive | `source-def26d414….tar.gz` BLAKE3 `4718042f4ec3f43f5cd0480a925d8bdd27dd9cc83a5ca25486e14f17ec54a98f`; inner tar `cc6eb3b76a5a96da9c66869aec5955fc9f12d2529f84b6802497341451824201`; 15,040 files |
| Freeze provenance | archived `src/*.rs`, `adr/0161…`, `Cargo.toml`, `Cargo.lock` byte-identical to `git show def26d414:<path>` |
| Assembly (task 1737) | exit 0; complete source `5ebee42294f64efb44f4cb3e8e34f1d5936ad6caf9dbc7840688ab943f1b5b20`; compressed `82a03d6c106f5459444415c0df3f93ffeb1e8aad136e64f733ea1b108c895de6`; code tree `cc6eb3b7…`; AWS original pins unchanged (`282f13d4…`, tree `969d75db…`, normalized `8f731ad6…`) |
| Canonical qualification (task 1738) | exit 0; 11 `test result: ok` legs, including 154 release-parity, 172, 59, 52, 22, 21, 18, 9, 6, 5, 4 passed; 0 failed |
| Isolated runner qualification (task 1739) | exit 0; 280 passed, 28 ignored, 0 failed; source tree `5ebee422…` identical before and after; `mantle` `d2dd815e0d8e9bafabce46376ebb4c112b1b6951c2c06710d39084111fdf37bb`, `adopt-source-built-stagex` `30fe1ca22d7952e185691a88a138ae5039bd0ab31a7bfc569acef7a23bd1d02d`, `check-native-row-receipts` `9d4164fb67e27fe603f0ba7d3b169da8ac23d735e2af541b58705af7c022e26d` |
| Reviewed-inspector copy | 11 files copied from the frozen assembly's `2026-11-17-immutable-compiler-source`; `graph-audit.py` `8bd9ad9a…`, `identity-audit.py` `ebcd9f1d…` unchanged from the reviewed bytes |
| Reviewer-model authorization | `reviewer-model-authorization.json` BLAKE3 `8ba69a1d33ac0078680a39df121581206b3108ecc27a31673ffc2078162afc88` |
| Archive determinism | rebuilding the archive from the same revision reproduces both identities exactly (`4718042f…` gzip, `cc6eb3b7…` tar) |
| Mantle-source refresh (task 1740, in flight) | new input profile `source-profile-def26d414.json`, 11,730,713,001 bytes, raw BLAKE3 `456f9b8f5d55497997ade97bb08496baed03529b00d41172a7a1b81f066a2458` |
| Source-only preparation (task 1740) | exit 0; input manifest `60ce7bef22f83df935c9fae0ed2e4ce4cd8c9c6f1c079667b4b9a7e227d7d77c`, input plan `f03675b327a2404fb5dcf4faf269375ce4be165a4d2dffff6cc4463a2ec28021`, cohort manifest `172322ccff01853b25a3d6330d69a547a9195464297e656c2be8b82c2922f1f1`; program records native `7e11faf0…`, rust `575c05c9…`, mantle `d687b993…`, stagex `0f26462f…` |
| Cohort profile | 17,318,227,476 bytes, raw BLAKE3 `725195d6d2d69471ea0d04cd995992f6af8bad278fcde943d4b2efdd31e9a658` |
| Subject facts (task 1741) | exit 0; facts `433c999f91585cef28eca7626d891882072870a48b5bfc5a7bede33807fdd903` |
| Full-profile stream parity | ssh stream BLAKE3 `725195d6…` equals the remote file hash; `profile_transport_equal: true` |
| Review subject | `review-subject.json` BLAKE3 `9b48bed3b0de891e3765a777fbdb0cdfbf7ce13c00d1d1ca20fe387d4b0fc192`; remote copy identical |
| Admission rehearsal on the new cohort | passed on the actual new recipes: `mantle-stage1`/`mantle-stage2` each 1,121 rustc actions, 1,047 units, 119 templated units |
| Strict template invariants on the new recipes | 188 templated actions per stage, `failures: []`; output BLAKE3 `44647575…`, identical to the previous cycle because only embedded source digests changed |
| Subject builder validation | reproducing the 70db0868 subject from its own facts, preparation report, plan and complete-source snapshot yields byte BLAKE3 `2081b08f4933319c236800f1b709144c2581d1423c6b1d8a9a8077f50a33652d`, identical to the reviewed subject |

## Pending

Source-only preparation (task 1740), subject facts and profile stream parity,
subject construction, six re-pinned inspections plus the admission rehearsal on
the new recipes, two fresh complete reviews, sealed acceptance, four Cairn
reports, one genuine cold attempt, then signed adoption and native rows only
after a verified v2 receipt.

No provider or stage execution, fixed point, verified receipt or native-row
acceptance is claimed by anything above.
