## Implementation

- [x] [serial] I1 Add a pure search-path plan model that maps PATH entries to declared tool refs, alias views, or accepted host-tool inventory records. r[build_correctness.receipt_bound_path]
- [x] [serial] I2 Construct strict-mode PATH from the search-path plan and reject ambient, duplicate-conflicting, or unclassified entries before execution. r[build_correctness.receipt_bound_path]
- [x] [serial] I3 Record the search-path digest, alias map, and real tool refs in build/proof reports and operator docs. r[build_correctness.receipt_bound_path]

## Verification

- [x] [serial] V1 Positive: run a strict fixture that resolves tools only through declared PATH entries and records stable tool refs. r[build_correctness.receipt_bound_path]
- [x] [serial] V2 Negative: poison PATH with an undeclared executable and drift an alias target, and prove strict mode fails closed before accepting output. r[build_correctness.receipt_bound_path]
- [x] [serial] V3 Run focused PATH-planning/protected-exec tests plus Cairn validate and proposal/design/tasks gates for this change. r[build_correctness.receipt_bound_path]
