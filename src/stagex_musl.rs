use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use serde::Serialize;

use crate::source_bundle::SourceFileType;
use crate::source_bundle::SourceRecord;
use crate::source_bundle::SourceRecordKind;
use crate::source_bundle::materialize_source_record_for_offline_use;
use crate::source_bundle::read_source_bundle;

const SOURCE_RECORD_NAME_KEY: &str = "name";
const MUSL_RECORD_NAME: &str = "musl-1.1.24-src";
const MUSL_SOURCE_OUTPUT_NAME: &str = "musl-1.1.24";
pub(crate) const MUSL_SOURCE_ARTIFACT_ID: &str = "musl-1.1.24-source";
pub(crate) const MUSL_SOURCE_CONTENT_BLAKE3: &str = "2abcaf25adf8f529b9f45e5549bfaa5cc5827305624c081e9388ac2ad828ba17";
pub(crate) const MUSL_RECIPE_SOURCE_ARTIFACT_ID: &str = "musl-1.1.24-recipe-source";
pub(crate) const MUSL_RECIPE_SOURCE_BLAKE3: &str = "cbcdc20f7e285b11921a9a0e0799363132296e9d1725d2a4e3baae9a7e24a9d9";
const MUSL_RECIPE_SOURCE: &[u8] = include_bytes!("../bootstrap/musl-1.1.24-tcc.ncl");
const MUSL_SOURCE_REPORT_FORMAT: &str = "mantle-stagex-musl-source-materialization-v1";
const MUSL_REPORT_FORMAT: &str = "mantle-stagex-musl-1.1.24-inventory-v1";
const MUSL_SOURCE_NON_CLAIM: &str =
    "musl source materialization proves authenticated offline archive identity and fixed-output parity only";
const MUSL_NON_CLAIM: &str = "this inventory binds the reduced first-musl static archive, headers, startup objects, and compile/link observations only; it does not prove a complete libc, dynamic runtime, or provider admission";
const MUSL_FILE_BYTES_MAX: u64 = 64 * 1_024 * 1_024;
const MUSL_TREE_ENTRY_COUNT_MAX: usize = 16_384;
const MUSL_HELPER_COUNT: usize = 24;
const MUSL_PASS2_HELPER_COUNT: usize = 28;
const MUSL_PASS2_RECIPE_SOURCE_ARTIFACT_ID: &str = "musl-pass2-recipe-source";
const MUSL_PASS2_RECIPE_SOURCE_BLAKE3: &str = "5f85fc84eb8eb235ca15d3179db8a3504668804f140d46f15fe7af2e53a0dec5";
const MUSL_PASS2_RECIPE_SOURCE: &[u8] = include_bytes!("../bootstrap/musl-1.1.24-tcc-musl.ncl");
const MUSL_PASS2_REPORT_FORMAT: &str = "mantle-stagex-musl-1.1.24-pass2-inventory-v1";
const MUSL_PASS2_NON_CLAIM: &str = "this inventory binds the second reduced musl static archive built by musl-linked TinyCC and bounded compile/link observations only; it does not prove a complete libc, dynamic runtime, or provider admission";
pub(crate) const MUSL_PASS2_CONFIGURED_SOURCE_BLAKE3: &str =
    "575e19d9e8f5d23be1ae0e87b9e40bebbaa3be7845632980586f38ef2d906e5d";
pub(crate) const MUSL_PASS2_LIBC_BLAKE3: &str = "86f238f807b2580bcb814ef914a64a288e34bb89101c8b5f094cd2a124febaed";
const MUSL_PASS2_CRT1_BLAKE3: &str = "c34258edb3d4de07a67e1d20c7dc5946543ce4ba078ace182b0b99cd4bbb7257";
const MUSL_PASS2_CRTI_BLAKE3: &str = "ecf4006e5ea51c3ad49a243165909cd4f9eddfbe181b576ac43513ede8578be4";
const MUSL_PASS2_CRTN_BLAKE3: &str = "b3b3153225cc100b8c429f5916ae36b96ef4903169b86f841f86498c0fd7251d";
const MUSL_PASS2_HEADERS_BLAKE3: &str = "4d5a63f48ef27c8377a5319d3d68cd1a2b724a974ee5d8281a67f8fb0b5dc1c7";
const MUSL_PASS2_SMOKE_BINARY_BLAKE3: &str = "30c18329ab3981639dba0dbdef48cd01beae52abab89c8b732f8f6eb1248fc57";
const MUSL_EMPTY_ARCHIVE_COUNT: u32 = 8;
const MUSL_SMOKE_COMMAND_COUNT: u32 = 3;
const MUSL_OUTPUT_COUNT: usize = 1 + 3 + MUSL_EMPTY_ARCHIVE_COUNT as usize + 1 + 1;
const EMPTY_ARCHIVE_BYTES: &[u8] = b"!<arch>\n";
const POSITIVE_SMOKE_SOURCE: &[u8] = b"#include <stddef.h>\nint main(void) { return sizeof(size_t) == 0; }\n";
const MALFORMED_SMOKE_SOURCE: &[u8] = b"int broken( {\n";
pub(crate) const MUSL_CONFIGURED_SOURCE_BLAKE3: &str =
    "d1d3f1b8c7b99ba3902a4b999f55ee4ba3410cfd7ebac9462176f2d1d7565dcf";
pub(crate) const MUSL_LIBC_BLAKE3: &str = "b3a7f8bd311e8d3eaa6b1767e87276d3d02dce04a95808ed187082b05bb6211d";
const MUSL_CRT1_BLAKE3: &str = "701c5afce9f5e09d0bcca0702764c64148fcacfe7e1477e2321b249b03af66be";
const MUSL_CRTI_BLAKE3: &str = "ecf4006e5ea51c3ad49a243165909cd4f9eddfbe181b576ac43513ede8578be4";
const MUSL_CRTN_BLAKE3: &str = "b3b3153225cc100b8c429f5916ae36b96ef4903169b86f841f86498c0fd7251d";
const MUSL_HEADERS_BLAKE3: &str = "4d5a63f48ef27c8377a5319d3d68cd1a2b724a974ee5d8281a67f8fb0b5dc1c7";
const MUSL_SMOKE_BINARY_BLAKE3: &str = "92ebe6be234ab289a07f5560ce0f25b1a696917d4515d2ec92c330fa0cd3a8a4";
const EMPTY_ARCHIVE_BLAKE3: &str = "3ba363b1c314e158a3ed3769a2d8c73272b3a01da712c43ecc25af2e4295a3bb";

const REMOVED_SOURCE_DIRECTORIES: [&str; 28] = [
    "src/complex",
    "src/aio",
    "src/legacy",
    "src/linux",
    "src/locale",
    "src/math",
    "src/misc",
    "src/mman",
    "src/mq",
    "src/multibyte",
    "src/network",
    "src/passwd",
    "src/prng",
    "src/process",
    "src/regex",
    "src/sched",
    "src/search",
    "src/select",
    "src/signal",
    "src/stat",
    "src/stdio",
    "src/stdlib",
    "src/temp",
    "src/termios",
    "src/thread",
    "src/time",
    "src/unistd",
    "src/malloc",
];

const COMMON_COMPILE_FLAGS: [&str; 12] = [
    "-static",
    "-nostdinc",
    "-std=c99",
    "-ffreestanding",
    "-D_XOPEN_SOURCE=700",
    "-Iarch/x86_64",
    "-Iarch/generic",
    "-Iobj/src/internal",
    "-Isrc/include",
    "-Isrc/internal",
    "-Iobj/include",
    "-Iinclude",
];

#[derive(Debug, Clone, Copy)]
struct MuslHelperSpec {
    artifact_id: &'static str,
    target: &'static str,
    digest_blake3: &'static str,
}

