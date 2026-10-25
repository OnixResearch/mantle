# Authoring validation

Captured 2026-10-15. These are structural proposal/design/tasks gates for proposed lifecycle text. All implementation tasks remain unchecked; no compilation, performance, compiler-fix, output-admission, sync, archive, or release evidence is claimed.

Repository-wide validation is recorded separately in `pipeline-rust-interface-readiness/evidence/repository-validation.md` under the active changes directory.

## Command

```text
$ cairn gate proposal pipeline-rust-interface-readiness --root .
exit_code=0
```

### Stdout

```json
{
  "acceptance_ids": [],
  "change": "pipeline-rust-interface-readiness",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "215071257c1db4201969ca3763813caaa94d0c977640bd7156a84f16e90c2db0",
  "issues": [],
  "layout": "cairn",
  "non_claims": [
    "workflow state proves only the selected policy graph and observed structural artifact facts",
    "workflow instructions do not prove artifact truth, review quality, or implementation correctness",
    "workflow state does not satisfy proposal, design, tasks, sync, archive, evidence, or release gates",
    "workflow commands are read-only and do not create, edit, delete, or approve lifecycle artifacts"
  ],
  "policy": "mantle-default",
  "policy_hash": "10455600e4ec15e5afa2b7759c70856ed9a1dc4ed1e18b635b32ee3d33cc6b4b",
  "policy_selection": {
    "blake3": "31d095609943dd430aa8ab172cd1b8f5116550d41e5b1c8463b89875017e0d12",
    "installation_receipt": false,
    "locator": "cairn-policy/generated/cairn-policy.json",
    "source": "legacy_default"
  },
  "probe_evidence": null,
  "profile_identity_hash": "a51e352493e2511d670e7c9bd30fe95d47963c463430b0c8e94d12bfaf4740ac",
  "receipt_hash": "6b6dd7e54a48db1bf8fd967a023a5b4a717d4bf88492bee0bff84f64fb1a11a6",
  "review_receipt_hashes": [],
  "spec_effect": "delta",
  "stage": "proposal",
  "substance": [
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./.cairn/changes/pipeline-rust-interface-readiness/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 74,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    }
  ],
  "task_ordering": null,
  "valid": true,
  "verdict": "PASS",
  "workflow_profile": "spec-driven",
  "workflow_profile_compatibility_fallback": false,
  "workflow_profile_source": "change_create",
  "workflow_profile_version": 1
}
```

### Stderr

```text
(empty)
```

## Command

```text
$ cairn gate design pipeline-rust-interface-readiness --root .
exit_code=0
```

### Stdout

```json
{
  "acceptance_ids": [],
  "change": "pipeline-rust-interface-readiness",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "1de11e5d57765fcc32a0142f468b4dc6b61d863796ed093cdab4101dbeeb24f3",
  "issues": [],
  "layout": "cairn",
  "non_claims": [
    "workflow state proves only the selected policy graph and observed structural artifact facts",
    "workflow instructions do not prove artifact truth, review quality, or implementation correctness",
    "workflow state does not satisfy proposal, design, tasks, sync, archive, evidence, or release gates",
    "workflow commands are read-only and do not create, edit, delete, or approve lifecycle artifacts"
  ],
  "policy": "mantle-default",
  "policy_hash": "10455600e4ec15e5afa2b7759c70856ed9a1dc4ed1e18b635b32ee3d33cc6b4b",
  "policy_selection": {
    "blake3": "31d095609943dd430aa8ab172cd1b8f5116550d41e5b1c8463b89875017e0d12",
    "installation_receipt": false,
    "locator": "cairn-policy/generated/cairn-policy.json",
    "source": "legacy_default"
  },
  "probe_evidence": null,
  "profile_identity_hash": "a51e352493e2511d670e7c9bd30fe95d47963c463430b0c8e94d12bfaf4740ac",
  "receipt_hash": "33c1f4b120581bf5c0143403b38f46a51f025019b47da5a00f8dcf82690b42ef",
  "review_receipt_hashes": [],
  "spec_effect": "delta",
  "stage": "design",
  "substance": [
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./.cairn/changes/pipeline-rust-interface-readiness/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 133,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./.cairn/changes/pipeline-rust-interface-readiness/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 74,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./.cairn/changes/pipeline-rust-interface-readiness/specs/rust-package-planning/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 15,
      "substantive_lines": 100,
      "substantive_requirement_blocks": 5,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    }
  ],
  "task_ordering": null,
  "valid": true,
  "verdict": "PASS",
  "workflow_profile": "spec-driven",
  "workflow_profile_compatibility_fallback": false,
  "workflow_profile_source": "change_create",
  "workflow_profile_version": 1
}
```

