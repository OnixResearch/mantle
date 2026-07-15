use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::ArtifactOutputClass;
use crate::Blake3Digest;
use crate::CompilationPlan;
use crate::ExperimentBlocker;
use crate::ExperimentProfile;
use crate::KernelArchitecture;
use crate::blocker::blocker;
use crate::digest::canonical_identity;
use crate::digest::count_above_bound;

pub const ELF_CLASS_64: u8 = 2;
pub const ELF_TYPE_RELOCATABLE: u16 = 1;
pub const ELF_TYPE_EXECUTABLE: u16 = 2;
pub const ELF_TYPE_DYNAMIC: u16 = 3;
pub const ELF_MACHINE_X86_64: u16 = 62;
pub const ELF_MACHINE_AARCH64: u16 = 183;
pub const ELF_MACHINE_BPF: u16 = 247;

const ELF_MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];
const ELF_DATA_LITTLE_ENDIAN: u8 = 1;
const ELF_IDENT_VERSION_CURRENT: u8 = 1;
const ELF64_HEADER_BYTES: usize = 64;
const ELF64_SECTION_HEADER_BYTES: usize = 64;
const ELF_CLASS_OFFSET: usize = 4;
const ELF_DATA_OFFSET: usize = 5;
const ELF_IDENT_VERSION_OFFSET: usize = 6;
const ELF_TYPE_OFFSET: usize = 16;
const ELF_MACHINE_OFFSET: usize = 18;
const ELF_SECTION_TABLE_OFFSET: usize = 40;
const ELF_SECTION_ENTRY_SIZE_OFFSET: usize = 58;
const ELF_SECTION_COUNT_OFFSET: usize = 60;
const ELF_SECTION_NAMES_INDEX_OFFSET: usize = 62;
const SECTION_NAME_OFFSET: usize = 0;
const SECTION_FILE_OFFSET: usize = 24;
const SECTION_SIZE_OFFSET: usize = 32;
const BTF_SECTION_NAME: &str = ".BTF";
const MODULE_INFO_SECTION_NAME: &str = ".modinfo";
const BTF_MAGIC: [u8; 2] = [0x9f, 0xeb];
const BTF_VERSION_OFFSET: usize = 2;
const BTF_FLAGS_OFFSET: usize = 3;
const BTF_HEADER_LENGTH_OFFSET: usize = 4;
const BTF_TYPE_OFFSET_OFFSET: usize = 8;
const BTF_TYPE_LENGTH_OFFSET: usize = 12;
const BTF_STRING_OFFSET_OFFSET: usize = 16;
const BTF_STRING_LENGTH_OFFSET: usize = 20;
const BTF_HEADER_BYTES_MIN: usize = 24;
const BTF_VERSION_CURRENT: u8 = 1;
const BTF_FLAGS_NONE: u8 = 0;
const BTF_STRING_TABLE_INITIAL_BYTE: u8 = 0;
const MODULE_VERMAGIC_PREFIX: &str = "vermagic=";

