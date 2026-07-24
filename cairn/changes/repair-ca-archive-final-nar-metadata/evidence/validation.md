# Validation evidence

## Identity split and runtime behavior

Mantle now treats CA path identity and final payload identity as separate facts. Single- and multi-output CA finalization derives each logical store path from the marker-normalized NAR SHA-256, rewrites the marker to the final path, freshly renders the rewritten node, and persists that final NAR size/SHA-256 while retaining the marker hash only in `PathInfo.ca`.

The single-output runtime fixture writes its provisional `$out` value as the output bytes. After finalization it proves that the stored bytes equal the resolved logical store path, the recorded NAR facts equal a fresh render, and the CA hash differs. The same assertions pass for two independently rewritten outputs. The single-output test then exports that builder-origin signed PathInfo and imports it into a fresh store under the trusted test key, proving the corrected metadata round-trips through the real archive boundary.

Archive export now fresh-renders every selected closure node during complete planning, before archive magic. Import validates supported CA metadata as independent path identity, calls NAR ingestion without treating that CA as the expected final content hash, and still requires payload BLAKE3, final NAR SHA-256/size, exact reconstructed node, store prefix, signature, and trust-policy acceptance before persistence. An identical existing local PathInfo/content is freshly checked against its final NAR facts before the cheap skip path is admitted.

## Focused positive and negative tests

Task `437` ran the selected CA tests after the final-NAR repair:

```text
test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 625 filtered out; finished in 0.02s
```

Tasks `425` and `429` ran the archive regression set and the explicit ordinary reference-aware CA-path check:

```text
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 208 filtered out; finished in 0.04s
test archive::tests::ca_path_identity_accepts_reference_aware_standard_path ... ok
```

The archive set includes positive marker-CA/final-NAR round trip and negative stale export, stale existing local hit, wrong CA, unsupported CA, conflicting local metadata, wrong prefix, untrusted signature, truncated payload, and tampered payload cases. The stale export assertion requires the writer to remain empty; failed import paths assert no new PathInfo persistence.

Final package runs were current after all implementation edits:

```text
$ nix develop -c cargo test -p crunch-build --lib --tests
 test result: ok. 648 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.91s
 test export_api_is_public ... ok

$ nix develop -c cargo test -p crunch-store --lib --tests
 test result: ok. 225 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
```

Task `170` passed focused strict Clippy for `crunch-build` and `crunch-store`, both complete package suites, and the operator CLI archive suite:

```text
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
```

## Retained stale-state migration boundary

The retained signed Bison PathInfo cannot be silently repaired because its signature binds the stale NAR size/SHA-256. Running the repaired CLI against that exact state and selector now fails during export planning, before archive magic or payload:

```text
exit_status=3
archive_bytes=0
error: archive export: export: stale final NAR facts for 57dqxg2kjddkvjmwkvkr54nifqfs38l0-bison-2.3-gcc-v6: recorded size 972064 sha256 fff5e607c24805da4403a7619a5a967b61d0ea3fca268103773a47bde813f3eb, observed size 972064 sha256 c4bc724b6e5f92047cc57e6d9da09a71eb6c46110fac04254eba87062f1c6738
```

This is the intended rebuild-or-future-explicit-migration boundary. No PathInfo is rewritten or resigned by export/import.

## Broad final-source gates

Pueue task `424` ran the final committed-source first-party quality, machine-contract, blocker, Cairn, and Tracey chain after the GCC heredoc repair. It completed successfully with:

```text
machine schema contract self-test: PASS
machine schema contract check: PASS (21 contracted, 50 classified)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
bootstrap blocker inventory: 0 findings across 0 classes, 437 evidence-backed suppressions, 0 promotion claims, enforce=true
"valid": true
traceability coverage ok: 145/145 referenced (profile mantle-default)
```

The exact first-party quality script completed Rustfmt, strict Clippy, and serialized first-party tests. Pueue task `425` then ran build-mode `nix flake check -L` from commit `8744faa7`. The unavailable SSH builder warning fell back locally; the complete host-compatible result was:

```text
mantle-nextest>      Summary [  78.848s] 4048 tests run: 4048 passed, 8 skipped
mantle> test result: ok. 1596 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.71s
all checks passed!
warning: The check omitted these incompatible systems: aarch64-darwin, aarch64-linux, x86_64-darwin
```

The first full-Nix attempt had exposed two Tiger Style findings in the new pure CA identity helper (assertion density and predicate naming). Commit `3726bea5` made those invariants explicit; the focused Tiger Style derivation passed before the final full run.

## Authenticated committed-source fixed point

Pueue task `259` was a real failed proof attempt, not evidence of success. It exposed premature outer-heredoc command substitution in the separately archived GCC configure bridge and failed at `gcc-4.0.4-native-gas-v45.drv` after 3,883.01 seconds. Commit `8744faa7` deferred `pwd`, `readlink`, `dirname`, `wc`, and count-file `cat` to the authorized configure child and added checker/test negatives for eager substitution.

After final broad gates, task `427` ran the authenticated offline fixed-point proof from clean commit `8744faa7`. The exact pueue log result was:

```text
test self_hosting_stage0_stage1_stage2 ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 55 filtered out; finished in 11163.89s
```

Proof bundle: `/home/brittonr/.cache/mantle-full-source-proof-20260723/proof-bundle-v9-ca-archive`.

Exact bound identities from `fresh-clone-fixed-point.json`:

```text
expected_manifest_blake3=edfe4135f4573f680dfcfcd87ea6fe592c8575c41d095a1203c953cbcc5c4fa0
source_state_blake3=4142c316fae5eba69259bfa7ddeaf3f49df917c90b210e18d68f558921e930ed
hydration_report_blake3=7d993e60dc0220c4e99907bafb262dca6a5e49a7ccf96e6f5209b821e8e6c120
provider_kind=full-source
stage0_source_policy=require-override
stage0_source_override_count=61
stage0_live_fetch_events=0
stage0_hermeticity_mode=practical
stage0_fallback_event_count=2
stage2_source_policy=require-override
stage2_source_override_count=61
stage2_live_fetch_events=0
stage2_hermeticity_mode=strict
stage2_fallback_event_count=0
stage1_binary_blake3=eac97c4e997b3ab498be10156f6b58caebf812da6af4112f08e7ad57777f138c
stage2_binary_blake3=eac97c4e997b3ab498be10156f6b58caebf812da6af4112f08e7ad57777f138c
fixed_point=true
```

The archive-stable `bootstrap/evidence/full-source-provider-fixed-point.json` is byte-identical to that generated report. This proves one hydrated fixed point for the recorded provider, source authority, and platform; it does not prove compiler correctness, bootstrap-seed correctness, independent rebuild agreement, release reproducibility, deployment success, or external archive compatibility.
