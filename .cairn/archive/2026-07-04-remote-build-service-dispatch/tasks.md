## Implementation

- [x] [serial] I1 Extract normalized remote build key, request planning, upload manifest planning, and response admission into pure remote-build core helpers. r[remote_builds.scheduler_build_service_dispatch]
- [x] [serial] I2 Implement a scheduler-compatible remote build service adapter that dispatches concrete ready goals and imports verified outputs through the existing store path. r[remote_builds.scheduler_build_service_dispatch]
- [x] [serial] I3 Wire lazy worker route selection so fetcher, local sandbox, substitution, and remote execution share goal dedupe and dependency interleaving. r[remote_builds.scheduler_build_service_dispatch]
- [x] [serial] I4 Preserve phase-classified remote failures and allow fallback only under explicit route policy. r[remote_builds.scheduler_build_service_dispatch]

## Verification

- [x] [serial] V1 Positive: two identical ready goals attach to one remote-dispatched build and import the verified output once. r[remote_builds.scheduler_build_service_dispatch]
- [x] [serial] V2 Negative: raw Nickel/frontend evaluation requests, mismatched output identity, untrusted output key, and invalid phase fallback fail closed. r[remote_builds.scheduler_build_service_dispatch]
- [x] [serial] V3 Run focused scheduler/remote-build-service tests plus Cairn validate and proposal/design/tasks gates for this change. r[remote_builds.scheduler_build_service_dispatch]