const _: () = {
    assert!(ELF_MAGIC.len() <= ELF64_HEADER_BYTES, "ELF magic must fit in the ELF64 header");
    assert!(
        matches!(
            ELF_MACHINE_OFFSET.checked_add(core::mem::size_of::<u16>()),
            Some(end_bytes) if end_bytes <= ELF64_HEADER_BYTES
        ),
        "ELF machine field must fit in the ELF64 header"
    );
    assert!(
        matches!(
            ELF_SECTION_NAMES_INDEX_OFFSET.checked_add(core::mem::size_of::<u16>()),
            Some(end_bytes) if end_bytes <= ELF64_HEADER_BYTES
        ),
        "ELF section-name index must fit in the ELF64 header"
    );
    assert!(
        matches!(
            SECTION_SIZE_OFFSET.checked_add(core::mem::size_of::<u64>()),
            Some(end_bytes) if end_bytes <= ELF64_SECTION_HEADER_BYTES
        ),
        "ELF section size must fit in an ELF64 section header"
    );
    assert!(
        matches!(
            BTF_STRING_LENGTH_OFFSET.checked_add(core::mem::size_of::<u32>()),
            Some(end_bytes) if end_bytes <= BTF_HEADER_BYTES_MIN
        ),
        "BTF string length must fit in the minimum BTF header"
    );
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedOutput {
    pub output_class: ArtifactOutputClass,
    pub relative_path: String,
    pub bytes: Vec<u8>,
    pub plan_identity_blake3: Blake3Digest,
    pub target_kernel_build_identity: String,
    pub target_architecture: KernelArchitecture,
    pub target_kernel_release: String,
    pub generated_source_members: Vec<Blake3Digest>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutputInspection {
    pub output_class: ArtifactOutputClass,
    pub relative_path: String,
    pub digest_blake3: Blake3Digest,
    pub size_bytes: u64,
    pub elf_class: u8,
    pub elf_type: u16,
    pub elf_machine: u16,
    pub section_names: Vec<String>,
    pub plan_identity_blake3: Blake3Digest,
    pub target_kernel_build_identity: String,
    pub target_architecture: KernelArchitecture,
    pub target_kernel_release: String,
    pub generated_source_members: Vec<Blake3Digest>,
    pub accepted: bool,
    pub inspection_identity_blake3: Blake3Digest,
}

#[derive(Serialize)]
struct InspectionIdentityInput {
    output_class: ArtifactOutputClass,
    relative_path: String,
    digest_blake3: Blake3Digest,
    size_bytes: u64,
    elf_class: u8,
    elf_type: u16,
    elf_machine: u16,
    section_names: Vec<String>,
    plan_identity_blake3: Blake3Digest,
    target_kernel_build_identity: String,
    target_architecture: KernelArchitecture,
    target_kernel_release: String,
    generated_source_members: Vec<Blake3Digest>,
    accepted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InspectionResult {
    pub inspection: Option<OutputInspection>,
    pub blockers: Vec<ExperimentBlocker>,
}

struct ElfSectionFacts {
    name: String,
    file_offset_bytes: usize,
    size_bytes: usize,
}

struct ElfFacts {
    elf_class: u8,
    elf_type: u16,
    elf_machine: u16,
    sections: Vec<ElfSectionFacts>,
}

#[derive(Debug, Clone, Copy)]
struct ElfEncoding {
    class: u8,
    data: u8,
    version: u8,
}

#[derive(Debug, Clone, Copy)]
struct SectionTableShape {
    entry_size_bytes: usize,
    section_count: usize,
    names_index: usize,
    maximum_section_count: usize,
}

#[derive(Debug, Clone, Copy)]
struct SectionTableLayout {
    table_offset_bytes: usize,
    entry_size_bytes: usize,
    section_count: usize,
    names_offset_bytes: usize,
    names_end_bytes: usize,
}

#[derive(Debug, Clone, Copy)]
struct ByteRange {
    start_bytes: usize,
    end_bytes: usize,
}

struct ModuleElfValidation<'a> {
    bytes: &'a [u8],
    expected_machine: u16,
    expected_release: &'a str,
    max_text_bytes: u32,
}

#[derive(Debug, Clone, Copy)]
struct BtfRange {
    offset_bytes: u32,
    length_bytes: u32,
}

pub fn inspect_output(profile: ExperimentProfile, plan: CompilationPlan, observed: ObservedOutput) -> InspectionResult {
    let mut blockers = crate::validate_profile(profile.clone()).blockers;
    validate_observation_binding(&profile, &plan, &observed, &mut blockers);
    let elf = parse_elf(&observed.bytes, &profile, &mut blockers);
    if let Some(elf) = &elf {
        validate_elf_class(&profile, &observed, elf, &mut blockers);
    }
    let inspection = build_inspection(observed, elf, &mut blockers);
    debug_assert!(blockers.is_empty() == inspection.is_some());
    debug_assert!(blockers.iter().all(|item| !item.code.is_empty()));
    InspectionResult { inspection, blockers }
}

fn validate_observation_binding(
    profile: &ExperimentProfile,
    plan: &CompilationPlan,
    observed: &ObservedOutput,
    blockers: &mut Vec<ExperimentBlocker>,
) {
    let initial_blocker_count: usize = blockers.len();
    let is_plan_identity_intact =
        crate::planner::expected_compilation_plan_identity(plan).as_ref() == Some(&plan.plan_identity_blake3);
    if observed.plan_identity_blake3 != plan.plan_identity_blake3 || !is_plan_identity_intact {
        blockers.push(blocker(
            "stale-output-plan",
            &observed.relative_path,
            "output was not produced by an intact inspected compilation plan",
        ));
    }
    if observed.target_kernel_build_identity != profile.target.kernel_build_identity
        || observed.target_architecture != profile.target.architecture
        || observed.target_kernel_release != profile.target.kernel_release
    {
        blockers.push(blocker(
            "stale-output-target",
            &observed.relative_path,
            "output target binding differs from the profile",
        ));
    }
    if !profile.output_classes.contains(&observed.output_class) {
        blockers.push(blocker(
            "unselected-output-class",
            &observed.relative_path,
            "output class was not selected by the profile",
        ));
    }
    if !plan_contains_output(plan, observed.output_class, &observed.relative_path) {
        blockers.push(blocker(
            "unplanned-output-path",
            &observed.relative_path,
            "output path is absent from the explicit compilation plan",
        ));
    }
    let expected_sources = expected_generated_source_members(plan, observed.output_class, &observed.relative_path);
    let mut actual_sources = observed.generated_source_members.clone();
    actual_sources.sort();
    if expected_sources.is_empty() || actual_sources != expected_sources {
        blockers.push(blocker(
            "output-source-binding",
            &observed.relative_path,
            "output does not bind the exact generated source identities consumed by its plan step",
        ));
    }
    validate_output_shape(profile, observed, blockers);
    debug_assert!(blockers.len() >= initial_blocker_count);
    debug_assert!(blockers.iter().all(|item| !item.code.is_empty()));
}

fn validate_output_shape(
    profile: &ExperimentProfile,
    observed: &ObservedOutput,
    blockers: &mut Vec<ExperimentBlocker>,
) {
    let initial_blocker_count: usize = blockers.len();
    let size_bytes = match u64::try_from(observed.bytes.len()) {
        Ok(size_bytes) => size_bytes,
        Err(_) => {
            blockers.push(blocker(
                "output-byte-bound",
                &observed.relative_path,
                "output byte count is not representable",
            ));
            return;
        }
    };
    if size_bytes == 0 || size_bytes > profile.bounds.max_output_bytes {
        blockers.push(blocker(
            "output-byte-bound",
            &observed.relative_path,
            "output is empty or exceeds the profile byte bound",
        ));
    }
    if !safe_relative_path(&observed.relative_path) {
        blockers.push(blocker(
            "output-path-escape",
            &observed.relative_path,
            "output path is absolute, escaping, or non-canonical",
        ));
    }
    let has_source_members = !observed.generated_source_members.is_empty();
    let is_source_count_bounded =
        !count_above_bound(observed.generated_source_members.len(), profile.bounds.max_generated_files);
    if !has_source_members || !is_source_count_bounded {
        blockers.push(blocker(
            "output-source-binding",
            &observed.relative_path,
            "output must bind a bounded non-empty generated source set",
        ));
    }
    debug_assert!(blockers.len() >= initial_blocker_count);
    debug_assert!(blockers.iter().all(|item| !item.code.is_empty()));
}

fn plan_contains_output(plan: &CompilationPlan, class: ArtifactOutputClass, path: &str) -> bool {
    plan.steps
        .iter()
        .any(|step| step.output_class == class && step.outputs.iter().any(|output| output == path))
}

pub(crate) fn expected_generated_source_members(
    plan: &CompilationPlan,
    class: ArtifactOutputClass,
    path: &str,
) -> Vec<Blake3Digest> {
    let Some(step) = plan
        .steps
        .iter()
        .find(|step| step.output_class == class && step.outputs.iter().any(|output| output == path))
    else {
        return Vec::new();
    };
    let mut members = plan
        .generated_members
        .iter()
        .filter(|member| step.inputs.iter().any(|input| input == &member.relative_path))
        .map(|member| member.digest_blake3.clone())
        .collect::<Vec<_>>();
    members.sort();
    members
}

fn parse_elf(bytes: &[u8], profile: &ExperimentProfile, blockers: &mut Vec<ExperimentBlocker>) -> Option<ElfFacts> {
    let initial_blocker_count: usize = blockers.len();
    debug_assert!(blockers.iter().all(|item| !item.code.is_empty()));
    if !has_complete_elf_header(bytes) {
        blockers.push(blocker("malformed-elf-header", "output", "output is not a complete ELF object"));
        return None;
    }
    let elf_class: u8 = bytes[ELF_CLASS_OFFSET];
    let elf_data: u8 = bytes[ELF_DATA_OFFSET];
    let elf_version: u8 = bytes[ELF_IDENT_VERSION_OFFSET];
    if !has_supported_elf_encoding(ElfEncoding {
        class: elf_class,
        data: elf_data,
        version: elf_version,
    }) {
        blockers.push(blocker(
            "unsupported-elf-encoding",
            "output",
            "only ELF64 little-endian current-version objects are admitted",
        ));
        return None;
    }
    let Some(elf_type) = read_u16(bytes, ELF_TYPE_OFFSET) else {
        blockers.push(blocker("malformed-elf-header", "output", "ELF type is truncated"));
        return None;
    };
    let Some(elf_machine) = read_u16(bytes, ELF_MACHINE_OFFSET) else {
        blockers.push(blocker("malformed-elf-header", "output", "ELF machine is truncated"));
        return None;
    };
    let section_blocker_count: usize = blockers.len();
    let Some(sections) = parse_sections(bytes, profile, blockers) else {
        if blockers.len() == section_blocker_count {
            blockers.push(blocker(
                "malformed-elf-section-table",
                "output",
                "ELF section metadata is truncated or overflows",
            ));
        }
        return None;
    };
    debug_assert!(blockers.len() >= initial_blocker_count);
    Some(ElfFacts {
        elf_class,
        elf_type,
        elf_machine,
        sections,
    })
}

fn has_complete_elf_header(bytes: &[u8]) -> bool {
    bytes.len() >= ELF64_HEADER_BYTES && bytes.get(0..ELF_MAGIC.len()) == Some(ELF_MAGIC.as_slice())
}

fn has_supported_elf_encoding(encoding: ElfEncoding) -> bool {
    encoding.class == ELF_CLASS_64
        && encoding.data == ELF_DATA_LITTLE_ENDIAN
        && encoding.version == ELF_IDENT_VERSION_CURRENT
}

fn parse_sections(
    bytes: &[u8],
    profile: &ExperimentProfile,
    blockers: &mut Vec<ExperimentBlocker>,
) -> Option<Vec<ElfSectionFacts>> {
    debug_assert!(bytes.len() >= ELF64_HEADER_BYTES);
    debug_assert!(blockers.iter().all(|item| !item.code.is_empty()));
    let table_offset_bytes = usize_from_u64(read_u64(bytes, ELF_SECTION_TABLE_OFFSET)?)?;
    let entry_size_bytes = usize::from(read_u16(bytes, ELF_SECTION_ENTRY_SIZE_OFFSET)?);
    let section_count = usize::from(read_u16(bytes, ELF_SECTION_COUNT_OFFSET)?);
    let names_index = usize::from(read_u16(bytes, ELF_SECTION_NAMES_INDEX_OFFSET)?);
    let maximum_section_count = usize::try_from(profile.bounds.max_elf_sections).ok()?;
    if !has_valid_section_table_shape(SectionTableShape {
        entry_size_bytes,
        section_count,
        names_index,
        maximum_section_count,
    }) {
        blockers.push(blocker(
            "invalid-elf-section-table",
            "output",
            "ELF section table shape exceeds the admitted profile",
        ));
        return None;
    }
    let table_size_bytes = entry_size_bytes.checked_mul(section_count)?;
    let table_end_bytes = table_offset_bytes.checked_add(table_size_bytes)?;
    if table_end_bytes > bytes.len() {
        blockers.push(blocker("truncated-elf-section-table", "output", "ELF section table exceeds output bytes"));
        return None;
    }
    let names_header_offset_bytes = table_offset_bytes.checked_add(entry_size_bytes.checked_mul(names_index)?)?;
    let names_offset_bytes =
        usize_from_u64(read_u64(bytes, names_header_offset_bytes.checked_add(SECTION_FILE_OFFSET)?)?)?;
    let names_size_bytes =
        usize_from_u64(read_u64(bytes, names_header_offset_bytes.checked_add(SECTION_SIZE_OFFSET)?)?)?;
    let names_end_bytes = names_offset_bytes.checked_add(names_size_bytes)?;
    if names_end_bytes > bytes.len() {
        blockers.push(blocker(
            "truncated-elf-string-table",
            "output",
            "ELF section-name string table exceeds output bytes",
        ));
        return None;
    }
    collect_sections(
        bytes,
        SectionTableLayout {
            table_offset_bytes,
            entry_size_bytes,
            section_count,
            names_offset_bytes,
            names_end_bytes,
        },
        profile,
        blockers,
    )
}

fn has_valid_section_table_shape(shape: SectionTableShape) -> bool {
    let has_expected_entry_size_bytes = shape.entry_size_bytes == ELF64_SECTION_HEADER_BYTES;
    let has_sections = shape.section_count > 0;
    let is_section_count_bounded = shape.section_count <= shape.maximum_section_count;
    let is_names_index_bounded = shape.names_index < shape.section_count;
    has_expected_entry_size_bytes && has_sections && is_section_count_bounded && is_names_index_bounded
}

fn collect_sections(
    bytes: &[u8],
    layout: SectionTableLayout,
    profile: &ExperimentProfile,
    blockers: &mut Vec<ExperimentBlocker>,
) -> Option<Vec<ElfSectionFacts>> {
    let initial_blocker_count: usize = blockers.len();
    let mut names = BTreeSet::new();
    let mut sections = Vec::with_capacity(layout.section_count);
    for index in 0..layout.section_count {
        let header_offset_bytes = layout.table_offset_bytes.checked_add(layout.entry_size_bytes.checked_mul(index)?)?;
        let name_index =
            usize::try_from(read_u32(bytes, header_offset_bytes.checked_add(SECTION_NAME_OFFSET)?)?).ok()?;
        if name_index == 0 {
            continue;
        }
        let name_start_bytes = layout.names_offset_bytes.checked_add(name_index)?;
        let name = section_name(
            bytes,
            ByteRange {
                start_bytes: name_start_bytes,
                end_bytes: layout.names_end_bytes,
            },
            profile.bounds.max_section_name_bytes,
        )?;
        if !names.insert(name.clone()) {
            blockers.push(blocker("duplicate-elf-section-name", "output", "ELF section names contain duplicates"));
            return None;
        }
        let file_offset_bytes =
            usize_from_u64(read_u64(bytes, header_offset_bytes.checked_add(SECTION_FILE_OFFSET)?)?)?;
        let size_bytes = usize_from_u64(read_u64(bytes, header_offset_bytes.checked_add(SECTION_SIZE_OFFSET)?)?)?;
        let section_end_bytes = file_offset_bytes.checked_add(size_bytes)?;
        if section_end_bytes > bytes.len() {
            blockers.push(blocker("truncated-elf-section", &name, "ELF section exceeds output bytes"));
            return None;
        }
        sections.push(ElfSectionFacts {
            name,
            file_offset_bytes,
            size_bytes,
        });
    }
    sections.sort_by(|left, right| left.name.cmp(&right.name));
    debug_assert!(sections.len() <= layout.section_count);
    debug_assert!(blockers.len() >= initial_blocker_count);
    Some(sections)
}

fn section_name(bytes: &[u8], range: ByteRange, max_name_bytes: u32) -> Option<String> {
    if range.start_bytes >= range.end_bytes {
        return None;
    }
    let maximum_name_bytes = usize::try_from(max_name_bytes).ok()?;
    let bounded_end_bytes = range.start_bytes.checked_add(maximum_name_bytes)?.min(range.end_bytes);
    let relative_end_bytes = bytes.get(range.start_bytes..bounded_end_bytes)?.iter().position(|byte| *byte == 0)?;
    let name_end_bytes = range.start_bytes.checked_add(relative_end_bytes)?;
    let name = core::str::from_utf8(bytes.get(range.start_bytes..name_end_bytes)?).ok()?;
    if name.is_empty() || name.chars().any(char::is_control) {
        return None;
    }
    Some(String::from(name))
}

fn validate_elf_class(
    profile: &ExperimentProfile,
    observed: &ObservedOutput,
    elf: &ElfFacts,
    blockers: &mut Vec<ExperimentBlocker>,
) {
    let expected_machine = target_machine(profile.target.architecture);
    debug_assert_eq!(elf.elf_class, ELF_CLASS_64);
    debug_assert!(expected_machine > 0);
    match observed.output_class {
        ArtifactOutputClass::EbpfObject => validate_bpf_elf(elf, &observed.bytes, blockers),
        ArtifactOutputClass::KernelModule => validate_module_elf(
            elf,
            ModuleElfValidation {
                bytes: &observed.bytes,
                expected_machine,
                expected_release: &profile.target.kernel_release,
                max_text_bytes: profile.bounds.max_text_bytes,
            },
            blockers,
        ),
        ArtifactOutputClass::UserspaceLoader | ArtifactOutputClass::TestBinary => {
            validate_executable_elf(elf, expected_machine, blockers);
        }
        _ => blockers.push(blocker(
            "unsupported-inspection-class",
            &observed.relative_path,
            "selected output class does not have an ELF inspection profile",
        )),
    }
}

fn validate_bpf_elf(elf: &ElfFacts, bytes: &[u8], blockers: &mut Vec<ExperimentBlocker>) {
    if elf.elf_type != ELF_TYPE_RELOCATABLE || elf.elf_machine != ELF_MACHINE_BPF {
        blockers.push(blocker("invalid-bpf-elf", "output", "eBPF output must be an EM_BPF relocatable ELF object"));
    }
    let Some(section) = find_section(elf, BTF_SECTION_NAME) else {
        blockers.push(blocker("missing-bpf-btf-section", "output", "eBPF output lacks required .BTF metadata"));
        return;
    };
    let is_valid_btf = section_bytes(bytes, section).is_some_and(valid_btf_header);
    if !is_valid_btf {
        blockers.push(blocker(
            "malformed-bpf-btf-metadata",
            "output",
            "eBPF .BTF metadata is malformed or out of bounds",
        ));
    }
}

fn validate_module_elf(elf: &ElfFacts, validation: ModuleElfValidation<'_>, blockers: &mut Vec<ExperimentBlocker>) {
    let initial_blocker_count: usize = blockers.len();
    debug_assert!(validation.expected_machine > 0);
    if elf.elf_type != ELF_TYPE_RELOCATABLE || elf.elf_machine != validation.expected_machine {
        blockers.push(blocker(
            "invalid-module-elf",
            "output",
            "kernel module must be a target-machine relocatable ELF object",
        ));
    }
    let Some(section) = find_section(elf, MODULE_INFO_SECTION_NAME) else {
        blockers.push(blocker("missing-module-metadata", "output", "kernel module lacks required .modinfo metadata"));
        return;
    };
    let is_valid_metadata = section_bytes(validation.bytes, section).is_some_and(|metadata| {
        valid_module_metadata(metadata, validation.expected_release, validation.max_text_bytes)
    });
    if !is_valid_metadata {
        blockers.push(blocker(
            "malformed-module-metadata",
            "output",
            "kernel module metadata is malformed, unbounded, or names another kernel release",
        ));
    }
    debug_assert!(blockers.len() >= initial_blocker_count);
}

fn find_section<'a>(elf: &'a ElfFacts, name: &str) -> Option<&'a ElfSectionFacts> {
    elf.sections.iter().find(|section| section.name == name)
}

