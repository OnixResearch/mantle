# Evidence: V3 OpenSpec gates

Task-ID: V3
Covers: release.evidence.workflow.witnessed.selfhosting, release.evidence.workflow.witnessed.crossmachine.docs, release.verification.tech.witness.rebuild.cli
Status: passed

## Commands

```bash
openspec validate replayable-witness-rebuilds
openspec_gate stage=design change=replayable-witness-rebuilds
```

## Results

### `openspec validate replayable-witness-rebuilds`

```text
Change 'replayable-witness-rebuilds' is valid
```

### `openspec_gate stage=design change=replayable-witness-rebuilds`

```text
VERDICT: PASS
```

## Coverage notes

The design gate passed with only low-severity refinement suggestions around:
- making timestamp bracketing semantics more explicit,
- clarifying helper-vs-CLI scratch-root ownership, and
- noting that request-directory immutability is a first-slice convention.
