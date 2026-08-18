# Aspen replay blocked after fresh release evidence

Date: 2026-06-29T18:55:34Z

## Question

Can Aspen produce the required separate-machine witness replay for fresh release stabilized-self-hosting-witness-replay-2026-06-29 in this session?

## Fresh local release evidence

Release id: stabilized-self-hosting-witness-replay-2026-06-29

Release evidence transcript: cairn/changes/stabilize-self-hosting-witness-replay/evidence/fresh-release-evidence-stabilized-self-hosting-witness-replay-2026-06-29.md

Full proof transcript: cairn/changes/stabilize-self-hosting-witness-replay/evidence/full-self-hosting-proof-no-fuse-after-alias-proof-fix-20260629T180403Z.md

### Release manifest digest fields

~~~text
{"schema":"mantle-release-evidence-v1","release_id":"stabilized-self-hosting-witness-replay-2026-06-29","claim_scope":"packaged-integrity-evidence","workflow":{"command":"./scripts/prove-self-hosting.sh","version":"mantle-self-hosting-proof-v2"},"source_archive":{"kind":"file","relative_path":"source/.tmpCteJh7","size_bytes":972889088,"digest_blake3":"6a3068af1d14b04c45d306fa119a8fcf257a55491c94884ca464a8b2e61f1e21"},"binaries":[{"kind":"file","relative_path":"binaries/01-stage2-mantle","size_bytes":63530144,"digest_blake3":"99d40790a4d356348b07b81be26fab12bb3f02270ba1e87adc60238a5279d328"}],"proof_bundle":{"kind":"directory","relative_path":"proof/self-hosting","size_bytes":127642286,"digest_blake3":"ba93f766f726397e28bb63229ac8a7fdb360973c82b0fb26f3c94c68be46377e"},"prerequisite_inventory":{"kind":"file","relative_path":"proof/inventory.md","size_bytes":14244,"digest_blake3":"3d6cd61c3bfd893ec271556fec7974cacae70a6d989cab21f9213f1dbd5ffadb"},"proof_linkage":{"release_id":"stabilized-self-hosting-witness-replay-2026-06-29","source_archive_digest_blake3":"6a3068af1d14b04c45d306fa119a8fcf257a55491c94884ca464a8b2e61f1e21","proof_bundle_schema":"mantle-self-hosting-proof-v2","proof_mode":"fixed-point","selected_provider_kind":"legacy-fetch","staged_source":"/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmpC61EeS/store/zi7ps2paplpbljym0i0z8jkdq0ymibxq-mantle-src","stage2_binary_digest_blake3":"99d40790a4d356348b07b81be26fab12bb3f02270ba1e87adc60238a5279d328","prerequisite_inventory_digest_blake3":"3d6cd61c3bfd893ec271556fec7974cacae70a6d989cab21f9213f1dbd5ffadb","proof_manifest_digest_blake3":"e2321890ff26a9041f3f15293bb5dde7f33fde0df0b2e392c853ea5be6b8f696"}}
~~~

### Self-hosting proof stage fields

~~~text
stage1_binary: 99d40790a4d356348b07b81be26fab12bb3f02270ba1e87adc60238a5279d328 binaries/stage1-mantle
stage2_binary: 99d40790a4d356348b07b81be26fab12bb3f02270ba1e87adc60238a5279d328 binaries/stage2-mantle
stage1_equals_stage2: true
stage0_hermeticity_mode: practical
stage0_fallback_events: ["bwrap-host-fallback:/nix/store/gr9l6ql3wg70idpqlqhnfdx81hak22c8-bubblewrap-0.11.0/bin/bwrap", "source-host-discovery:/home/brittonr/git/mantle"]
stage2_hermeticity_mode: strict
stage2_fallback_events: []
~~~

Provider proof status: absent from this fresh one-binary release bundle; release manifest records self-hosting proof selected_provider_kind=legacy-fetch and no provider_fixed_point_proof field.

Intended witness identity: aspen1 (policy initialized in the fresh release evidence transcript).

## Aspen reachability attempts

### ssh aspen1 hostname

~~~text
ssh: Could not resolve hostname aspen1: Name or service not known
~~~

exit status: 255

### ssh aspen hostname

~~~text
ssh: Could not resolve hostname aspen: Name or service not known
~~~

exit status: 255

### ssh aspen3 hostname

~~~text
ssh: connect to host 192.168.1.229 port 22: No route to host
~~~

exit status: 255

## Decision

Aspen witness replay is blocked by remote host reachability, not by a new local release or self-hosting failure. Do not mark the Cairn validation/evidence tasks complete, sync, or archive until a real separate-machine witness replay transcript exists and final release verification is recorded.

## Bounded non-claims

- No separate-machine witness was produced in this session.
- No Aspen witness sidecars were imported.
- No final quorum-satisfied release verification is claimed for stabilized-self-hosting-witness-replay-2026-06-29.
