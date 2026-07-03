# stabilize-native-rust-plan-validation evidence

## Host environment

```text
PATH=/home/brittonr/.cargo/bin:/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:/nix/store/wci2b3l9gs8nq3alx6czffsq55bg44cv-fd-10.4.2/bin:/nix/store/gxcm97c2a4sdnnabigp8mns7y1g5llnd-ripgrep-15.1.0/bin:/home/brittonr/.local/bin:/run/wrappers/bin:/etc/profiles/per-user/brittonr/bin:/run/current-system/sw/bin
PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig
SNIX_BUILD_SANDBOX_SHELL=/nix/store/d7fc5i7y71rj8cr5jwmaxwjnyvfiybdp-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox
CARGO_TARGET_DIR=/tmp/mantle-rust-plan-validation-target
```

## Focused serial rail

```text
$ cargo test -p mantle --bin mantle rust_plan::tests:: -- --test-threads=1
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling proc-macro2 v1.0.106
   Compiling unicode-ident v1.0.24
   Compiling quote v1.0.45
   Compiling serde_core v1.0.228
   Compiling cfg-if v1.0.4
   Compiling serde v1.0.228
   Compiling memchr v2.8.0
   Compiling libc v0.2.186
   Compiling regex-syntax v0.8.10
   Compiling smallvec v1.15.1
   Compiling scopeguard v1.2.0
   Compiling parking_lot_core v0.9.12
   Compiling equivalent v1.0.2
   Compiling version_check v0.9.5
   Compiling itoa v1.0.18
   Compiling once_cell v1.21.4
   Compiling typenum v1.20.1
   Compiling allocator-api2 v0.2.21
   Compiling subtle v2.6.1
   Compiling thiserror v2.0.18
   Compiling find-msvc-tools v0.1.9
   Compiling lock_api v0.4.14
   Compiling shlex v1.3.0
   Compiling crossbeam-utils v0.8.21
   Compiling generic-array v0.14.7
   Compiling stable_deref_trait v1.2.1
   Compiling bitflags v2.11.0
   Compiling foldhash v0.2.0
   Compiling same-file v1.0.6
   Compiling fastrand v2.4.1
   Compiling crc32fast v1.5.0
   Compiling aho-corasick v1.1.4
   Compiling tinyvec_macros v0.1.1
   Compiling gix-trace v0.1.18
   Compiling walkdir v2.5.0
   Compiling cpufeatures v0.2.17
   Compiling hashbrown v0.16.1
   Compiling tinyvec v1.11.0
   Compiling byteorder v1.5.0
   Compiling pin-project-lite v0.2.17
   Compiling fnv v1.0.7
   Compiling bytesize v2.3.1
   Compiling unicode-normalization v0.1.25
   Compiling human_format v1.2.1
   Compiling zlib-rs v0.6.3
   Compiling futures-core v0.3.32
   Compiling winnow v0.7.15
   Compiling heapless v0.8.0
   Compiling hash32 v0.3.1
   Compiling rustix v1.1.4
   Compiling regex-automata v0.4.14
   Compiling zmij v1.0.21
   Compiling linux-raw-sys v0.12.1
   Compiling serde_json v1.0.149
   Compiling jiff v0.2.23
   Compiling rand_core v0.10.0
   Compiling getrandom v0.4.2
   Compiling futures-io v0.3.32
   Compiling futures-sink v0.3.32
   Compiling slab v0.4.12
   Compiling futures-channel v0.3.32
   Compiling futures-task v0.3.32
   Compiling rustversion v1.0.22
   Compiling zeroize v1.8.2
   Compiling strsim v0.11.1
   Compiling fs_extra v1.3.0
   Compiling foldhash v0.1.5
   Compiling semver v1.0.28
   Compiling dunce v1.0.5
   Compiling log v0.4.29
   Compiling hashbrown v0.15.5
   Compiling tracing-core v0.1.36
   Compiling rustc_version v0.4.1
   Compiling either v1.15.0
   Compiling arrayvec v0.7.6
   Compiling percent-encoding v2.3.2
   Compiling aws-lc-rs v1.16.2
   Compiling rustls-pki-types v1.14.0
   Compiling encoding_rs v0.8.35
   Compiling untrusted v0.9.0
   Compiling signal-hook v0.4.4
   Compiling writeable v0.6.3
   Compiling httparse v1.10.1
   Compiling litemap v0.8.2
   Compiling pkg-config v0.3.32
   Compiling base64 v0.22.1
   Compiling ident_case v1.0.1
   Compiling rustls v0.23.37
   Compiling crossbeam-channel v0.5.15
   Compiling static_assertions v1.1.0
   Compiling utf8_iter v1.0.4
   Compiling icu_normalizer_data v2.2.0
   Compiling nonempty v0.12.0
   Compiling hashbrown v0.14.5
   Compiling block-buffer v0.10.4
   Compiling crypto-common v0.1.7
   Compiling icu_properties_data v2.2.0
   Compiling unicode-xid v0.2.6
   Compiling autocfg v1.5.0
   Compiling indexmap v2.13.1
   Compiling lazy_static v1.5.0
   Compiling atomic-waker v1.1.2
   Compiling digest v0.10.7
   Compiling rayon-core v1.13.0
   Compiling try-lock v0.2.5
   Compiling unicode-width v0.2.2
   Compiling tower-service v0.3.3
   Compiling ryu v1.0.23
   Compiling faster-hex v0.10.0
   Compiling crossbeam-epoch v0.9.18
   Compiling cfg_aliases v0.2.1
   Compiling want v0.3.1
   Compiling sha1 v0.10.6
   Compiling form_urlencoded v1.2.2
   Compiling syn v2.0.117
   Compiling sync_wrapper v1.0.2
   Compiling ipnet v2.12.0
   Compiling unicode-bom v2.0.3
   Compiling openssl-probe v0.2.1
   Compiling tower-layer v0.3.3
   Compiling crossbeam-deque v0.8.6
   Compiling constant_time_eq v0.3.1
   Compiling arrayref v0.3.9
   Compiling iri-string v0.7.12
   Compiling kstring v2.0.2
   Compiling jobserver v0.1.34
   Compiling sha1-checked v0.10.0
   Compiling mime v0.3.17
   Compiling unicode-segmentation v1.13.2
   Compiling shell-words v1.1.1
   Compiling rustls-native-certs v0.8.3
   Compiling colorchoice v1.0.5
   Compiling num-traits v0.2.19
   Compiling anstyle v1.0.14
   Compiling portable-atomic v1.13.1
   Compiling utf8parse v0.2.2
   Compiling cc v1.2.59
   Compiling data-encoding v2.10.0
   Compiling heck v0.5.0
   Compiling libm v0.2.16
   Compiling anyhow v1.0.102
   Compiling is_terminal_polyfill v1.70.2
   Compiling siphasher v1.0.2
   Compiling anstyle-parse v1.0.0
   Compiling anstyle-query v1.1.5
   Compiling errno v0.3.14
   Compiling socket2 v0.6.3
   Compiling mio v1.2.0
   Compiling getrandom v0.2.17
   Compiling cmake v0.1.58
   Compiling memmap2 v0.9.10
   Compiling ring v0.17.14
   Compiling blake3 v1.8.2
   Compiling signal-hook-registry v1.4.8
   Compiling zstd-sys v2.0.16+zstd.1.5.7
   Compiling parking_lot v0.12.5
   Compiling dashmap v6.1.0
   Compiling filetime v0.2.27
   Compiling aws-lc-sys v0.39.1
   Compiling winnow v1.0.1
   Compiling anstream v1.0.0
   Compiling prodash v31.0.0
   Compiling lzma-sys v0.1.20
   Compiling bzip2-sys v0.1.13+1.0.8
   Compiling tempfile v3.27.0
   Compiling libmimalloc-sys v0.1.44
   Compiling phf_shared v0.11.3
   Compiling gix-sec v0.13.2
   Compiling convert_case v0.10.0
   Compiling sha2 v0.10.9
   Compiling memoffset v0.6.5
   Compiling vte v0.15.0
   Compiling regex v1.12.3
   Compiling toml_parser v1.1.2+spec-1.1.0
   Compiling serde_derive v1.0.228
   Compiling thiserror-impl v2.0.18
   Compiling tokio-macros v2.7.0
   Compiling synstructure v0.13.2
   Compiling futures-macro v0.3.32
   Compiling zerovec-derive v0.11.3
   Compiling displaydoc v0.2.5
   Compiling tracing-attributes v0.1.31
   Compiling darling_core v0.23.0
   Compiling zerofrom-derive v0.1.7
   Compiling yoke-derive v0.8.2
   Compiling async-trait v0.1.89
   Compiling keccak v0.1.6
   Compiling zstd-safe v7.2.4
   Compiling zerofrom v0.1.7
   Compiling new_debug_unreachable v1.0.6
   Compiling toml_datetime v1.1.1+spec-1.1.0
   Compiling precomputed-hash v0.1.1
   Compiling fixedbitset v0.5.7
   Compiling futures-util v0.3.32
   Compiling tracing v0.1.44
   Compiling clap_lex v1.1.0
   Compiling rustix v0.38.44
   Compiling yoke v0.8.2
   Compiling term v1.2.1
   Compiling typeid v1.0.3
   Compiling bytemuck v1.25.0
   Compiling bit-vec v0.8.0
   Compiling petgraph v0.7.1
   Compiling ena v0.14.4
   Compiling zerovec v0.11.6
   Compiling zerotrie v0.2.4
   Compiling ascii-canvas v4.0.0
   Compiling darling_macro v0.23.0
   Compiling clap_builder v4.6.0
   Compiling bit-set v0.8.0
   Compiling safe_arch v0.7.4
   Compiling string_cache v0.8.9
   Compiling toml_edit v0.25.10+spec-1.1.0
   Compiling sha3 v0.10.8
   Compiling darling v0.23.0
   Compiling tinystr v0.8.3
   Compiling potential_utf v0.1.5
   Compiling lalrpop-util v0.22.2
   Compiling serde_with_macros v3.18.0
   Compiling icu_collections v2.2.0
   Compiling icu_locale_core v2.2.0
   Compiling vt100 v0.16.2
   Compiling derive_more-impl v2.1.1
   Compiling clap_derive v4.6.0
   Compiling pin-project-internal v1.1.11
   Compiling arc-swap v1.9.1
   Compiling console v0.16.3
   Compiling n0-future v0.3.2
   Compiling nix v0.31.3
   Compiling sharded-slab v0.1.7
   Compiling icu_provider v2.2.0
   Compiling tracing-log v0.2.0
   Compiling itertools v0.14.0
   Compiling heapless v0.7.17
   Compiling curve25519-dalek v4.1.3
   Compiling icu_properties v2.2.0
   Compiling icu_normalizer v2.2.0
   Compiling matchers v0.2.0
   Compiling thread_local v1.1.9
   Compiling cpufeatures v0.3.0
   Compiling futures-executor v0.3.32
   Compiling thiserror v1.0.69
   Compiling cordyceps v0.3.4
   Compiling spin v0.10.0
   Compiling simd-adler32 v0.3.9
   Compiling erased-serde v0.4.10
   Compiling malachite-nz v0.6.1
   Compiling diatomic-waker v0.2.3
   Compiling pico-args v0.5.0
   Compiling nu-ansi-term v0.50.3
   Compiling unit-prefix v0.5.2
   Compiling parking v2.2.1
   Compiling bitflags v1.3.2
   Compiling linux-raw-sys v0.4.15
   Compiling adler2 v2.0.1
   Compiling futures-buffered v0.2.13
   Compiling lalrpop v0.22.2
   Compiling futures-lite v2.6.1
   Compiling tracing-subscriber v0.3.23
   Compiling indicatif v0.18.4
   Compiling malachite-base v0.6.1
   Compiling miniz_oxide v0.8.9
   Compiling idna_adapter v1.2.1
   Compiling idna v1.1.0
   Compiling futures v0.3.32
   Compiling chacha20 v0.10.0
   Compiling clap v4.6.0
   Compiling pin-project v1.1.11
   Compiling derive_more v2.1.1
   Compiling wide v0.7.33
   Compiling proc-macro-crate v3.5.0
   Compiling thiserror-impl v1.0.69
   Compiling spez v0.1.2
   Compiling n0-error-macros v0.1.3
   Compiling async-stream-impl v0.3.6
   Compiling curve25519-dalek-derive v0.1.1
   Compiling darling_core v0.20.11
   Compiling xattr v1.6.1
   Compiling logos-codegen v0.15.1
   Compiling hash32 v0.2.1
   Compiling spin v0.9.8
   Compiling proc-macro2-diagnostics v0.10.1
   Compiling nibble_vec v0.1.0
   Compiling fuse-backend-rs v0.12.0 (/home/brittonr/git/mantle/vendor/fuse-backend-rs)
   Compiling redb v3.1.3
   Compiling matchit v0.8.4
   Compiling iana-time-zone v0.1.65
   Compiling endian-type v0.1.2
   Compiling yansi v1.0.1
   Compiling signature v2.2.0
   Compiling mimalloc v0.1.48
   Compiling radix_trie v0.2.1
   Compiling n0-error v0.1.3
   Compiling ed25519 v2.2.3
   Compiling async-stream v0.3.6
   Compiling num_enum_derive v0.7.6
   Compiling nix v0.24.3
   Compiling clap-verbosity-flag v3.0.4
   Compiling rand v0.10.1
   Compiling tracing-indicatif v0.3.14
   Compiling flate2 v1.1.9
   Compiling darling_macro v0.20.11
   Compiling vmm-sys-util v0.11.2
   Compiling cobs v0.3.0
   Compiling irpc-derive v0.10.0
   Compiling num_cpus v1.17.0
   Compiling vm-memory v0.10.0
   Compiling caps v0.5.6
   Compiling mio v0.8.11
   Compiling md-5 v0.10.6
   Compiling itertools v0.15.0
   Compiling rustc-hash v2.1.2
   Compiling fixedbitset v0.4.2
   Compiling nix-compat v0.1.0 (/home/brittonr/git/mantle/vendor/nix-compat)
   Compiling humantime v2.3.0
   Compiling snix-castore v0.1.0 (/home/brittonr/git/mantle/vendor/snix-castore)
   Compiling beef v0.5.2
   Compiling ed25519-dalek v2.2.0
   Compiling threadpool v1.8.1
   Compiling darling v0.20.11
   Compiling num_enum v0.7.6
   Compiling auto_impl v1.3.0
   Compiling nix-compat-derive v0.1.0 (/home/brittonr/git/mantle/vendor/nix-compat-derive)
   Compiling nom v8.0.0
   Compiling bitmaps v3.2.1
   Compiling termcolor v1.4.1
   Compiling zerocopy v0.8.48
   Compiling heck v0.4.1
   Compiling wu-manber v0.1.0 (https://github.com/tvlfyi/wu-manber.git#0d5b22be)
   Compiling toml_writer v1.1.1+spec-1.1.0
   Compiling derive_builder_core v0.20.2
   Compiling nickel-lang-parser v0.1.1
   Compiling proc-macro-error-attr2 v2.0.0
   Compiling imbl-sized-chunks v0.1.3
   Compiling hashlink v0.10.0
   Compiling logos-derive v0.15.1
   Compiling ouroboros_macro v0.18.5
   Compiling imara-diff v0.1.8
   Compiling imara-diff v0.2.0
   Compiling bzip2 v0.5.2
   Compiling typed-arena v2.0.2
   Compiling paste v1.0.15
   Compiling aliasable v0.1.3
   Compiling arraydeque v0.5.1
   Compiling arrayvec v0.5.2
   Compiling snix-store v0.1.0 (/home/brittonr/git/mantle/vendor/snix-store)
   Compiling derive_builder_macro v0.20.2
   Compiling logos v0.15.1
   Compiling proc-macro-error2 v2.0.1
   Compiling saphyr-parser v0.0.6
   Compiling pretty v0.12.5
   Compiling nickel-lang-core v0.16.1
   Compiling nix v0.29.0
   Compiling uluru v3.1.0
   Compiling lru v0.16.4
   Compiling clru v0.6.3
   Compiling vte v0.14.1
   Compiling unsafe-libyaml v0.2.11
   Compiling ouroboros v0.18.5
   Compiling simple-counter v0.1.0
   Compiling bumpalo v3.20.2
   Compiling count-write v0.1.0
   Compiling strip-ansi-escapes v0.2.1
   Compiling getset v0.1.6
   Compiling derive_builder v0.20.2
   Compiling typed-builder-macro v0.22.0
   Compiling strum_macros v0.26.4
   Compiling maybe-async v0.2.10
   Compiling rand_core v0.6.4
   Compiling fs2 v0.4.3
   Compiling io-close v0.3.7
   Compiling sha-1 v0.10.1
   Compiling json_scanner v0.1.0
   Compiling indoc v2.0.7
   Compiling crc-catalog v2.4.0
   Compiling snix-build v0.1.0 (/home/brittonr/git/mantle/vendor/snix-build)
   Compiling strum v0.26.3
   Compiling typed-builder v0.22.0
   Compiling uuid v1.23.0
   Compiling webpki-roots v1.0.6
   Compiling crc v3.4.0
   Compiling itertools v0.12.1
   Compiling utf8-zero v0.8.1
   Compiling twox-hash v2.1.2
   Compiling gethostname v0.5.0
   Compiling tar v0.4.45
   Compiling lzma-rs v0.3.0
   Compiling ruzstd v0.8.2
   Compiling bzip2-rs v0.1.2
   Compiling toml_write v0.1.2
   Compiling predicates-core v1.0.10
   Compiling float-cmp v0.10.0
   Compiling normalize-line-endings v0.3.0
   Compiling assert_cmd v2.2.0
   Compiling serde_with v3.18.0
   Compiling serde_bytes v0.11.19
   Compiling toml_datetime v0.7.5+spec-1.1.0
   Compiling serde_spanned v1.1.1
   Compiling termtree v0.5.1
   Compiling toml v0.9.12+spec-1.1.0
   Compiling difflib v0.4.0
   Compiling predicates v3.1.4
   Compiling predicates-tree v1.0.13
   Compiling wait-timeout v0.2.1
   Compiling diff v0.1.13
   Compiling crunch-delta-core v0.1.0 (/home/brittonr/git/mantle/crates/crunch-delta-core)
   Compiling ppv-lite86 v0.2.21
   Compiling pretty_assertions v1.4.1
   Compiling bytes v1.11.1
   Compiling bstr v1.12.1
   Compiling url v2.5.8
   Compiling serde_urlencoded v0.7.1
   Compiling quick-xml v0.40.1
   Compiling chrono v0.4.44
   Compiling serde_tagged v0.3.0
   Compiling serde_qs v0.12.0
   Compiling petgraph v0.6.5
   Compiling codespan-reporting v0.13.1
   Compiling toml_edit v0.23.10+spec-1.0.0
   Compiling tokio v1.51.0
   Compiling http v1.4.0
   Compiling crunch-attestation-core v0.1.0 (/home/brittonr/git/mantle/crates/crunch-attestation-core)
   Compiling gix-validate v0.11.0
   Compiling gix-utils v0.3.1
   Compiling gix-error v0.2.1
   Compiling gix-packetline v0.21.2
   Compiling postcard v1.1.3
   Compiling codespan v0.13.1
   Compiling gix-path v0.11.2
   Compiling gix-date v0.15.1
   Compiling gix-chunk v0.7.0
   Compiling gix-quote v0.7.0
   Compiling gix-bitmap v0.3.0
   Compiling nickel-lang-vector v0.1.0
   Compiling serde_yaml v0.9.34+deprecated
   Compiling oci-spec v0.7.1
   Compiling gix-features v0.46.2
   Compiling gix-config-value v0.17.1
   Compiling gix-command v0.8.0
   Compiling gix-url v0.35.2
   Compiling gix-actor v0.40.0
   Compiling http-body v1.0.1
   Compiling ureq-proto v0.6.0
   Compiling rand_chacha v0.3.1
   Compiling serde_spanned v0.6.9
   Compiling gix-prompt v0.14.1
   Compiling http-body-util v0.1.3
   Compiling rand v0.8.6
   Compiling toml_datetime v0.6.11
   Compiling gix-hash v0.23.0
   Compiling gix-fs v0.19.2
   Compiling gix-glob v0.24.0
   Compiling gix-mailmap v0.32.0
   Compiling crunch-shell-core v0.1.0 (/home/brittonr/git/mantle/crates/crunch-shell-core)
   Compiling crunch-release-core v0.1.0 (/home/brittonr/git/mantle/crates/crunch-release-core)
   Compiling gix-credentials v0.37.1
   Compiling crunch-bootstrap-core v0.1.0 (/home/brittonr/git/mantle/crates/crunch-bootstrap-core)
   Compiling toml_edit v0.22.27
   Compiling gix-hashtable v0.13.0
   Compiling gix-commitgraph v0.35.0
   Compiling gix-tempfile v21.0.2
   Compiling gix-attributes v0.31.0
   Compiling gix-object v0.58.0
   Compiling gix-ignore v0.19.1
   Compiling crunch-attestation v0.1.0 (/home/brittonr/git/mantle/crates/crunch-attestation)
   Compiling crunch-project-core v0.1.0 (/home/brittonr/git/mantle/crates/crunch-project-core)
   Compiling crunch-shell v0.1.0 (/home/brittonr/git/mantle/crates/crunch-shell)
   Compiling gix-lock v21.0.2
   Compiling gix-shallow v0.10.0
   Compiling gix-pathspec v0.16.1
   Compiling gix-revwalk v0.29.0
   Compiling gix-ref v0.61.0
   Compiling gix-filter v0.28.0
   Compiling gix-pack v0.68.0
   Compiling gix-traverse v0.55.0
   Compiling gix-revision v0.43.0
   Compiling gix-negotiate v0.29.0
   Compiling toml v0.8.23
   Compiling xz2 v0.1.7
   Compiling gix-index v0.49.0
   Compiling gix-worktree-stream v0.30.0
   Compiling gix-refspec v0.39.0
   Compiling gix-archive v0.30.0
   Compiling gix-discover v0.49.0
   Compiling zstd v0.13.3
   Compiling gix-config v0.54.0
   Compiling gix-worktree v0.50.0
   Compiling gix-odb v0.78.0
   Compiling gix-diff v0.61.0
   Compiling gix-dir v0.23.0
   Compiling gix-worktree-state v0.28.0
   Compiling tokio-util v0.7.18
   Compiling tower v0.5.3
   Compiling tokio-stream v0.1.18
   Compiling async-compression v0.4.19
   Compiling malachite-q v0.6.1
   Compiling crunch-project v0.1.0 (/home/brittonr/git/mantle/crates/crunch-project)
   Compiling astral-tokio-tar v0.6.3
   Compiling fastcdc v3.2.1
   Compiling tower-http v0.6.8
   Compiling gix-submodule v0.28.0
   Compiling gix-status v0.28.0
   Compiling gix-blame v0.11.0
   Compiling h2 v0.4.13
   Compiling malachite-float v0.6.1
   Compiling irpc v0.13.0
   Compiling malachite v0.6.1
   Compiling crunch-glue v0.1.0 (/home/brittonr/git/mantle/crates/crunch-glue)
   Compiling hyper v1.9.0
   Compiling hyper-util v0.1.20
   Compiling rustls-webpki v0.103.13
   Compiling tokio-rustls v0.26.4
   Compiling rustls-platform-verifier v0.6.2
   Compiling ureq v3.3.0
   Compiling hyper-rustls v0.27.7
   Compiling reqwest v0.13.2
   Compiling reqwest-middleware v0.5.1
   Compiling object_store v0.14.0
   Compiling gix-transport v0.55.1
   Compiling reqwest-tracing v0.6.0
   Compiling gix-protocol v0.59.0
   Compiling snix-tracing v0.1.0 (/home/brittonr/git/mantle/vendor/snix-tracing)
   Compiling gix v0.81.0
   Compiling crunch-store v0.1.0 (/home/brittonr/git/mantle/crates/crunch-store)
   Compiling crunch-build v0.1.0 (/home/brittonr/git/mantle/crates/crunch-build)
   Compiling crunch-delta v0.1.0 (/home/brittonr/git/mantle/crates/crunch-delta)
   Compiling nickel-lang v2.0.0
   Compiling crunch-eval v0.1.0 (/home/brittonr/git/mantle/crates/crunch-eval)
   Compiling crunch-pipeline v0.1.0 (/home/brittonr/git/mantle/crates/crunch-pipeline)
   Compiling mantle v0.1.0 (/home/brittonr/git/mantle)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1m 07s
     Running unittests src/main.rs (/tmp/mantle-rust-plan-validation-target/debug/deps/mantle-2027653048753230)

running 189 tests
test rust_plan::tests::allowed_rust_topology_compile_env_rejects_unrelated_ambient_env ... ok
test rust_plan::tests::append_all_produced_dependency_search_paths_keeps_duplicate_package_variant_dirs ... ok
test rust_plan::tests::aws_lc_memcmp_guard_failure_becomes_stable_compiler_guard_blocker ... ok
test rust_plan::tests::bind_all_host_artifacts_adds_proc_macro_extern_without_dependency_placeholder ... ok
test rust_plan::tests::bind_all_host_artifacts_does_not_add_extern_for_custom_build_artifact ... ok
test rust_plan::tests::bind_all_host_artifacts_leaves_unknown_custom_build_alias_placeholder ... ok
test rust_plan::tests::bind_all_host_artifacts_rewrites_custom_build_main_alias_placeholder ... ok
test rust_plan::tests::bind_all_host_artifacts_rewrites_existing_proc_macro_placeholder_without_duplicate ... ok
test rust_plan::tests::bind_all_host_artifacts_with_index_blocks_ambiguous_package_only_host_candidates ... ok
test rust_plan::tests::bind_build_script_metadata_adds_dep_env_for_linked_dependency ... ok
test rust_plan::tests::bind_build_script_metadata_with_index_blocks_ambiguous_package_only_metadata_candidates ... ok
test rust_plan::tests::bind_build_script_metadata_with_index_keeps_same_alias_duplicates_ambiguous ... ok
test rust_plan::tests::bind_dependency_artifacts_uses_selected_unit_variant ... ok
test rust_plan::tests::bind_target_unit_artifacts_adds_proc_macro_search_path_for_transitive_metadata ... ok
test rust_plan::tests::blocked_receipt_records_diagnostic_context_and_bounded_replay_evidence ... ok
test rust_plan::tests::build_script_child_env_adds_c_prefix_map_flags_for_receipt_bound_gnu_compiler ... ok
test rust_plan::tests::build_script_child_env_does_not_forward_compiler_guard_bypass_env ... ok
test rust_plan::tests::build_script_child_env_ignores_ambient_profile_env ... ok
test rust_plan::tests::build_script_child_env_omits_manifest_dir_without_source_arg ... ok
test rust_plan::tests::build_script_child_env_sets_tool_target_and_manifest_package_name ... ok
test rust_plan::tests::build_script_metadata_out_dir_is_scoped_by_output_root_and_unit_id ... ok
test rust_plan::tests::build_script_metadata_producer_index_keeps_same_alias_duplicates_ambiguous ... ok
test rust_plan::tests::build_script_metadata_success_receipt_records_selected_compiler_route ... ok
test rust_plan::tests::build_script_profile_env_child_ignores_ambient_process_env_probe ... ok
test rust_plan::tests::build_script_profile_env_derives_dev_and_release_defaults ... ok
test rust_plan::tests::build_script_target_cfg_env_derives_x86_64_linux_values ... ok
test rust_plan::tests::build_script_target_cfg_env_uses_empty_env_for_wasm_unknown ... ok
test rust_plan::tests::c_prefix_map_flags_reject_unknown_compiler_family_and_empty_remaps ... ok
test rust_plan::tests::captures_normalized_oracle_receipt ... ok
test rust_plan::tests::cargo_unit_derivation_promotes_proc_macro_crate_type_to_host_unit ... ok
test rust_plan::tests::collect_native_manifest_lock_texts_fails_closed_on_missing_lockfile ... ok
test rust_plan::tests::collect_native_manifest_lock_texts_reads_root_member_manifests_and_lockfile ... ok
test rust_plan::tests::combined_unit_topology_blocks_missing_linked_metadata_producer ... ok
test rust_plan::tests::combined_unit_topology_blocks_missing_target_host_artifact_producer ... ok
test rust_plan::tests::combined_unit_topology_keeps_standalone_host_units ... ok
test rust_plan::tests::combined_unit_topology_orders_host_build_script_before_same_package_proc_macro ... ok
test rust_plan::tests::combined_unit_topology_orders_host_dependency_lib_before_host_unit_not_target_lib ... ok
test rust_plan::tests::combined_unit_topology_orders_linked_metadata_before_dependent_build_script ... ok
test rust_plan::tests::combined_unit_topology_orders_proc_macro_dependency_lib_before_host_unit ... ok
test rust_plan::tests::combined_unit_topology_orders_target_host_and_proc_macro_edges ... ok
test rust_plan::tests::compiler_policy_audit_mode_records_adapter_identity_and_waivers ... ok
test rust_plan::tests::compiler_policy_deny_mode_rejects_adapter_failure ... ok
test rust_plan::tests::compiler_policy_expected_manifest_digest_mismatch_blocks_required_mode ... ok
test rust_plan::tests::compiler_policy_json_receipt_contains_identity_and_waiver_summary ... ok
test rust_plan::tests::compiler_policy_report_path_is_scoped_by_output_root_and_unit_id ... ok
test rust_plan::tests::compiler_policy_required_mode_blocks_missing_driver ... ok
test rust_plan::tests::compiler_policy_required_mode_blocks_missing_lint_library ... ok
test rust_plan::tests::compiler_policy_required_mode_blocks_missing_standards_artifact ... ok
test rust_plan::tests::compiler_policy_required_mode_blocks_policy_digest_mismatch ... ok
test rust_plan::tests::compiler_policy_required_mode_blocks_toolchain_mismatch ... ok
test rust_plan::tests::compiler_policy_required_mode_records_complete_static_identity ... ok
test rust_plan::tests::compiler_policy_required_mode_rejects_raw_rustc_cached_output ... ok
test rust_plan::tests::custom_build_manifest_dir_stays_real_in_deterministic_mode ... ok
test rust_plan::tests::dependency_chain_blocks_missing_producer_before_consumer_rustc ... ok
test rust_plan::tests::deterministic_release_path_remaps_prefer_provider_toolchain_inside_source_root ... ok
test rust_plan::tests::deterministic_release_paths_add_remaps_and_provider_placeholder_env ... ok
test rust_plan::tests::executes_dependency_chain_from_produced_lib_artifact ... ok
test rust_plan::tests::executes_first_supported_lib_unit_from_derivation_graph ... ok
test rust_plan::tests::finalized_receipt_records_role_triple_policy_and_identity ... ok
test rust_plan::tests::host_dependency_derivations_clone_libs_for_host_consumers_without_rewriting_targets ... ok
test rust_plan::tests::host_dependency_derivations_do_not_guess_ambiguous_package_fallbacks ... ok
test rust_plan::tests::host_dependency_remap_preserves_explicit_non_target_producers ... ok
test rust_plan::tests::host_dependency_topology_accepts_proc_macro_host_producer ... ok
test rust_plan::tests::host_dependency_topology_accepts_target_lib_producer ... ok
test rust_plan::tests::host_dependency_topology_blocks_missing_producer ... ok
test rust_plan::tests::native_cargo_package_env_inherits_optional_workspace_metadata ... ok
test rust_plan::tests::native_cargo_package_env_sets_version_components_and_empty_defaults ... ok
test rust_plan::tests::native_feature_resolver_blocks_malformed_feature_edges ... ok
test rust_plan::tests::native_feature_resolver_exposes_bare_optional_dependency_as_cfg_feature ... ok
test rust_plan::tests::native_feature_resolver_leaves_optional_dependency_unselected_without_feature ... ok
test rust_plan::tests::native_feature_resolver_models_dependency_feature_edges_and_rejects_unknown_entries ... ok
test rust_plan::tests::native_feature_resolver_reaches_fixed_point_for_defaults_explicit_and_optional_dependencies ... ok
test rust_plan::tests::native_feature_role_resolver_keeps_normal_build_and_host_features_separate ... ok
test rust_plan::tests::native_git_dependency_resolution_blocks_ambiguous_same_url_sources ... ok
test rust_plan::tests::native_git_dependency_resolution_blocks_mismatched_revision ... ok
test rust_plan::tests::native_git_dev_dependency_resolution_blocks_mismatched_revision ... ok
test rust_plan::tests::native_git_source_planning_binds_captured_source_and_resolves_dependency_edge ... ok
test rust_plan::tests::native_git_source_planning_blocks_missing_captured_manifest ... ok
test rust_plan::tests::native_host_dependencies_drop_build_script_artifacts_for_proc_macro_units ... ok
test rust_plan::tests::native_host_dependencies_use_normal_deps_only_for_proc_macro_units ... ok
test rust_plan::tests::native_host_derivation_adds_compiler_proc_macro_extern ... ok
test rust_plan::tests::native_host_derivation_caps_lints_for_registry_source ... ok
test rust_plan::tests::native_host_derivation_carries_manifest_package_name_for_build_script_env ... ok
test rust_plan::tests::native_host_metadata_dependencies_follow_selected_target_artifacts_only ... ok
test rust_plan::tests::native_host_metadata_dependencies_ignore_unselected_linked_manifest_edges ... ok
test rust_plan::tests::native_host_planning_binds_run_custom_build_alias_to_exact_unique_build_unit ... ok
test rust_plan::tests::native_host_planning_does_not_bind_run_custom_build_alias_to_first_duplicate ... ok
test rust_plan::tests::native_host_planning_keeps_selected_same_package_build_script_for_proc_macro ... ok
test rust_plan::tests::native_host_planning_links_proc_macro_dependency_proc_macros_without_cargo_edges ... ok
test rust_plan::tests::native_host_planning_normalizes_proc_macro_target_names_and_follows_selected_units_only ... ok
test rust_plan::tests::native_host_planning_preserves_duplicate_selected_custom_build_unit_ids ... ok
test rust_plan::tests::native_host_planning_preserves_duplicate_selected_proc_macro_unit_ids ... ok
test rust_plan::tests::native_host_planning_selects_lib_kind_proc_macro_crate_type ... ok
test rust_plan::tests::native_host_unit_derivation_emits_metadata_disambiguator ... ok
test rust_plan::tests::native_manifest_declared_edition_feeds_host_derivation_args ... ok
test rust_plan::tests::native_manifest_declared_edition_feeds_target_derivation_args ... ok
test rust_plan::tests::native_manifest_lock_unsupported_blockers_accept_supported_subset ... ok
test rust_plan::tests::native_manifest_lock_unsupported_blockers_reject_patch_replace_target_build_and_unknown_lock_source ... ok
test rust_plan::tests::native_manifest_missing_edition_uses_cargo_default ... ok
test rust_plan::tests::native_manifest_package_build_path_and_false_control_targets ... ok
test rust_plan::tests::native_manifest_proc_macro_alias_feeds_target_planning ... ok
test rust_plan::tests::native_manifest_workspace_edition_feeds_target_derivation_args ... ok
test rust_plan::tests::native_package_oracle_comparison_accepts_supported_fixture ... ok
test rust_plan::tests::native_package_oracle_comparison_blocks_identity_mismatch_and_missing_cargo_package ... ok
test rust_plan::tests::native_package_target_fragment_ignores_out_of_scope_oracle_target_kinds ... ok
test rust_plan::tests::native_package_target_fragment_matches_supported_path_workspace ... ok
test rust_plan::tests::native_registry_source_planning_binds_declared_vendor_source ... ok
test rust_plan::tests::native_registry_source_planning_blocks_missing_vendor_material ... ok
test rust_plan::tests::native_target_cfg_predicate_scope_blocks_unknown_or_malformed_syntax ... ok
test rust_plan::tests::native_target_cfg_predicate_scope_evaluates_nested_common_predicates ... ok
test rust_plan::tests::native_unit_derivation_adds_selected_feature_cfg_args ... ok
test rust_plan::tests::native_unit_derivation_caps_lints_for_registry_and_git_sources ... ok
test rust_plan::tests::native_unit_derivation_emits_resolved_feature_closure_cfg_args ... ok
test rust_plan::tests::native_unit_derivation_emits_stable_metadata_disambiguator ... ok
test rust_plan::tests::native_unit_derivation_leaves_path_sources_uncapped ... ok
test rust_plan::tests::native_unit_graph_adds_build_dependency_producer_unit ... ok
test rust_plan::tests::native_unit_graph_adds_transitive_registry_producer_unit ... ok
test rust_plan::tests::native_unit_graph_blocks_native_dependency_without_package_fact ... ok
test rust_plan::tests::native_unit_graph_blocks_native_dependency_without_source_fact ... ok
test rust_plan::tests::native_unit_graph_blocks_registry_dependency_without_lib_producer ... ok
test rust_plan::tests::native_unit_graph_filters_package_self_dependency_from_lib_unit ... ok
test rust_plan::tests::native_unit_graph_follows_native_dependency_facts_without_cargo_unit_edges ... ok
test rust_plan::tests::native_unit_graph_fragment_blocks_mismatch_and_missing_edges ... ok
test rust_plan::tests::native_unit_graph_fragment_feeds_supported_unit_derivations ... ok
test rust_plan::tests::native_unit_graph_keeps_selected_lib_dependency_with_host_target_sibling ... ok
test rust_plan::tests::native_unit_graph_uses_lib_crate_name_when_package_name_differs ... ok
test rust_plan::tests::native_unit_graph_uses_stable_native_unit_identity ... ok
test rust_plan::tests::native_unit_metadata_disambiguator_ignores_ambient_tool_roots ... ok
test rust_plan::tests::no_cargo_capture_binds_captured_git_source ... ok
test rust_plan::tests::no_cargo_capture_binds_declared_vendored_registry_source ... ok
test rust_plan::tests::no_cargo_capture_blocks_missing_vendored_registry_source ... ok
test rust_plan::tests::non_aws_lc_guard_text_stays_plain_build_script_failure ... ok
test rust_plan::tests::package_only_dependency_artifacts_bind_single_renamed_crate_candidate ... ok
test rust_plan::tests::package_only_dependency_artifacts_fail_on_ambiguous_unit_variants ... ok
test rust_plan::tests::parse_build_script_metadata_accepts_bounded_link_lib_forms ... ok
test rust_plan::tests::parse_build_script_metadata_captures_link_metadata ... ok
test rust_plan::tests::parse_build_script_metadata_rejects_bad_custom_key ... ok
test rust_plan::tests::parse_build_script_metadata_rejects_unsafe_link_lib_forms ... ok
test rust_plan::tests::parse_native_lockfile_text_extracts_source_identities_revisions_checksums_and_edges ... ok
test rust_plan::tests::parse_native_lockfile_text_rejects_malformed_package_records ... ok
test rust_plan::tests::parse_native_manifest_text_extracts_core_workspace_package_target_and_dependency_facts ... ok
test rust_plan::tests::parse_native_manifest_text_rejects_invalid_inherited_package_version ... ok
test rust_plan::tests::path_source_digest_ignores_root_metadata_without_hiding_source_changes ... ok
test rust_plan::tests::proc_macro_crate_type_only_overrides_lib_shaped_targets ... ok
test rust_plan::tests::process_global_env_mutation_guard_detects_set_and_remove_var_calls ... ok
test rust_plan::tests::redacted_diagnostic_limits_lines_and_redacts_temp_paths ... ok
test rust_plan::tests::redacted_diagnostic_preserves_non_temp_context ... ok
test rust_plan::tests::registry_dependency_source_rejects_missing_exact_same_name_version ... ok
test rust_plan::tests::registry_dependency_source_uses_default_caret_semver_compatibility ... ok
test rust_plan::tests::registry_dependency_source_uses_exact_version_when_names_repeat ... ok
test rust_plan::tests::registry_dependency_source_uses_highest_matching_comparator_range ... ok
test rust_plan::tests::replay_evidence_json_rejects_invalid_digest_fields ... ok
test rust_plan::tests::replay_evidence_json_rejects_malformed_and_oversized_inputs ... ok
test rust_plan::tests::role_validation_rejects_host_consumer_bound_to_target_dependency ... ok
test rust_plan::tests::role_validation_rejects_host_unit_target_triple_mismatch ... ok
test rust_plan::tests::role_validation_rejects_metadata_hash_mismatch ... ok
test rust_plan::tests::role_validation_rejects_source_package_mismatch ... ok
test rust_plan::tests::role_validation_rejects_toolchain_policy_digest_mismatch ... ok
test rust_plan::tests::rust_plan_tests_do_not_mutate_process_global_environment_in_process ... ok
test rust_plan::tests::rust_topology_child_env_forwards_allowlisted_compile_env ... ok
test rust_plan::tests::rust_topology_child_env_omits_empty_inherited_path ... ok
test rust_plan::tests::rust_topology_child_env_prefers_explicit_compile_env ... ok
test rust_plan::tests::rust_topology_child_env_prefers_explicit_derivation_path ... ok
test rust_plan::tests::rust_topology_child_env_preserves_non_empty_inherited_path ... ok
test rust_plan::tests::rust_topology_child_env_rejects_ambient_compiler_guard_bypass_env ... ok
test rust_plan::tests::rust_topology_runtime_args_disable_self_contained_linker_by_default ... ok
test rust_plan::tests::rust_topology_runtime_args_ignore_near_match_linker_mode ... ok
test rust_plan::tests::rust_topology_runtime_args_preserve_joined_explicit_linker_mode ... ok
test rust_plan::tests::rust_topology_runtime_args_preserve_split_explicit_linker_mode ... ok
test rust_plan::tests::rust_unit_execution_blocks_missing_declared_output_before_rustc ... ok
test rust_plan::tests::rust_unit_execution_blocks_missing_dependency_artifact_before_rustc ... ok
test rust_plan::tests::rust_unit_execution_blocks_missing_host_artifact_before_rustc ... ok
test rust_plan::tests::rust_unit_execution_blocks_missing_source_before_rustc ... ok
test rust_plan::tests::rust_unit_execution_blocks_source_closure_blocker_before_rustc ... ok
test rust_plan::tests::rustc_metadata_disambiguator_distinguishes_same_crate_package_versions ... ok
test rust_plan::tests::selected_dependency_search_paths_follow_unit_variant_closure_only ... ok
test rust_plan::tests::source_built_c_compiler_route_json_rejects_non_compiler_route ... ok
test rust_plan::tests::source_built_c_compiler_route_json_validates_receipt_bound_identity ... ok
test rust_plan::tests::source_closure_blocks_registry_without_checksum ... ok
test rust_plan::tests::source_closure_records_registry_git_and_path_identities ... ok
test rust_plan::tests::target_dependency_producer_index_uses_selected_unit_variant ... ok
test rust_plan::tests::unit_dependency_artifacts_preserve_selected_cargo_unit_id ... ok
test rust_plan::tests::unit_dependency_artifacts_skip_host_producer_edges ... ok
test rust_plan::tests::unit_derivation_graph_binds_proc_macro_host_artifacts_to_target_units ... ok
test rust_plan::tests::unit_derivation_graph_blocks_doctest_or_non_build_modes ... ok
test rust_plan::tests::unit_derivation_graph_blocks_unsupported_target_kinds ... ok
test rust_plan::tests::unit_derivation_graph_emits_binary_with_dependency_artifact ... ok
test rust_plan::tests::unit_derivation_graph_represents_build_script_host_units ... ok
test rust_plan::tests::unit_graph_failure_fails_closed ... ok

test result: ok. 189 passed; 0 failed; 0 ignored; 0 measured; 951 filtered out; finished in 0.23s

```

