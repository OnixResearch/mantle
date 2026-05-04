# V4 host-leakage scan evidence

Task-ID: V4
Covers: bootstrap.part.make.3.82.runtime-validation

## Inputs scanned

- `evidence/build.stdout.log`
- `evidence/build.stderr.log`
- `evidence/build.derivation.log`
- `evidence/validation-summary.json`

## Result

The validation runner reported no leakage findings in `validation-summary.json`:

```json
"leakage_findings": []
```

Additional coarse needle scan over captured transcripts:

- `build.stdout.log`: `/home/brittonr` x3

The `/home/brittonr` matches are limited to Crunch runner metadata paths in
`build.stdout.log` (`output_dir`, `state_dir`, and `saved_log_path`), not builder
input paths. The sandboxed derivation transcript (`build.derivation.log`) had no
matches for `/home/brittonr`, `/nix/store`, `/usr/bin`, `/bin/bash`, `/etc/nixos`,
`CARGO_HOME`, or `RUSTUP_HOME`.

Conclusion: no undeclared host-tool/path/environment leakage was found in the
captured failed-build evidence. The remaining blocker is the builder segfault,
not host leakage.