fn section_bytes<'a>(bytes: &'a [u8], section: &ElfSectionFacts) -> Option<&'a [u8]> {
    let end_bytes = section.file_offset_bytes.checked_add(section.size_bytes)?;
    bytes.get(section.file_offset_bytes..end_bytes)
}

fn valid_btf_header(bytes: &[u8]) -> bool {
    if bytes.len() < BTF_HEADER_BYTES_MIN || bytes.get(0..BTF_MAGIC.len()) != Some(BTF_MAGIC.as_slice()) {
        return false;
    }
    if bytes[BTF_VERSION_OFFSET] != BTF_VERSION_CURRENT || bytes[BTF_FLAGS_OFFSET] != BTF_FLAGS_NONE {
        return false;
    }
    let Some(header_length_bytes) =
        read_u32(bytes, BTF_HEADER_LENGTH_OFFSET).and_then(|value| usize::try_from(value).ok())
    else {
        return false;
    };
    if header_length_bytes < BTF_HEADER_BYTES_MIN || header_length_bytes > bytes.len() {
        return false;
    }
    valid_btf_payload(bytes, header_length_bytes)
}

fn valid_btf_payload(bytes: &[u8], header_length_bytes: usize) -> bool {
    debug_assert!(header_length_bytes >= BTF_HEADER_BYTES_MIN);
    debug_assert!(header_length_bytes <= bytes.len());
    let Some(type_offset_bytes) = read_u32(bytes, BTF_TYPE_OFFSET_OFFSET) else {
        return false;
    };
    let Some(type_length_bytes) = read_u32(bytes, BTF_TYPE_LENGTH_OFFSET) else {
        return false;
    };
    let Some(string_offset_bytes) = read_u32(bytes, BTF_STRING_OFFSET_OFFSET) else {
        return false;
    };
    let Some(string_length_bytes) = read_u32(bytes, BTF_STRING_LENGTH_OFFSET) else {
        return false;
    };
    let Some(payload_length_bytes) = bytes.len().checked_sub(header_length_bytes) else {
        return false;
    };
    let is_type_range_valid = btf_range_within_payload(
        BtfRange {
            offset_bytes: type_offset_bytes,
            length_bytes: type_length_bytes,
        },
        payload_length_bytes,
    );
    let is_string_range_valid = btf_range_within_payload(
        BtfRange {
            offset_bytes: string_offset_bytes,
            length_bytes: string_length_bytes,
        },
        payload_length_bytes,
    );
    let has_ordered_ranges = type_offset_bytes
        .checked_add(type_length_bytes)
        .is_some_and(|type_end_bytes| type_end_bytes <= string_offset_bytes);
    let string_start_bytes = usize::try_from(string_offset_bytes)
        .ok()
        .and_then(|offset_bytes| header_length_bytes.checked_add(offset_bytes));
    let has_initial_null = string_start_bytes.and_then(|offset_bytes| bytes.get(offset_bytes)).copied()
        == Some(BTF_STRING_TABLE_INITIAL_BYTE);
    is_type_range_valid && is_string_range_valid && string_length_bytes > 0 && has_ordered_ranges && has_initial_null
}

