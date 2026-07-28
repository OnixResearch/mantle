use std::collections::BTreeMap;
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
const COREUTILS_RECORD_NAME: &str = "coreutils-5.0-src";
const COREUTILS_SOURCE_OUTPUT_NAME: &str = "coreutils-5.0";
pub(crate) const COREUTILS_SOURCE_ARTIFACT_ID: &str = "coreutils-5.0-source";
pub(crate) const COREUTILS_SOURCE_CONTENT_BLAKE3: &str =
    "82d3dd8a0cbb5b5032512d747b5bb79b5dfdfd94b2cd11d9ae50a1dd0b05d814";
pub(crate) const COREUTILS_CONFIG_SOURCE_ARTIFACT_ID: &str = "coreutils-5.0-config-header-source";
pub(crate) const COREUTILS_CONFIG_SOURCE_BLAKE3: &str =
    "03f96124348df7f962ee7c2e23ecfb97a7eeadcbd32a50e503850d5d0c6513b0";
const COREUTILS_CONFIG_SOURCE: &[u8] = include_bytes!("../bootstrap/seeds/coreutils-5.0-config.h");
const COREUTILS_PATCH_COUNT: u32 = 8;

#[derive(Debug, Clone, Copy)]
struct CoreutilsPatchSpec {
    artifact_id: &'static str,
    file_name: &'static str,
    digest_blake3: &'static str,
    bytes: &'static [u8],
}

const COREUTILS_PATCHES: [CoreutilsPatchSpec; COREUTILS_PATCH_COUNT as usize] = [
    CoreutilsPatchSpec {
        artifact_id: "coreutils-modechange-patch-source",
        file_name: "coreutils-5.0-modechange.patch",
        digest_blake3: "b3be0a3a118ad92371d0a1b5d664c216d33964a22e18060184e9d10d65d0ddf8",
        bytes: include_bytes!("../bootstrap/seeds/coreutils-5.0-modechange.patch"),
    },
    CoreutilsPatchSpec {
        artifact_id: "coreutils-mbstate-patch-source",
        file_name: "coreutils-5.0-mbstate.patch",
        digest_blake3: "d6a42d969bc7665e7b9193f15b0bb583c5d6fd1277ab2aa5a180c808aa70a877",
        bytes: include_bytes!("../bootstrap/seeds/coreutils-5.0-mbstate.patch"),
    },
    CoreutilsPatchSpec {
        artifact_id: "coreutils-ls-strcmp-patch-source",
        file_name: "coreutils-5.0-ls-strcmp.patch",
        digest_blake3: "80c602c5e65288de9bf4153acc139b71477d89572ef6dc100bc39d40aa93cbd8",
        bytes: include_bytes!("../bootstrap/seeds/coreutils-5.0-ls-strcmp.patch"),
    },
    CoreutilsPatchSpec {
        artifact_id: "coreutils-touch-getdate-patch-source",
        file_name: "coreutils-5.0-touch-getdate.patch",
        digest_blake3: "ea379c4de61b38f39aebb7cac135e0c462814065f7849b9a5bb819154d0ee7f5",
        bytes: include_bytes!("../bootstrap/seeds/coreutils-5.0-touch-getdate.patch"),
    },
    CoreutilsPatchSpec {
        artifact_id: "coreutils-tac-uint64-patch-source",
        file_name: "coreutils-5.0-tac-uint64.patch",
        digest_blake3: "e9441c3a23908b695aa7084b50a05065ed7daf852d573645ba82499bbdd6c995",
        bytes: include_bytes!("../bootstrap/seeds/coreutils-5.0-tac-uint64.patch"),
    },
    CoreutilsPatchSpec {
        artifact_id: "coreutils-expr-strcmp-patch-source",
        file_name: "coreutils-5.0-expr-strcmp.patch",
        digest_blake3: "0b808546ae32923f754a12dc0c7578aa68624400159a5c97a93cd49a6252f1a6",
        bytes: include_bytes!("../bootstrap/seeds/coreutils-5.0-expr-strcmp.patch"),
    },
    CoreutilsPatchSpec {
        artifact_id: "coreutils-sort-locale-patch-source",
        file_name: "coreutils-5.0-sort-locale.patch",
        digest_blake3: "69792c09234934fd70f1795affca68cdae6c622a7532898d55794572597ae7c7",
        bytes: include_bytes!("../bootstrap/seeds/coreutils-5.0-sort-locale.patch"),
    },
    CoreutilsPatchSpec {
        artifact_id: "coreutils-hash-tcc-patch-source",
        file_name: "coreutils-5.0-hash-tcc.patch",
        digest_blake3: "407c0d0457158c2ceb0a165babfeb5e2f574c3f3bea44b774eb336203a2d6daf",
        bytes: include_bytes!("../bootstrap/seeds/coreutils-5.0-hash-tcc.patch"),
    },
];
const COREUTILS_SOURCE_REPORT_FORMAT: &str = "mantle-stagex-coreutils-source-materialization-v1";
const COREUTILS_REPORT_FORMAT: &str = "mantle-stagex-coreutils-5.0-inventory-v1";
const COREUTILS_SOURCE_NON_CLAIM: &str =
    "coreutils source materialization proves authenticated offline archive identity and fixed-output parity only";
const COREUTILS_NON_CLAIM: &str = "this inventory binds a bounded coreutils 5.0 utility set and positive and negative filesystem observations only; it does not prove a general POSIX environment or provider admission";
const COREUTILS_FILE_BYTES_MAX: u64 = 64 * 1_024 * 1_024;
const COREUTILS_LIBRARY_COMPILE_COUNT: u32 = 96;
const COREUTILS_ARCHIVE_COMMAND_COUNT: u32 = 1;
const COREUTILS_UTILITY_SOURCE_COMPILE_COUNT: u32 = 34;
const COREUTILS_UTILITY_LINK_COUNT: u32 = 25;
const COREUTILS_BUILD_COMMAND_COUNT: u32 = COREUTILS_LIBRARY_COMPILE_COUNT
    + COREUTILS_ARCHIVE_COMMAND_COUNT
    + COREUTILS_UTILITY_SOURCE_COMPILE_COUNT
    + COREUTILS_UTILITY_LINK_COUNT;
const COREUTILS_SMOKE_COMMAND_COUNT: u32 = 6;
const COREUTILS_BINARY_COUNT: usize = 26;
const COREUTILS_OUTPUT_COUNT: usize = COREUTILS_BINARY_COUNT + 1;
const COREUTILS_SOURCE_ARTIFACT_COUNT: usize = 2 + COREUTILS_PATCH_COUNT as usize;
const COREUTILS_SMOKE_INPUT: &[u8] = b"coreutils-50-ok\n";
pub(crate) const COREUTILS_CONFIGURED_SOURCE_BLAKE3: &str =
    "232ccf0c23c2e8a028c5ca42cbb46f3afa1aad664a5d5187e11fecdcb04334d4";

