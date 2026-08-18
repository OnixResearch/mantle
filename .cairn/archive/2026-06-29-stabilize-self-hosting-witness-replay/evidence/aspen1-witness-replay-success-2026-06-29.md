# Aspen1 witness replay success

Date: 2026-06-29

## Question

Can `aspen1.local` produce an independent witness replay for release `stabilized-self-hosting-witness-replay-2026-06-29` after the self-hosting path normalization fixes?

## Decision

Yes. Aspen1 replayed the exported request, produced signed witness sidecars for identity `aspen1`, the rebuilt output digest matched the published release binary digest exactly, and local final release verification reached `final_class = quorum-satisfied` with one counted independent witness.

## Commands and evidence

### SSH reachability

~~~text
ssh -o BatchMode=yes -o ConnectTimeout=10 aspen1.local 'hostname; pwd'

Certificate invalid: name is not a listed principal
aspen1
/home/brittonr
~~~

### Check-only preflight on Aspen1

Pueue task: `1737`.

The helper ran inside a public Nix tool shell with nightly Rust, Clang, mold, pkg-config, bubblewrap, git, tar, a copied static busybox sandbox shell, and the release-bundled `stage2-mantle` binary as `CRUNCH_WITNESS_REBUILD_CLI_BIN`.

~~~text
rustc 1.98.0-nightly (df6ee909e 2026-06-28)
cargo 1.98.0-nightly (a335d47ff 2026-06-26)
openssl-pkgconfig-ok
witness request: /home/brittonr/mantle-witness/stabilized-self-hosting-witness-replay-2026-06-29/request
witness scratch root: /home/brittonr/mantle-witness/stabilized-self-hosting-witness-replay-2026-06-29/request.work
SNIX_BUILD_SANDBOX_SHELL: /home/brittonr/mantle-witness/stabilized-self-hosting-witness-replay-2026-06-29/bin/busybox
bwrap: /nix/store/p4zxfmj8r66x3y1gwlb3a15cz33nl6yv-mantle-witness-tools/bin/bwrap
check-only mode: this validates prerequisites and request parsing only; it does not produce publishable witness sidecars
witness rebuild preflight OK: /home/brittonr/mantle-witness/stabilized-self-hosting-witness-replay-2026-06-29/request
release id: stabilized-self-hosting-witness-replay-2026-06-29
scratch root: /home/brittonr/mantle-witness/stabilized-self-hosting-witness-replay-2026-06-29/request.work
verification output: /home/brittonr/mantle-witness/stabilized-self-hosting-witness-replay-2026-06-29/request.work/release-verification/stabilized-self-hosting-witness-replay-2026-06-29
proof bundle output: /home/brittonr/mantle-witness/stabilized-self-hosting-witness-replay-2026-06-29/request.work/proof-bundle
audit metadata: /home/brittonr/mantle-witness/stabilized-self-hosting-witness-replay-2026-06-29/request.work/witness-rebuild-audit/meta.json
check only: no rebuild executed, no witness sidecars written
~~~

### Full Aspen1 witness rebuild

Pueue task: `1740`.

~~~text
rustc 1.98.0-nightly (df6ee909e 2026-06-28)
cargo 1.98.0-nightly (a335d47ff 2026-06-26)
openssl-pkgconfig-ok
witness request: /home/brittonr/mantle-witness/stabilized-self-hosting-witness-replay-2026-06-29/request
witness scratch root: /home/brittonr/mantle-witness/stabilized-self-hosting-witness-replay-2026-06-29/request.work
SNIX_BUILD_SANDBOX_SHELL: /home/brittonr/mantle-witness/stabilized-self-hosting-witness-replay-2026-06-29/bin/busybox
bwrap: /nix/store/p4zxfmj8r66x3y1gwlb3a15cz33nl6yv-mantle-witness-tools/bin/bwrap
witness rebuild completed: /home/brittonr/mantle-witness/stabilized-self-hosting-witness-replay-2026-06-29/request
release id: stabilized-self-hosting-witness-replay-2026-06-29
scratch root: /home/brittonr/mantle-witness/stabilized-self-hosting-witness-replay-2026-06-29/request.work
verification output: /home/brittonr/mantle-witness/stabilized-self-hosting-witness-replay-2026-06-29/request.work/release-verification/stabilized-self-hosting-witness-replay-2026-06-29
witness attestation: /home/brittonr/mantle-witness/stabilized-self-hosting-witness-replay-2026-06-29/request.work/release-verification/stabilized-self-hosting-witness-replay-2026-06-29/witnesses/aspen1.json
signature: /home/brittonr/mantle-witness/stabilized-self-hosting-witness-replay-2026-06-29/request.work/release-verification/stabilized-self-hosting-witness-replay-2026-06-29/witnesses/aspen1.json.sig
rebuild audit: /home/brittonr/mantle-witness/stabilized-self-hosting-witness-replay-2026-06-29/request.work/witness-rebuild-audit/meta.json
crunch-aspen1-1:u2hcpnqzst+Kq7V8KNlBeb6ko8TG3FTiU6kLUTsfb6M=
~~~

Copied returned artifacts locally under:

~~~text
target/aspen1-witness-stabilized-self-hosting-witness-replay-2026-06-29/
~~~

### Aspen1 witness audit summary

Source: `target/aspen1-witness-stabilized-self-hosting-witness-replay-2026-06-29/witness-rebuild-audit/meta.json`.

