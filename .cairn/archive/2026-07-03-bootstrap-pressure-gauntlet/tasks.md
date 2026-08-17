## Implementation

- [x] [serial] I1 Define the bootstrap pressure profile ladder, verdicts, remaining-trusted-root fields, seed inventory bindings, and protected-exec audit bindings. r[verification_evidence.bootstrap_pressure_gauntlet]
- [x] [serial] I2 Implement runner/report plumbing for default, non-Nix-host, no-host-tools, source-built-provider, reduced-seed, and full-source-root-attempt profiles. r[verification_evidence.bootstrap_pressure_gauntlet]
- [x] [serial] I3 Surface source-root blockers and non-claims in release/global reproducibility summaries without promoting partial bootstrap proofs. r[verification_evidence.bootstrap_pressure_gauntlet]

## Verification

- [x] [serial] V1 Positive: run a declared-inventory no-host-tools or source-built-provider profile that records fixed-point status, audit digests, and remaining trusted root. r[verification_evidence.bootstrap_pressure_gauntlet]
- [x] [serial] V2 Negative: run missing-inventory and undeclared-host-exec fixtures and prove they fail closed before stronger bootstrap evidence is emitted. r[verification_evidence.bootstrap_pressure_gauntlet]
- [x] [serial] V3 Run focused self-build/protected-exec tests, bootstrap report checks, docs checks, and Cairn validate/gates for this change. r[verification_evidence.bootstrap_pressure_gauntlet]