const MUSL_HELPERS: [MuslHelperSpec; MUSL_HELPER_COUNT] = [
    MuslHelperSpec {
        artifact_id: "musl-minmath-source",
        target: "src/math/minmath.c",
        digest_blake3: "69fc61b70094ca956699dc2fa39f752cab7c65d568bd8fe05d4a936322c3960b",
    },
    MuslHelperSpec {
        artifact_id: "musl-minunistd-source",
        target: "src/unistd/minunistd.c",
        digest_blake3: "05695e5b671545adf3d7b175c06e5abc9d7f3568f1cfdeee904da8b5daca199e",
    },
    MuslHelperSpec {
        artifact_id: "musl-minstdlib-source",
        target: "src/stdlib/minstdlib.c",
        digest_blake3: "c00f0c394966d3976dc551183d23ae074b38071b7e10f36fca82865a0f27c46c",
    },
    MuslHelperSpec {
        artifact_id: "musl-minstdio-source",
        target: "src/stdio/minstdio.c",
        digest_blake3: "c77a6ae68839a92336105833735770df1455bf76466d9e8ef9bbc606f9dbce90",
    },
    MuslHelperSpec {
        artifact_id: "musl-malloc-source",
        target: "src/malloc/malloc.c",
        digest_blake3: "57a8675cd8c7bba2de72af0bbc8a5e31e29c71c3136dc8a5f057d6754765ca84",
    },
    MuslHelperSpec {
        artifact_id: "musl-sysconf-source",
        target: "src/conf/sysconf.c",
        digest_blake3: "4e0b9e10f5d0cb0b2cfdb9a7952b835805901275757978bd0df7491d777e5f2e",
    },
    MuslHelperSpec {
        artifact_id: "musl-mb-cur-max-source",
        target: "src/ctype/__ctype_get_mb_cur_max.c",
        digest_blake3: "5a4e448755715c58f92a4e59913a2013eda231bbb5a7718e954408e629371f30",
    },
    MuslHelperSpec {
        artifact_id: "musl-init-tls-source",
        target: "src/env/__init_tls.c",
        digest_blake3: "a92a92721c1f5045b81dcb38d3c2d2d47bafa822132021c59f5c0ee2e80861cf",
    },
    MuslHelperSpec {
        artifact_id: "musl-reset-tls-source",
        target: "src/env/__reset_tls.c",
        digest_blake3: "0d47c776d69f29aaf091831b88487376786bef2cda19b03de55dfbf1ce7c0abe",
    },
    MuslHelperSpec {
        artifact_id: "musl-stack-fail-source",
        target: "src/env/__stack_chk_fail.c",
        digest_blake3: "dad22f13907c8a5c01177ea03c9b7439d88f82cad27c249bfea98a7cfd0c47cc",
    },
    MuslHelperSpec {
        artifact_id: "musl-start-main-source",
        target: "src/env/__libc_start_main.c",
        digest_blake3: "69c3f1dde3ebc3b778a8e586c9fb83656700a8606d37679b18a89ff8288135ac",
    },
    MuslHelperSpec {
        artifact_id: "musl-errno-location-source",
        target: "src/errno/__errno_location.c",
        digest_blake3: "5362208b59d4de384c2c80c1a9141f163836d9538b8b8218794d25f7a32d7b5a",
    },
    MuslHelperSpec {
        artifact_id: "musl-strerror-source",
        target: "src/errno/strerror.c",
        digest_blake3: "357e6527f4faddb05e93bb0ee6f6cdb70c8c60c9d08ddd5bbcab3ffbe974b9f8",
    },
    MuslHelperSpec {
        artifact_id: "musl-abort-source",
        target: "src/exit/abort.c",
        digest_blake3: "497171f9dc9dcf6b118c3b559d2ae224982710179603e91f1f6ae5390066d018",
    },
    MuslHelperSpec {
        artifact_id: "musl-open-source",
        target: "src/fcntl/open.c",
        digest_blake3: "7f3afdf5148358c90c7f8915de354742da02d7a60d85bbfe22b41c99e2f98bf5",
    },
    MuslHelperSpec {
        artifact_id: "musl-openat-source",
        target: "src/fcntl/openat.c",
        digest_blake3: "8876c1c3b87682ba175d0ca3da3c19aafd773d6780baa63638049ec99c735343",
    },
    MuslHelperSpec {
        artifact_id: "musl-fadvise-source",
        target: "src/fcntl/posix_fadvise.c",
        digest_blake3: "b4ba89e259263927792ffb789b4efc42c72d5e6834a5a0dc1c2f7175469164ee",
    },
    MuslHelperSpec {
        artifact_id: "musl-fcntl-source",
        target: "src/fcntl/fcntl.c",
        digest_blake3: "fd96b7cf48005c85e7756e1aeb548127a07c1162861e4a68d717644521469282",
    },
    MuslHelperSpec {
        artifact_id: "musl-crt1-source",
        target: "crt/crt1.c",
        digest_blake3: "49f71cb10539bd5244198385c8d26d8e70955c5c7d1289a6580c5a6e5e7aa65f",
    },
    MuslHelperSpec {
        artifact_id: "musl-confstr-source",
        target: "src/conf/confstr.c",
        digest_blake3: "b54e82c47c18b998c196fe0e7451edbeb56649e36ebb78719093a6ee8f26f419",
    },
    MuslHelperSpec {
        artifact_id: "musl-crypt-source",
        target: "src/crypt/crypt_sha256.c",
        digest_blake3: "4f36091d58b669daef9b9001f82acac22f13ecd171831058d10c90a31502e672",
    },
    MuslHelperSpec {
        artifact_id: "musl-semctl-source",
        target: "src/ipc/semctl.c",
        digest_blake3: "22289baa86062d7aa482f84bd3b3b1492519b225419e9df6ba46397e86d27e2d",
    },
    MuslHelperSpec {
        artifact_id: "musl-dlerror-source",
        target: "src/ldso/dlerror.c",
        digest_blake3: "9e63330518eb48f3e5b3caf961cc7e05ffef11e3f13768c6e8b22d066d6804c7",
    },
    MuslHelperSpec {
        artifact_id: "musl-fenv-source",
        target: "src/fenv/x86_64/fenv.s",
        digest_blake3: "2e7056cbed4ae4e55a676ebfdad8ccfe39bbb06090e9ca86dd2a445fa3cfb33c",
    },
];