## Focused parallel stress rail

```text
$ for run in 1 2 3; do cargo test -p mantle --bin mantle rust_plan::tests:: -- --test-threads=8; done
### parallel run 1
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.20s
     Running unittests src/main.rs (/tmp/mantle-rust-plan-validation-target/debug/deps/mantle-2027653048753230)

running 189 tests
test rust_plan::tests::aws_lc_memcmp_guard_failure_becomes_stable_compiler_guard_blocker ... ok
test rust_plan::tests::allowed_rust_topology_compile_env_rejects_unrelated_ambient_env ... ok
test rust_plan::tests::bind_build_script_metadata_adds_dep_env_for_linked_dependency ... ok
test rust_plan::tests::bind_all_host_artifacts_rewrites_custom_build_main_alias_placeholder ... ok
test rust_plan::tests::bind_all_host_artifacts_does_not_add_extern_for_custom_build_artifact ... ok
test rust_plan::tests::bind_all_host_artifacts_adds_proc_macro_extern_without_dependency_placeholder ... ok
test rust_plan::tests::bind_all_host_artifacts_leaves_unknown_custom_build_alias_placeholder ... ok
test rust_plan::tests::append_all_produced_dependency_search_paths_keeps_duplicate_package_variant_dirs ... ok
test rust_plan::tests::bind_all_host_artifacts_rewrites_existing_proc_macro_placeholder_without_duplicate ... ok
test rust_plan::tests::bind_build_script_metadata_with_index_blocks_ambiguous_package_only_metadata_candidates ... ok
test rust_plan::tests::bind_build_script_metadata_with_index_keeps_same_alias_duplicates_ambiguous ... ok
test rust_plan::tests::bind_all_host_artifacts_with_index_blocks_ambiguous_package_only_host_candidates ... ok
test rust_plan::tests::bind_dependency_artifacts_uses_selected_unit_variant ... ok
test rust_plan::tests::build_script_child_env_does_not_forward_compiler_guard_bypass_env ... ok
test rust_plan::tests::bind_target_unit_artifacts_adds_proc_macro_search_path_for_transitive_metadata ... ok
test rust_plan::tests::blocked_receipt_records_diagnostic_context_and_bounded_replay_evidence ... ok
test rust_plan::tests::build_script_child_env_adds_c_prefix_map_flags_for_receipt_bound_gnu_compiler ... ok
test rust_plan::tests::build_script_child_env_omits_manifest_dir_without_source_arg ... ok
test rust_plan::tests::build_script_profile_env_child_ignores_ambient_process_env_probe ... ok
test rust_plan::tests::build_script_profile_env_derives_dev_and_release_defaults ... ok
test rust_plan::tests::build_script_target_cfg_env_derives_x86_64_linux_values ... ok
test rust_plan::tests::build_script_child_env_sets_tool_target_and_manifest_package_name ... ok
test rust_plan::tests::build_script_target_cfg_env_uses_empty_env_for_wasm_unknown ... ok
test rust_plan::tests::build_script_metadata_producer_index_keeps_same_alias_duplicates_ambiguous ... ok
test rust_plan::tests::build_script_metadata_out_dir_is_scoped_by_output_root_and_unit_id ... ok
test rust_plan::tests::combined_unit_topology_blocks_missing_linked_metadata_producer ... ok
test rust_plan::tests::combined_unit_topology_blocks_missing_target_host_artifact_producer ... ok
test rust_plan::tests::build_script_metadata_success_receipt_records_selected_compiler_route ... ok
test rust_plan::tests::c_prefix_map_flags_reject_unknown_compiler_family_and_empty_remaps ... ok
test rust_plan::tests::combined_unit_topology_keeps_standalone_host_units ... ok
test rust_plan::tests::combined_unit_topology_orders_linked_metadata_before_dependent_build_script ... ok
test rust_plan::tests::combined_unit_topology_orders_host_build_script_before_same_package_proc_macro ... ok
test rust_plan::tests::combined_unit_topology_orders_proc_macro_dependency_lib_before_host_unit ... ok
test rust_plan::tests::combined_unit_topology_orders_host_dependency_lib_before_host_unit_not_target_lib ... ok
test rust_plan::tests::combined_unit_topology_orders_target_host_and_proc_macro_edges ... ok
test rust_plan::tests::cargo_unit_derivation_promotes_proc_macro_crate_type_to_host_unit ... ok
test rust_plan::tests::collect_native_manifest_lock_texts_fails_closed_on_missing_lockfile ... ok
test rust_plan::tests::collect_native_manifest_lock_texts_reads_root_member_manifests_and_lockfile ... ok
test rust_plan::tests::compiler_policy_report_path_is_scoped_by_output_root_and_unit_id ... ok
test rust_plan::tests::captures_normalized_oracle_receipt ... ok
test rust_plan::tests::build_script_child_env_ignores_ambient_profile_env ... ok
test rust_plan::tests::compiler_policy_required_mode_blocks_missing_lint_library ... ok
test rust_plan::tests::compiler_policy_required_mode_blocks_missing_standards_artifact ... ok
test rust_plan::tests::compiler_policy_required_mode_blocks_policy_digest_mismatch ... ok
test rust_plan::tests::compiler_policy_expected_manifest_digest_mismatch_blocks_required_mode ... ok
test rust_plan::tests::custom_build_manifest_dir_stays_real_in_deterministic_mode ... ok
test rust_plan::tests::dependency_chain_blocks_missing_producer_before_consumer_rustc ... ok
test rust_plan::tests::compiler_policy_required_mode_blocks_missing_driver ... ok
test rust_plan::tests::deterministic_release_path_remaps_prefer_provider_toolchain_inside_source_root ... ok
test rust_plan::tests::deterministic_release_paths_add_remaps_and_provider_placeholder_env ... ok
test rust_plan::tests::compiler_policy_required_mode_blocks_toolchain_mismatch ... ok
test rust_plan::tests::finalized_receipt_records_role_triple_policy_and_identity ... ok
test rust_plan::tests::compiler_policy_deny_mode_rejects_adapter_failure ... ok
test rust_plan::tests::host_dependency_derivations_clone_libs_for_host_consumers_without_rewriting_targets ... ok
test rust_plan::tests::host_dependency_derivations_do_not_guess_ambiguous_package_fallbacks ... ok
test rust_plan::tests::host_dependency_remap_preserves_explicit_non_target_producers ... ok
test rust_plan::tests::host_dependency_topology_accepts_proc_macro_host_producer ... ok
test rust_plan::tests::host_dependency_topology_accepts_target_lib_producer ... ok
test rust_plan::tests::host_dependency_topology_blocks_missing_producer ... ok
test rust_plan::tests::native_cargo_package_env_sets_version_components_and_empty_defaults ... ok
test rust_plan::tests::native_feature_resolver_blocks_malformed_feature_edges ... ok
test rust_plan::tests::native_cargo_package_env_inherits_optional_workspace_metadata ... ok
test rust_plan::tests::native_feature_resolver_exposes_bare_optional_dependency_as_cfg_feature ... ok
test rust_plan::tests::native_feature_resolver_leaves_optional_dependency_unselected_without_feature ... ok
test rust_plan::tests::native_feature_resolver_models_dependency_feature_edges_and_rejects_unknown_entries ... ok
test rust_plan::tests::native_feature_resolver_reaches_fixed_point_for_defaults_explicit_and_optional_dependencies ... ok
test rust_plan::tests::native_feature_role_resolver_keeps_normal_build_and_host_features_separate ... ok
test rust_plan::tests::native_git_dependency_resolution_blocks_mismatched_revision ... ok
test rust_plan::tests::native_git_dependency_resolution_blocks_ambiguous_same_url_sources ... ok
test rust_plan::tests::native_git_dev_dependency_resolution_blocks_mismatched_revision ... ok
test rust_plan::tests::native_git_source_planning_blocks_missing_captured_manifest ... ok
test rust_plan::tests::native_host_dependencies_drop_build_script_artifacts_for_proc_macro_units ... ok
test rust_plan::tests::native_host_dependencies_use_normal_deps_only_for_proc_macro_units ... ok
test rust_plan::tests::native_git_source_planning_binds_captured_source_and_resolves_dependency_edge ... ok
test rust_plan::tests::native_host_derivation_adds_compiler_proc_macro_extern ... ok
test rust_plan::tests::compiler_policy_json_receipt_contains_identity_and_waiver_summary ... ok
test rust_plan::tests::native_host_derivation_caps_lints_for_registry_source ... ok
test rust_plan::tests::native_host_metadata_dependencies_ignore_unselected_linked_manifest_edges ... ok
test rust_plan::tests::native_host_metadata_dependencies_follow_selected_target_artifacts_only ... ok
test rust_plan::tests::native_host_derivation_carries_manifest_package_name_for_build_script_env ... ok
test rust_plan::tests::compiler_policy_required_mode_records_complete_static_identity ... ok
test rust_plan::tests::compiler_policy_audit_mode_records_adapter_identity_and_waivers ... ok
test rust_plan::tests::native_host_planning_keeps_selected_same_package_build_script_for_proc_macro ... ok
test rust_plan::tests::native_host_planning_binds_run_custom_build_alias_to_exact_unique_build_unit ... ok
test rust_plan::tests::native_host_planning_links_proc_macro_dependency_proc_macros_without_cargo_edges ... ok
test rust_plan::tests::native_host_planning_does_not_bind_run_custom_build_alias_to_first_duplicate ... ok
test rust_plan::tests::native_host_unit_derivation_emits_metadata_disambiguator ... ok
test rust_plan::tests::native_host_planning_normalizes_proc_macro_target_names_and_follows_selected_units_only ... ok
test rust_plan::tests::native_host_planning_selects_lib_kind_proc_macro_crate_type ... ok
test rust_plan::tests::native_host_planning_preserves_duplicate_selected_proc_macro_unit_ids ... ok
test rust_plan::tests::native_host_planning_preserves_duplicate_selected_custom_build_unit_ids ... ok
test rust_plan::tests::native_manifest_lock_unsupported_blockers_accept_supported_subset ... ok
test rust_plan::tests::compiler_policy_required_mode_rejects_raw_rustc_cached_output ... ok
test rust_plan::tests::native_manifest_lock_unsupported_blockers_reject_patch_replace_target_build_and_unknown_lock_source ... ok
test rust_plan::tests::native_manifest_declared_edition_feeds_host_derivation_args ... ok
test rust_plan::tests::native_manifest_proc_macro_alias_feeds_target_planning ... ok
test rust_plan::tests::native_package_oracle_comparison_accepts_supported_fixture ... ok
test rust_plan::tests::native_package_oracle_comparison_blocks_identity_mismatch_and_missing_cargo_package ... ok
test rust_plan::tests::native_manifest_declared_edition_feeds_target_derivation_args ... ok
test rust_plan::tests::native_manifest_package_build_path_and_false_control_targets ... ok
test rust_plan::tests::native_manifest_missing_edition_uses_cargo_default ... ok
test rust_plan::tests::native_target_cfg_predicate_scope_blocks_unknown_or_malformed_syntax ... ok
test rust_plan::tests::native_registry_source_planning_blocks_missing_vendor_material ... ok
test rust_plan::tests::native_target_cfg_predicate_scope_evaluates_nested_common_predicates ... ok
test rust_plan::tests::native_unit_derivation_adds_selected_feature_cfg_args ... ok
test rust_plan::tests::native_unit_derivation_caps_lints_for_registry_and_git_sources ... ok
test rust_plan::tests::native_package_target_fragment_ignores_out_of_scope_oracle_target_kinds ... ok
test rust_plan::tests::native_unit_derivation_emits_resolved_feature_closure_cfg_args ... ok
test rust_plan::tests::native_manifest_workspace_edition_feeds_target_derivation_args ... ok
test rust_plan::tests::native_unit_derivation_emits_stable_metadata_disambiguator ... ok
test rust_plan::tests::native_unit_derivation_leaves_path_sources_uncapped ... ok
test rust_plan::tests::native_unit_graph_adds_build_dependency_producer_unit ... ok
test rust_plan::tests::native_package_target_fragment_matches_supported_path_workspace ... ok
test rust_plan::tests::native_registry_source_planning_binds_declared_vendor_source ... ok
test rust_plan::tests::native_unit_graph_blocks_native_dependency_without_package_fact ... ok
test rust_plan::tests::native_unit_graph_blocks_native_dependency_without_source_fact ... ok
test rust_plan::tests::native_unit_graph_filters_package_self_dependency_from_lib_unit ... ok
test rust_plan::tests::native_unit_graph_follows_native_dependency_facts_without_cargo_unit_edges ... ok
test rust_plan::tests::native_unit_graph_keeps_selected_lib_dependency_with_host_target_sibling ... ok
test rust_plan::tests::native_unit_graph_uses_lib_crate_name_when_package_name_differs ... ok
test rust_plan::tests::native_unit_graph_fragment_blocks_mismatch_and_missing_edges ... ok
test rust_plan::tests::native_unit_metadata_disambiguator_ignores_ambient_tool_roots ... ok
test rust_plan::tests::native_unit_graph_uses_stable_native_unit_identity ... ok
test rust_plan::tests::native_unit_graph_blocks_registry_dependency_without_lib_producer ... ok
test rust_plan::tests::native_unit_graph_adds_transitive_registry_producer_unit ... ok
test rust_plan::tests::non_aws_lc_guard_text_stays_plain_build_script_failure ... ok
test rust_plan::tests::package_only_dependency_artifacts_bind_single_renamed_crate_candidate ... ok
test rust_plan::tests::parse_build_script_metadata_accepts_bounded_link_lib_forms ... ok
test rust_plan::tests::package_only_dependency_artifacts_fail_on_ambiguous_unit_variants ... ok
test rust_plan::tests::parse_build_script_metadata_captures_link_metadata ... ok
test rust_plan::tests::parse_build_script_metadata_rejects_bad_custom_key ... ok
test rust_plan::tests::parse_build_script_metadata_rejects_unsafe_link_lib_forms ... ok
test rust_plan::tests::parse_native_lockfile_text_rejects_malformed_package_records ... ok
test rust_plan::tests::native_unit_graph_fragment_feeds_supported_unit_derivations ... ok
test rust_plan::tests::parse_native_lockfile_text_extracts_source_identities_revisions_checksums_and_edges ... ok
test rust_plan::tests::parse_native_manifest_text_rejects_invalid_inherited_package_version ... ok
test rust_plan::tests::parse_native_manifest_text_extracts_core_workspace_package_target_and_dependency_facts ... ok
test rust_plan::tests::proc_macro_crate_type_only_overrides_lib_shaped_targets ... ok
test rust_plan::tests::process_global_env_mutation_guard_detects_set_and_remove_var_calls ... ok
test rust_plan::tests::redacted_diagnostic_preserves_non_temp_context ... ok
test rust_plan::tests::redacted_diagnostic_limits_lines_and_redacts_temp_paths ... ok
test rust_plan::tests::registry_dependency_source_rejects_missing_exact_same_name_version ... ok
test rust_plan::tests::registry_dependency_source_uses_default_caret_semver_compatibility ... ok
test rust_plan::tests::path_source_digest_ignores_root_metadata_without_hiding_source_changes ... ok
test rust_plan::tests::registry_dependency_source_uses_exact_version_when_names_repeat ... ok
test rust_plan::tests::registry_dependency_source_uses_highest_matching_comparator_range ... ok
test rust_plan::tests::replay_evidence_json_rejects_malformed_and_oversized_inputs ... ok
test rust_plan::tests::replay_evidence_json_rejects_invalid_digest_fields ... ok
test rust_plan::tests::role_validation_rejects_host_consumer_bound_to_target_dependency ... ok
test rust_plan::tests::role_validation_rejects_host_unit_target_triple_mismatch ... ok
test rust_plan::tests::role_validation_rejects_metadata_hash_mismatch ... ok
test rust_plan::tests::role_validation_rejects_source_package_mismatch ... ok
test rust_plan::tests::role_validation_rejects_toolchain_policy_digest_mismatch ... ok
test rust_plan::tests::rust_topology_child_env_forwards_allowlisted_compile_env ... ok
test rust_plan::tests::rust_topology_child_env_omits_empty_inherited_path ... ok
test rust_plan::tests::rust_topology_child_env_prefers_explicit_compile_env ... ok
test rust_plan::tests::rust_topology_child_env_prefers_explicit_derivation_path ... ok
test rust_plan::tests::rust_topology_child_env_preserves_non_empty_inherited_path ... ok
test rust_plan::tests::rust_topology_child_env_rejects_ambient_compiler_guard_bypass_env ... ok
test rust_plan::tests::rust_topology_runtime_args_disable_self_contained_linker_by_default ... ok
test rust_plan::tests::rust_topology_runtime_args_ignore_near_match_linker_mode ... ok
test rust_plan::tests::rust_topology_runtime_args_preserve_joined_explicit_linker_mode ... ok
test rust_plan::tests::rust_topology_runtime_args_preserve_split_explicit_linker_mode ... ok
test rust_plan::tests::rust_plan_tests_do_not_mutate_process_global_environment_in_process ... ok
test rust_plan::tests::no_cargo_capture_blocks_missing_vendored_registry_source ... ok
test rust_plan::tests::no_cargo_capture_binds_captured_git_source ... ok
test rust_plan::tests::rust_unit_execution_blocks_missing_declared_output_before_rustc ... ok
test rust_plan::tests::rust_unit_execution_blocks_missing_dependency_artifact_before_rustc ... ok
test rust_plan::tests::rustc_metadata_disambiguator_distinguishes_same_crate_package_versions ... ok
test rust_plan::tests::source_built_c_compiler_route_json_rejects_non_compiler_route ... ok
test rust_plan::tests::source_built_c_compiler_route_json_validates_receipt_bound_identity ... ok
test rust_plan::tests::source_closure_blocks_registry_without_checksum ... ok
test rust_plan::tests::rust_unit_execution_blocks_source_closure_blocker_before_rustc ... ok
test rust_plan::tests::target_dependency_producer_index_uses_selected_unit_variant ... ok
test rust_plan::tests::unit_dependency_artifacts_preserve_selected_cargo_unit_id ... ok
test rust_plan::tests::unit_dependency_artifacts_skip_host_producer_edges ... ok
test rust_plan::tests::selected_dependency_search_paths_follow_unit_variant_closure_only ... ok
test rust_plan::tests::no_cargo_capture_binds_declared_vendored_registry_source ... ok
test rust_plan::tests::source_closure_records_registry_git_and_path_identities ... ok
test rust_plan::tests::executes_first_supported_lib_unit_from_derivation_graph ... ok
test rust_plan::tests::unit_derivation_graph_blocks_doctest_or_non_build_modes ... ok
test rust_plan::tests::unit_derivation_graph_blocks_unsupported_target_kinds ... ok
test rust_plan::tests::unit_graph_failure_fails_closed ... ok
test rust_plan::tests::unit_derivation_graph_binds_proc_macro_host_artifacts_to_target_units ... ok
test rust_plan::tests::unit_derivation_graph_emits_binary_with_dependency_artifact ... ok
test rust_plan::tests::unit_derivation_graph_represents_build_script_host_units ... ok
test rust_plan::tests::rust_unit_execution_blocks_missing_host_artifact_before_rustc ... ok
test rust_plan::tests::rust_unit_execution_blocks_missing_source_before_rustc ... ok
test rust_plan::tests::executes_dependency_chain_from_produced_lib_artifact ... ok

test result: ok. 189 passed; 0 failed; 0 ignored; 0 measured; 951 filtered out; finished in 0.04s

### parallel run 2
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.18s
     Running unittests src/main.rs (/tmp/mantle-rust-plan-validation-target/debug/deps/mantle-2027653048753230)

running 189 tests
test rust_plan::tests::aws_lc_memcmp_guard_failure_becomes_stable_compiler_guard_blocker ... ok
test rust_plan::tests::allowed_rust_topology_compile_env_rejects_unrelated_ambient_env ... ok
test rust_plan::tests::bind_build_script_metadata_adds_dep_env_for_linked_dependency ... ok
test rust_plan::tests::bind_all_host_artifacts_leaves_unknown_custom_build_alias_placeholder ... ok
test rust_plan::tests::bind_build_script_metadata_with_index_blocks_ambiguous_package_only_metadata_candidates ... ok
test rust_plan::tests::bind_all_host_artifacts_adds_proc_macro_extern_without_dependency_placeholder ... ok
test rust_plan::tests::bind_all_host_artifacts_does_not_add_extern_for_custom_build_artifact ... ok
test rust_plan::tests::append_all_produced_dependency_search_paths_keeps_duplicate_package_variant_dirs ... ok
test rust_plan::tests::bind_all_host_artifacts_rewrites_custom_build_main_alias_placeholder ... ok
test rust_plan::tests::bind_build_script_metadata_with_index_keeps_same_alias_duplicates_ambiguous ... ok
test rust_plan::tests::bind_all_host_artifacts_rewrites_existing_proc_macro_placeholder_without_duplicate ... ok
test rust_plan::tests::bind_all_host_artifacts_with_index_blocks_ambiguous_package_only_host_candidates ... ok
test rust_plan::tests::build_script_child_env_does_not_forward_compiler_guard_bypass_env ... ok
test rust_plan::tests::build_script_child_env_adds_c_prefix_map_flags_for_receipt_bound_gnu_compiler ... ok
test rust_plan::tests::bind_dependency_artifacts_uses_selected_unit_variant ... ok
test rust_plan::tests::blocked_receipt_records_diagnostic_context_and_bounded_replay_evidence ... ok
test rust_plan::tests::build_script_profile_env_child_ignores_ambient_process_env_probe ... ok
test rust_plan::tests::build_script_metadata_producer_index_keeps_same_alias_duplicates_ambiguous ... ok
test rust_plan::tests::build_script_profile_env_derives_dev_and_release_defaults ... ok
test rust_plan::tests::build_script_child_env_omits_manifest_dir_without_source_arg ... ok
test rust_plan::tests::build_script_target_cfg_env_uses_empty_env_for_wasm_unknown ... ok
test rust_plan::tests::build_script_target_cfg_env_derives_x86_64_linux_values ... ok
test rust_plan::tests::build_script_child_env_sets_tool_target_and_manifest_package_name ... ok
test rust_plan::tests::build_script_metadata_out_dir_is_scoped_by_output_root_and_unit_id ... ok
test rust_plan::tests::build_script_metadata_success_receipt_records_selected_compiler_route ... ok
test rust_plan::tests::combined_unit_topology_blocks_missing_linked_metadata_producer ... ok
test rust_plan::tests::combined_unit_topology_blocks_missing_target_host_artifact_producer ... ok
test rust_plan::tests::c_prefix_map_flags_reject_unknown_compiler_family_and_empty_remaps ... ok
test rust_plan::tests::combined_unit_topology_keeps_standalone_host_units ... ok
test rust_plan::tests::combined_unit_topology_orders_host_build_script_before_same_package_proc_macro ... ok
test rust_plan::tests::combined_unit_topology_orders_host_dependency_lib_before_host_unit_not_target_lib ... ok
test rust_plan::tests::combined_unit_topology_orders_linked_metadata_before_dependent_build_script ... ok
test rust_plan::tests::combined_unit_topology_orders_proc_macro_dependency_lib_before_host_unit ... ok
test rust_plan::tests::cargo_unit_derivation_promotes_proc_macro_crate_type_to_host_unit ... ok
test rust_plan::tests::combined_unit_topology_orders_target_host_and_proc_macro_edges ... ok
test rust_plan::tests::bind_target_unit_artifacts_adds_proc_macro_search_path_for_transitive_metadata ... ok
test rust_plan::tests::collect_native_manifest_lock_texts_fails_closed_on_missing_lockfile ... ok
test rust_plan::tests::collect_native_manifest_lock_texts_reads_root_member_manifests_and_lockfile ... ok
test rust_plan::tests::compiler_policy_report_path_is_scoped_by_output_root_and_unit_id ... ok
test rust_plan::tests::captures_normalized_oracle_receipt ... ok
test rust_plan::tests::compiler_policy_expected_manifest_digest_mismatch_blocks_required_mode ... ok
test rust_plan::tests::compiler_policy_deny_mode_rejects_adapter_failure ... ok
test rust_plan::tests::compiler_policy_audit_mode_records_adapter_identity_and_waivers ... ok
test rust_plan::tests::compiler_policy_json_receipt_contains_identity_and_waiver_summary ... ok
test rust_plan::tests::build_script_child_env_ignores_ambient_profile_env ... ok
test rust_plan::tests::custom_build_manifest_dir_stays_real_in_deterministic_mode ... ok
test rust_plan::tests::dependency_chain_blocks_missing_producer_before_consumer_rustc ... ok
test rust_plan::tests::deterministic_release_path_remaps_prefer_provider_toolchain_inside_source_root ... ok
test rust_plan::tests::deterministic_release_paths_add_remaps_and_provider_placeholder_env ... ok
test rust_plan::tests::compiler_policy_required_mode_blocks_missing_driver ... ok
test rust_plan::tests::compiler_policy_required_mode_blocks_missing_standards_artifact ... ok
test rust_plan::tests::compiler_policy_required_mode_blocks_missing_lint_library ... ok
test rust_plan::tests::compiler_policy_required_mode_blocks_policy_digest_mismatch ... ok
test rust_plan::tests::finalized_receipt_records_role_triple_policy_and_identity ... ok
test rust_plan::tests::host_dependency_derivations_do_not_guess_ambiguous_package_fallbacks ... ok
test rust_plan::tests::host_dependency_remap_preserves_explicit_non_target_producers ... ok
test rust_plan::tests::host_dependency_derivations_clone_libs_for_host_consumers_without_rewriting_targets ... ok
test rust_plan::tests::host_dependency_topology_accepts_proc_macro_host_producer ... ok
test rust_plan::tests::host_dependency_topology_accepts_target_lib_producer ... ok
test rust_plan::tests::host_dependency_topology_blocks_missing_producer ... ok
test rust_plan::tests::compiler_policy_required_mode_blocks_toolchain_mismatch ... ok
test rust_plan::tests::native_feature_resolver_exposes_bare_optional_dependency_as_cfg_feature ... ok
test rust_plan::tests::native_feature_resolver_blocks_malformed_feature_edges ... ok
test rust_plan::tests::native_feature_resolver_leaves_optional_dependency_unselected_without_feature ... ok
test rust_plan::tests::native_feature_resolver_reaches_fixed_point_for_defaults_explicit_and_optional_dependencies ... ok
test rust_plan::tests::native_feature_resolver_models_dependency_feature_edges_and_rejects_unknown_entries ... ok
test rust_plan::tests::native_feature_role_resolver_keeps_normal_build_and_host_features_separate ... ok
test rust_plan::tests::native_cargo_package_env_sets_version_components_and_empty_defaults ... ok
test rust_plan::tests::native_cargo_package_env_inherits_optional_workspace_metadata ... ok
test rust_plan::tests::native_git_dev_dependency_resolution_blocks_mismatched_revision ... ok
test rust_plan::tests::native_git_dependency_resolution_blocks_mismatched_revision ... ok
test rust_plan::tests::native_git_dependency_resolution_blocks_ambiguous_same_url_sources ... ok
test rust_plan::tests::native_host_dependencies_drop_build_script_artifacts_for_proc_macro_units ... ok
test rust_plan::tests::native_git_source_planning_blocks_missing_captured_manifest ... ok
test rust_plan::tests::native_host_dependencies_use_normal_deps_only_for_proc_macro_units ... ok
test rust_plan::tests::native_host_derivation_adds_compiler_proc_macro_extern ... ok
test rust_plan::tests::native_host_derivation_caps_lints_for_registry_source ... ok
test rust_plan::tests::native_git_source_planning_binds_captured_source_and_resolves_dependency_edge ... ok
test rust_plan::tests::native_host_derivation_carries_manifest_package_name_for_build_script_env ... ok
test rust_plan::tests::native_host_metadata_dependencies_follow_selected_target_artifacts_only ... ok
test rust_plan::tests::native_host_metadata_dependencies_ignore_unselected_linked_manifest_edges ... ok
test rust_plan::tests::native_host_planning_keeps_selected_same_package_build_script_for_proc_macro ... ok
test rust_plan::tests::native_host_planning_links_proc_macro_dependency_proc_macros_without_cargo_edges ... ok
test rust_plan::tests::native_host_planning_binds_run_custom_build_alias_to_exact_unique_build_unit ... ok
test rust_plan::tests::native_host_planning_does_not_bind_run_custom_build_alias_to_first_duplicate ... ok
test rust_plan::tests::native_host_planning_normalizes_proc_macro_target_names_and_follows_selected_units_only ... ok
test rust_plan::tests::native_host_planning_preserves_duplicate_selected_custom_build_unit_ids ... ok
test rust_plan::tests::native_host_planning_selects_lib_kind_proc_macro_crate_type ... ok
test rust_plan::tests::native_host_unit_derivation_emits_metadata_disambiguator ... ok
test rust_plan::tests::native_host_planning_preserves_duplicate_selected_proc_macro_unit_ids ... ok
test rust_plan::tests::native_manifest_lock_unsupported_blockers_accept_supported_subset ... ok
test rust_plan::tests::compiler_policy_required_mode_records_complete_static_identity ... ok
test rust_plan::tests::native_manifest_lock_unsupported_blockers_reject_patch_replace_target_build_and_unknown_lock_source ... ok
test rust_plan::tests::native_manifest_proc_macro_alias_feeds_target_planning ... ok
test rust_plan::tests::native_manifest_declared_edition_feeds_target_derivation_args ... ok
test rust_plan::tests::native_manifest_declared_edition_feeds_host_derivation_args ... ok
test rust_plan::tests::native_package_oracle_comparison_accepts_supported_fixture ... ok
test rust_plan::tests::native_manifest_package_build_path_and_false_control_targets ... ok
test rust_plan::tests::native_package_oracle_comparison_blocks_identity_mismatch_and_missing_cargo_package ... ok
test rust_plan::tests::native_manifest_missing_edition_uses_cargo_default ... ok
test rust_plan::tests::native_registry_source_planning_blocks_missing_vendor_material ... ok
test rust_plan::tests::native_target_cfg_predicate_scope_blocks_unknown_or_malformed_syntax ... ok
test rust_plan::tests::native_target_cfg_predicate_scope_evaluates_nested_common_predicates ... ok
test rust_plan::tests::native_unit_derivation_adds_selected_feature_cfg_args ... ok
test rust_plan::tests::native_package_target_fragment_ignores_out_of_scope_oracle_target_kinds ... ok
test rust_plan::tests::native_manifest_workspace_edition_feeds_target_derivation_args ... ok
test rust_plan::tests::native_unit_derivation_caps_lints_for_registry_and_git_sources ... ok
test rust_plan::tests::compiler_policy_required_mode_rejects_raw_rustc_cached_output ... ok
test rust_plan::tests::native_unit_derivation_emits_resolved_feature_closure_cfg_args ... ok
test rust_plan::tests::native_unit_derivation_emits_stable_metadata_disambiguator ... ok
test rust_plan::tests::native_unit_derivation_leaves_path_sources_uncapped ... ok
test rust_plan::tests::native_package_target_fragment_matches_supported_path_workspace ... ok
test rust_plan::tests::native_unit_graph_adds_build_dependency_producer_unit ... ok
test rust_plan::tests::native_registry_source_planning_binds_declared_vendor_source ... ok
test rust_plan::tests::native_unit_graph_blocks_native_dependency_without_package_fact ... ok
test rust_plan::tests::native_unit_graph_blocks_native_dependency_without_source_fact ... ok
test rust_plan::tests::native_unit_graph_filters_package_self_dependency_from_lib_unit ... ok
test rust_plan::tests::native_unit_graph_follows_native_dependency_facts_without_cargo_unit_edges ... ok
test rust_plan::tests::native_unit_graph_keeps_selected_lib_dependency_with_host_target_sibling ... ok
test rust_plan::tests::native_unit_graph_uses_lib_crate_name_when_package_name_differs ... ok
test rust_plan::tests::native_unit_graph_fragment_blocks_mismatch_and_missing_edges ... ok
test rust_plan::tests::native_unit_graph_adds_transitive_registry_producer_unit ... ok
test rust_plan::tests::native_unit_metadata_disambiguator_ignores_ambient_tool_roots ... ok
test rust_plan::tests::native_unit_graph_blocks_registry_dependency_without_lib_producer ... ok
test rust_plan::tests::native_unit_graph_uses_stable_native_unit_identity ... ok
test rust_plan::tests::non_aws_lc_guard_text_stays_plain_build_script_failure ... ok
test rust_plan::tests::package_only_dependency_artifacts_bind_single_renamed_crate_candidate ... ok
test rust_plan::tests::parse_build_script_metadata_accepts_bounded_link_lib_forms ... ok
test rust_plan::tests::package_only_dependency_artifacts_fail_on_ambiguous_unit_variants ... ok
test rust_plan::tests::parse_build_script_metadata_captures_link_metadata ... ok
test rust_plan::tests::parse_build_script_metadata_rejects_bad_custom_key ... ok
test rust_plan::tests::parse_build_script_metadata_rejects_unsafe_link_lib_forms ... ok
test rust_plan::tests::parse_native_lockfile_text_rejects_malformed_package_records ... ok
test rust_plan::tests::parse_native_lockfile_text_extracts_source_identities_revisions_checksums_and_edges ... ok
test rust_plan::tests::parse_native_manifest_text_rejects_invalid_inherited_package_version ... ok
test rust_plan::tests::parse_native_manifest_text_extracts_core_workspace_package_target_and_dependency_facts ... ok
test rust_plan::tests::proc_macro_crate_type_only_overrides_lib_shaped_targets ... ok
test rust_plan::tests::process_global_env_mutation_guard_detects_set_and_remove_var_calls ... ok
test rust_plan::tests::redacted_diagnostic_limits_lines_and_redacts_temp_paths ... ok
test rust_plan::tests::redacted_diagnostic_preserves_non_temp_context ... ok
test rust_plan::tests::native_unit_graph_fragment_feeds_supported_unit_derivations ... ok
test rust_plan::tests::registry_dependency_source_rejects_missing_exact_same_name_version ... ok
test rust_plan::tests::registry_dependency_source_uses_default_caret_semver_compatibility ... ok
test rust_plan::tests::path_source_digest_ignores_root_metadata_without_hiding_source_changes ... ok
test rust_plan::tests::registry_dependency_source_uses_exact_version_when_names_repeat ... ok
test rust_plan::tests::registry_dependency_source_uses_highest_matching_comparator_range ... ok
test rust_plan::tests::replay_evidence_json_rejects_malformed_and_oversized_inputs ... ok
test rust_plan::tests::replay_evidence_json_rejects_invalid_digest_fields ... ok
test rust_plan::tests::role_validation_rejects_host_unit_target_triple_mismatch ... ok
test rust_plan::tests::role_validation_rejects_host_consumer_bound_to_target_dependency ... ok
test rust_plan::tests::role_validation_rejects_source_package_mismatch ... ok
test rust_plan::tests::role_validation_rejects_metadata_hash_mismatch ... ok
test rust_plan::tests::role_validation_rejects_toolchain_policy_digest_mismatch ... ok
test rust_plan::tests::rust_topology_child_env_omits_empty_inherited_path ... ok
test rust_plan::tests::rust_topology_child_env_forwards_allowlisted_compile_env ... ok
test rust_plan::tests::rust_topology_child_env_prefers_explicit_compile_env ... ok
test rust_plan::tests::rust_topology_child_env_prefers_explicit_derivation_path ... ok
test rust_plan::tests::rust_topology_child_env_preserves_non_empty_inherited_path ... ok
test rust_plan::tests::rust_topology_child_env_rejects_ambient_compiler_guard_bypass_env ... ok
test rust_plan::tests::rust_topology_runtime_args_disable_self_contained_linker_by_default ... ok
test rust_plan::tests::rust_topology_runtime_args_ignore_near_match_linker_mode ... ok
test rust_plan::tests::rust_topology_runtime_args_preserve_joined_explicit_linker_mode ... ok
test rust_plan::tests::rust_topology_runtime_args_preserve_split_explicit_linker_mode ... ok
test rust_plan::tests::rust_plan_tests_do_not_mutate_process_global_environment_in_process ... ok
test rust_plan::tests::rust_unit_execution_blocks_missing_dependency_artifact_before_rustc ... ok
test rust_plan::tests::no_cargo_capture_blocks_missing_vendored_registry_source ... ok
test rust_plan::tests::rust_unit_execution_blocks_missing_declared_output_before_rustc ... ok
test rust_plan::tests::rustc_metadata_disambiguator_distinguishes_same_crate_package_versions ... ok
test rust_plan::tests::rust_unit_execution_blocks_source_closure_blocker_before_rustc ... ok
test rust_plan::tests::source_built_c_compiler_route_json_rejects_non_compiler_route ... ok
test rust_plan::tests::source_built_c_compiler_route_json_validates_receipt_bound_identity ... ok
test rust_plan::tests::source_closure_blocks_registry_without_checksum ... ok
test rust_plan::tests::selected_dependency_search_paths_follow_unit_variant_closure_only ... ok
test rust_plan::tests::target_dependency_producer_index_uses_selected_unit_variant ... ok
test rust_plan::tests::unit_dependency_artifacts_preserve_selected_cargo_unit_id ... ok
test rust_plan::tests::unit_dependency_artifacts_skip_host_producer_edges ... ok
test rust_plan::tests::no_cargo_capture_binds_captured_git_source ... ok
test rust_plan::tests::no_cargo_capture_binds_declared_vendored_registry_source ... ok
test rust_plan::tests::source_closure_records_registry_git_and_path_identities ... ok
test rust_plan::tests::unit_derivation_graph_blocks_doctest_or_non_build_modes ... ok
test rust_plan::tests::unit_derivation_graph_blocks_unsupported_target_kinds ... ok
test rust_plan::tests::unit_derivation_graph_emits_binary_with_dependency_artifact ... ok
test rust_plan::tests::unit_derivation_graph_binds_proc_macro_host_artifacts_to_target_units ... ok
test rust_plan::tests::unit_derivation_graph_represents_build_script_host_units ... ok
test rust_plan::tests::unit_graph_failure_fails_closed ... ok
test rust_plan::tests::rust_unit_execution_blocks_missing_source_before_rustc ... ok
test rust_plan::tests::rust_unit_execution_blocks_missing_host_artifact_before_rustc ... ok
test rust_plan::tests::executes_first_supported_lib_unit_from_derivation_graph ... ok
test rust_plan::tests::executes_dependency_chain_from_produced_lib_artifact ... ok

test result: ok. 189 passed; 0 failed; 0 ignored; 0 measured; 951 filtered out; finished in 0.05s

### parallel run 3
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.20s
     Running unittests src/main.rs (/tmp/mantle-rust-plan-validation-target/debug/deps/mantle-2027653048753230)

running 189 tests
test rust_plan::tests::aws_lc_memcmp_guard_failure_becomes_stable_compiler_guard_blocker ... ok
test rust_plan::tests::allowed_rust_topology_compile_env_rejects_unrelated_ambient_env ... ok
test rust_plan::tests::bind_build_script_metadata_adds_dep_env_for_linked_dependency ... ok
test rust_plan::tests::bind_all_host_artifacts_rewrites_custom_build_main_alias_placeholder ... ok
test rust_plan::tests::bind_all_host_artifacts_adds_proc_macro_extern_without_dependency_placeholder ... ok
test rust_plan::tests::bind_all_host_artifacts_leaves_unknown_custom_build_alias_placeholder ... ok
test rust_plan::tests::bind_all_host_artifacts_does_not_add_extern_for_custom_build_artifact ... ok
test rust_plan::tests::bind_all_host_artifacts_rewrites_existing_proc_macro_placeholder_without_duplicate ... ok
test rust_plan::tests::bind_all_host_artifacts_with_index_blocks_ambiguous_package_only_host_candidates ... ok
test rust_plan::tests::bind_build_script_metadata_with_index_blocks_ambiguous_package_only_metadata_candidates ... ok
test rust_plan::tests::bind_build_script_metadata_with_index_keeps_same_alias_duplicates_ambiguous ... ok
test rust_plan::tests::build_script_child_env_does_not_forward_compiler_guard_bypass_env ... ok
test rust_plan::tests::bind_target_unit_artifacts_adds_proc_macro_search_path_for_transitive_metadata ... ok
test rust_plan::tests::build_script_child_env_adds_c_prefix_map_flags_for_receipt_bound_gnu_compiler ... ok
test rust_plan::tests::build_script_child_env_omits_manifest_dir_without_source_arg ... ok
test rust_plan::tests::bind_dependency_artifacts_uses_selected_unit_variant ... ok
test rust_plan::tests::blocked_receipt_records_diagnostic_context_and_bounded_replay_evidence ... ok
test rust_plan::tests::build_script_profile_env_child_ignores_ambient_process_env_probe ... ok
test rust_plan::tests::build_script_metadata_producer_index_keeps_same_alias_duplicates_ambiguous ... ok
test rust_plan::tests::build_script_metadata_out_dir_is_scoped_by_output_root_and_unit_id ... ok
test rust_plan::tests::build_script_profile_env_derives_dev_and_release_defaults ... ok
test rust_plan::tests::build_script_child_env_sets_tool_target_and_manifest_package_name ... ok
test rust_plan::tests::build_script_target_cfg_env_derives_x86_64_linux_values ... ok
test rust_plan::tests::build_script_target_cfg_env_uses_empty_env_for_wasm_unknown ... ok
test rust_plan::tests::build_script_metadata_success_receipt_records_selected_compiler_route ... ok
test rust_plan::tests::combined_unit_topology_blocks_missing_linked_metadata_producer ... ok
test rust_plan::tests::c_prefix_map_flags_reject_unknown_compiler_family_and_empty_remaps ... ok
test rust_plan::tests::combined_unit_topology_blocks_missing_target_host_artifact_producer ... ok
test rust_plan::tests::combined_unit_topology_orders_host_build_script_before_same_package_proc_macro ... ok
test rust_plan::tests::combined_unit_topology_keeps_standalone_host_units ... ok
test rust_plan::tests::combined_unit_topology_orders_linked_metadata_before_dependent_build_script ... ok
test rust_plan::tests::combined_unit_topology_orders_host_dependency_lib_before_host_unit_not_target_lib ... ok
test rust_plan::tests::combined_unit_topology_orders_proc_macro_dependency_lib_before_host_unit ... ok
test rust_plan::tests::combined_unit_topology_orders_target_host_and_proc_macro_edges ... ok
test rust_plan::tests::cargo_unit_derivation_promotes_proc_macro_crate_type_to_host_unit ... ok
test rust_plan::tests::collect_native_manifest_lock_texts_fails_closed_on_missing_lockfile ... ok
test rust_plan::tests::collect_native_manifest_lock_texts_reads_root_member_manifests_and_lockfile ... ok
test rust_plan::tests::compiler_policy_report_path_is_scoped_by_output_root_and_unit_id ... ok
test rust_plan::tests::captures_normalized_oracle_receipt ... ok
test rust_plan::tests::build_script_child_env_ignores_ambient_profile_env ... ok
test rust_plan::tests::compiler_policy_expected_manifest_digest_mismatch_blocks_required_mode ... ok
test rust_plan::tests::compiler_policy_required_mode_blocks_missing_driver ... ok
test rust_plan::tests::compiler_policy_required_mode_blocks_missing_lint_library ... ok
test rust_plan::tests::compiler_policy_required_mode_blocks_missing_standards_artifact ... ok
test rust_plan::tests::compiler_policy_required_mode_blocks_policy_digest_mismatch ... ok
test rust_plan::tests::custom_build_manifest_dir_stays_real_in_deterministic_mode ... ok
test rust_plan::tests::compiler_policy_required_mode_blocks_toolchain_mismatch ... ok
test rust_plan::tests::deterministic_release_path_remaps_prefer_provider_toolchain_inside_source_root ... ok
test rust_plan::tests::dependency_chain_blocks_missing_producer_before_consumer_rustc ... ok
test rust_plan::tests::deterministic_release_paths_add_remaps_and_provider_placeholder_env ... ok
test rust_plan::tests::append_all_produced_dependency_search_paths_keeps_duplicate_package_variant_dirs ... ok
test rust_plan::tests::finalized_receipt_records_role_triple_policy_and_identity ... ok
test rust_plan::tests::host_dependency_derivations_clone_libs_for_host_consumers_without_rewriting_targets ... ok
test rust_plan::tests::host_dependency_derivations_do_not_guess_ambiguous_package_fallbacks ... ok
test rust_plan::tests::host_dependency_remap_preserves_explicit_non_target_producers ... ok
test rust_plan::tests::host_dependency_topology_accepts_proc_macro_host_producer ... ok
test rust_plan::tests::host_dependency_topology_accepts_target_lib_producer ... ok
test rust_plan::tests::host_dependency_topology_blocks_missing_producer ... ok
test rust_plan::tests::native_cargo_package_env_inherits_optional_workspace_metadata ... ok
test rust_plan::tests::compiler_policy_json_receipt_contains_identity_and_waiver_summary ... ok
test rust_plan::tests::compiler_policy_audit_mode_records_adapter_identity_and_waivers ... ok
test rust_plan::tests::native_feature_resolver_exposes_bare_optional_dependency_as_cfg_feature ... ok
test rust_plan::tests::native_feature_resolver_blocks_malformed_feature_edges ... ok
test rust_plan::tests::native_feature_resolver_leaves_optional_dependency_unselected_without_feature ... ok
test rust_plan::tests::native_cargo_package_env_sets_version_components_and_empty_defaults ... ok
test rust_plan::tests::native_feature_resolver_models_dependency_feature_edges_and_rejects_unknown_entries ... ok
test rust_plan::tests::native_feature_resolver_reaches_fixed_point_for_defaults_explicit_and_optional_dependencies ... ok
test rust_plan::tests::native_feature_role_resolver_keeps_normal_build_and_host_features_separate ... ok
test rust_plan::tests::native_git_dev_dependency_resolution_blocks_mismatched_revision ... ok
test rust_plan::tests::native_git_dependency_resolution_blocks_mismatched_revision ... ok
test rust_plan::tests::native_git_dependency_resolution_blocks_ambiguous_same_url_sources ... ok
test rust_plan::tests::native_host_dependencies_drop_build_script_artifacts_for_proc_macro_units ... ok
test rust_plan::tests::native_git_source_planning_blocks_missing_captured_manifest ... ok
test rust_plan::tests::native_host_dependencies_use_normal_deps_only_for_proc_macro_units ... ok
test rust_plan::tests::native_host_derivation_adds_compiler_proc_macro_extern ... ok
test rust_plan::tests::native_host_derivation_caps_lints_for_registry_source ... ok
test rust_plan::tests::native_host_metadata_dependencies_follow_selected_target_artifacts_only ... ok
test rust_plan::tests::native_host_metadata_dependencies_ignore_unselected_linked_manifest_edges ... ok
test rust_plan::tests::native_host_derivation_carries_manifest_package_name_for_build_script_env ... ok
test rust_plan::tests::compiler_policy_required_mode_records_complete_static_identity ... ok
test rust_plan::tests::native_git_source_planning_binds_captured_source_and_resolves_dependency_edge ... ok
test rust_plan::tests::native_host_planning_binds_run_custom_build_alias_to_exact_unique_build_unit ... ok
test rust_plan::tests::compiler_policy_deny_mode_rejects_adapter_failure ... ok
test rust_plan::tests::native_host_planning_links_proc_macro_dependency_proc_macros_without_cargo_edges ... ok
test rust_plan::tests::native_host_planning_does_not_bind_run_custom_build_alias_to_first_duplicate ... ok
test rust_plan::tests::native_host_planning_keeps_selected_same_package_build_script_for_proc_macro ... ok
test rust_plan::tests::native_host_unit_derivation_emits_metadata_disambiguator ... ok
test rust_plan::tests::compiler_policy_required_mode_rejects_raw_rustc_cached_output ... ok
test rust_plan::tests::native_host_planning_selects_lib_kind_proc_macro_crate_type ... ok
test rust_plan::tests::native_host_planning_preserves_duplicate_selected_proc_macro_unit_ids ... ok
test rust_plan::tests::native_host_planning_normalizes_proc_macro_target_names_and_follows_selected_units_only ... ok
test rust_plan::tests::native_manifest_lock_unsupported_blockers_accept_supported_subset ... ok
test rust_plan::tests::native_host_planning_preserves_duplicate_selected_custom_build_unit_ids ... ok
test rust_plan::tests::native_manifest_lock_unsupported_blockers_reject_patch_replace_target_build_and_unknown_lock_source ... ok
test rust_plan::tests::native_manifest_declared_edition_feeds_target_derivation_args ... ok
test rust_plan::tests::native_package_oracle_comparison_accepts_supported_fixture ... ok
test rust_plan::tests::native_manifest_declared_edition_feeds_host_derivation_args ... ok
test rust_plan::tests::native_package_oracle_comparison_blocks_identity_mismatch_and_missing_cargo_package ... ok
test rust_plan::tests::native_manifest_proc_macro_alias_feeds_target_planning ... ok
test rust_plan::tests::native_manifest_package_build_path_and_false_control_targets ... ok
test rust_plan::tests::native_manifest_missing_edition_uses_cargo_default ... ok
test rust_plan::tests::native_target_cfg_predicate_scope_blocks_unknown_or_malformed_syntax ... ok
test rust_plan::tests::native_target_cfg_predicate_scope_evaluates_nested_common_predicates ... ok
test rust_plan::tests::native_registry_source_planning_blocks_missing_vendor_material ... ok
test rust_plan::tests::native_package_target_fragment_ignores_out_of_scope_oracle_target_kinds ... ok
test rust_plan::tests::native_unit_derivation_adds_selected_feature_cfg_args ... ok
test rust_plan::tests::native_unit_derivation_caps_lints_for_registry_and_git_sources ... ok
test rust_plan::tests::native_unit_derivation_emits_resolved_feature_closure_cfg_args ... ok
test rust_plan::tests::native_manifest_workspace_edition_feeds_target_derivation_args ... ok
test rust_plan::tests::native_unit_derivation_emits_stable_metadata_disambiguator ... ok
test rust_plan::tests::native_unit_derivation_leaves_path_sources_uncapped ... ok
test rust_plan::tests::native_unit_graph_adds_build_dependency_producer_unit ... ok
test rust_plan::tests::native_unit_graph_blocks_native_dependency_without_package_fact ... ok
test rust_plan::tests::native_unit_graph_blocks_native_dependency_without_source_fact ... ok
test rust_plan::tests::native_package_target_fragment_matches_supported_path_workspace ... ok
test rust_plan::tests::native_registry_source_planning_binds_declared_vendor_source ... ok
test rust_plan::tests::native_unit_graph_filters_package_self_dependency_from_lib_unit ... ok
test rust_plan::tests::native_unit_graph_follows_native_dependency_facts_without_cargo_unit_edges ... ok
test rust_plan::tests::native_unit_graph_keeps_selected_lib_dependency_with_host_target_sibling ... ok
test rust_plan::tests::native_unit_graph_uses_lib_crate_name_when_package_name_differs ... ok
test rust_plan::tests::native_unit_metadata_disambiguator_ignores_ambient_tool_roots ... ok
test rust_plan::tests::native_unit_graph_fragment_blocks_mismatch_and_missing_edges ... ok
test rust_plan::tests::native_unit_graph_adds_transitive_registry_producer_unit ... ok
test rust_plan::tests::native_unit_graph_uses_stable_native_unit_identity ... ok
test rust_plan::tests::non_aws_lc_guard_text_stays_plain_build_script_failure ... ok
test rust_plan::tests::native_unit_graph_blocks_registry_dependency_without_lib_producer ... ok
test rust_plan::tests::package_only_dependency_artifacts_bind_single_renamed_crate_candidate ... ok
test rust_plan::tests::parse_build_script_metadata_accepts_bounded_link_lib_forms ... ok
test rust_plan::tests::package_only_dependency_artifacts_fail_on_ambiguous_unit_variants ... ok
test rust_plan::tests::parse_build_script_metadata_captures_link_metadata ... ok
test rust_plan::tests::parse_build_script_metadata_rejects_bad_custom_key ... ok
test rust_plan::tests::parse_build_script_metadata_rejects_unsafe_link_lib_forms ... ok
test rust_plan::tests::native_unit_graph_fragment_feeds_supported_unit_derivations ... ok
test rust_plan::tests::parse_native_lockfile_text_rejects_malformed_package_records ... ok
test rust_plan::tests::parse_native_manifest_text_rejects_invalid_inherited_package_version ... ok
test rust_plan::tests::parse_native_lockfile_text_extracts_source_identities_revisions_checksums_and_edges ... ok
test rust_plan::tests::proc_macro_crate_type_only_overrides_lib_shaped_targets ... ok
test rust_plan::tests::process_global_env_mutation_guard_detects_set_and_remove_var_calls ... ok
test rust_plan::tests::parse_native_manifest_text_extracts_core_workspace_package_target_and_dependency_facts ... ok
test rust_plan::tests::redacted_diagnostic_limits_lines_and_redacts_temp_paths ... ok
test rust_plan::tests::redacted_diagnostic_preserves_non_temp_context ... ok
test rust_plan::tests::registry_dependency_source_rejects_missing_exact_same_name_version ... ok
test rust_plan::tests::registry_dependency_source_uses_default_caret_semver_compatibility ... ok
test rust_plan::tests::path_source_digest_ignores_root_metadata_without_hiding_source_changes ... ok
test rust_plan::tests::registry_dependency_source_uses_exact_version_when_names_repeat ... ok
test rust_plan::tests::registry_dependency_source_uses_highest_matching_comparator_range ... ok
test rust_plan::tests::replay_evidence_json_rejects_malformed_and_oversized_inputs ... ok
test rust_plan::tests::role_validation_rejects_host_consumer_bound_to_target_dependency ... ok
test rust_plan::tests::role_validation_rejects_host_unit_target_triple_mismatch ... ok
test rust_plan::tests::replay_evidence_json_rejects_invalid_digest_fields ... ok
test rust_plan::tests::role_validation_rejects_metadata_hash_mismatch ... ok
test rust_plan::tests::role_validation_rejects_source_package_mismatch ... ok
test rust_plan::tests::role_validation_rejects_toolchain_policy_digest_mismatch ... ok
test rust_plan::tests::rust_topology_child_env_omits_empty_inherited_path ... ok
test rust_plan::tests::rust_topology_child_env_forwards_allowlisted_compile_env ... ok
test rust_plan::tests::rust_topology_child_env_prefers_explicit_compile_env ... ok
test rust_plan::tests::rust_topology_child_env_prefers_explicit_derivation_path ... ok
test rust_plan::tests::rust_topology_child_env_preserves_non_empty_inherited_path ... ok
test rust_plan::tests::rust_topology_child_env_rejects_ambient_compiler_guard_bypass_env ... ok
test rust_plan::tests::rust_topology_runtime_args_disable_self_contained_linker_by_default ... ok
test rust_plan::tests::rust_topology_runtime_args_ignore_near_match_linker_mode ... ok
test rust_plan::tests::rust_topology_runtime_args_preserve_joined_explicit_linker_mode ... ok
test rust_plan::tests::rust_topology_runtime_args_preserve_split_explicit_linker_mode ... ok
test rust_plan::tests::rust_plan_tests_do_not_mutate_process_global_environment_in_process ... ok
test rust_plan::tests::rust_unit_execution_blocks_missing_declared_output_before_rustc ... ok
test rust_plan::tests::no_cargo_capture_blocks_missing_vendored_registry_source ... ok
test rust_plan::tests::rust_unit_execution_blocks_missing_dependency_artifact_before_rustc ... ok
test rust_plan::tests::no_cargo_capture_binds_captured_git_source ... ok
test rust_plan::tests::no_cargo_capture_binds_declared_vendored_registry_source ... ok
test rust_plan::tests::rustc_metadata_disambiguator_distinguishes_same_crate_package_versions ... ok
test rust_plan::tests::source_built_c_compiler_route_json_rejects_non_compiler_route ... ok
test rust_plan::tests::source_built_c_compiler_route_json_validates_receipt_bound_identity ... ok
test rust_plan::tests::source_closure_blocks_registry_without_checksum ... ok
test rust_plan::tests::target_dependency_producer_index_uses_selected_unit_variant ... ok
test rust_plan::tests::rust_unit_execution_blocks_source_closure_blocker_before_rustc ... ok
test rust_plan::tests::unit_dependency_artifacts_preserve_selected_cargo_unit_id ... ok
test rust_plan::tests::unit_dependency_artifacts_skip_host_producer_edges ... ok
test rust_plan::tests::selected_dependency_search_paths_follow_unit_variant_closure_only ... ok
test rust_plan::tests::source_closure_records_registry_git_and_path_identities ... ok
test rust_plan::tests::unit_derivation_graph_blocks_doctest_or_non_build_modes ... ok
test rust_plan::tests::unit_derivation_graph_blocks_unsupported_target_kinds ... ok
test rust_plan::tests::unit_derivation_graph_binds_proc_macro_host_artifacts_to_target_units ... ok
test rust_plan::tests::unit_derivation_graph_represents_build_script_host_units ... ok
test rust_plan::tests::unit_graph_failure_fails_closed ... ok
test rust_plan::tests::unit_derivation_graph_emits_binary_with_dependency_artifact ... ok
test rust_plan::tests::rust_unit_execution_blocks_missing_source_before_rustc ... ok
test rust_plan::tests::rust_unit_execution_blocks_missing_host_artifact_before_rustc ... ok
test rust_plan::tests::executes_first_supported_lib_unit_from_derivation_graph ... ok
test rust_plan::tests::executes_dependency_chain_from_produced_lib_artifact ... ok

test result: ok. 189 passed; 0 failed; 0 ignored; 0 measured; 951 filtered out; finished in 0.05s

```

