## Context

`Builder::prepare_build()` has two paths: `is_builtin_fetcher()` → 
`build_fetcher()` (100+ lines, bypasses sandbox), or normal sandbox
dispatch via BuildService::do_build(). The orchestrator knows about
URLs, tarballs, git clones, and hash verification.

## Goals / Non-Goals

**Goals:** Make the fetcher a BuildService implementation. Remove
fetcher knowledge from the orchestrator. Make fetchers testable
independently.

**Non-Goals:** Add new fetcher types. Change fetch behavior.
Change the serialized `BuildRequest` format.

## Decisions

### 1. FetchBuildService implements BuildService

**Choice:** A new struct `FetchBuildService<BS, DS>` implements
`BuildService::do_build()`.

**Rationale:** The BuildService trait is the dispatch boundary. Fetchers
should go through it. `FetchBuildService` owns only fetch execution:
decode the request, download or unpack the resource, ingest it into
castore, and return a `BuildResult`.

**Challenge:** The current `build_fetcher()` path also verifies
fixed-output hashes, creates `PathInfo`, and exports to disk.

**Resolution:** Keep those steps in the shared `Builder::finish_build()`
path. `FetchBuildService` returns output nodes; the Builder performs
hash verification, `PathInfo` persistence, export, and mismatch cleanup
for both fetchers and sandbox builds.

### 2. Reuse the existing BuildRequest fields

**Choice:** Encode fetch requests in the existing `BuildRequest`
structure instead of adding new fields.

**Contract:**
- `command_args[0]` is the builder selector. Fetch requests use
  `builtin:fetchurl`.
- `environment_vars` carries fetch parameters by their existing names:
  `url`, `unpack`, `type`, `rev`, `executable`.
- `outputs[0]` is the expected output path.

**Rationale:** This keeps the `BuildService` API stable and avoids a
BuildRequest format change. The fetch request data already exists in the
Derivation environment; this change only preserves it through
`derivation_to_build_request()`.

**Alternative:** Add explicit `builder` or `fetch` fields to
`BuildRequest`. Rejected because it expands the public request format for
one dispatch case.

### 3. DispatchBuildService as the composition point

**Choice:** `DispatchBuildService<Fetch, Sandbox>` checks
`request.command_args.first()` and delegates.

**Rationale:** The Builder constructs one BuildService and dispatches
through it. The fetch/sandbox split lives at the BuildService boundary,
not in `prepare_build()`.

**Alternative:** Have `Builder::prepare_build()` inspect the builder and
pick a service directly. Rejected because it preserves the special case
we are removing.

### 4. FetchBuildService needs blob/directory services

**Choice:** FetchBuildService takes `BS: BlobService` and
`DS: DirectoryService` for ingesting downloaded content into the
castore.

**Rationale:** After downloading, the fetcher must produce a
`BuildResult` with a castore `Node`. It needs to ingest the file or tree
into blob and directory services. This is the same operation used for
source input ingestion.

**Alternative:** Return raw bytes and let the Builder ingest them.
Rejected because the BuildService contract returns `BuildOutput` nodes,
not raw payloads.

### 5. FOD verification stays in Builder::finish_build

**Choice:** All fixed-output verification happens in `finish_build()`.

**Rationale:** `verify_fod_hash()` already understands flat and
recursive fixed-output hashes. Keeping one verifier avoids split logic
between fetch and sandbox paths and makes mismatch handling consistent.

## Risks / Trade-offs

**[BuildRequest parsing is stringly-typed]** `DispatchBuildService`
identifies fetchers from `command_args[0]`, and `FetchBuildService`
reads named env vars from `environment_vars`. That is less explicit than
new typed fields, so the implementation needs helper functions and unit
tests that lock the contract down.

**[Finish-build cleanup must stay shared]** Moving flat-hash
verification into `finish_build()` means the shared post-build path must
preserve the current cleanup behavior on mismatches. That needs test
coverage for both flat and recursive fetchers.
