# Project soundness diagnostics implementation evidence

Date: 2026-06-30

## Baseline

Direct Cargo was unavailable in the ambient shell:

```text
$ cargo test -p crunch-project-core --lib
sh: line 1: cargo: command not found
```

Nix dev-shell baseline before core edits passed:

```text
$ nix develop -c cargo test -p crunch-project-core --lib
test result: ok. 77 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Gates before task completion

Captured in `gates-before-completion.txt`:

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- gate proposal project-soundness-diagnostics --root .
verdict: PASS
$ nix run path:/home/brittonr/git/cairn#cairn -- gate design project-soundness-diagnostics --root .
verdict: PASS
$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks project-soundness-diagnostics --root .
verdict: PASS
```

## Implementation checks

Focused core soundness tests:

```text
$ nix develop -c cargo test -p crunch-project-core --lib soundness
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 77 filtered out; finished in 0.00s
```

Full project-core library tests:

```text
$ nix develop -c cargo test -p crunch-project-core --lib
test result: ok. 89 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Project CLI tests:

```text
$ nix develop -c cargo test -p mantle --test project_cli
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
```

Formatting:

```text
$ nix develop -c cargo fmt --check -p crunch-project-core -p crunch-project -p mantle
finished successfully
```

## Notes

The implementation adds a no-std pure soundness core over parsed manifest, lockfile, generated-input content, and normalized supplemental facts for freshness, fetch policy, trust policy, and retention roots. The CLI remains the shell for file reads and rendering. Default `mantle check` remains no-network; `--probes` and `--trust` label the report mode so future adapters can feed supplemental facts without changing the report schema.