## Negative ambient-env rail

```text
$ cargo test -p mantle --bin mantle rust_plan::tests::build_script_child_env_ignores_ambient_profile_env -- --exact --nocapture
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.19s
     Running unittests src/main.rs (/tmp/mantle-rust-plan-validation-target/debug/deps/mantle-2027653048753230)

running 1 test
test rust_plan::tests::build_script_child_env_ignores_ambient_profile_env ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1139 filtered out; finished in 0.00s

$ cargo test -p mantle --bin mantle rust_plan::tests::rust_plan_tests_do_not_mutate_process_global_environment_in_process -- --exact --nocapture
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.20s
     Running unittests src/main.rs (/tmp/mantle-rust-plan-validation-target/debug/deps/mantle-2027653048753230)

running 1 test
test rust_plan::tests::rust_plan_tests_do_not_mutate_process_global_environment_in_process ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1139 filtered out; finished in 0.01s

$ cargo test -p mantle --bin mantle rust_plan::tests::process_global_env_mutation_guard_detects_set_and_remove_var_calls -- --exact --nocapture
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.21s
     Running unittests src/main.rs (/tmp/mantle-rust-plan-validation-target/debug/deps/mantle-2027653048753230)

running 1 test
test rust_plan::tests::process_global_env_mutation_guard_detects_set_and_remove_var_calls ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1139 filtered out; finished in 0.00s

```

