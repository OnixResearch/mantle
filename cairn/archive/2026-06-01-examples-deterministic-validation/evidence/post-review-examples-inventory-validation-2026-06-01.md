# Post-review examples inventory validation

Purpose: make the previously summarized `examples_inventory: 7 passed` claim reviewable with a fresh current-code Cargo transcript.

Command environment:

```text
PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/27ddg2m33wnp6xrr7il86yjyr41n88ya-mold-2.40.4/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:/run/current-system/sw/bin:/usr/bin:/bin
CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=cc
PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig
SNIX_BUILD_SANDBOX_SHELL=/nix/store/d7fc5i7y71rj8cr5jwmaxwjnyvfiybdp-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox
cargo=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/cargo
rustc=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustc
cc=/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin/cc
mold=/nix/store/27ddg2m33wnp6xrr7il86yjyr41n88ya-mold-2.40.4/bin/mold
cargo 1.97.0-nightly (06ac0e7c0 2026-04-21)
rustc 1.97.0-nightly (913e4bea8 2026-04-22)
```

## cargo test -p mantle --test examples_inventory -- --nocapture

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling proc-macro2 v1.0.106
   Compiling quote v1.0.45
   Compiling unicode-ident v1.0.24
   Compiling serde_core v1.0.228
   Compiling serde v1.0.228
   Compiling cfg-if v1.0.4
   Compiling memchr v2.8.0
   Compiling libc v0.2.184
   Compiling regex-syntax v0.8.10
   Compiling smallvec v1.15.1
   Compiling scopeguard v1.2.0
   Compiling parking_lot_core v0.9.12
   Compiling equivalent v1.0.2
   Compiling version_check v0.9.5
   Compiling itoa v1.0.18
   Compiling once_cell v1.21.4
   Compiling typenum v1.19.0
   Compiling allocator-api2 v0.2.21
   Compiling subtle v2.6.1
   Compiling thiserror v2.0.18
   Compiling shlex v1.3.0
   Compiling find-msvc-tools v0.1.9
   Compiling crossbeam-utils v0.8.21
   Compiling stable_deref_trait v1.2.1
   Compiling lock_api v0.4.14
   Compiling bitflags v2.11.0
   Compiling foldhash v0.2.0
   Compiling same-file v1.0.6
   Compiling fastrand v2.4.1
   Compiling crc32fast v1.5.0
   Compiling tinyvec_macros v0.1.1
   Compiling generic-array v0.14.7
   Compiling gix-trace v0.1.18
   Compiling byteorder v1.5.0
   Compiling cpufeatures v0.2.17
   Compiling aho-corasick v1.1.4
   Compiling pin-project-lite v0.2.17
   Compiling fnv v1.0.7
   Compiling hashbrown v0.16.1
   Compiling tinyvec v1.11.0
   Compiling human_format v1.2.1
   Compiling bytesize v2.3.1
   Compiling walkdir v2.5.0
   Compiling futures-core v0.3.32
   Compiling zlib-rs v0.6.3
   Compiling jobserver v0.1.34
   Compiling winnow v0.7.15
   Compiling heapless v0.8.0
   Compiling crossbeam-channel v0.5.15
   Compiling hash32 v0.3.1
   Compiling rustix v1.1.4
   Compiling linux-raw-sys v0.12.1
   Compiling errno v0.3.14
   Compiling socket2 v0.6.3
   Compiling mio v1.2.0
   Compiling zmij v1.0.21
   Compiling serde_json v1.0.149
   Compiling rand_core v0.10.0
   Compiling getrandom v0.4.2
   Compiling block-buffer v0.10.4
   Compiling cc v1.2.59
   Compiling crypto-common v0.1.7
   Compiling unicode-normalization v0.1.25
   Compiling syn v2.0.117
   Compiling jiff v0.2.23
   Compiling regex-automata v0.4.14
   Compiling futures-io v0.3.32
   Compiling futures-sink v0.3.32
   Compiling futures-task v0.3.32
   Compiling signal-hook-registry v1.4.8
   Compiling parking_lot v0.12.5
   Compiling zeroize v1.8.2
   Compiling slab v0.4.12
   Compiling digest v0.10.7
   Compiling indexmap v2.13.1
   Compiling rustversion v1.0.22
   Compiling futures-channel v0.3.32
   Compiling dunce v1.0.5
   Compiling strsim v0.11.1
   Compiling fs_extra v1.3.0
   Compiling faster-hex v0.10.0
   Compiling log v0.4.29
   Compiling prodash v31.0.0
   Compiling tracing-core v0.1.36
   Compiling cmake v0.1.58
   Compiling foldhash v0.1.5
   Compiling sha1 v0.10.6
   Compiling semver v1.0.28
   Compiling getrandom v0.2.17
   Compiling percent-encoding v2.3.2
   Compiling ring v0.17.14
   Compiling memmap2 v0.9.10
   Compiling aws-lc-rs v1.16.2
   Compiling arrayvec v0.7.6
   Compiling either v1.15.0
   Compiling rustls-pki-types v1.14.0
   Compiling base64 v0.22.1
   Compiling hashbrown v0.15.5
   Compiling aws-lc-sys v0.39.1
   Compiling sha1-checked v0.10.0
   Compiling rustc_version v0.4.1
   Compiling untrusted v0.9.0
   Compiling litemap v0.8.2
   Compiling httparse v1.10.1
   Compiling writeable v0.6.3
   Compiling pkg-config v0.3.32
   Compiling icu_normalizer_data v2.2.0
   Compiling signal-hook v0.4.4
   Compiling icu_properties_data v2.2.0
   Compiling utf8_iter v1.0.4
   Compiling rustls v0.23.37
   Compiling tempfile v3.27.0
   Compiling encoding_rs v0.8.35
   Compiling hashbrown v0.14.5
   Compiling ident_case v1.0.1
   Compiling static_assertions v1.1.0
   Compiling nonempty v0.12.0
   Compiling unicode-xid v0.2.6
   Compiling try-lock v0.2.5
   Compiling ryu v1.0.23
   Compiling tower-service v0.3.3
   Compiling autocfg v1.5.0
   Compiling atomic-waker v1.1.2
   Compiling rayon-core v1.13.0
   Compiling synstructure v0.13.2
   Compiling serde_derive v1.0.228
   Compiling thiserror-impl v2.0.18
   Compiling tokio-macros v2.7.0
   Compiling futures-macro v0.3.32
   Compiling zerovec-derive v0.11.3
   Compiling displaydoc v0.2.5
   Compiling tracing-attributes v0.1.31
   Compiling want v0.3.1
   Compiling dashmap v6.1.0
   Compiling unicode-width v0.2.2
   Compiling lazy_static v1.5.0
   Compiling openssl-probe v0.2.1
   Compiling form_urlencoded v1.2.2
   Compiling sync_wrapper v1.0.2
   Compiling crossbeam-epoch v0.9.18
   Compiling tower-layer v0.3.3
   Compiling futures-util v0.3.32
   Compiling ipnet v2.12.0
   Compiling itertools v0.14.0
   Compiling blake3 v1.8.2
   Compiling filetime v0.2.27
   Compiling zerofrom-derive v0.1.7
   Compiling yoke-derive v0.8.2
   Compiling tracing v0.1.44
   Compiling rustls-native-certs v0.8.3
   Compiling iri-string v0.7.12
   Compiling unicode-bom v2.0.3
   Compiling constant_time_eq v0.3.1
   Compiling arrayref v0.3.9
   Compiling kstring v2.0.2
   Compiling colorchoice v1.0.5
   Compiling crossbeam-deque v0.8.6
   Compiling unicode-segmentation v1.13.2
   Compiling shell-words v1.1.1
   Compiling zerofrom v0.1.7
   Compiling num-traits v0.2.19
   Compiling mime v0.3.17
   Compiling anstyle v1.0.14
   Compiling cfg_aliases v0.2.1
   Compiling darling_core v0.23.0
   Compiling zstd-sys v2.0.16+zstd.1.5.7
   Compiling heck v0.5.0
   Compiling utf8parse v0.2.2
   Compiling data-encoding v2.10.0
   Compiling portable-atomic v1.13.1
   Compiling yoke v0.8.2
   Compiling is_terminal_polyfill v1.70.2
   Compiling anstyle-query v1.1.5
   Compiling siphasher v1.0.2
   Compiling winnow v1.0.1
   Compiling anyhow v1.0.102
   Compiling anstyle-parse v1.0.0
   Compiling bytes v1.11.1
   Compiling bstr v1.12.1
   Compiling zerovec v0.11.6
   Compiling zerotrie v0.2.4
   Compiling libm v0.2.16
   Compiling convert_case v0.10.0
   Compiling phf_shared v0.11.3
   Compiling anstream v1.0.0
   Compiling memoffset v0.6.5
   Compiling regex v1.12.3
   Compiling async-trait v0.1.89
   Compiling lzma-sys v0.1.20
   Compiling darling_macro v0.23.0
   Compiling bzip2-sys v0.1.13+1.0.8
   Compiling vte v0.15.0
   Compiling sha2 v0.10.9
   Compiling tokio v1.51.0
   Compiling http v1.4.0
   Compiling libmimalloc-sys v0.1.44
   Compiling tinystr v0.8.3
   Compiling potential_utf v0.1.5
   Compiling gix-sec v0.13.2
   Compiling gix-validate v0.11.0
   Compiling gix-utils v0.3.1
   Compiling gix-error v0.2.1
   Compiling gix-packetline v0.21.2
   Compiling toml_parser v1.1.2+spec-1.1.0
   Compiling darling v0.23.0
   Compiling fixedbitset v0.5.7
   Compiling bytemuck v1.25.0
   Compiling keccak v0.1.6
   Compiling bit-vec v0.8.0
   Compiling toml_datetime v1.1.1+spec-1.1.0
   Compiling icu_collections v2.2.0
   Compiling icu_locale_core v2.2.0
   Compiling typeid v1.0.3
   Compiling precomputed-hash v0.1.1
   Compiling gix-date v0.15.1
   Compiling gix-path v0.11.2
   Compiling gix-chunk v0.7.0
   Compiling gix-quote v0.7.0
   Compiling term v1.2.1
   Compiling zstd-safe v7.2.4
   Compiling clap_lex v1.1.0
   Compiling new_debug_unreachable v1.0.6
   Compiling sha3 v0.10.8
   Compiling ena v0.14.4
   Compiling http-body v1.0.1
   Compiling bit-set v0.8.0
   Compiling petgraph v0.7.1
   Compiling toml_edit v0.25.10+spec-1.1.0
   Compiling safe_arch v0.7.4
   Compiling gix-bitmap v0.3.0
   Compiling serde_with_macros v3.18.0
   Compiling string_cache v0.8.9
   Compiling vt100 v0.16.2
   Compiling clap_builder v4.6.0
   Compiling gix-features v0.46.2
   Compiling gix-actor v0.40.0
   Compiling gix-command v0.8.0
   Compiling gix-config-value v0.17.1
   Compiling http-body-util v0.1.3
   Compiling ascii-canvas v4.0.0
   Compiling icu_provider v2.2.0
   Compiling derive_more-impl v2.1.1
   Compiling lalrpop-util v0.22.2
   Compiling futures-executor v0.3.32
   Compiling clap_derive v4.6.0
   Compiling n0-future v0.3.2
   Compiling gix-hash v0.23.0
   Compiling gix-fs v0.19.2
   Compiling gix-glob v0.24.0
   Compiling icu_properties v2.2.0
   Compiling icu_normalizer v2.2.0
   Compiling console v0.16.3
   Compiling sharded-slab v0.1.7
   Compiling matchers v0.2.0
   Compiling pin-project-internal v1.1.11
   Compiling heapless v0.7.17
   Compiling curve25519-dalek v4.1.3
   Compiling tracing-log v0.2.0
   Compiling gix-hashtable v0.13.0
   Compiling gix-tempfile v21.0.2
   Compiling gix-commitgraph v0.35.0
   Compiling gix-attributes v0.31.0
   Compiling arc-swap v1.9.1
   Compiling thread_local v1.1.9
   Compiling thiserror v1.0.69
   Compiling nu-ansi-term v0.50.3
   Compiling parking v2.2.1
   Compiling simd-adler32 v0.3.9
   Compiling erased-serde v0.4.10
   Compiling cpufeatures v0.3.0
   Compiling malachite-nz v0.6.1
   Compiling cordyceps v0.3.4
   Compiling gix-object v0.58.0
   Compiling adler2 v2.0.1
   Compiling pico-args v0.5.0
   Compiling spin v0.10.0
   Compiling gix-lock v21.0.2
   Compiling idna_adapter v1.2.1
   Compiling diatomic-waker v0.2.3
   Compiling bitflags v1.3.2
   Compiling unit-prefix v0.5.2
   Compiling chacha20 v0.10.0
   Compiling futures-lite v2.6.1
   Compiling clap v4.6.0
   Compiling tracing-subscriber v0.3.23
   Compiling malachite-base v0.6.1
   Compiling tokio-util v0.7.18
   Compiling tower v0.5.3
   Compiling miniz_oxide v0.8.9
   Compiling tokio-stream v0.1.18
   Compiling lalrpop v0.22.2
   Compiling pin-project v1.1.11
   Compiling idna v1.1.0
   Compiling futures-buffered v0.2.13
   Compiling indicatif v0.18.4
   Compiling gix-ignore v0.19.1
   Compiling futures v0.3.32
   Compiling derive_more v2.1.1
   Compiling proc-macro-crate v3.5.0
   Compiling gix-revwalk v0.29.0
   Compiling gix-filter v0.28.0
   Compiling gix-ref v0.61.0
   Compiling tower-http v0.6.8
   Compiling wide v0.7.33
   Compiling serde_with v3.18.0
   Compiling url v2.5.8
   Compiling serde_urlencoded v0.7.1
   Compiling h2 v0.4.13
   Compiling n0-error-macros v0.1.3
   Compiling curve25519-dalek-derive v0.1.1
   Compiling thiserror-impl v1.0.69
   Compiling async-stream-impl v0.3.6
   Compiling gix-traverse v0.55.0
   Compiling spez v0.1.2
   Compiling darling_core v0.20.11
   Compiling logos-codegen v0.15.1
   Compiling xattr v1.6.1
   Compiling hash32 v0.2.1
   Compiling spin v0.9.8
   Compiling proc-macro2-diagnostics v0.10.1
   Compiling nibble_vec v0.1.0
   Compiling fuse-backend-rs v0.12.0 (/home/brittonr/git/mantle/vendor/fuse-backend-rs)
   Compiling iana-time-zone v0.1.65
   Compiling endian-type v0.1.2
   Compiling yansi v1.0.1
   Compiling signature v2.2.0
   Compiling redb v3.1.3
   Compiling matchit v0.8.4
   Compiling gix-index v0.49.0
   Compiling zstd v0.13.3
   Compiling n0-error v0.1.3
   Compiling async-stream v0.3.6
   Compiling mimalloc v0.1.48
   Compiling ed25519 v2.2.3
   Compiling radix_trie v0.2.1
   Compiling chrono v0.4.44
   Compiling tracing-indicatif v0.3.14
   Compiling num_enum_derive v0.7.6
   Compiling nix v0.24.3
   Compiling bzip2 v0.5.2
   Compiling flate2 v1.1.9
   Compiling rand v0.10.1
   Compiling darling_macro v0.20.11
   Compiling clap-verbosity-flag v3.0.4
   Compiling vmm-sys-util v0.11.2
   Compiling gix-pathspec v0.16.1
   Compiling crunch-attestation-core v0.1.0 (/home/brittonr/git/mantle/crates/crunch-attestation-core)
   Compiling quick-xml v0.39.2
   Compiling cobs v0.3.0
   Compiling irpc-derive v0.10.0
   Compiling gix-worktree v0.50.0
   Compiling mio v0.8.11
   Compiling hyper v1.9.0
   Compiling md-5 v0.10.6
   Compiling vm-memory v0.10.0
   Compiling num_cpus v1.17.0
   Compiling caps v0.5.6
   Compiling nix-compat v0.1.0 (/home/brittonr/git/mantle/vendor/nix-compat)
   Compiling beef v0.5.2
   Compiling snix-castore v0.1.0 (/home/brittonr/git/mantle/vendor/snix-castore)
   Compiling humantime v2.3.0
   Compiling rustc-hash v2.1.2
   Compiling fixedbitset v0.4.2
   Compiling postcard v1.1.3
   Compiling irpc v0.13.0
   Compiling ed25519-dalek v2.2.0
   Compiling darling v0.20.11
   Compiling threadpool v1.8.1
   Compiling serde_tagged v0.3.0
   Compiling num_enum v0.7.6
   Compiling fastcdc v3.2.1
   Compiling xz2 v0.1.7
   Compiling serde_qs v0.12.0
   Compiling astral-tokio-tar v0.6.0
   Compiling petgraph v0.6.5
   Compiling gix-url v0.35.2
   Compiling nix-compat-derive v0.1.0 (/home/brittonr/git/mantle/vendor/nix-compat-derive)
   Compiling auto_impl v1.3.0
   Compiling serde_bytes v0.11.19
   Compiling hyper-util v0.1.20
   Compiling toml_datetime v0.7.5+spec-1.1.0
   Compiling nom v8.0.0
   Compiling bitmaps v3.2.1
   Compiling async-compression v0.4.19
   Compiling wu-manber v0.1.0 (https://github.com/tvlfyi/wu-manber.git#0d5b22be)
   Compiling heck v0.4.1
   Compiling toml_writer v1.1.1+spec-1.1.0
   Compiling termcolor v1.4.1
   Compiling zerocopy v0.8.48
   Compiling derive_builder_core v0.20.2
   Compiling nickel-lang-parser v0.1.1
   Compiling crunch-attestation v0.1.0 (/home/brittonr/git/mantle/crates/crunch-attestation)
   Compiling logos-derive v0.15.1
   Compiling gix-revision v0.43.0
   Compiling gix-prompt v0.14.1
   Compiling hashlink v0.10.0
   Compiling imara-diff v0.1.8
   Compiling imara-diff v0.2.0
   Compiling ouroboros_macro v0.18.5
   Compiling proc-macro-error-attr2 v2.0.0
   Compiling typed-arena v2.0.2
   Compiling aliasable v0.1.3
   Compiling imbl-sized-chunks v0.1.3
   Compiling codespan-reporting v0.13.1
   Compiling paste v1.0.15
   Compiling arraydeque v0.5.1
   Compiling snix-store v0.1.0 (/home/brittonr/git/mantle/vendor/snix-store)
   Compiling arrayvec v0.5.2
   Compiling gix-credentials v0.37.1
   Compiling derive_builder_macro v0.20.2
   Compiling proc-macro-error2 v2.0.1
   Compiling logos v0.15.1
   Compiling toml_edit v0.23.10+spec-1.0.0
   Compiling nickel-lang-core v0.16.1
   Compiling gix-discover v0.49.0
   Compiling ouroboros v0.18.5
   Compiling gix-refspec v0.39.0
   Compiling gix-diff v0.61.0
   Compiling nix v0.29.0
   Compiling nickel-lang-vector v0.1.0
   Compiling saphyr-parser v0.0.6
   Compiling pretty v0.12.5
   Compiling codespan v0.13.1
   Compiling uluru v3.1.0
   Compiling clru v0.6.3
   Compiling lru v0.16.4
   Compiling serde_spanned v1.1.1
   Compiling vte v0.14.1
   Compiling simple-counter v0.1.0
   Compiling rustix v0.38.44
   Compiling count-write v0.1.0
   Compiling unsafe-libyaml v0.2.11
   Compiling bumpalo v3.20.2
   Compiling gix-dir v0.23.0
   Compiling getset v0.1.6
   Compiling derive_builder v0.20.2
   Compiling gix-pack v0.68.0
   Compiling gix-config v0.54.0
   Compiling toml v0.9.12+spec-1.1.0
   Compiling gix-worktree-stream v0.30.0
   Compiling strip-ansi-escapes v0.2.1
   Compiling gix-negotiate v0.29.0
   Compiling gix-shallow v0.10.0
   Compiling strum_macros v0.26.4
   Compiling typed-builder-macro v0.22.0
   Compiling maybe-async v0.2.10
   Compiling rand_core v0.6.4
   Compiling sha-1 v0.10.1
   Compiling fs2 v0.4.3
   Compiling io-close v0.3.7
   Compiling json_scanner v0.1.0
   Compiling linux-raw-sys v0.4.15
   Compiling indoc v2.0.7
   Compiling strum v0.26.3
   Compiling crc-catalog v2.4.0
   Compiling serde_yaml v0.9.34+deprecated
   Compiling snix-build v0.1.0 (/home/brittonr/git/mantle/vendor/snix-build)
   Compiling gix-archive v0.30.0
   Compiling typed-builder v0.22.0
   Compiling gix-status v0.28.0
   Compiling gix-blame v0.11.0
   Compiling malachite-q v0.6.1
   Compiling gix-worktree-state v0.28.0
   Compiling gix-mailmap v0.32.0
   Compiling ureq-proto v0.6.0
   Compiling webpki-roots v1.0.6
   Compiling itertools v0.12.1
   Compiling oci-spec v0.7.1
   Compiling crc v3.4.0
   Compiling uuid v1.23.0
   Compiling gix-submodule v0.28.0
   Compiling twox-hash v2.1.2
   Compiling utf8-zero v0.8.1
   Compiling tar v0.4.45
   Compiling serde_spanned v0.6.9
   Compiling toml_datetime v0.6.11
   Compiling bzip2-rs v0.1.2
   Compiling toml_write v0.1.2
   Compiling gix-odb v0.78.0
   Compiling lzma-rs v0.3.0
   Compiling crunch-project-core v0.1.0 (/home/brittonr/git/mantle/crates/crunch-project-core)
   Compiling ruzstd v0.8.2
   Compiling crunch-shell-core v0.1.0 (/home/brittonr/git/mantle/crates/crunch-shell-core)
   Compiling malachite-float v0.6.1
   Compiling gethostname v0.5.0
   Compiling ppv-lite86 v0.2.21
   Compiling crunch-release-core v0.1.0 (/home/brittonr/git/mantle/crates/crunch-release-core)
   Compiling crunch-glue v0.1.0 (/home/brittonr/git/mantle/crates/crunch-glue)
   Compiling crunch-bootstrap-core v0.1.0 (/home/brittonr/git/mantle/crates/crunch-bootstrap-core)
   Compiling predicates-core v1.0.10
   Compiling float-cmp v0.10.0
   Compiling crunch-shell v0.1.0 (/home/brittonr/git/mantle/crates/crunch-shell)
   Compiling normalize-line-endings v0.3.0
   Compiling toml_edit v0.22.27
   Compiling difflib v0.4.0
   Compiling assert_cmd v2.2.0
   Compiling termtree v0.5.1
   Compiling wait-timeout v0.2.1
   Compiling crunch-delta-core v0.1.0 (/home/brittonr/git/mantle/crates/crunch-delta-core)
   Compiling diff v0.1.13
   Compiling rand_chacha v0.3.1
   Compiling crunch-project v0.1.0 (/home/brittonr/git/mantle/crates/crunch-project)
   Compiling predicates-tree v1.0.13
   Compiling predicates v3.1.4
   Compiling pretty_assertions v1.4.1
   Compiling malachite v0.6.1
   Compiling rand v0.8.6
   Compiling toml v0.8.23
   Compiling rustls-webpki v0.103.12
   Compiling tokio-rustls v0.26.4
   Compiling rustls-platform-verifier v0.6.2
   Compiling ureq v3.3.0
   Compiling hyper-rustls v0.27.7
   Compiling reqwest v0.13.2
   Compiling reqwest v0.12.28
   Compiling object_store v0.13.2
   Compiling reqwest-middleware v0.5.1
   Compiling gix-transport v0.55.1
   Compiling reqwest-tracing v0.6.0
   Compiling gix-protocol v0.59.0
   Compiling snix-tracing v0.1.0 (/home/brittonr/git/mantle/vendor/snix-tracing)
   Compiling gix v0.81.0
   Compiling crunch-store v0.1.0 (/home/brittonr/git/mantle/crates/crunch-store)
   Compiling crunch-delta v0.1.0 (/home/brittonr/git/mantle/crates/crunch-delta)
   Compiling crunch-build v0.1.0 (/home/brittonr/git/mantle/crates/crunch-build)
   Compiling nickel-lang v2.0.0
   Compiling crunch-eval v0.1.0 (/home/brittonr/git/mantle/crates/crunch-eval)
   Compiling crunch-pipeline v0.1.0 (/home/brittonr/git/mantle/crates/crunch-pipeline)
   Compiling mantle v0.1.0 (/home/brittonr/git/mantle)
warning: struct `NativeTextInput` is never constructed
    --> src/rust_plan.rs:1013:8
     |
1013 | struct NativeTextInput {
     |        ^^^^^^^^^^^^^^^
     |
     = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: struct `NativeManifestLockInputTexts` is never constructed
    --> src/rust_plan.rs:1019:8
     |
1019 | struct NativeManifestLockInputTexts {
     |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: struct `NativeManifestLockUnsupportedBlocker` is never constructed
    --> src/rust_plan.rs:1025:8
     |
1025 | struct NativeManifestLockUnsupportedBlocker {
     |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: struct `NativeFeatureRoleResolutionRequest` is never constructed
    --> src/rust_plan.rs:1049:8
     |
1049 | struct NativeFeatureRoleResolutionRequest {
     |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: struct `NativeFeatureRoleResolution` is never constructed
    --> src/rust_plan.rs:1056:8
     |
1056 | struct NativeFeatureRoleResolution {
     |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `collect_native_manifest_lock_texts` is never used
    --> src/rust_plan.rs:3633:4
     |
3633 | fn collect_native_manifest_lock_texts(root: &Path) -> Result<NativeManifestLockInputTexts, String> {
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `read_native_text_input` is never used
    --> src/rust_plan.rs:3658:4
     |
3658 | fn read_native_text_input(path: &Path, label: &str) -> Result<NativeTextInput, String> {
     |    ^^^^^^^^^^^^^^^^^^^^^^

warning: function `native_manifest_lock_unsupported_blockers` is never used
    --> src/rust_plan.rs:3667:4
     |
3667 | fn native_manifest_lock_unsupported_blockers(
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `native_manifest_unsupported_blockers` is never used
    --> src/rust_plan.rs:3680:4
     |
3680 | fn native_manifest_unsupported_blockers(input: &NativeTextInput) -> Vec<NativeManifestLockUnsupportedBlocker> {
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `native_target_table_unsupported_blockers` is never used
    --> src/rust_plan.rs:3711:4
     |
3711 | fn native_target_table_unsupported_blockers(
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `native_lockfile_unsupported_blockers` is never used
    --> src/rust_plan.rs:3741:4
     |
3741 | fn native_lockfile_unsupported_blockers(input: &NativeTextInput) -> Vec<NativeManifestLockUnsupportedBlocker> {
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `lock_source_supported` is never used
    --> src/rust_plan.rs:3773:4
     |
3773 | fn lock_source_supported(source: &str) -> bool {
     |    ^^^^^^^^^^^^^^^^^^^^^

warning: function `native_manifest_lock_blocker` is never used
    --> src/rust_plan.rs:3777:4
     |
3777 | fn native_manifest_lock_blocker(path: &str, class: &str, message: &str) -> NativeManifestLockUnsupportedBlocker {
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `resolve_native_feature_roles` is never used
    --> src/rust_plan.rs:5047:4
     |
5047 | fn resolve_native_feature_roles(request: NativeFeatureRoleResolutionRequest) -> NativeFeatureRoleResolution {
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: `mantle` (bin "crunch") generated 14 warnings
warning: `mantle` (bin "mantle") generated 14 warnings (14 duplicates)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1m 04s
     Running tests/examples_inventory.rs (/home/brittonr/.cargo-target/debug/deps/examples_inventory-f099cf4a3bfa12d9)

running 7 tests
test examples_catalog_rejects_stale_crunch_branding_in_user_docs ... ok
test documented_crunch_ncl_identifier_is_allowed ... ok
test examples_catalog_rejects_readme_omissions_and_stale_paths ... ok
test examples_catalog_rejects_duplicate_ids_and_paths ... ok
test examples_catalog_rejects_invalid_tier_and_silent_skip ... ok
test examples_catalog_supports_lane_inventory_for_docs ... ok
test examples_catalog_covers_checked_in_user_facing_examples ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s


```

Exit: `0`
