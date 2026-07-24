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

## Pending broad evidence

First-party quality, machine-contract, blocker, Cairn, Tracey, full Nix, and authenticated committed-source fixed-point evidence are recorded only after those exact final-source commands complete. No broad or fixed-point claim is made by this interim transcript.
