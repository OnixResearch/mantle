# Evidence: V3 OpenSpec gates

Task-ID: V3
Covers: release.evidence.workflow.witnessed.selfhosting, release.evidence.workflow.witnessed.crossmachine.docs, release.verification.tech.witness.rebuild.cli
Status: pending

## Planned commands

```bash
openspec validate replayable-witness-rebuilds
openspec_gate stage=design change=replayable-witness-rebuilds
```

## Notes

Pending implementation. The tasks-stage gate transcript is intentionally
collected later at archive/closeout time rather than used as circular evidence
inside this task packet.
