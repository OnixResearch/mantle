# Design: two-output release witness rebuild

## Problem

`run_witness_rebuild_workflow()` delegates to `scripts/prove-self-hosting.sh`, which writes a self-hosting proof bundle containing `manifest.json` and durable binary copies under `binaries/stage1-mantle` and `binaries/stage2-mantle`. `collect_rebuilt_output_paths()` only reads `stage2.report.output_binary` and rejects any request whose release manifest has more than one expected output.

The refreshed provider-bound release intentionally has two outputs:

1. `binaries/01-mantle`: the provider fixed-point Mantle binary.
2. `binaries/02-stage2-mantle`: the classic self-hosting stage2 Mantle binary.

The helper must not fabricate evidence, but it can accept a proof bundle output only when the proof manifest already records an exact BLAKE3 digest matching the release output.

## Approach

Keep the imperative shell unchanged: it runs the workflow, writes logs, and asks the pure-ish collection/validation helpers which output paths are admissible.

Update the collection core to:

- parse the proof bundle manifest with enough binary digest/path data for `stage1`, `stage2`, and the legacy `stage2.report.output_binary` path;
- build a bounded list of candidate proof artifacts from the proof manifest and bundle-local binary copies;
- match each expected release output by BLAKE3 digest, preserving release manifest order;
- fail closed if an expected digest has no candidate, matches more than one distinct path, escapes the proof bundle when bundle-relative, or points at a missing file;
- return one rebuilt output path per expected release output so existing `validate_rebuilt_output_digests()` and `create_witness_attestation()` continue to verify the final digest set.

The helper remains conservative: a same digest can satisfy multiple published names only if the release itself publishes duplicate digest entries, and each returned path is validated again by digest before signing.

## Tests

Add focused unit tests in `src/witness_rebuild.rs`:

- positive: a proof manifest with `binaries.stage1` and `binaries.stage2` candidates collects two expected outputs in release order;
- positive/backward compatibility: a one-output request still collects the legacy `stage2.report.output_binary` candidate;
- negative: an expected digest absent from the proof candidates fails closed with a deterministic diagnostic;
- negative: a proof candidate that escapes the proof bundle fails closed.

These tests exercise the functional core without running the expensive witness workflow. Existing CLI tests still cover witness sidecar creation/import/release-verify.
