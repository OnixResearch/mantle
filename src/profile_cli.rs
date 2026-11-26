//! Thin explicit-target profile CLI; package and publication policy live in user_profile_cmd.
use std::path::PathBuf;

use clap::Args;
use clap::Subcommand;
use crunch_store::StoreConfig;
use mantle_application_contract::rust_operations::RustEvidence;
use mantle_application_contract::rust_operations::RustOperation;
use nix_compat::narinfo::VerifyingKey;

use crate::errors::RunError;
use crate::user_profile_cmd::ProfileOperation;
use crate::user_profile_cmd::ProfileRequest;

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
    let is_inspection = matches!(args.action, ProfileAction::List | ProfileAction::Generations);
    let Some(value) = args.expect_generation.as_deref() else {
        if is_inspection {
            return Ok(None);
        }
        return Err(RunError::Internal("profile mutations require --expect-generation empty|N".into()));
    };
    if value == "empty" {
        return Ok(None);
    }
    debug_assert_ne!(value, "empty");
    let generation_num =
        value.parse::<u64>().map_err(|_| RunError::Internal("invalid profile expected generation".into()))?;
    if generation_num == 0 || generation_num.to_string() != value {
        return Err(RunError::Internal("profile expected generation must be canonical positive u64 or empty".into()));
    }
    debug_assert!(generation_num > 0);
    Ok(Some(generation_num))
}

fn parse_trusted_keys(raw_keys: &[String]) -> Result<Vec<VerifyingKey>, RunError> {
    let trusted_keys = raw_keys
        .iter()
        .map(|key| {
            VerifyingKey::parse(key).map_err(|error| RunError::Internal(format!("profile trusted key: {error}")))
        })
        .collect::<Result<Vec<_>, _>>()?;
    debug_assert_eq!(trusted_keys.len(), raw_keys.len());
    Ok(trusted_keys)
}

fn build_profile_operation(action: &ProfileAction) -> Result<ProfileOperation, RunError> {
    let operation = match action {
        ProfileAction::Install { package, replace } => {
            debug_assert!(!package.as_os_str().is_empty());
            ProfileOperation::Install {
                package: crate::rust_plan_execution::through_rust_port(
                    RustOperation::ReadProfilePackage,
                    || crate::user_profile_cmd::read_package_request(package).map_err(RunError::Build),
                    |package| RustEvidence::ProfilePackage {
                        name: &package.name,
                        output_path: &package.output_path,
                    },
                )?,
                replace: *replace,
            }
        }
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
    Ok(operation)
}

fn profile_operation_metadata(action: &ProfileAction) -> (&'static str, bool, bool) {
    match action {
        ProfileAction::Install { .. } => ("install", false, false),
        ProfileAction::Remove { .. } => ("remove", false, false),
        ProfileAction::List => ("list", true, false),
        ProfileAction::Generations => ("generations", true, false),
        ProfileAction::Switch { .. } => ("switch", false, false),
        ProfileAction::Rollback => ("rollback", false, false),
        ProfileAction::RemoveGeneration { .. } => ("remove-generation", false, true),
    }
}

pub fn run(args: &ProfileArgs, store: StoreConfig) -> Result<(), RunError> {
    let expected_active = expected_generation(args)?;
    if !args.profile.is_absolute() {
        return Err(RunError::Internal("profile target must be absolute".into()));
    }
    debug_assert!(args.profile.is_absolute());
    let trusted_keys = parse_trusted_keys(&args.trusted_public_keys)?;
    let operation = build_profile_operation(&args.action)?;
    let runtime =
        tokio::runtime::Runtime::new().map_err(|error| RunError::Internal(format!("profile runtime: {error}")))?;
    let (operation_name, is_inspection, is_removal) = profile_operation_metadata(&args.action);
    debug_assert!(!operation_name.is_empty());
    let rust_operation = if is_inspection {
        RustOperation::InspectProfile
    } else {
        RustOperation::MutateProfile
    };
    let profile_result = crate::rust_plan_execution::through_rust_port(
        rust_operation,
        || {
            runtime
                .block_on(crate::user_profile_cmd::execute_profile(ProfileRequest {
                    profile: args.profile.clone(),
                    store,
                    trusted_keys,
                    expected_active,
                    apply: args.apply,
                    operation,
                }))
                .map_err(RunError::Build)
        },
        |outcome| RustEvidence::Profile {
            schema_valid: outcome.schema == "mantle-user-profile-report-v1",
            operation_matches: outcome.operation == operation_name,
            applied: outcome.applied,
            apply_requested: args.apply,
            inspection: is_inspection,
            removal: is_removal,
            admission_checked: outcome.admission_checked,
            active: outcome.active,
            previous_active: outcome.previous_active,
            target: outcome.target_generation,
            target_retained: outcome
                .target_generation
                .is_some_and(|target| outcome.retained_generations.contains(&target)),
            has_manifest: outcome.manifest.is_some(),
        },
    )?;
    println!(
        "{}",
        serde_json::to_string_pretty(&profile_result).map_err(|error| RunError::Internal(error.to_string()))?
    );
    Ok(())
}