### Stderr

```text
(empty)
```

## Command

```text
$ cairn gate tasks pipeline-rust-interface-readiness --root .
exit_code=0
```

### Stdout

```json
{
  "acceptance_ids": [],
  "change": "pipeline-rust-interface-readiness",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "b9d5938b452e2bc2ca7430699c7e99c0f94ed37c70e6cddc8d60bcacc5600444",
  "issues": [],
  "layout": "cairn",
  "non_claims": [
    "workflow state proves only the selected policy graph and observed structural artifact facts",
    "workflow instructions do not prove artifact truth, review quality, or implementation correctness",
    "workflow state does not satisfy proposal, design, tasks, sync, archive, evidence, or release gates",
    "workflow commands are read-only and do not create, edit, delete, or approve lifecycle artifacts"
  ],
  "policy": "mantle-default",
  "policy_hash": "10455600e4ec15e5afa2b7759c70856ed9a1dc4ed1e18b635b32ee3d33cc6b4b",
  "policy_selection": {
    "blake3": "31d095609943dd430aa8ab172cd1b8f5116550d41e5b1c8463b89875017e0d12",
    "installation_receipt": false,
    "locator": "cairn-policy/generated/cairn-policy.json",
    "source": "legacy_default"
  },
  "probe_evidence": {
    "declarations_present": false,
    "probes": []
  },
  "profile_identity_hash": "a51e352493e2511d670e7c9bd30fe95d47963c463430b0c8e94d12bfaf4740ac",
  "receipt_hash": "43502eb91fd42d9efba1c79d6696564e79782c2481440981f0a95b962a659310",
  "review_receipt_hashes": [],
  "spec_effect": "delta",
  "stage": "tasks",
  "substance": [
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./.cairn/changes/pipeline-rust-interface-readiness/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 133,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./.cairn/changes/pipeline-rust-interface-readiness/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 74,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./.cairn/changes/pipeline-rust-interface-readiness/specs/rust-package-planning/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 15,
      "substantive_lines": 100,
      "substantive_requirement_blocks": 5,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 16,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./.cairn/changes/pipeline-rust-interface-readiness/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 19,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 16,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 16
    }
  ],
  "task_ordering": {
    "active": false,
    "edges": [],
    "issues": [],
    "nodes": [],
    "order": [],
    "ready_tasks": [],
    "valid": true
  },
  "valid": true,
  "verdict": "PASS",
  "workflow_profile": "spec-driven",
  "workflow_profile_compatibility_fallback": false,
  "workflow_profile_source": "change_create",
  "workflow_profile_version": 1
}
```

### Stderr

```text
(empty)
```

## Command

```text
$ cairn change depend list pipeline-rust-interface-readiness --root .
exit_code=0
```

### Stdout

```json
{
  "change": "pipeline-rust-interface-readiness",
  "command": "change depend list",
  "dependencies": [
    "bound-provisional-rust-execution",
    "explain-rust-build-waits",
    "validate-rust-pipelining-equivalence"
  ],
  "issues": [],
  "layout": "cairn",
  "mutated": false,
  "receipt_hash": "1045549b1c145682c286fac36df044313febd1450d5954e0dd3431b664fcae0d",
  "root": ".",
  "valid": true
}
```

### Stderr

```text
(empty)
```

## Current implementation-admission inspection (2026-10-23)

This section is fresh capability inspection, not an implementation acceptance
receipt. No build, test, formatter, linter, session, or compiler workload ran.
The historical authoring commands above remain historical only.

### Actual selected compiler and Cargo

The interactive PATH has no `rustc` (`rustc -vV`: exit 127). Resolving the
checked-in development environment, without building a derivation, succeeded:

