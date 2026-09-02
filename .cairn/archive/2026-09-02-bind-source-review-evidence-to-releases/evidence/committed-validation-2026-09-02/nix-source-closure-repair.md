# Nix test-source closure repair

The local full check first stopped because six `include_bytes!` uses could not read five tracked V47/V48 evidence files under `.cairn/archive/`. A source-path probe showed that `cairn/archive/` was present in the filtered Nix source, but `.cairn/archive/` was absent.

The source filter now includes tracked `.cairn/archive/` evidence. The next focused Nextest run compiled the missing evidence references and exposed a negative fixture that declared a nonexistent Rust compiler path. That fixture now copies the current compiler bytes to a distinct temporary path, so path canonicalization succeeds and the test reaches its intended `host-tool-leakage` rejection. The focused test passes.

The following Nextest run reached a root `fixtures/composition-roots/` request that the selective fixture filter had omitted. The source filter now includes all checked root `fixtures/`. `srcWithoutContentBoundRequirements` still removes the content-bound-requirement fixture where that narrower source is required.

A source-path probe confirms that both the promoted `.cairn/archive/` evidence and `fixtures/composition-roots/equivalent-a.json` are present. The durable-publication receipt was refreshed for the changed `flake.nix` bytes only:

- `flake.nix` BLAKE3: `5bebb36334e2dc528406c9882bd2f7aabbda8315ecdac3438f8b66b044a7d962`
- receipt JSON BLAKE3: `e8e8455e7b92e231ef4afbe58146663a095995b07f6347519b66348f4557875e`

The dependency RID, revision, source URL, Cargo bindings, Nix lock binding, mapping, authority, rollback policy, and non-claims did not change. The focused durable-publication gate passes.

With these closure repairs, focused Nextest starts 6,032 tests and passes the first 305. It then stops at the unrelated `evaluator_budget_cli::cancellation_is_terminal_and_reaps_a_late_worker` test because the command unexpectedly returns success. The exact output is in `nix-nextest-after-source-closure.log`. This change does not weaken, skip, or reclassify that test.
