# ADR 0032: Deny live source acquisition in hydrated fixed-point proofs

## Status

Accepted (2026-07-17)

## Context

ADR 0031 makes the ignored Cargo directory and pinned legacy-provider inputs portable to a fresh clone, but those three records are not the complete fixed-fetch closure evaluated by `bootstrap/bwrap.ncl`, `bootstrap/busybox.ncl`, and the Mantle self-build root. The existing fixed-point proof can therefore reach byte-identical stage1 and stage2 binaries while still acquiring additional URL or Git inputs live.

Environmental network blocking detects some missing inputs late, but it does not prove that Mantle refused fallback before URL parsing, proxy use, DNS, Git transport, or HTTP acquisition. Preloading fetch outputs into a store would also weaken the evidence by bypassing the fetch-service policy boundary and could smuggle unrelated producer build state.

## Decision Drivers

- Preserve the evaluated derivation URLs, revisions, hash modes, expected hashes, and store identities.
- Materialize actual source payload bytes on an explicit connected producer.
- Require independently authenticated source authority in the fresh clone.
- Give stage0 and stage2 fresh build state seeded only with the same authenticated source records and pin.
- Reject every unmatched builtin fetch before any live acquisition attempt.
- Retain path-redacted hydration, source-policy, stage, and binary-equality evidence.
- Preserve ADR 0031's exact three-record `fresh-clone-inputs` contract.

## Decision

Mantle adds the distinct `fresh-clone-fixed-point` source profile. A connected producer evaluates the selected self-build roots and uses `source bundle export --fetch-missing` to materialize every missing fixed URL or Git payload through Mantle's ordinary fetch implementation and fixed-output verification. Tarball records retain the compressed acquisition bytes rather than duplicating their expanded trees; consumers unpack those bytes with the same bounded extractor and rerun recursive fixed-output verification before the override is admitted. Payload files larger than 64 MiB use deterministic contiguous entries while every individual entry remains within the existing 64 MiB bound. The full profile combines those materialized records with the vendored Cargo record, unpacked legacy-provider archive, and runtime provider manifest. It does not replace or widen the established three-record profile.

The consumer hydrates that full profile with `hydrate-self-build` and an expected manifest BLAKE3 obtained independently from the bundle. The proof helper copies only `source-bundles/records` and `source-bundles/pins` from that source-only state into each fresh stage state. Both self-build invocations pass the same manifest authority.

`FetchBuildService` owns enforcement. `RequireOverride` mode requires an exact kind, URL, and Git revision match against the validated override plan. A miss returns a deterministic error before the ordinary fetch path is called. Successful stage proof lines bind the manifest BLAKE3, source-state BLAKE3, override count, `require-override` policy, and zero live-fetch events.

The proof bundle retains the contracted hydration report and emits `mantle-hydrated-fresh-clone-fixed-point-v1`, a path-redacted machine report identifying the source authority, staged source store identity, provider kind, platform, proof mode, stage policies, stage binary BLAKE3 digests, and fixed-point result.

## Alternatives Considered

### Preseed fixed-output store paths

Rejected because cache hits can bypass fetch-policy evidence and permit arbitrary producer build outputs to influence the proof.

### Rewrite fixed-fetch URLs to local files

Rejected because URL changes alter derivation fingerprints and weaken comparison with the committed bootstrap graph.

### Rely only on network namespaces or firewall denial

Rejected because it proves an environmental failure mode, not that Mantle structurally refused an undeclared acquisition attempt before transport.

### Add fixed-fetch records to `fresh-clone-inputs`

Rejected because that profile's exact three-record contract is already an accepted compatibility and review boundary. The stronger proof needs a distinct profile.

### Store expanded tarball trees in the handoff

Rejected because it duplicates large toolchain trees and makes single-file JSON handoffs needlessly expensive. Retaining acquisition bytes is smaller while replaying the same extractor and fixed-output verifier before use.

## Consequences

- A successful hydrated proof demonstrates byte-identical stage1 and stage2 binaries without live fixed-source fallback for the recorded evaluated closure.
- The connected export remains network-dependent and can fail on unavailable upstreams or fixed-output mismatch.
- Full source bundles remain large because the current JSON representation hex-encodes payload bytes, but compressed tarball acquisition bytes avoid duplicating expanded toolchain trees and deterministic chunks preserve the per-entry bound.
- The proof still trusts the legacy musl.cc seed provider, checkout stage0 binary, selected platform, and recorded tool boundary.
- This decision does not establish full-source bootstrap, compiler correctness, seed trust removal, bit-for-bit release reproducibility, independent rebuild agreement, deployment success, or full Cargo compatibility.