const MUSL_PASS2_HELPERS: [MuslHelperSpec; MUSL_PASS2_HELPER_COUNT] = [
    MuslHelperSpec {
        artifact_id: "musl-pass2-minmath-source",
        target: "src/math/minmath.c",
        digest_blake3: "69fc61b70094ca956699dc2fa39f752cab7c65d568bd8fe05d4a936322c3960b",
    },
    MuslHelperSpec {
        artifact_id: "musl-pass2-minunistd-source",
        target: "src/unistd/minunistd.c",
        digest_blake3: "7a201a50dbb656dccea6b064ae51b5f5fcffcd1bd5d0de2f95b16af50d5b3045",
    },
    MuslHelperSpec {
        artifact_id: "musl-pass2-minstdlib-source",
        target: "src/stdlib/minstdlib.c",
        digest_blake3: "a4b5ad9096f8826607865f54b26136f8c19da0058dbe86dcb78ad1d06ec4f7d1",
    },
    MuslHelperSpec {
        artifact_id: "musl-pass2-minstdio-source",
        target: "src/stdio/minstdio.c",
        digest_blake3: "52c1a3ee69221559e4c38b1efe6c82e3a8f84c87fa9ce8d3835cbac6e732c697",
    },
    MuslHelperSpec {
        artifact_id: "musl-pass2-malloc-source",
        target: "src/malloc/malloc.c",
        digest_blake3: "57a8675cd8c7bba2de72af0bbc8a5e31e29c71c3136dc8a5f057d6754765ca84",
    },
    MuslHelperSpec {
        artifact_id: "musl-pass2-sysconf-source",
        target: "src/conf/sysconf.c",
        digest_blake3: "4e0b9e10f5d0cb0b2cfdb9a7952b835805901275757978bd0df7491d777e5f2e",
    },
    MuslHelperSpec {
        artifact_id: "musl-pass2-mb-cur-max-source",
        target: "src/ctype/__ctype_get_mb_cur_max.c",
        digest_blake3: "5a4e448755715c58f92a4e59913a2013eda231bbb5a7718e954408e629371f30",
    },
    MuslHelperSpec {
        artifact_id: "musl-pass2-init-tls-source",
        target: "src/env/__init_tls.c",
        digest_blake3: "a92a92721c1f5045b81dcb38d3c2d2d47bafa822132021c59f5c0ee2e80861cf",
    },
    MuslHelperSpec {
        artifact_id: "musl-pass2-reset-tls-source",
        target: "src/env/__reset_tls.c",
        digest_blake3: "0d47c776d69f29aaf091831b88487376786bef2cda19b03de55dfbf1ce7c0abe",
    },
    MuslHelperSpec {
        artifact_id: "musl-pass2-stack-fail-source",
        target: "src/env/__stack_chk_fail.c",
        digest_blake3: "dad22f13907c8a5c01177ea03c9b7439d88f82cad27c249bfea98a7cfd0c47cc",
    },
    MuslHelperSpec {
        artifact_id: "musl-pass2-start-main-source",
        target: "src/env/__libc_start_main.c",
        digest_blake3: "69c3f1dde3ebc3b778a8e586c9fb83656700a8606d37679b18a89ff8288135ac",
    },
    MuslHelperSpec {
        artifact_id: "musl-pass2-errno-location-source",
        target: "src/errno/__errno_location.c",
        digest_blake3: "5362208b59d4de384c2c80c1a9141f163836d9538b8b8218794d25f7a32d7b5a",
    },
    MuslHelperSpec {
        artifact_id: "musl-pass2-strerror-source",
        target: "src/errno/strerror.c",
        digest_blake3: "357e6527f4faddb05e93bb0ee6f6cdb70c8c60c9d08ddd5bbcab3ffbe974b9f8",
    },
    MuslHelperSpec {
        artifact_id: "musl-pass2-abort-source",
        target: "src/exit/abort.c",
        digest_blake3: "497171f9dc9dcf6b118c3b559d2ae224982710179603e91f1f6ae5390066d018",
    },
    MuslHelperSpec {
        artifact_id: "musl-pass2-open-source",
        target: "src/fcntl/open.c",
        digest_blake3: "7f3afdf5148358c90c7f8915de354742da02d7a60d85bbfe22b41c99e2f98bf5",
    },
    MuslHelperSpec {
        artifact_id: "musl-pass2-openat-source",
        target: "src/fcntl/openat.c",
        digest_blake3: "8876c1c3b87682ba175d0ca3da3c19aafd773d6780baa63638049ec99c735343",
    },
    MuslHelperSpec {
        artifact_id: "musl-pass2-fadvise-source",
        target: "src/fcntl/posix_fadvise.c",
        digest_blake3: "b4ba89e259263927792ffb789b4efc42c72d5e6834a5a0dc1c2f7175469164ee",
    },
    MuslHelperSpec {
        artifact_id: "musl-pass2-fcntl-source",
        target: "src/fcntl/fcntl.c",
        digest_blake3: "fd96b7cf48005c85e7756e1aeb548127a07c1162861e4a68d717644521469282",
    },
    MuslHelperSpec {
        artifact_id: "musl-pass2-minstat-source",
        target: "src/stat/minstat.c",
        digest_blake3: "e4d5e5096162c3979357fab4392cdff388a5df93bbf048dbd72b86d1cca8318b",
    },
    MuslHelperSpec {
        artifact_id: "musl-pass2-remove-source",
        target: "src/stdio/remove.c",
        digest_blake3: "dc2a168ba09d83414283a3aa337dddd71a13d7f88b5cc06186f762c85460c495",
    },
    MuslHelperSpec {
        artifact_id: "musl-pass2-utime-source",
        target: "src/time/utime.c",
        digest_blake3: "670df56105afbae06807aea52df008850ac343be79850c6c6a6a8fc1b5ac0f82",
    },
    MuslHelperSpec {
        artifact_id: "musl-pass2-signal-source",
        target: "src/signal/signal.c",
        digest_blake3: "d157fbf502c72be6a283483115ddb04e18ce295ce731bc98112147759f4608d0",
    },
    MuslHelperSpec {
        artifact_id: "musl-pass2-crt1-source",
        target: "crt/crt1.c",
        digest_blake3: "df822d5849c76562ea29882c4b5ed6cf4dceca203eb65d52442f0c0871ca29fc",
    },
    MuslHelperSpec {
        artifact_id: "musl-pass2-confstr-source",
        target: "src/conf/confstr.c",
        digest_blake3: "b54e82c47c18b998c196fe0e7451edbeb56649e36ebb78719093a6ee8f26f419",
    },
    MuslHelperSpec {
        artifact_id: "musl-pass2-crypt-source",
        target: "src/crypt/crypt_sha256.c",
        digest_blake3: "4f36091d58b669daef9b9001f82acac22f13ecd171831058d10c90a31502e672",
    },
    MuslHelperSpec {
        artifact_id: "musl-pass2-semctl-source",
        target: "src/ipc/semctl.c",
        digest_blake3: "22289baa86062d7aa482f84bd3b3b1492519b225419e9df6ba46397e86d27e2d",
    },
    MuslHelperSpec {
        artifact_id: "musl-pass2-dlerror-source",
        target: "src/ldso/dlerror.c",
        digest_blake3: "9e63330518eb48f3e5b3caf961cc7e05ffef11e3f13768c6e8b22d066d6804c7",
    },
    MuslHelperSpec {
        artifact_id: "musl-pass2-fenv-source",
        target: "src/fenv/x86_64/fenv.s",
        digest_blake3: "2e7056cbed4ae4e55a676ebfdad8ccfe39bbb06090e9ca86dd2a445fa3cfb33c",
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MuslExpectedOutput {
    pub artifact_id: &'static str,
    pub digest_blake3: &'static str,
}

pub(crate) const MUSL_EXPECTED_OUTPUTS: [MuslExpectedOutput; MUSL_OUTPUT_COUNT] = [
    MuslExpectedOutput {
        artifact_id: "musl-libc",
        digest_blake3: MUSL_LIBC_BLAKE3,
    },
    MuslExpectedOutput {
        artifact_id: "musl-crt1",
        digest_blake3: MUSL_CRT1_BLAKE3,
    },
    MuslExpectedOutput {
        artifact_id: "musl-crti",
        digest_blake3: MUSL_CRTI_BLAKE3,
    },
    MuslExpectedOutput {
        artifact_id: "musl-crtn",
        digest_blake3: MUSL_CRTN_BLAKE3,
    },
    MuslExpectedOutput {
        artifact_id: "musl-libm",
        digest_blake3: EMPTY_ARCHIVE_BLAKE3,
    },
    MuslExpectedOutput {
        artifact_id: "musl-librt",
        digest_blake3: EMPTY_ARCHIVE_BLAKE3,
    },
    MuslExpectedOutput {
        artifact_id: "musl-libpthread",
        digest_blake3: EMPTY_ARCHIVE_BLAKE3,
    },
    MuslExpectedOutput {
        artifact_id: "musl-libcrypt",
        digest_blake3: EMPTY_ARCHIVE_BLAKE3,
    },
    MuslExpectedOutput {
        artifact_id: "musl-libutil",
        digest_blake3: EMPTY_ARCHIVE_BLAKE3,
    },
    MuslExpectedOutput {
        artifact_id: "musl-libxnet",
        digest_blake3: EMPTY_ARCHIVE_BLAKE3,
    },
    MuslExpectedOutput {
        artifact_id: "musl-libresolv",
        digest_blake3: EMPTY_ARCHIVE_BLAKE3,
    },
    MuslExpectedOutput {
        artifact_id: "musl-libdl",
        digest_blake3: EMPTY_ARCHIVE_BLAKE3,
    },
    MuslExpectedOutput {
        artifact_id: "musl-headers",
        digest_blake3: MUSL_HEADERS_BLAKE3,
    },
    MuslExpectedOutput {
        artifact_id: "musl-smoke-binary",
        digest_blake3: MUSL_SMOKE_BINARY_BLAKE3,
    },
];

pub(crate) const MUSL_PASS2_EXPECTED_OUTPUTS: [MuslExpectedOutput; MUSL_OUTPUT_COUNT] = [
    MuslExpectedOutput {
        artifact_id: "musl-pass2-libc",
        digest_blake3: MUSL_PASS2_LIBC_BLAKE3,
    },
    MuslExpectedOutput {
        artifact_id: "musl-pass2-crt1",
        digest_blake3: MUSL_PASS2_CRT1_BLAKE3,
    },
    MuslExpectedOutput {
        artifact_id: "musl-pass2-crti",
        digest_blake3: MUSL_PASS2_CRTI_BLAKE3,
    },
    MuslExpectedOutput {
        artifact_id: "musl-pass2-crtn",
        digest_blake3: MUSL_PASS2_CRTN_BLAKE3,
    },
    MuslExpectedOutput {
        artifact_id: "musl-pass2-libm",
        digest_blake3: EMPTY_ARCHIVE_BLAKE3,
    },
    MuslExpectedOutput {
        artifact_id: "musl-pass2-librt",
        digest_blake3: EMPTY_ARCHIVE_BLAKE3,
    },
    MuslExpectedOutput {
        artifact_id: "musl-pass2-libpthread",
        digest_blake3: EMPTY_ARCHIVE_BLAKE3,
    },
    MuslExpectedOutput {
        artifact_id: "musl-pass2-libcrypt",
        digest_blake3: EMPTY_ARCHIVE_BLAKE3,
    },
    MuslExpectedOutput {
        artifact_id: "musl-pass2-libutil",
        digest_blake3: EMPTY_ARCHIVE_BLAKE3,
    },
    MuslExpectedOutput {
        artifact_id: "musl-pass2-libxnet",
        digest_blake3: EMPTY_ARCHIVE_BLAKE3,
    },
    MuslExpectedOutput {
        artifact_id: "musl-pass2-libresolv",
        digest_blake3: EMPTY_ARCHIVE_BLAKE3,
    },
    MuslExpectedOutput {
        artifact_id: "musl-pass2-libdl",
        digest_blake3: EMPTY_ARCHIVE_BLAKE3,
    },
    MuslExpectedOutput {
        artifact_id: "musl-pass2-headers",
        digest_blake3: MUSL_PASS2_HEADERS_BLAKE3,
    },
    MuslExpectedOutput {
        artifact_id: "musl-pass2-smoke-binary",
        digest_blake3: MUSL_PASS2_SMOKE_BINARY_BLAKE3,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MuslProfile {
    First,
    Second,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct MuslSourceMaterializationReport {
    pub format: &'static str,
    pub source_bundle_manifest_blake3: String,
    pub artifact_id: &'static str,
    pub record_name: &'static str,
    pub record_identity: String,
    pub record_content_blake3: String,
    pub output_path: PathBuf,
    pub non_claim: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct MuslOutputReport {
    pub artifact_id: String,
    pub path: PathBuf,
    pub bytes_len: u64,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct MuslInventoryReport {
    pub format: &'static str,
    pub configured_source_digest_blake3: String,
    pub helper_count: u32,
    pub source_compile_count: u32,
    pub crt_compile_count: u32,
    pub archive_command_count: u32,
    pub smoke_command_count: u32,
    pub outputs: Vec<MuslOutputReport>,
    pub protected_exec_enforced: bool,
    pub fallback_events: Vec<String>,
    pub non_claim: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct MuslInventoryRequest<'a> {
    pub source_root: &'a Path,
    pub compiler_root: &'a Path,
    pub profile: MuslProfile,
    pub scratch_dir: &'a Path,
    pub protected_exec_enforced: bool,
}

impl MuslProfile {
    fn recipe(self) -> &'static [u8] {
        match self {
            Self::First => MUSL_RECIPE_SOURCE,
            Self::Second => MUSL_PASS2_RECIPE_SOURCE,
        }
    }

    fn recipe_artifact(self) -> (&'static str, &'static str) {
        match self {
            Self::First => (MUSL_RECIPE_SOURCE_ARTIFACT_ID, MUSL_RECIPE_SOURCE_BLAKE3),
            Self::Second => (MUSL_PASS2_RECIPE_SOURCE_ARTIFACT_ID, MUSL_PASS2_RECIPE_SOURCE_BLAKE3),
        }
    }

    fn helpers(self) -> &'static [MuslHelperSpec] {
        match self {
            Self::First => &MUSL_HELPERS,
            Self::Second => &MUSL_PASS2_HELPERS,
        }
    }

    fn compiler_relative(self) -> &'static str {
        match self {
            Self::First => "bin/tcc-musl-prep",
            Self::Second => "bin/tcc-0.9.27-musl",
        }
    }

    fn compiler_digest(self) -> &'static str {
        match self {
            Self::First => crate::stagex_tcc_musl_prep::TCC_MUSL_PREP_FINAL_BLAKE3,
            Self::Second => crate::stagex_tcc_musl::TCC_MUSL_FINAL_BLAKE3,
        }
    }

    fn runtime_archive_relative(self) -> &'static str {
        match self {
            Self::First => "lib/mes/tcc/libtcc1.a",
            Self::Second => "lib/tcc/libtcc1.a",
        }
    }

    fn configured_source_digest(self) -> &'static str {
        match self {
            Self::First => MUSL_CONFIGURED_SOURCE_BLAKE3,
            Self::Second => MUSL_PASS2_CONFIGURED_SOURCE_BLAKE3,
        }
    }

    fn output_prefix(self) -> &'static str {
        match self {
            Self::First => "musl",
            Self::Second => "musl-pass2",
        }
    }

    fn expected_outputs(self) -> &'static [MuslExpectedOutput] {
        match self {
            Self::First => &MUSL_EXPECTED_OUTPUTS,
            Self::Second => &MUSL_PASS2_EXPECTED_OUTPUTS,
        }
    }

    fn report_format(self) -> &'static str {
        match self {
            Self::First => MUSL_REPORT_FORMAT,
            Self::Second => MUSL_PASS2_REPORT_FORMAT,
        }
    }

    fn non_claim(self) -> &'static str {
        match self {
            Self::First => MUSL_NON_CLAIM,
            Self::Second => MUSL_PASS2_NON_CLAIM,
        }
    }
}

