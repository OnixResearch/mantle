# Fetcher Architecture Specification — Delta

## ADDED Requirements

### Requirement: FetchBuildService

The system MUST provide a `FetchBuildService` that implements the
`BuildService` trait. When `do_build()` receives a BuildRequest whose
builder is `builtin:fetchurl`, it MUST perform the download, extraction,
hash verification, and produce a `BuildResult` with the output node.

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

- GIVEN a BuildRequest with builder "builtin:fetchurl" and env.url set
- WHEN `fetch_service.do_build(request)` is called
- THEN the URL is downloaded, verified, and a BuildResult is returned
  with the output node

#### Scenario: Non-fetch request rejected

- GIVEN a BuildRequest with builder "/bin/sh"
- WHEN `fetch_service.do_build(request)` is called
- THEN an error is returned (this service only handles fetchers)

### Requirement: DispatchBuildService

The system MUST provide a `DispatchBuildService` that wraps a
`FetchBuildService` and a sandbox `BuildService`. It inspects the
BuildRequest and routes to the appropriate implementation.

```rust
pub struct DispatchBuildService<F, S> {
    fetch: F,
    sandbox: S,
}
```

#### Scenario: Fetch derivation dispatched to fetch service

- GIVEN a DispatchBuildService wrapping fetch + sandbox services
- WHEN a BuildRequest with builder "builtin:fetchurl" arrives
- THEN it is dispatched to the FetchBuildService

#### Scenario: Regular derivation dispatched to sandbox

- GIVEN a DispatchBuildService wrapping fetch + sandbox services
- WHEN a BuildRequest with builder "/bin/sh" arrives
- THEN it is dispatched to the sandbox BuildService

## MODIFIED Requirements

### Requirement: Builtin fetcher bypass in orchestrator (modified)

The orchestrator MUST NOT contain fetcher-specific branching. The
`prepare_build()` method MUST treat all derivations the same — it
constructs a BuildRequest and dispatches via the BuildService. The
BuildService layer (DispatchBuildService) handles routing.

#### Scenario: No is_builtin_fetcher check

- GIVEN the orchestrator's `prepare_build()` method
- WHEN inspected
- THEN it does not check `is_builtin_fetcher()` or call `build_fetcher()`
- AND all derivations go through the same dispatch path

## REMOVED Requirements

### Builder::build_fetcher()

The `build_fetcher()` method on Builder MUST be removed. Its
functionality is subsumed by `FetchBuildService`.
