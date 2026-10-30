# Integration run, 2026-10-30

Executed in worktree `/home/brittonr/.cargo-target/mantle-pilot-materialization-20261029`
on branch `cairn/molten-pilot-materialization-publish`, created from upstream
`origin/main` `9a618c27362a7f2e0f8dcf61913e598e8b022523` with the pilot work
cherry-picked onto it. Nix ran with `--max-jobs 2 --cores 4 --builders ''`.

## Producer and negative checks

```sh
nix build --max-jobs 2 --cores 4 --builders '' --no-link --print-out-paths --keep-going -L \
  .#spacewasm-reference-bundle .#checks.x86_64-linux.spacewasm-reference-negative
```

Exit 0. Outputs:

- bundle `/nix/store/938i1dqs5b54s7fm485wzgadspps66fn-mantle-spacewasm-reference-bundle-e24cf09355a90497148eb5029fdb8e3400bd63e3`
- negative check `/nix/store/6xzx3cnbym4a7l2nrqhgslip3jrm9hi9-mantle-spacewasm-reference-negative-check`
- pilot fixture check (a build input of the negative check)
  `/nix/store/0flsqybcsiym7psm2i16n5acndml9sb0-spacewasm-pilot-fixture-negative-check`

`results.txt` of the pilot fixture check:

```text
missing rejected
mismatch rejected
substitution rejected
```

The existing negative check output lists `wrong-source-digest`,
`unsupported-claim-class`, `tampered-stable-report`, `missing-required-member`
and `changed-member-digest` as rejected.

## Independent complete-bundle verification

```sh
/nix/store/gvb0p21wxwnbl58xcb86scsisk5s1vi6-mantle-spacewasm-reference-bundler-0.1.0/bin/mantle-spacewasm-reference \
  verify /nix/store/938i1dqs5b54s7fm485wzgadspps66fn-mantle-spacewasm-reference-bundle-e24cf09355a90497148eb5029fdb8e3400bd63e3
```

Exit 0:

```json
{
  "schema": "mantle-spacewasm-bundle-verify-summary-v1",
  "valid": true,
  "bundle_identity_blake3": "774331f6c2defe75e30c354b2b3080838cf56438bbd99268779fe4590f8c9f39",
  "diagnostics": []
}
```

`fixtures/wasm/` contains `molten-host-segments.wasm` and the nine
`molten-abi-*.wasm` members.

## Limits

These are producer materialization and byte-integrity facts only. No consumer
ABI, hostcall, memory, trap, stack or fuel scenario ran here; those belong to
Molten and ChaosControl receipts. The verifier does not authenticate a
publication, and a local rebuild with identical bytes does not establish
producer provenance.
