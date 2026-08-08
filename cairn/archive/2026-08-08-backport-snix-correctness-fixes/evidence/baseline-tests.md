# Snix correctness backport baseline

Date: 2026-08-08

## `SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo test -p snix-store --lib`

```text
warning: /home/brittonr/git/OnixResearch/mantle/.pi/worktrees/cairn-audit-repair-split-snix-20260808T050243Z/Cargo.toml: file `/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/cairn-audit-repair-split-snix-20260808T050243Z/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling proc-macro2 v1.0.106
   Compiling quote v1.0.45
   Compiling unicode-ident v1.0.24
   Compiling libc v0.2.186
   Compiling cfg-if v1.0.4
   Compiling shlex v1.3.0
   Compiling find-msvc-tools v0.1.9
   Compiling serde_core v1.0.228
   Compiling pin-project-lite v0.2.17
   Compiling serde v1.0.228
   Compiling futures-core v0.3.32
   Compiling memchr v2.8.0
   Compiling futures-io v0.3.32
   Compiling slab v0.4.12
   Compiling once_cell v1.21.4
   Compiling equivalent v1.0.2
   Compiling smallvec v1.15.1
   Compiling futures-sink v0.3.32
   Compiling itoa v1.0.18
   Compiling stable_deref_trait v1.2.1
   Compiling futures-task v0.3.32
   Compiling bitflags v2.11.0
   Compiling subtle v2.6.1
   Compiling zeroize v1.8.2
   Compiling fs_extra v1.3.0
   Compiling dunce v1.0.5
   Compiling tracing-core v0.1.36
   Compiling crossbeam-utils v0.8.21
   Compiling futures-channel v0.3.32
   Compiling allocator-api2 v0.2.21
   Compiling pkg-config v0.3.32
   Compiling autocfg v1.5.0
   Compiling version_check v0.9.5
   Compiling aws-lc-rs v1.16.2
   Compiling foldhash v0.2.0
   Compiling log v0.4.29
   Compiling litemap v0.8.2
   Compiling writeable v0.6.3
   Compiling rustls-pki-types v1.14.0
   Compiling typenum v1.20.1
   Compiling percent-encoding v2.3.2
   Compiling icu_properties_data v2.2.0
   Compiling icu_normalizer_data v2.2.0
   Compiling generic-array v0.14.7
   Compiling hashbrown v0.16.1
   Compiling utf8_iter v1.0.4
   Compiling strsim v0.11.1
   Compiling parking v2.2.1
   Compiling httparse v1.10.1
   Compiling rustls v0.23.37
   Compiling untrusted v0.9.0
   Compiling atomic-waker v1.1.2
   Compiling thiserror v2.0.18
   Compiling try-lock v0.2.5
   Compiling semver v1.0.28
   Compiling tower-service v0.3.3
   Compiling fnv v1.0.7
   Compiling rustix v1.1.4
   Compiling rustc_version v0.4.1
   Compiling want v0.3.1
   Compiling zerocopy v0.8.48
   Compiling cfg_aliases v0.2.1
   Compiling linux-raw-sys v0.12.1
   Compiling form_urlencoded v1.2.2
   Compiling indexmap v2.13.1
   Compiling sync_wrapper v1.0.2
   Compiling fastrand v2.4.1
   Compiling ipnet v2.12.0
   Compiling arrayvec v0.7.6
   Compiling openssl-probe v0.2.1
   Compiling regex-syntax v0.8.10
   Compiling base64 v0.22.1
   Compiling tower-layer v0.3.3
   Compiling rustls-native-certs v0.8.3
   Compiling utf8parse v0.2.2
   Compiling portable-atomic v1.13.1
   Compiling crossbeam-epoch v0.9.20
   Compiling ident_case v1.0.1
   Compiling iri-string v0.7.12
   Compiling futures-lite v2.6.1
   Compiling encoding_rs v0.8.35
   Compiling anstyle-parse v1.0.0
   Compiling is_terminal_polyfill v1.70.2
   Compiling block-buffer v0.10.4
   Compiling crypto-common v0.1.7
   Compiling colorchoice v1.0.5
   Compiling rustversion v1.0.22
   Compiling mime v0.3.17
   Compiling digest v0.10.7
   Compiling concurrent-queue v2.5.0
   Compiling anstyle v1.0.14
   Compiling rayon-core v1.13.0
   Compiling unicode-segmentation v1.13.2
   Compiling syn v2.0.117
   Compiling unicode-width v0.2.2
   Compiling anyhow v1.0.102
   Compiling winnow v1.0.1
   Compiling rand_core v0.10.0
   Compiling jobserver v0.1.34
   Compiling errno v0.3.14
   Compiling mio v1.2.0
   Compiling socket2 v0.6.3
   Compiling zmij v1.0.21
   Compiling getrandom v0.4.2
   Compiling cc v1.2.59
   Compiling signal-hook-registry v1.4.8
   Compiling lazy_static v1.5.0
   Compiling anstyle-query v1.1.5
   Compiling convert_case v0.10.0
   Compiling toml_parser v1.1.2+spec-1.1.0
   Compiling anstream v1.0.0
   Compiling vte v0.15.0
   Compiling num-traits v0.2.19
   Compiling parking_lot_core v0.9.12
   Compiling cmake v0.1.58
   Compiling zstd-sys v2.0.16+zstd.1.5.7
   Compiling bzip2-sys v0.1.13+1.0.8
   Compiling lzma-sys v0.1.20
   Compiling zstd-safe v7.2.4
   Compiling aws-lc-sys v0.39.1
   Compiling heck v0.5.0
   Compiling toml_datetime v1.1.1+spec-1.1.0
   Compiling unicode-xid v0.2.6
   Compiling serde_json v1.0.149
   Compiling cpufeatures v0.2.17
   Compiling typeid v1.0.3
   Compiling serde_derive v1.0.228
   Compiling tokio-macros v2.7.0
   Compiling synstructure v0.13.2
   Compiling futures-macro v0.3.32
   Compiling zerovec-derive v0.11.3
   Compiling displaydoc v0.2.5
   Compiling zerofrom-derive v0.1.7
   Compiling yoke-derive v0.8.2
   Compiling tracing-attributes v0.1.31
   Compiling thiserror-impl v2.0.18
   Compiling ppv-lite86 v0.2.21
   Compiling darling_core v0.23.0
   Compiling futures-util v0.3.32
   Compiling zerofrom v0.1.7
   Compiling pin-project-internal v1.1.11
   Compiling async-trait v0.1.89
   Compiling clap_lex v1.1.0
   Compiling crc32fast v1.5.0
   Compiling yoke v0.8.2
   Compiling tracing v0.1.44
   Compiling clap_builder v4.6.0
   Compiling aho-corasick v1.1.4
   Compiling pin-project v1.1.11
   Compiling zerovec v0.11.6
   Compiling zerotrie v0.2.4
   Compiling clap_derive v4.6.0
   Compiling derive_more-impl v2.1.1
   Compiling tinystr v0.8.3
   Compiling potential_utf v0.1.5
   Compiling darling_macro v0.23.0
   Compiling vt100 v0.16.2
   Compiling icu_collections v2.2.0
   Compiling blake3 v1.8.2
   Compiling icu_locale_core v2.2.0
   Compiling libmimalloc-sys v0.1.44
   Compiling crossbeam-deque v0.8.6
   Compiling darling v0.23.0
   Compiling toml_edit v0.25.10+spec-1.1.0
   Compiling sharded-slab v0.1.7
   Compiling event-listener v5.4.1
   Compiling console v0.16.3
   Compiling getrandom v0.2.17
   Compiling n0-future v0.3.2
   Compiling nix v0.31.3
   Compiling icu_provider v2.2.0
   Compiling curve25519-dalek v4.1.3
   Compiling tracing-log v0.2.0
   Compiling icu_properties v2.2.0
   Compiling icu_normalizer v2.2.0
   Compiling memoffset v0.6.5
   Compiling async-io v2.6.0
   Compiling thread_local v1.1.9
   Compiling rustix v0.38.44
   Compiling spin v0.10.0
   Compiling cordyceps v0.3.4
   Compiling erased-serde v0.4.10
   Compiling diatomic-waker v0.2.3
   Compiling simd-adler32 v0.3.9
   Compiling adler2 v2.0.1
   Compiling thiserror v1.0.69
   Compiling scopeguard v1.2.0
   Compiling unit-prefix v0.5.2
   Compiling nu-ansi-term v0.50.3
   Compiling cpufeatures v0.3.0
   Compiling miniz_oxide v0.8.9
   Compiling lock_api v0.4.14
   Compiling indicatif v0.18.4
   Compiling chacha20 v0.10.0
   Compiling futures-buffered v0.2.13
   Compiling regex-automata v0.4.14
   Compiling event-listener-strategy v0.5.4
   Compiling rand_core v0.6.4
   Compiling futures-executor v0.3.32
   Compiling serde_with_macros v3.18.0
   Compiling clap v4.6.0
   Compiling n0-error-macros v0.1.3
   Compiling async-stream-impl v0.3.6
   Compiling curve25519-dalek-derive v0.1.1
   Compiling spez v0.1.2
   Compiling thiserror-impl v1.0.69
   Compiling rstest_macros v0.26.1
   Compiling iana-time-zone v0.1.65
   Compiling signature v2.2.0
   Compiling constant_time_eq v0.3.1
   Compiling linux-raw-sys v0.4.15
   Compiling idna_adapter v1.2.1
   Compiling ryu v1.0.23
   Compiling bitflags v1.3.2
   Compiling matchit v0.8.4
   Compiling foldhash v0.1.5
   Compiling arrayref v0.3.9
   Compiling either v1.15.0
   Compiling same-file v1.0.6
   Compiling redb v3.1.3
   Compiling idna v1.1.0
   Compiling walkdir v2.5.0
   Compiling itertools v0.15.0
   Compiling parking_lot v0.12.5
   Compiling hashbrown v0.15.5
   Compiling proc-macro-crate v3.5.0
   Compiling ed25519 v2.2.3
   Compiling n0-error v0.1.3
   Compiling async-stream v0.3.6
   Compiling rand_chacha v0.3.1
   Compiling flate2 v1.1.9
   Compiling clap-verbosity-flag v3.0.4
   Compiling bzip2 v0.5.2
   Compiling futures v0.3.32
   Compiling serde_with v3.18.0
   Compiling rand v0.10.1
   Compiling num_enum_derive v0.7.6
   Compiling cobs v0.3.0
   Compiling derive_more v2.1.1
   Compiling sha2 v0.10.9
   Compiling irpc-derive v0.10.0
   Compiling nibble_vec v0.1.0
   Compiling nix-compat v0.1.0 (/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/cairn-audit-repair-split-snix-20260808T050243Z/vendor/nix-compat)
   Compiling humantime v2.3.0
   Compiling fixedbitset v0.4.2
   Compiling data-encoding v2.10.0
   Compiling snix-castore v0.1.0 (/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/cairn-audit-repair-split-snix-20260808T050243Z/vendor/snix-castore)
   Compiling fuse-backend-rs v0.12.0 (/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/cairn-audit-repair-split-snix-20260808T050243Z/vendor/fuse-backend-rs)
   Compiling relative-path v1.9.3
   Compiling rustc-hash v2.1.2
   Compiling glob v0.3.3
   Compiling endian-type v0.1.2
   Compiling petgraph v0.6.5
   Compiling radix_trie v0.2.1
   Compiling rand v0.8.6
   Compiling bytes v1.11.1
   Compiling url v2.5.8
   Compiling chrono v0.4.44
   Compiling quick-xml v0.40.1
   Compiling serde_urlencoded v0.7.1
   Compiling serde_tagged v0.3.0
   Compiling postcard v1.1.3
   Compiling serde_qs v0.12.0
   Compiling mimalloc v0.1.48
   Compiling num_enum v0.7.6
   Compiling nix v0.24.3
   Compiling serde_bytes v0.11.19
   Compiling tokio v1.51.0
   Compiling http v1.4.0
   Compiling vmm-sys-util v0.11.2
   Compiling async-lock v3.4.2
   Compiling ed25519-dalek v2.2.0
   Compiling async-channel v2.5.0
   Compiling arc-swap v1.9.1
   Compiling nix-compat-derive v0.1.0 (/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/cairn-audit-repair-split-snix-20260808T050243Z/vendor/nix-compat-derive)
   Compiling auto_impl v1.3.0
   Compiling mio v0.8.11
   Compiling caps v0.5.6
   Compiling vm-memory v0.10.0
   Compiling nom v8.0.0
   Compiling wu-manber v0.1.0 (https://github.com/tvlfyi/wu-manber.git#0d5b22be)
   Compiling async-task v4.7.1
   Compiling snix-store v0.1.0 (/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/cairn-audit-repair-split-snix-20260808T050243Z/vendor/snix-store)
   Compiling rstest_reuse v0.7.0
   Compiling sha1 v0.10.6
   Compiling md-5 v0.10.6
   Compiling lru v0.16.4
   Compiling hex-literal v0.4.1
   Compiling count-write v0.1.0
   Compiling regex v1.12.3
   Compiling http-body v1.0.1
   Compiling http-body-util v0.1.3
   Compiling matchers v0.2.0
   Compiling bstr v1.12.1
   Compiling tracing-subscriber v0.3.23
   Compiling xz2 v0.1.7
   Compiling polling v3.11.0
   Compiling xattr v1.6.1
   Compiling tempfile v3.27.0
   Compiling async-signal v0.2.13
   Compiling async-process v2.5.0
   Compiling tracing-indicatif v0.3.14
   Compiling zstd v0.13.3
   Compiling rstest v0.26.1
   Compiling tokio-util v0.7.18
   Compiling tower v0.5.3
   Compiling tokio-stream v0.1.18
   Compiling async-compression v0.4.19
   Compiling tokio-retry v0.3.0
   Compiling fastcdc v3.2.1
   Compiling astral-tokio-tar v0.6.3
   Compiling tower-http v0.6.8
   Compiling h2 v0.4.13
   Compiling irpc v0.13.0
   Compiling hyper v1.9.0
   Compiling hyper-util v0.1.20
   Compiling rustls-webpki v0.103.13
   Compiling tokio-rustls v0.26.4
   Compiling rustls-platform-verifier v0.6.2
   Compiling hyper-rustls v0.27.7
   Compiling reqwest v0.13.2
   Compiling reqwest-middleware v0.5.1
   Compiling object_store v0.14.0
   Compiling reqwest-tracing v0.6.0
   Compiling snix-tracing v0.1.0 (/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/cairn-audit-repair-split-snix-20260808T050243Z/vendor/snix-tracing)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 27.99s
     Running unittests src/lib.rs (/tmp/mantle-snix-backport-baseline-target/debug/deps/snix_store-b8c96cc9ad908caa)

running 86 tests
test nar::import::test::ingest_with_cahash_mismatch::case_3_flat_md5 ... ok
test nar::import::test::ingest_with_cahash_mismatch::case_4_nar_symlink_sha1 ... ok
test nar::import::test::ingest_with_cahash_correct::case_4_nar_symlink_sha1 ... ok
test nar::import::test::single_file ... ok
test nar::import::test::single_symlink ... ok
test pathinfoservice::from_addr::tests::test_from_addr_tokio::case_19_grpc_invalid_host_and_path ... ok
test pathinfoservice::from_addr::tests::test_from_addr_tokio::case_18_grpc_unsupported_https_host_without_port ... ok
test pathinfoservice::from_addr::tests::test_from_addr_tokio::case_17_grpc_unsupported_http_host_without_port ... ok
test nar::import::test::ingest_with_flat_non_file::case_1_nar_sha256 ... ok
test pathinfoservice::lru::test::evict ... ok
test nar::import::test::ingest_with_cahash_correct::case_1_nar_sha256 ... ok
test pathinfoservice::nix_http::tests::parse_url::case_01_correct_nix_https ... ok
test nar::import::test::ingest_with_cahash_mismatch::case_1_nar_sha256 ... ok
test pathinfoservice::nix_http::tests::parse_url::case_02_correct_nix_http ... ok
test pathinfoservice::nix_http::tests::parse_url::case_07_wrong_scheme ... ok
test pathinfoservice::nix_http::tests::parse_url::case_06_correct_nix_https_with_two_trusted_public_keys ... ok
test pathinfoservice::nix_http::tests::parse_url::case_03_correct_nix_http_with_subpath ... ok
test pathinfoservice::nix_http::tests::parse_url::case_04_correct_nix_http_with_subpath_and_port ... ok
test nar::import::test::complicated ... ok
test pathinfoservice::nix_http::tests::parse_url::case_08_missing_host ... ok
test pathinfoservice::nix_http::tests::parse_url::case_05_correct_nix_https_with_trusted_public_key ... ok
test pathinfoservice::nix_http::tests::parse_url::case_09_missing_authority ... ok
test pathinfoservice::nix_http::tests::parse_url::case_10_trusted_public_keys_no_sequence ... ok
test pathinfoservice::nix_http::tests::parse_url::case_11_trusted_public_keys_wrong_pubkey ... ok
test proto::tests::pathinfo::convert_fail_entry::case_1_directory_invalid_digest_length ... ok
test proto::tests::pathinfo::convert_fail_entry::case_2_directory_invalid_entry_name_no_storepath ... ok
test proto::tests::pathinfo::convert_fail_entry::case_3_file_invalid_digest_len ... ok
test nar::import::test::ingest_with_flat_non_file::case_2_nar_symlink_sha1 ... ok
test proto::tests::pathinfo::convert_fail_entry::case_4_file_invalid_entry_name ... ok
test nar::import::test::ingest_with_cahash_mismatch::case_2_nar_sha512 ... ok
test proto::tests::pathinfo::convert_fail_entry::case_5_symlink_invalid_entry_name ... ok
test proto::tests::pathinfo::convert_inconsistent_narinfo_reference_name_digest ... ok
test proto::tests::pathinfo::convert_inconsistent_num_refs_fail ... ok
test proto::tests::pathinfo::convert_invalid_narinfo_reference_name ... ok
test proto::tests::pathinfo::convert_invalid_deriver ... ok
test proto::tests::pathinfo::convert_invalid_reference_digest_len ... ok
test proto::tests::pathinfo::convert_pathinfo_wrong_entries::case_2_no_entry_2 ... ok
test proto::tests::pathinfo::convert_pathinfo_wrong_entries::case_1_no_entry ... ok
test proto::tests::pathinfo::convert_valid ... ok
test proto::tests::pathinfo::convert_valid_deriver ... ok
test proto::tests::pathinfo::convert_without_narinfo_fail ... ok
test proto::tests::postcard_roundtrip::ca_hash_enum_roundtrip ... ok
test proto::tests::pathinfo::convert_wrong_nar_sha256 ... ok
test proto::tests::postcard_roundtrip::pathinfo_roundtrip ... ok
test pathinfoservice::cache::test::no_backfill_listing_merges_precedence_and_exact_layers ... ok
test nar::import::test::ingest_with_cahash_correct::case_3_flat_md5 ... ok
test nar::import::test::ingest_with_cahash_correct::case_2_nar_sha512 ... ok
test pathinfoservice::from_addr::tests::test_from_addr_tokio::case_16_grpc_unsupported_ipv6_localhost_port_12345 ... ok
test pathinfoservice::from_addr::tests::test_from_addr_tokio::case_13_grpc_unsupported_unix_socket ... ok
test pathinfoservice::from_addr::tests::test_from_addr_tokio::case_11_redb_memory_invalid_authority_path ... ok
test pathinfoservice::from_addr::tests::test_from_addr_tokio::case_02_redb_invalid_missing_path ... ok
test pathinfoservice::from_addr::tests::test_from_addr_tokio::case_04_redb_invalid_host ... ok
test pathinfoservice::from_addr::tests::test_from_addr_tokio::case_14_grpc_invalid_unix_socket_authority ... ok
test pathinfoservice::from_addr::tests::test_from_addr_tokio::case_01_unsupported_scheme ... ok
test pathinfoservice::from_addr::tests::test_from_addr_tokio::case_10_redb_memory_invalid_authority ... ok
test pathinfoservice::from_addr::tests::test_from_addr_tokio::case_15_grpc_invalid_unix_socket_and_host ... ok
test pathinfoservice::from_addr::tests::test_from_addr_tokio::case_05_redb_invalid_path_authority ... ok
test pathinfoservice::from_addr::tests::test_from_addr_tokio::case_03_redb_invalid_root ... ok
test pathinfoservice::from_addr::tests::test_from_addr_tokio::case_09_redb_memory_invalid_path ... ok
test pathinfoservice::from_addr::tests::test_from_addr_tokio::case_07_redb_invalid_host_with_valid_path ... ok
test pathinfoservice::from_addr::tests::test_from_addr_tokio::case_06_redb_valid_path ... ok
test pathinfoservice::tests::not_found::case_1_redb ... ok
test pathinfoservice::tests::not_found::case_2_signing ... ok
test pathinfoservice::tests::put_get::case_1_redb ... ok
test pathinfoservice::tests::put_get::case_2_signing ... ok
test tests::nar_renderer::seekable::case_1_symlink ... ok
test tests::nar_renderer::seekable::case_4_too_small ... ok
test tests::nar_renderer::seekable::case_5_complicated ... ok
test tests::nar_renderer::single_file_missing_blob ... ok
test tests::nar_renderer::seekable::case_3_too_big ... ok
test tests::nar_renderer::seekable::case_2_helloworld ... ok
test pathinfoservice::signing_wrapper::test::put_and_verify_signature ... ok
test pathinfoservice::from_addr::tests::test_from_addr_tokio::case_08_redb_memory_valid ... ok
test pathinfoservice::cache::test::test_populate_cache ... ok
test pathinfoservice::cache::test::no_backfill_reports_far_layer_and_preserves_near_miss ... ok
test tests::nar_renderer_seekable::read_to_end::case_3_too_big ... ok
test tests::nar_renderer_seekable::read_to_end::case_2_helloworld ... ok
test tests::nar_renderer_seekable::read_to_end::case_4_too_small ... ok
test tests::nar_renderer_seekable::seek_twice ... ok
test tests::nar_renderer_seekable::seek ... ok
test tests::nar_renderer_seekable::read_to_end::case_1_symlink ... ok
test tests::nar_renderer_seekable::read_to_end::case_5_complicated ... ok
test pathinfoservice::from_addr::tests::test_from_addr_tokio::case_12_nix_http ... ok
test pathinfoservice::nix_http::tests::get_references_fetches_only_narinfo_metadata ... ok
test pathinfoservice::nix_http::tests::narinfo_parsing_uses_the_configured_store_directory ... ok
test pathinfoservice::nix_http::tests::get_still_fetches_nar_payload_for_full_pathinfo ... ok

test result: ok. 86 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s


exit_status=0
```

## `SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo test -p snix-castore --lib`

```text
warning: /home/brittonr/git/OnixResearch/mantle/.pi/worktrees/cairn-audit-repair-split-snix-20260808T050243Z/Cargo.toml: file `/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/cairn-audit-repair-split-snix-20260808T050243Z/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling hashbrown v0.16.1
   Compiling tokio-util v0.7.18
   Compiling crypto-common v0.1.7
   Compiling url v2.5.8
   Compiling async-compression v0.4.19
   Compiling snix-tracing v0.1.0 (/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/cairn-audit-repair-split-snix-20260808T050243Z/vendor/snix-tracing)
   Compiling tokio-test v0.4.5
   Compiling bstr v1.12.1
   Compiling digest v0.10.7
   Compiling blake3 v1.8.2
   Compiling indexmap v2.13.1
   Compiling n0-future v0.3.2
   Compiling irpc v0.13.0
   Compiling h2 v0.4.13
   Compiling petgraph v0.6.5
   Compiling hyper v1.9.0
   Compiling hyper-util v0.1.20
   Compiling hyper-rustls v0.27.7
   Compiling reqwest v0.13.2
   Compiling object_store v0.14.0
   Compiling snix-castore v0.1.0 (/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/cairn-audit-repair-split-snix-20260808T050243Z/vendor/snix-castore)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 12.46s
     Running unittests src/lib.rs (/tmp/mantle-snix-backport-baseline-target/debug/deps/snix_castore-37f6094d9fb7ec1a)

running 253 tests
test blobservice::chunked_reader::test::from_iter ... ok
test blobservice::chunked_reader::test::chunk_idx_for_position ... ok
test blobservice::chunked_reader::test::from_iter_empty - should panic ... ok
test blobservice::chunker::tests::boundary_validation_accepts_empty_blob_without_chunks ... ok
test blobservice::chunker::tests::boundary_validation_rejects_gap ... ok
test blobservice::chunker::tests::boundary_validation_rejects_overlap ... ok
test blobservice::chunker::tests::boundary_validation_rejects_short_non_final_chunk ... ok
test blobservice::chunker::tests::boundary_validation_rejects_zero_length ... ok
test blobservice::chunker::tests::fastcdc_default_boundaries_cover_blob ... ok
test blobservice::chunked_reader::test::test_read ... ok
test blobservice::chunked_reader::test::test_read_missing_chunks ... ok
test blobservice::chunked_reader::test::test_seek ... ok
test blobservice::from_addr::tests::test_from_addr_tokio::case_02_memory_valid ... ok
test blobservice::from_addr::tests::test_from_addr_tokio::case_01_unsupported_scheme ... ok
test blobservice::combinator::tests::explicit_write_routes_to_near_only ... ok
test blobservice::combinator::tests::no_backfill_reports_far_layer_without_near_blob ... ok
test blobservice::from_addr::tests::test_from_addr_tokio::case_04_memory_invalid_root_path ... ok
test blobservice::from_addr::tests::test_from_addr_tokio::case_03_memory_invalid_host ... ok
test blobservice::from_addr::tests::test_from_addr_tokio::case_06_grpc_unsupported_unix_socket ... ok
test blobservice::from_addr::tests::test_from_addr_tokio::case_09_grpc_unsupported_ipv6_localhost_port_12345 ... ok
test blobservice::from_addr::tests::test_from_addr_tokio::case_08_grpc_invalid_unix_socket_and_host ... ok
test blobservice::from_addr::tests::test_from_addr_tokio::case_05_memory_invalid_root_path_foo ... ok
test blobservice::from_addr::tests::test_from_addr_tokio::case_07_grpc_invalid_unix_socket_and_authority ... ok
test blobservice::from_addr::tests::test_from_addr_tokio::case_10_grpc_unsupported_http_host_without_port ... ok
test blobservice::from_addr::tests::test_from_addr_tokio::case_11_grpc_unsupported_https_host_without_port ... ok
test blobservice::from_addr::tests::test_from_addr_tokio::case_12_grpc_invalid_has_path ... ok
test blobservice::from_addr::tests::test_from_addr_tokio::case_14_objectstore_valid_file ... ok
test blobservice::from_addr::tests::test_from_addr_tokio::case_13_objectstore_valid_memory ... ok
test blobservice::tests::chunks_nonexistent_false::case_1_memory ... ok
test blobservice::tests::not_found_read::case_1_memory ... ok
test blobservice::tests::has_nonexistent_false::case_1_memory ... ok
test blobservice::tests::not_found_read::case_2_objectstore_memory ... ok
test blobservice::tests::chunks_nonexistent_false::case_2_objectstore_memory ... ok
test blobservice::tests::has_nonexistent_false::case_2_objectstore_memory ... ok
test directoryservice::directory_graph::tests::directory_graph::case_02_ltr_empty_directory ... ok
test directoryservice::directory_graph::tests::directory_graph::case_03_ltr_simple_closure ... ok
test directoryservice::directory_graph::tests::directory_graph::case_07_ltr_dangling_pointer ... ok
test directoryservice::directory_graph::tests::directory_graph::case_09_rtl_empty_directory ... ok
test composition::test::reject_recursion ... ok
test directoryservice::directory_graph::tests::rtl_wrong_digest ... ok
test directoryservice::directory_graph::tests::directory_graph::case_13_rtl_wrong_size_in_parent ... ok
test directoryservice::directory_graph::tests::directory_graph::case_06_ltr_unconnected_node ... ok
test directoryservice::directory_graph::tests::directory_graph::case_12_rtl_unconnected_node ... ok
test directoryservice::from_addr::tests::test_from_addr_tokio::case_01_unsupported_scheme ... ok
test directoryservice::directory_graph::tests::directory_graph::case_01_ltr_empty_graph ... ok
test directoryservice::directory_graph::tests::directory_graph::case_04_ltr_same_child ... ok
test directoryservice::directory_graph::tests::directory_graph::case_10_rtl_simple_closure ... ok
test directoryservice::from_addr::tests::test_from_addr_tokio::case_02_redb_invalid_missing_path ... ok
test directoryservice::directory_graph::tests::directory_graph::case_11_rtl_same_child_dedup ... ok
test directoryservice::from_addr::tests::test_from_addr_tokio::case_04_redb_invalid_host ... ok
test directoryservice::from_addr::tests::test_from_addr_tokio::case_03_redb_invalid_root ... ok
test blobservice::object_store::test::test_chunk_and_upload::case_1_a ... ok
test directoryservice::from_addr::tests::test_from_addr_tokio::case_05_redb_invalid_path_authority ... ok
test composition::test::concurrent ... ok
test directoryservice::directory_graph::tests::directory_graph::case_08_ltr_wrong_size_in_parent ... ok
test directoryservice::directory_graph::tests::directory_graph::case_05_ltr_same_child_dedup ... ok
test directoryservice::from_addr::tests::test_from_addr_tokio::case_12_grpc_invalid_unix_socket_and_authority ... ok
test directoryservice::from_addr::tests::test_from_addr_tokio::case_10_redb_memory_invalid_authority_path ... ok
test directoryservice::order_validator::tests::leaves_to_root::case_1_empty_directory ... ok
test directoryservice::order_validator::tests::leaves_to_root::case_2_simple_closure ... ok
test directoryservice::from_addr::tests::test_from_addr_tokio::case_17_grpc_invalid_host_and_path ... ok
test directoryservice::order_validator::tests::leaves_to_root::case_4_same_child_dedup ... ok
test directoryservice::from_addr::tests::test_from_addr_tokio::case_16_grpc_unsupported_https_host_without_port ... ok
test directoryservice::order_validator::tests::leaves_to_root::case_3_same_child ... ok
test directoryservice::order_validator::tests::leaves_to_root::case_7_empty ... ok
test directoryservice::from_addr::tests::test_from_addr_tokio::case_15_grpc_unsupported_http_host_without_port ... ok
test directoryservice::order_validator::tests::leaves_to_root::case_5_unconnected_node ... ok
test directoryservice::order_validator::tests::root_to_leaves::case_2_simple_closure ... ok
test directoryservice::order_validator::tests::leaves_to_root::case_6_dangling_pointer ... ok
test directoryservice::order_validator::tests::root_to_leaves::case_3_same_child_dedup ... ok
test blobservice::tests::put_seek::case_1_memory ... ok
test directoryservice::order_validator::tests::root_to_leaves::case_5_with_root_sent_twice ... ok
test directoryservice::order_validator::tests::root_to_leaves::case_4_same_child_redundant ... ok
test directoryservice::order_validator::tests::root_to_leaves::case_7_unconnected_node ... ok
test directoryservice::order_validator::tests::root_to_leaves::case_6_more_levels ... ok
test directoryservice::order_validator::tests::root_to_leaves::case_1_empty_directory ... ok
test directoryservice::order_validator::tests::root_to_leaves_root_mismatch ... ok
test directoryservice::from_addr::tests::test_from_addr_tokio::case_06_redb_valid_path ... ok
test directoryservice::order_validator::tests::root_to_leaves_stream ... ok
test blobservice::object_store::test::new_local_persists_across_instances ... ok
test blobservice::tests::put_has_get::case_1_memory ... ok
test directoryservice::tests::upload_reject_unconnected::case_2_objectstore ... ok
test directoryservice::tests::test_non_exist::case_2_objectstore ... ok
test directoryservice::from_addr::tests::test_from_addr_tokio::case_11_grpc_unsupported_unix_socket ... ok
test directoryservice::redb::tests::read_only_nonexistent ... ok
test directoryservice::tests::put_get_multiple_dedup::case_2_objectstore ... ok
test directoryservice::tests::put_get_multiple_success::case_2_objectstore ... ok
test directoryservice::from_addr::tests::test_from_addr_tokio::case_18_anonymous_url_composition ... ok
test directoryservice::from_addr::tests::test_from_addr_tokio::case_14_grpc_unsupported_ipv6_localhost_port_12345 ... ok
test directoryservice::from_addr::tests::test_from_addr_tokio::case_08_redb_memory_invalid_path ... ok
test directoryservice::tests::upload_reject_wrong_size::case_2_objectstore ... ok
test directoryservice::tests::upload_reject_dangling_pointer::case_2_objectstore ... ok
test directoryservice::tests::put_get_foo::case_2_objectstore ... ok
test hashing_reader::tests::test_b3_hashing_reader::case_1_blob_a ... ok
test directoryservice::tests::put_get::case_2_objectstore ... ok
test directoryservice::from_addr::tests::test_from_addr_tokio::case_13_grpc_invalid_unix_socket_and_host ... ok
test directoryservice::from_addr::tests::test_from_addr_tokio::case_09_redb_memory_invalid_authority ... ok
test blobservice::object_store::test::new_local_roundtrip ... ok
test import::archive::test::node_ingestion_error::case_1_no_top_level_entries ... ok
test hashing_reader::tests::test_b3_hashing_reader::case_3_empty_blob ... ok
test import::archive::test::node_ingestion_error::case_2_multiple_top_level_dirs ... ok
test hashing_reader::tests::test_b3_hashing_reader::case_2_blob_b ... ok
test import::archive::test::node_ingestion_error::case_3_top_level_file_entry ... ok
test import::archive::test::node_ingestion_success::case_1_implicit_directories ... ok
test directoryservice::redb::tests::open_rw_and_ro ... ok
test import::archive::test::node_ingestion_success::case_2_explicit_directories ... ok
test import::archive::test::node_ingestion_success::case_3_inaccesible_tree ... ok
test directoryservice::tests::upload_reject_wrong_size::case_1_memory ... ok
test blobservice::tests::put_has_get::case_2_objectstore_memory ... ok
test blobservice::tests::put_seek::case_2_objectstore_memory ... ok
test nodes::symlink_target::tests::error_toolong ... ok
test nodes::directory::test::validate_overflow ... ok
test nodes::symlink_target::tests::errors::case_1_empty ... ok
test nodes::symlink_target::tests::success::case_1_boring ... ok
test nodes::directory::test::add_nodes_to_directory ... ok
test nodes::symlink_target::tests::errors::case_2_null ... ok
test nodes::directory::test::from_iter_multiple ... ok
test nodes::directory::test::add_duplicate_node_to_directory ... ok
test nodes::directory::test::from_iter_single ... ok
test import::fs::tests::upload_blob_rejects_short_read_against_expected_size ... ok
test directoryservice::redb::tests::reopen_as_read_only ... ok
test nodes::symlink_target::tests::success::case_2_dot ... ok
test path::component::tests::errors::case_3_curdir ... ok
test nodes::symlink_target::tests::success::case_3_dotsandslashes ... ok
test nodes::symlink_target::tests::success::case_5_slashes ... ok
test nodes::symlink_target::tests::success::case_4_dotdot ... ok
test nodes::symlink_target::tests::success::case_6_slashes_and_absolute ... ok
test nodes::symlink_target::tests::success::case_7_invalid_utf8 ... ok
test path::component::tests::errors::case_1_empty ... ok
test path::component::tests::errors::case_2_null ... ok
test path::component::tests::error_toolong ... ok
test path::component::tests::errors::case_4_parent ... ok
test path::component::tests::errors::case_6_slashes2 ... ok
test path::component::tests::errors::case_5_slashes1 ... ok
test path::component::tests::extension::case_2_simple ... ok
test path::component::tests::extension::case_3_empty ... ok
test path::component::tests::extension::case_4_multiple ... ok
test path::component::tests::success ... ok
test path::test::components_bytes::case_1_empty ... ok
test path::component::tests::extension::case_1_without_dot ... ok
test path::test::components_bytes::case_2 ... ok
test path::test::extension::case_2_simple ... ok
test path::test::extension::case_4_multiple ... ok
test path::test::extension::case_3_empty ... ok
test path::test::extension::case_5_with_components ... ok
test path::test::components_bytes::case_3 ... ok
test path::test::components_bytes::case_4 ... ok
test path::test::extension::case_1_without_dot ... ok
test path::test::from_host_path::case_04_double_slash_middle ... ok
test path::test::extension::case_6_path ... ok
test path::test::from_host_path::case_01_empty ... ok
test path::test::from_host_path::case_03_path2 ... ok
test path::test::from_host_path::case_02_path ... ok
test path::test::from_host_path::case_05_dot ... ok
test path::test::from_host_path::case_06_dot_start ... ok
test path::test::from_host_path::case_08_dot_end ... ok
test path::test::from_host_path::case_07_dot_middle ... ok
test path::test::from_host_path::case_11_dotdot_canonicalize2 ... ok
test path::test::from_host_path::case_09_trailing_slash ... ok
test path::test::from_host_path::case_10_dotdot_canonicalize ... ok
test path::test::from_host_path::case_12_faux_prefix ... ok
test path::test::from_host_path::case_13_faux_letter ... ok
test path::test::from_host_path_fail::case_1_absolute ... ok
test path::test::from_host_path_fail::case_2_dotdot_root ... ok
test path::test::from_host_path_fail::case_3_dotdot_root_canonicalize ... ok
test path::test::from_host_path_fail::case_4_dotdot_root_no_canonicalize ... ok
test path::test::from_host_path_fail::case_5_invalid_name ... ok
test path::test::from_str::case_1_empty ... ok
test path::test::from_str::case_2 ... ok
test path::test::from_str::case_3 ... ok
test path::test::from_str_fail::case_03_two_forward_slashes_middle ... ok
test path::test::from_str_fail::case_04_trailing_slash ... ok
test path::test::from_str::case_4 ... ok
test path::test::from_str::case_5_cursed ... ok
test path::test::from_str::case_6_cursed ... ok
test path::test::from_str_fail::case_01_absolute ... ok
test path::test::from_str_fail::case_02_two_forward_slashes_start ... ok
test path::test::from_str_fail::case_05_dot ... ok
test path::test::from_str_fail::case_06_dotdot ... ok
test path::test::from_str_fail::case_08_dotdot_start ... ok
test path::test::from_str_fail::case_07_dot_start ... ok
test path::test::from_str_fail::case_09_dot_middle ... ok
test path::test::from_str_fail::case_10_dotdot_middle ... ok
test path::test::from_str_fail::case_11_dot_end ... ok
test path::test::from_str_fail::case_12_dotdot_end ... ok
test path::test::from_str_fail::case_13_null ... ok
test directoryservice::combinators::tests::no_backfill_reports_far_layer_and_preserves_near_miss ... ok
test directoryservice::combinators::tests::cache_mode_preserves_far_to_near_backfill ... ok
test path::test::join_push::case_2 ... ok
test path::test::join_push_fail::case_1 ... ok
test blobservice::from_addr::tests::test_from_addr_tokio::case_15_objectstore_valid_http_url ... ok
test directoryservice::tests::test_non_exist::case_1_memory ... ok
test path::test::join_push_fail::case_3 ... ok
test path::test::join_push_fail::case_4 ... ok
test directoryservice::tests::upload_reject_dangling_pointer::case_1_memory ... ok
test path::test::join_push_fail::case_6 ... ok
test path::test::join_push_fail::case_7 ... ok
test path::test::join_push_fail::case_5 ... ok
test path::test::join_push_fail::case_8 ... ok
test path::test::no_parent ... ok
test path::test::parent::case_1 ... ok
test path::test::parent::case_2 ... ok
test path::test::parent::case_3 ... ok
test path::test::join_push_fail::case_2 ... ok
test path::test::parent::case_4 ... ok
test proto::tests::convert_symlink_empty_target_invalid ... ok
test proto::tests::directory::size_unchecked_saturate ... ignored
test proto::tests::convert_anonymous_with_name_fail ... ok
test proto::tests::directory::size_checked ... ok
test proto::tests::directory::size ... ok
test proto::tests::directory::validate_empty ... ok
test proto::tests::directory::validate_invalid_digest ... ok
test proto::tests::directory::validate_invalid_names ... ok
test proto::tests::postcard_roundtrip::empty_directory_roundtrip ... ok
test proto::tests::directory::validate_sorting ... ok
test proto::tests::postcard_roundtrip::directory_roundtrip ... ok
test proto::tests::postcard_roundtrip::entry_roundtrip ... ok
test proto::tests::convert_symlink_target_null_byte_invalid ... ok
test proto::tests::postcard_roundtrip::stat_blob_response_roundtrip ... ok
test proto::url::test::roundtrip::case_1_directory_complicated ... ok
test refscan::tests::test_no_patterns ... ok
test proto::url::test::roundtrip::case_2_blob_helloworld ... ok
test refscan::tests::test_multiple_matches ... ok
test proto::tests::directory::size_unchecked_panic - should panic ... ok
test refscan::tests::test_single_match ... ok
test refscan::tests::test_reference_reader_no_patterns ... ok
test proto::tests::directory::digest ... ok
test blobservice::object_store::test::test_chunk_and_upload::case_2_b ... ok
test refscan::tests::test_reference_reader::case_2_small_capacity ... ok
test directoryservice::tests::upload_reject_unconnected::case_1_memory ... ok
test refscan::tests::test_reference_reader::case_1_normal ... ok
test refscan::tests::test_reference_reader::case_4_all_small ... ok
test refscan::tests::test_reference_reader::case_3_small_read ... ok
test directoryservice::from_addr::tests::test_from_addr_tokio::case_07_redb_memory_valid ... ok
test directoryservice::combinators::tests::no_backfill_rejects_closure_above_bound_before_returning_data ... ok
test import::test::test_ingestion::case_1_single_file ... ok
test import::test::test_ingestion::case_2_single_symlink ... ok
test directoryservice::tests::put_get_foo::case_1_memory ... ok
test directoryservice::tests::put_get_multiple_dedup::case_1_memory ... ok
test import::test::test_end_of_stream::case_1_empty_entries ... ok
test directoryservice::tests::put_get::case_1_memory ... ok
test path::test::join_push::case_1 ... ok
test directoryservice::tests::put_get_multiple_success::case_1_memory ... ok
test import::test::test_end_of_stream::case_2_missing_intermediate_dir ... ok
test directoryservice::traversal::descend_to::tests::test_descend_to ... ok
test import::test::test_ingestion::case_3_single_dir ... ok
test import::test::test_ingestion_fail::case_1_leaf_after_parent - should panic ... ok
test import::test::test_ingestion::case_4_dir_with_keep ... ok
test import::test::test_ingestion::case_5_directory_complicated ... ok
test import::test::test_ingestion_fail::case_2_root_in_entry - should panic ... ok
test tests::import::symlink ... ok
test tests::import::single_file ... ok
test tests::import::complicated ... ok

test result: ok. 252 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.05s


exit_status=0
```

## `SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo test -p snix-tracing --lib`

```text
warning: /home/brittonr/git/OnixResearch/mantle/.pi/worktrees/cairn-audit-repair-split-snix-20260808T050243Z/Cargo.toml: file `/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/cairn-audit-repair-split-snix-20260808T050243Z/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling memchr v2.8.0
   Compiling smallvec v1.15.1
   Compiling libc v0.2.186
   Compiling syn v2.0.117
   Compiling vte v0.15.0
   Compiling vt100 v0.16.2
   Compiling errno v0.3.14
   Compiling console v0.16.3
   Compiling mio v1.2.0
   Compiling signal-hook-registry v1.4.8
   Compiling indicatif v0.18.4
   Compiling tracing-attributes v0.1.31
   Compiling tokio-macros v2.7.0
   Compiling thiserror-impl v2.0.18
   Compiling tokio v1.51.0
   Compiling tracing v0.1.44
   Compiling thiserror v2.0.18
   Compiling tracing-subscriber v0.3.23
   Compiling tracing-indicatif v0.3.14
   Compiling snix-tracing v0.1.0 (/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/cairn-audit-repair-split-snix-20260808T050243Z/vendor/snix-tracing)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 4.54s
     Running unittests src/lib.rs (/tmp/mantle-snix-backport-baseline-target/debug/deps/snix_tracing-8739b3c1461a694a)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


exit_status=0
```

## `SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo test -p crunch-store --lib`

```text
warning: /home/brittonr/git/OnixResearch/mantle/.pi/worktrees/cairn-audit-repair-split-snix-20260808T050243Z/Cargo.toml: file `/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/cairn-audit-repair-split-snix-20260808T050243Z/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling serde_json v1.0.149
   Compiling regex-syntax v0.8.10
   Compiling aho-corasick v1.1.4
   Compiling rustix v1.1.4
   Compiling linux-raw-sys v0.12.1
   Compiling tokio v1.51.0
   Compiling num-traits v0.2.19
   Compiling darling_core v0.20.11
   Compiling getrandom v0.3.4
   Compiling num_cpus v1.17.0
   Compiling proc-macro-error-attr2 v2.0.0
   Compiling snix-castore v0.1.0 (/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/cairn-audit-repair-split-snix-20260808T050243Z/vendor/snix-castore)
   Compiling nix v0.29.0
   Compiling strum_macros v0.26.4
   Compiling typed-builder-macro v0.22.0
   Compiling strum v0.26.3
   Compiling snix-build v0.1.0 (/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/cairn-audit-repair-split-snix-20260808T050243Z/vendor/snix-build)
   Compiling itertools v0.12.1
   Compiling uuid v1.23.0
   Compiling artifact-auth-core v0.1.0 (ssh://git@github.com/OnixResearch/onix-artifact.git?rev=c932138d880ddf4c2967f4c024b489b5c0022bf1#c932138d)
   Compiling filetime v0.2.27
   Compiling threadpool v1.8.1
   Compiling proc-macro-error2 v2.0.1
   Compiling ppv-lite86 v0.2.21
   Compiling typed-builder v0.22.0
   Compiling crunch-gc-core v0.1.0 (/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/cairn-audit-repair-split-snix-20260808T050243Z/crates/crunch-gc-core)
   Compiling getset v0.1.6
   Compiling crunch-overlay-core v0.1.0 (/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/cairn-audit-repair-split-snix-20260808T050243Z/crates/crunch-overlay-core)
   Compiling crunch-repair-core v0.1.0 (/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/cairn-audit-repair-split-snix-20260808T050243Z/crates/crunch-repair-core)
   Compiling fs2 v0.4.3
   Compiling wait-timeout v0.2.1
   Compiling darling_macro v0.20.11
   Compiling quick-error v1.2.3
   Compiling bit-vec v0.8.0
   Compiling unarray v0.1.4
   Compiling libbz2-rs-sys v0.2.3
   Compiling darling v0.20.11
   Compiling regex-automata v0.4.14
   Compiling yansi v1.0.1
   Compiling derive_builder_core v0.20.2
   Compiling diff v0.1.13
   Compiling rand_core v0.9.5
   Compiling crunch-delta-core v0.1.0 (/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/cairn-audit-repair-split-snix-20260808T050243Z/crates/crunch-delta-core)
   Compiling bit-set v0.8.0
   Compiling pretty_assertions v1.4.1
   Compiling rand_xorshift v0.4.0
   Compiling rand v0.9.3
   Compiling derive_builder_macro v0.20.2
   Compiling derive_builder v0.20.2
   Compiling rand_chacha v0.9.0
   Compiling bzip2 v0.6.1
   Compiling chrono v0.4.44
   Compiling crunch-attestation-core v0.1.0 (/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/cairn-audit-repair-split-snix-20260808T050243Z/crates/crunch-attestation-core)
   Compiling crunch-action-result-core v0.1.0 (/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/cairn-audit-repair-split-snix-20260808T050243Z/crates/crunch-action-result-core)
   Compiling xattr v1.6.1
   Compiling tempfile v3.27.0
   Compiling nix-archive v0.1.0
   Compiling tar v0.4.45
   Compiling rusty-fork v0.3.1
   Compiling crunch-attestation v0.1.0 (/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/cairn-audit-repair-split-snix-20260808T050243Z/crates/crunch-attestation)
   Compiling proptest v1.11.0
   Compiling matchers v0.2.0
   Compiling bstr v1.12.1
   Compiling regex v1.12.3
   Compiling tracing-subscriber v0.3.23
   Compiling oci-spec v0.7.1
   Compiling tokio-util v0.7.18
   Compiling tower v0.5.3
   Compiling tokio-rustls v0.26.4
   Compiling tokio-stream v0.1.18
   Compiling async-compression v0.4.19
   Compiling nix-compat v0.1.0 (/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/cairn-audit-repair-split-snix-20260808T050243Z/vendor/nix-compat)
   Compiling tracing-indicatif v0.3.14
   Compiling astral-tokio-tar v0.6.3
   Compiling fastcdc v3.2.1
   Compiling tower-http v0.6.8
   Compiling h2 v0.4.13
   Compiling n0-future v0.3.2
   Compiling irpc v0.13.0
   Compiling crunch-nar v0.1.0 (/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/cairn-audit-repair-split-snix-20260808T050243Z/crates/crunch-nar)
   Compiling hyper v1.9.0
   Compiling hyper-util v0.1.20
   Compiling hyper-rustls v0.27.7
   Compiling reqwest v0.13.2
   Compiling reqwest-middleware v0.5.1
   Compiling object_store v0.14.0
   Compiling reqwest-tracing v0.6.0
   Compiling snix-tracing v0.1.0 (/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/cairn-audit-repair-split-snix-20260808T050243Z/vendor/snix-tracing)
   Compiling snix-store v0.1.0 (/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/cairn-audit-repair-split-snix-20260808T050243Z/vendor/snix-store)
   Compiling crunch-store v0.1.0 (/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/cairn-audit-repair-split-snix-20260808T050243Z/crates/crunch-store)
   Compiling crunch-delta v0.1.0 (/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/cairn-audit-repair-split-snix-20260808T050243Z/crates/crunch-delta)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 28.59s
     Running unittests src/lib.rs (/tmp/mantle-snix-backport-baseline-target/debug/deps/crunch_store-92703f72f1022c65)

running 338 tests
test action_result::tests::offline_discovery_never_opens_remote_sources ... ok
test action_result::tests::aggregate_candidate_limit_rejects_the_offending_source_response ... ok
test archive::tests::import_action_core_rejects_untrusted_and_skips_present ... ok
test archive::tests::archive_list_rejects_header_end_count_mismatch ... ok
test archive::tests::archive_list_rejects_bad_magic ... ok
test action_result::tests::source_limit_bounds_empty_or_failing_remote_sources ... ok
test action_result::tests::duplicate_detached_signatures_are_rejected_before_publication ... ok
test archive::tests::pathinfo_fixture_service_is_bounded ... ok
test action_result::tests::local_interrupted_publication_is_not_discoverable ... ok
test action_result::tests::local_conflicting_existing_record_is_not_overwritten ... ok
test action_result::tests::local_dangling_or_poisoned_index_fails_closed ... ok
test action_result::tests::local_publish_is_atomic_no_clobber_and_duplicate_safe ... ok
test attestation::tests::artifact_attestation_file_uses_canonical_bytes ... ok
test build_io::tests::equal_length_replacement_preserves_binary_shape_and_occurrences ... ok
test action_result::tests::gc_retains_metadata_only_while_outputs_are_independently_live ... ok
test build_io::tests::missing_blob_hash_fails_closed ... ok
test action_result::tests::discovery_only_base_is_readable_but_never_receives_publication ... ok
test build_io::tests::read_file_node_rejects_non_file_and_over_limit_inputs ... ok
test build_io::tests::unequal_length_replacement_is_rejected - should panic ... ok
test attestation::tests::artifact_attestation_round_trips_by_logical_store_path ... ok
test ca_mapping::tests::insert_and_get ... ok
test ca_mapping::tests::load_missing_file_returns_empty ... ok
test capability::tests::missing_output_does_not_create_a_root ... ok
test ca_mapping::tests::save_and_load_roundtrip ... ok
test closure::tests::cycle_terminates ... ok
test attestation::tests::runtime_closure_attestation_uses_stored_artifact_digests ... ok
test attestation::tests::runtime_closure_attestation_synthesizes_missing_member_artifact ... ok
test attestation::tests::runtime_closure_attestation_refreshes_after_member_digest_changes ... ok
test capability::tests::output_lookup_and_selected_root_registration_share_exact_identity ... ok
test attestation::tests::runtime_closure_attestation_is_cached_by_root_selection ... ok
test closure::tests::diamond_dedup ... ok
test closure::tests::linear_chain ... ok
test closure::tests::local_query_error_can_still_use_remote_refs ... ok
test closure::tests::practical_mode_records_degraded_root_lookup ... ok
test closure::tests::practical_mode_records_degraded_child_lookup ... ok
test closure::tests::remote_fallback ... ok
test closure::tests::self_reference ... ok
test closure::tests::remote_metadata_lookup_failure_degrades_practical_mode ... ok
test closure::tests::strict_mode_rejects_missing_root_closure_facts ... ok
test completeness::tests::blob_node_requires_blob_presence ... ok
test completeness::tests::blob_node_rejects_declared_size_mismatch ... ok
test closure::tests::single_path_no_refs ... ok
test completeness::tests::chunked_blob_metadata_must_match_declared_size ... ok
test completeness::tests::completeness_rechecks_and_rejects_removed_directory ... ok
test completeness::tests::directory_with_missing_blob_child_is_incomplete ... ok
test completeness::tests::symlink_is_always_complete ... ok
test completeness::tests::empty_directory_requires_existence ... ok
test completeness::tests::bounded_depth_rejects_extremely_deep_trees ... ok
test completeness::tests::node_visit_limit_accepts_last_supported_node_and_rejects_overflow ... ok
test archive::tests::archive_export_rejects_stale_final_nar_facts_before_writing ... ok
test archive::tests::missing_closure_reference_fails_export_plan ... ok
test archive::tests::archive_export_refuses_unsigned_without_escape_hatch ... ok
test action_result::tests::http_urls_strip_query_fragment_and_preserve_cache_subpath ... ok
test closure::tests::depth_limit_enforced ... ok
test archive::tests::ca_path_identity_accepts_reference_aware_standard_path ... ok
test archive::tests::export_closure_includes_references_deterministically ... ok
test archive::tests::archive_export_list_round_trip_preserves_metadata_before_payload ... ok
test archive::tests::archive_list_drains_non_seekable_payloads_in_bounded_chunks ... ok
test archive::tests::archive_import_rejects_unsupported_ca_metadata_without_persisting ... ok
test build_io::tests::blob_and_nar_hashes_cover_all_supported_algorithms ... ok
test build_io::tests::rewrite_and_nar_hash_preserve_named_operation_behavior ... ok
test archive::tests::archive_import_rejects_conflicting_local_pathinfo ... ok
test archive::tests::archive_import_rejects_truncated_payload_without_persisting ... ok
test archive::tests::archive_import_rejects_store_prefix_mismatch_before_persisting ... ok
test archive::tests::archive_import_rejects_untrusted_signature_without_persisting ... ok
test archive::tests::archive_import_rejects_ca_metadata_for_another_store_path ... ok
test gc::tests::file_cleanup_reports_failure_after_attempting_later_independent_paths ... ok
test archive::tests::archive_import_rejects_existing_path_with_stale_final_nar_facts ... ok
test archive::tests::archive_import_rejects_tampered_payload_without_persisting ... ok
test archive::tests::archive_round_trip_preserves_distinct_marker_ca_and_final_nar_identities ... ok
test archive::tests::archive_import_round_trip_and_skip_existing_are_idempotent ... ok
test gc::tests::path_explanation_rejects_retaining_root_links_above_policy_limit ... ok
test export::tests::export_file_empty_content ... ok
test export::tests::export_directory_sets_permissions_and_mtime ... ok
test gc::tests::reclaim_observation_preserves_unknown_bytes_and_plan_identity_binds_shape ... ok
test export::tests::export_file_creates_parent_directories ... ok
test export::tests::export_directory_with_symlink ... ok
test gc::tests::remove_path_accepts_an_already_missing_export ... ok
test export::tests::export_directory_creates_files ... ok
test export::tests::export_file_non_executable_sets_permissions ... ok
test gc::tests::remove_path_does_not_follow_a_symlink_outside_the_export_tree ... ok
test gc::tests::remove_path_removes_nested_read_only_export_tree ... ok
test gc::tests::snapshot_pathinfos_finishes_before_later_mutation ... ok
test export::tests::export_file_writes_content ... ok
test handle::tests::action_result_nar_byte_accounting_distinguishes_transfer_reuse_and_overflow ... ok
test export::tests::export_file_executable_sets_permissions ... ok
test gc::tests::gc_aborts_when_root_registry_is_corrupt ... ok
test export::tests::export_file_sets_mtime ... ok
test gc::tests::operation_reporting_preserves_first_failure_and_records_later_work ... ok
test export::tests::export_missing_directory_returns_error ... ok
test export::tests::export_missing_blob_returns_error ... ok
test export::tests::export_file_rejects_short_blob ... ok
test handle::tests::delta_capability_url_preserves_cache_subpath_with_trailing_slash ... ok
test handle::tests::delta_capability_url_preserves_cache_subpath_without_trailing_slash ... ok
test handle::tests::delta_capability_url_uses_root_cache_authority ... ok
test action_result::tests::http_interrupted_index_publication_leaves_record_undiscoverable ... ok
test export::tests::export_nested_directory ... ok
test handle::tests::local_protocol_v1_matches_crunch_delta_wire_contract ... ok
test export::tests::export_symlink_creates_link ... ok
test gc::tests::explicit_castore_root_survives_while_unreachable_blob_is_reclaimed ... ok
test gc::tests::gc_aborts_when_retained_root_pathinfo_is_missing ... ok
test gc::tests::directory_outputs_survive_reopen_and_gc ... ok
test export::tests::export_symlink_sets_lmtime ... ok
test export::tests::export_moderate_depth_succeeds ... ok
test gc::tests::pathinfo_rewrite_failure_stops_before_export_deletion ... ok
test gc::tests::stale_plan_is_rejected_after_root_change_without_deletion ... ok
test handle::tests::overlay_missing_base_fails_closed ... ok
test gc::tests::gc_operation_order_matches_design ... ok
test gc::tests::dry_run_reports_same_candidates_as_real_run ... ok
test action_result::tests::http_corrupt_record_and_poisoned_index_fail_closed ... ok
test handle::tests::overlay_duplicate_base_declaration_fails_before_overlay_creation ... ok
test gc::tests::stale_plan_is_rejected_after_export_symlink_substitution ... ok
test handle::tests::cached_node_for_path_rejects_incomplete_session_node ... ok
test gc::tests::shared_blob_survives_when_reachable_path_still_references_it ... ok
test build_io::tests::host_path_hash_is_deterministic_and_missing_paths_fail_closed ... ok
test gc::tests::retained_root_keeps_transitive_closure_and_sidecars ... ok
test handle::tests::cached_node_for_path_reuses_local_pathinfo_node ... ok
test handle::tests::action_result_admission_preserves_current_detailed_artifact_attestation ... ok
test gc::tests::unreachable_output_removes_pathinfo_exports_and_attestations ... ok
test handle::tests::check_cache_accepts_ca_mapping_with_custom_store_prefix ... ok
test handle::tests::failed_persist_does_not_register_root ... ok
test handle::tests::check_cache_preserves_current_detailed_artifact_attestation ... ok
test handle::tests::check_cache_directory_output_with_missing_child_is_castore_incomplete ... ok
test handle::tests::check_cache_replaces_stale_artifact_attestation_facts ... ok
test handle::tests::overlay_base_state_mutation_blocks_output_admission ... ok
test handle::tests::noop_publisher_is_default_and_skips_all_outputs ... ok
test handle::tests::overlay_prefix_mismatch_fails_closed ... ok
test handle::tests::overlay_rejects_base_with_write_permission ... ok
test handle::tests::overlay_generation_drift_blocks_later_read ... ok
test action_result::tests::http_publication_is_record_first_discoverable_and_duplicate_safe ... ok
test handle::tests::overlay_ca_mapping_precedence_and_publication_are_layer_bounded ... ok
test handle::tests::overlay_incomplete_shadow_blocks_complete_base_fallback ... ok
test handle::tests::overlay_shadowed_path_does_not_inherit_base_trust ... ok
test handle::tests::overlay_read_through_base_hit_does_not_mutate_overlay ... ok
test handle::tests::overlay_writes_route_to_overlay_only ... ok
test handle::tests::persist_signed_output_rejects_store_path_mismatch ... ok
test handle::tests::overlay_gc_execution_rejects_stale_base_generation ... ok
test handle::tests::overlay_rejects_base_pathinfo_with_invalid_layer_signature ... ok
test handle::tests::persist_signed_output_rejects_unsigned_pathinfo ... ok
test handle::tests::persist_signed_output_writes_artifact_attestation ... ok
test handle::tests::persist_signed_output_registers_build_root ... ok
test handle::tests::remote_trusted_key_parser_accepts_indexed_keys_and_rejects_duplicate_indexes ... ok
test handle::tests::persist_signed_output_registers_self_build_root ... ok
test handle::tests::practical_pathinfo_open_fallback_records_audit_event ... ok
test handle::tests::resolve_same_authority_endpoint_rejects_cross_origin_urls ... ok
test handle::tests::strict_pathinfo_open_fallback_is_rejected ... ok
test handle::tests::persistent_output_calls_configured_publisher ... ok
test handle::tests::persistent_output_does_not_fail_on_publisher_error ... ok
test handle::tests::overlay_gc_rejects_base_to_overlay_reference ... ok
test handle::tests::overlay_shadows_base_pathinfo ... ok
test handle::tests::overlay_gc_retains_base_reachability_without_base_mutation ... ok
test handle::tests::remote_substitution_registers_bootstrap_root ... ok
test handle::tests::remote_substitution_without_cache_url_skips_probe_and_full_fetches ... ok
test http_closure::tests::conflicting_digest_path_identity_is_rejected ... ok
test http_closure::tests::depth_limit_fails_closed ... ok
test handle::tests::try_substitute_remote_negative_miss_prevents_repeat_probe ... ok
test http_closure::tests::diamond_and_cycle_are_deduplicated ... ok
test http_closure::tests::duplicate_reference_is_rejected ... ok
test http_closure::tests::incomplete_plan_and_active_request_fail_closed ... ok
test http_closure::tests::invalid_limits_and_identity_are_rejected ... ok
test handle::tests::root_export_refreshes_stale_materialized_path ... ok
test http_closure::tests::linear_plan_imports_root_last ... ok
test http_closure::tests::member_limit_fails_closed ... ok
test http_closure::tests::observation_without_active_request_is_rejected ... ok
test handle::tests::remote_substitution_writes_artifact_attestation ... ok
test handle::tests::try_substitute_remote_records_metadata_cache_on_hit ... ok
test http_closure::tests::one_member_plan_is_stable_and_root_last ... ok
test http_closure::tests::reference_limit_fails_before_pending_members_change ... ok
test layer::tests::layered_value_preserves_exact_index_through_map ... ok
test http_closure::tests::total_nar_size_limit_fails_closed ... ok
test http_closure::tests::returned_path_mismatch_is_rejected ... ok
test layer::tests::store_layer_booleans_are_disjoint ... ok
test layer::tests::store_layer_display_includes_exact_base_index ... ok
test http_closure::tests::reference_order_does_not_change_plan_identity ... ok
test layer::tests::zero_service_index_is_overlay ... ok
test metadata_cache::tests::force_refresh_disables_get ... ok
test metadata_cache::tests::load_empty_cache_from_nonexistent_file ... ok
test metadata_cache::tests::load_corrupt_file_returns_empty ... ok
test metadata_cache::tests::put_and_get_roundtrip ... ok
test metadata_cache::tests::evict_expired_removes_only_expired_entries ... ok
test metadata_cache::tests::put_replaces_existing_entry ... ok
test metadata_cache::tests::remove_returns_false_for_missing_key ... ok
test metadata_cache::tests::remove_removes_existing_entry ... ok
test overlay::tests::embedded_overlay_policy_is_typed_and_bounded ... ok
test mutation_lock::tests::try_acquire_rejects_second_mutator ... ok
test metadata_cache::tests::save_and_reload_persists_entries ... ok
test metadata_cache::tests::save_evicts_excess_entries ... ok
test path_identity::tests::marker_normalized_ca_path_is_accepted ... ok
test archive::tests::large_archive_payload_stays_on_the_chunked_castore_ingest_path ... ok
test path_identity::tests::mismatched_ca_path_is_rejected ... ok
test overlay::tests::identity_record_rejects_prefix_drift ... ok
test provenance::tests::byte_reference_admission_requires_a_valid_store_path_digest ... ok
test overlay::tests::generation_observation_rejects_symlink_members ... ok
test overlay::tests::writable_base_is_rejected_before_generation_admission ... ok
test provenance::tests::classifier_covers_data_elf_script_and_unknown_executable ... ok
test provenance::tests::cpio_and_gzip_initrd_readers_classify_nested_executable_scripts ... ok
test provenance::tests::equivalent_observation_order_canonicalizes_deterministically ... ok
test provenance::tests::exact_reference_resolution_accepts_declared_target_and_rejects_foreign_unknown_and_escape ... ok
test provenance::tests::generic_compressed_streams_are_bounded_and_scanned_as_single_payloads ... ok
test handle::tests::verified_local_output_adoption_rejects_a_missing_physical_path ... ok
test handle::tests::verified_local_output_adoption_ingests_signs_and_persists ... ok
test provenance::tests::identity_shape_is_bounded_and_stable ... ok
test provenance::tests::named_limits_all_fail_closed ... ok
test provenance::tests::malformed_and_bounded_containers_fail_closed ... ok
test provenance::tests::policy_accepts_bounded_sorted_profile_paths_and_rejects_bad_limits ... ok
test provenance::tests::preserved_identity_paths_are_targets_not_untranslated_foreign_references ... ok
test provenance::tests::shebang_accepts_profile_and_rejects_relative_missing_and_non_utf8_targets ... ok
test provenance::tests::symlink_resolution_rejects_escape_missing_and_loop ... ok
test provenance::tests::tar_reader_finds_hidden_unclassified_executable_and_path_escape ... ok
test provenance::tests::castore_scan_accepts_complete_signed_blob_without_host_fallback ... ok
test provenance::tests::castore_directory_scan_detects_symlink_loop_without_following_links ... ok
test provenance::tests::castore_scan_rejects_untrusted_and_receipt_inconsistent_pathinfo_before_blob_reads ... ok
test provenance::tests::castore_scan_reports_missing_blob_before_claiming_complete_traversal ... ok
test handle::tests::overlay_reads_file_and_directory_content_from_distinct_bases_without_backfill ... ok
test handle::tests::verified_source_ingest_rejects_conflicting_existing_content ... ok
test handle::tests::verified_local_output_adoption_rejects_changed_existing_content ... ok
test provenance::tests::castore_scan_counts_duplicate_nodes_and_enforces_duplicate_limit ... ok
test handle::tests::try_substitute_remote_fallback_to_subsequent_url_when_primary_missing ... ok
test handle::tests::verified_source_ingest_preserves_exact_path_and_reuses_matching_content ... ok
test action_result::tests::http_timeout_rejects_source_without_fabricating_lookup ... ok
test handle::tests::overlay_report_records_selected_base_descriptor ... ok
test export::tests::export_depth_limit_enforced ... ok
test handle::tests::overlay_two_bases_stack_in_declaration_order ... ok
test handle::tests::remote_substitution_404_delta_probe_falls_back_to_full_fetch ... ok
test handle::tests::remote_substitution_cross_authority_capability_falls_back_to_full_fetch ... ok
test handle::tests::remote_substitution_malformed_delta_capability_json_falls_back_to_full_fetch ... ok
test handle::tests::remote_substitution_probe_error_falls_back_to_full_fetch ... ok
test provenance::tests::lexical_relative_resolution_never_returns_a_path_outside_root ... ok
test handle::tests::remote_substitution_directory_delta_without_local_directory_closure_falls_back_to_full_fetch ... ok
test handle::tests::delta_and_full_substitution_record_same_attestation_and_root_metadata ... ok
test pull::tests::http_closure_path_mismatch_fails_before_nar_download ... ok
test handle::tests::remote_substitution_malformed_stream_json_falls_back_to_full_fetch ... ok
test handle::tests::remote_substitution_accepts_delta_chunk_stream_without_full_fetch ... ok
test handle::tests::remote_substitution_closure_scoped_delta_accepts_requested_output_and_reports_reuse ... ok
test handle::tests::remote_substitution_missing_local_backing_content_is_absent_from_receiver_manifest ... ok
test handle::tests::remote_substitution_receiver_manifest_stays_bounded_to_requested_output ... ok
test handle::tests::remote_substitution_untrusted_delta_pathinfo_falls_back_to_full_fetch ... ok
test pull::tests::http_closure_duplicate_reference_fails_before_nar_download ... ok
test pull::tests::http_closure_narinfo_limit_fails_before_nar_download ... ok
test pull::tests::http_closure_plan_validator_rejects_before_nar_download_or_store_mutation ... ok
test pull::tests::http_closure_total_nar_limit_fails_before_nar_download ... ok
test pull::tests::http_closure_untrusted_root_fails_before_nar_download ... ok
test pull::tests::http_closure_missing_dependency_fails_before_nar_download ... ok
test pull::tests::http_closure_dependency_content_failure_keeps_root_absent ... ok
test pull::tests::http_pull_detects_store_path_mismatch_and_client_metadata ... ok
test pull::tests::http_pull_rejects_cache_base_url_userinfo ... ok
test pull::tests::http_pull_accepts_unknown_key_signature_when_trust_unsigned ... ok
test handle::tests::remote_substitution_probes_delta_capability_once_per_session ... ok
test export::tests::export_file_allows_many_small_reads ... ok
test pull::tests::http_pull_accepts_unsigned_when_trust_unsigned ... ok
test pull::tests::http_pull_blocks_cross_scheme_redirects ... ok
test pull::tests::http_pull_continues_after_narinfo_fetch_transport_failure ... ok
test pull::tests::http_pull_does_not_recurse_into_missing_references ... ok
test pull::tests::http_pull_export_failure_after_persistence_is_fatal ... ok
test pull::tests::http_closure_changed_dependency_nar_keeps_closure_absent ... ok
test pull::tests::http_pull_handles_compressed_xz_nar ... ok
test pull::tests::http_closure_pull_discovers_all_metadata_and_imports_root_last ... ok
test pull::tests::pull_accepts_unsigned_when_trust_unsigned ... ok
test pull::tests::pull_nonexistent_source_returns_error ... ok
test pull::tests::http_pull_maps_narinfo_http_403_to_missing_nar_count ... ok
test pull::tests::http_pull_maps_narinfo_http_5xx_to_parse_error_count ... ok
test handle::tests::remote_substitution_stream_failure_falls_back_through_real_http_cache ... ok
test pull::tests::http_closure_refetches_incomplete_local_dependency ... ok
test pull::tests::pull_detects_nar_hash_mismatch ... ok
test push::tests::deriver_normalization_rejects_an_empty_base_name ... ok
test pull::tests::pull_rejects_store_dir_mismatch ... ok
test pull::tests::http_pull_maps_nar_http_failure_to_missing_nar_count ... ok
test push::tests::push_custom_store_dir_narinfo_uses_correct_prefix ... ok
test pull::tests::pull_rejects_untrusted_signature ... ok
test pull::tests::http_pull_nix_cache_info_redirect_rejection_warns_and_proceeds ... ok
test push::tests::push_includes_unsigned_when_trusted ... ok
test push::tests::push_narinfo_references_match ... ok
test push::tests::push_idempotent_skip ... ok
test pull::tests::pull_single_signed_path_round_trip ... ok
test pull::tests::pull_skips_missing_nar ... ok
test pull::tests::pull_skips_already_present ... ok
test pull::tests::http_pull_rejects_malformed_narinfo_text ... ok
test query::tests::store_sign_adds_signature ... ok
test query::tests::store_sign_replaces_same_key_signature ... ok
test query::tests::store_sign_appends_different_key_signature ... ok
test query::tests::store_sign_all_skips_already_signed_entries ... ok
test query::tests::store_verify_missing_for_path_not_on_disk_in_custom_store_dir ... ok
test query::tests::store_verify_mismatch_for_tampered_disk_content ... ok
test query::tests::store_verify_read_failure_does_not_persist_pathinfo ... ok
test query::tests::store_verify_ok_for_exported_path_in_custom_store_dir ... ok
test push::tests::push_multiple_paths ... ok
test query::tests::verify_signatures_reports_untrusted_signer ... ok
test push::tests::push_normalizes_deriver_suffix_and_writes_parseable_narinfo ... ok
test pull::tests::http_pull_rejects_absolute_nar_url_without_download ... ok
test query::tests::store_sign_then_verify_roundtrips_under_custom_prefix ... ok
test push::tests::push_skips_unsigned_by_default ... ok
test query::tests::store_verify_signatures_rejects_wrong_prefix ... ok
test query::tests::verify_signatures_accepts_trusted_key ... ok
test repair::tests::pure_plan_distinguishes_current_and_stale_facts ... ok
test repair::tests::pure_plan_rejects_each_unsafe_candidate ... ok
test push::tests::push_preserves_existing_nix_cache_info ... ok
test push::tests::push_single_signed_path ... ok
test provenance::tests::payload_classification_is_deterministic_for_arbitrary_bytes ... ok
test retention::tests::embedded_policy_is_typed_and_bounded ... ok
test roots::tests::corrupt_registry_is_rejected_without_replacement ... ok
test roots::tests::legacy_record_migrates_to_protected_unmanaged_state ... ok
test roots::tests::pin_rejects_nonexistent_and_unreadable_paths ... ok
test pull::tests::pull_multiple_paths ... ok
test roots::tests::managed_generation_batch_is_atomic_and_shares_generation_number ... ok
test roots::tests::project_generation_reuses_identity_and_advances_new_identity ... ok
test roots::tests::unmanaged_remote_registration_uses_remote_owner_scope ... ok
test roots::tests::shell_lease_renewal_is_bounded_and_requires_lease_facts ... ok
test roots::tests::unpin_removes_existing_versioned_record ... ok
test roots::tests::register_root_survives_reload_with_versioned_provenance ... ok
test repair::tests::missing_exact_pathinfo_is_rejected ... ok
test repair::tests::dry_run_does_not_mutate_stale_pathinfo_or_attestation ... ok
test repair::tests::current_pathinfo_is_an_idempotent_no_op ... ok
test pull::tests::pull_with_path_filter ... ok
test repair::tests::incomplete_content_is_rejected_without_pathinfo_mutation ... ok
test repair::tests::invalid_ca_identity_is_rejected_without_pathinfo_mutation ... ok
test pull::tests::http_pull_store_dir_mismatch_is_hard_error_before_narinfo_fetch ... ok
test pull::tests::http_pull_rejects_narinfo_store_path_prefix_mismatch_after_matching_preflight ... ok
test repair::tests::execute_repairs_facts_replaces_signatures_and_preserves_attestation_graph ... ok
test repair::tests::unsigned_stale_pathinfo_is_rejected_without_mutation ... ok
test pull::tests::http_pull_parses_references_with_local_store_prefix ... ok
test repair::tests::stale_artifact_attestation_is_rejected_without_pathinfo_mutation ... ok
test pull::tests::http_pull_rejects_unsigned_when_trust_unsigned_is_false ... ok
test pull::tests::http_closure_diamond_fetches_shared_member_once ... ok
test pull::tests::http_pull_rejects_untrusted_signature ... ok
test repair::tests::pathinfo_persistence_failure_is_reported_without_mutation ... ok
test pull::tests::http_pull_skips_missing_narinfo_404 ... ok
test pull::tests::http_pull_pathinfo_persistence_failure_is_fatal ... ok
test pull::tests::http_closure_pull_reuses_complete_dependency ... ok
test pull::tests::http_pull_network_failure_continues_for_remaining_paths ... ok
test pull::tests::http_pull_single_signed_path_round_trip ... ok
test pull::tests::http_pull_missing_and_malformed_nix_cache_info_both_proceed ... ok
test pull::tests::http_pull_skips_already_present_without_narinfo_request ... ok
test pull::tests::http_pull_nix_cache_info_client_error_warning_proceeds ... ok
test pull::tests::http_pull_maps_other_narinfo_http_statuses_to_parse_error_count ... ok
test pull::tests::http_pull_handles_gzip_bzip2_and_zstd_nar ... ok
test pull::tests::http_pull_rejects_malformed_or_wrong_prefix_references_before_persistence ... ok
test pull::tests::http_pull_maps_other_nar_http_statuses_to_missing_nar_count ... ok
test pull::tests::http_pull_keeps_path_prefixed_cache_urls_stable ... ok

test result: ok. 338 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.37s


exit_status=0
```

