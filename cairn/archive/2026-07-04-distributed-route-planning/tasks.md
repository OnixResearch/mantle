## Implementation

- [x] [serial] I1 Add pure route input facts for remote-builder candidates: capabilities, source/input readiness, upload budget, output trust, network policy, and requested claim strength. r[realization_routing.remote_route_plan_cli]
- [x] [serial] I2 Extend `mantle build --plan` to include remote-builder eligible/rejected routes without opening remote sessions or uploading bytes. r[realization_routing.remote_route_plan_cli]
- [x] [serial] I3 Render human and JSON plan reports with selected route, rejected remote reason codes, redacted trust/capability facts, and non-claim text. r[realization_routing.remote_route_plan_cli]

## Verification

- [x] [serial] V1 Positive: remote route is eligible when concrete inputs, matching capabilities, upload feasibility, and output trust are present. r[realization_routing.remote_route_plan_cli]
- [x] [serial] V2 Negative: remote route is rejected for offline mode, no output trust, no concrete inputs, capability mismatch, upload overflow, and missing source readiness. r[realization_routing.remote_route_plan_cli]
- [x] [serial] V3 Run focused route-planner/build-plan tests plus Cairn validate and proposal/design/tasks gates for this change. r[realization_routing.remote_route_plan_cli]
