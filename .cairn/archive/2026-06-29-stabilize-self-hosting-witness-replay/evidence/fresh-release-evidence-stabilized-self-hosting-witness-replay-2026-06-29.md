# Fresh release evidence for stabilized-self-hosting-witness-replay-2026-06-29

Date: 2026-06-29T18:52:33Z

Proof bundle: target/self-hosting-proof/run-20260629T140404Z-3929825

Release binary: target/self-hosting-proof/run-20260629T140404Z-3929825/binaries/stage2-mantle

## target/self-hosting-proof/run-20260629T140404Z-3929825/binaries/stage2-mantle release create --release-id stabilized-self-hosting-witness-replay-2026-06-29 --binary target/self-hosting-proof/run-20260629T140404Z-3929825/binaries/stage2-mantle --proof-bundle target/self-hosting-proof/run-20260629T140404Z-3929825

~~~text
release evidence bundle: /home/brittonr/git/mantle/target/release-evidence/stabilized-self-hosting-witness-replay-2026-06-29
release id: stabilized-self-hosting-witness-replay-2026-06-29
source archive: source/.tmpCteJh7
proof bundle: proof/self-hosting
binaries: 1
manifest: manifest.json
~~~

exit status: 0

## target/self-hosting-proof/run-20260629T140404Z-3929825/binaries/stage2-mantle release verify target/release-evidence/stabilized-self-hosting-witness-replay-2026-06-29

~~~text
release evidence verified: /home/brittonr/git/mantle/target/release-evidence/stabilized-self-hosting-witness-replay-2026-06-29
release id: stabilized-self-hosting-witness-replay-2026-06-29
binaries: 1
source digest: 6a3068af1d14b04c45d306fa119a8fcf257a55491c94884ca464a8b2e61f1e21
stage2 digest: 99d40790a4d356348b07b81be26fab12bb3f02270ba1e87adc60238a5279d328
proof mode: fixed-point
reproducibility: absent
deterministic release: absent
provider fixed-point proof: absent
provider fixed-point proof source: absent
~~~

exit status: 0

## env CRUNCH_CONFIG_DIR=/home/brittonr/git/mantle/target/release-signing/stabilized-self-hosting-witness-replay-2026-06-29/publisher target/self-hosting-proof/run-20260629T140404Z-3929825/binaries/stage2-mantle release attest target/release-evidence/stabilized-self-hosting-witness-replay-2026-06-29 --verification-dir target/release-verification/stabilized-self-hosting-witness-replay-2026-06-29

~~~text
Generated signing key: crunch-britton-desktop-1 (/home/brittonr/git/mantle/target/release-signing/stabilized-self-hosting-witness-replay-2026-06-29/publisher/signing-key)
release attestation: /home/brittonr/git/mantle/target/release-verification/stabilized-self-hosting-witness-replay-2026-06-29/release-attestation.json
signature: /home/brittonr/git/mantle/target/release-verification/stabilized-self-hosting-witness-replay-2026-06-29/release-attestation.json.sig
release id: stabilized-self-hosting-witness-replay-2026-06-29
digest: 635ab117644ca42869f5156b82a26487a8c395c7c899135146ffd34e6b5fa18f
signer: crunch-britton-desktop-1
~~~

exit status: 0

## env CRUNCH_CONFIG_DIR=/home/brittonr/git/mantle/target/release-signing/stabilized-self-hosting-witness-replay-2026-06-29/publisher target/self-hosting-proof/run-20260629T140404Z-3929825/binaries/stage2-mantle attest key-show

~~~text
crunch-britton-desktop-1:EZyF0JD3bJiHePrdN+XEnRsICArnYpQa29IQ+Lh9tco=
~~~

exit status: 0

Release trusted key: crunch-britton-desktop-1:EZyF0JD3bJiHePrdN+XEnRsICArnYpQa29IQ+Lh9tco=

## target/self-hosting-proof/run-20260629T140404Z-3929825/binaries/stage2-mantle attest policy-init target/release-verification/stabilized-self-hosting-witness-replay-2026-06-29 --profile single-witness --trusted-release-signer crunch-britton-desktop-1 --trusted-witness-identity aspen1 --force

