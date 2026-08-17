# Evidence: expose-nix-free-demo-bundle-cli

Date: 2026-07-03

## Focused tests

```text
$ SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo test -p mantle --bin mantle nix_free_demo
running 6 tests
test nix_free_demo_bundle::tests::demo_bundle_validator_rejects_missing_guard_denial ... ok
test nix_free_demo_cmd::tests::malformed_summary_report_fails_closed_with_stable_code ... ok
test nix_free_demo_cmd::tests::report_for_summary_preserves_claimability_and_verdict ... ok
test nix_free_demo_bundle::tests::demo_bundle_validator_rejects_missing_fixed_point_evidence ... ok
test nix_free_demo_bundle::tests::demo_bundle_validator_accepts_matching_fixed_point_and_guard_denials ... ok
test nix_free_demo_bundle::tests::generated_readme_is_derived_from_machine_summary ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 1113 filtered out; finished in 0.00s

$ SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo test -p mantle --test nix_free_demo_cli
running 4 tests
test nix_free_demo_cli_rejects_missing_fixed_point_without_success_claim ... ok
test nix_free_demo_cli_rejects_missing_guard_as_json ... ok
test nix_free_demo_cli_validates_claimable_bundle_as_json ... ok
test nix_free_demo_cli_renders_readme_from_summary ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

## Documentation guard

```text
$ SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo -Zscript scripts/check-operator-proof-guide.rs
operator proof guide drift check passed

$ SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo -Zscript scripts/check-operator-proof-guide.rs --self-test
operator proof guide checker self-test passed
```

## Formatting and Cairn gates

```text
$ SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo fmt -p mantle --check
# no output; exit 0

$ git diff --check
# no output; exit 0

$ nix run path:/home/brittonr/git/cairn#cairn -- gate proposal expose-nix-free-demo-bundle-cli --root .
verdict: PASS

$ nix run path:/home/brittonr/git/cairn#cairn -- gate design expose-nix-free-demo-bundle-cli --root .
verdict: PASS

$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks expose-nix-free-demo-bundle-cli --root .
verdict: PASS
```

## Post-archive validation

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 4,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 19,
  "valid": true
}
```
