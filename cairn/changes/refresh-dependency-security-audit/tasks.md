## Implementation

- [ ] [serial] I1 Run the dependency/security audit with the checked-in policy and record command, policy path or digest, and tool version when available. r[verification_evidence.dependency_security_audit]
- [ ] [serial] I2 Classify every finding as fixed, accepted waiver, upstream-blocked, action-required, or tooling/config issue. r[verification_evidence.dependency_security_audit]
- [ ] [serial] I3 Apply safe dependency, lockfile, waiver, or documentation updates supported by the audit evidence. r[verification_evidence.dependency_security_audit]
- [ ] [serial] I4 Record a concise evidence transcript that distinguishes clean results from accepted or upstream-blocked residual risk. r[verification_evidence.dependency_security_audit]

## Verification

- [ ] [serial] V1 Positive: rerun the audit rail after updates and record the exact result or classified residual findings. r[verification_evidence.dependency_security_audit]
- [ ] [serial] V2 Negative: run or fixture the missing-policy/default-policy case and assert it is not accepted as audit evidence. r[verification_evidence.dependency_security_audit]
- [ ] [serial] V3 Positive: run lockfile/manifest consistency checks relevant to any dependency updates. r[verification_evidence.dependency_security_audit]
- [ ] [serial] V4 Run `git diff --check`, Cairn validation, and Cairn proposal/design/tasks gates. r[verification_evidence.dependency_security_audit]