## Formatting and Cairn gates

```text
$ cargo fmt -p mantle --check
$ git diff --check
$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 3,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 19,
  "valid": true
}
$ nix run path:/home/brittonr/git/cairn#cairn -- gate proposal stabilize-native-rust-plan-validation --root .
{
  "change": "stabilize-native-rust-plan-validation",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "3352d4adfb2cd25851117f036618af624c916b7064215d678c4953b71fadcc71",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "c93a15c80df4ca908fb6a6a3b7cb24edf90e97f262fd816a72965cd45947ea86",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
$ nix run path:/home/brittonr/git/cairn#cairn -- gate design stabilize-native-rust-plan-validation --root .
{
  "change": "stabilize-native-rust-plan-validation",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "c69caf9d9e46cf671e66d945c7469f13f7337aa1dcae1918c494049844cc6f7b",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "4046a53e8184f2c273ea9f4cc5280db6404a1cbee957a4931454f34f44a2224b",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks stabilize-native-rust-plan-validation --root .
{
  "change": "stabilize-native-rust-plan-validation",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "bf5b9d7d6a90fbb23c6830b8fcb00d46d9814e034085075fa303d224efb45ca9",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "628dc8f168a00ad6530d7b0f89dc6ed0b159080d670830c0fe75cfbe00d98403",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## Accepted spec sync repair

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- sync stabilize-native-rust-plan-validation --root . --execute
Executed before manual repair; dry-run/execute reported no blockers but did not materialize the full-spec-shaped requirement IDs.
$ rg "native_topology_validation_stability|native_topology_race_free_fixtures|native_topology_parallel_evidence" cairn/specs/rust-package-planning/spec.md
r[rust_package_planning.native_topology_validation_stability] Native Rust topology validation SHOULD provide focused commands that pass deterministically under documented host tooling and do not rely on hidden fixture ordering.
r[rust_package_planning.native_topology_race_free_fixtures] Native rust-plan tests MUST isolate or explicitly lock shared fixture state such as rustc wrappers, cargo shims, OUT_DIRs, execution roots, compiler-policy files, and ambient environment probes.
r[rust_package_planning.native_topology_parallel_evidence] Mantle validation evidence MUST distinguish serial-only, parallel-safe, stress-tested, and remaining-blocked native rust-plan rails.
$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 3,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 19,
  "valid": true
}
$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks stabilize-native-rust-plan-validation --root .
{
  "change": "stabilize-native-rust-plan-validation",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "18be8e44fd036063c5c64621ad31df7765e87ce6a48849a2dad4687db31c8e48",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "60037f2a3daa7d9c1c15296422431585abe7f7ad4c8073fc805dd96d34824ced",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## Archive execution and post-archive validation

```text
$ CAIRN_ARCHIVE_DATE=2026-07-03 nix run path:/home/brittonr/git/cairn#cairn -- archive stabilize-native-rust-plan-validation --root . --execute
Executed successfully; change moved to cairn/archive/2026-07-03-stabilize-native-rust-plan-validation.
$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 2,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 18,
  "valid": true
}
```
