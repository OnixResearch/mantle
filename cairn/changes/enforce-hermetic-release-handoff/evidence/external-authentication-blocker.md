# External authenticated-handoff blocker

Recorded: 2026-07-12

Requirement: `mantle.release_provenance.cairn_evidence_handoff.cross_repo_dependency`

Mantle does not claim authenticated Cairn provenance in this change. The local
validation receipt is fixed to `authentication_status: "not-authenticated"` and
rejects promotion to `authenticated`.

The required external Cairn change is not consumable:

- active path: `/home/brittonr/git/OnixResearch/cairn/cairn/changes/authenticate-stack-provenance-inputs/`
- archive search: no path matching `*authenticate-stack-provenance-inputs*`
  under `/home/brittonr/git/OnixResearch/cairn/cairn/archive/`
- metadata status: `blocked`
- task counts from Cairn status: `done: 0`, `todo: 13`, `total: 13`
- every task line in the active package remains unchecked

Current evidence command:

```text
$ nix run .#cairn -- status --root . --change authenticate-stack-provenance-inputs
{
  "name": "authenticate-stack-provenance-inputs",
  "path": "./cairn/changes/authenticate-stack-provenance-inputs",
  "metadata": {
    "depends_on": [
      "evidence-chain-readiness-gates",
      "function-address-preserves-readiness",
      "preserves-evidence-envelope-ingest"
    ],
    "groups": ["current"],
    "status": "blocked",
    "target_date": null
  },
  "task_counts": {
    "done": 0,
    "in_progress": 0,
    "todo": 13,
    "total": 13,
    "unmarked": 0
  }
}
```

Decision: keep Mantle's authenticated-handoff dependency and lifecycle closeout
tasks unchecked. Do not fabricate an authenticated receipt, sync, or archive.

Owner: Cairn `authenticate-stack-provenance-inputs` change owners.

Next action: consume and validate the archived Cairn receipt only after that
external change reaches archived status with completed authenticated-input,
producer-signature, policy-identity, and negative-fixture evidence.
