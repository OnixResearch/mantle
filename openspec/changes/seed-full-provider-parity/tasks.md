## Phase 1: Provider parity split

- [ ] [serial] Add OpenSpec requirements for axis-specific seed provider parity evidence.
- [ ] [depends:bootstrap.parity.provider.axis.evidence] Split `seed-full` parity reporting into Guix source-root and StageX lineage-provider rows with fail-closed StageX gating.
- [ ] [depends:seed-full-row-split] Add parity tests for source-root contract completion, legacy metadata rejection, and StageX `--require` failure.
- [ ] [depends:verification] Run parity/OpenSpec verification and archive the completed change.
