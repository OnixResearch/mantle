use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::VecDeque;
use std::process::Command;

const CORE: &str = "crunch-remote-core";
const ALLOWED_DIRECT: &[&str] = &["blake3", "serde"];

fn authority_class(package: &str) -> Option<&'static str> {
    if package.starts_with("snix-") || package == "nix-compat" {
        Some("snix")
    } else if package == "crunch-store" {
        Some("store")
    } else {
        match package {
            "tokio" => Some("async-runtime"),
            "tempfile" | "cap-std" => Some("filesystem"),
            "duct" | "xshell" => Some("process"),
            "dotenvy" | "envy" => Some("environment"),
            "chrono" | "time" => Some("clock"),
            "rand" | "getrandom" => Some("random"),
            "reqwest" | "ureq" => Some("network"),
            "secrecy" | "crunch-credentials" => Some("credential"),
            "clap" => Some("cli"),
            "serde_json" | "tera" => Some("rendering"),
            _ => None,
        }
    }
}

fn audit_runtime_graph(graph: &BTreeMap<String, Vec<String>>) -> Result<(), String> {
    let mut queued = VecDeque::from([(CORE.to_string(), vec![CORE.to_string()])]);
    let mut visited = BTreeSet::new();
    while let Some((package, path)) = queued.pop_front() {
        if !visited.insert(package.clone()) {
            continue;
        }
        let dependencies = graph
            .get(&package)
            .ok_or_else(|| format!("missing-runtime-dependency-node:{}", path.join(" -> ")))?;
        for dependency in dependencies {
            let mut next_path = path.clone();
            next_path.push(dependency.clone());
            let class = authority_class(dependency);
            if let Some(class) = class {
                return Err(format!("remote-core-forbidden-{class}:{}", next_path.join(" -> ")));
            }
            if package == CORE && !ALLOWED_DIRECT.contains(&dependency.as_str()) {
                return Err(format!("remote-core-unclassified-runtime-authority:{}", next_path.join(" -> ")));
            }
            queued.push_back((dependency.clone(), next_path));
        }
    }
    Ok(())
}

fn cargo_runtime_graph() -> BTreeMap<String, Vec<String>> {
    let output = Command::new(env!("CARGO"))
        .args(["metadata", "--format-version", "1", "--locked", "--offline"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run offline Cargo dependency metadata");
    assert!(output.status.success(), "metadata failed: {}", String::from_utf8_lossy(&output.stderr));
    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout).expect("Cargo metadata JSON");
    let packages = metadata["packages"].as_array().expect("packages");
    let by_id = packages
        .iter()
        .map(|package| (package["id"].as_str().expect("package ID"), package["name"].as_str().expect("package name")))
        .collect::<BTreeMap<_, _>>();
    let core_id = packages.iter().find(|package| package["name"] == CORE).expect("remote core package")["id"]
        .as_str()
        .expect("remote core package ID");
    let nodes = metadata["resolve"]["nodes"].as_array().expect("resolved dependencies");
    let mut graph = BTreeMap::new();
    for node in nodes {
        let id = node["id"].as_str().expect("resolved package ID");
        let name = by_id[id];
        let dependencies = node["deps"]
            .as_array()
            .expect("resolved dependency edges")
            .iter()
            .filter(|dependency| {
                dependency["dep_kinds"]
                    .as_array()
                    .expect("dependency kinds")
                    .iter()
                    .any(|kind| kind["kind"].is_null())
            })
            .map(|dependency| by_id[dependency["pkg"].as_str().expect("dependency package ID")].to_string())
            .collect::<Vec<_>>();
        graph.insert(name.to_string(), dependencies);
    }
    assert!(graph.contains_key(by_id[core_id]));
    graph
}

#[test]
fn remote_core_runtime_dependency_graph_has_no_host_authority() {
    audit_runtime_graph(&cargo_runtime_graph()).expect("no forbidden production authority may reach the core");
}
