## Implementation

- [x] [serial] I1 Define adversarial hermeticity profiles and pure expected-outcome classification for host-tool, env, network, time, locale, umask, temp-path, store-path, and randomness perturbations. r[verification_evidence.adversarial_hermeticity_gauntlet]
- [x] [serial] I2 Add isolated gauntlet fixtures and runner/report plumbing for strict and practical hermeticity modes. r[verification_evidence.adversarial_hermeticity_gauntlet]
- [x] [serial] I3 Integrate gauntlet blocker classes with release/global reproducibility evidence boundaries. r[verification_evidence.adversarial_hermeticity_gauntlet]

## Verification

- [x] [serial] V1 Positive: run a clean strict-mode fixture that emits accepted hermeticity evidence and stable output digest evidence. r[verification_evidence.adversarial_hermeticity_gauntlet]
- [x] [serial] V2 Negative: run fixtures that attempt undeclared host exec, network access, ambient-env leakage, and temp-path dependence, and prove strict mode fails closed or records blockers. r[verification_evidence.adversarial_hermeticity_gauntlet]
- [x] [serial] V3 Run focused hermeticity tests, protected-exec tests where supported, report checks, and Cairn validate/gates for this change. r[verification_evidence.adversarial_hermeticity_gauntlet]