```text
$ nix eval --raw .#devShells.x86_64-linux.default.drvPath
/nix/store/xvzyhzf2l28bll4v5dh3d39kbfpgm1cs-nix-shell.drv
$ nix eval --json .#devShells.x86_64-linux.default.nativeBuildInputs --apply 'xs: map (x: { name = x.name or ""; path = toString x; }) xs'
```

The Rust entries select the already present
`/nix/store/1hz41jg6cc3py3szsn6cxw3m5p42r4fj-rust-default-1.96.0-nightly-2026-04-08`.
All following binary paths are under that exact store root's `bin/`:

```text
$ /nix/store/1hz41jg6cc3py3szsn6cxw3m5p42r4fj-rust-default-1.96.0-nightly-2026-04-08/bin/rustc -vV
rustc 1.96.0-nightly (c75612477 2026-04-07)
binary: rustc
commit-hash: c756124775121dea0e640652c5ee3c89e3dd0eb4
commit-date: 2026-04-07
host: x86_64-unknown-linux-gnu
release: 1.96.0-nightly
LLVM version: 22.1.2
$ /nix/store/1hz41jg6cc3py3szsn6cxw3m5p42r4fj-rust-default-1.96.0-nightly-2026-04-08/bin/rustc -Zearly-metadata --print cfg
error: unknown unstable option: `early-metadata`
exit_code=1
$ /nix/store/1hz41jg6cc3py3szsn6cxw3m5p42r4fj-rust-default-1.96.0-nightly-2026-04-08/bin/cargo -Zheadstart locate-project
error: unknown `-Z` flag specified: headstart
exit_code=101
```

The `--version` fast paths accepted both flag spellings while only printing
versions. Those invocations **do not establish capability**; the processing
probes above reject the actual options before compilation. Cargo's version is
`cargo 1.96.0-nightly (a357df4c2 2026-04-03)`. SHA-256 of the probed files:

```text
da7395d56ccf185ecbf48f5eee4c87736e7b28160923ab84e1543469bc59699e  bin/rustc
73e66f823132b4ed7b1b8ac3a74dbfa48055f45a3448f6f51533fe9caae3c8af  bin/cargo
```

The source-built provider declarations in `bootstrap/rust-source-plan.ncl`
and `bootstrap/rust-source-musl-host-plan.ncl` select upstream Rust 1.94.1
source (SHA-256
`4c142a625f12e3cdf716c68ae19f4f60d98ad1482627b08579b15838e95ad514`),
not a Headstart patch/source identity. This is source-declaration inspection,
not a probe of a built source-provider executable. No admitted Headstart/rustd
provider appears in the inspected Cargo/flake dependency declarations,
bootstrap plans, or local `patches/` directory.

### Exact pinned upstream contracts, reread from source

At Headstart revision `ff81bfecda2116d54776a7fec538fc2f4e349d1f`:

