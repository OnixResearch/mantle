# Tasks: Support Portable Remote Client

## 1. Command and dependency matrix

- [x] [serial] I1 Inventory every root command and classify portable-client, Linux local-executor, worker, server, bootstrap, proof, and unsupported dependencies. r[realization_routing.portable_client_command_matrix]
- [x] [serial] I2 Add typed platform support profiles to the accepted operator inventory, including effects, required remote capabilities, trust inputs, and stable blockers. r[realization_routing.portable_client_command_matrix]
- [x] [parallel] I3 Add a portable-client source and dependency guard that rejects bwrap, FUSE, seccomp, cgroup, protected-exec, worker-server, bootstrap, or proof imports outside allowed shells. r[realization_routing.portable_client_validation]

## 2. Portable client architecture

- [x] [serial] I4 Split portable request, route, upload, response-admission, and report logic into a pure core with a thin portable filesystem/network/credential/materialization shell. r[realization_routing.portable_client_core]
- [x] [serial] I5 Make local executor capability explicit and allow non-Linux clients to select cache, import, or remote routes without initializing local execution. r[realization_routing.non_linux_remote_route] r[realization_routing.no_local_execution_on_portable_client]
- [x] [parallel] I6 Keep project and Nickel evaluation client-side and send only frontend-neutral concrete build requests, immutable refs, policy identities, selected target facts, and bounded upload plans. r[realization_routing.portable_client_concrete_inputs]

## 3. Validation and documentation

- [x] [serial] V1 Add positive and negative platform-matrix, malformed-request, no-local-execution, trust, protocol, upload-limit, and materialization tests. r[realization_routing.portable_client_validation] [evidence=cairn/changes/support-portable-remote-client/evidence/verification.md]
- [x] [parallel] V2 Run Linux-hosted checks for both Darwin targets and native Darwin tests where available; record native-vs-cross status explicitly. r[realization_routing.portable_client_validation] [evidence=cairn/changes/support-portable-remote-client/evidence/verification.md]
- [x] [parallel] V3 Run dependency/source guards and secret scans for the portable client closure. r[realization_routing.portable_client_credentials] r[realization_routing.portable_client_validation] [evidence=cairn/changes/support-portable-remote-client/evidence/verification.md]
- [x] [serial] V4 Verify existing Linux local build and worker behavior remains unchanged. r[realization_routing.no_local_execution_on_portable_client] [evidence=cairn/changes/support-portable-remote-client/evidence/verification.md]
- [x] [serial] V5 Document supported platforms, client-vs-target semantics, remote prerequisites, output materialization, stable blockers, and remaining non-claims. r[realization_routing.portable_output_materialization] [evidence=cairn/changes/support-portable-remote-client/evidence/verification.md]