~~~json
{
  "schema": "mantle-witness-rebuild-audit-v1",
  "release_id": "stabilized-self-hosting-witness-replay-2026-06-29",
  "workflow_command": "./scripts/prove-self-hosting.sh",
  "workflow_version": "mantle-self-hosting-proof-v2",
  "status": "success",
  "rebuilt_outputs": [
    {
      "published_name": "binaries/01-stage2-mantle",
      "digest_blake3": "99d40790a4d356348b07b81be26fab12bb3f02270ba1e87adc60238a5279d328"
    }
  ],
  "failure_message": null,
  "diagnostics": null
}
~~~

### Aspen1 self-hosting proof summary

Source: `target/aspen1-witness-stabilized-self-hosting-witness-replay-2026-06-29/proof-bundle/summary.txt`.

~~~text
proof_mode: FixedPoint
selected_provider_kind: legacy-fetch
protected_exec_result: success
stage1_binary: 99d40790a4d356348b07b81be26fab12bb3f02270ba1e87adc60238a5279d328 binaries/stage1-mantle
stage2_binary: 99d40790a4d356348b07b81be26fab12bb3f02270ba1e87adc60238a5279d328 binaries/stage2-mantle
stage0_bwrap: 93cacbf9a523c439d160745228073492be1121b62074889345e676858fd1adf2 .../pys6ig7mb98pf53s7cq35iafz2wll35f-bwrap/bin/bwrap
stage0_busybox: ce321b321d0f06650ebbb24b834f4d2d600f5765d9f5828fb09c41a3ccd31ed5 .../wp1dyh7nl2ld4yl0ia730cdfm7js9mza-busybox/bin/busybox
stage2_bwrap: 93cacbf9a523c439d160745228073492be1121b62074889345e676858fd1adf2 .../pys6ig7mb98pf53s7cq35iafz2wll35f-bwrap/bin/bwrap
stage2_busybox: ce321b321d0f06650ebbb24b834f4d2d600f5765d9f5828fb09c41a3ccd31ed5 .../wp1dyh7nl2ld4yl0ia730cdfm7js9mza-busybox/bin/busybox
stage1_equals_stage2: true
stage0_bwrap_equals_stage2_bwrap: true
stage0_busybox_equals_stage2_busybox: true
stage1_embedded_store_paths: []
stage2_embedded_store_paths: []
stage0_hermeticity_mode: practical
stage2_hermeticity_mode: strict
stage2_fallback_events: []
~~~

### Import and final release verification

Command used locally with publisher key plus Aspen witness key:

~~~text
target/self-hosting-proof/run-20260629T140404Z-3929825/binaries/stage2-mantle attest witness-import \
  target/release-verification/stabilized-self-hosting-witness-replay-2026-06-29 \
  target/aspen1-witness-stabilized-self-hosting-witness-replay-2026-06-29/release-verification/stabilized-self-hosting-witness-replay-2026-06-29

target/self-hosting-proof/run-20260629T140404Z-3929825/binaries/stage2-mantle --json attest release-verify \
  target/release-verification/stabilized-self-hosting-witness-replay-2026-06-29 \
  --trusted-public-key 'crunch-britton-desktop-1:EZyF0JD3bJiHePrdN+XEnRsICArnYpQa29IQ+Lh9tco=' \
  --trusted-public-key 'crunch-aspen1-1:u2hcpnqzst+Kq7V8KNlBeb6ko8TG3FTiU6kLUTsfb6M='
~~~

Saved JSON: `target/aspen1-witness-stabilized-self-hosting-witness-replay-2026-06-29/final-release-verify.json`.

~~~json
{
  "release_attestation_digest": "635ab117644ca42869f5156b82a26487a8c395c7c899135146ffd34e6b5fa18f",
  "release_signer_key_name": "crunch-britton-desktop-1",
  "discovered_witness_count": 1,
  "considered_witness_count": 1,
  "technical_class": "external-witness-match",
  "policy_status": "satisfied",
  "final_class": "quorum-satisfied",
  "matching_witness_count": 1,
  "independent_witness_identities": 1,
  "revoked_witness_count": 0,
  "independent_agreement_status": "satisfied",
  "independent_agreement_class": "independent-rebuild-agreement",
  "independent_agreement_report_digest": "25d455b794770c2ae11dcf64ee99baae28ef789ed9d0a73ef12cf6fdf2f846b5",
  "independent_agreement_counted_witness_count": 1,
  "independent_agreement_skipped_witness_count": 0,
  "independent_agreement_failed_witness_count": 0,
  "independent_agreement_witnesses": [
    {
      "witness_identity": "aspen1",
      "signer_key_name": "crunch-aspen1-1",
      "signature_valid": true,
      "digest_match": true,
      "independence_domain": "aspen1",
      "policy_counted": true,
      "classification_reason": "counted"
    }
  ]
}
~~~

## Bounded non-claims

- This proves one Aspen1 witness for release `stabilized-self-hosting-witness-replay-2026-06-29`; it does not claim any other host or release.
- The release remains `selected_provider_kind: legacy-fetch`; this transcript proves independent replay agreement for the published artifact, not a source-root provider migration.
- Stage0 used practical hermeticity with recorded host bwrap/source-discovery fallback; stage2 used strict hermeticity with no fallback events.
