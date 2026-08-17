## Implementation

- [x] [serial] I1 Define the strict hermeticity regression suite inventory with declared cases, expected verdicts, blocker/audit classes, and unsupported-host non-claims. r[verification_evidence.strict_hermeticity_regression_suite]
- [x] [serial] I2 Add isolated positive and negative fixtures for environment, PATH, host execution, network, closure facts, umask, temp-root, and nondeterminism boundaries. r[verification_evidence.strict_hermeticity_regression_suite]
- [x] [serial] I3 Emit deterministic suite evidence and documentation that bounds readiness claims to the axes that ran. r[verification_evidence.strict_hermeticity_regression_suite]

## Verification

- [x] [serial] V1 Positive: run the clean strict fixture and prove it records accepted hermeticity evidence and stable output digests. r[verification_evidence.strict_hermeticity_regression_suite]
- [x] [serial] V2 Negative: run each poisoned fixture and prove strict mode fails closed or reports the required blocker without corrupting store state. r[verification_evidence.strict_hermeticity_regression_suite]
- [x] [serial] V3 Run the suite report checker plus Cairn validate and proposal/design/tasks gates for this change. r[verification_evidence.strict_hermeticity_regression_suite]
