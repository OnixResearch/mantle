Task-ID: V1
Covers: bootstrap.source.chain.implementation,bootstrap.fullsource.claim.evidence,bootstrap.stagex.selfbuild.proof

# Early stage build validation

Result: PASS.

Provider selection: `legacy` for these early bootstrap stage builds. This V1 evidence validates the pre-transition stages only; it does not claim full-source or StageX provider completion.

Command environment:

```sh
export CRUNCH_NO_FUSE=1
export SNIX_BUILD_SANDBOX_SHELL=/nix/store/4mdqc2snfiihr6r61ln1rqs4fis6br9b-busybox-static-x86_64-unknown-linux-musl-1.36.1/bin/busybox
export PATH="$HOME/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:$HOME/.cargo/bin:/nix/store/dk9qhjgg469lv6mriys7v4c59igarmvx-bubblewrap-0.11.1/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH"
```

Store/state:

```text
store: target/live-part-tinycc-0-9-27/run-current/store
state: target/live-part-tinycc-0-9-27/run-current/state
```

## Commands and results

### `bootstrap/stage0-posix.ncl`

```sh
./target/debug/crunch build bootstrap/stage0-posix.ncl \
  --store "$PWD/target/live-part-tinycc-0-9-27/run-current/store" \
  --state-dir "$PWD/target/live-part-tinycc-0-9-27/run-current/state" \
  --no-substitute -j 1 --verbose --log-level info
```

- Exit status: 0
- Output path: `target/live-part-tinycc-0-9-27/run-current/store/35ljc87nc2gcn7cxpj078qjmch8qpqzh-stage0-posix`
- Fallback status: `hermeticity: practical (no degraded facts)`
- Placeholder/deferred/archive rejection: no placeholder text observed; this consumes the archived `live-part-stage0-posix` detailed evidence for part-level source-pin/build/smoke/host-leakage proof.

### `bootstrap/mes.ncl`

```sh
./target/debug/crunch build bootstrap/mes.ncl \
  --store "$PWD/target/live-part-tinycc-0-9-27/run-current/store" \
  --state-dir "$PWD/target/live-part-tinycc-0-9-27/run-current/state" \
  --no-substitute -j 1 --verbose --log-level info
```

- Exit status: 0
- Output path: `target/live-part-tinycc-0-9-27/run-current/store/5kfa2griyaqdm9jrkvj8v9s26hww0x1w-mes`
- Fallback status: `hermeticity: practical (no degraded facts)`
- Placeholder/deferred/archive rejection: no placeholder text observed; detailed part evidence exists in archived `live-part-mes-0-27`.

### `bootstrap/tinycc.ncl`

```sh
./target/debug/crunch build bootstrap/tinycc.ncl \
  --store "$PWD/target/live-part-tinycc-0-9-27/run-current/store" \
  --state-dir "$PWD/target/live-part-tinycc-0-9-27/run-current/state" \
  --no-substitute -j 1 --verbose --log-level info
```

- Exit status: 0
- Output path: `target/live-part-tinycc-0-9-27/run-current/store/0vz94d750q8wbylhln3sza1zwcwsvb1p-tinycc-0.9.27`
- Fallback status: `hermeticity: practical (no degraded facts)`
- Placeholder/deferred/archive rejection: no placeholder text observed; detailed part evidence exists in archived `live-part-tinycc-0-9-26` and `live-part-tinycc-0-9-27`.

Full transcript: `evidence/V1-early-stage-builds-full.log`.
