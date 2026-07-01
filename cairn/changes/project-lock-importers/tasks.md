# Tasks

## Contract

- [ ] [serial] Define external pin importer contracts, normalized external pin model, no-mutate plan shape, apply constraints, blockers, and non-claim wording. r[project_workflows.project_lock_importers]
- [ ] [serial] Define Nixtamal import mapping for supported source kinds, mirrors, patches, hash algorithms, frozen inputs, freshness, fetch policy, and trust policy. r[project_workflows.nixtamal_importer]

## Implementation

- [ ] [serial] Implement pure importer conversion from normalized external pin sets to Mantle project manifest/lock/generated-input file plans. r[project_workflows.project_lock_importers]
- [ ] [serial] Implement shell parser/adapter for Nixtamal fixtures and no-mutate/apply CLI surfaces. r[project_workflows.nixtamal_importer]
- [ ] [serial] Add future-adapter seams for flakes, npins, and niv without importing recursive composition semantics into Mantle core. r[project_workflows.project_lock_importers]

## Verification

- [ ] [serial] Add positive pure tests for Nixtamal file, tarball, Git, mirrors, patches, BLAKE3, frozen inputs, and supported metadata mapping. r[project_workflows.nixtamal_importer]
- [ ] [serial] Add negative pure tests for unsupported source kinds, malformed external files, recursive semantics, lossy hash downgrade, unknown patch references, conflicting existing files, and apply attempts with blockers. r[project_workflows.project_lock_importers] r[project_workflows.nixtamal_importer]
- [ ] [serial] Add CLI tests proving plan is no-mutate and apply writes only planned Mantle-owned files. r[project_workflows.project_lock_importers]
- [ ] [serial] Run `nix run path:/home/brittonr/git/cairn#cairn -- validate --root .` and proposal/design/tasks gates before marking implementation tasks complete. r[project_workflows.project_lock_importers]
