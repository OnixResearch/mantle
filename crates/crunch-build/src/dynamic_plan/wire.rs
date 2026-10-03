use super::DeclaredSourceInput;
use super::DynamicDerivation;
use super::DynamicInput;
use super::DynamicPlanError;
use super::DynamicPlanV1;
use super::DynamicUnit;
use super::DynamicUnitPolicy;
use super::FixedOutputSpec;
use super::MAX_DYNAMIC_PLAN_DEPENDENCIES_PER_UNIT;
use super::MAX_DYNAMIC_PLAN_ENV_ENTRIES_PER_UNIT;
use super::MAX_DYNAMIC_PLAN_OUTPUTS_PER_UNIT;
use super::MAX_DYNAMIC_PLAN_UNITS;
use super::NarDigest;
use super::OutputName;
use super::PlanProducer;
use super::SourceId;
use super::StorePathString;
use super::UnitId;
use super::WireDeclaredSourceInput;
use super::WireDynamicDerivation;
use super::WireDynamicInput;
use super::WireDynamicPlanV1;
use super::WireDynamicUnit;
use super::WireDynamicUnitPolicy;
use super::WireFixedOutputSpec;
use super::WirePlanProducer;
use super::validate_len_limit;
use super::validate_plan_v1;
use super::validate_store_prefix;

/// Admit a structural wire plan into the checked dynamic-plan core.
pub fn admit_plan_v1(wire: WireDynamicPlanV1, store_prefix: &str) -> Result<DynamicPlanV1, DynamicPlanError> {
    validate_store_prefix(store_prefix)?;
    validate_wire_limits(&wire)?;

    let plan = DynamicPlanV1 {
        schema: wire.schema,
        producer: admit_producer(wire.producer),
        sources: wire
            .sources
            .into_iter()
            .map(|source| admit_source(source, store_prefix))
            .collect::<Result<Vec<_>, _>>()?,
        units: wire.units.into_iter().map(|unit| admit_unit(unit, store_prefix)).collect::<Result<Vec<_>, _>>()?,
        roots: wire.roots.into_iter().map(UnitId::new).collect::<Result<Vec<_>, _>>()?,
        provenance: wire.provenance,
    };
    validate_plan_v1(&plan, store_prefix)?;
    Ok(plan)
}

fn validate_wire_limits(wire: &WireDynamicPlanV1) -> Result<(), DynamicPlanError> {
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
    Ok(())
}

fn admit_producer(wire: WirePlanProducer) -> PlanProducer {
    PlanProducer {
        logical_name: wire.logical_name,
        goal_hint: wire.goal_hint,
    }
}

fn admit_source(wire: WireDeclaredSourceInput, store_prefix: &str) -> Result<DeclaredSourceInput, DynamicPlanError> {
    Ok(DeclaredSourceInput {
        id: SourceId::new(wire.id)?,
        path: StorePathString::new(wire.path, store_prefix)?,
        nar_blake3: wire.nar_blake3.map(NarDigest::new).transpose()?,
    })
}

pub(in crate::dynamic_plan) fn admit_unit(
    wire: WireDynamicUnit,
    store_prefix: &str,
) -> Result<DynamicUnit, DynamicPlanError> {
    Ok(DynamicUnit {
        id: UnitId::new(wire.id)?,
        derivation: admit_derivation(wire.derivation, store_prefix)?,
        requested_outputs: wire.requested_outputs.into_iter().map(OutputName::new).collect::<Result<Vec<_>, _>>()?,
        policy: admit_policy(wire.policy),
    })
}

fn admit_policy(wire: WireDynamicUnitPolicy) -> DynamicUnitPolicy {
    DynamicUnitPolicy {
        sandbox: wire.sandbox,
        substitutions: wire.substitutions,
        store_prefix: wire.store_prefix,
        host_paths: wire.host_paths,
    }
}