fn btf_range_within_payload(range: BtfRange, payload_length_bytes: usize) -> bool {
    let Some(end_bytes) = range.offset_bytes.checked_add(range.length_bytes) else {
        return false;
    };
    usize::try_from(end_bytes).is_ok_and(|end_bytes| end_bytes <= payload_length_bytes)
}

fn valid_module_metadata(bytes: &[u8], expected_release: &str, max_text_bytes: u32) -> bool {
    debug_assert!(!MODULE_VERMAGIC_PREFIX.is_empty());
    debug_assert!(MODULE_VERMAGIC_PREFIX.ends_with('='));
    let is_bounded =
        usize::try_from(max_text_bytes).is_ok_and(|maximum_bytes| !bytes.is_empty() && bytes.len() <= maximum_bytes);
    if !is_bounded || bytes.last().copied() != Some(0) {
        return false;
    }
    let mut has_metadata = false;
    let mut has_matching_release = false;
    for raw in bytes.split(|byte| *byte == 0).filter(|segment| !segment.is_empty()) {
        let Ok(field) = core::str::from_utf8(raw) else {
            return false;
        };
        if field.chars().any(char::is_control) {
            return false;
        }
        has_metadata = true;
        if let Some(value) = field.strip_prefix(MODULE_VERMAGIC_PREFIX) {
            has_matching_release = value.split_ascii_whitespace().next() == Some(expected_release);
        }
    }
    has_metadata && has_matching_release
}