const COMMON_FLAGS: [&str; 6] = [
    "-I.",
    "-Ilib",
    "-Isrc",
    "-DHAVE_CONFIG_H",
    "-D_FILE_OFFSET_BITS=64",
    "-c",
];
const LINK_FLAGS: [&str; 5] = ["-I.", "-Ilib", "-Isrc", "-DHAVE_CONFIG_H", "-D_FILE_OFFSET_BITS=64"];
const LIBRARY_SOURCES: [&str; COREUTILS_LIBRARY_COMPILE_COUNT as usize] = [
    "lib/acl.c",
    "lib/posixtm.c",
    "lib/posixver.c",
    "lib/strftime.c",
    "lib/getopt.c",
    "lib/getopt1.c",
    "lib/hash.c",
    "lib/hash-pjw.c",
    "lib/addext.c",
    "lib/argmatch.c",
    "lib/backupfile.c",
    "lib/basename.c",
    "lib/canon-host.c",
    "lib/closeout.c",
    "lib/cycle-check.c",
    "lib/diacrit.c",
    "lib/dirname.c",
    "lib/dup-safer.c",
    "lib/error.c",
    "lib/exclude.c",
    "lib/exitfail.c",
    "lib/filemode.c",
    "lib/__fpending.c",
    "lib/file-type.c",
    "lib/fnmatch.c",
    "lib/fopen-safer.c",
    "lib/full-read.c",
    "lib/full-write.c",
    "lib/gethostname.c",
    "lib/getline.c",
    "lib/getstr.c",
    "lib/gettime.c",
    "lib/hard-locale.c",
    "lib/human.c",
    "lib/idcache.c",
    "lib/isdir.c",
    "lib/imaxtostr.c",
    "lib/linebuffer.c",
    "lib/localcharset.c",
    "lib/long-options.c",
    "lib/makepath.c",
    "lib/mbswidth.c",
    "lib/md5.c",
    "lib/memcasecmp.c",
    "lib/memcoll.c",
    "lib/modechange.c",
    "lib/offtostr.c",
    "lib/path-concat.c",
    "lib/physmem.c",
    "lib/quote.c",
    "lib/quotearg.c",
    "lib/readtokens.c",
    "lib/rpmatch.c",
    "lib/safe-read.c",
    "lib/safe-write.c",
    "lib/same.c",
    "lib/save-cwd.c",
    "lib/savedir.c",
    "lib/settime.c",
    "lib/sha.c",
    "lib/stpcpy.c",
    "lib/stripslash.c",
    "lib/strtoimax.c",
    "lib/strtoumax.c",
    "lib/umaxtostr.c",
    "lib/unicodeio.c",
    "lib/userspec.c",
    "lib/version-etc.c",
    "lib/xgetcwd.c",
    "lib/xgethostname.c",
    "lib/xmalloc.c",
    "lib/xmemcoll.c",
    "lib/xnanosleep.c",
    "lib/xreadlink.c",
    "lib/xstrdup.c",
    "lib/xstrtod.c",
    "lib/xstrtol.c",
    "lib/xstrtoul.c",
    "lib/xstrtoimax.c",
    "lib/xstrtoumax.c",
    "lib/yesno.c",
    "lib/strnlen.c",
    "lib/getcwd.c",
    "lib/sig2str.c",
    "lib/mountlist.c",
    "lib/regex.c",
    "lib/canonicalize.c",
    "lib/mkstemp.c",
    "lib/memrchr.c",
    "lib/euidaccess.c",
    "lib/ftw.c",
    "lib/dirfd.c",
    "lib/obstack.c",
    "lib/strverscmp.c",
    "lib/tempname.c",
    "lib/tsearch.c",
];

#[derive(Debug, Clone, Copy)]
struct UtilitySpec {
    name: &'static str,
    sources: &'static [&'static str],
}