fn admit_derivation(wire: WireDynamicDerivation, store_prefix: &str) -> Result<DynamicDerivation, DynamicPlanError> {
    Ok(DynamicDerivation {
        name: wire.name,
        builder: StorePathString::new(wire.builder, store_prefix)?,
        system: wire.system,
        args: wire.args,
        outputs: wire.outputs.into_iter().map(OutputName::new).collect::<Result<Vec<_>, _>>()?,
        env: wire.env,
        inputs: wire
            .inputs
            .into_iter()
            .map(|input| admit_input(input, store_prefix))
            .collect::<Result<Vec<_>, _>>()?,
        fixed_output: wire.fixed_output.map(admit_fixed_output),
        addressing_mode: wire.addressing_mode,
        sandbox: wire.sandbox,
        dynamic_plan_outputs: wire
            .dynamic_plan_outputs
            .into_iter()
            .map(OutputName::new)
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn admit_input(wire: WireDynamicInput, store_prefix: &str) -> Result<DynamicInput, DynamicPlanError> {
    match wire {
        WireDynamicInput::StorePath { path } => Ok(DynamicInput::StorePath {
            path: StorePathString::new(path, store_prefix)?,
        }),
        WireDynamicInput::Source { source } => Ok(DynamicInput::Source {
            source: SourceId::new(source)?,
        }),
        WireDynamicInput::UnitOutput { unit, output } => Ok(DynamicInput::UnitOutput {
            unit: UnitId::new(unit)?,
            output: OutputName::new(output)?,
        }),
    }
}

fn admit_fixed_output(wire: WireFixedOutputSpec) -> FixedOutputSpec {
    FixedOutputSpec {
        mode: wire.mode,
        algo: wire.algo,
        hash: wire.hash,
    }
}

impl From<&DynamicPlanV1> for WireDynamicPlanV1 {
    fn from(plan: &DynamicPlanV1) -> Self {
        Self {
            schema: plan.schema.clone(),
            producer: WirePlanProducer::from(&plan.producer),
            sources: plan.sources.iter().map(WireDeclaredSourceInput::from).collect(),
            units: plan.units.iter().map(WireDynamicUnit::from).collect(),
            roots: plan.roots.iter().map(|root| root.as_str().to_owned()).collect(),
            provenance: plan.provenance.clone(),
        }
    }
}

impl From<&PlanProducer> for WirePlanProducer {
    fn from(producer: &PlanProducer) -> Self {
        Self {
            logical_name: producer.logical_name.clone(),
            goal_hint: producer.goal_hint.clone(),
        }
    }
}

impl From<&DeclaredSourceInput> for WireDeclaredSourceInput {
    fn from(source: &DeclaredSourceInput) -> Self {
        Self {
            id: source.id.as_str().to_owned(),
            path: source.path.as_str().to_owned(),
            nar_blake3: source.nar_blake3.as_ref().map(|digest| digest.as_str().to_owned()),
        }
    }
}

impl From<&DynamicUnit> for WireDynamicUnit {
    fn from(unit: &DynamicUnit) -> Self {
        Self {
            id: unit.id.as_str().to_owned(),
            derivation: WireDynamicDerivation::from(&unit.derivation),
            requested_outputs: unit.requested_outputs.iter().map(|output| output.as_str().to_owned()).collect(),
            policy: WireDynamicUnitPolicy::from(&unit.policy),
        }
    }
}

impl From<&DynamicUnitPolicy> for WireDynamicUnitPolicy {
    fn from(policy: &DynamicUnitPolicy) -> Self {
        Self {
            sandbox: policy.sandbox.clone(),
            substitutions: policy.substitutions.clone(),
            store_prefix: policy.store_prefix.clone(),
            host_paths: policy.host_paths.clone(),
        }
    }
}

impl From<&DynamicDerivation> for WireDynamicDerivation {
    fn from(derivation: &DynamicDerivation) -> Self {
        Self {
            name: derivation.name.clone(),
            builder: derivation.builder.as_str().to_owned(),
            system: derivation.system.clone(),
            args: derivation.args.clone(),
            outputs: derivation.outputs.iter().map(|output| output.as_str().to_owned()).collect(),
            env: derivation.env.clone(),
            inputs: derivation.inputs.iter().map(WireDynamicInput::from).collect(),
            fixed_output: derivation.fixed_output.as_ref().map(WireFixedOutputSpec::from),
            addressing_mode: derivation.addressing_mode.clone(),
            sandbox: derivation.sandbox.clone(),
            dynamic_plan_outputs: derivation
                .dynamic_plan_outputs
                .iter()
                .map(|output| output.as_str().to_owned())
                .collect(),
        }
    }
}

impl From<&DynamicInput> for WireDynamicInput {
    fn from(input: &DynamicInput) -> Self {
        match input {
            DynamicInput::StorePath { path } => Self::StorePath {
                path: path.as_str().to_owned(),
            },
            DynamicInput::Source { source } => Self::Source {
                source: source.as_str().to_owned(),
            },
            DynamicInput::UnitOutput { unit, output } => Self::UnitOutput {
                unit: unit.as_str().to_owned(),
                output: output.as_str().to_owned(),
            },
        }
    }
}

impl From<&FixedOutputSpec> for WireFixedOutputSpec {
    fn from(spec: &FixedOutputSpec) -> Self {
        Self {
            mode: spec.mode.clone(),
            algo: spec.algo.clone(),
            hash: spec.hash.clone(),
        }
    }
}
