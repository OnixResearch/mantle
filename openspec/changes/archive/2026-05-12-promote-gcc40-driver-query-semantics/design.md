## Context

The GCC 4.0 artifact currently installs a shell driver that delegates compilation to TinyCC and reports a pass1 bridge version string. This is intentionally not a complete GCC, but downstream bootstrap stages still rely on normal GCC driver query behavior to discover target and support files.

## Goals / Non-Goals

**Goals:**
- Add exact behavior for a small set of read-only GCC driver query flags.
- Verify the query flags inside the derivation and with parity regression text checks.
- Preserve partial/blocking parity classification.

**Non-Goals:**
- Do not claim `cc1` is native or correct.
- Do not replace the TinyCC compilation delegation.
- Do not promote live-bootstrap/Guix parity to complete.

## Decisions

### 1. Bake installed paths into the pass1 driver

**Choice:** Generate the shell driver with concrete `$out` paths for libgcc/search-dir queries, while keeping runtime argument expansion escaped.

**Rationale:** This is deterministic and directly tied to the installed artifact; it avoids host probing and ambient path leakage.

### 2. Verify query flags before completing the derivation

**Choice:** Add shell checks after `libgcc.a` is written.

**Rationale:** `-print-libgcc-file-name` must point to an actually installed file, so the check belongs after archive creation.

## Risks / Trade-offs

- **False completeness:** The driver becomes more GCC-shaped but is still a bridge. Mitigation: keep parity status partial and add wording that query semantics are bounded evidence only.
- **Shell escaping:** Runtime `$1`/`$@` must stay escaped in the generated script. Mitigation: syntax-check generated derivation script and build the derivation.
