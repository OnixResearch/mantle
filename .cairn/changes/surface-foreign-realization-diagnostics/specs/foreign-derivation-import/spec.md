## ADDED Requirements

### Requirement: Content-admission refusals name their cause

r[foreign_derivation_import.content_admission_diagnostics] The HTTP NAR ingestion shared by the
binary-cache pull paths MUST return every refusal as a value carrying a stable refusal class and a
bounded detail string, not only a missing ingestion and a tracing line. The strict seeded closure
import MUST report a refused member as
`http-closure-content-admission-failed: <path>: <class>: <detail>`. The class MUST be a stable
kebab-case token that distinguishes at least a rejected NAR URL, a rejected redirect, a non-success
HTTP status, a request error, unsupported compression, and a NAR ingestion failure. The detail MUST
include the HTTP status when a status caused the refusal, and MUST be truncated on a UTF-8 boundary
to a named `MAX_*` byte limit. The detail MUST name the NAR by its narinfo-relative locator, not by
the resolved absolute URL, and MUST NOT contain the cache URL's userinfo, query tokens, or another
absolute URL, because the detail reaches JSON errors and receipts; HTTP client errors MUST be
rendered without their URL. Existing tracing warnings MUST remain.

The flat HTTP pull path MUST keep its skip-and-count behavior and counters, including for a refusal
that now carries a class and detail. A successful ingestion MUST produce the same PathInfo,
report, and counters as before.

#### Scenario: A status refusal names its class and detail

- GIVEN a seeded closure member whose NAR route answers with HTTP 503
- WHEN the strict closure import pulls that member
- THEN it MUST fail with `http-closure-content-admission-failed`
- AND the message MUST name the member path, the `nar-http-status` class, and the HTTP status.

#### Scenario: An ingestion refusal names its class and detail

- GIVEN a seeded closure member whose NAR bytes are not the signed NAR
- WHEN the strict closure import ingests that member
- THEN it MUST fail with `http-closure-content-admission-failed`
- AND the message MUST name the candidate path, the `nar-ingestion-failed` class, and a non-empty
  bounded detail.

#### Scenario: A refusal never echoes cache credentials

- GIVEN a cache URL carrying userinfo and a query token, and a member whose NAR route fails
- WHEN the strict closure import refuses that member
- THEN the message MUST name the refusal class and the narinfo-relative NAR locator
- AND it MUST NOT contain the userinfo, the query token, or an absolute cache URL.

#### Scenario: The flat pull still skips and counts

- GIVEN a flat HTTP pull request whose NAR response has a non-success status
- WHEN the pull runs
- THEN the path MUST be skipped and counted exactly as before
- AND the pull MUST NOT return an error for the refusal.

#### Scenario: A successful import is unchanged

- GIVEN a seeded closure whose members are served intact
- WHEN the strict closure import runs
- THEN every member MUST be admitted with its declared counters and PathInfo as before.

### Requirement: Successful realizations keep their build logs

r[foreign_derivation_import.realization_build_logs] `foreign-import realize` MUST persist the
captured build log of every executed unit outcome that has one, and the retained failure log of
every failed selected root that has one, to `<state-dir>/logs/<drv>.log` using the same file name,
header format, and writer as `mantle build`. An outcome served from cache or reuse without a log
MUST write nothing and MUST NOT overwrite an existing log.

A log-write failure MUST NOT change the realization result, the receipt, the published receipt
bytes, or the built-root index. It MUST be reported as a diagnostic persistence failure the way
`mantle build` reports one, and `--json` stdout MUST remain the receipt alone with no additional
prose on stderr. No receipt field or schema version MUST change.

#### Scenario: A built root keeps its log

- GIVEN a `cache-inputs-build-roots-v2` plan whose selected root is absent locally
- WHEN realization builds the root successfully
- THEN `<state-dir>/logs/<root derivation basename>.log` MUST exist
- AND it MUST contain the builder's captured output
- AND `mantle --state-dir <state-dir> log <drv>` MUST show that output.

#### Scenario: A reuse run writes no log

- GIVEN a first realization built and indexed a selected root, leaving its log
- WHEN realization reuses that root unchanged
- THEN it MUST NOT create or rewrite the root's log file.

#### Scenario: A log write failure leaves the realization intact

- GIVEN the logs directory cannot be created or written
- WHEN realization otherwise succeeds
- THEN the receipt MUST still be published and the realization MUST still succeed
- AND the failure MUST be reported as a diagnostic persistence failure without changing the
  receipt.
