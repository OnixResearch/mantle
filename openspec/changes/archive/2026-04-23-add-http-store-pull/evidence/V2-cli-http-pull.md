Evidence-ID: add-http-store-pull-v2-cli-http-pull
Task-ID: V2
Artifact-Type: verification-note
Covers: binary.cache.cli.storepull, binary.cache.storepull.http, binary.cache.cli.storepull.httpurl, binary.cache.cli.storepull.httpallrejected, binary.cache.cli.storepull.httprequirespaths, binary.cache.cli.storepull.unsupportedurlscheme, binary.cache.cli.storepull.userinforejected
Reviewer-Role: agent
Verdict: pass
Reviewed-At: 2026-04-23

Validation commands:
- `cargo test -p crunch --test integration store_pull_ -- --nocapture`
- `cargo test -p crunch --test smoke smoke_build_push_http_pull_round_trip -- --nocapture`

Results:
- `cargo test -p crunch --test integration store_pull_ -- --nocapture` -> `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 51 filtered out; finished in 0.07s`
- `cargo test -p crunch --test smoke smoke_build_push_http_pull_round_trip -- --nocapture` -> `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 16 filtered out; finished in 0.00s` with host output `SKIP: /nix/store not writable or bwrap missing`

The CLI integration suite now covers:
- `store_pull_http_requires_explicit_paths`
- `store_pull_http_rejects_all`
- `store_pull_rejects_unsupported_url_scheme`
- `store_pull_rejects_http_url_with_userinfo`
- `store_pull_http_round_trip_imports_path`

Source inspection confirms the CLI shell changes in `src/main.rs` and `src/store_cmd.rs`:
- `store pull --from` stays a raw `String` until dispatch, avoiding `PathBuf` URL mangling
- exact `http://`/`https://` inputs become HTTP URL candidates
- unsupported `scheme://...` sources are rejected before local-path dispatch
- HTTP URL userinfo is rejected before dispatch
- HTTP selector parsing still requires explicit logical store paths
- the README documents explicit-path HTTP pull examples and the directory-only `--all` rule