fn validate_executable_elf(elf: &ElfFacts, expected_machine: u16, blockers: &mut Vec<ExperimentBlocker>) {
    let is_executable_type = elf.elf_type == ELF_TYPE_EXECUTABLE || elf.elf_type == ELF_TYPE_DYNAMIC;
    if !is_executable_type || elf.elf_machine != expected_machine {
        blockers.push(blocker(
            "invalid-userspace-elf",
            "output",
            "userspace/test output must be a target-machine executable ELF",
        ));
    }
}

fn build_inspection(
    observed: ObservedOutput,
    elf: Option<ElfFacts>,
    blockers: &mut Vec<ExperimentBlocker>,
) -> Option<OutputInspection> {
    debug_assert!(blockers.iter().all(|item| !item.code.is_empty()));
    debug_assert!(elf.as_ref().is_none_or(|facts| facts.elf_class == ELF_CLASS_64));
    if !blockers.is_empty() {
        return None;
    }
    let elf = elf?;
    let input = InspectionIdentityInput {
        output_class: observed.output_class,
        relative_path: observed.relative_path,
        digest_blake3: Blake3Digest::from_slice(&observed.bytes),
        size_bytes: u64::try_from(observed.bytes.len()).ok()?,
        elf_class: elf.elf_class,
        elf_type: elf.elf_type,
        elf_machine: elf.elf_machine,
        section_names: elf.sections.into_iter().map(|section| section.name).collect(),
        plan_identity_blake3: observed.plan_identity_blake3,
        target_kernel_build_identity: observed.target_kernel_build_identity,
        target_architecture: observed.target_architecture,
        target_kernel_release: observed.target_kernel_release,
        generated_source_members: observed.generated_source_members,
        accepted: true,
    };
    let inspection_identity_blake3 = match canonical_identity(&input) {
        Ok(identity) => identity,
        Err(error) => {
            blockers.push(blocker("inspection-identity-failed", "output", &error.to_string()));
            return None;
        }
    };
    Some(inspection_from_input(input, inspection_identity_blake3))
}

