# Live Nixpkgs hello export-to-plan proof

## Host Nix

```text
/run/current-system/sw/bin/nix
nix (Nix) 2.35.0
drv_path=/nix/store/m74651b793zgyyvlk9gx7v1cl1ywslib-hello-2.12.3.drv
metadata_path=cairn/changes/live-nixpkgs-foreign-import-proof/evidence/live-nixpkgs-hello/nixpkgs-metadata.txt
```

## Export recursive derivation JSON

```text
$ nix derivation show --recursive "$DRV" > derivation-json.json
bytes=1892859
```

## Build Mantle CLI

```text
$ nix develop -c cargo build -p mantle --bin mantle
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Blocking waiting for file lock on artifact directory
   Compiling mantle v0.1.0 (/home/brittonr/git/mantle)
warning: function `child_blocker` is never used
    --> src/cargo_free_self_build.rs:1449:4
     |
1449 | fn child_blocker(status_code: Option<i32>, execution_status: &str, cargo_marker_absent: bool) -> Option<String> {
     |    ^^^^^^^^^^^^^
     |
     = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: function `nix_free_demo_claim` is never used
   --> src/nix_free_demo_bundle.rs:216:15
    |
216 | pub(crate) fn nix_free_demo_claim(summary: &NixFreeDemoMachineSummary) -> Option<String> {
    |               ^^^^^^^^^^^^^^^^^^^

warning: constant `TRUST_WINDOW_START_UNIX_S` is never used
  --> src/portable_receipt.rs:29:7
   |
29 | const TRUST_WINDOW_START_UNIX_S: u64 = 0;
   |       ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `build_receipt_bundle` is never used
   --> src/portable_receipt.rs:226:8
    |
226 | pub fn build_receipt_bundle(
    |        ^^^^^^^^^^^^^^^^^^^^

warning: function `verify_receipt_bundle_against_state` is never used
   --> src/portable_receipt.rs:324:8
    |
324 | pub fn verify_receipt_bundle_against_state(
    |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `verify_receipt_bundle_against_archive_report` is never used
   --> src/portable_receipt.rs:353:8
    |
353 | pub fn verify_receipt_bundle_against_archive_report(
    |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `RUST_UNIT_REPLAY_EVIDENCE_MAX_KIBIBYTES` is never used
   --> src/rust_plan.rs:121:7
    |
121 | const RUST_UNIT_REPLAY_EVIDENCE_MAX_KIBIBYTES: usize = 16;
    |       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `RUST_UNIT_REPLAY_EVIDENCE_MAX_JSON_BYTES` is never used
   --> src/rust_plan.rs:122:7
    |
122 | const RUST_UNIT_REPLAY_EVIDENCE_MAX_JSON_BYTES: usize = RUST_UNIT_REPLAY_EVIDENCE_MAX_KIBIBYTES * BYTES_PER_KIBIBYTE;
    |       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: `mantle` (bin "mantle") generated 8 warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 28.31s
mantle_bin=/home/brittonr/.cargo-target/debug/mantle
```

## Produce foreign import artifacts

```text
$ mantle --json foreign-import produce-nix ...
{
  "schema": "mantle-foreign-import-cli-v1",
  "command": "produce-nix",
  "verdict": "accepted",
  "accepted": true,
  "diagnostics": [],
  "receipt": null,
  "plan": null,
  "producer_artifacts": {
    "graph_path": "cairn/changes/live-nixpkgs-foreign-import-proof/evidence/live-nixpkgs-hello/artifacts/nixpkgs.graph.json",
    "package_index_path": "cairn/changes/live-nixpkgs-foreign-import-proof/evidence/live-nixpkgs-hello/artifacts/nixpkgs.index.json"
  },
  "non_claims": [
    "not-build-success",
    "not-package-correctness",
    "not-bootstrap-parity",
    "not-output-trust",
    "not-reproducibility",
    "not-foreign-frontend-availability"
  ]
}

```

## Validate with fake PATH (no Nix during consumption)

