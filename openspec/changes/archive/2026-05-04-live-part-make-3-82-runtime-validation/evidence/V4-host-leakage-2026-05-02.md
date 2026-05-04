# V4 host-leakage scan evidence (2026-05-02 rerun)

Task-ID: V4
Covers: bootstrap.part.make.3.82.runtime-validation

## Inputs scanned

- `openspec/changes/live-part-make-3-82-runtime-validation/evidence/build.stdout.log`
- `openspec/changes/live-part-make-3-82-runtime-validation/evidence/build.stderr.log`
- `openspec/changes/live-part-make-3-82-runtime-validation/evidence/build.derivation-2026-05-02.log`
- `openspec/changes/live-part-make-3-82-runtime-validation/evidence/validation-summary.json`

## Result

The validation runner reported no leakage findings in `validation-summary.json`:

```json
"leakage_findings": []
```

Additional coarse needle scan over captured transcripts:

- `openspec/changes/live-part-make-3-82-runtime-validation/evidence/build.stdout.log`: /home/brittonr x3
- `openspec/changes/live-part-make-3-82-runtime-validation/evidence/build.stderr.log`: no matches
- `openspec/changes/live-part-make-3-82-runtime-validation/evidence/build.derivation-2026-05-02.log`: no matches
- `openspec/changes/live-part-make-3-82-runtime-validation/evidence/validation-summary.json`: /home/brittonr x8

The `/home/brittonr` matches are limited to Crunch runner metadata paths in the structured build report/summary (`output_dir`, `state_dir`, `saved_log_path`, and evidence paths), not sandbox builder input paths. The sandboxed derivation transcript had no matches for `/home/brittonr`, `/nix/store`, `/usr/bin`, `/bin/bash`, `/etc/nixos`, `CARGO_HOME`, or `RUSTUP_HOME`.

Conclusion: no undeclared host-tool/path/environment leakage was found in the latest failed-build evidence. The remaining blocker is the `make-3.82-tcc` builder segfault.