#[derive(Debug)]
pub(crate) enum StagexMuslError {
    SourceRecordNotFound,
    Materialization(String),
    Runtime(crate::stagex_mes_lib::MesLibraryPlanError),
}

impl std::fmt::Display for StagexMuslError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SourceRecordNotFound => write!(formatter, "musl source record was not found"),
            Self::Materialization(message) => write!(formatter, "musl materialization failed: {message}"),
            Self::Runtime(error) => write!(formatter, "musl runtime failed: {error}"),
        }
    }
}

impl std::error::Error for StagexMuslError {}

impl From<crate::stagex_mes_lib::MesLibraryPlanError> for StagexMuslError {
    fn from(error: crate::stagex_mes_lib::MesLibraryPlanError) -> Self {
        Self::Runtime(error)
    }
}

impl From<crate::stagex_tinycc::TinyccError> for StagexMuslError {
    fn from(error: crate::stagex_tinycc::TinyccError) -> Self {
        Self::Materialization(error.to_string())
    }
}

pub(crate) fn source_artifact_digests() -> Vec<(&'static str, &'static str)> {
    profile_source_artifact_digests(MuslProfile::First, true)
}

pub(crate) fn pass2_source_artifact_digests() -> Vec<(&'static str, &'static str)> {
    profile_source_artifact_digests(MuslProfile::Second, false)
}

fn profile_source_artifact_digests(
    profile: MuslProfile,
    include_shared_source: bool,
) -> Vec<(&'static str, &'static str)> {
    let helpers = profile.helpers();
    let mut artifacts = Vec::with_capacity(usize::from(include_shared_source) + 1 + helpers.len());
    if include_shared_source {
        artifacts.push((MUSL_SOURCE_ARTIFACT_ID, MUSL_SOURCE_CONTENT_BLAKE3));
    }
    artifacts.push(profile.recipe_artifact());
    artifacts.extend(helpers.iter().map(|helper| (helper.artifact_id, helper.digest_blake3)));
    assert_eq!(artifacts.len(), usize::from(include_shared_source) + 1 + helpers.len());
    assert!(artifacts.iter().all(|(_, digest)| !digest.is_empty()));
    artifacts
}

pub(crate) fn materialize_authenticated_musl_source(
    bundle_path: &Path,
    expected_manifest_blake3: &str,
    scratch_dir: &Path,
) -> Result<MuslSourceMaterializationReport, StagexMuslError> {
    if !bundle_path.is_absolute() || !bundle_path.is_file() {
        return Err(StagexMuslError::Materialization(format!(
            "source bundle is not an absolute file: {}",
            bundle_path.display()
        )));
    }
    validate_bound_recipe()?;
    let manifest = read_source_bundle(bundle_path)
        .map_err(|error| StagexMuslError::Materialization(format!("reading source bundle: {error}")))?;
    if manifest.manifest_blake3 != expected_manifest_blake3 {
        return Err(StagexMuslError::Materialization(format!(
            "source-bundle manifest mismatch: expected {expected_manifest_blake3}, observed {}",
            manifest.manifest_blake3
        )));
    }
    let record = find_source_record(&manifest.records)?;
    validate_source_record(record)?;
    fs::create_dir(scratch_dir)
        .map_err(|error| StagexMuslError::Materialization(format!("creating musl source scratch: {error}")))?;
    let output_path = scratch_dir.join(MUSL_SOURCE_OUTPUT_NAME);
    materialize_source_record_for_offline_use(record, &output_path).map_err(|error| {
        StagexMuslError::Materialization(format!("materializing musl source {}: {error}", record.identity))
    })?;
    let output_path = resolve_materialized_source_root(&output_path)?;
    if !output_path.join("configure").is_file() || !output_path.join("src/internal/libc.h").is_file() {
        return Err(StagexMuslError::Materialization("materialized musl tree lacks required source files".to_string()));
    }
    assert!(output_path.is_absolute());
    assert_eq!(record.content_blake3, MUSL_SOURCE_CONTENT_BLAKE3);
    Ok(MuslSourceMaterializationReport {
        format: MUSL_SOURCE_REPORT_FORMAT,
        source_bundle_manifest_blake3: manifest.manifest_blake3,
        artifact_id: MUSL_SOURCE_ARTIFACT_ID,
        record_name: MUSL_RECORD_NAME,
        record_identity: record.identity.clone(),
        record_content_blake3: record.content_blake3.clone(),
        output_path,
        non_claim: MUSL_SOURCE_NON_CLAIM,
    })
}

pub(crate) fn derive_musl_inventory(request: MuslInventoryRequest<'_>) -> Result<MuslInventoryReport, StagexMuslError> {
    validate_inventory_inputs(&request)?;
    fs::create_dir(request.scratch_dir)
        .map_err(|error| StagexMuslError::Materialization(format!("creating musl scratch: {error}")))?;
    let source_root = request.scratch_dir.join(MUSL_SOURCE_OUTPUT_NAME);
    crate::stagex_mes_lib::copy_tree_bounded(request.source_root, &source_root)?;
    crate::stagex_tinycc::make_tree_owner_writable(&source_root)?;
    configure_source_tree(&source_root, request.profile)?;
    generate_headers(&source_root)?;
    let libc_sources = selected_sources(&source_root.join("src"))?;
    let crt_sources = selected_sources(&source_root.join("crt"))?;
    let configured_source_digest_blake3 =
        configured_source_digest(&source_root, &libc_sources, &crt_sources, request.profile)?;
    let output_root = request.scratch_dir.join("output");
    fs::create_dir_all(output_root.join("lib"))
        .map_err(|error| StagexMuslError::Materialization(format!("creating musl output: {error}")))?;
    install_headers(&source_root, &output_root.join("include"))?;
    let compiler = request.compiler_root.join(request.profile.compiler_relative());
    let objects = compile_sources(&request, &compiler, &source_root, &libc_sources, "libc")?;
    let crt_objects = compile_sources(&request, &compiler, &source_root, &crt_sources, "crt")?;
    let libc = archive_libc(&request, &compiler, &source_root, &output_root, &objects)?;
    install_crt_objects(&output_root, &crt_sources, &crt_objects)?;
    materialize_empty_archives(&output_root)?;
    let smoke_binary = run_smokes(&request, &compiler, &source_root, &output_root)?;
    let outputs = collect_outputs(&output_root, &libc, &smoke_binary, request.profile)?;
    validate_expected_outputs(&outputs, request.profile)?;
    let report = MuslInventoryReport {
        format: request.profile.report_format(),
        configured_source_digest_blake3,
        helper_count: u32::try_from(request.profile.helpers().len()).unwrap(),
        source_compile_count: u32::try_from(libc_sources.len())
            .map_err(|_| StagexMuslError::Materialization("musl source count overflow".to_string()))?,
        crt_compile_count: u32::try_from(crt_sources.len())
            .map_err(|_| StagexMuslError::Materialization("musl CRT count overflow".to_string()))?,
        archive_command_count: 1,
        smoke_command_count: MUSL_SMOKE_COMMAND_COUNT,
        outputs,
        protected_exec_enforced: request.protected_exec_enforced,
        fallback_events: Vec::new(),
        non_claim: request.profile.non_claim(),
    };
    crate::stagex_mes_lib::write_create_new(
        &request.scratch_dir.join("musl-inventory.json"),
        &serde_json::to_vec_pretty(&report)
            .map_err(|error| StagexMuslError::Materialization(format!("serializing musl report: {error}")))?,
    )?;
    assert_eq!(report.outputs.len(), MUSL_OUTPUT_COUNT);
    assert!(report.fallback_events.is_empty());
    Ok(report)
}

fn validate_inventory_inputs(request: &MuslInventoryRequest<'_>) -> Result<(), StagexMuslError> {
    if request.scratch_dir.exists() {
        return Err(StagexMuslError::Materialization(format!(
            "create-new musl scratch exists: {}",
            request.scratch_dir.display()
        )));
    }
    for (label, root) in [
        ("musl source", request.source_root),
        ("musl compiler", request.compiler_root),
    ] {
        if !root.is_absolute() || !root.is_dir() {
            return Err(StagexMuslError::Materialization(format!(
                "{label} is not an absolute directory: {}",
                root.display()
            )));
        }
    }
    validate_file_digest(
        &request.compiler_root.join(request.profile.compiler_relative()),
        request.profile.compiler_digest(),
        "musl compiler",
    )?;
    validate_bound_recipe_for_profile(request.profile)?;
    validate_helper_digests(request.profile)?;
    assert!(request.source_root.join("Makefile").is_file());
    assert!(request.compiler_root.join(request.profile.runtime_archive_relative()).is_file());
    Ok(())
}

