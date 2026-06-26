## Context

The latest provider-backed proof bundle is:

```text
/home/brittonr/git/mantle-source-built-rust-provider-fixed-point-snix-sandbox-shell-2026-06-26
```

Its stage1 receipt contains 660 unit executions and one failed unit:

```text
native:4e60377238cb00c19fb50f30e49935e5a711ee4f71876ecf9b7f92ce579e95f0:path+native#mantle@0.1.0:mantle:bin:build
```

The blocker class is `rustc-failed`, and the receipt diagnostics name unused local items visible to the binary build. These warnings are not proof of program failure; they are hygiene leaks from test-only code and intentionally dormant feature surfaces into non-test compilation.

## Decisions

### 1. Keep test-only helpers in test scope

**Choice:** Move test-only imports/constants into `#[cfg(test)]` scope instead of adding broad allow attributes for them.

**Rationale:** Non-test provider topology compilation should not see items used only by tests. Scoped test helpers remove those warnings at their source and do not weaken lint behavior.

### 2. Use scoped dormant-code allowances for intentionally parked surfaces

**Choice:** Add item- or module-scoped `dead_code` allowances only for surfaces that are deliberately compiled for near-term provider/front-end/native-planner work but not wired into the current binary path.

**Rationale:** The provider proof should reveal real topology/link/runtime blockers, not stop at dormant-code warnings. The allowances are local to named dormant surfaces rather than global crate lint suppression.

### 3. Keep provider proof claims bounded

**Choice:** After the hygiene fix, rerun the real provider-backed fixed-point proof and record either success or the next deterministic blocker.

**Rationale:** This change should not claim fixed-point success from local checks alone. The proof bundle remains the authority for whether the provider-backed frontier moved.

## Risks / Trade-offs

- The provider proof may reveal another deterministic blocker after local warnings are removed; that is expected and must be recorded as the current frontier.
- Removing or moving test-only helpers must preserve the existing unit tests that used those helpers.
- Scoped dormant-code allowances can hide future unused-code drift inside those surfaces; keep them local and do not convert them into crate-wide suppression.