- [patch series](https://github.com/PowderworksCode/headstart/blob/ff81bfecda2116d54776a7fec538fc2f4e349d1f/patches/README.md)
  requires six rustc patches on
  `6006fd03947bd7b04ae5cb5f7f9bb66b44dc8c6e` and three Cargo patches on
  `4f3fb2428282fb2467c4bb5be0163d2f42a31491`. The live artifacts are
  `early-metadata`, `wait-metadata`, and `resume`; ordinary rustc metadata is
  not an implementation of that contract.
- [incremental identity bug](https://github.com/PowderworksCode/headstart/blob/ff81bfecda2116d54776a7fec538fc2f4e349d1f/docs/ice-early-metadata-svh.md)
  explicitly says the early/full SVH disagreement is not fixed at this pin.
  Its known failing workload was not rerun. A different or newer upstream
  revision has not been admitted by this inspection.
- [readiness](https://github.com/PowderworksCode/headstart/blob/ff81bfecda2116d54776a7fec538fc2f4e349d1f/docs/readiness.md)
  explicitly identifies token-free resumed compilation, uncapped paused
  residents, and a metadata-swap invariant checked only by a hand-maintained
  query list. This is not Mantle's strict reservation policy.

At rustd revision `131de832c0035b3214289eb65e6a945342180bd8`,
[main.rs](https://github.com/DioxusLabs/rustd/blob/131de832c0035b3214289eb65e6a945342180bd8/crates/rustd/src/main.rs)
has only `sync` and `patch` request variants (`Request`, lines 1300-1323).
`ask` and `requested` use unbounded `read_line` (1330-1353). `WatchedFile`
stores `mtime` and the update loop compares modification times (1194-1240,
1477-1490). Replies do not bind a session/partition/base/revision/sequence,
and this request enum has no versioned quiescent-update or stop/drain operation.
Its [toolchain pin](https://github.com/DioxusLabs/rustd/blob/131de832c0035b3214289eb65e6a945342180bd8/harness/toolchain)
is `nightly-2026-05-10`, distinct from the probed Mantle compiler. These are
source observations, not a provider execution or a retained-reuse result.

### Host enforcement prerequisite

`/proc/self/cgroup` currently reports
`0::/user.slice/user-1555.slice/session-7.scope`. `stat -c '%A %U:%G %n'`
on that scope and its control files reports root-owned `0755` scope,
root-owned `0644` `cgroup.procs`, `memory.max`, and `pids.max`, and root-owned
`0200` `cgroup.kill`. Both maxima read `max`; `cgroup.subtree_control` is empty.
The machine exposes cgroup-v2 memory/pids controllers at the root, but this
scope does not provide an already delegated finite session boundary to the
current user. No cgroup was created or modified. This observation does not
claim that all privileged services or all other hosts lack containment.

### Existing authority hooks and exact integration seam

The real native application remains serial: `execute_existing_units` in
`crates/mantle-rust-plan-app/src/application.rs` prepares/adopts producer facts,
admits the complete resolved process, restores or executes, and finishes the
unit before visiting the dependent. `classify_unit_execution`,
`admit_observed_processes`, and `validate_successful_unit_processes` then
validate full attempt identity and final successful process observations.

In `src/rust_plan.rs`, `execute_uncached_rust_unit` calls
`finalize_successful_rust_unit_execution` **inside** `execute_miss`, before
application classification. The finalizer constructs the success receipt,
publishes local cache results (`publish_local_cache_result`), publishes shared
results (`publish_shared_cache_result`), and writes the execution receipt.
`NativeTopologyUnitExecutor::finish_unit` records completed libraries/host
artifacts and may run a confirmed build script. Therefore delaying only the
application's returned success observation would be insufficient: a real
session must withhold that finalizer and topology finish until exact-attempt
confirmation, retaining private outputs and diagnostics separately. No
provisional caller has been routed into these whole-unit success hooks.

The wrapper's `compile_and_publish` and `finish_successful_compile` validate
successful exit, unchanged declared inputs and complete staged output before
`publish_verified_compile`. Its `execute_compiler` joins bounded stdout/stderr
readers after `wait_for_compiler`, so it does not forward live readiness.
`rust-cache/daemon/README.md` still admits one output file in V1 and excludes
incremental invocations from strong eligibility. The shared cache candidate
marker remains the visibility edge after complete object and signed envelope
publication (`docs/shared-rust-unit-cache.md`). None is an early-interface or
resident database bridge.

### Disposition and irreducible prerequisites

There is no enabled provisional, early-interface, or retained-provider route,
and none is added by this inspection. No unsupported-mode fallback, synthetic
live event, separate interface compiler action, or unused session DTO/port is
introduced. Existing serial/full-metadata baselines retain their own meaning;
they do not satisfy native early-interface or warm-workspace acceptance.

Implementation remains blocked on (1) an admitted source-fixed compiler/Cargo
closure demonstrating same-binary off/on with ordinary incremental metadata
identity and the required swap regressions; (2) for warm workspaces, a real
provider closure with bounded versioned content-bound update/acknowledgment,
coherent error-state retention, structured work observations, and stop/drain
semantics; and (3) a selected enforceable finite session memory/task/teardown
boundary. A qualified notification-only compiler can conservatively retain
reservations across waits; upstream Cargo's token recycling is not accepted.
Import/reuse permissions and provider ABI qualification remain admission
requirements, not presumed approvals. All runtime and acceptance tasks remain
unchecked. No ADR enabling unsupported execution is accepted here.
