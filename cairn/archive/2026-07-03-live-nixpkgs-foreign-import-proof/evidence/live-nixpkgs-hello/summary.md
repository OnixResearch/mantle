# Live Nixpkgs hello proof summary

- Question: can a live host-Nix export of `nixpkgs#hello` be lowered, validated, and planned by Mantle without Nix during consumption?
- Inspected evidence: `transcript.md`, `produce-report.json`, `validate-report.json`, `plan-report.json`, and generated artifacts under `artifacts/`.
- Decision: yes for admitted/planned state only.
- Owner: Mantle foreign derivation import.
- Next action: real cache.nixos.org substitution proof; local rebuild compatibility remains separate.

## Facts

- Host Nix drv path: `/nix/store/m74651b793zgyyvlk9gx7v1cl1ywslib-hello-2.12.3.drv`
- Derivation JSON bytes: 1892859
- Graph artifact bytes: 3412207
- Package-index bytes: 415
- Validate accepted: true
- Plan accepted: true
- Plan forbidden process invocations: 0
- Substitution audit entries: 1
- Receipt hash-domain records: 954

## Non-claims

No real cache substitution, no local rebuild compatibility, no output trust, no package correctness, and no reproducibility are claimed by this proof.