fn configure_source_tree(root: &Path, profile: MuslProfile) -> Result<(), StagexMuslError> {
    for relative in REMOVED_SOURCE_DIRECTORIES {
        let path = root.join(relative);
        if path.exists() {
            fs::remove_dir_all(&path).map_err(|error| {
                StagexMuslError::Materialization(format!(
                    "removing declared musl source directory {}: {error}",
                    path.display()
                ))
            })?;
        }
    }
    for relative in ["crt/Scrt1.c", "crt/rcrt1.c"] {
        let path = root.join(relative);
        if path.exists() {
            fs::remove_file(&path).map_err(|error| {
                StagexMuslError::Materialization(format!(
                    "removing declared musl source file {}: {error}",
                    path.display()
                ))
            })?;
        }
    }
    let alltypes = root.join("arch/x86_64/bits/alltypes.h.in");
    crate::stagex_tinycc::replace_required_text(
        &alltypes,
        "TYPEDEF __builtin_va_list va_list;",
        "#if defined(__TINYC__)\ntypedef char *__builtin_va_list;\n#endif\nTYPEDEF __builtin_va_list va_list;",
    )?;
    let syscall = root.join("src/internal/syscall.h");
    if syscall.is_file() {
        let text = fs::read_to_string(&syscall)
            .map_err(|error| StagexMuslError::Materialization(format!("reading musl syscall.h: {error}")))?;
        let marker = "fixup legacy";
        let marker_index = text
            .find(marker)
            .ok_or_else(|| StagexMuslError::Materialization("musl syscall.h lacks legacy fixup marker".to_string()))?;
        let line_start = text[..marker_index].rfind('\n').map_or(0, |index| index + 1);
        let mut kept = text[..line_start].to_string();
        kept.push_str("#endif\n");
        fs::write(&syscall, kept)
            .map_err(|error| StagexMuslError::Materialization(format!("writing bounded musl syscall.h: {error}")))?;
    }
    materialize_helpers(root, profile)?;
    assert!(!root.join("src/network").exists());
    assert!(root.join("src/math/minmath.c").is_file());
    Ok(())
}

fn materialize_helpers(root: &Path, profile: MuslProfile) -> Result<(), StagexMuslError> {
    for helper in profile.helpers() {
        let bytes = extract_helper(profile.recipe(), helper.target)?;
        let target = root.join(helper.target);
        if target.exists() {
            if target.is_dir() {
                return Err(StagexMuslError::Materialization(format!(
                    "musl helper target is a directory: {}",
                    target.display()
                )));
            }
            fs::remove_file(&target).map_err(|error| {
                StagexMuslError::Materialization(format!("removing musl helper target {}: {error}", target.display()))
            })?;
        }
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                StagexMuslError::Materialization(format!("creating musl helper parent {}: {error}", parent.display()))
            })?;
        }
        crate::stagex_mes_lib::write_create_new(&target, &bytes)?;
    }
    assert!(root.join("crt/crt1.c").is_file());
    assert!(root.join("src/fenv/x86_64/fenv.s").is_file());
    Ok(())
}

fn extract_helper(source: &[u8], target: &str) -> Result<Vec<u8>, StagexMuslError> {
    let text = std::str::from_utf8(source)
        .map_err(|error| StagexMuslError::Materialization(format!("musl recipe is not UTF-8: {error}")))?;
    let marker = format!("cat > {target} <<'EOF'\n");
    let starts = text.match_indices(&marker).map(|(index, _)| index).collect::<Vec<_>>();
    if starts.len() != 1 {
        return Err(StagexMuslError::Materialization(format!(
            "musl helper target {target} occurs {} times",
            starts.len()
        )));
    }
    let start = starts[0]
        .checked_add(marker.len())
        .ok_or_else(|| StagexMuslError::Materialization("musl helper start overflow".to_string()))?;
    let relative_end = text[start..]
        .find("\nEOF\n")
        .ok_or_else(|| StagexMuslError::Materialization(format!("musl helper target {target} lacks terminator")))?;
    let end = start
        .checked_add(relative_end)
        .and_then(|value| value.checked_add(1))
        .ok_or_else(|| StagexMuslError::Materialization("musl helper end overflow".to_string()))?;
    let bytes = source[start..end].to_vec();
    if bytes.is_empty() || !bytes.ends_with(b"\n") {
        return Err(StagexMuslError::Materialization(format!("musl helper target {target} is empty or unterminated")));
    }
    assert!(end <= source.len());
    assert!(bytes.ends_with(b"\n"));
    Ok(bytes)
}

fn validate_helper_digests(profile: MuslProfile) -> Result<(), StagexMuslError> {
    let helpers = profile.helpers();
    let mut mismatches = Vec::new();
    for helper in helpers {
        let observed = blake3::hash(&extract_helper(profile.recipe(), helper.target)?).to_hex().to_string();
        if observed != helper.digest_blake3 {
            mismatches.push(format!("{}={observed}", helper.artifact_id));
        }
    }
    if !mismatches.is_empty() {
        return Err(StagexMuslError::Materialization(format!(
            "musl helper BLAKE3 mismatches: {}",
            mismatches.join(",")
        )));
    }
    assert!(!helpers.is_empty());
    assert!(helpers.iter().all(|helper| helper.digest_blake3.len() == blake3::OUT_LEN * 2));
    Ok(())
}

fn generate_headers(root: &Path) -> Result<(), StagexMuslError> {
    let object_bits = root.join("obj/include/bits");
    let object_internal = root.join("obj/src/internal");
    fs::create_dir_all(&object_bits)
        .map_err(|error| StagexMuslError::Materialization(format!("creating musl generated bits: {error}")))?;
    fs::create_dir_all(&object_internal)
        .map_err(|error| StagexMuslError::Materialization(format!("creating musl generated internal: {error}")))?;
    let arch_alltypes = fs::read_to_string(root.join("arch/x86_64/bits/alltypes.h.in"))
        .map_err(|error| StagexMuslError::Materialization(format!("reading arch alltypes: {error}")))?;
    let generic_alltypes = fs::read_to_string(root.join("include/alltypes.h.in"))
        .map_err(|error| StagexMuslError::Materialization(format!("reading generic alltypes: {error}")))?;
    let alltypes = transform_alltypes(&format!("{arch_alltypes}{generic_alltypes}"))?;
    crate::stagex_mes_lib::write_create_new(&object_bits.join("alltypes.h"), alltypes.as_bytes())?;
    let syscall_input = fs::read_to_string(root.join("arch/x86_64/bits/syscall.h.in"))
        .map_err(|error| StagexMuslError::Materialization(format!("reading musl syscall input: {error}")))?;
    let mut syscall = syscall_input.clone();
    for line in syscall_input.lines() {
        if line.contains("__NR_") {
            syscall.push_str(&line.replacen("__NR_", "SYS_", 1));
            syscall.push('\n');
        }
    }
    crate::stagex_mes_lib::write_create_new(&object_bits.join("syscall.h"), syscall.as_bytes())?;
    crate::stagex_mes_lib::write_create_new(&object_internal.join("version.h"), b"#define VERSION \"1.1.24\"\n")?;
    assert!(object_bits.join("alltypes.h").is_file());
    assert!(object_bits.join("syscall.h").is_file());
    Ok(())
}

fn transform_alltypes(input: &str) -> Result<String, StagexMuslError> {
    let mut output = String::new();
    for line in input.lines() {
        if let Some(rest) = line.strip_prefix("TYPEDEF ") {
            let (definition, name_with_semicolon) = rest
                .rsplit_once(' ')
                .ok_or_else(|| StagexMuslError::Materialization(format!("malformed musl TYPEDEF line: {line}")))?;
            let name = name_with_semicolon
                .strip_suffix(';')
                .ok_or_else(|| StagexMuslError::Materialization(format!("unterminated musl TYPEDEF line: {line}")))?;
            output.push_str(&format!(
                "#if defined(__NEED_{name}) && !defined(__DEFINED_{name})\ntypedef {definition} {name};\n#define __DEFINED_{name}\n#endif\n\n"
            ));
        } else if let Some(rest) = line.strip_prefix("STRUCT ") {
            let (name, definition) = rest
                .split_once(' ')
                .ok_or_else(|| StagexMuslError::Materialization(format!("malformed musl STRUCT line: {line}")))?;
            let definition = definition
                .strip_suffix(';')
                .ok_or_else(|| StagexMuslError::Materialization(format!("unterminated musl STRUCT line: {line}")))?;
            output.push_str(&format!(
                "#if defined(__NEED_struct_{name}) && !defined(__DEFINED_struct_{name})\nstruct {name} {definition};\n#define __DEFINED_struct_{name}\n#endif\n\n"
            ));
        } else if let Some(rest) = line.strip_prefix("UNION ") {
            let (name, definition) = rest
                .split_once(' ')
                .ok_or_else(|| StagexMuslError::Materialization(format!("malformed musl UNION line: {line}")))?;
            let definition = definition
                .strip_suffix(';')
                .ok_or_else(|| StagexMuslError::Materialization(format!("unterminated musl UNION line: {line}")))?;
            output.push_str(&format!(
                "#if defined(__NEED_union_{name}) && !defined(__DEFINED_union_{name})\nunion {name} {definition};\n#define __DEFINED_union_{name}\n#endif\n\n"
            ));
        } else {
            output.push_str(line);
            output.push('\n');
        }
    }
    if !output.contains("__DEFINED_size_t") || !output.contains("__DEFINED_struct_timespec") {
        return Err(StagexMuslError::Materialization("generated musl alltypes lacks required definitions".to_string()));
    }
    assert!(output.len() >= input.len());
    assert!(output.ends_with('\n'));
    Ok(output)
}