fn inspection_from_input(input: InspectionIdentityInput, identity: Blake3Digest) -> OutputInspection {
    OutputInspection {
        output_class: input.output_class,
        relative_path: input.relative_path,
        digest_blake3: input.digest_blake3,
        size_bytes: input.size_bytes,
        elf_class: input.elf_class,
        elf_type: input.elf_type,
        elf_machine: input.elf_machine,
        section_names: input.section_names,
        plan_identity_blake3: input.plan_identity_blake3,
        target_kernel_build_identity: input.target_kernel_build_identity,
        target_architecture: input.target_architecture,
        target_kernel_release: input.target_kernel_release,
        generated_source_members: input.generated_source_members,
        accepted: input.accepted,
        inspection_identity_blake3: identity,
    }
}

pub(crate) fn expected_inspection_identity(inspection: &OutputInspection) -> Option<Blake3Digest> {
    canonical_identity(&InspectionIdentityInput {
        output_class: inspection.output_class,
        relative_path: inspection.relative_path.clone(),
        digest_blake3: inspection.digest_blake3.clone(),
        size_bytes: inspection.size_bytes,
        elf_class: inspection.elf_class,
        elf_type: inspection.elf_type,
        elf_machine: inspection.elf_machine,
        section_names: inspection.section_names.clone(),
        plan_identity_blake3: inspection.plan_identity_blake3.clone(),
        target_kernel_build_identity: inspection.target_kernel_build_identity.clone(),
        target_architecture: inspection.target_architecture,
        target_kernel_release: inspection.target_kernel_release.clone(),
        generated_source_members: inspection.generated_source_members.clone(),
        accepted: inspection.accepted,
    })
    .ok()
}

fn target_machine(architecture: KernelArchitecture) -> u16 {
    match architecture {
        KernelArchitecture::X86_64 => ELF_MACHINE_X86_64,
        KernelArchitecture::Aarch64 => ELF_MACHINE_AARCH64,
    }
}

fn read_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    let end = offset.checked_add(core::mem::size_of::<u16>())?;
    Some(u16::from_le_bytes(bytes.get(offset..end)?.try_into().ok()?))
}

fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    let end = offset.checked_add(core::mem::size_of::<u32>())?;
    Some(u32::from_le_bytes(bytes.get(offset..end)?.try_into().ok()?))
}

fn read_u64(bytes: &[u8], offset: usize) -> Option<u64> {
    let end = offset.checked_add(core::mem::size_of::<u64>())?;
    Some(u64::from_le_bytes(bytes.get(offset..end)?.try_into().ok()?))
}

fn usize_from_u64(value: u64) -> Option<usize> {
    usize::try_from(value).ok()
}

fn safe_relative_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.contains('\\')
        && path.split('/').all(|part| !part.is_empty() && part != "." && part != "..")
}
