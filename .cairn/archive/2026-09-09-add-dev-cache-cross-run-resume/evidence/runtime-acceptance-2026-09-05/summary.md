# Runtime acceptance progress

## Outcome

The host flake evaluation now passes without package, source, or lockfile changes. The active cold proof remains unchanged. V2, V3, and V4 remain open.

The current consumer is the Mantle dev-resume validation cycle. Mantle maintains the report reviewer and cohort descriptor in this evidence directory. Their fixtures make the report checks repeatable. They do not replace the production checkpoint validator or runtime proof.

## Nix observations

The focused Nickel evaluation returned `/nix/store/mgf4mizpx3zngjhbqpj4cdz6zn1wp7f3-nickel-lang-rev-fixup-1.17.0.drv`. Nix fetched its intermediate source from a configured cache. The previously missing `666qf3k9djcfzv6lwcp4l4v7fgck77ng-source.drv` is now valid.

The Nickel cohort check passed its positive and negative importer and validator fixtures. It reported:

```text
Nickel cohort verified: cli=1.17.0 embedded=2.2.0/0.18.0 revision=1320a983e6c3d1e2fb53dd2464b084b4903b1426
nickel_cohort_exit_code=0
```

The first whole-flake replay exposed another unavailable evaluation prerequisite: `dv1lsa10lkdpv9gdv5za341pd8c8c6c1-mantle-spacewasm-bundler-source.drv`.

An ordinary evaluation of the host package derivation paths built that exact source derivation. The next command passed:

```text
nix flake check --no-build --no-eval-cache --show-trace --option min-free 0 --option max-free 0
all checks passed!
```

This command evaluated the host outputs. It did not build all checks. Nix omitted the incompatible `aarch64-darwin`, `aarch64-linux`, and `x86_64-darwin` systems. The existing strict Octet and broader build limitations are not closed by this result.

The missing-store-object route produced the required host evaluation result. No source-level package repair was necessary. The cause of the original missing Nickel derivation remains unproven. The review passes share one reviewer and are correlated.

The work used the original focused evaluation, one source observation, one focused cohort gate, and the recorded extension for host prerequisites and a final whole-flake replay. No diagnostic exceeded its ten-minute bound.

## Prepared report review

`runtime-cohort.ncl` binds source commit `8e941df2`, its binary, source profile, vendor identity, dev plan, and later run names. The descriptor keeps the promoted plan separate. Its status is `prepared-not-executed`.

Nickel typechecking and export passed. The malformed-digest and relative-cache fixtures both failed with the expected contract error.

`dev-report-review.scm` checks the report shape and declared expectations. It covers the cold case, all six resume prefixes, and provider adoption. It checks exact stage order, publication counts, distinct digest identities, the selected bundle, the dev disposition, and false receipt and alias flags.

The result was:

```text
dev-report-review: PASS positive_cases=8 negative_cases=29 runtime_proven=false scope=fixture-only-report-review
```

The caller must obtain the expected plan and selected bundle independently of the report. An accepted result does not validate source bytes, payloads, execution, fixed-point equality, or actual alias state.

The first Pi file-mode launch could not resolve the relative module from its temporary runner. The corrected call loaded the test module by its absolute path. A later shell call failed because `steel` was absent from PATH. The successful shell replay used the installed executable:

```text
/nix/store/5fa55ca4462mff909kgjy72ifjg6pbl9-steel-0.8.2/bin/steel
```

The JSON-transform wrapper also accepted a parsed adoption fixture with JSON nulls. Its cache fingerprint is `01d9f701edb8d2231174734ba23229ec16292ac596f2a9ba4139a12246f05c22`. The wrapper references the current worktree module path. The source files, rather than that temporary path, are the retained replay assets.

## Active proof and collection

The existing attempt is `dev-cold-8e941df2`, with Mantle PID `1624266` and watcher `1265`. The observation at `2026-09-05T20:19:42-04:00` found StageX transition report files. The attempt status still said `running`, with no recorded blocker. No publication manifest existed at the earlier publication observation.

The thread snapshot recorded `jbd2_log_wait_commit`. This is an I/O observation, not a deadlock diagnosis or proof result.

At `2026-09-05T21:04:01-04:00`, the active attempt had published the transition and StageX provider manifests. Their bundle identities are `ab3128b159d5bae377837625be9617a00f1621ceabe933a19750c4627c287971` and `ea3db1964745186097d558d14ca97ff204cc5b64bde443eb8b67d539d8e888c9`. Both bind the expected dev plan and the `db1ee54c...` orchestrator. Local and remote file hashes matched. `live-provider-prefixes/` retains the exact manifests and the active-attempt observation.

This proves that these two manifests became visible before the cold attempt finished. It does not prove a later restore, full cold success, or the complete dev cycle.

The alias snapshot at `2026-09-05T20:27:57-04:00` found both `latest` and `latest-source-built-fixed-point` absent. It is explicitly a mid-cold baseline, not a retroactive pre-launch observation.

Terminal metadata collection is queued as Pueue `1526`, after the existing watcher. The collector has a five-minute bound. It copies only bounded report and manifest observations, never provider payloads or cache state. It records failed terminal attempts too. Its syntax check passed. The active-attempt negative control returned the expected rejection and created no observation directory.

Collector success means that metadata was copied. It does not establish runtime acceptance.

Pueue `1609` waits for that collector, then reviews the cold report before a cached run. It requires cold exit status zero, the expected plan, six declared stage executions and publications, and absent aliases. The remote command rechecks the binary identity, cache shape, fresh output paths, and free space. Production resume admission still owns payload validation.

The continuation rejected missing terminal evidence. The remote cached command rejected the active cold attempt before it wrote a status or output. The stdin adapter accepted a cold JSON fixture and rejected malformed JSON. Its input stream has an explicit byte bound.

The cached command has a one-day observation deadline. A transport error or deadline does not authorize a retry. The remote status must be inspected first. The installed Steel executable has a temporary GC root at `/home/brittonr/.cache/mantle-dev-resume-tools-8e941df2/steel` for this queued work.

No cached, adoption, prefix-view, or promoted run started during this continuation.

## Remaining acceptance

After the cold attempt finishes, inspect its terminal status, reports, checkpoint objects, and alias observations. A successful cold result permits the cached and adoption checks. The six prefix checks must use private cache views after the cache producer stops.

The old helpers still target `97f47ae2` and require unsupported reflinks. Do not use them for this cohort. The new descriptor is preparation data. `cached-run.command.txt` and `continue-after-cold.command.txt` supply the guarded cached-run commands. Adoption and prefix views still require their own reviewed observations.

A separate promoted cold proof remains necessary. Final gates, accepted-spec sync, archive, and main integration must wait for the required runtime evidence.
