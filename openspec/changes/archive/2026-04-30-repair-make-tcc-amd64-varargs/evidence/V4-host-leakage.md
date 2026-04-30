Task-ID: V4
Covers: bootstrap.part.make.3.82.amd64.execution

Status: deferred
Deferred to: repair-make-tcc-amd64-varargs-runtime-validation

The derivation source was rechecked by V1 after the mirror fixes, but the full host-leakage scan also depends on the completed build transcript and output tree from V2. The local build timed out before producing that transcript/output, so this parent change cannot honestly claim final leakage proof.

The runtime-validation successor must scan the full build log and resulting output for undeclared host tools, host paths, and environment leakage after a successful or diagnostically useful long-running build.

Verified: 2026-04-30T23:47:11Z