const UTILITY_SPECS: [UtilitySpec; COREUTILS_UTILITY_LINK_COUNT as usize] = [
    UtilitySpec {
        name: "cat",
        sources: &["src/cat.c"],
    },
    UtilitySpec {
        name: "chmod",
        sources: &["src/chmod.c"],
    },
    UtilitySpec {
        name: "cp",
        sources: &["src/cp.c", "src/copy.c", "src/cp-hash.c"],
    },
    UtilitySpec {
        name: "echo",
        sources: &["src/echo.c"],
    },
    UtilitySpec {
        name: "install",
        sources: &["src/install.c", "src/copy.c", "src/cp-hash.c"],
    },
    UtilitySpec {
        name: "ln",
        sources: &["src/ln.c"],
    },
    UtilitySpec {
        name: "ls",
        sources: &["src/ls.c", "src/ls-ls.c"],
    },
    UtilitySpec {
        name: "mkdir",
        sources: &["src/mkdir.c"],
    },
    UtilitySpec {
        name: "mv",
        sources: &["src/mv.c", "src/copy.c", "src/cp-hash.c", "src/remove.c"],
    },
    UtilitySpec {
        name: "rm",
        sources: &["src/rm.c", "src/remove.c"],
    },
    UtilitySpec {
        name: "rmdir",
        sources: &["src/rmdir.c"],
    },
    UtilitySpec {
        name: "sort",
        sources: &["src/sort.c"],
    },
    UtilitySpec {
        name: "test",
        sources: &["src/test.c"],
    },
    UtilitySpec {
        name: "true",
        sources: &["src/true.c"],
    },
    UtilitySpec {
        name: "false",
        sources: &["src/false.c"],
    },
    UtilitySpec {
        name: "head",
        sources: &["src/head.c"],
    },
    UtilitySpec {
        name: "tail",
        sources: &["src/tail.c"],
    },
    UtilitySpec {
        name: "wc",
        sources: &["src/wc.c"],
    },
    UtilitySpec {
        name: "basename",
        sources: &["src/basename.c"],
    },
    UtilitySpec {
        name: "dirname",
        sources: &["src/dirname.c"],
    },
    UtilitySpec {
        name: "tr",
        sources: &["src/tr.c"],
    },
    UtilitySpec {
        name: "uniq",
        sources: &["src/uniq.c"],
    },
    UtilitySpec {
        name: "expr",
        sources: &["src/expr.c"],
    },
    UtilitySpec {
        name: "tee",
        sources: &["src/tee.c"],
    },
    UtilitySpec {
        name: "touch",
        sources: &["src/touch.c"],
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CoreutilsExpectedOutput {
    pub artifact_id: &'static str,
    pub digest_blake3: &'static str,
}

pub(crate) const COREUTILS_EXPECTED_OUTPUTS: [CoreutilsExpectedOutput; COREUTILS_OUTPUT_COUNT] = [
    CoreutilsExpectedOutput {
        artifact_id: "coreutils-cat",
        digest_blake3: "3d9de9e7f81612a74e75312f22d038b665e69c5d9590662c293843e9711cf020",
    },
    CoreutilsExpectedOutput {
        artifact_id: "coreutils-chmod",
        digest_blake3: "8fe8251243646c8e44ffe50e827cb27f2197eed344c498b31c365c7d27e07779",
    },
    CoreutilsExpectedOutput {
        artifact_id: "coreutils-cp",
        digest_blake3: "8c11ead9af84f2371178ce892afbac6daa416c445f2575a2300b2b5ffadf17f3",
    },
    CoreutilsExpectedOutput {
        artifact_id: "coreutils-echo",
        digest_blake3: "3ce3cfec07fda35f97485b30fddd8a14f4cb04bd8427d2aa3acc4e94fd650a3f",
    },
    CoreutilsExpectedOutput {
        artifact_id: "coreutils-install",
        digest_blake3: "7f332bcf208b458595c202f4b1f2faf3fffadd80f7cd34ff67c711ee6183b210",
    },
    CoreutilsExpectedOutput {
        artifact_id: "coreutils-ln",
        digest_blake3: "8146091298782331503b99b3f0bf02b0a92de93a5fe7b57fba08b5d870087c33",
    },
    CoreutilsExpectedOutput {
        artifact_id: "coreutils-ls",
        digest_blake3: "32e2fcd7893befd1096a15b0a44992dcfd522044aed0601df96f434b39a60683",
    },
    CoreutilsExpectedOutput {
        artifact_id: "coreutils-mkdir",
        digest_blake3: "b94fc26390ef29d55b71e608b800fa0451ff477b33c15ff3d2be3c8a2a06621b",
    },
    CoreutilsExpectedOutput {
        artifact_id: "coreutils-mv",
        digest_blake3: "d765750db422dcaf98b1203911d1bbd0569a1cbe5a2e7006c2e26a476e420886",
    },
    CoreutilsExpectedOutput {
        artifact_id: "coreutils-rm",
        digest_blake3: "f1bf6e9330c0b9e1a88b85185f5834a38b2b89526b4ca95bbb095930e57714da",
    },
    CoreutilsExpectedOutput {
        artifact_id: "coreutils-rmdir",
        digest_blake3: "0a8235d566edcd18d68ed94df5ed2449e0fa9a4d37394367cb832536fa554dbc",
    },
    CoreutilsExpectedOutput {
        artifact_id: "coreutils-sort",
        digest_blake3: "8b861f615fa0f7e868107b5c788562ba1d4dc2fff2ed9e9bca0f7d7df6c455bf",
    },
    CoreutilsExpectedOutput {
        artifact_id: "coreutils-test",
        digest_blake3: "87baf4f61820a2861bbb1a08b3bd9b4c7a5b8baed66ddb0e152d7a981ae27669",
    },
    CoreutilsExpectedOutput {
        artifact_id: "coreutils-true",
        digest_blake3: "deb8e7848bca7728c6594600c9d5a1b40d64beb6729fec2e71785236af2cc1e9",
    },
    CoreutilsExpectedOutput {
        artifact_id: "coreutils-false",
        digest_blake3: "b476958f8326512fe4fa9563cb97bf79a658900b646f64c8f9f84426ae099ac9",
    },
    CoreutilsExpectedOutput {
        artifact_id: "coreutils-head",
        digest_blake3: "71a84fad92471f61509ba9b781414934bb3b0975fd1635cef51709f952bf682b",
    },
    CoreutilsExpectedOutput {
        artifact_id: "coreutils-tail",
        digest_blake3: "d0840c1a769287b4c1acf913dd8c4cc396294158dd57e9cd5bcf3e7cdca06962",
    },
    CoreutilsExpectedOutput {
        artifact_id: "coreutils-wc",
        digest_blake3: "bc5ae003d38d9c4ca64568d97fe444d23e62258c93a936015e242806fb028f49",
    },
    CoreutilsExpectedOutput {
        artifact_id: "coreutils-basename",
        digest_blake3: "e6c5b456a4f2279400f46ff272734dfb68829281c9e702c9fcc1e1c852eb1074",
    },
    CoreutilsExpectedOutput {
        artifact_id: "coreutils-dirname",
        digest_blake3: "b21c5d4963d0d336150374699a4398c920fb2b323c265be463baf5264dae940e",
    },
    CoreutilsExpectedOutput {
        artifact_id: "coreutils-tr",
        digest_blake3: "8c7c183c9af7a4fddcf526b8f0318ffb847faea0be75967de1bb72e0d51b4ea0",
    },
    CoreutilsExpectedOutput {
        artifact_id: "coreutils-uniq",
        digest_blake3: "fadb7ca3f75ac06d73d01e8e304c86f06364a91f6001ba13717e69eb1dcbc328",
    },
    CoreutilsExpectedOutput {
        artifact_id: "coreutils-expr",
        digest_blake3: "75770b1702a18bec30618d11cc55eb07eafc5f1349ca0d1d0567e24534b437f2",
    },
    CoreutilsExpectedOutput {
        artifact_id: "coreutils-tee",
        digest_blake3: "d5ec19db795aef5f981b4f99e310a4e977c0543dffbf192cc4539bcb0fff3975",
    },
    CoreutilsExpectedOutput {
        artifact_id: "coreutils-touch",
        digest_blake3: "a2b77b46e85c94390f459f158ebb33c529bbc5eb730c7719f24632f1fd6b6a4c",
    },
    CoreutilsExpectedOutput {
        artifact_id: "coreutils-bracket",
        digest_blake3: "87baf4f61820a2861bbb1a08b3bd9b4c7a5b8baed66ddb0e152d7a981ae27669",
    },
    CoreutilsExpectedOutput {
        artifact_id: "coreutils-smoke",
        digest_blake3: "fd65316f83e13de60a964675b6ab49ee8bd0e40db3aef391c8c1cbee240375d8",
    },
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct CoreutilsSourceMaterializationReport {
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
pub(crate) struct CoreutilsOutputReport {
    pub artifact_id: String,
    pub path: PathBuf,
    pub bytes_len: u64,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct CoreutilsInventoryReport {
    pub format: &'static str,
    pub configured_source_digest_blake3: String,
    pub patch_command_count: u32,
    pub library_compile_count: u32,
    pub archive_command_count: u32,
    pub utility_source_compile_count: u32,
    pub utility_link_count: u32,
    pub build_command_count: u32,
    pub smoke_command_count: u32,
    pub outputs: Vec<CoreutilsOutputReport>,
    pub protected_exec_enforced: bool,
    pub fallback_events: Vec<String>,
    pub non_claim: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct CoreutilsInventoryRequest<'a> {
    pub source_root: &'a Path,
    pub tinycc27_root: &'a Path,
    pub patch_root: &'a Path,
    pub scratch_dir: &'a Path,
    pub protected_exec_enforced: bool,
}

#[derive(Debug)]
pub(crate) enum StagexCoreutilsError {
    SourceRecordNotFound,
    Materialization(String),
    Runtime(crate::stagex_mes_lib::MesLibraryPlanError),
}

impl std::fmt::Display for StagexCoreutilsError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SourceRecordNotFound => write!(formatter, "coreutils source record was not found"),
            Self::Materialization(message) => write!(formatter, "coreutils materialization failed: {message}"),
            Self::Runtime(error) => write!(formatter, "coreutils runtime failed: {error}"),
        }
    }
}

impl std::error::Error for StagexCoreutilsError {}

impl From<crate::stagex_mes_lib::MesLibraryPlanError> for StagexCoreutilsError {
    fn from(error: crate::stagex_mes_lib::MesLibraryPlanError) -> Self {
        Self::Runtime(error)
    }
}

impl From<crate::stagex_tinycc::TinyccError> for StagexCoreutilsError {
    fn from(error: crate::stagex_tinycc::TinyccError) -> Self {
        Self::Materialization(error.to_string())
    }
}

pub(crate) fn source_artifact_digests() -> Vec<(&'static str, &'static str)> {
    let mut artifacts = Vec::with_capacity(COREUTILS_SOURCE_ARTIFACT_COUNT);
    artifacts.push((COREUTILS_SOURCE_ARTIFACT_ID, COREUTILS_SOURCE_CONTENT_BLAKE3));
    artifacts.push((COREUTILS_CONFIG_SOURCE_ARTIFACT_ID, COREUTILS_CONFIG_SOURCE_BLAKE3));
    artifacts.extend(COREUTILS_PATCHES.iter().map(|patch| (patch.artifact_id, patch.digest_blake3)));
    assert_eq!(artifacts.len(), COREUTILS_SOURCE_ARTIFACT_COUNT);
    assert!(artifacts.iter().all(|(_, digest)| digest.len() == blake3::OUT_LEN * 2));
    artifacts
}

pub(crate) fn materialize_authenticated_coreutils_source(
    bundle_path: &Path,
    expected_manifest_blake3: &str,
    scratch_dir: &Path,
) -> Result<CoreutilsSourceMaterializationReport, StagexCoreutilsError> {
    if !bundle_path.is_absolute() || !bundle_path.is_file() {
        return Err(StagexCoreutilsError::Materialization(format!(
            "source bundle is not an absolute file: {}",
            bundle_path.display()
        )));
    }
    let manifest = read_source_bundle(bundle_path)
        .map_err(|error| StagexCoreutilsError::Materialization(format!("reading source bundle: {error}")))?;
    if manifest.manifest_blake3 != expected_manifest_blake3 {
        return Err(StagexCoreutilsError::Materialization(format!(
            "source-bundle manifest mismatch: expected {expected_manifest_blake3}, observed {}",
            manifest.manifest_blake3
        )));
    }
    let record = find_source_record(&manifest.records)?;
    validate_source_record(record)?;
    fs::create_dir(scratch_dir).map_err(|error| {
        StagexCoreutilsError::Materialization(format!("creating coreutils source scratch: {error}"))
    })?;
    let output_path = scratch_dir.join(COREUTILS_SOURCE_OUTPUT_NAME);
    materialize_source_record_for_offline_use(record, &output_path).map_err(|error| {
        StagexCoreutilsError::Materialization(format!("materializing coreutils source {}: {error}", record.identity))
    })?;
    if !output_path.join("src/cat.c").is_file() || !output_path.join("lib/error.c").is_file() {
        return Err(StagexCoreutilsError::Materialization(
            "materialized coreutils tree lacks required source files".to_string(),
        ));
    }
    assert!(output_path.is_absolute());
    assert_eq!(record.content_blake3, COREUTILS_SOURCE_CONTENT_BLAKE3);
    Ok(CoreutilsSourceMaterializationReport {
        format: COREUTILS_SOURCE_REPORT_FORMAT,
        source_bundle_manifest_blake3: manifest.manifest_blake3,
        artifact_id: COREUTILS_SOURCE_ARTIFACT_ID,
        record_name: COREUTILS_RECORD_NAME,
        record_identity: record.identity.clone(),
        record_content_blake3: record.content_blake3.clone(),
        output_path,
        non_claim: COREUTILS_SOURCE_NON_CLAIM,
    })
}

pub(crate) fn derive_coreutils_inventory(
    request: CoreutilsInventoryRequest<'_>,
) -> Result<CoreutilsInventoryReport, StagexCoreutilsError> {
    validate_inventory_inputs(&request)?;
    fs::create_dir(request.scratch_dir)
        .map_err(|error| StagexCoreutilsError::Materialization(format!("creating coreutils scratch: {error}")))?;
    let source_root = request.scratch_dir.join(COREUTILS_SOURCE_OUTPUT_NAME);
    crate::stagex_mes_lib::copy_tree_bounded(request.source_root, &source_root)?;
    crate::stagex_tinycc::make_tree_owner_writable(&source_root)?;
    materialize_generated_headers(&source_root)?;
    crate::stagex_mes_lib::write_create_new(&source_root.join("config.h"), COREUTILS_CONFIG_SOURCE)?;
    apply_source_patches(&request, &source_root)?;
    let configured_source_digest_blake3 = configured_source_digest_blake3();
    let output_root = request.scratch_dir.join("output/bin");
    fs::create_dir_all(&output_root)
        .map_err(|error| StagexCoreutilsError::Materialization(format!("creating coreutils output: {error}")))?;
    build_coreutils(&request, &source_root, &output_root)?;
    run_coreutils_smokes(&request, &source_root, &output_root)?;
    let outputs = collect_outputs(&source_root, &output_root)?;
    validate_expected_outputs(&outputs)?;
    let report = CoreutilsInventoryReport {
        format: COREUTILS_REPORT_FORMAT,
        configured_source_digest_blake3,
        patch_command_count: COREUTILS_PATCH_COUNT,
        library_compile_count: COREUTILS_LIBRARY_COMPILE_COUNT,
        archive_command_count: COREUTILS_ARCHIVE_COMMAND_COUNT,
        utility_source_compile_count: COREUTILS_UTILITY_SOURCE_COMPILE_COUNT,
        utility_link_count: COREUTILS_UTILITY_LINK_COUNT,
        build_command_count: COREUTILS_BUILD_COMMAND_COUNT,
        smoke_command_count: COREUTILS_SMOKE_COMMAND_COUNT,
        outputs,
        protected_exec_enforced: request.protected_exec_enforced,
        fallback_events: Vec::new(),
        non_claim: COREUTILS_NON_CLAIM,
    };
    crate::stagex_mes_lib::write_create_new(
        &request.scratch_dir.join("coreutils-inventory.json"),
        &serde_json::to_vec_pretty(&report)
            .map_err(|error| StagexCoreutilsError::Materialization(format!("serializing coreutils report: {error}")))?,
    )?;
    assert_eq!(report.outputs.len(), COREUTILS_OUTPUT_COUNT);
    assert!(report.fallback_events.is_empty());
    Ok(report)
}

fn validate_inventory_inputs(request: &CoreutilsInventoryRequest<'_>) -> Result<(), StagexCoreutilsError> {
    if request.scratch_dir.exists() {
        return Err(StagexCoreutilsError::Materialization(format!(
            "create-new coreutils scratch already exists: {}",
            request.scratch_dir.display()
        )));
    }
    for (label, root) in [
        ("coreutils source", request.source_root),
        ("TinyCC 0.9.27 runtime", request.tinycc27_root),
        ("GNU patch runtime", request.patch_root),
    ] {
        if !root.is_absolute() || !root.is_dir() {
            return Err(StagexCoreutilsError::Materialization(format!(
                "{label} root is not an absolute directory: {}",
                root.display()
            )));
        }
    }
    validate_file_digest(
        &request.tinycc27_root.join("bin/tcc"),
        crate::stagex_tinycc27::TINYCC27_FINAL_BLAKE3,
        "TinyCC 0.9.27 compiler",
    )?;
    validate_bound_bytes(COREUTILS_CONFIG_SOURCE, COREUTILS_CONFIG_SOURCE_BLAKE3, "coreutils config header")?;
    validate_file_digest(
        &request.patch_root.join("bin/patch"),
        crate::stagex_gnu_patch::GNU_PATCH_FINAL_BLAKE3,
        "GNU patch 2.5.9",
    )?;
    for patch in COREUTILS_PATCHES {
        validate_bound_bytes(patch.bytes, patch.digest_blake3, patch.file_name)?;
    }
    assert!(request.source_root.join("src/cat.c").is_file());
    assert!(request.tinycc27_root.join("lib/mes/tcc/libtcc1.a").is_file());
    Ok(())
}

fn apply_source_patches(
    request: &CoreutilsInventoryRequest<'_>,
    source_root: &Path,
) -> Result<(), StagexCoreutilsError> {
    let patch_dir = request.scratch_dir.join("patches");
    fs::create_dir(&patch_dir).map_err(|error| {
        StagexCoreutilsError::Materialization(format!("creating coreutils patch directory: {error}"))
    })?;
    let patch_executable = request.patch_root.join("bin/patch");
    for (index, patch) in COREUTILS_PATCHES.iter().enumerate() {
        let patch_path = patch_dir.join(patch.file_name);
        crate::stagex_mes_lib::write_create_new(&patch_path, patch.bytes)?;
        crate::stagex_mes_lib::run_bounded_process(
            &patch_executable,
            &[
                "-Np1",
                "-i",
                crate::stagex_mes_lib::utf8_absolute(&patch_path, "coreutils patch")?,
            ],
            source_root,
            &BTreeMap::<String, String>::new(),
            &request.scratch_dir.join(format!("coreutils-patch-{index:02}.stderr.txt")),
        )?;
    }
    assert_eq!(COREUTILS_PATCHES.len(), usize::try_from(COREUTILS_PATCH_COUNT).unwrap());
    assert!(source_root.join("lib/mbstate_t.h").is_file());
    Ok(())
}

fn materialize_generated_headers(source_root: &Path) -> Result<(), StagexCoreutilsError> {
    for (source_name, target_name) in [
        ("lib/fnmatch_.h", "lib/fnmatch.h"),
        ("lib/ftw_.h", "lib/ftw.h"),
        ("lib/search_.h", "lib/search.h"),
    ] {
        let bytes = fs::read(source_root.join(source_name)).map_err(|error| {
            StagexCoreutilsError::Materialization(format!("reading generated header source {source_name}: {error}"))
        })?;
        crate::stagex_mes_lib::write_create_new(&source_root.join(target_name), &bytes)?;
        if bytes.is_empty() {
            return Err(StagexCoreutilsError::Materialization(format!(
                "generated header source is empty: {source_name}"
            )));
        }
    }
    assert!(source_root.join("lib/fnmatch.h").is_file());
    assert!(source_root.join("lib/ftw.h").is_file());
    Ok(())
}

fn build_coreutils(
    request: &CoreutilsInventoryRequest<'_>,
    source_root: &Path,
    output_root: &Path,
) -> Result<(), StagexCoreutilsError> {
    let compiler = request.tinycc27_root.join("bin/tcc");
    let mut objects = Vec::with_capacity(LIBRARY_SOURCES.len());
    for (index, source_name) in LIBRARY_SOURCES.iter().enumerate() {
        let object_name = source_name.trim_end_matches(".c").to_string() + ".o";
        compile_library_source(request, &compiler, source_root, source_name, &object_name, index)?;
        objects.push(object_name);
    }
    archive_library(request, &compiler, source_root, &objects)?;
    for (index, utility) in UTILITY_SPECS.iter().enumerate() {
        link_utility(request, &compiler, source_root, output_root, utility, index)?;
    }
    let test = output_root.join("test");
    let bracket = output_root.join("[");
    fs::copy(&test, &bracket)
        .map_err(|error| StagexCoreutilsError::Materialization(format!("copying bracket utility: {error}")))?;
    crate::stagex_tinycc::set_owner_executable(&bracket)?;
    validate_nonempty_file(&bracket, "coreutils bracket")?;
    assert_eq!(objects.len(), LIBRARY_SOURCES.len());
    assert!(bracket.is_file());
    Ok(())
}

fn compile_library_source(
    request: &CoreutilsInventoryRequest<'_>,
    compiler: &Path,
    source_root: &Path,
    source_name: &str,
    object_name: &str,
    index: usize,
) -> Result<(), StagexCoreutilsError> {
    let mut args = compiler_prefix(request.tinycc27_root)?;
    args.extend(COMMON_FLAGS.iter().map(|flag| (*flag).to_string()));
    args.extend([source_name.to_string(), "-o".to_string(), object_name.to_string()]);
    crate::stagex_mes_lib::run_bounded_process(
        compiler,
        &args,
        source_root,
        &BTreeMap::<String, String>::new(),
        &request.scratch_dir.join(format!("coreutils-library-{index:02}.stderr.txt")),
    )?;
    validate_nonempty_file(&source_root.join(object_name), object_name)?;
    assert!(source_name.starts_with("lib/"));
    assert!(object_name.ends_with(".o"));
    Ok(())
}

fn archive_library(
    request: &CoreutilsInventoryRequest<'_>,
    compiler: &Path,
    source_root: &Path,
    objects: &[String],
) -> Result<(), StagexCoreutilsError> {
    let mut args = vec!["-ar".to_string(), "cr".to_string(), "libcu.a".to_string()];
    args.extend(objects.iter().cloned());
    crate::stagex_mes_lib::run_bounded_process(
        compiler,
        &args,
        source_root,
        &BTreeMap::<String, String>::new(),
        &request.scratch_dir.join("coreutils-archive.stderr.txt"),
    )?;
    validate_nonempty_file(&source_root.join("libcu.a"), "coreutils support archive")?;
    assert_eq!(objects.len(), LIBRARY_SOURCES.len());
    assert!(args.len() > objects.len());
    Ok(())
}

fn link_utility(
    request: &CoreutilsInventoryRequest<'_>,
    compiler: &Path,
    source_root: &Path,
    output_root: &Path,
    utility: &UtilitySpec,
    index: usize,
) -> Result<(), StagexCoreutilsError> {
    let output = output_root.join(utility.name);
    let objects = compile_utility_sources(request, compiler, source_root, utility, index)?;
    let libdir = request.tinycc27_root.join("lib/mes");
    let runtime = libdir.join("tcc");
    let mut args = vec![
        "-static".to_string(),
        "-nostdlib".to_string(),
        "-B".to_string(),
        crate::stagex_mes_lib::utf8_absolute(&runtime, "TinyCC runtime")?.to_string(),
    ];
    args.extend(compiler_prefix(request.tinycc27_root)?);
    args.extend(LINK_FLAGS.iter().map(|flag| (*flag).to_string()));
    args.extend([
        "-o".to_string(),
        crate::stagex_mes_lib::utf8_absolute(&output, "coreutils output")?.to_string(),
    ]);
    args.extend(objects.iter().cloned());
    args.extend([
        "-L.".to_string(),
        "-lcu".to_string(),
        crate::stagex_mes_lib::utf8_absolute(&libdir.join("crt1.o"), "TinyCC crt1")?.to_string(),
        crate::stagex_mes_lib::utf8_absolute(&libdir.join("libc.a"), "TinyCC libc")?.to_string(),
        crate::stagex_mes_lib::utf8_absolute(&runtime.join("libtcc1.a"), "TinyCC libtcc1")?.to_string(),
    ]);
    crate::stagex_mes_lib::run_bounded_process(
        compiler,
        &args,
        source_root,
        &BTreeMap::<String, String>::new(),
        &request.scratch_dir.join(format!("coreutils-link-{index:02}-{}.stderr.txt", utility.name)),
    )?;
    crate::stagex_tinycc::set_owner_executable(&output)?;
    validate_nonempty_file(&output, utility.name)?;
    assert_eq!(objects.len(), utility.sources.len());
    assert!(output.is_file());
    Ok(())
}

fn compile_utility_sources(
    request: &CoreutilsInventoryRequest<'_>,
    compiler: &Path,
    source_root: &Path,
    utility: &UtilitySpec,
    utility_index: usize,
) -> Result<Vec<String>, StagexCoreutilsError> {
    let mut objects = Vec::with_capacity(utility.sources.len());
    for (source_index, source) in utility.sources.iter().enumerate() {
        let object = format!("utility-{utility_index:02}-{source_index:02}.o");
        let mut args = compiler_prefix(request.tinycc27_root)?;
        args.extend(COMMON_FLAGS.iter().map(|flag| (*flag).to_string()));
        args.extend([(*source).to_string(), "-o".to_string(), object.clone()]);
        crate::stagex_mes_lib::run_bounded_process(
            compiler,
            &args,
            source_root,
            &BTreeMap::<String, String>::new(),
            &request
                .scratch_dir
                .join(format!("coreutils-utility-compile-{utility_index:02}-{source_index:02}.stderr.txt")),
        )?;
        validate_nonempty_file(&source_root.join(&object), &object)?;
        objects.push(object);
    }
    assert_eq!(objects.len(), utility.sources.len());
    assert!(!objects.is_empty());
    Ok(objects)
}

fn compiler_prefix(tinycc27_root: &Path) -> Result<Vec<String>, StagexCoreutilsError> {
    Ok(vec![
        "-I".to_string(),
        crate::stagex_mes_lib::utf8_absolute(&tinycc27_root.join("include/mes"), "TinyCC include")?.to_string(),
    ])
}

fn run_coreutils_smokes(
    request: &CoreutilsInventoryRequest<'_>,
    source_root: &Path,
    output_root: &Path,
) -> Result<(), StagexCoreutilsError> {
    let smoke_root = source_root.join("smoke");
    fs::create_dir(&smoke_root)
        .map_err(|error| StagexCoreutilsError::Materialization(format!("creating coreutils smoke root: {error}")))?;
    crate::stagex_mes_lib::write_create_new(&smoke_root.join("input"), COREUTILS_SMOKE_INPUT)?;
    let empty_env = BTreeMap::<String, String>::new();
    let commands: [(&str, &[&str]); 5] = [
        ("mkdir", &["smoke/dir"]),
        ("echo", &["coreutils-smoke"]),
        ("cp", &["smoke/input", "smoke/dir/output"]),
        ("cat", &["smoke/dir/output"]),
        ("test", &["-f", "smoke/dir/output"]),
    ];
    for (index, (name, args)) in commands.iter().enumerate() {
        crate::stagex_mes_lib::run_bounded_process(
            &output_root.join(name),
            args,
            source_root,
            &empty_env,
            &request.scratch_dir.join(format!("coreutils-smoke-{index:02}-{name}.stderr.txt")),
        )?;
    }
    require_expected_process_failure(
        &output_root.join("cp"),
        &["smoke/missing", "smoke/dir/rejected"],
        source_root,
        &request.scratch_dir.join("coreutils-negative-cp.stderr.txt"),
    )?;
    let output = fs::read(smoke_root.join("dir/output"))
        .map_err(|error| StagexCoreutilsError::Materialization(format!("reading coreutils smoke output: {error}")))?;
    if output != COREUTILS_SMOKE_INPUT || smoke_root.join("dir/rejected").exists() {
        return Err(StagexCoreutilsError::Materialization(
            "coreutils positive or negative smoke observation mismatch".to_string(),
        ));
    }
    assert_eq!(commands.len() + 1, usize::try_from(COREUTILS_SMOKE_COMMAND_COUNT).unwrap());
    assert_eq!(output, COREUTILS_SMOKE_INPUT);
    Ok(())
}

fn require_expected_process_failure(
    executable: &Path,
    args: &[&str],
    current_dir: &Path,
    stderr_path: &Path,
) -> Result<(), StagexCoreutilsError> {
    match crate::stagex_mes_lib::run_bounded_process(
        executable,
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
        Ok(()) => Err(StagexCoreutilsError::Materialization("coreutils cp accepted a missing source".to_string())),
        Err(error) => Err(StagexCoreutilsError::Runtime(error)),
    }
}

fn collect_outputs(source_root: &Path, output_root: &Path) -> Result<Vec<CoreutilsOutputReport>, StagexCoreutilsError> {
    let mut outputs = Vec::with_capacity(COREUTILS_OUTPUT_COUNT);
    for utility in UTILITY_SPECS {
        push_output(&mut outputs, &format!("coreutils-{}", utility.name), &output_root.join(utility.name))?;
    }
    push_output(&mut outputs, "coreutils-bracket", &output_root.join("["))?;
    push_output(&mut outputs, "coreutils-smoke", &source_root.join("smoke/dir/output"))?;
    assert_eq!(outputs.len(), COREUTILS_OUTPUT_COUNT);
    assert!(outputs.iter().all(|output| output.bytes_len > 0));
    Ok(outputs)
}

fn push_output(
    outputs: &mut Vec<CoreutilsOutputReport>,
    artifact_id: &str,
    path: &Path,
) -> Result<(), StagexCoreutilsError> {
    let metadata = fs::metadata(path)
        .map_err(|error| StagexCoreutilsError::Materialization(format!("reading {artifact_id} metadata: {error}")))?;
    outputs.push(CoreutilsOutputReport {
        artifact_id: artifact_id.to_string(),
        path: path.to_path_buf(),
        bytes_len: metadata.len(),
        digest_blake3: blake3_file_hex(path, artifact_id)?,
    });
    assert!(metadata.is_file());
    assert!(metadata.len() > 0);
    Ok(())
}

fn validate_expected_outputs(outputs: &[CoreutilsOutputReport]) -> Result<(), StagexCoreutilsError> {
    if outputs.len() != COREUTILS_EXPECTED_OUTPUTS.len() {
        return Err(StagexCoreutilsError::Materialization(format!(
            "coreutils output count mismatch: expected {}, observed {}",
            COREUTILS_EXPECTED_OUTPUTS.len(),
            outputs.len()
        )));
    }
    let mut mismatches = Vec::new();
    for expected in COREUTILS_EXPECTED_OUTPUTS {
        let observed = outputs.iter().find(|output| output.artifact_id == expected.artifact_id).ok_or_else(|| {
            StagexCoreutilsError::Materialization(format!("coreutils output {} is missing", expected.artifact_id))
        })?;
        if observed.digest_blake3 != expected.digest_blake3 {
            mismatches.push(format!("{}={}", expected.artifact_id, observed.digest_blake3));
        }
    }
    if !mismatches.is_empty() {
        return Err(StagexCoreutilsError::Materialization(format!(
            "coreutils output BLAKE3 mismatches: {}",
            mismatches.join(",")
        )));
    }
    assert_eq!(outputs.len(), COREUTILS_EXPECTED_OUTPUTS.len());
    assert!(outputs.iter().all(|output| output.digest_blake3.len() == blake3::OUT_LEN * 2));
    Ok(())
}

fn find_source_record(records: &[SourceRecord]) -> Result<&SourceRecord, StagexCoreutilsError> {
    records
        .iter()
        .find(|record| record.metadata.get(SOURCE_RECORD_NAME_KEY).map(String::as_str) == Some(COREUTILS_RECORD_NAME))
        .ok_or(StagexCoreutilsError::SourceRecordNotFound)
}

fn validate_source_record(record: &SourceRecord) -> Result<(), StagexCoreutilsError> {
    if record.kind != SourceRecordKind::FixedUrl || record.content_blake3 != COREUTILS_SOURCE_CONTENT_BLAKE3 {
        return Err(StagexCoreutilsError::Materialization(format!(
            "coreutils source record {} has substituted kind or content identity",
            record.identity
        )));
    }
    if record.files.len() != 1 || record.files.iter().any(|entry| entry.file_type != SourceFileType::Regular) {
        return Err(StagexCoreutilsError::Materialization(
            "coreutils source record must contain exactly one archive payload".to_string(),
        ));
    }
    assert_eq!(record.kind, SourceRecordKind::FixedUrl);
    assert_eq!(record.content_blake3, COREUTILS_SOURCE_CONTENT_BLAKE3);
    Ok(())
}

fn configured_source_digest_blake3() -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"mantle-stagex-coreutils-configured-source-v1\0");
    hasher.update(COREUTILS_SOURCE_CONTENT_BLAKE3.as_bytes());
    hasher.update(b"\0");
    hasher.update(COREUTILS_CONFIG_SOURCE);
    hasher.update(b"separate-utility-object-compilation-v1\0");
    for patch in COREUTILS_PATCHES {
        hasher.update(patch.bytes);
        hasher.update(b"\0");
    }
    for value in COMMON_FLAGS.iter().chain(LINK_FLAGS.iter()).chain(LIBRARY_SOURCES.iter()).chain(
        UTILITY_SPECS
            .iter()
            .flat_map(|utility| std::iter::once(&utility.name).chain(utility.sources.iter())),
    ) {
        hasher.update(value.as_bytes());
        hasher.update(b"\0");
    }
    hasher.finalize().to_hex().to_string()
}

