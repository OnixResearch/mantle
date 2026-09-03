# Archive summary

Cairn archived `bind-source-observations-and-monotonic-ingest` successfully.

- Archive receipt: `0e99da73e9e2b84b4efc78b99b1425711b958297f9170fba3d6731ff75db0408`.
- Archive mutation manifest: `2b324e3df3f4577996154229418d5a0d33a7d5d96817e3f8df6fee2dbb34998b`.
- All 21 tasks were complete before archive.
- The active change directory is absent.

Cairn created the known `1970-01-01` archive path.
The operator renamed it to:

`.cairn/archive/2026-09-03-bind-source-observations-and-monotonic-ingest/`

The original command output remains in `archive.command.log`.
It records the original path and the successful mutation receipt.

## Post-archive validation

`post-archive.log` records the exact post-archive commands.
Every command completed with status `0`.

- The generated Cairn policy is fresh.
- Strict Cairn validation passed.
- Default Tracey coverage passed with 157 of 157 requirements referenced.
- Focused source-observations Tracey coverage passed with 177 of 177 requirements referenced.
- Both Tracey profiles reported zero missing and zero dangling references.
- Source-observation architecture reported zero findings across 16 negative fixtures.
- Machine contracts passed with 27 contracted and 60 classified surfaces.
- `nix flake check --no-build -L` passed.
- Post-archive source-architecture, durable-publication, and formatting Nix builds passed.
- `git diff --check` passed.

The post-archive durable-publication adoption receipt has BLAKE3:

`72dfadbf2e37c35f1e780c9bd578355601bd0de5c56969f7d6eb7a79615e772e`

V98 remains unchanged and bound to its original source commit.
This archived change makes no current-source fixed-point claim.
