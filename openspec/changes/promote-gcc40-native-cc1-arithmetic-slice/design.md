## Context

The current GCC 4.0 row has substantial bounded evidence: libgcc member semantics, driver query semantics, installed `cc1` object-output bridge behavior, generator boundary receipts, demangle semantics, placeholder inventory, and a native-boundary receipt. The native-boundary receipt still names `cc1-tcc-delegation` as a blocker, meaning the compiler frontend path remains a bridge rather than native GCC correctness.

## Goals / Non-Goals

**Goals:**
- Promote exactly one bounded native `cc1` arithmetic/control-flow slice.
- Add machine-checkable evidence that the selected smoke is not emitted by TinyCC delegation.
- Preserve partial/blocking parity semantics until full GCC 4.0 correctness is proven.

**Non-Goals:**
- Full native GCC 4.0 completion.
- Full C language frontend correctness.
- Replacing all generator empty-boundary outputs.
- Unblocking live-bootstrap or Guix parity in this slice.

## Decisions

### 1. Target installed `cc1`, not another libgcc member

**Choice:** The slice focuses on installed `cc1` compiling a bounded C function with integer arithmetic, comparison, branch, and return behavior.

**Rationale:** The explicit frontier blocker is `cc1-tcc-delegation`; another libgcc/member or generator boundary would not reduce the main compiler-frontier uncertainty.

**Alternative:** Promote another libgcc helper or generator boundary. Rejected for this step because those are lower unlock now than reducing the `cc1` delegation frontier.

### 2. Require no-TinyCC-delegation evidence

**Choice:** The receipt must prove the bounded smoke did not use the installed TinyCC delegation path, either by removing that path for the slice or by recording a deterministic transcript/marker that excludes it.

**Rationale:** Without this guard the slice could be another bridge smoke and would not improve native compiler evidence.

### 3. Keep parity partial

**Choice:** Even a passing native arithmetic slice only upgrades evidence quality; it does not complete `gcc.4.0`.

**Rationale:** Generator correctness, broader C frontend behavior, and full native compiler self-host evidence remain open.

## Risks / Trade-offs

**Implementation may reveal a deeper native `cc1` blocker** → Record the precise frontier and fail closed rather than weakening the no-delegation requirement.

**Smoke could be too broad** → Keep the C input tiny: a single function exercising arithmetic, comparison, branch, and return.

**False native claim risk** → Negative tests must cover TinyCC delegation, stale receipt markers, unsupported schema, and parity overclaiming.
