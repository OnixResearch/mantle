# Protected transition v84 Flex mode boundary

## Status

Diagnostic only. The transition did not complete and does not authorize provider publication.

## Command

Pueue task `3845` ran the exact protected transition test with source bundle `stagex-source-closure-v22-diffutils-20260728.json`, manifest digest `4ad1f3b14dfa219fc64faf752dd0050e678f293af7828a65262f510cc4228e2a`, and scratch root `stagex-protected-transition-v84-provider-roles-final-20260730`.

## Result

```text
called `Result::unwrap()` on an `Err` value: ProtectedExec("GNU Flex runtime failed: reading Flex executable: Permission denied (os error 13)")

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1941 filtered out; finished in 1341.16s
```

The failure occurred before Flex execution. TinyCC produced the linked file without owner-read permission. The new bounded section-name canonicalizer tried to read it before the existing final `0755` mode normalization.

## Repair

The canonicalization shell now sets the linked file to exact mode `0644` before reading or rewriting it. After canonicalization, the existing link shell sets exact mode `0755` before any protected execution.

A positive shell test starts with mode `000`, runs canonicalization, requires exact mode `0644`, and checks that a second pure canonicalization is byte-identical. The existing negative core test still rejects changed runtime bytes.

This evidence proves only the diagnosed mode boundary. It does not prove transition completion or provider admission.
