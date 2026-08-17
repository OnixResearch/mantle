## Why

`PATH` is executable authority. If build or proof paths search ambient host directories, an undeclared tool can change outputs or proof outcomes while leaving receipts looking clean. Strict builds should search only declared tool outputs or explicitly attested host-tool wrappers, and receipts should bind the real tool identities behind any stable aliases.

## What Changes

- Generate strict-mode `PATH` entries only from declared tool references or attested host-tool inventory records.
- Record alias wrappers separately from the real content-addressed tool refs they expose.
- Reject ambient or poisoned `PATH` entries before strict execution.
- Include the normalized search-path digest and tool ref list in build/proof evidence.

## Impact

- **Files**: action planning, build-request PATH construction, host-tool/protected-exec integration, reports, docs, and Cairn build-correctness spec delta.
- **Testing**: positive declared-tool PATH fixture; negative ambient PATH poisoning and alias-drift fixtures; Cairn validation and gates.

## Out of Scope

- Removing explicit impure diagnostics paths.
- Making shell activation profiles strict proof evidence.