fn selected_sources(root: &Path) -> Result<Vec<String>, StagexMuslError> {
    let source_prefix = root
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| StagexMuslError::Materialization("musl source root has no UTF-8 name".to_string()))?;
    let mut base = BTreeMap::<String, String>::new();
    let mut arch = BTreeMap::<String, String>::new();
    for path in bounded_tree_paths(root)? {
        let metadata = fs::symlink_metadata(&path).map_err(|error| {
            StagexMuslError::Materialization(format!("reading musl source metadata {}: {error}", path.display()))
        })?;
        if !metadata.file_type().is_file() || !is_compile_source(&path) {
            continue;
        }
        let relative = path
            .strip_prefix(root)
            .map_err(|error| StagexMuslError::Materialization(format!("finding musl source relative path: {error}")))?;
        let relative = relative
            .to_str()
            .ok_or_else(|| StagexMuslError::Materialization("musl source path is not UTF-8".to_string()))?;
        if let Some((directory, leaf)) = relative.split_once("/x86_64/") {
            arch.insert(format!("{directory}/{leaf}"), format!("{source_prefix}/{relative}"));
        } else {
            let slash_count = relative.matches('/').count();
            let is_base_source = if source_prefix == "src" {
                slash_count == 1
            } else {
                slash_count == 0
            };
            if is_base_source {
                base.insert(relative.to_string(), format!("{source_prefix}/{relative}"));
            }
        }
    }
    for replaced in arch.keys() {
        base.remove(replaced);
    }
    let mut selected = base.into_values().chain(arch.into_values()).collect::<Vec<_>>();
    selected.sort();
    selected.dedup();
    if selected.is_empty() || selected.len() > MUSL_TREE_ENTRY_COUNT_MAX {
        return Err(StagexMuslError::Materialization(format!(
            "musl selected source count is outside bounds: {}",
            selected.len()
        )));
    }
    assert!(!selected.is_empty());
    assert!(selected.windows(2).all(|pair| pair[0] < pair[1]));
    Ok(selected)
}

fn is_compile_source(path: &Path) -> bool {
    matches!(path.extension().and_then(|value| value.to_str()), Some("c" | "s" | "S"))
}

fn compile_sources(
    request: &MuslInventoryRequest<'_>,
    compiler: &Path,
    source_root: &Path,
    sources: &[String],
    group: &str,
) -> Result<Vec<PathBuf>, StagexMuslError> {
    let object_root = source_root.join(format!("objects-{group}"));
    fs::create_dir(&object_root)
        .map_err(|error| StagexMuslError::Materialization(format!("creating musl object root: {error}")))?;
    let mut objects = Vec::with_capacity(sources.len());
    for (index, relative) in sources.iter().enumerate() {
        let object = object_root.join(format!("{group}-{index:04}.o"));
        let mut args = COMMON_COMPILE_FLAGS.iter().map(|value| (*value).to_string()).collect::<Vec<_>>();
        args.extend([
            "-c".to_string(),
            "-o".to_string(),
            absolute_utf8(&object, "musl object")?,
            relative.to_string(),
        ]);
        crate::stagex_mes_lib::run_bounded_process(
            compiler,
            &args,
            source_root,
            &BTreeMap::<String, String>::new(),
            &request.scratch_dir.join(format!("musl-{group}-{index:04}.stderr.txt")),
        )?;
        validate_nonempty_file(&object, "musl object")?;
        objects.push(object);
    }
    assert_eq!(objects.len(), sources.len());
    assert!(objects.iter().all(|path| path.is_file()));
    Ok(objects)
}

fn archive_libc(
    request: &MuslInventoryRequest<'_>,
    compiler: &Path,
    source_root: &Path,
    output_root: &Path,
    objects: &[PathBuf],
) -> Result<PathBuf, StagexMuslError> {
    let libc = output_root.join("lib/libc.a");
    let mut args = vec!["-ar".to_string(), "rc".to_string(), absolute_utf8(&libc, "musl libc")?];
    for object in objects {
        args.push(absolute_utf8(object, "musl libc object")?);
    }
    crate::stagex_mes_lib::run_bounded_process(
        compiler,
        &args,
        source_root,
        &BTreeMap::<String, String>::new(),
        &request.scratch_dir.join("musl-archive.stderr.txt"),
    )?;
    validate_nonempty_file(&libc, "musl libc archive")?;
    assert!(!objects.is_empty());
    assert!(libc.is_file());
    Ok(libc)
}

fn install_crt_objects(output_root: &Path, sources: &[String], objects: &[PathBuf]) -> Result<(), StagexMuslError> {
    if sources.len() != objects.len() {
        return Err(StagexMuslError::Materialization("musl CRT source/object count mismatch".to_string()));
    }
    let mut installed = BTreeSet::new();
    for (source, object) in sources.iter().zip(objects) {
        let name = Path::new(source)
            .file_stem()
            .and_then(|value| value.to_str())
            .ok_or_else(|| StagexMuslError::Materialization(format!("invalid musl CRT source name: {source}")))?;
        let target = output_root.join(format!("lib/{name}.o"));
        if !installed.insert(name.to_string()) {
            return Err(StagexMuslError::Materialization(format!("duplicate musl CRT output name: {name}")));
        }
        crate::stagex_mes_lib::copy_file_exact(object, &target)?;
    }
    for required in ["crt1", "crti", "crtn"] {
        if !installed.contains(required) {
            return Err(StagexMuslError::Materialization(format!("musl CRT set lacks {required}.o")));
        }
    }
    assert_eq!(installed.len(), objects.len());
    assert!(output_root.join("lib/crt1.o").is_file());
    Ok(())
}

fn materialize_empty_archives(output_root: &Path) -> Result<(), StagexMuslError> {
    for name in ["m", "rt", "pthread", "crypt", "util", "xnet", "resolv", "dl"] {
        crate::stagex_mes_lib::write_create_new(&output_root.join(format!("lib/lib{name}.a")), EMPTY_ARCHIVE_BYTES)?;
    }
    assert_eq!(blake3::hash(EMPTY_ARCHIVE_BYTES).to_hex().to_string(), EMPTY_ARCHIVE_BLAKE3);
    assert!(output_root.join("lib/libm.a").is_file());
    Ok(())
}

fn install_headers(source_root: &Path, output_include: &Path) -> Result<(), StagexMuslError> {
    crate::stagex_mes_lib::copy_tree_bounded(&source_root.join("include"), output_include)?;
    for source in [
        source_root.join("arch/generic/bits"),
        source_root.join("arch/x86_64/bits"),
        source_root.join("obj/include/bits"),
    ] {
        for path in bounded_tree_paths(&source)? {
            let metadata = fs::symlink_metadata(&path).map_err(|error| {
                StagexMuslError::Materialization(format!("reading musl header metadata {}: {error}", path.display()))
            })?;
            if !metadata.file_type().is_file() {
                continue;
            }
            let relative = path.strip_prefix(&source).map_err(|error| {
                StagexMuslError::Materialization(format!("finding musl header relative path: {error}"))
            })?;
            let target = output_include.join("bits").join(relative);
            if target.exists() {
                fs::remove_file(&target).map_err(|error| {
                    StagexMuslError::Materialization(format!(
                        "removing replaced musl header {}: {error}",
                        target.display()
                    ))
                })?;
            }
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).map_err(|error| {
                    StagexMuslError::Materialization(format!(
                        "creating musl header parent {}: {error}",
                        parent.display()
                    ))
                })?;
            }
            crate::stagex_mes_lib::copy_file_exact(&path, &target)?;
        }
    }
    assert!(output_include.join("stdio.h").is_file());
    assert!(output_include.join("bits/alltypes.h").is_file());
    Ok(())
}

fn run_smokes(
    request: &MuslInventoryRequest<'_>,
    compiler: &Path,
    source_root: &Path,
    output_root: &Path,
) -> Result<PathBuf, StagexMuslError> {
    let smoke = request.scratch_dir.join("smoke");
    fs::create_dir(&smoke)
        .map_err(|error| StagexMuslError::Materialization(format!("creating musl smoke root: {error}")))?;
    crate::stagex_mes_lib::write_create_new(&smoke.join("positive.c"), POSITIVE_SMOKE_SOURCE)?;
    crate::stagex_mes_lib::write_create_new(&smoke.join("malformed.c"), MALFORMED_SMOKE_SOURCE)?;
    let include = absolute_utf8(&output_root.join("include"), "musl smoke include")?;
    let object = smoke.join("positive.o");
    crate::stagex_mes_lib::run_bounded_process(
        compiler,
        &[
            "-nostdinc",
            "-I",
            &include,
            "-c",
            "-o",
            &absolute_utf8(&object, "musl smoke object")?,
            "positive.c",
        ],
        &smoke,
        &BTreeMap::<String, String>::new(),
        &smoke.join("positive-compile.stderr.txt"),
    )?;
    let binary = smoke.join("positive");
    crate::stagex_mes_lib::run_bounded_process(
        compiler,
        &[
            "-static",
            "-nostdlib",
            "-o",
            &absolute_utf8(&binary, "musl smoke binary")?,
            &absolute_utf8(&output_root.join("lib/crt1.o"), "musl smoke crt1")?,
            &absolute_utf8(&object, "musl smoke object")?,
            &absolute_utf8(&output_root.join("lib/libc.a"), "musl smoke libc")?,
            &absolute_utf8(
                &request.compiler_root.join(request.profile.runtime_archive_relative()),
                "musl smoke libtcc1",
            )?,
        ],
        source_root,
        &BTreeMap::<String, String>::new(),
        &smoke.join("positive-link.stderr.txt"),
    )?;
    require_expected_compile_failure(
        compiler,
        &["-nostdinc", "-I", &include, "-c", "-o", "malformed.o", "malformed.c"],
        &smoke,
        &smoke.join("malformed.stderr.txt"),
    )?;
    validate_nonempty_file(&binary, "musl smoke binary")?;
    assert!(object.is_file());
    assert!(!smoke.join("malformed.o").exists());
    Ok(binary)
}

