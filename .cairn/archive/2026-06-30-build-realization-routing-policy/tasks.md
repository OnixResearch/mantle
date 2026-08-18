# Tasks

## Contract

- [x] [serial] Define route planner inputs, route classes, deterministic ranking, rejected-route reason codes, redaction rules, and non-claim wording. r[realization_routing.route_plan_model]
- [x] [serial] Define deterministic route ranking and tie-breaker semantics so equivalent fact sets choose the same route without latency, discovery-order, map-order, or random races. r[realization_routing.deterministic_route_ranking]
- [x] [serial] Define offline-mode eligibility rules that reject routes requiring live network, remote lookup, undeclared source material, ambient package-manager/build caches, language-specific caches, or untrusted output import. r[realization_routing.offline_fail_closed]
- [x] [serial] Define route eligibility by requested claim strength, including practical build routes versus strong action-correctness or release-facing routes. r[realization_routing.claim_strength_routing]
- [x] [serial] Define P2P remote-builder route eligibility over concrete build inputs, capabilities, upload feasibility, resource limits, upload privacy policy, and separate output trust. r[realization_routing.remote_builder_eligibility] r[realization_routing.remote_upload_privacy]
- [x] [serial] Define build-plan report semantics for selected route, rejected alternatives, source gaps, trust blockers, archive candidates, remote-builder candidates, upload summaries, and claim-strength blockers. r[realization_routing.route_report]

## Implementation

- [x] [serial] Implement a pure routing core over in-memory local/cache/archive/source/builder/executor facts with deterministic route selection, tie-breakers, and reason codes. r[realization_routing.route_plan_model] r[realization_routing.deterministic_route_ranking]
- [x] [serial] Wire the `mantle build --plan` shell to gather bounded facts and render the extended route report without mutating store or source state. r[realization_routing.route_report]
- [x] [serial] Integrate offline-mode checks with source-bundle readiness and store/archive/substitution facts so hidden network or ambient cache fallback cannot become eligible. r[realization_routing.offline_fail_closed]
- [x] [serial] Integrate claim-strength checks so routes that cannot produce requested strong evidence are downgraded or rejected before execution. r[realization_routing.claim_strength_routing]
- [x] [serial] Integrate remote-builder eligibility facts and privacy-safe upload summaries without opening a builder session or accepting ticket resource access as output trust. r[realization_routing.remote_builder_eligibility] r[realization_routing.remote_upload_privacy]

## Verification

- [x] [serial] Add pure positive tests for local cache, trusted substitute, archive-import candidate, source-bundle-required, remote-build candidate, local-build candidate, deterministic tie-breaker, and claim-strength-compatible route selection. r[realization_routing.route_plan_model] r[realization_routing.deterministic_route_ranking] r[realization_routing.claim_strength_routing]
- [x] [serial] Add pure negative tests for offline network requirement, missing source state, untrusted builder output key, archive prefix mismatch, builder capability mismatch, upload-limit overflow, upload privacy denial, strong-evidence mismatch, and local executor preflight failure. r[realization_routing.offline_fail_closed] r[realization_routing.remote_builder_eligibility] r[realization_routing.remote_upload_privacy] r[realization_routing.claim_strength_routing]
- [x] [serial] Add CLI/build-plan tests proving JSON route reports include selected and rejected route reasons, remain parseable, and omit bearer tickets, private key paths, raw env values, and full argv; pure route tests cover upload summaries and claim-strength blockers until runtime transport facts exist. r[realization_routing.route_report]
- [x] [serial] Add non-claim tests proving route reports do not claim build success, output trust, remote execution, archive import, strong action-correctness, or release readiness before downstream evidence exists. r[realization_routing.route_report] r[realization_routing.claim_strength_routing]
- [x] [serial] Run `nix run path:/home/brittonr/git/cairn#cairn -- validate --root .` and proposal/design/tasks gates before implementation claims, then record focused implementation evidence before checking tasks complete. r[realization_routing.route_plan_model]
