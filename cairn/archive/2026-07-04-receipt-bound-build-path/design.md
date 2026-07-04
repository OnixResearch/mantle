## Context

Path search behavior is both an execution concern and a proof concern. The core needs a deterministic view of executable authority: the ordered entries, their source declarations, and the real tool identities behind aliases. The imperative shell should only materialize wrapper files and pass the normalized PATH to the child.

## Decisions

### 1. PATH is derived from tool refs

**Choice:** Strict-mode PATH entries are generated from declared tool outputs or accepted host-tool inventory records, never from the parent process PATH.

**Rationale:** This makes executable resolution reviewable and receipt-bound.

### 2. Aliases do not replace identity

**Choice:** Stable executable aliases are represented as convenience views that point to real content-addressed tool refs or attested host executables.

**Rationale:** Alias names are useful inside sandboxes but cannot be allowed to hide tool drift.

### 3. Path poisoning is a preflight blocker

**Choice:** Strict preflight rejects ambient or unclassified PATH entries before sandbox execution.

**Rationale:** Failing before execution avoids accidentally using the wrong program and then trying to infer damage afterward.

## Risks / Trade-offs

- Some practical workflows may need an explicit impure PATH escape hatch.
- Wrapper materialization must be deterministic and avoid embedding scratch paths into proof identity.