fn require_expected_compile_failure(
    compiler: &Path,
    args: &[&str],
    current_dir: &Path,
    stderr_path: &Path,
) -> Result<(), StagexMuslError> {
    match crate::stagex_mes_lib::run_bounded_process(
        compiler,
        args,
        current_dir,
        &BTreeMap::<String, String>::new(),
        stderr_path,
    ) {
        Err(crate::stagex_mes_lib::MesLibraryPlanError::ProcessFailure {
            exit_code: Some(exit_code),
            stderr,
            ..
        }) if exit_code != 0 && !stderr.trim().is_empty() => {
            assert_ne!(exit_code, 0);
            assert!(!stderr.trim().is_empty());
            Ok(())
        }
        Ok(()) => Err(StagexMuslError::Materialization("musl compiler accepted malformed source".to_string())),
        Err(error) => Err(StagexMuslError::Runtime(error)),
    }
}

fn collect_outputs(
    output_root: &Path,
    libc: &Path,
    smoke_binary: &Path,
    profile: MuslProfile,
) -> Result<Vec<MuslOutputReport>, StagexMuslError> {
    let prefix = profile.output_prefix();
    let specs = [
        (format!("{prefix}-libc"), libc.to_path_buf(), false),
        (format!("{prefix}-crt1"), output_root.join("lib/crt1.o"), false),
        (format!("{prefix}-crti"), output_root.join("lib/crti.o"), false),
        (format!("{prefix}-crtn"), output_root.join("lib/crtn.o"), false),
        (format!("{prefix}-libm"), output_root.join("lib/libm.a"), false),
        (format!("{prefix}-librt"), output_root.join("lib/librt.a"), false),
        (format!("{prefix}-libpthread"), output_root.join("lib/libpthread.a"), false),
        (format!("{prefix}-libcrypt"), output_root.join("lib/libcrypt.a"), false),
        (format!("{prefix}-libutil"), output_root.join("lib/libutil.a"), false),
        (format!("{prefix}-libxnet"), output_root.join("lib/libxnet.a"), false),
        (format!("{prefix}-libresolv"), output_root.join("lib/libresolv.a"), false),
        (format!("{prefix}-libdl"), output_root.join("lib/libdl.a"), false),
        (format!("{prefix}-headers"), output_root.join("include"), true),
        (format!("{prefix}-smoke-binary"), smoke_binary.to_path_buf(), false),
    ];
    let mut outputs = Vec::with_capacity(MUSL_OUTPUT_COUNT);
    for (artifact_id, path, is_tree) in specs {
        let (bytes_len, digest_blake3) = if is_tree {
            (tree_bytes_len(&path)?, tree_digest_blake3(&path)?)
        } else {
            let bytes = crate::stagex_mes_lib::read_bounded_file(&path, MUSL_FILE_BYTES_MAX, &artifact_id)?;
            (
                u64::try_from(bytes.len())
                    .map_err(|_| StagexMuslError::Materialization("musl output size overflow".to_string()))?,
                blake3::hash(&bytes).to_hex().to_string(),
            )
        };
        outputs.push(MuslOutputReport {
            artifact_id,
            path,
            bytes_len,
            digest_blake3,
        });
    }
    assert_eq!(outputs.len(), MUSL_OUTPUT_COUNT);
    assert!(outputs.iter().all(|output| output.bytes_len > 0));
    Ok(outputs)
}

fn validate_expected_outputs(outputs: &[MuslOutputReport], profile: MuslProfile) -> Result<(), StagexMuslError> {
    let expected_outputs = profile.expected_outputs();
    let mut mismatches = Vec::new();
    for expected in expected_outputs {
        let output = outputs
            .iter()
            .find(|output| output.artifact_id == expected.artifact_id)
            .ok_or_else(|| StagexMuslError::Materialization(format!("musl report lacks {}", expected.artifact_id)))?;
        if output.digest_blake3 != expected.digest_blake3 {
            mismatches.push(format!("{}={}", expected.artifact_id, output.digest_blake3));
        }
    }
    if !mismatches.is_empty() {
        return Err(StagexMuslError::Materialization(format!(
            "musl output BLAKE3 mismatches: {}",
            mismatches.join(",")
        )));
    }
    assert_eq!(outputs.len(), expected_outputs.len());
    assert!(outputs.iter().all(|output| output.digest_blake3.len() == blake3::OUT_LEN * 2));
    Ok(())
}

fn configured_source_digest(
    root: &Path,
    libc_sources: &[String],
    crt_sources: &[String],
    profile: MuslProfile,
) -> Result<String, StagexMuslError> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"mantle-stagex-musl-configured-source-v1\0");
    hasher.update(MUSL_SOURCE_CONTENT_BLAKE3.as_bytes());
    hasher.update(b"\0");
    hasher.update(profile.recipe_artifact().1.as_bytes());
    for helper in profile.helpers() {
        hasher.update(&extract_helper(profile.recipe(), helper.target)?);
        hasher.update(b"\0");
    }
    for value in REMOVED_SOURCE_DIRECTORIES.iter().chain(COMMON_COMPILE_FLAGS.iter()) {
        hasher.update(value.as_bytes());
        hasher.update(b"\0");
    }
    for value in libc_sources.iter().chain(crt_sources.iter()) {
        hasher.update(value.as_bytes());
        hasher.update(b"\0");
    }
    for relative in [
        "obj/include/bits/alltypes.h",
        "obj/include/bits/syscall.h",
        "obj/src/internal/version.h",
    ] {
        hasher.update(relative.as_bytes());
        hasher.update(b"\0");
        hasher.update(&fs::read(root.join(relative)).map_err(|error| {
            StagexMuslError::Materialization(format!("reading configured musl input {relative}: {error}"))
        })?);
        hasher.update(b"\0");
    }
    let digest = hasher.finalize().to_hex().to_string();
    if digest != profile.configured_source_digest() {
        return Err(StagexMuslError::Materialization(format!(
            "musl configured source BLAKE3 mismatch: expected {}, observed {digest}",
            profile.configured_source_digest()
        )));
    }
    assert!(!libc_sources.is_empty());
    assert!(!crt_sources.is_empty());
    Ok(digest)
}

fn resolve_materialized_source_root(output_path: &Path) -> Result<PathBuf, StagexMuslError> {
    if output_path.join("configure").is_file() {
        return Ok(output_path.to_path_buf());
    }
    let nested = output_path.join(MUSL_SOURCE_OUTPUT_NAME);
    if nested.join("configure").is_file() {
        assert!(nested.starts_with(output_path));
        assert!(nested.is_dir());
        return Ok(nested);
    }
    Err(StagexMuslError::Materialization(format!(
        "musl source root is not direct or singly nested under {}",
        output_path.display()
    )))
}

fn find_source_record(records: &[SourceRecord]) -> Result<&SourceRecord, StagexMuslError> {
    records
        .iter()
        .find(|record| record.metadata.get(SOURCE_RECORD_NAME_KEY).map(String::as_str) == Some(MUSL_RECORD_NAME))
        .ok_or(StagexMuslError::SourceRecordNotFound)
}

fn validate_source_record(record: &SourceRecord) -> Result<(), StagexMuslError> {
    if record.kind != SourceRecordKind::FixedUrl || record.content_blake3 != MUSL_SOURCE_CONTENT_BLAKE3 {
        return Err(StagexMuslError::Materialization(format!(
            "musl source record {} has substituted kind or content identity",
            record.identity
        )));
    }
    if record.files.len() != 1 || record.files.iter().any(|entry| entry.file_type != SourceFileType::Regular) {
        return Err(StagexMuslError::Materialization(
            "musl source record must contain exactly one archive payload".to_string(),
        ));
    }
    assert_eq!(record.kind, SourceRecordKind::FixedUrl);
    assert_eq!(record.content_blake3, MUSL_SOURCE_CONTENT_BLAKE3);
    Ok(())
}

fn validate_bound_recipe() -> Result<(), StagexMuslError> {
    validate_bound_recipe_for_profile(MuslProfile::First)
}

fn validate_bound_recipe_for_profile(profile: MuslProfile) -> Result<(), StagexMuslError> {
    let recipe = profile.recipe();
    let expected = profile.recipe_artifact().1;
    let observed = blake3::hash(recipe).to_hex().to_string();
    if observed != expected {
        return Err(StagexMuslError::Materialization(format!(
            "musl recipe BLAKE3 mismatch: expected {expected}, observed {observed}"
        )));
    }
    assert!(!recipe.is_empty());
    assert_eq!(observed.len(), blake3::OUT_LEN * 2);
    Ok(())
}

