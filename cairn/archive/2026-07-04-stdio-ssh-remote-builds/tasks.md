## Implementation

- [x] [serial] I1 Promote stdio and SSH-stdio command construction to supported remote-build client/server bindings with explicit argv, deadlines, and protocol-only stdout. r[remote_builds.stdio_ssh_hardened_bindings]
- [x] [serial] I2 Enforce cheap handshake ordering before store scans, input walks, sandbox setup, or local-build executor launch. r[remote_builds.stdio_ssh_hardened_bindings]
- [x] [serial] I3 Add bounded stderr/log capture, frame limits, timeout diagnostics, and phase-classified child/SSH failures. r[remote_builds.stdio_ssh_hardened_bindings]

## Verification

- [x] [serial] V1 Positive: local stdio child and SSH-stdio fixture carry the same framed request/response state machine and import a trusted output. r[remote_builds.stdio_ssh_hardened_bindings]
- [x] [serial] V2 Negative: stdout pollution, oversized input, invalid frame sequence, child exit failure, timeout, and capability mismatch fail before output import. r[remote_builds.stdio_ssh_hardened_bindings]
- [x] [serial] V3 Run focused stdio/SSH transport tests plus Cairn validate and proposal/design/tasks gates for this change. r[remote_builds.stdio_ssh_hardened_bindings]
