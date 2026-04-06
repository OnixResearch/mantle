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
Change the BuildRequest format.

## Decisions

### 1. FetchBuildService implements BuildService

**Choice:** A new struct `FetchBuildService<BS, DS>` that implements
`BuildService::do_build()`.

**Rationale:** The BuildService trait is the dispatch boundary. Fetchers
should go through it. The implementation reads the BuildRequest's
environment (url, unpack, type) and performs the fetch.

**Challenge:** BuildService::do_build takes a BuildRequest and returns
a BuildResult with output nodes. The current fetcher code (build_fetcher)
does additional work: FOD hash verification, PathInfo creation, disk
export. These post-build steps are the Builder's job, not the fetcher's.

**Resolution:** FetchBuildService only handles the download and
produces a BuildResult. The Builder's normal finish_build path handles
FOD verification, PathInfo, and disk export — same as sandbox builds.

### 2. DispatchBuildService as the composition point

**Choice:** `DispatchBuildService<Fetch, Sandbox>` checks the builder
string and delegates.

**Rationale:** The Builder constructs one BuildService and dispatches
everything through it. The dispatch logic is a one-line check on the
builder string.

**Alternative:** Have Builder::prepare_build check the builder string
and pick the right service. Rejected — that's what we're trying to
remove.

### 3. FetchBuildService needs blob/directory services

**Choice:** FetchBuildService takes `BS: BlobService` and
`DS: DirectoryService` for ingesting downloaded content into the
castore.

**Rationale:** After downloading, the fetcher must produce a
`BuildResult` with a castore Node. It needs to ingest the file/tree
into blob/directory services. This is the same as what
`ingest_path()` does for source inputs.

**Alternative:** Return raw bytes and let the Builder ingest. Rejected
— the BuildService contract returns Nodes, not raw bytes.

### 4. FOD verification stays in Builder::finish_build

**Choice:** `verify_fod_hash()` is called in `finish_build`, not in
FetchBuildService.

**Rationale:** FOD verification is a post-build step that applies to
all FODs (fetcher and sandbox). Keeping it in finish_build avoids
duplication.

## Risks / Trade-offs

**[BuildRequest for fetchers]** The current code skips
`derivation_to_build_request()` for fetchers. After this change,
BuildRequest must be constructable for fetcher derivations. The
outputs and environment fields are already present in the Derivation;
the BuildRequest just needs to carry them through.

**[Fetch-specific env parsing]** FetchBuildService needs to parse
env.url, env.unpack, env.type from the BuildRequest. This is
somewhat awkward — the BuildRequest wasn't designed for this. But
it's how Nix does it (the derivation environment carries fetch
parameters), and it keeps the BuildService interface clean.
