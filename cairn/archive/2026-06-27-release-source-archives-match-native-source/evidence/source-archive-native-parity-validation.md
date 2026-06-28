# Release source archive native parity validation

Change: `release-source-archives-match-native-source`

This transcript records focused validation for the release source archive contract. It proves the archive includes tracked package source visible to native source hashing and excludes untracked/private skipped paths. It does not claim an independent witness replay; provider/self-hosting proofs still need to be regenerated from this source snapshot.

### Format check

```text
$ /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/cargo fmt -p mantle --check
exit_status=0
```

### Mantle binary source archive test

```text
$ /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/cargo test -p mantle --bin mantle release_source::tests::source_archive_uses_current_worktree_verified_vendor_and_tracked_package_files -- --nocapture
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.18s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)

running 1 test
test release_source::tests::source_archive_uses_current_worktree_verified_vendor_and_tracked_package_files ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 878 filtered out; finished in 0.02s

exit_status=0
```

### Crunch binary source archive test

```text
$ /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/cargo test -p mantle --bin crunch release_source::tests::source_archive_uses_current_worktree_verified_vendor_and_tracked_package_files -- --nocapture
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.20s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/crunch-1c062a2eb29efb39)

running 1 test
test release_source::tests::source_archive_uses_current_worktree_verified_vendor_and_tracked_package_files ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 878 filtered out; finished in 0.02s

exit_status=0
```

### Cairn validate

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 7,
  "valid": true
}
exit_status=0
```

### Cairn proposal gate

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- gate proposal release-source-archives-match-native-source --root .
{
  "change": "release-source-archives-match-native-source",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "61db351339b58699646e3b71b8d040b0370712c32e90faad3199092cba3ec2dc",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "698dfe7dca4006714e2149d5a31214be8547642948deac910dd36b821aa97583",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
exit_status=0
```

### Cairn design gate

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- gate design release-source-archives-match-native-source --root .
{
  "change": "release-source-archives-match-native-source",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "a648477bc925244355663ab8c0ab275fe16bc0bdb7f21dc70bc4d68ab7f59fcf",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "0be4b8ccf6887eed4010475b82eddcdcf5916504f9226d9cb59b9b6f718f0796",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
exit_status=0
```

### Cairn tasks gate

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks release-source-archives-match-native-source --root .
{
  "change": "release-source-archives-match-native-source",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "f6bd0382f9b74beb13696a82e70a064bdf6ba657de677e953a26a97a94ca2a45",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "35a9879f950921c010b96fb70dabd76a030d1b56b6ae07c47f3183702eb1e2b5",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
exit_status=0
```

### Post-task Cairn validation

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 7,
  "valid": true
}
exit_status=0
$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks release-source-archives-match-native-source --root .
{
  "change": "release-source-archives-match-native-source",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "cfd7ad3510d2b092a330c0ba4cd71a87238a4e807f14a47d31021d28847abc41",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "555bb6e7b1b7c06040bffae561afe187832135b926057ba78aea5ac8245a523e",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
exit_status=0
```

### Post-archive validation

The Cairn archive command moved the change package but did not automatically merge the ADDED requirement into `cairn/specs/verification-evidence/spec.md`; the requirement was manually synced into the accepted spec before this validation.

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 0,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 6,
  "valid": true
}
exit_status=0
```
