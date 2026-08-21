# Tasks: Update cairn input

## Update

- [x] [serial] Pin the cairn root input at revision 695124d and expose its package in the dev shell. r[mantle.cairn_input.pinned]
- [x] [serial] Verify in-repo validation passes and record gate evidence. r[mantle.cairn_input.validation]

## Verification Coverage

- `Scenario: Dev shell resolves the pinned binary` -> PATH resolution check
- `Scenario: Lifecycle commands parse the committed policy` -> validate run