```text
$ PATH=fake-path mantle --json foreign-import validate ...
{
  "schema": "mantle-foreign-import-cli-v1",
  "command": "validate",
  "verdict": "accepted",
  "accepted": true,
  "diagnostics": [],
  "receipt": {
    "schema": "foreign-derivation-import-receipt-v1",
    "producer_identity": "live-nixpkgs#hello",
    "raw_graph_digest": "cc1bd58f0240218ba01bd31704e7263597fe335917fa3b0f3ad240afd30c62e8",
    "translation_policy_digest": "e7fd62962b19cec881ee06873c2f0abd94d0fda48de7bd4c68dafad0fb3ed3d5",
    "translated_graph_digest": "b22a493270d19f400c88b32b2febdb98b6a87857c273254248c2f04024fed8a3",
    "package_index_digest": "1fb8b0f89dbc07925fc00f7c8a700f846129975496eda619ac8d1b8920add02c",
    "fetch_cache_policy_digest": "1a64e6557d26665f7eef400f3fbfd3ce766a42b72e96b3cdc5570ff3f97b7ee4",
    "sandbox_policy_digest": "b80fdec27a0955fcb9748d1ac75b7da507959fdab796a2f223c6bc8d76424afc",
    "hash_domains": [
      {
        "domain": "mantle-receipt",
        "kind": "raw-graph",
        "algorithm": "blake3",
        "value": "cc1bd58f0240218ba01bd31704e7263597fe335917fa3b0f3ad240afd30c62e8"
      },
      {
        "domain": "mantle-receipt",
        "kind": "translated-graph",
        "algorithm": "blake3",
        "value": "b22a493270d19f400c88b32b2febdb98b6a87857c273254248c2f04024fed8a3"
      },
      {
        "domain": "mantle-receipt",
        "kind": "translation-policy",
        "algorithm": "blake3",
        "value": "e7fd62962b19cec881ee06873c2f0abd94d0fda48de7bd4c68dafad0fb3ed3d5"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/00d529rs5cfj1kwz79sm79qackf9gppk-strncpy.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/013mqc5ymx4cih72blz21l6ync49i3jg-expr-strcmp.patch.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/030bc69ppgsrcvxqxinlwn446dj86j52-fclose.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/037vygj573j4f43vw934yzayjzmxyqjw-mes-libc-mini-0.27.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/03h7kw2rpg8qw1x6bng65imsmilaz624-__getdirentries.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/03qi1b45nyw8bmswdwgkva41bh47z3x4-itoa-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/0442lf73plz43gym6ahgydmp752dxczk-acl-2.3.2.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/04pp7161mbyanbhclf56lwdgsk688ms2-autoconf-2.69.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/04sblc2j6gvwqf9hnyfi2ky7lk5rlx4i-search-path-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/07f6ncaqrcmwjki20pq4y0v1cf31zbz7-rename-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/07lxsila1467h3qxm5pw69qcvbbhcvbr-gettimeofday-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/07rmmdnlwybi90vx19shlmd37azxfgrb-uname-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/08fg4a5d85pnlagm8qrk1pga0zkyjsxm-coreutils-9.11.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/08fyipspfr7hz6mmp6hwh2a7hw35dsga-setup-hook.sh.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/0978f9k46zgxwpc8lwzk9fxajma156kr-tinycc-mes-chain-unstable-2025-12-03-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/09aln9yisls20di7x9wj1kiwhbp2gmm1-oputc.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/0a8p5fjb2va3jw6v84lisqsy0ygmfy8c-bash-5.3p9.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/0agi86w8bshcgjnmblg2dffwdq92shv0-pkg-config-role-hook.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/0ah1g18zgkkg0l9vhh8fz9rl6rawdprj-strlwr-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/0c1rgsk0dm2i9gc3yqqgks66qkidnmva-lzip-1.26.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/0cfizkf89lijcfm96bvkz5g45k07546c-musl-simple-program-1.2.6.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/0d9jqiq9g575wwcr0kv7b3f8l43ik83n-hash-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/0dl092g6chmw86gs84rfgvxj9nw2hnrx-libgetopt.a-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/0f9kajpkkz3mbcaj3ajwha1rw10iwzmi-fseek.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/0fclgm4r5525jlh8lv82r81kba6vh0cc-ln-boot-unstable-2023-05-22.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/0gy12dm9fs950k2pz0jjgwjr2whimg1v-bash-get-version-2.05b.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/0i89ydplmfyhw5rykihdpcr1ndki1bp3-patchelf-0.15.2.tar.bz2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/0ibwnm5f9q4r7j2d6v3239wgk59yn0xx-heirloom-devtools-070527.tar.bz2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/0j62lpv5g1lfxzd1i4iikf3ladai14r9-libtcc1.a.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/0ld69vrkv15sm7kwr2jrviajggb60d7g-update-autotools-gnu-config-scripts-hook.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/0mqi9ydgdc6as1hw0ydm7gg1f3yjgkdp-mes-get-version-0.27.1-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/0n6pqv072zwd9yg93bkjpf19bsf61fs0-coreutils-static-get-version-9.10.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/0nrb3h17g2hhf8ijisi7frcfvqwhya3w-coreutils-9.11.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/0p6bbqs599hggq6xvxn67x91pxszgqil-bison-3.8.2.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/0q1fkyw9jc9fafyswh9gkv48wph52yzy-isl-0.24.tar.bz2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/0rjn4fs706rp2zf5xfyycf1pfqnkjmgg-linux-6.5.6.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/0sm8bvvcpzcq48skj6glwachd0ydycg7-bootstrap-stage2-gcc-wrapper-15.2.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/0wj4x9yyqzmpks4ghvy8csnkga8j7cps-curl-8.20.0.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/0wybcx14pra94k7lxshah1knf12yiz73-mes-libc-0.27.1-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/10hawwrir5ybxn629yca8wjmn3qpn1zx-libgetopt.a-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/12aqc6cgq53348nizvwiybqr1l3h4wa7-stdenv-linux.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/12ia1mifcsy8d1yz20i7hyzrn1jpy8kr-gmp-6.3.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/12j2ylk2hir40iqqbia03rh5330884b1-strlwr.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/12kq01qr9jcw4gpgvzqg962gixgjg4w8-M1-macro-1.M1-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/130xp3xbhxwms383bbf7wpcvc6ksjjnk-findutils-4.10.0.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/15qfmzbh5fx4rbbsida19jyi9sqr8van-die-hook.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/16qh6i0fip1mxnydykdjhjc9a0x1sn4c-perl-5.42.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/18d4rgqv4dfdd3qq09r312hpzjjxwrky-gcc-4.6.4.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/19vlv640sa5fr6cpnr14s166v9vd2p9m-bzip2-1.0.8.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/1accm7zc9acis8qq72sidnkpnd4r49zm-bootstrap-coreutils-musl-get-version-9.10.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/1c2q0l0647l87h5d7ka4d7cgm383wdm7-gnu-config-2024-01-01.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/1cnglsb0a5017gk6f19k0hikb4f7nrpr-isxdigit-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/1f1ni464i1yi79k5ln2cp4wn6idg0a9p-_getcwd-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/1fihipa062xl2drcsm686iscb0zxy70h-curl-8.20.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/1h9r5qrmqbwfnahhlgv67k2b0gb54zd4-file-5.47.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/1hgh3ynq1lsicaxr0r5pdbvkbbhb0vrr-gmp-6.3.0.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/1i51x7j44ck1y9kc8sp5fk7c32sp85cx-strcat.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/1jsd2flxdv4s2lgg32p5l27rp466xap7-fread.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/1l0a69sq1vbdjjcbhnr0yic533yvkpdj-M1-macro-1-footer.M1-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/1l9zwpxb1njah0z7wvrpjih0src22nsi-strtold.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/1mpy7l8c822wij0cygkjydv5lv8lj0m8-crt1-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/1n7hk0448wph3jshbpli7l4di11zjp90-ldexp-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/1naynm0awl2hasqr3fkg3xndjpp23r6b-perl-5.42.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/1qjfvcwnwknh80nz8hdr1hcr7sqs9k59-isnumber-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/1qw6nj87xp2qd26y3qzxhh495rxm7ysd-patchelf-0.15.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/1vgxf29abq71i0mh3m5839my4664jfmh-tinycc-mes-unstable-2025-12-03.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/1vkbzh19qql99n9jv0c0ffqgyzjqlwhy-time-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/1wn9ym4cmvkn6nnbb7p2n1xf8bigi01x-mes.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/1wri1ai0flqq0jghyk9i4fdw568h0xms-tinycc-bootstrappable-unstable-2024-07-07-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/1x6yq984878wwl7xif86kj8z4i5fn6yv-heirloom-devtools-070527.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/200n6nl9ykx5d3lhbjpvxm9f3b81pb23-gnumake-4.4.1-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/20i1igy2hxwh5i5nrakzj41cmaafsm6z-crt-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/25xm28w9xrk9kgn0wmb40g3fc4l2z2w4-bash-get-version-2.05b-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/268p0jcm6cm8y48mzwfm47ylrz61ldg5-vector.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/27x328r5v1pwb73hih8b612c9f55psmg-tinycc-boot-mes-libs-unstable-2024-07-07-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/28incrmkqbjms8pb4i8qkplsisj49075-gawk-get-version-5.3.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/28x04lk2i4zy13bidwmj5ylbx3v880n4-tinycc-musl-unstable-2025-12-03.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/28ynjcjss9m1w5wx6zdbig3k0551i5hx-gmp-6.3.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/2abz4inzlk5dp473kdz0xlk8wkvlkgdm-gawk-5.4.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/2adhxc3jvhi49x2i4076yk9k97z831w1-free.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/2ayyjhs9mz8nb1yfml9lsmfnd1lkw8h4-gcc-15.2.0.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/2b2v62sgah3hyxxbvqgnww2ifckj93ps-libgetopt.a.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/2bj79bj56nqc3l0x4j9578qkm193j3fp-getpid.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/2bkgnravj3d7ybvh9704visgl070gnqd-glibc-iconv-2.42.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/2f88x1jf48lbk7wzjvfvzs5bsh5xbqph-libxcrypt-4.5.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/2h2jx5jrglyqws10sa53n2bvyjqpi9mp-putc-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/2hcix19ddnc0gssj508dfr9yzw29mkvf-linux-headers-6.5.6.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/2jwh5pdbg0mff2bhxvc2vam8bm05wa08-catm-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/2kbbzfsz1hzd21yj0h989s3z40cgrp58-stage0-posix-1.9.1-source.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/2kpjq9x1yy9zmq0cjjwkqw7l8mnjwigb-raise.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/2rmk35vbqb6q9r4aqmsz7iicmi44jx6d-automake-1.18.1.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/2rrbpy9sk06kcqxar40iib4d6vxgsgr0-bootstrap-stage1-stdenv-linux.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/2s8mrpwr478x64dsj2916a64aldibviq-libtool-2.5.4.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/2sakdmbkjgy4zzml0bmmmghlx8pfm4rl-gc.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/2sijrnn5rfzwq8hla21y9agsza9x2lg6-nuke-refs.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/2vknf2sz2fdbpqkpsph03x5qyp7pprzy-dev-tty.patch.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/2xy9v2qajsvlhlq7wdfscvc2658qvkk4-getenv.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/30pxaahlnn3ip5c4wkc52gk6k2b1i4xy-execvp-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/30vp5c32rqb6zhishsl4jz4bphdyg4w9-gnumake-musl-get-version-4.4.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/3298jfza8zv78425rrpr80xxcnnrbgr9-bzip2-1.0.8.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/3299wqz95dmz5wrh0fh2d60x9r31hmbd-execv-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/334rm4dwxknvj9h9mjy319wa1bkz0id8-isupper-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/33fk6f3q18k6ifvg92vgsnmck9dcsqli-hex2_linker-0.hex2-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/377arxf1jfpbv6shjl3iczj9v07v07h1-locale.patch.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/37zapxlhmx12r050xsjw39n4vw3nck3a-mpc-1.0.3.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/387n5gn04pv4cjjlrxxh4plc6c60ifl6-hex2_linker-2.hex2-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/3afkdgyk01g7y052ikq8gbdvdd5i81hz-patchelf-static-0.18.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/3as9iwgnarigh1ffsdb6frbhl586yli3-mprotect.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/3cmpdq0yp00gwfwah7572s0vmi8sa5zb-attr-2.5.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/3ff9aq5jx5hs2fdl2yfqpl1nh7dfkh68-zlib-1.3.2.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/3gqydiy1c03pjnci7iydqs22fzcrm7zj-bootstrap-stage-xgcc-gcc-wrapper-15.2.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/3hdx2i1mbzlrff0d52mkjsh8nbh77cbp-nghttp2-1.69.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/3hlqgwk5mfjp5yb5kw4j1x6g8gjwhh0q-putenv-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/3k613fvf47ib8g1r2whcz93z6wlczmz8-__getdirentries-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/3l89d34bc9vraawapkrxpxfnw6vh3ryq-texinfo-7.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/3lpzywq15v6lc464sls2hpqn7rf54csa-bzip2-static-1.0.8.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/3mqqa9m8hvliz348d6ikb0jg0lj9fsqh-gnused-static-get-version-4.9.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/3qg3iia0yapdaqy7wh3ysih94vi3ng0z-mes-libc.patch.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/3qi2a5212jjx0jn49vwxqlw369zabl3j-vsscanf-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/3rx2w7lxnr0ndimg0mavnb4klbm04amd-glibc-2.42-61.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/3siddy2kzlx0pj7h8s15jgqdicg3j3lr-pcre2-10.46.tar.bz2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/3vbridlri44i5lnczbcxqlwgzbxk3pg5-gawk-mes-3.0.6.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/3xs5n9hmbbmc5qis9zzs764ilbjk0v72-libidn2-2.3.8.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/3yvphn5v3laa2qrvllg878ij47wx92xw-tinycc-boot0-chain-unstable-2024-07-07.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/3zhi7fn92ym1ds0n3yybmjyjvksypi39-xz-5.8.3.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/41kp7bkyk4g0lk2xq45hiqn268nbmx8a-gawk-5.3.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/42v2kyprk2gsp40xspc0ssxfi807qcim-gcc-cxx-simple-program-4.6.4.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/43lgm1xpplnh04c5bszki39wv48ba9mc-module.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/440bjlrs4dxc1jpcx55rdnncm8x0s1gk-pkg-config-0.29.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/453w5gjvblvbv7sflcjxvrwnm76j3kza-mes-0.27.1-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/4683d3nfmlf0ff7gi0ihfdyqsymrn07s-toupper.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/46c2vwwb9lcd8976mmwaqiy2glsh8a6s-M2-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/46pcnsn6dy8jcq0ihi5zn2n8lz6dn777-ltoab-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/46q2kmbs4amzqac5achpxkr681r5r95w-ldexp.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/46s1ihj1l58kgdwy02vwvwam0z5vx1ar-getc.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/4745hdf59rrg9fanw4r357m8maqskhcp-strcpy.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/4a6f891gzagq63mjc60r102pn8pfmrhy-xz-5.8.3.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/4dld0775zm21fbclv6rx3v6c6mfqvsvf-expand-response-params.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/4fi5xvzq940y4fyag9sf3swmd04x23hn-getcwd-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/4hfq6p045k7knv86fc57kq5cnf2djia6-tinycc-musl-unstable-2025-12-03-libs.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/4i7p4sdfbaw3y9lr963ck03y409j3lq9-getdents-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/4lbcc0z7yzs1ml4al2iw0rfnv58gn21s-bzip2-1.0.8.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/4lh6y35gcg0zx5m9a1sx9hsp36xn1br0-strtof-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/4m76wqg80yx0yg0wv4z6kqc1y78yd58p-tar-1.35.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/4p5vrjdw8zl1kc7fl593dm7vrj6n7yn5-nuke-refs.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/4r537r355zybpnj6151m51l3c88gpkpp-_read-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/4s26xanhs14v8nvvm0r525cva3l2wl7s-struct.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/4wl34cclbr0f19xnsfb2jfsr10xz28kj-heirloom-070715.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/4wzln1qrnyz1zr6arar3813z3mn5a7rn-unlink.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/4xxv2r7w1zpqicpdqflas64dwvbanh0k-getcwd.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/4xywz0vr941lrjgspzvg5a8agl3qgxr5-tinycc-boot3-unstable-2024-07-07-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/4y9kzg6h4wxyfpvw3ml99ia9cgn8p6n7-perl-5.42.0.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/4zqkcs8r6kgaap0wcmwpwkqmr6ac5xzd-byacc-20241231.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/52m5687xb0sxg1dmw82mg9mlqc8yyp8d-wait4.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/53b8cx0879bqmw80q6jhl2q8v26lg3dz-fstat.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/54j8qmc5ra0m7dwhqxrjswwmlm5w6sny-isl-0.20.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/54jdq8vp944qn49b18harrbjw34dlxcp-bash-2.05b.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/55d91flpk4776srkw66wdx69wll7b5zr-libxcrypt-4.5.2.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/59pgig863abfc3p6lmz20vh8gjf2nsv1-musl-1.2.6.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/5aa45k5r21ni942hcdjlkgigpyzg7dbg-gnumake-static-4.4.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/5b0v0cj57rfjpqjrc4k38k0si2db4jyi-islower-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/5d3cd4ydvaafcfvq6gfjxpazbik7pfg7-utils.bash.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/5fwbsmcai8jq0ji1m6mh1v7i14rgxspp-tinycc-bootstrappable-libs-unstable-2024-07-07-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/5hpxjqykbkdk302khw74c9vpsyybm71a-open-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/5hyl2z8p5ywhnz1nmlx34hb00rah5x6m-bootstrap-stage4-stdenv-linux.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/5j58nxqnf1npah00yma30515vxl9rdfa-setenv-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/5kp3zisdmyvlrf52qxka2rdhp1vj1izy-tccdefs-unstable-2025-12-03.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/5lviidinj71nws31r1c7gjlg5p80lkjs-gnutar-musl-get-version-1.12.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/5m6vm7vkilsgas517yf8f321k5r3g1m3-bash53-003.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/5ma1s9g8s1r48yaqy114hj8n9bz0nd7b-__mes_debug.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/5pq8iapp8p7zwipgwlfzbq3c5ivjali0-write-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/5qbjq17pking8vfi5n2d1vsm6hknwdgs-tinycc-unstable-2025-12-03-source-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/5qdc1kr8ljllm244bh0jy3bvmz462nv8-gnused-mes-get-version-4.0.9.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/5vfq7hk4md3y9fnqipn07cmiyi13bqlh-memcpy.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/5x0ni3m200vsr0a38nlhp6grwspa3bwc-expand-response-params.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/5y5rhx8xkvnb1rv27ygq7qx2jh721n6c-fstat-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/5zga9hbg59nhww93dfwqdx6xc1q81pif-crt1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/61s6dgr2mn6jxmz42zxmgd6b105gmwak-libssh2-1.11.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/62fjvcsvb78qc4nl1ip1bwkdm2g4x9mj-setenv.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/63ylg04cgw4dqrlxy6zxx58l0mvc28qf-__assert_fail.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/64w55pscmwz6wl53nh5x11n5qag8gylp-sed-4.0.9.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/67nky23lmrxxyrmf2sd519iplfdkqfp9-libc.a-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6a6diq15i1w6piy38bf29s6y8sjfsxa5-bash-5.3.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6a8p8kb4charqds4z1zf6ka1gbj71p1n-gzip-1.14.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6b0xnqkbv7qnbbbc7zhqwahygjf7mc2i-findutils-4.10.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6b29gjz7rj4mw0ch0vy2m6qrqipz2bbb-pkg-config-0.29.2.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6bra21k8nb03inbdy8qk2ashyj6xhcdl-syscall-internal.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6brx5qj5qw0rj68wcz1xyxi0sdhv82n6-posix-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6c9czp98fd8gahfd13lkqhpq30ygckqm-lex_remove_wchar.patch.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6czr4pwq85m5206gfjxayx6c971id7sk-ferror.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6d922flrrliv0x67xmwni82a6yhrkrha-ntoab.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6dkd05snil6kxm0ml3ljisxrigr5wlza-expand-response-params.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6dv37m9j2n32qvhwr3ixlpranz4kmpis-patchelf-static-get-version-0.18.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6fq4fcp7sbxsz1yvl05qx341zqkyq5va-tinycc-mes-boot-unstable-2025-12-03-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6fv8xbk8ghibgzp7pvsmhv884fhip6an-binutils-static-2.46.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6fvs3jh21nkzmrsm8ka5yi7pglzy3pl0-sscanf-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6h3msgmh2a9l833qnsa3qxwagybrf2fp-crt.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6khyn9406a1k9b1wi5cbw4wysffrnav1-zlib-1.3.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6kxn0zj4ipzpg2p4csxzy64njknwydy6-autoconf-2.69.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6m7j106hrrldsrqmq2cwlf9aj492n4bz-lib-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6mmhdk9d1mw6grbi73sjmvbyv6vsl79l-crt-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6mq3813sfp0vm3c9nxcj10lc26kg9rgz-tinycc-musl-simple-program-unstable-2025-12-03.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6qw1xlhvhf6jrkygxgqc68kiyclygb4r-crt-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6r1diicimr9iaf3f7hxq0b0znqxv5rq9-tinycc-boot1-chain-unstable-2024-07-07-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6rncbxavqydmqxgg4l5n4wyxl75f2p03-update-autotools-gnu-config-scripts-hook.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6vf6b238jhhc823w4gw569pbsrm02ryl-binutils-get-version-2.46.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6vh5kn7pd1k9x103kbrb66yy91chanz0-utimensat-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6vppdb8z7bm6nsvhmbp7ajcbwnjdz39i-ioctl3-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6zgiwrc91phvp3xb1a09d018l02ibx7j-utils.bash.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/70dvs5gmw3dafy22kfxy4k4pjprcfck7-zlib-1.3.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/70ph6wa755qr51i6m9qjsj8j4mwq0r0x-utimensat.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/70pnb6xgjxzj6v6v27sxfjsbbbpaa3jy-fopen-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/71d53986b7wykj1ig13nh2wk81c7gl76-gnu-config-2024-01-01.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/71n1agkwjvki00q4v63dcn6rsakv49ak-nanosleep-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/71nyals4hficsk0xbbwsi3ckvchjc2c6-gawk-5.4.0.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/72cmq1ms8qih2svpnw3v1rs9a035h2gy-tar-1.35.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/72m9p36fk1v6a4m3989rw32kh45ssxf8-realloc-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/73k3dkk99vv55xybmxpzq958ymg4x2hi-patchelf-0.15.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/75x4zjzb2ydgf9bkizf7fndl9jg5vyl7-libc.a.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/786ngprql5h6i8qcpzm8987rf5mvr6km-strcpy-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/793jv57r8l6sw0cxbl8zbygdgs8hbxsv-gnugrep-get-version-2.4.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/7aclmj1lii828pwwinxwxja8wqqsip0q-mes-libc+tcc-0.27.1-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/7bq69blihkvvqq5np4liiszw674bcqqj-tinycc-mes-libs-unstable-2025-12-03.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/7bw87grjq72s0rdp5j3422lj9c30z4ck-eputs-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/7byvrfyi1f00i8jxsc1q8z997j9r10y5-attr-2.5.2.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/7fsmyydkmaza3z37m84xdcg1r7dz0rmy-fprintf-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/7gl04lx18jp6m4m2f6b26xmz4yd5z76s-math-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/7gxscgn6gvpczrdy9vhlpahw130p6zk9-module-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/7jyjq8p67kr0n0v92w1kmwhp5mvc9994-fdputc.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/7kfxq96dxafy46537pbpr586f1zbqxvz-findutils-static-4.10.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/7ladh3xrna3k85c58c1qps7djmprjjqp-localtime.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/7nq167q7zsw74gprqg48hzxzbcb3z42p-vector-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/7p1an6982kg69v5x7c249wysndkxy1pq-core.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/7q387c5zr2rzn1sfl6p4l0732ag834ig-ftell.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/7q3wpyfk2q0lcppd56x09ac34s5q7hfv-libtcc1.a.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/7q7rg321qf9gv1brx0gn4l76h3ycpcq8-texinfo-7.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/7syw1g6xr2zqzv38cpfq043pz9n8qffy-memmem-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/7vxf7g8swlihvbixhy0l6r3l8cyqqb11-automake-1.18.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/7w302d1c7mxnicpgqgh32zbsb9kc0f9a-math.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/7xw1yyq7h6ngza4cknmw7kcc6pa9f8bq-bootstrap-stage0-binutils-wrapper-2.46.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/7zq4yvhvgqqgn7ly5gasnlkfpm838k6w-fwrite.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/804g2dcnwn4bhkbf98ni9nmq4wghbxnf-close.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/8098ic14nbzr4vvb68k28ylxrykacpg3-pcre2-10.46.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/80i37dl6zigax7nh3w81ysgbjjs6j3gl-tinycc.patch.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/827wd86bx54ijslsnncg2vywdk2gq40d-fclose-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/82nchyb4xk0081577hr776lkywkp0imr-diffutils-static-get-version-3.12.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/83hnwff3zqi6b01b1bq409p05amxvjc5-vprintf-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/83kd55avbcj2wx0gc643yhph72ahdacn-hex2_linker-2-footer.M1-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/83x5nz0d278f729kj1rmwmgqmrcx494x-gnused-get-version-4.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/84w0zzmqzl2bn5s4rx7axsh9znws1rdf-mirrors-list.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/85b7b7c8bp00hcbvw1pv80s0sbv1qnhd-libgetopt.a.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/85fl7xw30igvdd4237dvqd3qx692kd7z-execve-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/86x3nhabnia98ihxzwcvmf19hy5fby1k-readdir.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/896vsnl7w7rhd17b65j8sp3g4kc0zh35-texinfo-7.2.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/898agdpj8623c7hn7xwm4qb3nlkckb2l-expand-response-params.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/8axs2x9qsak35kj0jf2ynfbj1wnv3kg2-libc.a.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/8d25x0aas65p79rc5kzr92vlhgq1w2ra-tinycc-bootstrappable-chain-unstable-2024-07-07.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/8djy268anfnnhb905fh81l734qz66wyj-gnu-config-2024-01-01.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/8f9nchqysay8mhymkghvb71fhz50g6al-minimal-bootstrap-test-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/8gjrq7nfvdhlpbc4licbbqfn291ikq85-fgetc.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/8i626qf2c7brpja4dj16wzyipg7i5s3s-autoconf-2.73.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/8ijrrxc1xxi3drxznpn7h195zkwpnp3g-setjmp.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/8k6l1754s5dxz5i5b242q27xhmhbk3d6-binutils-patchelfed-ld-wrapper-2.46.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/8kvh3k82j18fi5abj6bmqahnrc5yp680-strtol.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/8lfpv18kcxvb7ipxrbw10pahkg5iw61k-linux-6.18.7.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/8lp51hpsccqbr2ydb5wv9qhwhzb83cnd-__init_io-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/8m1p5s3mfpbp48ihlp8bnp1ssqj6k409-main.mk.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/8mlyq4wagf0gv2vdiqzpfg8ld4gqwbyp-fdopen.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/8psl70bgq47wkm69p5dm6kx1ylnpa6b8-_getcwd.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/8qpk6p6zrn3r72935w0rpgqbxjvwxjps-add-flags.sh.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/8rlmsvgdhb7sqgc1ry2hazkk2da5y6hm-tinycc-mes-libs-unstable-2025-12-03-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/8rw2jmn1zm1ygz0zzs4b82mxbgfczbw9-display-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/8sx493k2xnfwakm089q5w1g6pg2npr30-getenv-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/8szf5lgh88ymz0wjd7f62lww0ki0kl5r-memcpy-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/8w6mg2vwlxq16618mrphxkiwafalfbw9-pkg-config-wrapper-0.29.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/8y8aadripzcsxfvg4hsdacwcgw6ypm50-strncpy-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/8z0ip4s3i9ghw3yljf19c1mhvz4571pg-gnugrep-3.12.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/909rhv815886n625rjzvb6wlc1bak9d5-crt-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/90ghkblwz1x58v2bf4j2d2qk7p5ln594-wait4-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/91rcw4mwf0qzfajadx3zg5j4cmzqh9nm-glibc-2.42.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/91w94nj3l6sqaacc5vsjwadprsby7qkq-crt.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/9211kwvmqxz3424zn1y2md1vm6mw1bp6-gnu-config-2024-01-01.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/93hi8ypxk1ihc51l17y9k4bsiybr8y27-setjmp-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/953qy5sr53cjhdjpq9kai6kv0nx4243i-openssl-3.6.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/972r80d928xbv1ymadl6bv7pjl7ldlm5-crt-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/9859v83l3ldymlgh5y6d69b48w5d9gds-_write-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/98bvwkn5mp2g5nmb2007jvfxbh79z35s-gawk-static-get-version-5.3.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/9awbldasqn9q9kk6pjsg4v4k0cmp2hn2-wait.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/9bv6x8rkxfxdlzkqv2c35qr3jvam66sk-libc.a-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/9d2q1bl9997dm8502k6cxg5r8r85pmdh-kill-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/9dh6yj0xma2rsc8c2ylqhqvr6gvy6g78-strlen-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/9dvsrxv7mw64yqa5zyzwnws29kn4ry20-gawk-5.3.2.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/9gwpvw0ssr7xmfzvy97c6f493fhc2f7r-diffutils-3.8.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/9hjwncik9p8h51s705cvh9262f6jm1i3-eputc.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/9ihpw1hw617rlpx9mww1w1cs7ymaa3dv-tinycc-bootstrappable-chain-unstable-2024-07-07-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/9j5mh6n8bwswbwm4ys9nlw6irfzgv4lc-bootstrap-stage0-stdenv-linux.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/9knysaa0w5wbkbi2c1hbvvf7gjv8iqfb-blood-elf-0.M1-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/9ls1fxazpaq0wy94djljyqlwhnf4h6z4-bootstrap-stage2-stdenv-linux.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/9m9m75zvf97rz7f9cml70i1gz2i259d3-getpid-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/9mj2rqzivmn0skmx8kgy0qsrm8n2ph0z-M2-0-0.hex2-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/9ms5jx08n6hdg7ji8fxln4lsfbzd92fa-gnused-mes-4.0.9.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/9nh28gh5xv970hclrry924m5s9a7pc04-hex2_linker-0.M1-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/9qrfjhdgqij7dg7pvy85yw848hchjj86-M1-macro-0.hex2-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/9spyn0z9nrcw03qnavkybz7sh1ngmsmj-python-setup-hook.sh.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/9vfznpw8fbrvvjaky8rkc8lbsay31x56-libgetopt.a.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/9w2ljzcaf1crccchjhm2rsqqsnk9vnv8-mes-libc-0.27.1-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/9wbfdc5fqjxlyl27xa10759yvxcyf8ym-snprintf.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/9wgdxlkh86vnk9z6lpnvksw0pz9wj5b1-opendir-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/9xrg0ykb88mfyf536qk8rns6bxp1jmyx-ltoa.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/9zz696rw3sw7hjgavl83p3pbps1fn8g6-exit-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/a07893l2w29xc0jpqyq7gmckdqd07974-gc-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/a12jwdp6swyb0bzcbmlixi7h0hn3m8qg-mes-libc-0.27.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/a138sap1yaglvngipkh6lngvs33fq8wr-bootstrap-stage3-stdenv-linux.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/a1g2j692marrk5wlyqhxlvclv322gxdr-stat.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/a46fqsk4zs447755lavf26kp3dxpk24i-gzip-static-get-version-1.14.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/a57hhkslijbbzz65khax4qxc2gspx6g5-isxdigit.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/a609q8xwsiqmbq4vf1r2cg9kmw0h9nxc-mes-libc+tcc-0.27.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/a6bsdkrvajh1baiid0bf9ylxd47sv20n-gnu-config-2024-01-01.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/a79snlqv6f8b1vpgffljdnl9hv961wmd-bash-5.3p9.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/a9xrwk0gp9ni3pljp5nz5v0ch73gj5r6-openssl-3.6.2.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/agfp8f5mwgq3pvqla0gsh6zpycjjjbff-closedir-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/agsl5l6inmhgyggscpcn5ng2f1943b2j-memmove-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/akag406i2x9yrv04yz1ydq9hnh63pkks-zlib-1.3.2.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/al3c2iqx4nqjcl4d87m5x1sxnc7m1n0f-assert_msg-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/an791rac20in20im3mlfkirch2pwaw0s-fputc.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/anxcf04gjylc3lhrymdc4c3hmkhkr84g-struct-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/as2v8bpjgrwp7mji129lcpbhs2w0805a-__raise-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/asdvhbkjhgwcd51lafnwqiah250299j3-dtoab.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/aspfvh0wlsi4w6kva34p3m9kkcxr7vym-sort-locale.patch.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/avf6vbj4jmpnn5cdsjyj2693samnvqsa-bash53-001.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/avmqlilnsjmv8lg7442mjb21qx981d2j-bash53-007.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ax275rpcdyhncynghzfbz409pn1r9bjk-mes-m2-libs-0.27.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/axkmfgaz8nvjw8q0l0d1bhm7glqjjb7j-nghttp2-1.69.0.tar.bz2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ay5c52823flsrmd8887q1xdi9igxq7m5-strupr-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/az1hahmwb7l2hzvgqy0w6rvqbln04769-xz-get-version-5.8.3.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/azx7db7w8nnrdzwz1hasa4a1psi475r4-execvp.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/b0faqgmz5282qmis5v2m13b7bsc0y9jg-gnumake-4.4.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/b147bn8sgyn2s58vy3cw59d89vrd482g-mbstate.patch.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/b44mdg7iwqp8z0szrncb0asls9q2l323-globals-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/b4xxjawvjh9i025i567xfv8lh7r8aix7-autoconf-2.73.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/b61k4wb1f0c05y70qj035s9n98bsydx1-tinycc-boot1-libs-unstable-2024-07-07.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/b6266kgqhlnyhdkdp7027kdmsq2aw51l-libtcc1.a.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/b6yxpjizcj55gzmm8vpjzdxgl15246qv-byacc-20241231.tgz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/b9bd2bl75650mdckkiqshw1bld86vn0r-heirloom-070715.tar.bz2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/b9jn5s5rpykzz8q7isifnsx1phmwl175-sprintf.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/bamwxswxacs3cjdcydv0z7bj22d7g2kc-config.guess-948ae97.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/bbbyy52mymn1ghzh8kf3h0yxsgddj26d-ungetc.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/bc1pgwnxifikxdvgbag1i396jp1klzgx-ntoab-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/bccvb6y1c2ssd4zvmg7iag47241lirh2-strchr-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/bcl7s5w3dyj4sj5v3l2v8qh78b0j1pm7-libgetopt.a.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/bf7qin7fa9d8vpd5dna84yknfd2qrl35-sprintf-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/bha00fzgyxii9w9q9y3w14ynjzivi9k4-open.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/biz5hn7rgmvdry6kan1kr31v6mk6gavl-strtoll-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/bjlz9wzdlh767w232y6n5pjds14raw5h-dup2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/bkijx1mlrhppg2b1zandixpz9w2lxnvi-m4-1.4.21.tar.bz2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/bkllg3nvs9krj4lvxirs7321ckmf6a0l-display.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/bkmjhag9fkyxdmh3khll3hd7fbjqz5zl-nyacc-1.09.1.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/blibvc5hp8ky3qi4rpwkxcw501akd418-oputs-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/bll3p7icr7ylyxprj5n60j2r8bkf5qa3-mes-libc-0.27.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/bm21lvr3ishd7qqkhiyq7lqjz7j4sdgj-libc.a.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/bmlz5dr8h49pdq0dn4xdxsw3r9z1wx1z-remove-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/bpzkd3r26cl6hwr2v0y7giswnd9chr7x-memmove.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/br3yqsjafxyhh91879brxq1xzbyjwlwb-ls-strcmp.patch.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/bv4a5fw05q4ysq24wy0z06bd5jysq1p2-gzip-1.2.4.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/bv9pjdxwgwphc534rna0d483wvjxcmwl-heirloom-devtools-070527-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/bwbl3xl91p3p02ylm29nkjjhyqdsr4hs-umask.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/bx85m89qvvn4dgmnb3rafz4ffnk41rkx-kill.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/bxcayhyg3gnlg7zfcvyrwdb9cwxpl09v-patchelf-0.15.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/bxv5sgys76rnsjkp7jl41n4ml67j5l9h-bzip2-static-get-version-1.0.8.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/byylv1iwrisj83mlsi9qphsslfy20idd-autoconf-2.69.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/c08j9sk7k3dqfyllv0l5wcp80001rjpm-tinycc-boot2-unstable-2024-07-07.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/c2p74ajc9zfg6dvmv48f4v1xkr82j7ij-time.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/c41rl08zcn7y0d461qqsig8vv5xbzcyp-bootstrap-stage3-stdenv-linux.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/c4q67q5gj8f0y6hb33cak3f7sjpq63c7-fsync.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/c672ird0l7wv8dr5idknpgnh1rhzlnqf-coreutils-5.0.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/c8aqhk9y238r2swcqcw6dm5qh26lik9f-chmod.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/cblshqv4b81g5gxis3zpb4inagkaffi1-binutils-2.46.0.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/cc7v7pw1q359kfnkw3g4mcx0dldycpwy-glibc-iconv-2.42.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ccs62zg6j0khwdjgswyv9hnd2gwc0l8y-isspace-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ccvnn15aaggbqby30qjzyp17x49bjjb8-core-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/cj22vbp3vzj8xxw4h8wdf04gqfh4gv9n-strtod.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/cjzncxj18nk2xpm15gnm3a1lgrqbicpn-tccdefs-unstable-2025-12-03-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/cm0rq8z9vapni52r76hsn0r1y5rbq227-calloc-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/cmgw7lyp6b47wd2xk9lj8j6vx985k1p0-libtcc1.a-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/cqm0q08dhgvsdq3hvj85lxhq173x0m6d-tinycc-mes-boot-unstable-2025-12-03.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/cqq3w2y9gxzvjsggwxjvl0viwypnsfca-gnutar-1.35.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/cqzx6j0x2pa2p932j12nqw1cbz7mp27a-builtins-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/cs4lzs1qmk23nyzgf751a2a4kb7y667v-mes-get-version-0.27.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/csih3q578qgyycfjbypmnkmrxsg3rdpf-libtcc1.a.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/cv6mdrwwydc2927kkz3i29b6k9k639bd-printf-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/cxh8cwlp0z4a7xl6bk7zsqr7ci1hzdq6-make-4.4.1.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/cxpi1vdfflr3zh9xhgsqz44vqjk0jfk8-mpfr-2.4.2.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/cxyal2ckly5vlzvyxkggkl2y9xirz2f6-__buffered_read-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/czkqbq1rq0ch1lpla9bgsgwarc6hrb72-tinycc-boot3-libs-unstable-2024-07-07.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/d1simrxi41d15h88m043y00yd400fpd4-cb41cbfe717e4c00d7bb70035cda5ee5f0ff9341.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/d23sd66xkg81wd8j7m0jgw91wmsafspp-fputc-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/d4aaks7jhys92cqj8b0c26xwqhqijaaj-vsnprintf-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/d4p16n5n0kx7hss8kqa8dmrlf8zxf356-chdir-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/d6373avfzg9j0c79mdd4q8rva2izzl19-sigaction.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/d6dkk85sixrainacqixn91kap9z3z5n1-tinycc-boot3-chain-unstable-2024-07-07-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/d6kq0iqwxc8mff54r3ffa2kqkqkif008-binutils-patchelfed-ld-2.46.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/d756f43xmzg11rr9g98n77x6ah6w4kcw-libgetopt.a-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/d77vfr3v3vvbs9v964ngmxlq5dpylgnm-clock_gettime.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/d9hngw468h7b7fpcyx284kqwdgcn3gzy-bootstrap-coreutils-5.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/d9q3m3dkpm5f2yidcnf39bxqy3xniw6q-M1-macro-0.M1-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/db0z1k8vqx455ypsma6c8s698kd2fzxc-bootstrap-stage-xgcc-stdenv-linux.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/db1yx8fllcas5nni56p9fq7dghrki2gm-getchar-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/dbasqqw4gv4864jxfc7i9fir1ik1h91b-mescc-tools-extra-mkdir-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/dfly1dzyy8qx8y260mz1cgq0p2wnvnr4-tolower-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/dfxx6gjnya5f876sf152swznsacb95gp-vsscanf.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/dgpv1xd3s1mj9vc7z66610s5p1ap1k4y-strncmp-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/dhjhlihqj08f3fs1cvsja0fims0dqnlw-raw.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/dhkwk75n73r7622d36chy0j0pr8ak7vd-findutils-static-get-version-4.10.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/di9nrzh0qk5z7igcimmzxg8xx4q4s83v-string.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/dinjsy9j39gclj9hk2qaf7l296bj4053-binutils-2.46.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/dk7swnidjivlr8j15vjn7nww8aw3i4pp-crt.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/dl3gi6dn65ss5py6mnm0iqrxvcjhl67h-toupper-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/dl4wsj8kpyb1whdhhibqvqqx3c6rc60g-libidn2-2.3.8.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/dlks1y7nzrw0z05rbcrr5cg7lrdcj7k1-libtcc1.a-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/dmlvfmqi0x2hdzqb97fic9jrk0bcany0-fdungetc-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/dmz6h2m7nf6n698mm14i6zyms33dhaay-chmod-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/dpr00azdamvsr009q3jin35dgkqys66m-tinycc-boot2-unstable-2024-07-07-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/dpx3qn66di2g74234vi7xq4iizknmicd-gcc-get-version-4.6.4.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/dqg4r1dzlcq4zv2gmhdm2m2cdirchy2n-tinycc-boot2-libs-unstable-2024-07-07.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/drh372b18rwrl15nrww5bc105xpwd3kx-blood-elf-0-0.M1-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/dschwsfi21lwar5ldv5wdqnm1waf197a-M2-0-0.M1-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/dvl148vwwn460fa0zq13z8wlyrd2fz8k-gawk-3.0.6.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/dyffnfdiz10lpiraxazf9k9chkx1prqj-M1-macro-0-0.M1-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/dysgi0lbr4lwby0csjpgjx9qpalzwny7-libmpc-1.4.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/dzkfh6l54awc99d4hb1yi0v4mp1lacwa-gmp-with-cxx-6.3.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/f1flrx1z8dp8l9i1fq745wmbikj3pjlr-glibc-locales-2.42-61.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/f2yqpqsk58h7c7a0sv2q296a1b88svqc-bootstrap-stage0-glibc-iconv-minimal-bootstrap.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/f4svikqqff9hghyrwn25bq7kf2kid2dn-bootstrap-coreutils-5.0-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/f5g8kz53hxyz1k9q4nlnp7kpvik4yd15-rmdir-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/faa0yzm7giga1x8llmhy2sxxkh6j89f0-strtoul-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/faklixjfqrpfwz3z513s6p9lmk2qf3mw-mescc-tools-extra-cp-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/fax2439jq23r7zcxs44axd7aqzmwbnxf-gnused-4.9.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/fb3zzjczy1sbr0w1n2818d6zkdpkad90-diffutils-static-3.12.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/fbfri313mjc939zmfif03wrhj4nr5f07-add-flags.sh.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/fc9f5c6w5f8ap0gra21l3q7hvcgr9wnr-globals.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ffisxdinfy53xam8vhlqv8c5jkj29yi9-mes-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/fg5yn059hlcxgb0116bq8s47rbg4gzs2-M2-0.M1-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/fgw96sp22039kys7pyjaxns4qxckh829-syscall.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/fj69pic5k2hr161cw3spagly2cspp3x7-main.mk.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/fk7844pynkdlf9121b12ipbrf55c9g9h-fgetc-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/fl6mqjpa5193l28xwgdla6mj4i186568-tinycc-mes-chain-unstable-2025-12-03.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/flxbh2drlvipjlaabnkzij8dxcv56ayl-nuke-refs.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/fmhwfv6ikfd8wsg2mha7rw8nmdc4jxrx-hex1-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/fqdmyb945n7hx152y0n26bi0b0q7i8np-tinycc-mes-boot-libs-unstable-2025-12-03-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/fr4s6crprj60n9wwcyf7icplz8dxp1ls-binutils-wrapper-2.46.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/fsh8qdzcqnfw96hhy6ssy2vw0ljjsc8q-malloc-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/fslv8fzwwhv8zcvli06y1p66w9jh9b06-libtcc1.a.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/fslvmm29rxbvmrl8mssmh3f252k65xmd-fdputs.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/fsyp2yvcq1xd2ag2ww6f25ri5pm8ghh6-lzip-1.26.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/fw6qa513xbkv6cmywc5bmnl42vn3is7a-make-4.4.1.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/fx1ariccq0k411lvhjl3iwbql4qq6xiv-mescc-tools-extra-replace-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/fxgid84c7kz0jkac0gwpahxpwvr5nsap-rmdir.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/fxh7wabjwq8ifyb45vy2f09zawiynyca-ultoa.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/fxhhf9vlwrvj4q9c5smk87kmk4h9k63b-patchelf-0.18.0.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/fxii7wgh8167mckd8bbjal5mxq9l9p05-M2-0.hex2-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/fz6vky5jh09qakfdzx2ygsfasldr6p0d-tinycc-bootstrappable-unstable-2024-07-07-source-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/fz7m1s1j4v0gkiggw7s80c9yq0rqgj10-libc.a-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/g096i4ibwxnri1f7zmij06pjv7i4k651-assert_msg.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/g34h3g6sjisk7fqp6lc0f5mcls3x2rd6-lseek.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/g39zbiv8d8qbgzkqk2s03zdfc39b1j4a-which-2.23.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/g3m8vb911l1ij9ysibrqznbcf6y4vnzd-qsort-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/g5ifkaw6hws40d16zkprrgb1ddlb9wib-tinycc-boot0-unstable-2024-07-07-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/g5ms3szgrrh6p205vmz90qj0k181b54h-strtoull.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/g9cwbx1jfj0vfxla7ifxcrk2xjnw1bsz-readdir-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ga1b8b5z58fbkr8v0xv59ivzir4wyjqv-tinycc-boot-mes-get-version-unstable-2024-07-07.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/gb36vm7d2qjqdsc89bdcmyn7qchy7xcd-access.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/gfbn7q8wqsq5yzv3fi7sr0pf0kxj376m-bootstrap-stage2-stdenv-linux.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/gfjavirn9zvghj3r86wbbpkhw9xpipzi-builtins.mk.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/gh33z218ga15qwxb7vr31yfi774rbbly-fopen.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ghj56d7dhjni37apnjxrl01wiba0zghg-bash-5.3p9.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/gi3ss7gv2i9hx4x78yyaw0qnq8yg3lsm-vsnprintf.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/gi4k3pj06rc4b78l2ld3czi2qgd6zbby-sigemptyset.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/gi8pcvkd1knjmh59d57ka1yqxx4r1b4r-link-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/giffxpxvvdncny4f42vbv6ndmv6z7q7d-fprintf.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/gnm4qxbivv7y2n112w898rdabc0q6pxa-fflush.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/gp0g0x7dg0gvjh98xfkyajni6qa5v1n2-cc.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/gq86ghc61gmkl70f305w4cppzv0zjnjf-libc.a.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/gq999sv6ad4gpmqly6rm9jpq0g49pzx7-libtcc1.a-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/gqc0mq7fw0ckyzy3cp8qfblp4qldnl3c-lstat.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/gr7p5aghmkv93kv5vmq42nn0b3372k3c-m4-1.4.21.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/gs9mwc2r1p7hs6xrnqz5s36dsw3c9nqm-mescc-tools-extra-chmod-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/gvq841jfhz61gjq9g1mdy54973sqdf9i-update-autotools-gnu-config-scripts-hook.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/gy95ardpys0lm0djcwpq4mym7iid5636-brk.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/gz9irkx06fl1lavrwxkghxnmlm4h5fgh-bash53-008.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/gzp5g55jas54wn71b4ympihys0xfppr4-perl-5.42.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/h3fr2g42a1jn2j3wwlkf8jawdadj3nmh-atoi.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/h4fhkingkx88wnxhq1yw36xqdqim2fd8-buffered-read-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/h51d0h0pwlgli2m8rx4r5a97pihqvsm2-access-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/h545qb92mx5i9q4amb44d2phh9by0zrq-gcc-15.2.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/h5c3h1i4pn51c6d9jnm0hc0fb30apx1n-gcc-simple-program-15.2.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/h5z9yb9fcb61sij3pzybq04rpyzgx391-fork.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/h699vzb1lagddixp8liilf9kihibax6x-tinycc-ea3900f6d5e71776c5cfabcabee317652e3a19ee.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/h6pi2cyw7j5liy0fbzq8i1ippcqwdjd3-mes-src-0.27.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/h7ijrlrjxvy87frncwpqivqxn9zaqx5x-patch-2.8.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/haldh962ah38jwv0i8f72nr68nn7vw9r-Python-3.14.4.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/hf1vsgpn7231y185ckq6z2lc6v34b3kq-eputc-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/hg84x27sr3pglh61b22jf93x34q1694p-close-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/hggb46w9b2gp3cqs7z5z10379adalrga-tinycc-boot2-libs-unstable-2024-07-07-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/hh6gy6f43hwplqijcn0z6cgswh975p3y-autoreconf-hook.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/hhyhhkpw6v15rzpbyw2y8qzxrz3wv2gl-bootstrap-stage0-glibc-minimal-bootstrap.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/hq2cbwrkyyck3hl3vcddi1256qk4kd9y-M1-macro-1.hex2-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/hr30bp0br6nigp0zv5jmf9l4g3xsms2q-diffutils-3.12.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/hr6aiifp9n6gl6l8pc3ig55lzw0csw31-bootstrap-stage4-gcc-wrapper-15.2.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/hsw0bb8jg0nzvdf6bq2223ik4vcmic15-tinycc-boot0-libs-unstable-2024-07-07.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/hxagi71byf79kd6630nkzzslsmmjir34-gnutar-1.35.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/hxbzg49czsbmcxx8wlxa8gqaafznffd1-libxcrypt-4.5.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/hxsqmjd5sp9ddjjyhr6gmrhf58gpad1m-fdputc-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/hy5d29af1p9k653rxb16254s0xnmssn3-putenv.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/hyjhwkahcbyhh6yirys29d9v0530x4cv-crt.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/hymk33wkc3qs89h56hxmcs54n9nxafpa-execve.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/hypg46mi1fwnff9f7n14ialasjy7b550-strstr.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/hz2a2dlrpqz662khc2f2nqcfx03wgn3r-nyacc-1.09.1-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/hz4s5xc0rxrjcf6c51nbjcmbsl9zy39j-libgetopt.a-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/hzisswpz8nrmfgc5hgcvv9qlijlpazcx-update-autotools-gnu-config-scripts-hook.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/i0bhjf8ai1bl51bvpyc112bjmk46hby3-vsprintf-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/i16zp1zmdi4m4wcqq9y3prcbgfcr3hwm-libc.a-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/i41fymycfib1hv8qdjk3nsk0h7g578dw-strchr.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/i5c7m3r8bbas0w8zl539ng8zzr53mxr8-libtcc1.a-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/i71nsvc83jpla9pszrll657an2nsdsil-libtcc1.a-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/i8ss46xy764d9d593vn814niksfcriv6-tinycc-boot-mes-get-version-unstable-2024-07-07-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/i9jy5m8nyl16d2djd48p0d81xs4na9pp-fcntl-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ia4sjsi3cl16751r14yfv92603830nnn-gmp-4.3.2.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/icd8xv4gd1q59850bdnps90lnl1jq351-abtod-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/idh7xqk7a67n07f69jpsls2ncfz1ghnw-unlink-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ig80p3grw6fhna4qdz9nsxsb3z7q0h6z-strcmp.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ig962i9my0fqk3cw7wzzrx3kmdq72571-strrchr-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/igp30wh0ssfl580sfh321gwhfsjfnsjn-__raise.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ihmylc6qh8a0qlq60p7gj92c59cz7z49-gnused-static-4.9.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ii3jsqp9rrwv0g21lwbvb94ddg8c5m7n-bash53-002.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/iikvpxx1q5fzpz07klvi4r458y6nq3ks-krb5-1.22.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/iixshw8nhrf8cb503b57s9p94cl35k8n-nuke-refs.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ik8l0khmd493q0mxq1c1gscmir3c8rql-ferror-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ikynbpdjj2lr2ivhzlmzv9gw05fa7ibn-tinycc-bootstrappable-unstable-2024-07-07.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/imyy7rg9gyf2qyhg3jjnmal8grabvra9-glibc-2.42.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/in9n30q90i5kwqf9rahhp4vnxz5y117a-raise-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/iqppcir7yrxh3ajirzqd5krv64dd9vix-isdigit.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/irpf44fixgjjcisk8ifmwqwbm49mlrlx-mes-m2-libs-0.27.1-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/isxmry87mdhxx1g979v9wcr4m3wg4gpq-diffutils-get-version-3.8.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/iv3hf1ml1fdmiwipnbz2g6ijaxy0kss8-sed-4.9.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ix2s33fi4nfh78rs59mdsdnq5dd5gs7g-tinycc-boot3-chain-unstable-2024-07-07.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ixwy12didmgk7j4z7ghlirpw46ahmhbd-mpfr-4.2.2.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/izbfj3rckm6lyip0y99dw9hyyap52kg6-libtool-2.5.4.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/izqfaybr44m2b7vrs3j466z6g4cn32w2-libgetopt.a-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/j0l7801fhy6ralfrcw6b407bksgf3hp5-fread-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/j1aiw0pfh51nyay8zavyx3adq86xdnkl-tinycc-musl-unstable-2025-12-03-compiler.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/j2cw7yamyxg8lpsgb2yv46v0sgf8vlcr-builtins.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/j2h98vg3kzqg6bif79mfkzb3sv50id8p-version-check-hook.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/j4wjiwwh98lysm3nj7gidv06ch0zns2p-hex2_linker-2.M1-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/j65zznfqb602v6ar0k7a7bb061iwshr1-strncmp.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/j76if9fyh6aw41qvc8nqr9c8ha4azayn-hex0-seed.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/j7jfgy99zxzpj6xd682dg3s75l1scfwm-string-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/j7p42afk0zv32v4s14mpsmgmlnwlhgvw-isdigit-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/j8j2dnbhmvnry5ijz8qhl5n4ghjn150f-fsync-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/j96xnv3mwd9lrpw3p12c9dihl1nq7bi8-utoa.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/j9fv75h81b9rz9kjfcgxpc9vaxz0jywk-ed-1.22.5.tar.lz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/jbiiiby5kxq7vwa94liwyw56h23liih7-tinycc-boot-mes-unstable-2024-07-07.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/jfdsjqzp6ba2sxydfwvclbash9z0f3zf-gettext-1.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/jfgdmzd75pybl8jbyv0qharzncawvfjk-hex2-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/jfvmc8risbjpwar6fg5if3yy3zw7hzmc-fdgetc-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/jfyw0fak0943dzrsz6sqc1vw92x9p4wx-tinycc-boot0-libs-unstable-2024-07-07-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/jhbdwrvavmabzvnkmmfahrway87dqqqd-update-autotools-gnu-config-scripts-hook.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ji275smnd10rlnv1g0wik0jhwi96swlf-gnutar-static-1.35.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/jk8228a88vjsa0zk004sgdhxp5g4pnnv-hello-2.12.3.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/jm8c5a9ysfm3fckryllnl6pd3nm1hd17-bootstrap-coreutils-musl-9.10.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/jqvfw029liggk29pxy4rczkk6744x5db-minimal-bootstrap-test.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/jr8wxdxn0fznnd4nrximxkwbfix0cp38-vsprintf.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/jrvxjq9786fgbm7za49l915jfv1sm879-pkgs-config-setup-hook.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/jsdxi07zgfm66r2338nhsi3q5q6xzv8j-vprintf.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/jsplhakxa0pffas25325jalphic54571-libxcrypt-4.5.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/jsv60xbic776ir115ikf27qm7y169371-mpfr-4.2.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/jszdcrdanr4r34r4g5gv6dz15n7fzp79-locales-setup-hook.sh.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/jvn6rxfx2j66k0svyl5njyqzq2nh0v5s-gnutar-musl-1.12.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/jvxpzk8cr90bn2a19fdi3qbflvqf3cbs-tinycc-boot2-chain-unstable-2024-07-07.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/jyh0vzwz039ffsgmysy34jsbp0mq3w3v-remove.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/k1cn30bfjwfgsfvpm1yp1nms1si2wf2x-nanosleep.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/k1vkfqjn6h31hr48gdd2ki5yamwlzicr-fseek-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/k3bc319glc91sa222f5g8cdp43brmvyx-strtoul.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/k3f1j20sax11l61frp7kfn3sns54qv1x-putc.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/k5rg2vwnp79zgc3f0ybnr7hhvg1r37k2-gnum4-get-version-1.4.21.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/k6s1z4harj1xjv07yhx7j6ypqdmrmm60-islower.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/k8jw844syw3gyrpj9vkyfbgv1bh0jx6n-stack.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/k9kvp567r0m4ixkdanr0sxg8fyw1bhm9-bison-3.8.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/k9map8ma3f064ibw34sr5mrnd818fk35-globals.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/kbirnxddc12bv3px7x7mh6vha8rkrcdx-getchar.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/kcd8hlvcamwjs3p36is87ncigh6i1zbf-tinycc-musl-unstable-2025-12-03-compiler.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/kclz14inz8rmx1jg77kb5nl3hqgqjnc7-clock_gettime-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/kkr0ncy5fyd3rv59wdb679q0fhah7fn6-M0.hex2-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/kln9pm3dds6m4gz0npsp84gb1qqm3sqv-lib.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/kmm4m261ix0b3b74380z1z44d737jac3-fputs-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/knxryyvdxg6ppqb06f25p7xfg2lw2a7v-ftell-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/kp6wvw7jdv9ssi4zafs58y3vn5zk6ldx-strrchr.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/kpg9rvnnj20hmjlhagakbh370b0wgmj8-memcmp.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/kq40l5zlizl2nz5w2qz07vdsi79axxfy-heirloom-get-version-070715.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/kq769aa9ay0qz50alprfqlj2bmd0b7ir-libtcc1.a.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/krnzk3z85c0shx3jwz68v0pq2yjv9k2k-search-path.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ks1l1rm4andjkd4gxr1qj5iqwflgmcsi-tinycc-boot0-unstable-2024-07-07.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/kv577c7qpn57nn2mv7wixhqn7vm1q1lr-bootstrap-stage-xgcc-stdenv-linux.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/kx5chwyxn47w0mg2qbpmxfqcajvdd9fn-dup-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/kxb0bf0fan1sq37khpxqnsz9fmgxfq7m-getdents.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ky8l5xba6d7j3x545js7ihmw1cywbmyi-Python-3.13.13.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/kyq3pkcn2zym902ks396id0nhczxkw9q-snprintf-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/kzg7vv4ws2lyvisn80zlf63gzh5bff8m-bison-get-version-3.8.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/l0580hbqkpfwj5d8ggm480zq8k8sg2nk-localtime-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/l3c18sdal1zr6930p31awpmz1ncs68wy-gcc-core-4.6.4.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/l4q7jsk9n0ys3qb31zafgzwakpwq0360-libssh2-1.11.1.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/l4xhnq4jk2fxh396c3ksnlg7rbljlr5j-libgetopt.a.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/l5s22qij435ycw3f6is0bwyf5m67fs7m-realpath.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/l6a6rj0gy31gp6k7k80qzn4ni0yvyzgh-gnumake-musl-4.4.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/l6i9qfdw2j2c3ggjifrpsyc8lmn0mrr5-crt-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/l9j90v4nlsfd1zgmxhz6y4f4nrjnnlr0-gnupatch-2.5.9-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/lb92pc9xa37l3lqhw7r1rrg3gvfvr87m-libc.a.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/lbn6dc55b8f36v2p2hkqbpk2z63wnjyx-touch-dereference.patch.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ldgmv28v6i8dzq8vs5nf0da0qpzls80v-tinycc-boot1-libs-unstable-2024-07-07-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/lfxjv1zhyc2wl6sz7m55bvl896zgb4f2-symlink.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/li5ls9fwpvck2n5hmbvg949a12kay86w-variable-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/lj67kbyl99ja30x42rfcnbvzqzsxiwrj-libgetopt.a-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/lkkzb8ybhp5jjgvb04b13n68i51lxwd9-keyutils-1.6.3.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/llki4kvcykzkpn69hpkvxk86gpjb8cmn-oputs.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/lmflf3xziz6s4ia23j485l3gvd97vlg7-umask-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/lmj4kyrya73pxb4l75kqvcy9wpw4ilzn-tinycc-boot1-chain-unstable-2024-07-07.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/lmwdsxcxmjgjq861yz7mrqrqq918hqj2-memchr-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/lmx5h1xhlck345nc3as6dglfv57aq9rn-tinycc-musl-unstable-2025-12-03-libs.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/lnfwrgak7pvwc8mjp1c58h4mhb5857pw-tinycc-bootstrappable-unstable-2024-07-07-source.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/lniysvvb2862r072fmdbxahfkihryw0s-strtof.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/lpdjj0xvn86axxiz4jxaq68h5nnfjdgz-gnupatch-static-get-version-2.8.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/lqx91q2khs5xrb42p1lirxvnwdh23sg5-memmem.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ls8gxq4vrviapqfzbnvlzn6md9zgfy5b-fdungetc.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/lw7jnk0vrdy642zk8whqxm98f9273k62-make-binary-wrapper-hook.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/lw7n1gm9yimbrnr00h6l6pb4c9hkb6dc-bash53-009.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/lz4nn74jnxg612cl2jzpd49baqsl19mc-cc-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/m10ysgh47imr6jidkz6ij1h66rdfqjgl-tinycc-boot3-unstable-2024-07-07.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/m1gq6x7srswfl66arij6spxqnrp7sdcl-bash53-006.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/m2khnp0iw665w7zjkyvil62v5wjlbl8i-patch-2.5.9.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/m2q365smdp66pvh626qw4yjr17j90s8b-dup2-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/m4pkby9pv0nwasab4jrr4l2z9qghmqjg-glibc-2.42.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/m6wmvp7bp6c8kn3jlbl4my0g467kdl0l-strlen.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/m74651b793zgyyvlk9gx7v1cl1ywslib-hello-2.12.3.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/m7hf0m4lksycc3hf48zvi74lkna0v0kh-bootstrap-stage1-gcc-wrapper-15.2.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/m7qhh4k03ymh4myrj8nnanzgr2pcx5pp-yacc_remove_wchar.patch.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/m8i4xrdxpk18vi054wvs4j890g8mx728-ln-boot-unstable-2023-05-22-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/m9dxdssk2lq67wymiir61ihz7317y26f-gettext-1.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/m9i79vj3c32hfrxbbginyz4p252jrps3-sigaction-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ma479p2ss57bs6afjmhhi1xg27vin41s-gcc-10.4.0.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/mag3pq29n9f6jw0rb17crny8ilm4kfjy-diffutils-3.12.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/mbfqb9y6ryn2axyvg797s3lw1w49liz5-gzip-1.14.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/mc6k9j8kw27a9av46g6g4d85nscjxfxy-gnumake-4.4.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/mgl0nby6f64kzvgc236801pf7jvyv21x-bash-2.05b-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/mhrxc5w6drwi4m5ykbdrayz7869i9mx9-bzip2-1.0.8.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/mj28d302q2w5hixap96y39dq8b1la411-free-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/mkwbpkxwzssdkc9c1rsx0s37kj951sz4-missing-defines.patch.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/mla70gpi43cs9h1knjk2jvk5j832rin5-libgetopt.a.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/mldlm2l5hgpjlcmvzl45zrl2053pv129-stdenv-linux.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/mls5sqcps3yi6iwbn5bbyr1pymsk8cyy-kaem-unwrapped-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/mlw3q61rvs7x68gxr9pl6m3c6wb2mgfb-ed-1.22.5.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/mnv5bkp8cn6rc8x75gl1wc77mw45i284-which-2.23.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/mpyi8xvv5i4ydkmgv1ly2wcmliz8vq5l-putchar.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/mvwqinqcy1zjvb5zmhji91rjnmzh0fqf-tar-1.12.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/mwr39jx5bysz6lgcj42dp70q1yvjr4dh-fork-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/mxbj66h6gclhhqy318i4v01lx2qiy9z3-__mes_debug-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/mza46h92jsn5430vmsh4sx8zq3cc2v3y-atoi-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/n36amj3qiwfq27zx58bw257yv8vwyj2b-pkgs-config-setup-hook.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/n3sc2081xz33g8q4l4svcw8v6v3qh9r9-eval-apply.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/n446lla0nljzkn8bzwrv98kn2qd27rf6-gnutar-static-get-version-1.35.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/n4ys9wp9c51jzcwbf3vakzzbdlis2p8r-version-check-hook.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/n55h30m46ndihq18phz5n0cr6ii0jzmb-getc-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/n5j4h0w3fspyk6f5q197ii440gsjmk1k-crt-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/n5l3ikj9s3jc243w2rjszsqw930rd1qz-gnu-config-2024-01-01.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/n82czkqpwvkzyjar5xgf5x8s05ih49kv-memset-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/n8v417y9hc2r3zyfnbjf8fg30v6hm529-python3-minimal-3.13.13.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/n9w1dvf9k6617mqdqj9vr74ajzpq2b3a-bash53-004.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/n9y1n1w3cwwlivnqhv98axp155wv1b76-sed-4.2.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/nax1rvp3q4xnj7l9z726dm9nxablqlgl-tolower.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/nbrgq9hv4agp3mz77z3mysr1798ik6q9-libc.a-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/nbsdqpfzh1jlpmh95s69b3iivfcvv3lh-config.sub-948ae97.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ncn47n6zrpsy3kxc146392s75fk4lk2c-M1-macro-0-0.hex2-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ndg73iq0ha0rk67lfcr5i0ni1qx93wi0-libc.a-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/nf9d42gssxwv5i3ln0wclcyhnjszfqk4-zlib-1.3.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/nfg4vm8619w5b6fpal9swrkx75nsp3bx-crt.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ngq1yidsz2dihpmkfl4shq98ky4rgpq9-utoa-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ni3mx708vbjmn68ziv908hia51ah5am6-malloc.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/nix4lx9q3da89ip0fnd3rjpi1kjacif3-mes-libc-mini-0.27.1-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/nnkx70rnb45cksjrsh24ral5jd6al5h2-blood-elf-0.hex2-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/nq6l8wrdqwnhmzs5q71468jrgbwy1bdp-coreutils-9.11.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/nqn9nizwqy5201zcv0jn5l7pnjm8i637-kaem-footer.M1-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/nqvwimqscq7jb7wgbdikwj9f6i6w4ffj-libgetopt.a-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/nr843fdsa5jd0ib9dhyg68xsxfka8zyk-printf.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ns18qc97ciac3jfhnqzm1xmn4nnll4nr-strtold-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/nv9vbwdbppdb9zvv78m9a99vga207ykq-findutils-4.10.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/nwlwl6mkyy3a4s9hisffr3vajhk50c5p-posix.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/nx5vjcba7q56d0xpr7dj0vy0nln5i64k-bash-5.3p9.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/p0h6a81w4c00c94ac7wmzlbldh8v635c-expand-response-params.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/p1525f5i8db7wrjjq08xh0m8hyjc1wk6-uname.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/p1jb1am4inf7c82i05kybj8iqmiiyldh-libtcc1.a-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/p30fckwbymg431mqzcfixvd08p6v5h4k-chdir.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/p385mc566pxgq1xsw5lka31mmnm39i0i-libtcc1.a.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/p3a7v0sy252p81wn1wrz56nn6gqiwv3n-python-get-version-3.14.4.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/p3x8835w2rwiyy79ydrjmnz7kq5nsdia-tinycc-musl-unstable-2025-12-03.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/p5qdp6gr977n1s1518lpkn3zk2m70c8k-ltoab.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/p82iqqa3j8pphqij384b968pw78z8i84-keyutils-1.6.3.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/p93ssgf0b6f5bnzlsmb8zqfizhh9jfmr-xz-5.8.3.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/p9x2bbq9n0mqccsg834h99sadrgmsj7c-stat-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/pb2vzi7llzk14zmv1iv95qdmbf0lyjnn-libunistring-1.4.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/pbrsvvhf1i7d9w3h87gk60q3bsngp69z-wait-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/pfnj6kx8hcpl8srpk4grlj0c09sdnfaq-mprotect-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/pgnc438nsd0y87xi4fjwzyhc45i4n8v2-gnupatch-static-2.8.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ph9222zgigjrdbz0hlml3i5kv86hkqwf-patch-2.8.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/pj475p4a7p2hzb49xslzyrbzmnwvjp4c-cc_arch-0.hex2-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/pkdlmcijz51kcm5iinph1269fy14x7ym-gnugrep-static-get-version-3.12.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/plhffvbfj7978vpqgda595ypr1czg3s2-make-shell-wrapper-hook.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/pn63pwy6gv617hpy7r33m6l4zifafzfj-calloc.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/pp31fvcgghilkm5xbgbblgd24vv7rl7v-strstr-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ppw8q27avhjirlrdcharilyf3lfih28f-hex2_linker-0-footer.M1-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/pshx4r5vypzz5cjy5ikscgjyzhv2s99x-gnutar-1.12.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/psn06pbp7vp4gmhdbh53jq90wkx77k4m-findutils-get-version-4.10.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/pvhb5byc5ya3z3692i00lcsr7rc6d6yj-ultoa-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/pxxi73ah5w9h8987s2fdwspybf6jabvh-grep-2.4.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/pxzj1p0myjc6zd2q95i8f82y8vg4ymyi-__init_io.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/pyy0lwwdfhrxws0g4w4kq88x4qikyvc1-puts.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/pz3m6g7iabylyazm40p363y8795vybf9-mescc-tools-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/q00djxdnx6fivlqp60l7l37yc42i1fhg-gnutar-get-version-1.35.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/q0pxfghmib8rkqh9dfbc233llkihs3h5-isspace.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/q0xr619c14v9cl1vhyq83407r23npx4j-_read.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/q1rm5rl861j6yy2mi0avl6n6p2s32pf7-xz-5.8.3.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/q335r4z6sia41v85hpj7a2mzkbvr2dgi-putchar-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/q3px40945ahyqcfk1jd5i0ysarphz9dv-mpfr-4.2.2.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/q75f3s0l812jqmqks1x90f35srn6ac2m-strcat-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/q8c5l3jqyqw4g02k0anhhxh1g4307ni5-mes-src-0.27.1-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/q9nj3bw34c6ag70kcss8zlbanrqs5maq-ioctl3.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ql0nssy1bhgyf83zq0r4lrksd6zsgh7z-gzip-get-version-1.2.4.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ql7jhf1wzhazxw4cw627zyls49xm5g3n-sed-4.9.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/qp9mn19nkyp680m55w409i550zni62gh-modechange.patch.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/qpzlh718p7p4nxhhwmcmjirqi0818rh6-mes_open.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/qsyqpi5hy1i3rzmg1x5l2nm0d3hcdmdc-hex2-1-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/qsz7cv59yzz5a9yqxbaridbr2nax5ms2-sigemptyset-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/qw9jid44nkp18dpan0wsymqqgmxabrwp-pkg-config-role-hook.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/qwsbaxq6i3scbf6zqliy6jq832jabfi1-_open3.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/qxz8v1gxrrw9j138k2ajl36vcajra8mb-isl-0.20.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/r1rf2p01za5ckcbz87fjvdnz2mpvy3bz-gmp-6.2.1.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/r1yi2iw61sqxbjs84649li9jwqcgbg77-isl-0.20.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/r2qcv2rkxs62fjy3n514p78mhqlyxflh-tinycc-mes-boot-libs-unstable-2025-12-03.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/r5yk7nin7qyfhnvd04pcvzmcwqadgrbs-mpc-1.4.0.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/r6l6qkdqrxlx9zn9mmiq6pxb7ym90l2y-gcc-15.2.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/r70i48xmaw176ydwjhwdya8kmh1b34xy-abtol-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/r78s609apr1sim2gafawm5az6va9gkn0-__buffered_read.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/r7yy2b087clhd9zrdksjnsdygxg0dnh2-_exit-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ra09d7l6r1ld4jk1xbl9rmii31g753bb-gnugrep-static-3.12.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ra17f5z1cxikjsywysyd63qcp5nlxa7a-libc.a-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/rc6rff9rwixd0ah70cynhkw6fl79whwq-libc.a.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/rd7pipwli0pfwc4qdygaxh8prq71izjh-bootstrap-stage1-stdenv-linux.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/rdi2zjmbw7565gc4pa4fkzvzpgwpaaj3-bash-5.3p9.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/rdw25dyfrm5nxp6kanr64z1v9lh3kx6k-reader.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/rhmr6f9rbhfl1fjblqcdmr1jannl6qhs-itoa.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/rivkjf3vnb267xl9xpjwj8wyzis4y0hs-bash-2.05b.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/rpixjnc530mwhkqspphhicl4svcq10w4-rename.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/rq77y1w9cbyx76dqcnw4wa24w1wq3ybw-M2-0.c-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/rqhmv2i8vxlz871j0y1aly194zfhfha9-acl-2.3.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/rr0issf5mmnpsfmqv861x5qixs79rwpg-stack-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/rrqrgx623i8lkf40gaw366d07m7vl3lw-tinycc-mes-unstable-2025-12-03-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/rsn67qwb0kfb9x4p051xy558gf407zgk-pkg-config-wrapper-0.29.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/rvcb8pzv5f8ax35ggc1hdk2vh5n6fq25-gzip-1.2.4.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/rwlh0lyww7if8gd94fzbnwh3917m9p5m-realloc.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/rxpmjxrz1xnr2pkamsa59afl929cj57k-hex2-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/s012w05rhh5sbhykm5b956n86clnpxk6-mes_open-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/s0xhkwp277nc4pb46k2s70npdnyc54s7-globals-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/s2g0la1392ys0bp34d0m38kmlzwi30v1-binutils-2.46.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/s3jjrqw6ay6nllpng4f1m1cffa11bhsb-tar-1.12.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/s47mlxqkjc26fvmx2yrps4hkpigimxi1-cc_arch-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/sa2if2mwj7kpyqd1kgjnx00ny3l2xivj-libc.a.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/salhf0y3pwji5rc43kgh51czb4ck6qgx-hex2_linker-0-0.hex2-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/sanzvc0fmx3nq26axi9925wk6rb5zmk1-mpfr-4.2.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/sb1sw1wqkaswx04a9pds57ms69gfi2ya-gnugrep-2.4.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/sb4ni1gpi595rakb4qlrbfzz6znk8p71-isnumber.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/scff6n77m9an0n7wzdm221b5diz0k4rj-kaem.hex2-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/sf7cx6m8hjb2f8m0fhq0d2lwndq61xbg-bash53-005.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/sflg6mqskxkfxbkq67nkbm38qg72n4lm-setup-hook.sh.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/sfxibi1q7pz850ah56lk91lr2rc6qciv-dtoab-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/siqp0bc5va5ng2k6kvvxhbxd87jlbcx5-waitpid.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/skfpkjqyfc0gn4skvrf2d3frr94i9idn-xz-5.8.3.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/sl1ai37193yxd9cddpz7w6a55h0kyinq-strtol-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/snijyhj9mfzbgmncy8ib3xy10zq22nmx-mkdir.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/sp7cjk6p7a8r8cvcczx78v63q5583b32-bzip2-get-version-1.0.8.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/sqpd1xdcz0wqscmky1yj2gqsl95mhd9l-gnum4-1.4.21.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/srsdkcf2f392mxfk2c9w5a0q2dz26jja-isatty-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/sv1z4x04x9khz94vfm1n7j9djgy60ghq-bash-get-version-5.3.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/sv664h9bg5xkpnqa3ia7mw7fsdwp8isa-gcc-simple-program-15.2.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/svc88737cf1py5bv6qnxpcp66jc1ngwz-symbol.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/sw7gwv37bczvhkynh6vcbi9q6i84y3a4-libtcc1.a.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/v0wvdpyrdxf6y8qb2flcq775zwc71hlh-syscall-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/v0y90g5sah7n98g296v65xlnwb00vbsq-mes-libmescc-0.27.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/v1v5w7szkp8l1yr379x9r4vyw7fkmmkl-tinycc-boot1-unstable-2024-07-07-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/v2pk6avh3x1k3k29hkja9shvw97954n1-tinycc-mes-boot-chain-unstable-2025-12-03-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/v375qxd9ls8bc1ah525l7yasy60xjijs-xz-5.8.3.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/v4wzp9vgcg10cypm21sxmmcf111jpba4-bootstrap-stage4-stdenv-linux.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/v520nngwnn1hyzyx6pjsd77lyas14v36-gnum4-1.4.21.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/v7yw14dv8az2q00ii6xpqysajbbagva2-write-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/v8jl1cziqai2qqgn0axr9cv3masm5nb5-bash-5.3.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/va81nfrd7p4n32xi0y3q9x8clsclsix8-gcc-10.4.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/vb8v1pvxr5piwksxsffwmdd5zz6s2j39-fcntl.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/vcjywcg8j3zy9i3may1p25scpwjsa441-exit.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/vcp6y915w85d7sbb4d27qfq5w5r8za5v-qsort.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/vd81g2231n9h6pm282j0fij7dj88pmh0-write.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/vda3dxp2568dwv487kb113qqmylby9g7-eval-apply-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/vdwm176pyf9v2p3shy9ck53084ymcjx3-libc.a-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/vf5nz05lk9n6xm1h1sh3c8s951ybk08h-musl-1.2.6.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/vgjfnqbxgxa8a5575bhq07nm35b2l31m-bison-3.8.2.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/vhaqzkl380y6x9vlfmy7kd39f0fypdmi-abtod.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/vjacq537970f9g0wh6bm999dwbpixnrc-gettimeofday.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/vlf0sxa4sgdzfm30fzszg9wvjnkg3q3p-vfprintf.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/vp409v6lb83p243hmv8wgdmiaj7ivq6h-gcc-15.2.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/vq5bcm0daiam2xn12mq03afqlnzj4qln-python-setup-hook.sh.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/vr3f3vnds0i0dpkkj69x510j7nrk65id-strtoll.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/vr93scipsw4jv37apjfhxy5nspn09y8i-binutils-with-gold-2.46.tar.bz2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/vrkgrz8l2z0yq45fai507hpcaqp0i9cq-touch-getdate.patch.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/vsbymyzgxr2pbsbdk5zfn74nb73njni5-grep-3.12.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/vsvi58gj6rmkdwyxb8iw7glcc6vh0dfc-link.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/vv6kfrbh9q0s1qj9f8xprx52d01xw5hj-crt.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/vvhsac436ggb1pg732mqkhw7s9jppl13-symbol-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/vvv7kxv3b7mairchnx5vniz4vcd1dksp-crt.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/vx2wmbx2yqrkdz267650gpnn4bndjq5g-execv.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/vx4j21115j102bdd8wf5yf0x6qbpzwar-gzip-static-1.14.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/vxj9mw4549sighgb4hv2k718kl2p70sk-buffered-read.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/vyw7h2czv5x2dn19g4gxd6k7djqq940a-mes-0.27.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/vzpav2rbdhq77ig3l6n2y0bnnpy7pazp-binutils-wrapper-2.46.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/w22wfi526ln8j9jfm87lamjjsdrqcdwa-isupper.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/w30lzfrj78qpp2lnmp6c3r5bqwjna34k-strupr.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/w324cws23p14nxdwqbmafpkykcsqg8r7-_open3-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/w4dgzqw1gnzy5irzw00hn25jbsqhqywf-gawk-static-5.3.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/w4vpchc8ps72xfmi6gbbrs6iw7y9kqjn-cast-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/w5008ccs8cadyjvgpx3bzk9d387ml2dw-tinycc-unstable-2025-12-03-source.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/w7skxq7bn25rvbxazn7prxvc8c5jjpjr-syscall-internal-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/w9fk7j4q1g8wls0iwj94iqllxrisnh0z-glibc-simple-program-2.42.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/w9ncwnrxfgg0mhq5bnv9i25ahx7zfm7a-eputs.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/w9q6kw3yb7az7wwzsvm97hqw7c8nrv0g-crt.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/wa6xxcihyrcglxac311wc59lxsfssh9x-common.mk.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/waf4kczw67cway08qchwrcz38fih4y63-tinycc-mes-boot-chain-unstable-2025-12-03.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/wbaljcz177vkkjn844nya51h6jrl49hi-gcc-15.2.0.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/wbq1wksxsc1gsgdkcan2h40l4x4gxx6w-lstat-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/wfpz7dbg98phhbiwfqc61fza4vidwcjy-gcc-cxx-4.6.4.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/whkczx14wnzmkchfys92zgas16rnzqw9-make-4.4.1.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/whm5pdc61pzf843jhlvy2r44x993xldj-libgetopt.a-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/winwi7l97nmy056ljjgi0pamca39d14k-findutils-4.10.0.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/wjfp3sba56m3mgbpssdwpg8gc4558q33-sscanf.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/wjmzjx1jzp5h8pmrcjvbs3hc8d5g8y4v-musl-1.2.6.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/wjqrmkhykih7qy97sgd5x3ba7d42wkx0-fwrite-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/wk1dy4n41iad1gk1zzfjsq2in2jw441j-variable.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/wk8l4607gjnfyv4v2bnh69gfcqisqlx1-fdgetc.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/wl53f1adanfm6qfdkh77xqsj5c8dkwsf-fdopen-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/wlp9qi7x32sldkpfrmjqd831hwwmqvc2-mkdir-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/wn4nf9pqapc89jz1lkvn7z3g6b9rnclv-gawk-mes-get-version-3.0.6.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/wsd61ackh8aswrw4slh85j1fxvqsfxch-binutils-2.46.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/wv752wrcldm19d604ls1xz1ibja7qacl-pipe.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/wvfnp9binadwclp8lyczahspfddz22q7-fflush-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/wvk16ysmscid85nmvb54ia8wzcws6b2z-opendir.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/wxbms5xmv39yyyqy337w0xylps100055-nyacc-1.09.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/wy41a6rmv90s4ryk8ri9b88y1rid9aan-strcmp-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/wyzbc8bi1ff5fyasvmq7w09yjlkd86b6-tinycc-boot1-unstable-2024-07-07.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/wza14brqwg5p63dvzlffcvqrgqd45f7f-gnumake-static-get-version-4.4.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/wzbw9g2rzl3a9rlw6gm3yvy1gshawg79-brk-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/wzc5vlgxwlx8xjs4v6jgw8xan080jxks-M1-0-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/wzhb336rh5z7c4fmp4fgf0hl6akn539r-xgcc-15.2.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/wzkj5hhzwm6ppkw8nyan97h6aw358av2-memcmp-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/x2dafg049vyrb1sm6jkydck0b1da2zab-coreutils-9.10.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/x44jj7p878adq57sn3w36gklljk4vlfl-expand-response-params.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/x5wwz0vih6vxdwpvyq5dc0w2ih64rklg-gnutar-get-version-1.12.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/x7l0vwljyrwyncsxkyvzh6gmc1f8p2rw-main.mk.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/x7s8n5cd93py6hxzx3hgga322ffcvd25-libmpc-1.4.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/x9k1p6h2g1lmlz11cpcjg8m2i4m6xr98-bash-static-get-version-5.3.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/x9sj5ai5qx2c6wmy5qz6dpl90g6rbrxg-blood-elf-0-0.hex2-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/xapdzxdhzxcbid65c0maaxvdygqg8y9w-python3-minimal-3.13.13.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/xawlq0m3mqzjw2n8zvxsrqnwx51cmnpl-pipe-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/xd10s7g0zy3m2kycbx6pnlmkx8wbd16g-memset.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/xfz5c7irzix5sakph3m18v0h400s6id6-uniq-fopen.patch.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/xi8dy016hq7842iplkgfdmzribiy8a3m-tinycc-bootstrappable-libs-unstable-2024-07-07.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/xjhir7vz7kb75jz8g66qa4b4ks4m2k9p-ltoa-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/xjvd53cvzbkfy823qvzl7nq1bh2w47zr-bootstrap-stage3-gcc-wrapper-15.2.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/xk5gc7f6j4h9hcx2z3in2cz12knwmj1a-oputc-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/xlhf3ibq6r6qy2myin330ay7qscfzi1g-gnused-4.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/xmifliaki5fkzvl1pw3qr6iv55krvpca-libc.a.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/xp4239xk0iiba8z35i58i9jk4wljs517-closedir.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/xsn3z4sqkn6ac7mlq35b62qs31mlnhbw-blood-elf-0-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/xwkyb3k04y5vx2kpb12gr7plqvcy67c3-update-autotools-gnu-config-scripts-hook.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/xxxl49qjg7gnwxfcr4sxfgpw4shq510g-tinycc-boot0-chain-unstable-2024-07-07-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/y3l2fjfs2fjy1ljsgq8b7f36sxn2777a-mes-libmescc-0.27.1-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/y3p27ax0p5slfgw496qc068jj7dbrn9g-musl-1.2.6.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/y4ni69pgbj6r3yvgzkqb5aly6i0vn8ym-vfprintf-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/y7wnv3kycy9gpxzwmny1sppi7il9i31a-gcc-wrapper-15.2.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/y84saga4qsmk7xrprd0qv8rgdqjk67g1-bash-5.3.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/y8xglficy4lqy8g4k0vfb0lbr4zzgk5x-_write.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/y98z4i8g33aic7nq3pi943abwfc16994-binutils-static-get-version-2.46.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/yakk49j0k9yrnfskv63r6iz8qb0qymh2-python-3.14.4.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/yawmf6sx8qa1nylbxxz01723ddsy5h41-fputs.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/yfyk44frrhkfjh07xsgiiqvvaahdsk16-tinycc-boot2-chain-unstable-2024-07-07-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/yglzrih7p6n7myi80gidsxv1v8f4v1ka-diffutils-3.8.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/yh462hds2b7nzdqk3rrbivzqsjjq7zr5-crt-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/yhws1yr6pg2x51q0fabmzr99vl7dj5gp-libtcc1.a-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/yiv6sqyjk4mgai8fsbq0imh3vzca8ag0-musl-1.2.6.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/yjp7zwabw9xcm6g42736mbhlvcddw410-mescc-tools-extra-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/yk049frwz19wiswc9j1ssp5bbvarpazy-waitpid-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/yl1igz5ag7fwrk7cd7w1cdqr9z3z11dq-strtoull-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ymd4zqgir8lilx0q7l1djvgjp6v8cjyp-realpath-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ymxfm7hllmgkxfhpn62szs2gy9k025rf-libtcc1.a-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ynn3b7sb95w08pc639sr5m9n1s6m9lcc-_exit.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/yp4zsyn8fp1v5y8zpvp430lv339lvr7p-gcc-g++-4.6.4.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/yp67000wll97809fd49wjy5hbalpy8lw-tinycc-boot-mes-unstable-2024-07-07-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/yvwzkmzh716qqqj6qx331lmm18mpffva-tinycc-boot3-libs-unstable-2024-07-07-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/yvycxrsn9c2zmbkr9kvixpmazqlxzc7m-libgetopt.a.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ywwjm8pb0vv9dnz5mcqybzmznjz0ix79-ungetc-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/yx1ipzzcjya556sz0z3vc3gvgw86lb5c-cc_arch-1.hex2-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/yynym470zj9cl8d4n6m2zs6glypdrayy-gcc-simple-program-10.4.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/z1a6zj82v13glfvrfnhjkpic03y4kppc-bash-static-5.3.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/z1adhd9pfdiazma0ww8h5vds6j5a23vv-libgetopt.a.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/z4jpll2dilfbxm6ax4ymmhr67j3nbrvv-tinycc-boot-mes-libs-unstable-2024-07-07.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/z4x4wa4ahsc6xn40j847dsrnagxd41w0-gmp-6.3.0.tar.bz2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/z4xincpdf0q9ikjbma0s5sf5jrl3s187-M0-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/z5i7na9qpvfx6wx5phib8gwhqd1kyp99-fdputs-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/z6v5624dkfqg57qjzg6fadpxp07axkhv-libunistring-1.4.2.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/z82bg9br5cmdpwhpzf4qzjdag1calp2b-strtod-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zc4i6kv4525v20s2zh7fhxj1gcbcgggq-sigsetjmp.patch.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zdbga6f8g5jysvhngmnjlwmj674bzds2-gettext-1.0.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zf7smly23scq1jpgwy8lbv6vksd5si6b-M1-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zf8dvv680i9z2wpggik614r20n54hyii-hex0-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zfhd21zxiy48dijslxzcmsj1c2z2hnwv-coreutils-static-9.10.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zg19fm6fzvh5787swpwfrwj7gciidrj6-__assert_fail-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zgklqdh9majhk3nyivpl33ciflvsq5lc-krb5-1.22.1.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zhwaq1pghk6gqbjlcjgw3crf8ggrazay-M1-macro-0-footer.M1-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ziimwk3g7vc4nji5pj5mbivmd0rw6zyh-hash.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zkwvl1yljz9gp7zbppzslzdhyq5zwg19-mes-0.27.1.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zlsfxdxsnzp1nzzw113avl2v0s5mgjpr-kaem-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zm0h1rwqw0jc93rhhsfzqnfw0lk6k3r5-symlink-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zmvpmmh7i1qba20c97v4cl6yi7zqpv3a-gnupatch-2.5.9.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zn15kp0ivyafi24ap2g4745g8cs9faki-dup.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/znjxpfyh2zjd2a56yigva27wqrzk0afs-pkg-config-0.29.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zp92khnxilvlvdha1wan9c8xh3ns7qsc-kaem.M1-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zpph8w4vm2h6nxfm2ghxx9jvjq5dn25i-lseek-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zpwqvr4bvp8hgc8dnix7xyyrb371dxh5-memchr.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zqqcsys6awnmp60kp9q0d7a6r4rzgh30-abtol.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zrr64l06h5bg96dr9s460kkwk3fjk8jv-reader-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zrx02p5fi8vxf7ys4sjyz6ggxz85w0kp-isatty.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zw2ds5k1knfgqzqlrv3wyvnh3za6nmf6-puts-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zwb0f60mz9w9mjc07sqw8jkdx2xx8qri-linux-headers-6.18.7.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zwsm3s6dkvbbj74zg9x2zhigcrczy4js-bison-3.8.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zx5b0pppdvd1sja114g29zh80y1zn9f7-file-5.47.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zxr7znpy378mzc86kb3n15xpk4gw6xck-die-hook.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zy9kmif499i10lh2hqcpjmw0lgjv57wj-cast.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zzgcx00wj6bml7sdl4zzg1yy5zvzddl0-mpc-1.3.1.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zzpghyckmljy4pzbmnj5vnbigmjv4a3d-write.drv"
      }
    ],
    "diagnostics": [],
    "non_claims": [
      "not-build-success",
      "not-package-correctness",
      "not-bootstrap-parity",
      "not-output-trust",
      "not-reproducibility",
      "not-foreign-frontend-availability"
    ]
  },
  "plan": null,
  "non_claims": [
    "not-build-success",
    "not-package-correctness",
    "not-bootstrap-parity",
    "not-output-trust",
    "not-reproducibility",
    "not-foreign-frontend-availability"
  ]
}

```

