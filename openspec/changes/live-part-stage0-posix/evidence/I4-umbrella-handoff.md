Task-ID: I4
Covers: bootstrap.part.stage0.posix

# I4: Umbrella roll-up dependency handoff

Command/context:

```sh
rg -n 'live-part-stage0-posix|stage0-posix' openspec/changes/live-bootstrap-parts-index.md openspec/changes/live-bootstrap-source-chain/tasks.md
```

Files updated:

- `openspec/changes/live-bootstrap-parts-index.md`: added a completed handoff entry for `live-part-stage0-posix`, including the output path and ownership of source-pin/build/smoke/leakage evidence.
- `openspec/changes/live-bootstrap-source-chain/tasks.md`: added a V1 part-handoff note telling the umbrella roll-up to consume the part evidence for the `bootstrap/stage0-posix.ncl` portion instead of duplicating detailed transcripts.

Status: complete. The broad source-chain change now has an explicit dependency/handoff to this part's evidence while keeping its own V1 roll-up task open for `mes` and `tinycc`.
