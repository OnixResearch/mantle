## ADDED Requirements

### Requirement: grep 2.4 runtime validation remains explicit [r[bootstrap.part.grep.2.4.runtime-validation]]
The system MUST keep grep 2.4 runtime proof incomplete until a completed build transcript, output contract smoke test, and leakage scan are recorded.

#### Scenario: Long-running prerequisite build [r[bootstrap.part.grep.2.4.runtime-validation.long-build]]
- **GIVEN** Crunch needs to build prerequisite bootstrap inputs for `bootstrap/grep-2.4-musl.ncl`
- **WHEN** the validation run exceeds a short drain timeout
- **THEN** the runtime proof remains in this follow-up change rather than being silently treated as complete

#### Scenario: Output contract is proven [r[bootstrap.part.grep.2.4.runtime-validation.output-contract]]
- **GIVEN** `bootstrap/grep-2.4-musl.ncl` builds successfully
- **WHEN** the produced output is smoke-tested
- **THEN** `grep`, `egrep`, and `fgrep` are present and usable without undeclared host fallback