## Plan with fake PATH (no Nix during consumption)

```text
$ PATH=fake-path mantle --json foreign-import plan ...
{
  "schema": "mantle-foreign-import-cli-v1",
  "command": "plan",
  "verdict": "accepted",
  "accepted": true,
  "diagnostics": [],
  "receipt": {
    "schema": "foreign-derivation-import-receipt-v1",
    "producer_identity": "live-nixpkgs#hello",
    "raw_graph_digest": "cc1bd58f0240218ba01bd31704e7263597fe335917fa3b0f3ad240afd30c62e8",
    "translation_policy_digest": "e7fd62962b19cec881ee06873c2f0abd94d0fda48de7bd4c68dafad0fb3ed3d5",
    "translated_graph_digest": "b22a493270d19f400c88b32b2febdb98b6a87857c273254248c2f04024fed8a3",
    "package_index_digest": "1fb8b0f89dbc07925fc00f7c8a700f846129975496eda619ac8d1b8920add02c",
    "fetch_cache_policy_digest": "1a64e6557d26665f7eef400f3fbfd3ce766a42b72e96b3cdc5570ff3f97b7ee4",
    "sandbox_policy_digest": "b80fdec27a0955fcb9748d1ac75b7da507959fdab796a2f223c6bc8d76424afc",
    "hash_domains": [
      {
        "domain": "mantle-receipt",
        "kind": "raw-graph",
        "algorithm": "blake3",
        "value": "cc1bd58f0240218ba01bd31704e7263597fe335917fa3b0f3ad240afd30c62e8"
      },
      {
        "domain": "mantle-receipt",
        "kind": "translated-graph",
        "algorithm": "blake3",
        "value": "b22a493270d19f400c88b32b2febdb98b6a87857c273254248c2f04024fed8a3"
      },
      {
        "domain": "mantle-receipt",
        "kind": "translation-policy",
        "algorithm": "blake3",
        "value": "e7fd62962b19cec881ee06873c2f0abd94d0fda48de7bd4c68dafad0fb3ed3d5"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/00d529rs5cfj1kwz79sm79qackf9gppk-strncpy.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/013mqc5ymx4cih72blz21l6ync49i3jg-expr-strcmp.patch.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/030bc69ppgsrcvxqxinlwn446dj86j52-fclose.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/037vygj573j4f43vw934yzayjzmxyqjw-mes-libc-mini-0.27.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/03h7kw2rpg8qw1x6bng65imsmilaz624-__getdirentries.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/03qi1b45nyw8bmswdwgkva41bh47z3x4-itoa-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/0442lf73plz43gym6ahgydmp752dxczk-acl-2.3.2.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/04pp7161mbyanbhclf56lwdgsk688ms2-autoconf-2.69.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/04sblc2j6gvwqf9hnyfi2ky7lk5rlx4i-search-path-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/07f6ncaqrcmwjki20pq4y0v1cf31zbz7-rename-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/07lxsila1467h3qxm5pw69qcvbbhcvbr-gettimeofday-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/07rmmdnlwybi90vx19shlmd37azxfgrb-uname-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/08fg4a5d85pnlagm8qrk1pga0zkyjsxm-coreutils-9.11.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/08fyipspfr7hz6mmp6hwh2a7hw35dsga-setup-hook.sh.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/0978f9k46zgxwpc8lwzk9fxajma156kr-tinycc-mes-chain-unstable-2025-12-03-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/09aln9yisls20di7x9wj1kiwhbp2gmm1-oputc.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/0a8p5fjb2va3jw6v84lisqsy0ygmfy8c-bash-5.3p9.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/0agi86w8bshcgjnmblg2dffwdq92shv0-pkg-config-role-hook.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/0ah1g18zgkkg0l9vhh8fz9rl6rawdprj-strlwr-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/0c1rgsk0dm2i9gc3yqqgks66qkidnmva-lzip-1.26.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/0cfizkf89lijcfm96bvkz5g45k07546c-musl-simple-program-1.2.6.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/0d9jqiq9g575wwcr0kv7b3f8l43ik83n-hash-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/0dl092g6chmw86gs84rfgvxj9nw2hnrx-libgetopt.a-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/0f9kajpkkz3mbcaj3ajwha1rw10iwzmi-fseek.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/0fclgm4r5525jlh8lv82r81kba6vh0cc-ln-boot-unstable-2023-05-22.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/0gy12dm9fs950k2pz0jjgwjr2whimg1v-bash-get-version-2.05b.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/0i89ydplmfyhw5rykihdpcr1ndki1bp3-patchelf-0.15.2.tar.bz2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/0ibwnm5f9q4r7j2d6v3239wgk59yn0xx-heirloom-devtools-070527.tar.bz2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/0j62lpv5g1lfxzd1i4iikf3ladai14r9-libtcc1.a.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/0ld69vrkv15sm7kwr2jrviajggb60d7g-update-autotools-gnu-config-scripts-hook.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/0mqi9ydgdc6as1hw0ydm7gg1f3yjgkdp-mes-get-version-0.27.1-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/0n6pqv072zwd9yg93bkjpf19bsf61fs0-coreutils-static-get-version-9.10.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/0nrb3h17g2hhf8ijisi7frcfvqwhya3w-coreutils-9.11.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/0p6bbqs599hggq6xvxn67x91pxszgqil-bison-3.8.2.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/0q1fkyw9jc9fafyswh9gkv48wph52yzy-isl-0.24.tar.bz2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/0rjn4fs706rp2zf5xfyycf1pfqnkjmgg-linux-6.5.6.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/0sm8bvvcpzcq48skj6glwachd0ydycg7-bootstrap-stage2-gcc-wrapper-15.2.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/0wj4x9yyqzmpks4ghvy8csnkga8j7cps-curl-8.20.0.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/0wybcx14pra94k7lxshah1knf12yiz73-mes-libc-0.27.1-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/10hawwrir5ybxn629yca8wjmn3qpn1zx-libgetopt.a-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/12aqc6cgq53348nizvwiybqr1l3h4wa7-stdenv-linux.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/12ia1mifcsy8d1yz20i7hyzrn1jpy8kr-gmp-6.3.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/12j2ylk2hir40iqqbia03rh5330884b1-strlwr.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/12kq01qr9jcw4gpgvzqg962gixgjg4w8-M1-macro-1.M1-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/130xp3xbhxwms383bbf7wpcvc6ksjjnk-findutils-4.10.0.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/15qfmzbh5fx4rbbsida19jyi9sqr8van-die-hook.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/16qh6i0fip1mxnydykdjhjc9a0x1sn4c-perl-5.42.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/18d4rgqv4dfdd3qq09r312hpzjjxwrky-gcc-4.6.4.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/19vlv640sa5fr6cpnr14s166v9vd2p9m-bzip2-1.0.8.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/1accm7zc9acis8qq72sidnkpnd4r49zm-bootstrap-coreutils-musl-get-version-9.10.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/1c2q0l0647l87h5d7ka4d7cgm383wdm7-gnu-config-2024-01-01.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/1cnglsb0a5017gk6f19k0hikb4f7nrpr-isxdigit-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/1f1ni464i1yi79k5ln2cp4wn6idg0a9p-_getcwd-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/1fihipa062xl2drcsm686iscb0zxy70h-curl-8.20.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/1h9r5qrmqbwfnahhlgv67k2b0gb54zd4-file-5.47.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/1hgh3ynq1lsicaxr0r5pdbvkbbhb0vrr-gmp-6.3.0.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/1i51x7j44ck1y9kc8sp5fk7c32sp85cx-strcat.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/1jsd2flxdv4s2lgg32p5l27rp466xap7-fread.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/1l0a69sq1vbdjjcbhnr0yic533yvkpdj-M1-macro-1-footer.M1-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/1l9zwpxb1njah0z7wvrpjih0src22nsi-strtold.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/1mpy7l8c822wij0cygkjydv5lv8lj0m8-crt1-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/1n7hk0448wph3jshbpli7l4di11zjp90-ldexp-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/1naynm0awl2hasqr3fkg3xndjpp23r6b-perl-5.42.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/1qjfvcwnwknh80nz8hdr1hcr7sqs9k59-isnumber-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/1qw6nj87xp2qd26y3qzxhh495rxm7ysd-patchelf-0.15.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/1vgxf29abq71i0mh3m5839my4664jfmh-tinycc-mes-unstable-2025-12-03.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/1vkbzh19qql99n9jv0c0ffqgyzjqlwhy-time-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/1wn9ym4cmvkn6nnbb7p2n1xf8bigi01x-mes.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/1wri1ai0flqq0jghyk9i4fdw568h0xms-tinycc-bootstrappable-unstable-2024-07-07-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/1x6yq984878wwl7xif86kj8z4i5fn6yv-heirloom-devtools-070527.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/200n6nl9ykx5d3lhbjpvxm9f3b81pb23-gnumake-4.4.1-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/20i1igy2hxwh5i5nrakzj41cmaafsm6z-crt-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/25xm28w9xrk9kgn0wmb40g3fc4l2z2w4-bash-get-version-2.05b-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/268p0jcm6cm8y48mzwfm47ylrz61ldg5-vector.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/27x328r5v1pwb73hih8b612c9f55psmg-tinycc-boot-mes-libs-unstable-2024-07-07-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/28incrmkqbjms8pb4i8qkplsisj49075-gawk-get-version-5.3.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/28x04lk2i4zy13bidwmj5ylbx3v880n4-tinycc-musl-unstable-2025-12-03.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/28ynjcjss9m1w5wx6zdbig3k0551i5hx-gmp-6.3.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/2abz4inzlk5dp473kdz0xlk8wkvlkgdm-gawk-5.4.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/2adhxc3jvhi49x2i4076yk9k97z831w1-free.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/2ayyjhs9mz8nb1yfml9lsmfnd1lkw8h4-gcc-15.2.0.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/2b2v62sgah3hyxxbvqgnww2ifckj93ps-libgetopt.a.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/2bj79bj56nqc3l0x4j9578qkm193j3fp-getpid.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/2bkgnravj3d7ybvh9704visgl070gnqd-glibc-iconv-2.42.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/2f88x1jf48lbk7wzjvfvzs5bsh5xbqph-libxcrypt-4.5.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/2h2jx5jrglyqws10sa53n2bvyjqpi9mp-putc-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/2hcix19ddnc0gssj508dfr9yzw29mkvf-linux-headers-6.5.6.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/2jwh5pdbg0mff2bhxvc2vam8bm05wa08-catm-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/2kbbzfsz1hzd21yj0h989s3z40cgrp58-stage0-posix-1.9.1-source.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/2kpjq9x1yy9zmq0cjjwkqw7l8mnjwigb-raise.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/2rmk35vbqb6q9r4aqmsz7iicmi44jx6d-automake-1.18.1.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/2rrbpy9sk06kcqxar40iib4d6vxgsgr0-bootstrap-stage1-stdenv-linux.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/2s8mrpwr478x64dsj2916a64aldibviq-libtool-2.5.4.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/2sakdmbkjgy4zzml0bmmmghlx8pfm4rl-gc.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/2sijrnn5rfzwq8hla21y9agsza9x2lg6-nuke-refs.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/2vknf2sz2fdbpqkpsph03x5qyp7pprzy-dev-tty.patch.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/2xy9v2qajsvlhlq7wdfscvc2658qvkk4-getenv.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/30pxaahlnn3ip5c4wkc52gk6k2b1i4xy-execvp-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/30vp5c32rqb6zhishsl4jz4bphdyg4w9-gnumake-musl-get-version-4.4.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/3298jfza8zv78425rrpr80xxcnnrbgr9-bzip2-1.0.8.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/3299wqz95dmz5wrh0fh2d60x9r31hmbd-execv-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/334rm4dwxknvj9h9mjy319wa1bkz0id8-isupper-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/33fk6f3q18k6ifvg92vgsnmck9dcsqli-hex2_linker-0.hex2-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/377arxf1jfpbv6shjl3iczj9v07v07h1-locale.patch.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/37zapxlhmx12r050xsjw39n4vw3nck3a-mpc-1.0.3.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/387n5gn04pv4cjjlrxxh4plc6c60ifl6-hex2_linker-2.hex2-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/3afkdgyk01g7y052ikq8gbdvdd5i81hz-patchelf-static-0.18.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/3as9iwgnarigh1ffsdb6frbhl586yli3-mprotect.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/3cmpdq0yp00gwfwah7572s0vmi8sa5zb-attr-2.5.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/3ff9aq5jx5hs2fdl2yfqpl1nh7dfkh68-zlib-1.3.2.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/3gqydiy1c03pjnci7iydqs22fzcrm7zj-bootstrap-stage-xgcc-gcc-wrapper-15.2.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/3hdx2i1mbzlrff0d52mkjsh8nbh77cbp-nghttp2-1.69.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/3hlqgwk5mfjp5yb5kw4j1x6g8gjwhh0q-putenv-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/3k613fvf47ib8g1r2whcz93z6wlczmz8-__getdirentries-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/3l89d34bc9vraawapkrxpxfnw6vh3ryq-texinfo-7.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/3lpzywq15v6lc464sls2hpqn7rf54csa-bzip2-static-1.0.8.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/3mqqa9m8hvliz348d6ikb0jg0lj9fsqh-gnused-static-get-version-4.9.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/3qg3iia0yapdaqy7wh3ysih94vi3ng0z-mes-libc.patch.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/3qi2a5212jjx0jn49vwxqlw369zabl3j-vsscanf-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/3rx2w7lxnr0ndimg0mavnb4klbm04amd-glibc-2.42-61.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/3siddy2kzlx0pj7h8s15jgqdicg3j3lr-pcre2-10.46.tar.bz2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/3vbridlri44i5lnczbcxqlwgzbxk3pg5-gawk-mes-3.0.6.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/3xs5n9hmbbmc5qis9zzs764ilbjk0v72-libidn2-2.3.8.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/3yvphn5v3laa2qrvllg878ij47wx92xw-tinycc-boot0-chain-unstable-2024-07-07.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/3zhi7fn92ym1ds0n3yybmjyjvksypi39-xz-5.8.3.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/41kp7bkyk4g0lk2xq45hiqn268nbmx8a-gawk-5.3.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/42v2kyprk2gsp40xspc0ssxfi807qcim-gcc-cxx-simple-program-4.6.4.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/43lgm1xpplnh04c5bszki39wv48ba9mc-module.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/440bjlrs4dxc1jpcx55rdnncm8x0s1gk-pkg-config-0.29.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/453w5gjvblvbv7sflcjxvrwnm76j3kza-mes-0.27.1-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/4683d3nfmlf0ff7gi0ihfdyqsymrn07s-toupper.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/46c2vwwb9lcd8976mmwaqiy2glsh8a6s-M2-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/46pcnsn6dy8jcq0ihi5zn2n8lz6dn777-ltoab-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/46q2kmbs4amzqac5achpxkr681r5r95w-ldexp.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/46s1ihj1l58kgdwy02vwvwam0z5vx1ar-getc.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/4745hdf59rrg9fanw4r357m8maqskhcp-strcpy.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/4a6f891gzagq63mjc60r102pn8pfmrhy-xz-5.8.3.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/4dld0775zm21fbclv6rx3v6c6mfqvsvf-expand-response-params.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/4fi5xvzq940y4fyag9sf3swmd04x23hn-getcwd-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/4hfq6p045k7knv86fc57kq5cnf2djia6-tinycc-musl-unstable-2025-12-03-libs.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/4i7p4sdfbaw3y9lr963ck03y409j3lq9-getdents-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/4lbcc0z7yzs1ml4al2iw0rfnv58gn21s-bzip2-1.0.8.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/4lh6y35gcg0zx5m9a1sx9hsp36xn1br0-strtof-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/4m76wqg80yx0yg0wv4z6kqc1y78yd58p-tar-1.35.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/4p5vrjdw8zl1kc7fl593dm7vrj6n7yn5-nuke-refs.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/4r537r355zybpnj6151m51l3c88gpkpp-_read-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/4s26xanhs14v8nvvm0r525cva3l2wl7s-struct.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/4wl34cclbr0f19xnsfb2jfsr10xz28kj-heirloom-070715.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/4wzln1qrnyz1zr6arar3813z3mn5a7rn-unlink.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/4xxv2r7w1zpqicpdqflas64dwvbanh0k-getcwd.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/4xywz0vr941lrjgspzvg5a8agl3qgxr5-tinycc-boot3-unstable-2024-07-07-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/4y9kzg6h4wxyfpvw3ml99ia9cgn8p6n7-perl-5.42.0.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/4zqkcs8r6kgaap0wcmwpwkqmr6ac5xzd-byacc-20241231.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/52m5687xb0sxg1dmw82mg9mlqc8yyp8d-wait4.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/53b8cx0879bqmw80q6jhl2q8v26lg3dz-fstat.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/54j8qmc5ra0m7dwhqxrjswwmlm5w6sny-isl-0.20.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/54jdq8vp944qn49b18harrbjw34dlxcp-bash-2.05b.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/55d91flpk4776srkw66wdx69wll7b5zr-libxcrypt-4.5.2.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/59pgig863abfc3p6lmz20vh8gjf2nsv1-musl-1.2.6.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/5aa45k5r21ni942hcdjlkgigpyzg7dbg-gnumake-static-4.4.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/5b0v0cj57rfjpqjrc4k38k0si2db4jyi-islower-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/5d3cd4ydvaafcfvq6gfjxpazbik7pfg7-utils.bash.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/5fwbsmcai8jq0ji1m6mh1v7i14rgxspp-tinycc-bootstrappable-libs-unstable-2024-07-07-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/5hpxjqykbkdk302khw74c9vpsyybm71a-open-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/5hyl2z8p5ywhnz1nmlx34hb00rah5x6m-bootstrap-stage4-stdenv-linux.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/5j58nxqnf1npah00yma30515vxl9rdfa-setenv-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/5kp3zisdmyvlrf52qxka2rdhp1vj1izy-tccdefs-unstable-2025-12-03.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/5lviidinj71nws31r1c7gjlg5p80lkjs-gnutar-musl-get-version-1.12.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/5m6vm7vkilsgas517yf8f321k5r3g1m3-bash53-003.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/5ma1s9g8s1r48yaqy114hj8n9bz0nd7b-__mes_debug.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/5pq8iapp8p7zwipgwlfzbq3c5ivjali0-write-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/5qbjq17pking8vfi5n2d1vsm6hknwdgs-tinycc-unstable-2025-12-03-source-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/5qdc1kr8ljllm244bh0jy3bvmz462nv8-gnused-mes-get-version-4.0.9.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/5vfq7hk4md3y9fnqipn07cmiyi13bqlh-memcpy.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/5x0ni3m200vsr0a38nlhp6grwspa3bwc-expand-response-params.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/5y5rhx8xkvnb1rv27ygq7qx2jh721n6c-fstat-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/5zga9hbg59nhww93dfwqdx6xc1q81pif-crt1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/61s6dgr2mn6jxmz42zxmgd6b105gmwak-libssh2-1.11.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/62fjvcsvb78qc4nl1ip1bwkdm2g4x9mj-setenv.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/63ylg04cgw4dqrlxy6zxx58l0mvc28qf-__assert_fail.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/64w55pscmwz6wl53nh5x11n5qag8gylp-sed-4.0.9.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/67nky23lmrxxyrmf2sd519iplfdkqfp9-libc.a-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6a6diq15i1w6piy38bf29s6y8sjfsxa5-bash-5.3.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6a8p8kb4charqds4z1zf6ka1gbj71p1n-gzip-1.14.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6b0xnqkbv7qnbbbc7zhqwahygjf7mc2i-findutils-4.10.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6b29gjz7rj4mw0ch0vy2m6qrqipz2bbb-pkg-config-0.29.2.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6bra21k8nb03inbdy8qk2ashyj6xhcdl-syscall-internal.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6brx5qj5qw0rj68wcz1xyxi0sdhv82n6-posix-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6c9czp98fd8gahfd13lkqhpq30ygckqm-lex_remove_wchar.patch.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6czr4pwq85m5206gfjxayx6c971id7sk-ferror.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6d922flrrliv0x67xmwni82a6yhrkrha-ntoab.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6dkd05snil6kxm0ml3ljisxrigr5wlza-expand-response-params.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6dv37m9j2n32qvhwr3ixlpranz4kmpis-patchelf-static-get-version-0.18.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6fq4fcp7sbxsz1yvl05qx341zqkyq5va-tinycc-mes-boot-unstable-2025-12-03-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6fv8xbk8ghibgzp7pvsmhv884fhip6an-binutils-static-2.46.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6fvs3jh21nkzmrsm8ka5yi7pglzy3pl0-sscanf-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6h3msgmh2a9l833qnsa3qxwagybrf2fp-crt.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6khyn9406a1k9b1wi5cbw4wysffrnav1-zlib-1.3.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6kxn0zj4ipzpg2p4csxzy64njknwydy6-autoconf-2.69.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6m7j106hrrldsrqmq2cwlf9aj492n4bz-lib-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6mmhdk9d1mw6grbi73sjmvbyv6vsl79l-crt-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6mq3813sfp0vm3c9nxcj10lc26kg9rgz-tinycc-musl-simple-program-unstable-2025-12-03.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6qw1xlhvhf6jrkygxgqc68kiyclygb4r-crt-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6r1diicimr9iaf3f7hxq0b0znqxv5rq9-tinycc-boot1-chain-unstable-2024-07-07-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6rncbxavqydmqxgg4l5n4wyxl75f2p03-update-autotools-gnu-config-scripts-hook.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6vf6b238jhhc823w4gw569pbsrm02ryl-binutils-get-version-2.46.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6vh5kn7pd1k9x103kbrb66yy91chanz0-utimensat-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6vppdb8z7bm6nsvhmbp7ajcbwnjdz39i-ioctl3-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/6zgiwrc91phvp3xb1a09d018l02ibx7j-utils.bash.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/70dvs5gmw3dafy22kfxy4k4pjprcfck7-zlib-1.3.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/70ph6wa755qr51i6m9qjsj8j4mwq0r0x-utimensat.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/70pnb6xgjxzj6v6v27sxfjsbbbpaa3jy-fopen-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/71d53986b7wykj1ig13nh2wk81c7gl76-gnu-config-2024-01-01.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/71n1agkwjvki00q4v63dcn6rsakv49ak-nanosleep-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/71nyals4hficsk0xbbwsi3ckvchjc2c6-gawk-5.4.0.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/72cmq1ms8qih2svpnw3v1rs9a035h2gy-tar-1.35.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/72m9p36fk1v6a4m3989rw32kh45ssxf8-realloc-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/73k3dkk99vv55xybmxpzq958ymg4x2hi-patchelf-0.15.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/75x4zjzb2ydgf9bkizf7fndl9jg5vyl7-libc.a.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/786ngprql5h6i8qcpzm8987rf5mvr6km-strcpy-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/793jv57r8l6sw0cxbl8zbygdgs8hbxsv-gnugrep-get-version-2.4.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/7aclmj1lii828pwwinxwxja8wqqsip0q-mes-libc+tcc-0.27.1-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/7bq69blihkvvqq5np4liiszw674bcqqj-tinycc-mes-libs-unstable-2025-12-03.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/7bw87grjq72s0rdp5j3422lj9c30z4ck-eputs-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/7byvrfyi1f00i8jxsc1q8z997j9r10y5-attr-2.5.2.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/7fsmyydkmaza3z37m84xdcg1r7dz0rmy-fprintf-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/7gl04lx18jp6m4m2f6b26xmz4yd5z76s-math-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/7gxscgn6gvpczrdy9vhlpahw130p6zk9-module-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/7jyjq8p67kr0n0v92w1kmwhp5mvc9994-fdputc.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/7kfxq96dxafy46537pbpr586f1zbqxvz-findutils-static-4.10.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/7ladh3xrna3k85c58c1qps7djmprjjqp-localtime.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/7nq167q7zsw74gprqg48hzxzbcb3z42p-vector-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/7p1an6982kg69v5x7c249wysndkxy1pq-core.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/7q387c5zr2rzn1sfl6p4l0732ag834ig-ftell.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/7q3wpyfk2q0lcppd56x09ac34s5q7hfv-libtcc1.a.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/7q7rg321qf9gv1brx0gn4l76h3ycpcq8-texinfo-7.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/7syw1g6xr2zqzv38cpfq043pz9n8qffy-memmem-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/7vxf7g8swlihvbixhy0l6r3l8cyqqb11-automake-1.18.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/7w302d1c7mxnicpgqgh32zbsb9kc0f9a-math.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/7xw1yyq7h6ngza4cknmw7kcc6pa9f8bq-bootstrap-stage0-binutils-wrapper-2.46.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/7zq4yvhvgqqgn7ly5gasnlkfpm838k6w-fwrite.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/804g2dcnwn4bhkbf98ni9nmq4wghbxnf-close.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/8098ic14nbzr4vvb68k28ylxrykacpg3-pcre2-10.46.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/80i37dl6zigax7nh3w81ysgbjjs6j3gl-tinycc.patch.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/827wd86bx54ijslsnncg2vywdk2gq40d-fclose-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/82nchyb4xk0081577hr776lkywkp0imr-diffutils-static-get-version-3.12.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/83hnwff3zqi6b01b1bq409p05amxvjc5-vprintf-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/83kd55avbcj2wx0gc643yhph72ahdacn-hex2_linker-2-footer.M1-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/83x5nz0d278f729kj1rmwmgqmrcx494x-gnused-get-version-4.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/84w0zzmqzl2bn5s4rx7axsh9znws1rdf-mirrors-list.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/85b7b7c8bp00hcbvw1pv80s0sbv1qnhd-libgetopt.a.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/85fl7xw30igvdd4237dvqd3qx692kd7z-execve-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/86x3nhabnia98ihxzwcvmf19hy5fby1k-readdir.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/896vsnl7w7rhd17b65j8sp3g4kc0zh35-texinfo-7.2.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/898agdpj8623c7hn7xwm4qb3nlkckb2l-expand-response-params.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/8axs2x9qsak35kj0jf2ynfbj1wnv3kg2-libc.a.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/8d25x0aas65p79rc5kzr92vlhgq1w2ra-tinycc-bootstrappable-chain-unstable-2024-07-07.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/8djy268anfnnhb905fh81l734qz66wyj-gnu-config-2024-01-01.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/8f9nchqysay8mhymkghvb71fhz50g6al-minimal-bootstrap-test-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/8gjrq7nfvdhlpbc4licbbqfn291ikq85-fgetc.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/8i626qf2c7brpja4dj16wzyipg7i5s3s-autoconf-2.73.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/8ijrrxc1xxi3drxznpn7h195zkwpnp3g-setjmp.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/8k6l1754s5dxz5i5b242q27xhmhbk3d6-binutils-patchelfed-ld-wrapper-2.46.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/8kvh3k82j18fi5abj6bmqahnrc5yp680-strtol.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/8lfpv18kcxvb7ipxrbw10pahkg5iw61k-linux-6.18.7.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/8lp51hpsccqbr2ydb5wv9qhwhzb83cnd-__init_io-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/8m1p5s3mfpbp48ihlp8bnp1ssqj6k409-main.mk.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/8mlyq4wagf0gv2vdiqzpfg8ld4gqwbyp-fdopen.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/8psl70bgq47wkm69p5dm6kx1ylnpa6b8-_getcwd.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/8qpk6p6zrn3r72935w0rpgqbxjvwxjps-add-flags.sh.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/8rlmsvgdhb7sqgc1ry2hazkk2da5y6hm-tinycc-mes-libs-unstable-2025-12-03-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/8rw2jmn1zm1ygz0zzs4b82mxbgfczbw9-display-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/8sx493k2xnfwakm089q5w1g6pg2npr30-getenv-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/8szf5lgh88ymz0wjd7f62lww0ki0kl5r-memcpy-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/8w6mg2vwlxq16618mrphxkiwafalfbw9-pkg-config-wrapper-0.29.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/8y8aadripzcsxfvg4hsdacwcgw6ypm50-strncpy-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/8z0ip4s3i9ghw3yljf19c1mhvz4571pg-gnugrep-3.12.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/909rhv815886n625rjzvb6wlc1bak9d5-crt-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/90ghkblwz1x58v2bf4j2d2qk7p5ln594-wait4-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/91rcw4mwf0qzfajadx3zg5j4cmzqh9nm-glibc-2.42.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/91w94nj3l6sqaacc5vsjwadprsby7qkq-crt.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/9211kwvmqxz3424zn1y2md1vm6mw1bp6-gnu-config-2024-01-01.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/93hi8ypxk1ihc51l17y9k4bsiybr8y27-setjmp-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/953qy5sr53cjhdjpq9kai6kv0nx4243i-openssl-3.6.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/972r80d928xbv1ymadl6bv7pjl7ldlm5-crt-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/9859v83l3ldymlgh5y6d69b48w5d9gds-_write-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/98bvwkn5mp2g5nmb2007jvfxbh79z35s-gawk-static-get-version-5.3.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/9awbldasqn9q9kk6pjsg4v4k0cmp2hn2-wait.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/9bv6x8rkxfxdlzkqv2c35qr3jvam66sk-libc.a-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/9d2q1bl9997dm8502k6cxg5r8r85pmdh-kill-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/9dh6yj0xma2rsc8c2ylqhqvr6gvy6g78-strlen-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/9dvsrxv7mw64yqa5zyzwnws29kn4ry20-gawk-5.3.2.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/9gwpvw0ssr7xmfzvy97c6f493fhc2f7r-diffutils-3.8.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/9hjwncik9p8h51s705cvh9262f6jm1i3-eputc.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/9ihpw1hw617rlpx9mww1w1cs7ymaa3dv-tinycc-bootstrappable-chain-unstable-2024-07-07-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/9j5mh6n8bwswbwm4ys9nlw6irfzgv4lc-bootstrap-stage0-stdenv-linux.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/9knysaa0w5wbkbi2c1hbvvf7gjv8iqfb-blood-elf-0.M1-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/9ls1fxazpaq0wy94djljyqlwhnf4h6z4-bootstrap-stage2-stdenv-linux.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/9m9m75zvf97rz7f9cml70i1gz2i259d3-getpid-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/9mj2rqzivmn0skmx8kgy0qsrm8n2ph0z-M2-0-0.hex2-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/9ms5jx08n6hdg7ji8fxln4lsfbzd92fa-gnused-mes-4.0.9.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/9nh28gh5xv970hclrry924m5s9a7pc04-hex2_linker-0.M1-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/9qrfjhdgqij7dg7pvy85yw848hchjj86-M1-macro-0.hex2-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/9spyn0z9nrcw03qnavkybz7sh1ngmsmj-python-setup-hook.sh.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/9vfznpw8fbrvvjaky8rkc8lbsay31x56-libgetopt.a.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/9w2ljzcaf1crccchjhm2rsqqsnk9vnv8-mes-libc-0.27.1-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/9wbfdc5fqjxlyl27xa10759yvxcyf8ym-snprintf.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/9wgdxlkh86vnk9z6lpnvksw0pz9wj5b1-opendir-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/9xrg0ykb88mfyf536qk8rns6bxp1jmyx-ltoa.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/9zz696rw3sw7hjgavl83p3pbps1fn8g6-exit-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/a07893l2w29xc0jpqyq7gmckdqd07974-gc-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/a12jwdp6swyb0bzcbmlixi7h0hn3m8qg-mes-libc-0.27.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/a138sap1yaglvngipkh6lngvs33fq8wr-bootstrap-stage3-stdenv-linux.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/a1g2j692marrk5wlyqhxlvclv322gxdr-stat.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/a46fqsk4zs447755lavf26kp3dxpk24i-gzip-static-get-version-1.14.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/a57hhkslijbbzz65khax4qxc2gspx6g5-isxdigit.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/a609q8xwsiqmbq4vf1r2cg9kmw0h9nxc-mes-libc+tcc-0.27.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/a6bsdkrvajh1baiid0bf9ylxd47sv20n-gnu-config-2024-01-01.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/a79snlqv6f8b1vpgffljdnl9hv961wmd-bash-5.3p9.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/a9xrwk0gp9ni3pljp5nz5v0ch73gj5r6-openssl-3.6.2.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/agfp8f5mwgq3pvqla0gsh6zpycjjjbff-closedir-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/agsl5l6inmhgyggscpcn5ng2f1943b2j-memmove-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/akag406i2x9yrv04yz1ydq9hnh63pkks-zlib-1.3.2.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/al3c2iqx4nqjcl4d87m5x1sxnc7m1n0f-assert_msg-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/an791rac20in20im3mlfkirch2pwaw0s-fputc.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/anxcf04gjylc3lhrymdc4c3hmkhkr84g-struct-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/as2v8bpjgrwp7mji129lcpbhs2w0805a-__raise-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/asdvhbkjhgwcd51lafnwqiah250299j3-dtoab.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/aspfvh0wlsi4w6kva34p3m9kkcxr7vym-sort-locale.patch.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/avf6vbj4jmpnn5cdsjyj2693samnvqsa-bash53-001.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/avmqlilnsjmv8lg7442mjb21qx981d2j-bash53-007.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ax275rpcdyhncynghzfbz409pn1r9bjk-mes-m2-libs-0.27.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/axkmfgaz8nvjw8q0l0d1bhm7glqjjb7j-nghttp2-1.69.0.tar.bz2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ay5c52823flsrmd8887q1xdi9igxq7m5-strupr-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/az1hahmwb7l2hzvgqy0w6rvqbln04769-xz-get-version-5.8.3.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/azx7db7w8nnrdzwz1hasa4a1psi475r4-execvp.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/b0faqgmz5282qmis5v2m13b7bsc0y9jg-gnumake-4.4.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/b147bn8sgyn2s58vy3cw59d89vrd482g-mbstate.patch.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/b44mdg7iwqp8z0szrncb0asls9q2l323-globals-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/b4xxjawvjh9i025i567xfv8lh7r8aix7-autoconf-2.73.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/b61k4wb1f0c05y70qj035s9n98bsydx1-tinycc-boot1-libs-unstable-2024-07-07.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/b6266kgqhlnyhdkdp7027kdmsq2aw51l-libtcc1.a.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/b6yxpjizcj55gzmm8vpjzdxgl15246qv-byacc-20241231.tgz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/b9bd2bl75650mdckkiqshw1bld86vn0r-heirloom-070715.tar.bz2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/b9jn5s5rpykzz8q7isifnsx1phmwl175-sprintf.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/bamwxswxacs3cjdcydv0z7bj22d7g2kc-config.guess-948ae97.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/bbbyy52mymn1ghzh8kf3h0yxsgddj26d-ungetc.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/bc1pgwnxifikxdvgbag1i396jp1klzgx-ntoab-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/bccvb6y1c2ssd4zvmg7iag47241lirh2-strchr-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/bcl7s5w3dyj4sj5v3l2v8qh78b0j1pm7-libgetopt.a.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/bf7qin7fa9d8vpd5dna84yknfd2qrl35-sprintf-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/bha00fzgyxii9w9q9y3w14ynjzivi9k4-open.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/biz5hn7rgmvdry6kan1kr31v6mk6gavl-strtoll-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/bjlz9wzdlh767w232y6n5pjds14raw5h-dup2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/bkijx1mlrhppg2b1zandixpz9w2lxnvi-m4-1.4.21.tar.bz2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/bkllg3nvs9krj4lvxirs7321ckmf6a0l-display.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/bkmjhag9fkyxdmh3khll3hd7fbjqz5zl-nyacc-1.09.1.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/blibvc5hp8ky3qi4rpwkxcw501akd418-oputs-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/bll3p7icr7ylyxprj5n60j2r8bkf5qa3-mes-libc-0.27.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/bm21lvr3ishd7qqkhiyq7lqjz7j4sdgj-libc.a.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/bmlz5dr8h49pdq0dn4xdxsw3r9z1wx1z-remove-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/bpzkd3r26cl6hwr2v0y7giswnd9chr7x-memmove.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/br3yqsjafxyhh91879brxq1xzbyjwlwb-ls-strcmp.patch.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/bv4a5fw05q4ysq24wy0z06bd5jysq1p2-gzip-1.2.4.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/bv9pjdxwgwphc534rna0d483wvjxcmwl-heirloom-devtools-070527-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/bwbl3xl91p3p02ylm29nkjjhyqdsr4hs-umask.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/bx85m89qvvn4dgmnb3rafz4ffnk41rkx-kill.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/bxcayhyg3gnlg7zfcvyrwdb9cwxpl09v-patchelf-0.15.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/bxv5sgys76rnsjkp7jl41n4ml67j5l9h-bzip2-static-get-version-1.0.8.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/byylv1iwrisj83mlsi9qphsslfy20idd-autoconf-2.69.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/c08j9sk7k3dqfyllv0l5wcp80001rjpm-tinycc-boot2-unstable-2024-07-07.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/c2p74ajc9zfg6dvmv48f4v1xkr82j7ij-time.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/c41rl08zcn7y0d461qqsig8vv5xbzcyp-bootstrap-stage3-stdenv-linux.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/c4q67q5gj8f0y6hb33cak3f7sjpq63c7-fsync.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/c672ird0l7wv8dr5idknpgnh1rhzlnqf-coreutils-5.0.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/c8aqhk9y238r2swcqcw6dm5qh26lik9f-chmod.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/cblshqv4b81g5gxis3zpb4inagkaffi1-binutils-2.46.0.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/cc7v7pw1q359kfnkw3g4mcx0dldycpwy-glibc-iconv-2.42.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ccs62zg6j0khwdjgswyv9hnd2gwc0l8y-isspace-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ccvnn15aaggbqby30qjzyp17x49bjjb8-core-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/cj22vbp3vzj8xxw4h8wdf04gqfh4gv9n-strtod.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/cjzncxj18nk2xpm15gnm3a1lgrqbicpn-tccdefs-unstable-2025-12-03-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/cm0rq8z9vapni52r76hsn0r1y5rbq227-calloc-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/cmgw7lyp6b47wd2xk9lj8j6vx985k1p0-libtcc1.a-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/cqm0q08dhgvsdq3hvj85lxhq173x0m6d-tinycc-mes-boot-unstable-2025-12-03.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/cqq3w2y9gxzvjsggwxjvl0viwypnsfca-gnutar-1.35.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/cqzx6j0x2pa2p932j12nqw1cbz7mp27a-builtins-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/cs4lzs1qmk23nyzgf751a2a4kb7y667v-mes-get-version-0.27.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/csih3q578qgyycfjbypmnkmrxsg3rdpf-libtcc1.a.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/cv6mdrwwydc2927kkz3i29b6k9k639bd-printf-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/cxh8cwlp0z4a7xl6bk7zsqr7ci1hzdq6-make-4.4.1.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/cxpi1vdfflr3zh9xhgsqz44vqjk0jfk8-mpfr-2.4.2.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/cxyal2ckly5vlzvyxkggkl2y9xirz2f6-__buffered_read-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/czkqbq1rq0ch1lpla9bgsgwarc6hrb72-tinycc-boot3-libs-unstable-2024-07-07.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/d1simrxi41d15h88m043y00yd400fpd4-cb41cbfe717e4c00d7bb70035cda5ee5f0ff9341.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/d23sd66xkg81wd8j7m0jgw91wmsafspp-fputc-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/d4aaks7jhys92cqj8b0c26xwqhqijaaj-vsnprintf-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/d4p16n5n0kx7hss8kqa8dmrlf8zxf356-chdir-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/d6373avfzg9j0c79mdd4q8rva2izzl19-sigaction.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/d6dkk85sixrainacqixn91kap9z3z5n1-tinycc-boot3-chain-unstable-2024-07-07-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/d6kq0iqwxc8mff54r3ffa2kqkqkif008-binutils-patchelfed-ld-2.46.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/d756f43xmzg11rr9g98n77x6ah6w4kcw-libgetopt.a-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/d77vfr3v3vvbs9v964ngmxlq5dpylgnm-clock_gettime.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/d9hngw468h7b7fpcyx284kqwdgcn3gzy-bootstrap-coreutils-5.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/d9q3m3dkpm5f2yidcnf39bxqy3xniw6q-M1-macro-0.M1-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/db0z1k8vqx455ypsma6c8s698kd2fzxc-bootstrap-stage-xgcc-stdenv-linux.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/db1yx8fllcas5nni56p9fq7dghrki2gm-getchar-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/dbasqqw4gv4864jxfc7i9fir1ik1h91b-mescc-tools-extra-mkdir-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/dfly1dzyy8qx8y260mz1cgq0p2wnvnr4-tolower-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/dfxx6gjnya5f876sf152swznsacb95gp-vsscanf.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/dgpv1xd3s1mj9vc7z66610s5p1ap1k4y-strncmp-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/dhjhlihqj08f3fs1cvsja0fims0dqnlw-raw.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/dhkwk75n73r7622d36chy0j0pr8ak7vd-findutils-static-get-version-4.10.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/di9nrzh0qk5z7igcimmzxg8xx4q4s83v-string.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/dinjsy9j39gclj9hk2qaf7l296bj4053-binutils-2.46.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/dk7swnidjivlr8j15vjn7nww8aw3i4pp-crt.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/dl3gi6dn65ss5py6mnm0iqrxvcjhl67h-toupper-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/dl4wsj8kpyb1whdhhibqvqqx3c6rc60g-libidn2-2.3.8.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/dlks1y7nzrw0z05rbcrr5cg7lrdcj7k1-libtcc1.a-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/dmlvfmqi0x2hdzqb97fic9jrk0bcany0-fdungetc-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/dmz6h2m7nf6n698mm14i6zyms33dhaay-chmod-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/dpr00azdamvsr009q3jin35dgkqys66m-tinycc-boot2-unstable-2024-07-07-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/dpx3qn66di2g74234vi7xq4iizknmicd-gcc-get-version-4.6.4.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/dqg4r1dzlcq4zv2gmhdm2m2cdirchy2n-tinycc-boot2-libs-unstable-2024-07-07.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/drh372b18rwrl15nrww5bc105xpwd3kx-blood-elf-0-0.M1-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/dschwsfi21lwar5ldv5wdqnm1waf197a-M2-0-0.M1-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/dvl148vwwn460fa0zq13z8wlyrd2fz8k-gawk-3.0.6.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/dyffnfdiz10lpiraxazf9k9chkx1prqj-M1-macro-0-0.M1-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/dysgi0lbr4lwby0csjpgjx9qpalzwny7-libmpc-1.4.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/dzkfh6l54awc99d4hb1yi0v4mp1lacwa-gmp-with-cxx-6.3.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/f1flrx1z8dp8l9i1fq745wmbikj3pjlr-glibc-locales-2.42-61.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/f2yqpqsk58h7c7a0sv2q296a1b88svqc-bootstrap-stage0-glibc-iconv-minimal-bootstrap.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/f4svikqqff9hghyrwn25bq7kf2kid2dn-bootstrap-coreutils-5.0-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/f5g8kz53hxyz1k9q4nlnp7kpvik4yd15-rmdir-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/faa0yzm7giga1x8llmhy2sxxkh6j89f0-strtoul-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/faklixjfqrpfwz3z513s6p9lmk2qf3mw-mescc-tools-extra-cp-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/fax2439jq23r7zcxs44axd7aqzmwbnxf-gnused-4.9.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/fb3zzjczy1sbr0w1n2818d6zkdpkad90-diffutils-static-3.12.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/fbfri313mjc939zmfif03wrhj4nr5f07-add-flags.sh.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/fc9f5c6w5f8ap0gra21l3q7hvcgr9wnr-globals.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ffisxdinfy53xam8vhlqv8c5jkj29yi9-mes-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/fg5yn059hlcxgb0116bq8s47rbg4gzs2-M2-0.M1-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/fgw96sp22039kys7pyjaxns4qxckh829-syscall.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/fj69pic5k2hr161cw3spagly2cspp3x7-main.mk.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/fk7844pynkdlf9121b12ipbrf55c9g9h-fgetc-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/fl6mqjpa5193l28xwgdla6mj4i186568-tinycc-mes-chain-unstable-2025-12-03.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/flxbh2drlvipjlaabnkzij8dxcv56ayl-nuke-refs.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/fmhwfv6ikfd8wsg2mha7rw8nmdc4jxrx-hex1-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/fqdmyb945n7hx152y0n26bi0b0q7i8np-tinycc-mes-boot-libs-unstable-2025-12-03-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/fr4s6crprj60n9wwcyf7icplz8dxp1ls-binutils-wrapper-2.46.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/fsh8qdzcqnfw96hhy6ssy2vw0ljjsc8q-malloc-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/fslv8fzwwhv8zcvli06y1p66w9jh9b06-libtcc1.a.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/fslvmm29rxbvmrl8mssmh3f252k65xmd-fdputs.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/fsyp2yvcq1xd2ag2ww6f25ri5pm8ghh6-lzip-1.26.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/fw6qa513xbkv6cmywc5bmnl42vn3is7a-make-4.4.1.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/fx1ariccq0k411lvhjl3iwbql4qq6xiv-mescc-tools-extra-replace-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/fxgid84c7kz0jkac0gwpahxpwvr5nsap-rmdir.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/fxh7wabjwq8ifyb45vy2f09zawiynyca-ultoa.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/fxhhf9vlwrvj4q9c5smk87kmk4h9k63b-patchelf-0.18.0.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/fxii7wgh8167mckd8bbjal5mxq9l9p05-M2-0.hex2-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/fz6vky5jh09qakfdzx2ygsfasldr6p0d-tinycc-bootstrappable-unstable-2024-07-07-source-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/fz7m1s1j4v0gkiggw7s80c9yq0rqgj10-libc.a-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/g096i4ibwxnri1f7zmij06pjv7i4k651-assert_msg.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/g34h3g6sjisk7fqp6lc0f5mcls3x2rd6-lseek.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/g39zbiv8d8qbgzkqk2s03zdfc39b1j4a-which-2.23.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/g3m8vb911l1ij9ysibrqznbcf6y4vnzd-qsort-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/g5ifkaw6hws40d16zkprrgb1ddlb9wib-tinycc-boot0-unstable-2024-07-07-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/g5ms3szgrrh6p205vmz90qj0k181b54h-strtoull.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/g9cwbx1jfj0vfxla7ifxcrk2xjnw1bsz-readdir-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ga1b8b5z58fbkr8v0xv59ivzir4wyjqv-tinycc-boot-mes-get-version-unstable-2024-07-07.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/gb36vm7d2qjqdsc89bdcmyn7qchy7xcd-access.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/gfbn7q8wqsq5yzv3fi7sr0pf0kxj376m-bootstrap-stage2-stdenv-linux.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/gfjavirn9zvghj3r86wbbpkhw9xpipzi-builtins.mk.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/gh33z218ga15qwxb7vr31yfi774rbbly-fopen.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ghj56d7dhjni37apnjxrl01wiba0zghg-bash-5.3p9.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/gi3ss7gv2i9hx4x78yyaw0qnq8yg3lsm-vsnprintf.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/gi4k3pj06rc4b78l2ld3czi2qgd6zbby-sigemptyset.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/gi8pcvkd1knjmh59d57ka1yqxx4r1b4r-link-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/giffxpxvvdncny4f42vbv6ndmv6z7q7d-fprintf.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/gnm4qxbivv7y2n112w898rdabc0q6pxa-fflush.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/gp0g0x7dg0gvjh98xfkyajni6qa5v1n2-cc.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/gq86ghc61gmkl70f305w4cppzv0zjnjf-libc.a.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/gq999sv6ad4gpmqly6rm9jpq0g49pzx7-libtcc1.a-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/gqc0mq7fw0ckyzy3cp8qfblp4qldnl3c-lstat.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/gr7p5aghmkv93kv5vmq42nn0b3372k3c-m4-1.4.21.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/gs9mwc2r1p7hs6xrnqz5s36dsw3c9nqm-mescc-tools-extra-chmod-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/gvq841jfhz61gjq9g1mdy54973sqdf9i-update-autotools-gnu-config-scripts-hook.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/gy95ardpys0lm0djcwpq4mym7iid5636-brk.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/gz9irkx06fl1lavrwxkghxnmlm4h5fgh-bash53-008.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/gzp5g55jas54wn71b4ympihys0xfppr4-perl-5.42.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/h3fr2g42a1jn2j3wwlkf8jawdadj3nmh-atoi.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/h4fhkingkx88wnxhq1yw36xqdqim2fd8-buffered-read-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/h51d0h0pwlgli2m8rx4r5a97pihqvsm2-access-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/h545qb92mx5i9q4amb44d2phh9by0zrq-gcc-15.2.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/h5c3h1i4pn51c6d9jnm0hc0fb30apx1n-gcc-simple-program-15.2.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/h5z9yb9fcb61sij3pzybq04rpyzgx391-fork.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/h699vzb1lagddixp8liilf9kihibax6x-tinycc-ea3900f6d5e71776c5cfabcabee317652e3a19ee.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/h6pi2cyw7j5liy0fbzq8i1ippcqwdjd3-mes-src-0.27.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/h7ijrlrjxvy87frncwpqivqxn9zaqx5x-patch-2.8.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/haldh962ah38jwv0i8f72nr68nn7vw9r-Python-3.14.4.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/hf1vsgpn7231y185ckq6z2lc6v34b3kq-eputc-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/hg84x27sr3pglh61b22jf93x34q1694p-close-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/hggb46w9b2gp3cqs7z5z10379adalrga-tinycc-boot2-libs-unstable-2024-07-07-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/hh6gy6f43hwplqijcn0z6cgswh975p3y-autoreconf-hook.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/hhyhhkpw6v15rzpbyw2y8qzxrz3wv2gl-bootstrap-stage0-glibc-minimal-bootstrap.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/hq2cbwrkyyck3hl3vcddi1256qk4kd9y-M1-macro-1.hex2-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/hr30bp0br6nigp0zv5jmf9l4g3xsms2q-diffutils-3.12.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/hr6aiifp9n6gl6l8pc3ig55lzw0csw31-bootstrap-stage4-gcc-wrapper-15.2.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/hsw0bb8jg0nzvdf6bq2223ik4vcmic15-tinycc-boot0-libs-unstable-2024-07-07.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/hxagi71byf79kd6630nkzzslsmmjir34-gnutar-1.35.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/hxbzg49czsbmcxx8wlxa8gqaafznffd1-libxcrypt-4.5.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/hxsqmjd5sp9ddjjyhr6gmrhf58gpad1m-fdputc-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/hy5d29af1p9k653rxb16254s0xnmssn3-putenv.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/hyjhwkahcbyhh6yirys29d9v0530x4cv-crt.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/hymk33wkc3qs89h56hxmcs54n9nxafpa-execve.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/hypg46mi1fwnff9f7n14ialasjy7b550-strstr.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/hz2a2dlrpqz662khc2f2nqcfx03wgn3r-nyacc-1.09.1-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/hz4s5xc0rxrjcf6c51nbjcmbsl9zy39j-libgetopt.a-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/hzisswpz8nrmfgc5hgcvv9qlijlpazcx-update-autotools-gnu-config-scripts-hook.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/i0bhjf8ai1bl51bvpyc112bjmk46hby3-vsprintf-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/i16zp1zmdi4m4wcqq9y3prcbgfcr3hwm-libc.a-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/i41fymycfib1hv8qdjk3nsk0h7g578dw-strchr.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/i5c7m3r8bbas0w8zl539ng8zzr53mxr8-libtcc1.a-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/i71nsvc83jpla9pszrll657an2nsdsil-libtcc1.a-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/i8ss46xy764d9d593vn814niksfcriv6-tinycc-boot-mes-get-version-unstable-2024-07-07-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/i9jy5m8nyl16d2djd48p0d81xs4na9pp-fcntl-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ia4sjsi3cl16751r14yfv92603830nnn-gmp-4.3.2.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/icd8xv4gd1q59850bdnps90lnl1jq351-abtod-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/idh7xqk7a67n07f69jpsls2ncfz1ghnw-unlink-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ig80p3grw6fhna4qdz9nsxsb3z7q0h6z-strcmp.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ig962i9my0fqk3cw7wzzrx3kmdq72571-strrchr-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/igp30wh0ssfl580sfh321gwhfsjfnsjn-__raise.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ihmylc6qh8a0qlq60p7gj92c59cz7z49-gnused-static-4.9.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ii3jsqp9rrwv0g21lwbvb94ddg8c5m7n-bash53-002.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/iikvpxx1q5fzpz07klvi4r458y6nq3ks-krb5-1.22.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/iixshw8nhrf8cb503b57s9p94cl35k8n-nuke-refs.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ik8l0khmd493q0mxq1c1gscmir3c8rql-ferror-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ikynbpdjj2lr2ivhzlmzv9gw05fa7ibn-tinycc-bootstrappable-unstable-2024-07-07.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/imyy7rg9gyf2qyhg3jjnmal8grabvra9-glibc-2.42.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/in9n30q90i5kwqf9rahhp4vnxz5y117a-raise-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/iqppcir7yrxh3ajirzqd5krv64dd9vix-isdigit.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/irpf44fixgjjcisk8ifmwqwbm49mlrlx-mes-m2-libs-0.27.1-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/isxmry87mdhxx1g979v9wcr4m3wg4gpq-diffutils-get-version-3.8.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/iv3hf1ml1fdmiwipnbz2g6ijaxy0kss8-sed-4.9.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ix2s33fi4nfh78rs59mdsdnq5dd5gs7g-tinycc-boot3-chain-unstable-2024-07-07.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ixwy12didmgk7j4z7ghlirpw46ahmhbd-mpfr-4.2.2.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/izbfj3rckm6lyip0y99dw9hyyap52kg6-libtool-2.5.4.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/izqfaybr44m2b7vrs3j466z6g4cn32w2-libgetopt.a-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/j0l7801fhy6ralfrcw6b407bksgf3hp5-fread-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/j1aiw0pfh51nyay8zavyx3adq86xdnkl-tinycc-musl-unstable-2025-12-03-compiler.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/j2cw7yamyxg8lpsgb2yv46v0sgf8vlcr-builtins.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/j2h98vg3kzqg6bif79mfkzb3sv50id8p-version-check-hook.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/j4wjiwwh98lysm3nj7gidv06ch0zns2p-hex2_linker-2.M1-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/j65zznfqb602v6ar0k7a7bb061iwshr1-strncmp.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/j76if9fyh6aw41qvc8nqr9c8ha4azayn-hex0-seed.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/j7jfgy99zxzpj6xd682dg3s75l1scfwm-string-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/j7p42afk0zv32v4s14mpsmgmlnwlhgvw-isdigit-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/j8j2dnbhmvnry5ijz8qhl5n4ghjn150f-fsync-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/j96xnv3mwd9lrpw3p12c9dihl1nq7bi8-utoa.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/j9fv75h81b9rz9kjfcgxpc9vaxz0jywk-ed-1.22.5.tar.lz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/jbiiiby5kxq7vwa94liwyw56h23liih7-tinycc-boot-mes-unstable-2024-07-07.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/jfdsjqzp6ba2sxydfwvclbash9z0f3zf-gettext-1.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/jfgdmzd75pybl8jbyv0qharzncawvfjk-hex2-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/jfvmc8risbjpwar6fg5if3yy3zw7hzmc-fdgetc-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/jfyw0fak0943dzrsz6sqc1vw92x9p4wx-tinycc-boot0-libs-unstable-2024-07-07-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/jhbdwrvavmabzvnkmmfahrway87dqqqd-update-autotools-gnu-config-scripts-hook.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ji275smnd10rlnv1g0wik0jhwi96swlf-gnutar-static-1.35.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/jk8228a88vjsa0zk004sgdhxp5g4pnnv-hello-2.12.3.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/jm8c5a9ysfm3fckryllnl6pd3nm1hd17-bootstrap-coreutils-musl-9.10.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/jqvfw029liggk29pxy4rczkk6744x5db-minimal-bootstrap-test.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/jr8wxdxn0fznnd4nrximxkwbfix0cp38-vsprintf.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/jrvxjq9786fgbm7za49l915jfv1sm879-pkgs-config-setup-hook.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/jsdxi07zgfm66r2338nhsi3q5q6xzv8j-vprintf.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/jsplhakxa0pffas25325jalphic54571-libxcrypt-4.5.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/jsv60xbic776ir115ikf27qm7y169371-mpfr-4.2.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/jszdcrdanr4r34r4g5gv6dz15n7fzp79-locales-setup-hook.sh.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/jvn6rxfx2j66k0svyl5njyqzq2nh0v5s-gnutar-musl-1.12.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/jvxpzk8cr90bn2a19fdi3qbflvqf3cbs-tinycc-boot2-chain-unstable-2024-07-07.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/jyh0vzwz039ffsgmysy34jsbp0mq3w3v-remove.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/k1cn30bfjwfgsfvpm1yp1nms1si2wf2x-nanosleep.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/k1vkfqjn6h31hr48gdd2ki5yamwlzicr-fseek-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/k3bc319glc91sa222f5g8cdp43brmvyx-strtoul.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/k3f1j20sax11l61frp7kfn3sns54qv1x-putc.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/k5rg2vwnp79zgc3f0ybnr7hhvg1r37k2-gnum4-get-version-1.4.21.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/k6s1z4harj1xjv07yhx7j6ypqdmrmm60-islower.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/k8jw844syw3gyrpj9vkyfbgv1bh0jx6n-stack.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/k9kvp567r0m4ixkdanr0sxg8fyw1bhm9-bison-3.8.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/k9map8ma3f064ibw34sr5mrnd818fk35-globals.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/kbirnxddc12bv3px7x7mh6vha8rkrcdx-getchar.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/kcd8hlvcamwjs3p36is87ncigh6i1zbf-tinycc-musl-unstable-2025-12-03-compiler.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/kclz14inz8rmx1jg77kb5nl3hqgqjnc7-clock_gettime-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/kkr0ncy5fyd3rv59wdb679q0fhah7fn6-M0.hex2-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/kln9pm3dds6m4gz0npsp84gb1qqm3sqv-lib.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/kmm4m261ix0b3b74380z1z44d737jac3-fputs-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/knxryyvdxg6ppqb06f25p7xfg2lw2a7v-ftell-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/kp6wvw7jdv9ssi4zafs58y3vn5zk6ldx-strrchr.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/kpg9rvnnj20hmjlhagakbh370b0wgmj8-memcmp.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/kq40l5zlizl2nz5w2qz07vdsi79axxfy-heirloom-get-version-070715.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/kq769aa9ay0qz50alprfqlj2bmd0b7ir-libtcc1.a.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/krnzk3z85c0shx3jwz68v0pq2yjv9k2k-search-path.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ks1l1rm4andjkd4gxr1qj5iqwflgmcsi-tinycc-boot0-unstable-2024-07-07.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/kv577c7qpn57nn2mv7wixhqn7vm1q1lr-bootstrap-stage-xgcc-stdenv-linux.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/kx5chwyxn47w0mg2qbpmxfqcajvdd9fn-dup-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/kxb0bf0fan1sq37khpxqnsz9fmgxfq7m-getdents.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ky8l5xba6d7j3x545js7ihmw1cywbmyi-Python-3.13.13.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/kyq3pkcn2zym902ks396id0nhczxkw9q-snprintf-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/kzg7vv4ws2lyvisn80zlf63gzh5bff8m-bison-get-version-3.8.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/l0580hbqkpfwj5d8ggm480zq8k8sg2nk-localtime-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/l3c18sdal1zr6930p31awpmz1ncs68wy-gcc-core-4.6.4.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/l4q7jsk9n0ys3qb31zafgzwakpwq0360-libssh2-1.11.1.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/l4xhnq4jk2fxh396c3ksnlg7rbljlr5j-libgetopt.a.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/l5s22qij435ycw3f6is0bwyf5m67fs7m-realpath.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/l6a6rj0gy31gp6k7k80qzn4ni0yvyzgh-gnumake-musl-4.4.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/l6i9qfdw2j2c3ggjifrpsyc8lmn0mrr5-crt-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/l9j90v4nlsfd1zgmxhz6y4f4nrjnnlr0-gnupatch-2.5.9-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/lb92pc9xa37l3lqhw7r1rrg3gvfvr87m-libc.a.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/lbn6dc55b8f36v2p2hkqbpk2z63wnjyx-touch-dereference.patch.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ldgmv28v6i8dzq8vs5nf0da0qpzls80v-tinycc-boot1-libs-unstable-2024-07-07-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/lfxjv1zhyc2wl6sz7m55bvl896zgb4f2-symlink.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/li5ls9fwpvck2n5hmbvg949a12kay86w-variable-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/lj67kbyl99ja30x42rfcnbvzqzsxiwrj-libgetopt.a-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/lkkzb8ybhp5jjgvb04b13n68i51lxwd9-keyutils-1.6.3.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/llki4kvcykzkpn69hpkvxk86gpjb8cmn-oputs.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/lmflf3xziz6s4ia23j485l3gvd97vlg7-umask-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/lmj4kyrya73pxb4l75kqvcy9wpw4ilzn-tinycc-boot1-chain-unstable-2024-07-07.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/lmwdsxcxmjgjq861yz7mrqrqq918hqj2-memchr-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/lmx5h1xhlck345nc3as6dglfv57aq9rn-tinycc-musl-unstable-2025-12-03-libs.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/lnfwrgak7pvwc8mjp1c58h4mhb5857pw-tinycc-bootstrappable-unstable-2024-07-07-source.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/lniysvvb2862r072fmdbxahfkihryw0s-strtof.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/lpdjj0xvn86axxiz4jxaq68h5nnfjdgz-gnupatch-static-get-version-2.8.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/lqx91q2khs5xrb42p1lirxvnwdh23sg5-memmem.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ls8gxq4vrviapqfzbnvlzn6md9zgfy5b-fdungetc.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/lw7jnk0vrdy642zk8whqxm98f9273k62-make-binary-wrapper-hook.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/lw7n1gm9yimbrnr00h6l6pb4c9hkb6dc-bash53-009.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/lz4nn74jnxg612cl2jzpd49baqsl19mc-cc-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/m10ysgh47imr6jidkz6ij1h66rdfqjgl-tinycc-boot3-unstable-2024-07-07.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/m1gq6x7srswfl66arij6spxqnrp7sdcl-bash53-006.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/m2khnp0iw665w7zjkyvil62v5wjlbl8i-patch-2.5.9.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/m2q365smdp66pvh626qw4yjr17j90s8b-dup2-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/m4pkby9pv0nwasab4jrr4l2z9qghmqjg-glibc-2.42.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/m6wmvp7bp6c8kn3jlbl4my0g467kdl0l-strlen.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/m74651b793zgyyvlk9gx7v1cl1ywslib-hello-2.12.3.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/m7hf0m4lksycc3hf48zvi74lkna0v0kh-bootstrap-stage1-gcc-wrapper-15.2.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/m7qhh4k03ymh4myrj8nnanzgr2pcx5pp-yacc_remove_wchar.patch.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/m8i4xrdxpk18vi054wvs4j890g8mx728-ln-boot-unstable-2023-05-22-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/m9dxdssk2lq67wymiir61ihz7317y26f-gettext-1.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/m9i79vj3c32hfrxbbginyz4p252jrps3-sigaction-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ma479p2ss57bs6afjmhhi1xg27vin41s-gcc-10.4.0.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/mag3pq29n9f6jw0rb17crny8ilm4kfjy-diffutils-3.12.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/mbfqb9y6ryn2axyvg797s3lw1w49liz5-gzip-1.14.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/mc6k9j8kw27a9av46g6g4d85nscjxfxy-gnumake-4.4.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/mgl0nby6f64kzvgc236801pf7jvyv21x-bash-2.05b-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/mhrxc5w6drwi4m5ykbdrayz7869i9mx9-bzip2-1.0.8.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/mj28d302q2w5hixap96y39dq8b1la411-free-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/mkwbpkxwzssdkc9c1rsx0s37kj951sz4-missing-defines.patch.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/mla70gpi43cs9h1knjk2jvk5j832rin5-libgetopt.a.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/mldlm2l5hgpjlcmvzl45zrl2053pv129-stdenv-linux.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/mls5sqcps3yi6iwbn5bbyr1pymsk8cyy-kaem-unwrapped-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/mlw3q61rvs7x68gxr9pl6m3c6wb2mgfb-ed-1.22.5.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/mnv5bkp8cn6rc8x75gl1wc77mw45i284-which-2.23.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/mpyi8xvv5i4ydkmgv1ly2wcmliz8vq5l-putchar.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/mvwqinqcy1zjvb5zmhji91rjnmzh0fqf-tar-1.12.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/mwr39jx5bysz6lgcj42dp70q1yvjr4dh-fork-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/mxbj66h6gclhhqy318i4v01lx2qiy9z3-__mes_debug-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/mza46h92jsn5430vmsh4sx8zq3cc2v3y-atoi-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/n36amj3qiwfq27zx58bw257yv8vwyj2b-pkgs-config-setup-hook.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/n3sc2081xz33g8q4l4svcw8v6v3qh9r9-eval-apply.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/n446lla0nljzkn8bzwrv98kn2qd27rf6-gnutar-static-get-version-1.35.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/n4ys9wp9c51jzcwbf3vakzzbdlis2p8r-version-check-hook.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/n55h30m46ndihq18phz5n0cr6ii0jzmb-getc-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/n5j4h0w3fspyk6f5q197ii440gsjmk1k-crt-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/n5l3ikj9s3jc243w2rjszsqw930rd1qz-gnu-config-2024-01-01.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/n82czkqpwvkzyjar5xgf5x8s05ih49kv-memset-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/n8v417y9hc2r3zyfnbjf8fg30v6hm529-python3-minimal-3.13.13.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/n9w1dvf9k6617mqdqj9vr74ajzpq2b3a-bash53-004.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/n9y1n1w3cwwlivnqhv98axp155wv1b76-sed-4.2.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/nax1rvp3q4xnj7l9z726dm9nxablqlgl-tolower.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/nbrgq9hv4agp3mz77z3mysr1798ik6q9-libc.a-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/nbsdqpfzh1jlpmh95s69b3iivfcvv3lh-config.sub-948ae97.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ncn47n6zrpsy3kxc146392s75fk4lk2c-M1-macro-0-0.hex2-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ndg73iq0ha0rk67lfcr5i0ni1qx93wi0-libc.a-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/nf9d42gssxwv5i3ln0wclcyhnjszfqk4-zlib-1.3.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/nfg4vm8619w5b6fpal9swrkx75nsp3bx-crt.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ngq1yidsz2dihpmkfl4shq98ky4rgpq9-utoa-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ni3mx708vbjmn68ziv908hia51ah5am6-malloc.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/nix4lx9q3da89ip0fnd3rjpi1kjacif3-mes-libc-mini-0.27.1-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/nnkx70rnb45cksjrsh24ral5jd6al5h2-blood-elf-0.hex2-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/nq6l8wrdqwnhmzs5q71468jrgbwy1bdp-coreutils-9.11.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/nqn9nizwqy5201zcv0jn5l7pnjm8i637-kaem-footer.M1-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/nqvwimqscq7jb7wgbdikwj9f6i6w4ffj-libgetopt.a-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/nr843fdsa5jd0ib9dhyg68xsxfka8zyk-printf.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ns18qc97ciac3jfhnqzm1xmn4nnll4nr-strtold-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/nv9vbwdbppdb9zvv78m9a99vga207ykq-findutils-4.10.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/nwlwl6mkyy3a4s9hisffr3vajhk50c5p-posix.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/nx5vjcba7q56d0xpr7dj0vy0nln5i64k-bash-5.3p9.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/p0h6a81w4c00c94ac7wmzlbldh8v635c-expand-response-params.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/p1525f5i8db7wrjjq08xh0m8hyjc1wk6-uname.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/p1jb1am4inf7c82i05kybj8iqmiiyldh-libtcc1.a-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/p30fckwbymg431mqzcfixvd08p6v5h4k-chdir.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/p385mc566pxgq1xsw5lka31mmnm39i0i-libtcc1.a.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/p3a7v0sy252p81wn1wrz56nn6gqiwv3n-python-get-version-3.14.4.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/p3x8835w2rwiyy79ydrjmnz7kq5nsdia-tinycc-musl-unstable-2025-12-03.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/p5qdp6gr977n1s1518lpkn3zk2m70c8k-ltoab.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/p82iqqa3j8pphqij384b968pw78z8i84-keyutils-1.6.3.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/p93ssgf0b6f5bnzlsmb8zqfizhh9jfmr-xz-5.8.3.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/p9x2bbq9n0mqccsg834h99sadrgmsj7c-stat-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/pb2vzi7llzk14zmv1iv95qdmbf0lyjnn-libunistring-1.4.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/pbrsvvhf1i7d9w3h87gk60q3bsngp69z-wait-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/pfnj6kx8hcpl8srpk4grlj0c09sdnfaq-mprotect-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/pgnc438nsd0y87xi4fjwzyhc45i4n8v2-gnupatch-static-2.8.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ph9222zgigjrdbz0hlml3i5kv86hkqwf-patch-2.8.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/pj475p4a7p2hzb49xslzyrbzmnwvjp4c-cc_arch-0.hex2-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/pkdlmcijz51kcm5iinph1269fy14x7ym-gnugrep-static-get-version-3.12.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/plhffvbfj7978vpqgda595ypr1czg3s2-make-shell-wrapper-hook.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/pn63pwy6gv617hpy7r33m6l4zifafzfj-calloc.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/pp31fvcgghilkm5xbgbblgd24vv7rl7v-strstr-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ppw8q27avhjirlrdcharilyf3lfih28f-hex2_linker-0-footer.M1-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/pshx4r5vypzz5cjy5ikscgjyzhv2s99x-gnutar-1.12.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/psn06pbp7vp4gmhdbh53jq90wkx77k4m-findutils-get-version-4.10.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/pvhb5byc5ya3z3692i00lcsr7rc6d6yj-ultoa-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/pxxi73ah5w9h8987s2fdwspybf6jabvh-grep-2.4.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/pxzj1p0myjc6zd2q95i8f82y8vg4ymyi-__init_io.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/pyy0lwwdfhrxws0g4w4kq88x4qikyvc1-puts.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/pz3m6g7iabylyazm40p363y8795vybf9-mescc-tools-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/q00djxdnx6fivlqp60l7l37yc42i1fhg-gnutar-get-version-1.35.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/q0pxfghmib8rkqh9dfbc233llkihs3h5-isspace.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/q0xr619c14v9cl1vhyq83407r23npx4j-_read.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/q1rm5rl861j6yy2mi0avl6n6p2s32pf7-xz-5.8.3.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/q335r4z6sia41v85hpj7a2mzkbvr2dgi-putchar-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/q3px40945ahyqcfk1jd5i0ysarphz9dv-mpfr-4.2.2.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/q75f3s0l812jqmqks1x90f35srn6ac2m-strcat-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/q8c5l3jqyqw4g02k0anhhxh1g4307ni5-mes-src-0.27.1-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/q9nj3bw34c6ag70kcss8zlbanrqs5maq-ioctl3.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ql0nssy1bhgyf83zq0r4lrksd6zsgh7z-gzip-get-version-1.2.4.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ql7jhf1wzhazxw4cw627zyls49xm5g3n-sed-4.9.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/qp9mn19nkyp680m55w409i550zni62gh-modechange.patch.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/qpzlh718p7p4nxhhwmcmjirqi0818rh6-mes_open.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/qsyqpi5hy1i3rzmg1x5l2nm0d3hcdmdc-hex2-1-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/qsz7cv59yzz5a9yqxbaridbr2nax5ms2-sigemptyset-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/qw9jid44nkp18dpan0wsymqqgmxabrwp-pkg-config-role-hook.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/qwsbaxq6i3scbf6zqliy6jq832jabfi1-_open3.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/qxz8v1gxrrw9j138k2ajl36vcajra8mb-isl-0.20.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/r1rf2p01za5ckcbz87fjvdnz2mpvy3bz-gmp-6.2.1.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/r1yi2iw61sqxbjs84649li9jwqcgbg77-isl-0.20.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/r2qcv2rkxs62fjy3n514p78mhqlyxflh-tinycc-mes-boot-libs-unstable-2025-12-03.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/r5yk7nin7qyfhnvd04pcvzmcwqadgrbs-mpc-1.4.0.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/r6l6qkdqrxlx9zn9mmiq6pxb7ym90l2y-gcc-15.2.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/r70i48xmaw176ydwjhwdya8kmh1b34xy-abtol-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/r78s609apr1sim2gafawm5az6va9gkn0-__buffered_read.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/r7yy2b087clhd9zrdksjnsdygxg0dnh2-_exit-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ra09d7l6r1ld4jk1xbl9rmii31g753bb-gnugrep-static-3.12.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ra17f5z1cxikjsywysyd63qcp5nlxa7a-libc.a-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/rc6rff9rwixd0ah70cynhkw6fl79whwq-libc.a.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/rd7pipwli0pfwc4qdygaxh8prq71izjh-bootstrap-stage1-stdenv-linux.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/rdi2zjmbw7565gc4pa4fkzvzpgwpaaj3-bash-5.3p9.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/rdw25dyfrm5nxp6kanr64z1v9lh3kx6k-reader.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/rhmr6f9rbhfl1fjblqcdmr1jannl6qhs-itoa.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/rivkjf3vnb267xl9xpjwj8wyzis4y0hs-bash-2.05b.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/rpixjnc530mwhkqspphhicl4svcq10w4-rename.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/rq77y1w9cbyx76dqcnw4wa24w1wq3ybw-M2-0.c-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/rqhmv2i8vxlz871j0y1aly194zfhfha9-acl-2.3.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/rr0issf5mmnpsfmqv861x5qixs79rwpg-stack-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/rrqrgx623i8lkf40gaw366d07m7vl3lw-tinycc-mes-unstable-2025-12-03-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/rsn67qwb0kfb9x4p051xy558gf407zgk-pkg-config-wrapper-0.29.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/rvcb8pzv5f8ax35ggc1hdk2vh5n6fq25-gzip-1.2.4.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/rwlh0lyww7if8gd94fzbnwh3917m9p5m-realloc.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/rxpmjxrz1xnr2pkamsa59afl929cj57k-hex2-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/s012w05rhh5sbhykm5b956n86clnpxk6-mes_open-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/s0xhkwp277nc4pb46k2s70npdnyc54s7-globals-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/s2g0la1392ys0bp34d0m38kmlzwi30v1-binutils-2.46.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/s3jjrqw6ay6nllpng4f1m1cffa11bhsb-tar-1.12.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/s47mlxqkjc26fvmx2yrps4hkpigimxi1-cc_arch-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/sa2if2mwj7kpyqd1kgjnx00ny3l2xivj-libc.a.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/salhf0y3pwji5rc43kgh51czb4ck6qgx-hex2_linker-0-0.hex2-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/sanzvc0fmx3nq26axi9925wk6rb5zmk1-mpfr-4.2.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/sb1sw1wqkaswx04a9pds57ms69gfi2ya-gnugrep-2.4.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/sb4ni1gpi595rakb4qlrbfzz6znk8p71-isnumber.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/scff6n77m9an0n7wzdm221b5diz0k4rj-kaem.hex2-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/sf7cx6m8hjb2f8m0fhq0d2lwndq61xbg-bash53-005.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/sflg6mqskxkfxbkq67nkbm38qg72n4lm-setup-hook.sh.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/sfxibi1q7pz850ah56lk91lr2rc6qciv-dtoab-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/siqp0bc5va5ng2k6kvvxhbxd87jlbcx5-waitpid.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/skfpkjqyfc0gn4skvrf2d3frr94i9idn-xz-5.8.3.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/sl1ai37193yxd9cddpz7w6a55h0kyinq-strtol-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/snijyhj9mfzbgmncy8ib3xy10zq22nmx-mkdir.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/sp7cjk6p7a8r8cvcczx78v63q5583b32-bzip2-get-version-1.0.8.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/sqpd1xdcz0wqscmky1yj2gqsl95mhd9l-gnum4-1.4.21.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/srsdkcf2f392mxfk2c9w5a0q2dz26jja-isatty-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/sv1z4x04x9khz94vfm1n7j9djgy60ghq-bash-get-version-5.3.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/sv664h9bg5xkpnqa3ia7mw7fsdwp8isa-gcc-simple-program-15.2.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/svc88737cf1py5bv6qnxpcp66jc1ngwz-symbol.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/sw7gwv37bczvhkynh6vcbi9q6i84y3a4-libtcc1.a.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/v0wvdpyrdxf6y8qb2flcq775zwc71hlh-syscall-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/v0y90g5sah7n98g296v65xlnwb00vbsq-mes-libmescc-0.27.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/v1v5w7szkp8l1yr379x9r4vyw7fkmmkl-tinycc-boot1-unstable-2024-07-07-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/v2pk6avh3x1k3k29hkja9shvw97954n1-tinycc-mes-boot-chain-unstable-2025-12-03-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/v375qxd9ls8bc1ah525l7yasy60xjijs-xz-5.8.3.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/v4wzp9vgcg10cypm21sxmmcf111jpba4-bootstrap-stage4-stdenv-linux.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/v520nngwnn1hyzyx6pjsd77lyas14v36-gnum4-1.4.21.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/v7yw14dv8az2q00ii6xpqysajbbagva2-write-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/v8jl1cziqai2qqgn0axr9cv3masm5nb5-bash-5.3.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/va81nfrd7p4n32xi0y3q9x8clsclsix8-gcc-10.4.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/vb8v1pvxr5piwksxsffwmdd5zz6s2j39-fcntl.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/vcjywcg8j3zy9i3may1p25scpwjsa441-exit.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/vcp6y915w85d7sbb4d27qfq5w5r8za5v-qsort.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/vd81g2231n9h6pm282j0fij7dj88pmh0-write.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/vda3dxp2568dwv487kb113qqmylby9g7-eval-apply-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/vdwm176pyf9v2p3shy9ck53084ymcjx3-libc.a-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/vf5nz05lk9n6xm1h1sh3c8s951ybk08h-musl-1.2.6.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/vgjfnqbxgxa8a5575bhq07nm35b2l31m-bison-3.8.2.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/vhaqzkl380y6x9vlfmy7kd39f0fypdmi-abtod.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/vjacq537970f9g0wh6bm999dwbpixnrc-gettimeofday.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/vlf0sxa4sgdzfm30fzszg9wvjnkg3q3p-vfprintf.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/vp409v6lb83p243hmv8wgdmiaj7ivq6h-gcc-15.2.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/vq5bcm0daiam2xn12mq03afqlnzj4qln-python-setup-hook.sh.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/vr3f3vnds0i0dpkkj69x510j7nrk65id-strtoll.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/vr93scipsw4jv37apjfhxy5nspn09y8i-binutils-with-gold-2.46.tar.bz2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/vrkgrz8l2z0yq45fai507hpcaqp0i9cq-touch-getdate.patch.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/vsbymyzgxr2pbsbdk5zfn74nb73njni5-grep-3.12.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/vsvi58gj6rmkdwyxb8iw7glcc6vh0dfc-link.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/vv6kfrbh9q0s1qj9f8xprx52d01xw5hj-crt.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/vvhsac436ggb1pg732mqkhw7s9jppl13-symbol-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/vvv7kxv3b7mairchnx5vniz4vcd1dksp-crt.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/vx2wmbx2yqrkdz267650gpnn4bndjq5g-execv.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/vx4j21115j102bdd8wf5yf0x6qbpzwar-gzip-static-1.14.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/vxj9mw4549sighgb4hv2k718kl2p70sk-buffered-read.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/vyw7h2czv5x2dn19g4gxd6k7djqq940a-mes-0.27.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/vzpav2rbdhq77ig3l6n2y0bnnpy7pazp-binutils-wrapper-2.46.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/w22wfi526ln8j9jfm87lamjjsdrqcdwa-isupper.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/w30lzfrj78qpp2lnmp6c3r5bqwjna34k-strupr.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/w324cws23p14nxdwqbmafpkykcsqg8r7-_open3-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/w4dgzqw1gnzy5irzw00hn25jbsqhqywf-gawk-static-5.3.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/w4vpchc8ps72xfmi6gbbrs6iw7y9kqjn-cast-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/w5008ccs8cadyjvgpx3bzk9d387ml2dw-tinycc-unstable-2025-12-03-source.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/w7skxq7bn25rvbxazn7prxvc8c5jjpjr-syscall-internal-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/w9fk7j4q1g8wls0iwj94iqllxrisnh0z-glibc-simple-program-2.42.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/w9ncwnrxfgg0mhq5bnv9i25ahx7zfm7a-eputs.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/w9q6kw3yb7az7wwzsvm97hqw7c8nrv0g-crt.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/wa6xxcihyrcglxac311wc59lxsfssh9x-common.mk.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/waf4kczw67cway08qchwrcz38fih4y63-tinycc-mes-boot-chain-unstable-2025-12-03.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/wbaljcz177vkkjn844nya51h6jrl49hi-gcc-15.2.0.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/wbq1wksxsc1gsgdkcan2h40l4x4gxx6w-lstat-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/wfpz7dbg98phhbiwfqc61fza4vidwcjy-gcc-cxx-4.6.4.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/whkczx14wnzmkchfys92zgas16rnzqw9-make-4.4.1.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/whm5pdc61pzf843jhlvy2r44x993xldj-libgetopt.a-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/winwi7l97nmy056ljjgi0pamca39d14k-findutils-4.10.0.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/wjfp3sba56m3mgbpssdwpg8gc4558q33-sscanf.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/wjmzjx1jzp5h8pmrcjvbs3hc8d5g8y4v-musl-1.2.6.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/wjqrmkhykih7qy97sgd5x3ba7d42wkx0-fwrite-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/wk1dy4n41iad1gk1zzfjsq2in2jw441j-variable.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/wk8l4607gjnfyv4v2bnh69gfcqisqlx1-fdgetc.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/wl53f1adanfm6qfdkh77xqsj5c8dkwsf-fdopen-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/wlp9qi7x32sldkpfrmjqd831hwwmqvc2-mkdir-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/wn4nf9pqapc89jz1lkvn7z3g6b9rnclv-gawk-mes-get-version-3.0.6.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/wsd61ackh8aswrw4slh85j1fxvqsfxch-binutils-2.46.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/wv752wrcldm19d604ls1xz1ibja7qacl-pipe.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/wvfnp9binadwclp8lyczahspfddz22q7-fflush-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/wvk16ysmscid85nmvb54ia8wzcws6b2z-opendir.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/wxbms5xmv39yyyqy337w0xylps100055-nyacc-1.09.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/wy41a6rmv90s4ryk8ri9b88y1rid9aan-strcmp-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/wyzbc8bi1ff5fyasvmq7w09yjlkd86b6-tinycc-boot1-unstable-2024-07-07.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/wza14brqwg5p63dvzlffcvqrgqd45f7f-gnumake-static-get-version-4.4.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/wzbw9g2rzl3a9rlw6gm3yvy1gshawg79-brk-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/wzc5vlgxwlx8xjs4v6jgw8xan080jxks-M1-0-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/wzhb336rh5z7c4fmp4fgf0hl6akn539r-xgcc-15.2.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/wzkj5hhzwm6ppkw8nyan97h6aw358av2-memcmp-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/x2dafg049vyrb1sm6jkydck0b1da2zab-coreutils-9.10.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/x44jj7p878adq57sn3w36gklljk4vlfl-expand-response-params.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/x5wwz0vih6vxdwpvyq5dc0w2ih64rklg-gnutar-get-version-1.12.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/x7l0vwljyrwyncsxkyvzh6gmc1f8p2rw-main.mk.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/x7s8n5cd93py6hxzx3hgga322ffcvd25-libmpc-1.4.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/x9k1p6h2g1lmlz11cpcjg8m2i4m6xr98-bash-static-get-version-5.3.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/x9sj5ai5qx2c6wmy5qz6dpl90g6rbrxg-blood-elf-0-0.hex2-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/xapdzxdhzxcbid65c0maaxvdygqg8y9w-python3-minimal-3.13.13.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/xawlq0m3mqzjw2n8zvxsrqnwx51cmnpl-pipe-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/xd10s7g0zy3m2kycbx6pnlmkx8wbd16g-memset.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/xfz5c7irzix5sakph3m18v0h400s6id6-uniq-fopen.patch.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/xi8dy016hq7842iplkgfdmzribiy8a3m-tinycc-bootstrappable-libs-unstable-2024-07-07.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/xjhir7vz7kb75jz8g66qa4b4ks4m2k9p-ltoa-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/xjvd53cvzbkfy823qvzl7nq1bh2w47zr-bootstrap-stage3-gcc-wrapper-15.2.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/xk5gc7f6j4h9hcx2z3in2cz12knwmj1a-oputc-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/xlhf3ibq6r6qy2myin330ay7qscfzi1g-gnused-4.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/xmifliaki5fkzvl1pw3qr6iv55krvpca-libc.a.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/xp4239xk0iiba8z35i58i9jk4wljs517-closedir.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/xsn3z4sqkn6ac7mlq35b62qs31mlnhbw-blood-elf-0-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/xwkyb3k04y5vx2kpb12gr7plqvcy67c3-update-autotools-gnu-config-scripts-hook.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/xxxl49qjg7gnwxfcr4sxfgpw4shq510g-tinycc-boot0-chain-unstable-2024-07-07-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/y3l2fjfs2fjy1ljsgq8b7f36sxn2777a-mes-libmescc-0.27.1-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/y3p27ax0p5slfgw496qc068jj7dbrn9g-musl-1.2.6.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/y4ni69pgbj6r3yvgzkqb5aly6i0vn8ym-vfprintf-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/y7wnv3kycy9gpxzwmny1sppi7il9i31a-gcc-wrapper-15.2.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/y84saga4qsmk7xrprd0qv8rgdqjk67g1-bash-5.3.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/y8xglficy4lqy8g4k0vfb0lbr4zzgk5x-_write.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/y98z4i8g33aic7nq3pi943abwfc16994-binutils-static-get-version-2.46.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/yakk49j0k9yrnfskv63r6iz8qb0qymh2-python-3.14.4.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/yawmf6sx8qa1nylbxxz01723ddsy5h41-fputs.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/yfyk44frrhkfjh07xsgiiqvvaahdsk16-tinycc-boot2-chain-unstable-2024-07-07-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/yglzrih7p6n7myi80gidsxv1v8f4v1ka-diffutils-3.8.tar.xz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/yh462hds2b7nzdqk3rrbivzqsjjq7zr5-crt-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/yhws1yr6pg2x51q0fabmzr99vl7dj5gp-libtcc1.a-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/yiv6sqyjk4mgai8fsbq0imh3vzca8ag0-musl-1.2.6.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/yjp7zwabw9xcm6g42736mbhlvcddw410-mescc-tools-extra-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/yk049frwz19wiswc9j1ssp5bbvarpazy-waitpid-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/yl1igz5ag7fwrk7cd7w1cdqr9z3z11dq-strtoull-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ymd4zqgir8lilx0q7l1djvgjp6v8cjyp-realpath-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ymxfm7hllmgkxfhpn62szs2gy9k025rf-libtcc1.a-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ynn3b7sb95w08pc639sr5m9n1s6m9lcc-_exit.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/yp4zsyn8fp1v5y8zpvp430lv339lvr7p-gcc-g++-4.6.4.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/yp67000wll97809fd49wjy5hbalpy8lw-tinycc-boot-mes-unstable-2024-07-07-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/yvwzkmzh716qqqj6qx331lmm18mpffva-tinycc-boot3-libs-unstable-2024-07-07-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/yvycxrsn9c2zmbkr9kvixpmazqlxzc7m-libgetopt.a.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ywwjm8pb0vv9dnz5mcqybzmznjz0ix79-ungetc-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/yx1ipzzcjya556sz0z3vc3gvgw86lb5c-cc_arch-1.hex2-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/yynym470zj9cl8d4n6m2zs6glypdrayy-gcc-simple-program-10.4.0.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/z1a6zj82v13glfvrfnhjkpic03y4kppc-bash-static-5.3.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/z1adhd9pfdiazma0ww8h5vds6j5a23vv-libgetopt.a.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/z4jpll2dilfbxm6ax4ymmhr67j3nbrvv-tinycc-boot-mes-libs-unstable-2024-07-07.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/z4x4wa4ahsc6xn40j847dsrnagxd41w0-gmp-6.3.0.tar.bz2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/z4xincpdf0q9ikjbma0s5sf5jrl3s187-M0-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/z5i7na9qpvfx6wx5phib8gwhqd1kyp99-fdputs-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/z6v5624dkfqg57qjzg6fadpxp07axkhv-libunistring-1.4.2.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/z82bg9br5cmdpwhpzf4qzjdag1calp2b-strtod-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zc4i6kv4525v20s2zh7fhxj1gcbcgggq-sigsetjmp.patch.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zdbga6f8g5jysvhngmnjlwmj674bzds2-gettext-1.0.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zf7smly23scq1jpgwy8lbv6vksd5si6b-M1-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zf8dvv680i9z2wpggik614r20n54hyii-hex0-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zfhd21zxiy48dijslxzcmsj1c2z2hnwv-coreutils-static-9.10.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zg19fm6fzvh5787swpwfrwj7gciidrj6-__assert_fail-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zgklqdh9majhk3nyivpl33ciflvsq5lc-krb5-1.22.1.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zhwaq1pghk6gqbjlcjgw3crf8ggrazay-M1-macro-0-footer.M1-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/ziimwk3g7vc4nji5pj5mbivmd0rw6zyh-hash.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zkwvl1yljz9gp7zbppzslzdhyq5zwg19-mes-0.27.1.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zlsfxdxsnzp1nzzw113avl2v0s5mgjpr-kaem-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zm0h1rwqw0jc93rhhsfzqnfw0lk6k3r5-symlink-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zmvpmmh7i1qba20c97v4cl6yi7zqpv3a-gnupatch-2.5.9.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zn15kp0ivyafi24ap2g4745g8cs9faki-dup.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/znjxpfyh2zjd2a56yigva27wqrzk0afs-pkg-config-0.29.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zp92khnxilvlvdha1wan9c8xh3ns7qsc-kaem.M1-1.9.1.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zpph8w4vm2h6nxfm2ghxx9jvjq5dn25i-lseek-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zpwqvr4bvp8hgc8dnix7xyyrb371dxh5-memchr.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zqqcsys6awnmp60kp9q0d7a6r4rzgh30-abtol.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zrr64l06h5bg96dr9s460kkwk3fjk8jv-reader-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zrx02p5fi8vxf7ys4sjyz6ggxz85w0kp-isatty.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zw2ds5k1knfgqzqlrv3wyvnh3za6nmf6-puts-builder.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zwb0f60mz9w9mjc07sqw8jkdx2xx8qri-linux-headers-6.18.7.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zwsm3s6dkvbbj74zg9x2zhigcrczy4js-bison-3.8.2.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zx5b0pppdvd1sja114g29zh80y1zn9f7-file-5.47.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zxr7znpy378mzc86kb3n15xpk4gw6xck-die-hook.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zy9kmif499i10lh2hqcpjmw0lgjv57wj-cast.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zzgcx00wj6bml7sdl4zzg1yy5zvzddl0-mpc-1.3.1.tar.gz.drv"
      },
      {
        "domain": "nix-compatible",
        "kind": "derivation-store-path",
        "algorithm": "nix-store-path-sha256",
        "value": "/nix/store/zzpghyckmljy4pzbmnj5vnbigmjv4a3d-write.drv"
      }
    ],
    "diagnostics": [],
    "non_claims": [
      "not-build-success",
      "not-package-correctness",
      "not-bootstrap-parity",
      "not-output-trust",
      "not-reproducibility",
      "not-foreign-frontend-availability"
    ]
  },
  "plan": {
    "schema": "mantle-foreign-derivation-adapter-plan-v1",
    "roots": [
      {
        "package_name": "hello",
        "node_id": "nix:m74651b793zgyyvlk9gx7v1cl1ywslib-hello-2.12.3.drv",
        "output_paths": {
          "out": "/mantle/store/d7b1dd721fa58aff09ec2a0faa79448c-hello-2.12.3"
        }
      }
    ],
    "source_payloads": [
      {
        "payload_id": "nix-source:033a5ba7793b344f4ccb5e661c493a70",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/8fhj67p4n5lcp9lkl19vdcrlcbl3jcsi-add-hardening.sh",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:03bf9220f420142bd1afba497a5729b7",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/xpplvxiwb4li2qd5nvhyd2mngrpna0ya-mangle-NIX_STORE-in-__FILE__.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:044383884d59e8fdc8a23eca829742ab",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/wpap61z19ch5drvyg814sl5acik6cjgr-0002-Remove-impure-dirs.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:06b59c0d79e2c517ce6183a774cd23bb",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/ny42y6hs4p294rvnrwbmrpwzqghw2816-gettext-setup-hook.sh",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:06c90bb3bc81edbda85b8f6b634030d0",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/sj0qllprnrmk1cqnnk57vvn2cqgjynbx-reenable_DT_HASH.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:07ee2a95355ba1d8d8e56273d76b5d7e",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/b1w7zbvm39ff1i52iyjggyvw2rdxz104-dont-use-system-ld-so-cache.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:098ef9ccbc3b459a06c36bb6ca5ef755",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/hklwj38xvcaf4zbify884y5dk8d1vq43-cc-wrapper.sh",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:0a080f154cd76b0c8a042f39eef4f02f",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/jik02mkz72r2f6hhxnlhp6h5f0fi89gw-expand-response-params.c",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:0ee5e97775b643abeadb800490a204c5",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/0mb1pqwqkz9sk1fwgn897zlwzhhgfpbf-0001-Revert-Remove-all-usage-of-BASH-or-BASH-in-installed.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:0f3c2b43ce31702e337149a99871fa1c",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/zi0m9pfmvy5lw89x7a8x674rm99i8qiq-setup-hook.sh",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:0fd3eeec9bb0c302b23fc1d5a92a6506",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/sq4h6bqjx12v9whvm65pjss25hg1538q-nix-ssl-cert-file.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:110becdc8ad946a1018165eba83bb133",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/lypyhrdqir7lhwhsvrr1cp85ywh3dhas-pkg-config-wrapper.sh",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:1300ae6a0077e70dd4123157a5df88c8",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/6djbn4px1wssdh72j51bjg3xkqaix8dl-hook.sh",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:17966f310e77db497b43ca8005901c7d",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/z7k98578dfzi6l3hsvbivzm7hfqlk0zc-set-source-date-epoch-to-latest.sh",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:1c71890b742ba144512c564418ae252d",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/lpw1q2ska8qrwyc4fnwfb0iliklcygrv-bzip2-1.0.6.2-autoconfiscated.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:227f93bde45154f49861de61be072b0e",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/h2fcbw7ghgn3i4qadszdp272w4dab7ln-lzip-setup-hook.sh",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:2282562b181c7abcc4dddd5e26a2f3d6",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/6b9v7v02npab086yaba2j4yfqrph5mgp-utils.bash",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:23377e0ccbf7faa175fe522c727bc45a",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/rf3kjgy7pbvymp55hxw28dg5g937lmcv-plugins-no-BINDIR.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:27e57b9ca1b808e6b673f4280481570d",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/yqwx9yln5i68nw61mmp9gz066yz3ri99-0001-msginit-Do-not-use-POT-Creation-Date.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:2909c32b2a6a962681a165adab37b2f5",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/rchrgid6xky2qmz58srsa0qn24bszxj3-setup-hook.sh",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:2b34666932c604a134a756f37e31293a",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/cickvswrvann041nqxb0rxilc46svw1n-prune-libtool-files.sh",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:2ca0eefe7af310fb6f0cb0bd5d409ceb",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/l622p70vy8k5sh7y5wizi5f2mic6ynpg-source-stdenv.sh",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:2d827fca2dda6a9db5f843515ed6ef5b",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/p2fp6i7hjx9af1wbwr32k217wp2dxmiw-absolute-paths.diff",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:2e399b7c92e93fe70cfe595b41b494e4",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/f4bvwqvj0y3z6blvh0knz71a8yq1c45p-requires-private.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:2f516ae7a120f34092fc5e448ee851b7",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/p3l1a5y7nllfyrjn2krlwgcc3z0cd3fq-make-symlinks-relative.sh",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:30d960619bf2a492e6fcf841f53e6628",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/6cc64ayl3fd2nc28ffw47cqsqi2bg1sn-0002-remove-impure-dirs.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:31b1bba360bd5c15b7f7e1801580f671",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/kxfa7n7k3clgw13yafj5irj51gk2w3dg-c++tools-dont-check-enable-default-pie.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:32b411d46de024d5ce3e40c0911fe95d",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/i58agvhx8bmdjlspgkk4m3ylh5348iim-use-etc-ssl-certs.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:367eaa29e241e2e8608a7012f9ca0d5a",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/m48r7nsm8wgjy69jg442vc7jjdfmcnl7-write-text-file.kaem",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:3698da5e18fc34afd57a25fd1928f3fc",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/rdqv4vr2jrjclnk2hdcrj97zqv5ny9z6-2.42-master.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:3b06d164f5e12b118d88072b7d230538",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/w13mdyf1a5qb5r6inx7y1fpfqpscrccv-disable-programs.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:3bbb064433eaa3f52e7289368c0df092",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/rrnck32i2fypb4jrdn279bnnnqh2pymg-cp-no-socket.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:3cea4cc9e1df1f2cc48755e72b5526c1",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/7bqlyrbj1r9ddcmm825g9zzhrgwxy9fw-no-system-headers.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:3eb06f82d8447e87b6eee8ebceee1ab9",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/ncd0iigkkkk9ziag5q02wj101kh27hvs-build-mkdir.kaem",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:41db4b0646fc65ac674ba9431b5d7f84",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/dm81j9qdcdr4c458pqbc9wvq9ymgzk4m-setup-hook.sh",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:47f18e113a5de217a7c0e639c589b1da",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/4y82jy7zkqgql0f83iw5mxgir1fbj8dn-sysconf.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:48300883969a795aedd73db1a3753793",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/kwyjpvvi26l18qn9zc9mhshcjpg7d9c9-ldexpl.c",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:4ef785fb49d5741ff0f405750117f98a",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/m9qvr6m0bylrjqb5ind6hfzsax14xys9-gnu-binutils-strip-wrapper.sh",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:4f1c679865a7ee9ea655062b863c449b",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/0df8rz15sp4ai6md99q5qy9lf0srji5z-0001-Revert-libtool.m4-fix-nm-BSD-flag-detection.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:4f2ca30fd79edb5266841c44c5c863ae",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/6xizqkp4bnhydwc7ihyqi93171gbj5n4-darwin-sdk-setup.bash",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:50769b5c9668063a540ccb5af4d21366",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/819fzxfwzp7zhhi4wy5nkapimkb1bsx5-die.sh",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:508971c847efb4d080ea78e75c56fba6",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/pag6l61paj1dc9sv15l7bm5c17xn5kyk-move-systemd-user-units.sh",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:525efff0e6b96c474753ae868dd71247",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/xyff06pkhki3qy1ls77w10s0v79c9il0-reproducible-builds.sh",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:52d11186e8fa7cff3f9a8356825b0410",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/br38n06v4i618krk7hc03g1vyndghhni-make-binary-wrapper.sh",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:535c843bf696f0e41e70e6977e85afdd",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/9bsy65vc7cqbphmsvkph5rpcsqly30kj-0003-tinycc-support.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:5419e56c59475e6b861cd58536cc5801",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/6dd4aj4q0n602yg42cqz46g76plb7wif-no-sys-dirs.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:56706c03a86c2cf648b26df486371021",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/4y1pjd5plswh5qmadjdyz04k0vync79i-meslibc-support.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:5853fbec5650850aff6dbcee69084d65",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/3gb2r5srli7xwmvs6mk8bwk374g247cs-musl-llvm.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:586ad574e0274ea629a4fcc087a257a3",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/d6jzf3bhlvbcm08m0d0fvqv1xw8s2vq7-windres-locate-gcc.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:589ac34b6aa1ca127c9f30032b95ef44",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/7kaj69537nv722pgrifzyic4a3wpi7nf-gcc15.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:5993cd05998fabf9697ed7505ef6372e",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/0rw1qmkghff7lf6lm75c6i0pi7jrm2jm-export-variable.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:5d2e393b44345ead0433fa9f4ebbabb6",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/9hvp9axa7zm732xi5jvg8pckvfk6ixcj-bash-builder.sh",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:5d548ecb0c534826289818ff70f0a5f4",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/cnss4bmvsa7kjmghgksgcadrxsvkyla1-builder.sh",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:5e8a7ed7560b0b1fc83a1f6a28aa32ba",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/4r8s8hcwyvvvnpcncps09zscqkh5qapx-no-install-statedir.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:5f0441358dca82f2e17dea77803df363",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/xsknmwfnbkh1m643isys7f61z93k5rp0-avr-size.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:60db52fd3fd7f2cad5f25c9e93b6613b",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/x8c40nfigps493a07sdr2pm5s9j1cdc0-patch-shebangs.sh",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:6213464926705f0c3b7f8cb1ba518fdb",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/awl9cirmdhg3la4fgcd9hs1fdn7ifqr0-0001-libtool.m4-update-macos-version-detection-block.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:6359a2a99b883259b09562ba61ba8cff",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/5yzw0vhkyszf2d179m0qfkgxmp5wjjx4-move-docs.sh",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:63e988293bfd380287ef29e0806fee13",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/ppm26x60zvaqzapqjnp540rislz3ml48-autoreconf.sh",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:671cc0334249b05345e072f1057eb904",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/xrw086zw3xqsvy9injgil8n2qdkvkpff-0001-Revert-libtool.m4-fix-the-NM-nm-over-here-B-option-w.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:69534b945db1f5e9641a682052b5db39",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/69lyjyca86317hdsc1rwf3ahn7s5kiyn-no-relocs.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:72d52953ba110da60750ac14468795a4",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/632b0y5mkcdwbsw2g3xh5qznw2vv5axr-ppc-musl.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:73508496287adaa0120d93070ed4f453",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/sqbhaaayam0xw3a3164ks1vvbrdhl9vq-deterministic.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:761edaeedc10b147910cd1336f678451",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/rrz4afh2wdl5l76jnrvldiss27hiycs9-setup.sh",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:774c5cad15af7ec6b5b62a78c2e5b107",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/a5c6xhw0v2z1yn20c58n2nw5adjiqy40-build-chmod.kaem",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:780544f6087e8bab2653bf5a0d3da036",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/73bwxvdmw63kgb1x27nqr7hh227nz04g-virtualenv-permissions.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:7d7125b5fc0b4e8b388ef25dbe71dc76",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/0y5xmdb7qfvimjwbq7ibg1xdgkgjwqng-no-broken-symlinks.sh",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:7ea30b4d589c3a163077f141406d99ef",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/mcgd0wa6sv13ns20f9vh8pp5x5n730ll-gnulib-float-h-tests-port-to-C23-PowerPC-GCC.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:807cb2c4240b546c5022e20762dafbf4",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/g2r1gl4ddyamsv8zvq1v1bnq7qw4j90b-0002-PR-725-inliniac-Revert-previous-and-always-set-offse.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:80b83f702c7b644cf789138feff82810",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/v9034cqc4h5bm10z4vz3n1q2n55grv5y-role.bash",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:811c22a4827ba45ca8dbe1a68e5c7ba5",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/q4f342g3s8zrq54ibk2ppgzlwz2j0cc5-termios.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:82e1f4c28f8495e4da9088989cea9ec1",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/329zjqfysghmg8sa9svgdd10vswbzcka-nuke-refs",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:85d77be2b7303a18bf5d340a12dace44",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/p3prlmdjdwlry48i4nmwkxp6qxdi28lm-pkg-config-static.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:8631e83199dd10cca1f4c0ec96d79b17",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/vn1172nqc21cxaljl9i431s772kncxnf-vprintf.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:87d2ef4d645f04818e49b4c18465b94e",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/jqin2kzvzrvi30cxda5zp7qz1fanmz7v-no-ldconfig.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:88c2b8ecda28a2ab660d915357572898",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/pai5pnd7z8b6mg4pgq44rqgx994ibpfh-0004-Fix-signatures-for-getenv-getopt.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:894b3e5a979f9b386753cfd3d1566702",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/saadgl075fyc62568ydfb0hsz6qfmgfy-proctab.c",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:89777e33782ec6e7f31c027b3d23aa4c",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/fzb1iqki8fw6k6vppjnyc0bqnqclssaq-build-cp.kaem",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:8a92ed0fa3583fe2701b40d1829704c4",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/9577hmdlmhki67cg8ar85cvidyg7xr7p-gcc-12-no-sys-dirs.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:8af950a68bc2dc6354f3eb3c4892dd33",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/pilsssjjdxvdphlg2h19p0bfx5q0jzkn-strip.sh",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:8b7d538b348e551d89092950390c6de8",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/prb84n7lxm49jic6sj1lndwd20pmgm01-build.kaem",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:8daead2527964dd4b097734a50bc8da1",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/7kw224hdyxd7115lrqh9a4dv2x8msq2s-fix-x64-abi.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:8ee7f9efcd3fb5eeb57da7d98edd23d2",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/p3wxvhph5yqqdjcv6bycpn3l9ydpgd51-main.mk",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:9431c0f1544f45f973cce196e08c8324",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/5xr8gnqhk4s0ygg85hykglmi28v28l84-0003-Do-not-search-for-a-C-compiler-and-set-MAKE_CXX.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:95530fcfdc12469c473a1685f62df588",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/7x6bimj6ipi6ag859gi2fc6by87x37j7-no-sys-dirs-riscv.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:956c1b99dfb42a5323816027565fefc8",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/v7ihfx32zv7bdha6i9dd6a3r0knzs8j3-setup-hook.sh",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:997744b25363847bc92a2312a2ce8a46",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/nj51fd0zhgfbg6fwiv9lalcfyw63q3l2-strcoll.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:9b543d0b5194ef218bbcf7956127a7fd",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/fyaryjvghbkpfnsyw97hb3lyb37s1pd6-move-lib64.sh",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:9c759abd036ee83e7ab887b5be4fa03e",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/f1qmmr4f6mi99xr5bvfrqmynjx9j038p-0001-localedata-allow-reproducible-parallel-install-of-lo.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:9d0a68588740601287c67cb9b706b9c9",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/caakpwmdf9gpgdinaqrikc7vr8y62mf6-static-link.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:a045f5185d5af233f3c67c5029e844ad",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/1bs8wn3nwiji4vcwfzxpncwsa1pnh404-fix-tinycc-attribute.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:a1a54a0ec36035ed594775e6c0ff8ef1",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/96rvfw5vlv1hwwm9sdxhdkkpjyym6p2x-update-autotools-gnu-config-scripts.sh",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:a20629b39fc7e8d741526beca3a04aa1",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/shkw4qm9qcw5sc5n1k5jznc83ny02r39-default-builder.sh",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:a376accf4c1d134b224683ef3ea79fec",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/lp2nnib5zm4vl8ydz81yns04vzyxw926-add-hardening.sh",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:a3984a3f3a4d283b660e38e2a5bd74a0",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/88a62ypvi5xpa3m8znhrl6l7jc307i8r-conf-symlink.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:a415ef1ebcae518bc4299b4a9eb071a0",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/cmzya9irvxzlkh7lfy6i82gbp0saxqj3-multiple-outputs.sh",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:a56934b7448b81447c5256f58ed47529",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/paaqz9ad8p8ravgs8hw3kzypqc3fwqws-ld-wrapper.sh",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:a7d2fa6d24604d6cba35d69c4c3dbe8f",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/fnx1c1gsrqi4dq0pfng6niaw3f3x5p6j-gnulib-float-h-tests-port-to-C23-PowerPC-GCC.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:a86f90e1f0df9a00cdbbfdbb34089b81",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/iygnfrzfx9bvqk4rdqjkvrv9kxqckmp3-musl.h",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:a89add6660249873c9ad54ffb764d53d",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/85clx3b0xkdf58jn161iy80y5223ilbi-compress-man-pages.sh",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:a8f07236a63493a3fb382621cafb6e36",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/4cmjzk8yr6i5vls5d2050p653zzdvmvp-setup-hook.sh",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:a927a92d81aa0251e5dacebe835dea22",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/x7zciwy1fy9v9yc5y5lk5iycfwglkd5d-add-flags.sh",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:ab1fd29d15333101839813c848ab7cea",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/kqi6x4rd1a8q3dm1w6r03xy3czsiq1r6-config_tccdefs.h",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:b17322f1c6430a1ad505430db47eddea",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/cbkpzyi933j5g1fx93x0g3b3h12z0lv8-ln.c",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:b30268a6137f294e7f75cd1eed93b16f",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/i3jn2ygdzlj9dj3ypxn9my64dy3ykzhi-setjmp_x86_64.c",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:b377d2d895eeae3aae5aa8759890703d",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/h390waw61rbnzpx9fbnyvvmjy2fgnyjv-stubs.h",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:b4f3dbea402e0556efd15d5c471ba6ab",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/3cj8qm3xjz1g5f7qnnxlc95i9imkicbk-cfi_startproc-reorder-label-14-1.diff",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:b8621395790e2917bede188def239e89",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/cv1d7p48379km6a85h4zp6kr86brh32q-audit-tmpdir.sh",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:bcbaf827e78decdf5cf1d1ff2872dec2",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/2k52bklbjhhq47dn35gm833vlh06fgfn-0001-No-impure-bin-sh.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:bcdbd8eec1d6448c8480a27fea709fab",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/c4akajrb4jg50k72jw7zfbyv8z139ri0-setup-hook.sh",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:be0bd3d2b5f48ebe8e7102fa292439df",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/cklrwbwi889pp2fdsswdjvn12sdy5i5j-openssl-disable-kernel-detection.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:bfbb51809b7c12cc4bb3974f2b807055",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/n5kkahb82ykkcjwakgx2h6jbs7d9avji-gcc-15.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:c1b36d860e46ebedc8de69dfa5fccd6e",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/8k5y1zg5cg3y2pjn9fh8l5ln28rq6gdy-aarch64-sve-rtx.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:c41170b2508319e389e22bd6a1bdc611",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/znnnjr9ym1v5am6adg4x3pl0l6zx6xqk-setup-hook.sh",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:c895a46c51cc4fc35a2a3e8e1bba0fa3",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/vvlgjnnch92k53af32iwim3d7jdxai3y-builder.sh",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:ca9b008d8af21343f8afc2c73e81a585",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/0vvf7m9s6ng3cspkxx46saffdyyvx2ff-langinfo.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:cad4a168a5181b49180ef76e42f755de",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/s5myj3ybdn7wrsfpydj48mm24vnvw6bc-no-ldconfig.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:ce45b2ba6ace354c60733c3ff418e416",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/wc9c8k7z5w5afdqvz7gjrjlaqf16wfpr-musl.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:d0360862be8243ed5e8b00424a40fadd",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/01gahr13mk3p411jhgxk7ab3pnl0c9yv-0001-No-impure-bin-sh.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:d07adf0c41af9d442a992ae2cd7bc896",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/z0sjg5c7g1iqb4x5vyiqcm5n68mb5x5p-0001-Remove-unused-function-after_eq.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:d218f366cb2d7456b96309d5762780df",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/mrzpfh0ml9k07sw019ydagbb2z1q4sxz-add-flags.sh",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:d35290d2961302db43af0f4ae75dc948",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/yq0lz1byj4v2rym2ng23a3nj4n6pvqdj-pgrp-pipe-5.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:d71900710d9968d5e6ac5a8e6225268f",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/pa83jbilxjpv5d4f62l3as4wg2fri7r7-always-search-rpath.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:dbf8d7a08665a98a183268f90c73a88d",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/62qh08ij1zvw5lxj69n98lzv0byda0di-0002-Makeconfig-make-inst_complocaledir-overridable.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:dc8d8fb5f5eb5681c2b9a8143ae0ff3f",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/c5j9vs12r91gg1vm7yzsjq8wf75yx84x-config.h",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:dcb79d2fba117aedf2a6c77c80ec4c05",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/kd4xwxjpjxi71jkm6ka0np72if9rm3y0-move-sbin.sh",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:de49cec57de4b6a38086c2bf42f0b337",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/m9171q7kn5bbp08hqmhfra41mmn6p5nk-CVE-2026-7598.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:df7114d58e36aae6a49849a7732473ef",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/r989dk196nl9frhnfsa1lb7knhbyjxw6-separate-debug-info.sh",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:e02df100c18db980bbd13c90ee7f1176",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/4ychsxk19vnw9q0dg80bwrswgs2a5780-proctab.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:e0ed760e6a5ce4bc531eeab02062c4b5",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/vzs1lypq3dicxrr91h4qkag9f2cqwxzj-dont-link-lm.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:e1ce177b6ffe4765f8b5c2c496291359",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/4x1l3vi4j5045bmmdww4ra1g6v6mmwkl-make-wrapper.sh",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:e26cf87ddbf5a21e4ed5d7ccbb9adfa5",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/2l64cqpqcrb92x2f4lnl0k0gg0hv52a3-tcc-empty-ar.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:e44e729bbb1766ac22e4f50157a29bb2",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/llr530f1l6c470xkl2w80k2kkfqddmpm-fix_path_attribute_in_getconf.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:e4c7151c372bf59d7953373cc6719834",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/1ksmnsr3m6paw8gs7jp9b623agzdrqi2-add-flags.sh",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:e56ba8eb0530fa3ce82115d372541614",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/mnglr8rr7nl444h7p50ysyq8qd0fm1lm-dont-use-system-ld-so-preload.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:e68cce2ab4297aca7dba78824eb75a29",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/87srq5q8fbg306rs8x6mmaibyyxqn3pn-0005-Fix-signatures-for-getenv-getopt.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:e81b8daceb859502133ee077563abca9",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/sx3mhm7h4dhqa4p3avrckcda69yg0fid-write-mirror-list.sh",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:ea13479e7471652ecd2809ff3232627f",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/c97lni7llcvf2fgyang2iq7csaykyxrj-0001-PR-745-streamout-Don-t-flush-when-trying-to-set-nega.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:ea2b933acd1ec9d343b45c961ffea7fe",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/dxq2nj9xx3fvcaypniaxwzfya1chng7b-build.kaem",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:ebc3dfb3c2fce8ff84285343bdfe9731",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/5fs57gxp6iw4h8hbdn4g3cym0l452bzl-no-stamp.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:f0296109fbf957931201b5d4d779f19c",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/1py1bqrkcp6sipaf3znbprrsc2ac23wi-gnulib-float-h-tests-port-to-C23-PowerPC-GCC.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:f203937d53b975520a2714ff85758bec",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/70acbfgbz5cxsx4sqzcnj8qaqhb4i9ay-bash-builder.sh",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:f5655a9837cbc13232382d83a8739d1f",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/c9wqj29c0ywhx6s9pbm00khjskhrskns-fix-symver-on-non-elf.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:fa3029154ca7656479dd5e211bfd9793",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/9hv7fq2akyqckgzqa9wv02f2dsd398dv-utime.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:fa5b485bb4aeee9e306e1a42df0576d7",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/nc5hqqizlj7042vvhvx121kzf4nwh35p-nix-locale-archive.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:fb007e8325edfc107b88cb5e48c0a5a5",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/001gp43bjqzx60cg345n2slzg7131za8-nix-nss-open-files.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:fc251b8846948c1b6baf4719a78cd734",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/y3lgm711bp2dlrfqwpb0haiph0hkbg01-mksignames-flush.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:fdfe446384b5b6cec39c565cc3e85aa6",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/j7bd2a4fafw9rchrf1ag9xf920g37kk7-link-libiconv.patch",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:fdff3b3ef072f00e3562a81f5cb32286",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/zxvdc6f4clrv2dsx5k6cvrrkzk4cf0zn-build-replace.kaem",
        "embedded_text": null,
        "mirrors": []
      },
      {
        "payload_id": "nix-source:ffac27c9914d005ebe46e1565fb22612",
        "kind": "nix-input-source",
        "content_ref": "/mantle/store/sc7r0g8x9qhyr5y4yhm9x2h3ayajs94w-kaem-wrapper.kaem",
        "embedded_text": null,
        "mirrors": []
      }
    ],
    "sandbox_audit": [],
    "substitution_audit": [
      {
        "node_id": "nix:m74651b793zgyyvlk9gx7v1cl1ywslib-hello-2.12.3.drv",
        "cache_url": "https://cache.nixos.org",
        "trust_scope": "trusted-binary-cache",
        "classification": "cache-hint-policy-data-store-admission-required",
        "store_admission_required": true
      }
    ],
    "forbidden_process_invocations": [],
    "non_claims": [
      "not-build-success",
      "not-package-correctness",
      "not-bootstrap-parity",
      "not-output-trust",
      "not-reproducibility",
      "not-foreign-frontend-availability"
    ]
  },
  "non_claims": [
    "not-build-success",
    "not-package-correctness",
    "not-bootstrap-parity",
    "not-output-trust",
    "not-reproducibility",
    "not-foreign-frontend-availability"
  ]
}

```

## Scoped claim

Strongest proven state: admitted/planned. Non-claims: no real cache substitution, no local rebuild compatibility, no output trust, no package correctness, no reproducibility.
