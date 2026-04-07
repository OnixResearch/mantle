# Fetcher Architecture Specification — Delta

## ADDED Requirements

### Requirement: Fetch BuildRequest encoding

Fetcher derivations MUST reuse the existing `BuildRequest` fields.
They MUST NOT require a new request format.

A fetch request is encoded as follows:
- `command_args[0] == "builtin:fetchurl"`
- `environment_vars` carries the fetch parameters by their existing
  names: `url`, `unpack`, `type`, `rev`, `executable`
- `outputs[0]` is the fetch output path

#### Scenario: fetchurl request uses existing BuildRequest fields

- GIVEN a fetchurl derivation converted to `BuildRequest`
- WHEN the request is inspected
- THEN `command_args[0]` is `builtin:fetchurl`
- AND `environment_vars` still contains `url`
- AND no new BuildRequest fields are needed

### Requirement: FetchBuildService

The system MUST provide a `FetchBuildService` that implements the
`BuildService` trait. When `do_build()` receives a fetch request, it
MUST parse fetch parameters from the existing `BuildRequest` fields,
perform the download or extraction, ingest the result into castore, and
produce a `BuildResult` with the output node.

`FetchBuildService` MUST NOT perform fixed-output hash verification,
`PathInfo` persistence, or disk export. Those steps stay in the shared
post-build path.

```rust
pub struct FetchBuildService<BS: BlobService, DS: DirectoryService> {
    blob_service: BS,
    directory_service: DS,
}

#[async_trait]
impl<BS, DS> BuildService for FetchBuildService<BS, DS>
where BS: BlobService + Clone + 'static,
      DS: DirectoryService + Clone + 'static,
{
    async fn do_build(&self, request: BuildRequest) -> io::Result<BuildResult>;
}
```

#### Scenario: Fetcher produces BuildResult

- GIVEN a BuildRequest with `command_args[0] = "builtin:fetchurl"`
- AND `environment_vars` contains `url`
- WHEN `fetch_service.do_build(request)` is called
- THEN the resource is downloaded or unpacked
- AND a `BuildResult` is returned with the output node

#### Scenario: Non-fetch request rejected

- GIVEN a BuildRequest with `command_args[0] = "/bin/sh"`
- WHEN `fetch_service.do_build(request)` is called
- THEN an error is returned (this service only handles fetchers)

### Requirement: DispatchBuildService

The system MUST provide a `DispatchBuildService` that wraps a
`FetchBuildService` and a sandbox `BuildService`. It inspects
`request.command_args[0]` and routes to the appropriate implementation.

```rust
pub struct DispatchBuildService<F, S> {
    fetch: F,
    sandbox: S,
}
```

#### Scenario: Fetch derivation dispatched to fetch service

- GIVEN a DispatchBuildService wrapping fetch + sandbox services
- WHEN a BuildRequest with `command_args[0] = "builtin:fetchurl"` arrives
- THEN it is dispatched to the FetchBuildService
- AND the sandbox service is not called

#### Scenario: Regular derivation dispatched to sandbox

- GIVEN a DispatchBuildService wrapping fetch + sandbox services
- WHEN a BuildRequest with `command_args[0] = "/bin/sh"` arrives
- THEN it is dispatched to the sandbox BuildService

## MODIFIED Requirements

### Requirement: Builtin fetcher bypass in orchestrator (modified)

The orchestrator MUST NOT contain fetcher-specific branching or inline
fetch execution. The `prepare_build()` method MUST treat all derivations
the same: it constructs a `BuildRequest` and dispatches via the
`BuildService` trait.

Fetcher derivations still bypass the sandbox, but they do so by being
routed to `FetchBuildService` through `DispatchBuildService`, not by
bypassing `BuildService::do_build()` entirely.

#### Scenario: No is_builtin_fetcher check

- GIVEN the orchestrator's `prepare_build()` method
- WHEN inspected
- THEN it does not check `is_builtin_fetcher()` or call `build_fetcher()`
- AND all derivations go through the same dispatch path

### Requirement: Hash verification (modified)

After a fetch build returns a `BuildResult`, the shared post-build path
MUST verify the output against the declared hash:

- For `mode = 'flat` (fetchurl): hash the raw file bytes from the
  produced file node
- For `mode = 'recursive` (fetchTarball, fetchGit): compute the NAR hash
  of the produced output tree

A mismatch MUST:
1. Delete the produced output before persist/export
2. Report both expected and actual hash in SRI format
3. Report the `.ncl` source file and approximate location of the hash to
   update

#### Scenario: Hash matches

- GIVEN a fetchurl with `hash = "sha256-XXXX..."` and the downloaded
  file hashes to the same value
- WHEN verification runs in the shared post-build path
- THEN the fetch succeeds

#### Scenario: Hash mismatch

- GIVEN a fetchurl with `hash = "sha256-AAAA..."` but the actual
  content hashes to `sha256-BBBB...`
- WHEN verification runs in the shared post-build path
- THEN the output is deleted
- AND the error reports:
  ```
  hash mismatch for fetchurl 'foo-1.0.tar.gz':
    expected: sha256-AAAA...
    got:      sha256-BBBB...
    update hello.ncl to: hash = "sha256-BBBB..."
  ```

### Requirement: Network isolation (modified)

Fetcher execution MUST happen outside the build sandbox. Fetch
requests MAY flow through a composite `BuildService`, but they MUST be
routed to a non-sandbox implementation such as `FetchBuildService`.

Regular (non-fetcher) derivations MUST NOT have network access. The
sandbox BuildService MUST continue to block network for all non-builtin
builders.

#### Scenario: Fetch request bypasses the sandbox service

- GIVEN a fetch derivation encoded as a BuildRequest
- WHEN DispatchBuildService handles it
- THEN FetchBuildService runs with network access
- AND the sandbox BuildService is not called

#### Scenario: Regular build has no network

- GIVEN a derivation with `builder = "/bin/sh"`
- WHEN built in the sandbox
- THEN network access is blocked (existing behavior, unchanged)

## REMOVED Requirements

### Requirement: Builder::build_fetcher helper

The `build_fetcher()` method on Builder MUST be removed. Its
functionality is split between `FetchBuildService` and the shared
`finish_build()` path.
