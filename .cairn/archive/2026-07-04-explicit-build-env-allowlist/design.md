## Context

Build environment construction currently spans user declarations, deterministic defaults, sandbox internals, and convenience escape hatches. To improve hermeticity, the semantic decision about which variables are accepted should be a pure functional core, while the shell only reads ambient values for explicit diagnostic or practical-mode paths.

## Decisions

### 1. Environment policy is data

**Choice:** Model allowed, required, normalized, denied, and redacted variables as explicit policy data consumed by a pure normalizer.

**Rationale:** Reviewable policy avoids hidden string literals and makes positive and negative fixtures stable.

### 2. Strict mode never inherits by default

**Choice:** Strict builds construct the child environment from declared build env entries plus named deterministic defaults such as locale, timezone, and temp roots.

**Rationale:** Ambient inheritance is the leak. The only safe strict default is no inheritance unless a variable is deliberately modeled.

### 3. Secret-like variables are diagnostic-only

**Choice:** Denied secret-bearing variables produce redacted diagnostics and never appear in receipts, logs, or child environments.

**Rationale:** Hermeticity evidence must not become a token exfiltration channel.

## Risks / Trade-offs

- Some builders relying on undeclared ambient variables will fail in strict mode.
- Compatibility flags must be visibly impure or practical-only so they cannot satisfy strong claims.
