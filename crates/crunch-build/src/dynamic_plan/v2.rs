//! Versioned source-slice wire ABI and checked, effect-free admission.
use super::*;

// r[impl mantle.dynamic_plan_source_slices.versioned_schema]
pub const MANTLE_PLAN_V2_SCHEMA: &str = "mantle-plan-v2";
pub const MAX_PLAN_SLICES: u32 = 256;
pub const MAX_SLICE_SUBPATH_BYTES: u32 = 4096;
pub const MAX_SLICE_SUBPATH_DEPTH: u32 = 32;
pub const MAX_SLICE_ADMITTED_BYTES: u64 = 1_073_741_824;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireDynamicPlanV2 {
    pub schema: String,
    pub producer: WirePlanProducer,
    pub sources: Vec<WireSourceV2>,
    pub units: Vec<WireDynamicUnit>,
    pub roots: Vec<String>,
    pub provenance: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WireSourceV2 {
    Slice(WireSliceSource),
    StorePath(WireDeclaredSourceInput),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireSliceSource {
    pub id: String,
    pub producer_output: String,
    pub subpath: String,
    pub store_name: String,
    pub nar_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SliceSource {
    pub id: SourceId,
    pub producer_output: OutputName,
    pub subpath: String,
    pub store_name: String,
    pub nar_blake3: NarDigest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceV2 {
    Slice(SliceSource),
    StorePath(DeclaredSourceInput),
}

impl SourceV2 {
    pub fn id(&self) -> &SourceId {
        match self {
            Self::Slice(slice) => &slice.id,
            Self::StorePath(path) => &path.id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynamicPlanV2 {
    pub producer: PlanProducer,
    pub sources: Vec<SourceV2>,
    pub units: Vec<DynamicUnit>,
    pub roots: Vec<UnitId>,
    pub provenance: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalDynamicPlanV2 {
    pub plan: DynamicPlanV2,
    pub bytes: Vec<u8>,
    pub digest: PlanDigest,
}

/// Rejects host paths, backtracking, empty and dot components before any tree walk.
// r[impl mantle.dynamic_plan_source_slices.bounded_rejection]
pub fn validate_slice_subpath(value: &str) -> Result<(), DynamicPlanError> {
    if value.is_empty() || value.starts_with('/') {
        return invalid_scalar("slice subpath", value, "slice-subpath-invalid");
    }
    if value.contains('\0') || value.contains('\\') {
        return invalid_scalar("slice subpath", value, "slice-subpath-invalid");
    }
    validate_len_limit("slice subpath bytes", value.len(), MAX_SLICE_SUBPATH_BYTES)?;
    let mut depth = 0_u32;
    for component in value.split('/') {
        depth = depth.saturating_add(1);
        if depth > MAX_SLICE_SUBPATH_DEPTH {
            return limit_exceeded("slice subpath depth", u64::from(depth), MAX_SLICE_SUBPATH_DEPTH);
        }
        if component.is_empty() || component == "." || component == ".." {
            return invalid_scalar("slice subpath", value, "slice-subpath-invalid");
        }
    }
    assert!(depth > 0);
    assert!(depth <= MAX_SLICE_SUBPATH_DEPTH);
    Ok(())
}

fn validate_store_name(name: &str) -> Result<(), DynamicPlanError> {
    if name.is_empty() {
        return invalid_scalar("slice store name", name, "must have 1..64 bytes");
    }
    let name_bytes = len_as_u64("slice store name", name.len())?;
    if name_bytes > u64::from(MAX_DYNAMIC_PLAN_OUTPUT_NAME_BYTES) {
        return invalid_scalar("slice store name", name, "must have 1..64 bytes");
    }
    if nix_compat::store_path::validate_name(name).is_err() {
        return invalid_scalar("slice store name", name, "contains invalid Nix store name character");
    }
    Ok(())
}

fn admit_source(wire: WireSourceV2, prefix: &str) -> Result<SourceV2, DynamicPlanError> {
    match wire {
        WireSourceV2::StorePath(source) => Ok(SourceV2::StorePath(DeclaredSourceInput {
            id: SourceId::new(source.id)?,
            path: StorePathString::new(source.path, prefix)?,
            nar_blake3: source.nar_blake3.map(NarDigest::new).transpose()?,
        })),
        WireSourceV2::Slice(slice) => {
            validate_slice_subpath(&slice.subpath)?;
            validate_store_name(&slice.store_name)?;
            Ok(SourceV2::Slice(SliceSource {
                id: SourceId::new(slice.id)?,
                producer_output: OutputName::new(slice.producer_output)?,
                subpath: slice.subpath,
                store_name: slice.store_name,
                nar_blake3: NarDigest::new(slice.nar_blake3)?,
            }))
        }
    }
}

pub fn decode_plan_v2(bytes: &[u8]) -> Result<WireDynamicPlanV2, DynamicPlanError> {
    let actual_bytes = len_as_u64("plan bytes", bytes.len())?;
    if actual_bytes > MAX_DYNAMIC_PLAN_BYTES {
        return Err(DynamicPlanError::PlanTooLarge {
            actual_bytes,
            max_bytes: MAX_DYNAMIC_PLAN_BYTES,
        });
    }
    let value = decode_plan_json_value(bytes)?;
    validate_json_nesting_depth(&value)?;
    // v1's nullable fields remain required on path sources and on units.
    require_nullable_fields_present(&value)?;
    serde_json::from_value(value).map_err(|error| DynamicPlanError::JsonDecode {
        message: error.to_string(),
    })
}

pub fn admit_plan_v2(wire: WireDynamicPlanV2, prefix: &str) -> Result<DynamicPlanV2, DynamicPlanError> {
    admit_plan_v2_with_bytes(wire, prefix).map(|(plan, _)| plan)
}

// The producer-supplied wire fields are checked through fallible validators;
// the one assertion below only guards uniqueness after canonical deduplication.
#[allow(tigerstyle::assertion_density)]
fn admit_plan_v2_with_bytes(
    wire: WireDynamicPlanV2,
    prefix: &str,
) -> Result<(DynamicPlanV2, Vec<u8>), DynamicPlanError> {
    validate_store_prefix(prefix)?;
    if wire.schema != MANTLE_PLAN_V2_SCHEMA {
        return invalid_scalar("schema", wire.schema, "must be mantle-plan-v2");
    }
    validate_len_limit("units", wire.units.len(), MAX_DYNAMIC_PLAN_UNITS)?;
    for unit in &wire.units {
        validate_len_limit("unit dependencies", unit.derivation.inputs.len(), MAX_DYNAMIC_PLAN_DEPENDENCIES_PER_UNIT)?;
        validate_len_limit("unit outputs", unit.derivation.outputs.len(), MAX_DYNAMIC_PLAN_OUTPUTS_PER_UNIT)?;
        validate_len_limit("requested outputs", unit.requested_outputs.len(), MAX_DYNAMIC_PLAN_OUTPUTS_PER_UNIT)?;
        validate_len_limit(
            "dynamic plan outputs",
            unit.derivation.dynamic_plan_outputs.len(),
            MAX_DYNAMIC_PLAN_OUTPUTS_PER_UNIT,
        )?;
        validate_len_limit("unit environment", unit.derivation.env.len(), MAX_DYNAMIC_PLAN_ENV_ENTRIES_PER_UNIT)?;
    }
    let slice_count = wire.sources.iter().filter(|source| matches!(source, WireSourceV2::Slice(_))).count();
    validate_len_limit("slice count", slice_count, MAX_PLAN_SLICES)?;
    let mut plan = DynamicPlanV2 {
        producer: PlanProducer {
            logical_name: wire.producer.logical_name,
            goal_hint: wire.producer.goal_hint,
        },
        sources: wire.sources.into_iter().map(|source| admit_source(source, prefix)).collect::<Result<_, _>>()?,
        units: wire.units.into_iter().map(|unit| wire::admit_unit(unit, prefix)).collect::<Result<_, _>>()?,
        roots: wire.roots.into_iter().map(UnitId::new).collect::<Result<_, _>>()?,
        provenance: wire.provenance,
    };
    plan.sources.sort_by(|left, right| left.id().cmp(right.id()));
    for pair in plan.sources.windows(2) {
        if pair[0].id() == pair[1].id() && (pair[0] != pair[1] || !matches!(pair[0], SourceV2::Slice(_))) {
            return invalid_scalar("source id", pair[0].id().as_str(), "slice-conflict");
        }
    }
    plan.sources.dedup();
    validate_producer(&plan.producer, prefix)?;
    validate_units(&plan.units, prefix)?;
    validate_roots(&plan.roots)?;
    validate_provenance(&plan.provenance, prefix)?;
    let ids = plan.sources.iter().map(SourceV2::id).collect::<BTreeSet<_>>();
    assert_eq!(ids.len(), plan.sources.len());
    let outputs = collect_unit_outputs(&plan.units)?;
    validate_root_graph(&plan.roots, &outputs)?;
    validate_input_graph(&plan.units, &ids, &outputs)?;
    validate_unit_dependency_cycles(&plan.units)?;
    let bytes = canonical_plan_v2_bytes(&plan)?;
    Ok((plan, bytes))
}

fn project_source(source: &SourceV2) -> WireSourceV2 {
    match source {
        SourceV2::StorePath(source) => WireSourceV2::StorePath(WireDeclaredSourceInput::from(source)),
        SourceV2::Slice(slice) => WireSourceV2::Slice(WireSliceSource {
            id: slice.id.as_str().to_owned(),
            producer_output: slice.producer_output.as_str().to_owned(),
            subpath: slice.subpath.clone(),
            store_name: slice.store_name.clone(),
            nar_blake3: slice.nar_blake3.as_str().to_owned(),
        }),
    }
}

// Canonical JSON must be nonempty; oversized output returns PlanTooLarge.
// A second assertion would merely repeat that fallible byte-limit check.
#[allow(tigerstyle::assertion_density)]
pub fn canonical_plan_v2_bytes(plan: &DynamicPlanV2) -> Result<Vec<u8>, DynamicPlanError> {
    let mut sources = plan.sources.iter().collect::<Vec<_>>();
    sources.sort_by(|left, right| left.id().cmp(right.id()));
    let mut units = plan.units.iter().cloned().map(canonicalize_unit).collect::<Vec<_>>();
    units.sort_by(|left, right| left.id.cmp(&right.id));
    let mut roots = plan.roots.iter().map(|root| root.as_str().to_owned()).collect::<Vec<_>>();
    roots.sort();
    let wire = WireDynamicPlanV2 {
        schema: MANTLE_PLAN_V2_SCHEMA.to_owned(),
        producer: WirePlanProducer::from(&plan.producer),
        sources: sources.into_iter().map(project_source).collect(),
        units: units.iter().map(WireDynamicUnit::from).collect(),
        roots,
        provenance: plan.provenance.clone(),
    };
    let bytes = serde_json::to_vec(&wire).map_err(|error| DynamicPlanError::CanonicalJson {
        message: error.to_string(),
    })?;
    let actual_bytes = len_as_u64("canonical plan bytes", bytes.len())?;
    if actual_bytes > MAX_DYNAMIC_PLAN_BYTES {
        return Err(DynamicPlanError::PlanTooLarge {
            actual_bytes,
            max_bytes: MAX_DYNAMIC_PLAN_BYTES,
        });
    }
    assert!(!bytes.is_empty());
    Ok(bytes)
}

pub fn decode_validated_plan_v2(bytes: &[u8], prefix: &str) -> Result<CanonicalDynamicPlanV2, DynamicPlanError> {
    let (plan, bytes) = admit_plan_v2_with_bytes(decode_plan_v2(bytes)?, prefix)?;
    let digest = PlanDigest::from_canonical_bytes(&bytes);
    assert!(!bytes.is_empty());
    assert_eq!(digest.as_str().len(), BLAKE3_HEX_BYTES);
    Ok(CanonicalDynamicPlanV2 { plan, bytes, digest })
}
