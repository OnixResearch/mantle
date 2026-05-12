## Context

The GCC 4.0 stage now builds through a large native object graph and then installs a pass1 bridge. The build log shows progress through `cc1`, `gcov`, `collect2`, `xgcc`, `cpp`, CRT objects, and `libgcc.mk`; the remaining parity status is still partial because this is not native GCC correctness.

## Goals / Non-Goals

**Goals:**
- Make the current native-boundary claim machine-checked.
- Fail closed if the receipt omits the exact derivation markers that define the boundary.
- Preserve the `gcc.4.0` partial status.

**Non-Goals:**
- Claim native GCC completion.
- Replace another generator/header stub in this increment.
- Run an unbounded full bootstrap drain.

## Decisions

### 1. Separate receipt, checked from the GCC evidence gate

**Choice:** Add `bootstrap/evidence/gcc-4.0-native-boundary.json` and validate it from the existing `Gcc40PlaceholderInventory` parity gate.

**Rationale:** The GCC row already has a dedicated evidence gate. Extending that gate keeps the row partial but upgrades its evidence from marker inventory only to marker inventory plus native-boundary receipt.

**Alternative:** Add a new parity row. Rejected because this is evidence for the existing `gcc.4.0` row, not a separate bootstrap stage.

## Risks / Trade-offs

**Receipt can become stale** → Fail closed when required markers disappear or drift.

**Overstating progress** → Receipt status remains `boundary-only`; parity remains partial until native compiler correctness is proven.