fn bounded_tree_paths(root: &Path) -> Result<Vec<PathBuf>, StagexMuslError> {
    let mut pending = vec![root.to_path_buf()];
    let mut paths = Vec::new();
    while let Some(directory) = pending.pop() {
        let mut children = fs::read_dir(&directory)
            .map_err(|error| {
                StagexMuslError::Materialization(format!("reading tree directory {}: {error}", directory.display()))
            })?
            .map(|entry| {
                entry
                    .map(|value| value.path())
                    .map_err(|error| StagexMuslError::Materialization(format!("reading tree entry: {error}")))
            })
            .collect::<Result<Vec<_>, _>>()?;
        children.sort();
        for child in children.into_iter().rev() {
            let metadata = fs::symlink_metadata(&child).map_err(|error| {
                StagexMuslError::Materialization(format!("reading tree entry {}: {error}", child.display()))
            })?;
            if metadata.file_type().is_dir() {
                pending.push(child.clone());
            }
            paths.push(child);
            if paths.len() > MUSL_TREE_ENTRY_COUNT_MAX {
                return Err(StagexMuslError::Materialization(format!(
                    "tree exceeds {MUSL_TREE_ENTRY_COUNT_MAX} entries"
                )));
            }
        }
    }
    paths.sort();
    assert!(paths.len() <= MUSL_TREE_ENTRY_COUNT_MAX);
    assert!(paths.iter().all(|path| path.starts_with(root)));
    Ok(paths)
}

fn tree_digest_blake3(root: &Path) -> Result<String, StagexMuslError> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"mantle-stagex-musl-tree-v1\0");
    for path in bounded_tree_paths(root)? {
        let relative = path
            .strip_prefix(root)
            .map_err(|error| StagexMuslError::Materialization(format!("finding tree relative path: {error}")))?;
        let relative = relative
            .to_str()
            .ok_or_else(|| StagexMuslError::Materialization("tree path is not UTF-8".to_string()))?;
        let metadata = fs::symlink_metadata(&path).map_err(|error| {
            StagexMuslError::Materialization(format!("reading tree metadata {}: {error}", path.display()))
        })?;
        hasher.update(relative.as_bytes());
        hasher.update(b"\0");
        if metadata.file_type().is_file() {
            hasher.update(b"f\0");
            hasher.update(&fs::read(&path).map_err(|error| {
                StagexMuslError::Materialization(format!("reading tree file {}: {error}", path.display()))
            })?);
        } else if metadata.file_type().is_symlink() {
            hasher.update(b"l\0");
            hasher.update(
                fs::read_link(&path)
                    .map_err(|error| {
                        StagexMuslError::Materialization(format!("reading tree link {}: {error}", path.display()))
                    })?
                    .as_os_str()
                    .as_encoded_bytes(),
            );
        } else if metadata.file_type().is_dir() {
            hasher.update(b"d\0");
        } else {
            return Err(StagexMuslError::Materialization(format!("unsupported tree entry: {}", path.display())));
        }
        hasher.update(b"\0");
    }
    let digest = hasher.finalize().to_hex().to_string();
    assert!(root.is_dir());
    assert_eq!(digest.len(), blake3::OUT_LEN * 2);
    Ok(digest)
}

fn tree_bytes_len(root: &Path) -> Result<u64, StagexMuslError> {
    let mut total = 0_u64;
    for path in bounded_tree_paths(root)? {
        let metadata = fs::symlink_metadata(&path).map_err(|error| {
            StagexMuslError::Materialization(format!("reading tree metadata {}: {error}", path.display()))
        })?;
        if metadata.file_type().is_file() {
            total = total
                .checked_add(metadata.len())
                .ok_or_else(|| StagexMuslError::Materialization("tree byte count overflow".to_string()))?;
        }
    }
    if total == 0 || total > MUSL_FILE_BYTES_MAX {
        return Err(StagexMuslError::Materialization(format!("tree byte count is outside bounds: {total}")));
    }
    assert!(root.is_dir());
    assert!(total > 0);
    Ok(total)
}

fn absolute_utf8(path: &Path, label: &str) -> Result<String, StagexMuslError> {
    crate::stagex_mes_lib::utf8_absolute(path, label)
        .map(str::to_string)
        .map_err(StagexMuslError::Runtime)
}

fn validate_file_digest(path: &Path, expected: &str, label: &str) -> Result<(), StagexMuslError> {
    let bytes = crate::stagex_mes_lib::read_bounded_file(path, MUSL_FILE_BYTES_MAX, label)?;
    let observed = blake3::hash(&bytes).to_hex().to_string();
    if observed != expected {
        return Err(StagexMuslError::Materialization(format!(
            "{label} BLAKE3 mismatch: expected {expected}, observed {observed}"
        )));
    }
    assert!(path.is_file());
    assert!(!bytes.is_empty());
    Ok(())
}

fn validate_nonempty_file(path: &Path, label: &str) -> Result<(), StagexMuslError> {
    let bytes = crate::stagex_mes_lib::read_bounded_file(path, MUSL_FILE_BYTES_MAX, label)?;
    if bytes.is_empty() {
        return Err(StagexMuslError::Materialization(format!("{label} is empty")));
    }
    assert!(path.is_file());
    assert!(!bytes.is_empty());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const RETAINED_SOURCE_BUNDLE_ENV: &str = "MANTLE_STAGE_X_SOURCE_BUNDLE";
    const RETAINED_SOURCE_ROOT_ENV: &str = "MANTLE_STAGE_X_MUSL_SOURCE_ROOT";
    const RETAINED_TCC_MUSL_PREP_ROOT_ENV: &str = "MANTLE_STAGE_X_TCC_MUSL_PREP_ROOT";
    const RETAINED_BUILD_SCRATCH_ENV: &str = "MANTLE_STAGE_X_MUSL_BUILD_SCRATCH";
    const RETAINED_PASS2_BUILD_SCRATCH_ENV: &str = "MANTLE_STAGE_X_MUSL_PASS2_BUILD_SCRATCH";

    #[test]
    fn helper_digests_are_bound() {
        validate_helper_digests(MuslProfile::First).unwrap();
        validate_helper_digests(MuslProfile::Second).unwrap();
        assert_eq!(MUSL_HELPERS.len(), MUSL_HELPER_COUNT);
        assert_eq!(MUSL_PASS2_HELPERS.len(), MUSL_PASS2_HELPER_COUNT);
    }

    #[test]
    fn missing_helper_is_rejected() {
        let error = extract_helper(MUSL_RECIPE_SOURCE, "missing.c").unwrap_err();
        assert!(error.to_string().contains("occurs 0 times"));
        assert!(!error.to_string().is_empty());
    }

    #[test]
    fn alltypes_transform_accepts_typedef_and_rejects_malformed_input() {
        let output = transform_alltypes("TYPEDEF unsigned size_t;\nSTRUCT timespec { int x; };\n").unwrap();
        assert!(output.contains("__DEFINED_size_t"));
        assert!(output.contains("__DEFINED_struct_timespec"));
        let error = transform_alltypes("TYPEDEF broken\n").unwrap_err();
        assert!(error.to_string().contains("malformed"));
    }

    #[test]
    fn missing_source_record_is_rejected() {
        let error = find_source_record(&[]).unwrap_err();
        assert!(error.to_string().contains("not found"));
        assert!(!error.to_string().is_empty());
    }

    #[test]
    #[ignore = "requires retained authenticated source bundle"]
    fn materializes_retained_musl_source() {
        let bundle = PathBuf::from(std::env::var(RETAINED_SOURCE_BUNDLE_ENV).unwrap());
        let scratch = PathBuf::from(std::env::var(RETAINED_SOURCE_ROOT_ENV).unwrap());
        let expected_manifest = read_source_bundle(&bundle).unwrap().manifest_blake3;
        let report = materialize_authenticated_musl_source(&bundle, &expected_manifest, &scratch).unwrap();
        assert_eq!(report.record_content_blake3, MUSL_SOURCE_CONTENT_BLAKE3);
        assert!(report.output_path.join("configure").is_file());
    }

    #[test]
    #[ignore = "requires retained musl source and TinyCC musl-prep"]
    fn derives_retained_musl_inventory() {
        let source_root = PathBuf::from(std::env::var(RETAINED_SOURCE_ROOT_ENV).unwrap());
        let prep_root = PathBuf::from(std::env::var(RETAINED_TCC_MUSL_PREP_ROOT_ENV).unwrap());
        let scratch_dir = PathBuf::from(std::env::var(RETAINED_BUILD_SCRATCH_ENV).unwrap());
        let report = derive_musl_inventory(MuslInventoryRequest {
            source_root: &source_root,
            compiler_root: &prep_root,
            profile: MuslProfile::First,
            scratch_dir: &scratch_dir,
            protected_exec_enforced: false,
        })
        .unwrap();
        assert_eq!(report.helper_count, u32::try_from(MUSL_HELPER_COUNT).unwrap());
        assert!(report.fallback_events.is_empty());
    }

    #[test]
    #[ignore = "requires retained musl source and musl-linked TinyCC"]
    fn derives_retained_musl_pass2_inventory() {
        let source_root = PathBuf::from(std::env::var(RETAINED_SOURCE_ROOT_ENV).unwrap());
        let compiler_root = PathBuf::from(std::env::var(RETAINED_TCC_MUSL_PREP_ROOT_ENV).unwrap());
        let scratch_dir = PathBuf::from(std::env::var(RETAINED_PASS2_BUILD_SCRATCH_ENV).unwrap());
        let report = derive_musl_inventory(MuslInventoryRequest {
            source_root: &source_root,
            compiler_root: &compiler_root,
            profile: MuslProfile::Second,
            scratch_dir: &scratch_dir,
            protected_exec_enforced: false,
        })
        .unwrap();
        assert_eq!(report.helper_count, u32::try_from(MUSL_PASS2_HELPER_COUNT).unwrap());
        assert!(report.fallback_events.is_empty());
    }
}