fn validate_bound_bytes(bytes: &[u8], expected: &str, label: &str) -> Result<(), StagexCoreutilsError> {
    let observed = blake3::hash(bytes).to_hex().to_string();
    if observed != expected {
        return Err(StagexCoreutilsError::Materialization(format!(
            "{label} BLAKE3 mismatch: expected {expected}, observed {observed}"
        )));
    }
    assert!(!bytes.is_empty());
    assert_eq!(observed.len(), blake3::OUT_LEN * 2);
    Ok(())
}

fn validate_file_digest(path: &Path, expected: &str, label: &str) -> Result<(), StagexCoreutilsError> {
    let observed = blake3_file_hex(path, label)?;
    if observed != expected {
        return Err(StagexCoreutilsError::Materialization(format!(
            "{label} BLAKE3 mismatch: expected {expected}, observed {observed}"
        )));
    }
    assert!(path.is_file());
    assert_eq!(observed.len(), blake3::OUT_LEN * 2);
    Ok(())
}

fn blake3_file_hex(path: &Path, label: &str) -> Result<String, StagexCoreutilsError> {
    let metadata = fs::metadata(path)
        .map_err(|error| StagexCoreutilsError::Materialization(format!("reading {label} metadata: {error}")))?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > COREUTILS_FILE_BYTES_MAX {
        return Err(StagexCoreutilsError::Materialization(format!(
            "{label} is not a bounded non-empty file: {} bytes",
            metadata.len()
        )));
    }
    let bytes =
        fs::read(path).map_err(|error| StagexCoreutilsError::Materialization(format!("reading {label}: {error}")))?;
    assert_eq!(u64::try_from(bytes.len()).unwrap(), metadata.len());
    assert!(!bytes.is_empty());
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

fn validate_nonempty_file(path: &Path, label: &str) -> Result<(), StagexCoreutilsError> {
    let metadata = fs::metadata(path)
        .map_err(|error| StagexCoreutilsError::Materialization(format!("reading {label} metadata: {error}")))?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > COREUTILS_FILE_BYTES_MAX {
        return Err(StagexCoreutilsError::Materialization(format!(
            "{label} is not a bounded non-empty file: {} bytes",
            metadata.len()
        )));
    }
    assert!(path.is_file());
    assert!(metadata.len() <= COREUTILS_FILE_BYTES_MAX);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const RETAINED_SOURCE_BUNDLE_ENV: &str = "MANTLE_STAGE_X_SOURCE_BUNDLE";
    const RETAINED_SOURCE_ROOT_ENV: &str = "MANTLE_STAGE_X_COREUTILS_SOURCE_ROOT";
    const RETAINED_TINYCC27_ROOT_ENV: &str = "MANTLE_STAGE_X_TINYCC27_ROOT";
    const RETAINED_PATCH_ROOT_ENV: &str = "MANTLE_STAGE_X_PATCH_ROOT";
    const RETAINED_BUILD_SCRATCH_ENV: &str = "MANTLE_STAGE_X_COREUTILS_BUILD_SCRATCH";

    #[test]
    fn configured_source_digest_is_stable() {
        let digest = configured_source_digest_blake3();
        assert_eq!(digest.len(), blake3::OUT_LEN * 2);
        assert!(!digest.is_empty());
        assert_eq!(digest, COREUTILS_CONFIGURED_SOURCE_BLAKE3);
    }

    #[test]
    fn bounded_utility_set_has_unique_names_and_sources() {
        let names = UTILITY_SPECS.iter().map(|utility| utility.name).collect::<std::collections::BTreeSet<_>>();
        assert_eq!(names.len(), UTILITY_SPECS.len());
        assert!(UTILITY_SPECS.iter().all(|utility| !utility.sources.is_empty()));
        assert_eq!(UTILITY_SPECS.len(), usize::try_from(COREUTILS_UTILITY_LINK_COUNT).unwrap());
        let source_count = UTILITY_SPECS.iter().map(|utility| utility.sources.len()).sum::<usize>();
        assert_eq!(source_count, usize::try_from(COREUTILS_UTILITY_SOURCE_COMPILE_COUNT).unwrap());
    }

    #[test]
    fn source_record_validation_rejects_substitution() {
        let record = SourceRecord {
            kind: SourceRecordKind::FixedUrl,
            identity: "fixed-url-test".to_string(),
            store_prefix: Some("/mantle/store".to_string()),
            adapter: None,
            metadata: BTreeMap::from([(SOURCE_RECORD_NAME_KEY.to_string(), COREUTILS_RECORD_NAME.to_string())]),
            payload_bytes: 1,
            content_blake3: COREUTILS_SOURCE_CONTENT_BLAKE3.to_string(),
            files: vec![crate::source_bundle::SourceFileEntry {
                path: "payload".to_string(),
                file_type: SourceFileType::Regular,
                executable: false,
                size: 1,
                content_hex: Some("00".to_string()),
                symlink_target: None,
                chunk_index: None,
                chunk_count: None,
                blake3: "a".repeat(blake3::OUT_LEN * 2),
            }],
        };
        validate_source_record(&record).unwrap();
        let mut substituted = record;
        substituted.content_blake3 = "b".repeat(blake3::OUT_LEN * 2);
        assert!(validate_source_record(&substituted).unwrap_err().to_string().contains("substituted"));
    }

    #[test]
    #[ignore = "requires retained authenticated source bundle"]
    fn materializes_retained_coreutils_source() {
        let bundle = PathBuf::from(std::env::var(RETAINED_SOURCE_BUNDLE_ENV).unwrap());
        let scratch = PathBuf::from(std::env::var(RETAINED_SOURCE_ROOT_ENV).unwrap());
        let expected_manifest = read_source_bundle(&bundle).unwrap().manifest_blake3;
        let report = materialize_authenticated_coreutils_source(&bundle, &expected_manifest, &scratch).unwrap();
        assert_eq!(report.record_content_blake3, COREUTILS_SOURCE_CONTENT_BLAKE3);
        assert!(report.output_path.join("src/cat.c").is_file());
    }

    #[test]
    #[ignore = "requires retained coreutils source and TinyCC 0.9.27"]
    fn derives_retained_coreutils_inventory() {
        let source_root = PathBuf::from(std::env::var(RETAINED_SOURCE_ROOT_ENV).unwrap());
        let tinycc27_root = PathBuf::from(std::env::var(RETAINED_TINYCC27_ROOT_ENV).unwrap());
        let patch_root = PathBuf::from(std::env::var(RETAINED_PATCH_ROOT_ENV).unwrap());
        let scratch_dir = PathBuf::from(std::env::var(RETAINED_BUILD_SCRATCH_ENV).unwrap());
        let report = derive_coreutils_inventory(CoreutilsInventoryRequest {
            source_root: &source_root,
            tinycc27_root: &tinycc27_root,
            patch_root: &patch_root,
            scratch_dir: &scratch_dir,
            protected_exec_enforced: false,
        })
        .unwrap();
        assert_eq!(report.build_command_count, COREUTILS_BUILD_COMMAND_COUNT);
        assert!(report.fallback_events.is_empty());
    }
}
