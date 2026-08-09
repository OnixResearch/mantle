use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

use serde::Deserialize;

const COMMAND_COUNT_MAX: usize = 1_024;
const ARGUMENT_COUNT: usize = 2;
const SELF_TEST_FLAG: &str = "--self-test";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
struct Descriptor {
    path: String,
    aliases: Vec<String>,
    flags: Vec<String>,
    help: String,
    supports_json: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SupportTier {
    Daily,
    Advanced,
    Compatibility,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MutationClass {
    None,
    ProjectFiles,
    StoreState,
    ProjectAndStore,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum NetworkClass {
    None,
    Optional,
    Required,
}

fn main() -> ExitCode {
    match run(env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("operator contract generation failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    if args.as_slice() == [SELF_TEST_FLAG] {
        return self_test();
    }
    if args.len() != ARGUMENT_COUNT {
        return Err("usage: generate-operator-command-contract.rs <descriptors.json> <inventory.ncl>".to_string());
    }
    let input_path = Path::new(&args[0]);
    let output_path = Path::new(&args[1]);
    let bytes = fs::read(input_path).map_err(|error| format!("reading {}: {error}", input_path.display()))?;
    let descriptors: Vec<Descriptor> =
        serde_json::from_slice(&bytes).map_err(|error| format!("parsing {}: {error}", input_path.display()))?;
    let rendered = render_inventory(&descriptors)?;
    fs::write(output_path, rendered).map_err(|error| format!("writing {}: {error}", output_path.display()))?;
    Ok(())
}

fn render_inventory(descriptors: &[Descriptor]) -> Result<String, String> {
    validate_descriptors(descriptors)?;
    let mut output = inventory_header();
    output.push_str("{\n  schema = \"mantle-operator-surface-inventory-v1\",\n  surfaces = [\n");
    for descriptor in descriptors {
        output.push_str(&render_command_surface(descriptor));
    }
    output.push_str(static_surface_rows());
    output.push_str("  ],\n} | Inventory\n");
    Ok(output)
}

fn validate_descriptors(descriptors: &[Descriptor]) -> Result<(), String> {
    if descriptors.is_empty() {
        return Err("descriptor list is empty".to_string());
    }
    if descriptors.len() > COMMAND_COUNT_MAX {
        return Err(format!("descriptor count {} exceeds {COMMAND_COUNT_MAX}", descriptors.len()));
    }
    let mut previous = None::<&str>;
    for descriptor in descriptors {
        if descriptor.path.trim().is_empty() {
            return Err("descriptor path is empty".to_string());
        }
        if descriptor.help.trim().is_empty() {
            return Err(format!("descriptor help is empty for {}", descriptor.path));
        }
        if let Some(previous_path) = previous
            && previous_path >= descriptor.path.as_str()
        {
            return Err(format!("descriptor paths are not strictly sorted at {}", descriptor.path));
        }
        previous = Some(&descriptor.path);
    }
    Ok(())
}

fn inventory_header() -> String {
    String::from(
        "# Starter inventory from the public Clap graph. Review and own policy in this Nickel file.\n\
let Surface = {\n\
  kind | [| 'command, 'project-file, 'environment, 'machine-schema, 'identifier |],\n\
  canonical | String,\n\
  compatibility_spellings | Array String,\n\
  owner | String,\n\
  role | String,\n\
  supported_operations | Array String,\n\
  support_tier | [| 'daily, 'advanced, 'compatibility, 'internal |],\n\
  mutation | [| 'none, 'project-files, 'store-state, 'project-and-store |],\n\
  network | [| 'none, 'optional, 'required |],\n\
  compatibility_state | [| 'canonical, 'compatibility-read-write, 'compatibility-read-only, 'historical-only |],\n\
  flags | Array String,\n\
  exit_classes | Array String,\n\
  machine_schema | Dyn,\n\
  migration_reference | Dyn,\n\
  removal_gate | Dyn,\n\
} in\n\
let Inventory = {\n\
  schema | String,\n\
  surfaces | Array Surface,\n\
}\n\
in\n",
    )
}

fn render_command_surface(descriptor: &Descriptor) -> String {
    let tier = support_tier(descriptor.path.as_str());
    let mutation = mutation_class(descriptor.path.as_str());
    let network = network_class(descriptor.path.as_str());
    let compatibility_state = if tier == SupportTier::Compatibility {
        "compatibility-read-write"
    } else {
        "canonical"
    };
    let migration = if tier == SupportTier::Compatibility {
        "\"docs/mantle-naming.md\""
    } else {
        "null"
    };
    let removal_gate = if tier == SupportTier::Compatibility {
        "\"consumer inventory, migration instructions, and rollback evidence\""
    } else {
        "null"
    };
    let machine_schema = if descriptor.supports_json {
        "\"mantle-command-json-v1\""
    } else {
        "null"
    };
    format!(
        "    {{ kind = 'command, canonical = \"{}\", compatibility_spellings = {}, owner = \"mantle-cli\", role = \"public-command\", supported_operations = [\"invoke\", \"read-help\"], support_tier = '{}, mutation = '{}, network = '{}, compatibility_state = '{}, flags = {}, exit_classes = [\"policy-rejection\", \"success\", \"usage\"], machine_schema = {}, migration_reference = {}, removal_gate = {} }},\n",
        escape_nickel(&descriptor.path),
        render_string_array(&descriptor.aliases),
        support_tier_name(tier),
        mutation_name(mutation),
        network_name(network),
        compatibility_state,
        render_string_array(&descriptor.flags),
        machine_schema,
        migration,
        removal_gate,
    )
}

fn support_tier(path: &str) -> SupportTier {
    if path == "develop" {
        return SupportTier::Compatibility;
    }
    if matches!(path, "doctor" | "check" | "show" | "refresh" | "build" | "attest show") {
        return SupportTier::Daily;
    }
    SupportTier::Advanced
}

fn mutation_class(path: &str) -> MutationClass {
    let mutates_project = contains_path_word(path, &["apply", "init", "refresh", "upgrade", "fix"]);
    let mutates_store = contains_path_word(path, &[
        "build",
        "import",
        "export",
        "pin",
        "unpin",
        "gc",
        "sign",
        "create",
        "serve",
        "repair",
        "push",
        "pull",
        "run",
        "self-build",
        "hydrate",
        "rotate",
        "revoke",
    ]);
    match (mutates_project, mutates_store) {
        (false, false) => MutationClass::None,
        (true, false) => MutationClass::ProjectFiles,
        (false, true) => MutationClass::StoreState,
        (true, true) => MutationClass::ProjectAndStore,
    }
}

fn network_class(path: &str) -> NetworkClass {
    if contains_path_word(path, &["remote", "push", "pull", "refresh", "fetch", "serve"]) {
        return NetworkClass::Required;
    }
    if contains_path_word(path, &["build", "import", "export", "source", "bootstrap"]) {
        return NetworkClass::Optional;
    }
    NetworkClass::None
}

fn contains_path_word(path: &str, words: &[&str]) -> bool {
    path.split([' ', '-']).any(|part| words.contains(&part))
}

fn support_tier_name(tier: SupportTier) -> &'static str {
    match tier {
        SupportTier::Daily => "daily",
        SupportTier::Advanced => "advanced",
        SupportTier::Compatibility => "compatibility",
    }
}

fn mutation_name(class: MutationClass) -> &'static str {
    match class {
        MutationClass::None => "none",
        MutationClass::ProjectFiles => "project-files",
        MutationClass::StoreState => "store-state",
        MutationClass::ProjectAndStore => "project-and-store",
    }
}

fn network_name(class: NetworkClass) -> &'static str {
    match class {
        NetworkClass::None => "none",
        NetworkClass::Optional => "optional",
        NetworkClass::Required => "required",
    }
}

fn render_string_array(values: &[String]) -> String {
    let rendered = values.iter().map(|value| format!("\"{}\"", escape_nickel(value))).collect::<Vec<_>>().join(", ");
    format!("[{rendered}]")
}

fn escape_nickel(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn static_surface_rows() -> &'static str {
    "    { kind = 'project-file, canonical = \"crunch.ncl\", compatibility_spellings = [], owner = \"mantle-project\", role = \"legacy-project-input\", supported_operations = [\"read\", \"write\"], support_tier = 'compatibility, mutation = 'project-files, network = 'none, compatibility_state = 'compatibility-read-write, flags = [], exit_classes = [], machine_schema = null, migration_reference = \"docs/mantle-naming.md\", removal_gate = \"accepted project-file migration with consumer evidence\" },\n\
    { kind = 'project-file, canonical = \"crunch-project.ncl\", compatibility_spellings = [], owner = \"mantle-project\", role = \"legacy-project-input\", supported_operations = [\"read\", \"write\"], support_tier = 'compatibility, mutation = 'project-files, network = 'none, compatibility_state = 'compatibility-read-write, flags = [], exit_classes = [], machine_schema = null, migration_reference = \"docs/mantle-naming.md\", removal_gate = \"accepted project-file migration with consumer evidence\" },\n\
    { kind = 'project-file, canonical = \"crunch.lock\", compatibility_spellings = [], owner = \"mantle-project\", role = \"legacy-project-lock\", supported_operations = [\"read\", \"write\"], support_tier = 'compatibility, mutation = 'project-files, network = 'none, compatibility_state = 'compatibility-read-write, flags = [], exit_classes = [], machine_schema = null, migration_reference = \"docs/mantle-naming.md\", removal_gate = \"accepted lockfile migration with consumer evidence\" },\n\
    { kind = 'environment, canonical = \"CRUNCH_STATE_DIR\", compatibility_spellings = [], owner = \"mantle-cli\", role = \"legacy-environment-input\", supported_operations = [\"read\"], support_tier = 'compatibility, mutation = 'none, network = 'none, compatibility_state = 'compatibility-read-write, flags = [], exit_classes = [], machine_schema = null, migration_reference = \"docs/mantle-naming.md\", removal_gate = \"accepted environment migration with consumer evidence\" },\n\
    { kind = 'environment, canonical = \"CRUNCH_CONFIG_DIR\", compatibility_spellings = [], owner = \"mantle-cli\", role = \"legacy-environment-input\", supported_operations = [\"read\"], support_tier = 'compatibility, mutation = 'none, network = 'none, compatibility_state = 'compatibility-read-write, flags = [], exit_classes = [], machine_schema = null, migration_reference = \"docs/mantle-naming.md\", removal_gate = \"accepted environment migration with consumer evidence\" },\n\
    { kind = 'machine-schema, canonical = \"crunch-doctor-report-v1\", compatibility_spellings = [], owner = \"mantle-cli\", role = \"legacy-machine-output\", supported_operations = [\"produce\", \"read\"], support_tier = 'compatibility, mutation = 'none, network = 'none, compatibility_state = 'compatibility-read-only, flags = [], exit_classes = [], machine_schema = \"crunch-doctor-report-v1\", migration_reference = \"docs/mantle-naming.md\", removal_gate = \"schema migration and reader evidence\" },\n\
    { kind = 'machine-schema, canonical = \"crunch-build-report-v1\", compatibility_spellings = [], owner = \"mantle-cli\", role = \"legacy-machine-output\", supported_operations = [\"produce\", \"read\"], support_tier = 'compatibility, mutation = 'none, network = 'none, compatibility_state = 'compatibility-read-only, flags = [], exit_classes = [], machine_schema = \"crunch-build-report-v1\", migration_reference = \"docs/mantle-naming.md\", removal_gate = \"schema migration and reader evidence\" },\n\
    { kind = 'machine-schema, canonical = \"mantle-runtime-fingerprint-v1\", compatibility_spellings = [], owner = \"mantle-cli\", role = \"canonical-machine-output\", supported_operations = [\"produce\", \"read\"], support_tier = 'advanced, mutation = 'none, network = 'none, compatibility_state = 'canonical, flags = [], exit_classes = [], machine_schema = \"mantle-runtime-fingerprint-v1\", migration_reference = null, removal_gate = null },\n"
}

fn self_test() -> Result<(), String> {
    let valid = vec![Descriptor {
        path: "doctor".to_string(),
        aliases: Vec::new(),
        flags: vec!["--profile".to_string()],
        help: "Run checks".to_string(),
        supports_json: true,
    }];
    let rendered = render_inventory(&valid)?;
    if !rendered.contains("canonical = \"doctor\"")
        || !rendered.contains("role = \"public-command\"")
        || !rendered.contains("machine_schema = \"mantle-command-json-v1\"")
    {
        return Err("positive self-test did not render the command row".to_string());
    }

    let invalid = vec![Descriptor {
        path: String::new(),
        aliases: Vec::new(),
        flags: Vec::new(),
        help: "Missing path".to_string(),
        supports_json: false,
    }];
    let error = render_inventory(&invalid).expect_err("empty command path must fail");
    if error != "descriptor path is empty" {
        return Err(format!("negative self-test returned unexpected error: {error}"));
    }
    println!("operator command contract generator self-test: PASS");
    Ok(())
}
