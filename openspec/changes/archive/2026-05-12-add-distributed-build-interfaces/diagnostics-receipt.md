# Distributed Diagnostics Receipt

Representative operator-facing receipts covered by `DistributedDiagnostic::receipt()`:

```text
cache-hit key=<12-char-key> resolver=local-cache
cache-miss key=<12-char-key> resolver=[REDACTED]
publish-skipped key=<12-char-key> publisher=publisher reason=[REDACTED]
local-realization key=<12-char-key> realizer=local
remote-candidate key=<12-char-key> worker=[REDACTED] verified=true
remote-fallback key=<12-char-key> reason=[REDACTED]
verification-rejected key=<12-char-key> reason=digest mismatch
```

Redaction rule: bearer-like tokens, `token=...`, authorization headers, secrets, and passwords are replaced with `[REDACTED]` before receipt text is exposed.
