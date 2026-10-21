//! Thin explicit-target profile CLI; package and publication policy live in user_profile_cmd.
use std::path::PathBuf;

use clap::Args;
use clap::Subcommand;
use crunch_store::StoreConfig;
use mantle::user_profile_cmd::ProfileOperation;
use mantle::user_profile_cmd::ProfileRequest;
use nix_compat::narinfo::VerifyingKey;

use crate::errors::RunError;

#[derive(Args, Debug, Clone)]
pub struct ProfileArgs {
    /// Absolute target; no default home profile or global PATH mutation.
    #[arg(long)]
    pub profile: PathBuf,
    /// Exact expected active generation, or `empty`; required for mutations.
    #[arg(long)]
    pub expect_generation: Option<String>,
    /// Publish the admitted plan. Without this flag mutations are inspection only.
    #[arg(long)]
    pub apply: bool,
    /// Explicit trusted PathInfo signer; repeat for multiple keys.
    #[arg(long = "trusted-public-key")]
    pub trusted_public_keys: Vec<String>,
    #[command(subcommand)]
    pub action: ProfileAction,
}

#[derive(Subcommand, Debug, Clone)]
pub enum ProfileAction {
    /// Install an exact admitted package request, optionally replacing its named slot.
    Install {
        #[arg(long)]
        package: PathBuf,
        #[arg(long)]
        replace: bool,
    },
    /// Remove a named slot, retaining earlier generations.
    Remove { name: String },
    /// Inspect the selected package manifest.
    List,
    /// Inspect retained and pending generations.
    Generations,
    /// Select an existing generation after current output admission.
    Switch { generation: u64 },
    /// Select the preceding retained generation.
    Rollback,
    /// Delete an inactive generation and release only its own retention pins.
    RemoveGeneration { generation: u64 },
}

fn expected_generation(args: &ProfileArgs) -> Result<Option<u64>, RunError> {
    let mutation = !matches!(args.action, ProfileAction::List | ProfileAction::Generations);
    let Some(value) = args.expect_generation.as_deref() else {
        return if mutation {
            Err(RunError::Internal("profile mutations require --expect-generation empty|N".into()))
        } else {
            Ok(None)
        };
    };
    if value == "empty" {
        return Ok(None);
    }
    let number = value.parse::<u64>().map_err(|_| RunError::Internal("invalid profile expected generation".into()))?;
    if number == 0 || number.to_string() != value {
        return Err(RunError::Internal("profile expected generation must be canonical positive u64 or empty".into()));
    }
    Ok(Some(number))
}

pub fn run(args: &ProfileArgs, store: StoreConfig) -> Result<(), RunError> {
    let expected_active = expected_generation(args)?;
    if !args.profile.is_absolute() {
        return Err(RunError::Internal("profile target must be absolute".into()));
    }
    let trusted_keys = args
        .trusted_public_keys
        .iter()
        .map(|key| {
            VerifyingKey::parse(key).map_err(|error| RunError::Internal(format!("profile trusted key: {error}")))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let operation = match &args.action {
        ProfileAction::Install { package, replace } => ProfileOperation::Install {
            package: crate::rust_plan_execution::through_rust_port(
                mantle_application_contract::rust_operations::RustOperation::ReadProfilePackage,
                || mantle::user_profile_cmd::read_package_request(package).map_err(RunError::Build),
                |package| mantle_application_contract::rust_operations::RustEvidence::ProfilePackage {
                    name: &package.name,
                    output_path: &package.output_path,
                },
            )?,
            replace: *replace,
        },
        ProfileAction::Remove { name } => ProfileOperation::Remove { name: name.clone() },
        ProfileAction::List => ProfileOperation::List,
        ProfileAction::Generations => ProfileOperation::Generations,
        ProfileAction::Switch { generation } => ProfileOperation::Switch {
            generation: *generation,
        },
        ProfileAction::Rollback => ProfileOperation::Rollback,
        ProfileAction::RemoveGeneration { generation } => ProfileOperation::RemoveGeneration {
            generation: *generation,
        },
    };
    let runtime =
        tokio::runtime::Runtime::new().map_err(|error| RunError::Internal(format!("profile runtime: {error}")))?;
    use mantle_application_contract::rust_operations::RustEvidence;
    use mantle_application_contract::rust_operations::RustOperation;
    let (operation_name, inspection, removal) = match &args.action {
        ProfileAction::Install { .. } => ("install", false, false),
        ProfileAction::Remove { .. } => ("remove", false, false),
        ProfileAction::List => ("list", true, false),
        ProfileAction::Generations => ("generations", true, false),
        ProfileAction::Switch { .. } => ("switch", false, false),
        ProfileAction::Rollback => ("rollback", false, false),
        ProfileAction::RemoveGeneration { .. } => ("remove-generation", false, true),
    };
    let report = crate::rust_plan_execution::through_rust_port(
        if inspection {
            RustOperation::InspectProfile
        } else {
            RustOperation::MutateProfile
        },
        || {
            runtime
                .block_on(mantle::user_profile_cmd::execute_profile(ProfileRequest {
                    profile: args.profile.clone(),
                    store,
                    trusted_keys,
                    expected_active,
                    apply: args.apply,
                    operation,
                }))
                .map_err(RunError::Build)
        },
        |report| RustEvidence::Profile {
            schema_valid: report.schema == "mantle-user-profile-report-v1",
            operation_matches: report.operation == operation_name,
            applied: report.applied,
            apply_requested: args.apply,
            inspection,
            removal,
            admission_checked: report.admission_checked,
            active: report.active,
            previous_active: report.previous_active,
            target: report.target_generation,
            target_retained: report
                .target_generation
                .is_some_and(|target| report.retained_generations.contains(&target)),
            has_manifest: report.manifest.is_some(),
        },
    )?;
    println!("{}", serde_json::to_string_pretty(&report).map_err(|error| RunError::Internal(error.to_string()))?);
    Ok(())
}
