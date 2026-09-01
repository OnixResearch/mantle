# V98 bootstrap-parity promotion

## Verdict

The compatibility row `crunch.self-build` now consumes the archived V98
source-built Mantle fixed-point evidence. All three bootstrap parity axes pass
when explicitly required:

```text
live-bootstrap: complete
Guix: complete
StageX: complete
```

This is a bounded promotion over the recorded V98 source, lineage, providers,
platform, deterministic receipt, and root action trust evidence.

## Architecture

`src/source_built_parity_promotion.rs` is the functional core. It validates
content and cross-file meaning without filesystem access. It checks:

- deterministic proof schema, verdict, provider, source, vendor, rebuild, and
  output identities;
- two matching clean stage outputs with no authority, hermeticity, or
  substitution findings;
- five action adapters and checked action/event sums;
- exact root reconciliation with zero unknown, missing, authority, fallback,
  remote, or cache-only findings;
- strict local execution;
- proof, plan, reconciliation, and trust-report cross-links.

`src/source_built_parity_promotion_shell.rs` is the imperative shell. It permits
only bounded regular no-follow files under safe repository-relative paths,
checks exact BLAKE3 identities, parses JSON, and invokes the core.

`scripts/check-source-built-parity-promotion.rs` is the standalone entry point.
It imports the same core and shell instead of duplicating policy.

## Bound evidence

`bootstrap/evidence/real-self-build-proof-parity.json` binds:

- V98 deterministic receipt file BLAKE3
  `dabb7fae2b1f9de161ffa08b6237af460f82485c89bde039586c81efc1ab0e4b`;
- root action plan file BLAKE3
  `2fbd10e2374f8d2342962f0e3f447de20b8368b081dc3436e11ef0dc87fa25ce`;
- root reconciliation file BLAKE3
  `e134a481118373526cf3dfdbd97fc00d3b6c2d7a8e3fef6073a50e341545f10f`;
- complete trust report file BLAKE3
  `d48728409158ee368923e3bd59934518cfb90a018482b177f34b73d7bf36bfcc`.

The verified semantic result contains:

- provider kind `full-source`;
- deterministic receipt BLAKE3
  `a6f5e378a74d8e2e3f148b7f59744d3ad0b307e2cda2cbbc5515a3b89b956a33`;
- matching output BLAKE3
  `7d166e10df71f46a4031a63abf05fc735e7663183998b311bc4aa2ead9408c9c`;
- 1,914 planned and matched actions;
- 478,870 observed and matched events;
- local-only execution and zero blockers.

## Validation

Passed:

- promotion core and path boundary: 4 tests;
- focused bootstrap parity core: 91 tests;
- bootstrap parity CLI: 19 tests;
- trust-report CLI: 3 tests;
- standalone positive check;
- standalone digest-tamper and path-escape self-tests;
- Rust formatting;
- `git diff --check`;
- `parity-report --require live-bootstrap --require guix --require stagex`.

The broad first-party Clippy command still stops on the recorded repository-wide
warnings. `clippy.log` contains no finding in either new promotion module.

## Portfolio and adversarial review

Three correlated routes were checked:

1. trusting the old compact v1 descriptor was rejected because it lacked V98
   root action trust;
2. trusting only the complete trust-report status was rejected because it did
   not independently bind the archived files;
3. exact file bindings plus pure cross-link validation survived positive and
   tamper tests.

The validator rejects malformed BLAKE3, absolute or parent paths, non-regular or
symlink files, file digest drift, provider/source/vendor mismatch, output drift,
incomplete adapters, count drift, unknown or missing actions, authority or
fallback events, remote execution, cache-only completion, and nonlocal evidence.

## Review checkpoint

- Question: Can V98 close the compatibility self-build row without trusting a
  status field or an absolute proof path?
- Inspected evidence: archived V98 receipt, action plan, reconciliation, trust
  report, provider linkage, parity report, positive tests, and mutation tests.
- Decision: bind four repository-relative files by BLAKE3 and validate their
  cross-links through one shared pure core.
- Owner: `promote-full-bootstrap-parity`.
- Next action: integrate this promotion, then complete the remaining exportable
  per-action bundle and machine-contract lifecycle tasks.

## Non-claims

The standalone promotion checker validates the archived root plan,
reconciliation, and trust-report linkage. It does not reconstruct every original
provider or StageX event from scratch. It does not prove compiler or verifier
soundness, seed correctness, kernel isolation, independent rebuild agreement,
universal release reproducibility, deployment success, or full Cargo
compatibility.