~~~text
policy: /home/brittonr/git/mantle/target/release-verification/stabilized-self-hosting-witness-replay-2026-06-29/policy.json
revocations: /home/brittonr/git/mantle/target/release-verification/stabilized-self-hosting-witness-replay-2026-06-29/revocations.json
profile: single-witness
min matching witnesses: 1
trusted release signers: crunch-britton-desktop-1
trusted witness identities: aspen1
~~~

exit status: 0

## target/self-hosting-proof/run-20260629T140404Z-3929825/binaries/stage2-mantle release witness-export target/release-evidence/stabilized-self-hosting-witness-replay-2026-06-29 --verification-dir target/release-verification/stabilized-self-hosting-witness-replay-2026-06-29 --request-dir target/release-witness-requests/stabilized-self-hosting-witness-replay-2026-06-29

~~~text
witness request: /home/brittonr/git/mantle/target/release-witness-requests/stabilized-self-hosting-witness-replay-2026-06-29
request metadata: /home/brittonr/git/mantle/target/release-witness-requests/stabilized-self-hosting-witness-replay-2026-06-29/request.json
release id: stabilized-self-hosting-witness-replay-2026-06-29
layout version: 1
release bundle copy: /home/brittonr/git/mantle/target/release-witness-requests/stabilized-self-hosting-witness-replay-2026-06-29/release-evidence/stabilized-self-hosting-witness-replay-2026-06-29
verification seed: /home/brittonr/git/mantle/target/release-witness-requests/stabilized-self-hosting-witness-replay-2026-06-29/release-verification/stabilized-self-hosting-witness-replay-2026-06-29
~~~

exit status: 0

## Manifest summary

~~~text
{"schema":"mantle-release-evidence-v1","release_id":"stabilized-self-hosting-witness-replay-2026-06-29","claim_scope":"packaged-integrity-evidence","workflow":{"command":"./scripts/prove-self-hosting.sh","version":"mantle-self-hosting-proof-v2"},"source_archive":{"kind":"file","relative_path":"source/.tmpCteJh7","size_bytes":972889088,"digest_blake3":"6a3068af1d14b04c45d306fa119a8fcf257a55491c94884ca464a8b2e61f1e21"},"binaries":[{"kind":"file","relative_path":"binaries/01-stage2-mantle","size_bytes":63530144,"digest_blake3":"99d40790a4d356348b07b81be26fab12bb3f02270ba1e87adc60238a5279d328"}],"proof_bundle":{"kind":"directory","relative_path":"proof/self-hosting","size_bytes":127642286,"digest_blake3":"ba93f766f726397e28bb63229ac8a7fdb360973c82b0fb26f3c94c68be46377e"},"prerequisite_inventory":{"kind":"file","relative_path":"proof/inventory.md","size_bytes":14244,"digest_blake3":"3d6cd61c3bfd893ec271556fec7974cacae70a6d989cab21f9213f1dbd5ffadb"},"proof_linkage":{"release_id":"stabilized-self-hosting-witness-replay-2026-06-29","source_archive_digest_blake3":"6a3068af1d14b04c45d306fa119a8fcf257a55491c94884ca464a8b2e61f1e21","proof_bundle_schema":"mantle-self-hosting-proof-v2","proof_mode":"fixed-point","selected_provider_kind":"legacy-fetch","staged_source":"/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmpC61EeS/store/zi7ps2paplpbljym0i0z8jkdq0ymibxq-mantle-src","stage2_binary_digest_blake3":"99d40790a4d356348b07b81be26fab12bb3f02270ba1e87adc60238a5279d328","prerequisite_inventory_digest_blake3":"3d6cd61c3bfd893ec271556fec7974cacae70a6d989cab21f9213f1dbd5ffadb","proof_manifest_digest_blake3":"e2321890ff26a9041f3f15293bb5dde7f33fde0df0b2e392c853ea5be6b8f696"}}
~~~
