# Tasks: support a portable remote-first Mantle client

## Phase 1: Baseline and support policy

- [ ] [serial] I1 Inventory every root command and dependency as portable-client, Linux-worker, Linux-local-executor, server, bootstrap, proof, or unsupported. r[realization_routing.portable_client_command_matrix]
- [ ] [depends:stabilize-operator-command-contract] I2 Extend the accepted typed operator inventory with platform support state, effects, remote requirements, trust requirements, and stable blockers. r[realization_routing.portable_client_command_matrix]
- [ ] [parallel] I3 Add positive supported-command fixtures and negative unknown-platform, unknown-command, conflicting-effect, missing-trust, and unsupported-command fixtures. r[realization_routing.portable_client_validation]

## Phase 2: Portable dependency boundary

- [ ] [serial] I4 Extract pure remote request, route, upload, response-admission, and report logic into a portable client core. r[realization_routing.portable_client_core]
- [ ] [serial] I5 Add a thin portable shell for project files, Nickel evaluation, transport, explicit credentials, local castore, and output writes. r[realization_routing.portable_client_core]
- [ ] [serial] I6 Add compile and source guards that keep bwrap, FUSE, seccomp, cgroup, protected-exec, worker-server, bootstrap, and proof dependencies outside the portable client closure. r[realization_routing.no_local_execution_on_portable_client]
- [ ] [parallel] I7 Add compile-fail or dependency-graph fixtures for forbidden portable-to-Linux imports and role substitutions. r[realization_routing.portable_client_validation]

## Phase 3: Remote-first route and execution

- [ ] [serial] I8 Make local executor support an explicit route fact. Preserve local cache and import routes on non-Linux clients. r[realization_routing.non_linux_remote_route]
- [ ] [serial] I9 Let `mantle build` on a portable client select the native remote route after concrete input, source, upload, capability, credential, and output-trust admission. r[realization_routing.non_linux_remote_route]
- [ ] [serial] I10 Keep target system independent from client system and reject raw Nickel, Onix, Nix, flake, or package-manager evaluation payloads. r[realization_routing.portable_client_concrete_inputs]
- [ ] [parallel] I11 Add local-executor, bwrap, FUSE, seccomp, worker-launch, and Linux-tool discovery sentinels that must remain untouched during portable remote builds. r[realization_routing.no_local_execution_on_portable_client]

## Phase 4: Portable admission and materialization

- [ ] [serial] I12 Add report-only completion and optional admitted local materialization under an explicit unprivileged physical store. r[realization_routing.portable_output_materialization]
- [ ] [serial] I13 Preserve separate execution, upload, log, cancellation, signer, output-admission, and publication authority with explicit secret handles. r[realization_routing.portable_client_credentials]
- [ ] [parallel] I14 Add wrong signer, corrupt CAS, stale fence, prefix mismatch, incomplete closure, oversized upload, unsafe physical path, credential leak, and partial materialization fixtures. r[realization_routing.portable_client_validation]

## Phase 5: Native platforms and validation

- [ ] [serial] I15 Add native `aarch64-darwin`, native `x86_64-darwin`, and Linux support-matrix jobs with checked command fixtures. r[realization_routing.portable_client_validation]
- [ ] [serial] I16 Document installation, state and physical store defaults, remote worker setup, target selection, trust, unsupported commands, rollback, and non-claims. r[realization_routing.portable_client_command_matrix]
- [ ] [serial] V1 Run portable-core unit tests, route-planner tests, remote-client tests, local materialization tests, and Linux local-build parity tests. r[realization_routing.portable_client_validation]
- [ ] [serial] V2 Run `cargo check -p mantle --target aarch64-apple-darwin` and `cargo check -p mantle --target x86_64-apple-darwin` as supplemental checks, then run the declared native Darwin command fixtures. r[realization_routing.portable_client_validation]
- [ ] [serial] V3 Run focused formatting and Clippy with warnings denied, Nickel checks, dependency-boundary checks, secret scans, machine-contract checks, and `git diff --check`. r[realization_routing.portable_client_validation]
- [ ] [serial] V4 Run `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .` plus proposal, design, and tasks gates for this change. Record exact outputs before archive. r[realization_routing.portable_client_validation]
