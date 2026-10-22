//! pnpm imperative shell, invoked ONLY as an ordinary network-denied Mantle builder.
//! Store/output authority remains with the enclosing derivation; this helper has no cache.
mod admission;
pub mod files;
use std::fs;
use std::path::Path;
use std::process::Command;

use crunch_pnpm_core::NODE_VERSION;
use crunch_pnpm_core::PNPM_VERSION;
use crunch_pnpm_core::Request;
use files::Result;
use serde_json::Value;
use serde_json::json;

/// Working-set size of the explicit traversal stacks.
const STACK_CAPACITY: usize = 64;

pub fn execute(request: Request, source: &Path, mirror: &Path, tools: &Path, output: &Path) -> Result<()> {
    let crunch_pnpm_core::AdmittedRequest {
        request,
        identity,
        sources,
    } = crunch_pnpm_core::validate(request)?;
    verify_inputs(&request, source, mirror, tools)?;
    admission::admit(&request, source, mirror)?;
    if output.exists() {
        return Err("output must be absent; warm reuse belongs to Mantle's trusted store".into());
    }
    let scratch = tempfile::tempdir()?;
    let work = scratch.path().join("workspace");
    let home = scratch.path().join("home");
    let store = scratch.path().join("store");
    fs::create_dir(&home)?;
    fs::create_dir(&store)?;
    files::copy(source, &work)?;
    let lock_before = files::bytes(&work.join("pnpm-lock.yaml"))?;
    let shell = Shell {
        tools,
        home: &home,
        work: &work,
        store: &store,
    };
    verify_toolchain(&shell, &request)?;
    seed_archives(&shell, &request, mirror)?;
    install(&shell)?;
    validate_installed_links(&work, &work)?;
    run_targets(&shell, &request, &work, &lock_before)?;
    verify_frozen_lock(&work, &lock_before)?;
    fs::create_dir(output)?;
    let outputs = collect_outputs(&request, &work, output)?;
    files::write_json(
        &output.join("provenance.json"),
        &json!({
            "schema": "mantle-pnpm-output-v1", "identity_blake3": identity,
            "inputs": request, "sources": sources, "outputs": outputs,
            "execution": "pnpm-inside-mantle-network-denied-derivation",
            "non_claim": "This helper is not sandbox authority; only the enclosing signed Mantle build receipt establishes sandbox execution and trusted reuse."
        }),
    )?;
    debug_assert!(!request.targets.is_empty());
    debug_assert!(request.packages.len() <= crunch_pnpm_core::MAX_PACKAGES);
    Ok(())
}

/// Every declared input must match its admitted content identity and carry no symlink.
fn verify_inputs(request: &Request, source: &Path, mirror: &Path, tools: &Path) -> Result<()> {
    files::tree(mirror)?;
    let source_digest = files::tree(source)?;
    if source_digest != request.source_blake3 {
        return Err("source/tool content identity mismatch".into());
    }
    let tools_digest = files::tree(tools)?;
    if tools_digest != request.tools_blake3 {
        return Err("source/tool content identity mismatch".into());
    }
    files::reject_links(source)?;
    files::reject_links(mirror)?;
    verify_runtime_inputs(request)?;
    verify_tool_roots(request, tools)
}

/// Every declared runtime input must match its admitted content identity.
fn verify_runtime_inputs(request: &Request) -> Result<()> {
    for runtime in &request.runtime {
        if files::tree(Path::new(&runtime.path))? != runtime.blake3 {
            return Err("runtime identity mismatch".into());
        }
    }
    Ok(())
}

/// Every tool root may only link inside the declared roots.
fn verify_tool_roots(request: &Request, tools: &Path) -> Result<()> {
    let mut roots = vec![tools];
    roots.extend(request.runtime.iter().map(|input| Path::new(&input.path)));
    for root in &roots {
        files::tool_links(root, &roots)?;
    }
    debug_assert_eq!(roots.len(), request.runtime.len().saturating_add(1));
    Ok(())
}

/// The pinned Node and pnpm releases must be the ones this shell runs.
fn verify_toolchain(shell: &Shell, request: &Request) -> Result<()> {
    let node = shell.node(&[
        "--eval",
        "process.stdout.write(process.versions.node + '\\n' + process.platform + '-' + process.arch)",
    ])?;
    if node != format!("{NODE_VERSION}\n{}", request.platform) {
        return Err("pinned Node/pnpm version/platform mismatch".into());
    }
    if shell.pnpm_version()? != PNPM_VERSION {
        return Err("pinned Node/pnpm version/platform mismatch".into());
    }
    Ok(())
}

/// pnpm v9 indexes tarballs by SRI, independent of registry locator. Local archives
/// seed that native index without inventing a resolver or modifying the wanted lock.
fn seed_archives(shell: &Shell, request: &Request, mirror: &Path) -> Result<()> {
    for package in &request.packages {
        let archive = files::child(mirror, &package.archive)?;
        shell.pnpm(&["store", "add", "--offline", archive.to_str().ok_or("non-UTF8 archive")?])?;
    }
    Ok(())
}

/// Install the frozen lock without scripts, peer resolution or any resolution at all.
fn install(shell: &Shell) -> Result<()> {
    shell.pnpm(&[
        "install",
        "--offline",
        "--frozen-lockfile",
        "--ignore-scripts",
        "--strict-peer-dependencies",
    ])
}

/// Run every declared script and pack the declared package outputs.
fn run_targets(shell: &Shell, request: &Request, work: &Path, lock_before: &[u8]) -> Result<()> {
    for target in &request.targets {
        for script in &target.scripts {
            shell.pnpm(&["--dir", &target.workspace, "run", script])?;
            if files::bytes(&work.join("pnpm-lock.yaml"))? != lock_before {
                return Err("build mutated frozen lock".into());
            }
        }
        if let Some(destination) = &target.package_output {
            shell.pnpm(&["--dir", &target.workspace, "pack", "--pack-destination", destination])?;
        }
    }
    Ok(())
}

/// The install must not have mutated the frozen lock.
fn verify_frozen_lock(work: &Path, lock_before: &[u8]) -> Result<()> {
    if files::bytes(&work.join("pnpm-lock.yaml"))? != lock_before {
        return Err("install mutated frozen lock".into());
    }
    Ok(())
}

/// Copy every declared output into the output directory and record its identity.
fn collect_outputs(request: &Request, work: &Path, output: &Path) -> Result<Vec<Value>> {
    let mut outputs = Vec::with_capacity(request.targets.len());
    for target in &request.targets {
        for member in target.outputs.iter().chain(target.package_output.iter()) {
            let relative = format!("{}/{member}", target.workspace);
            let from = files::child(work, &relative)?;
            if !fs::canonicalize(&from)?.starts_with(work) {
                return Err("declared output escapes workspace".into());
            }
            let digest = files::tree(&from)?;
            files::reject_links(&from)?;
            files::copy(&from, &files::child(output, &relative)?)?;
            outputs.push(json!({"path": relative, "blake3": digest}));
        }
    }
    Ok(outputs)
}

struct Shell<'a> {
    tools: &'a Path,
    home: &'a Path,
    work: &'a Path,
    store: &'a Path,
}
impl Shell<'_> {
    fn command(&self) -> Command {
        let mut command = Command::new(self.tools.join("bin/node"));
        command
            .current_dir(self.work)
            .env_clear()
            .env("PATH", self.tools.join("bin"))
            .env("HOME", self.home)
            .env("XDG_CONFIG_HOME", self.home)
            .env("XDG_CACHE_HOME", self.home.join("cache"))
            .env("XDG_DATA_HOME", self.home.join("data"))
            .env("PNPM_HOME", self.home.join("pnpm"))
            .env("TMPDIR", self.home)
            .env("CI", "true")
            .env("LANG", "C")
            .env("LC_ALL", "C")
            .env("TZ", "UTC")
            .env("SOURCE_DATE_EPOCH", "1")
            .env("NPM_CONFIG_USERCONFIG", "/dev/null")
            .env("NPM_CONFIG_GLOBALCONFIG", "/dev/null")
            .env("NPM_CONFIG_STORE_DIR", self.store)
            .env("NPM_CONFIG_CACHE_DIR", self.home.join("cache"))
            .env("NPM_CONFIG_STATE_DIR", self.home.join("state"))
            .env("NPM_CONFIG_OFFLINE", "true")
            .env("NPM_CONFIG_IGNORE_SCRIPTS", "true")
            .env("NPM_CONFIG_ENABLE_PRE_POST_SCRIPTS", "false")
            .env("NPM_CONFIG_SCRIPT_SHELL", self.tools.join("bin/sh"))
            .env("NPM_CONFIG_MANAGE_PACKAGE_MANAGER_VERSIONS", "false")
            .env("NPM_CONFIG_PACKAGE_MANAGER_STRICT_VERSION", "true")
            .env("NPM_CONFIG_VERIFY_STORE_INTEGRITY", "true")
            .env("NPM_CONFIG_STRICT_STORE_PKG_CONTENT_CHECK", "true")
            .env("NPM_CONFIG_SIDE_EFFECTS_CACHE", "false")
            .env("NPM_CONFIG_PACKAGE_IMPORT_METHOD", "copy");
        command
    }
    fn node(&self, args: &[&str]) -> Result<String> {
        invoke(self.command().args(args))
    }
    fn pnpm_version(&self) -> Result<String> {
        invoke(self.command().arg(self.tools.join("pnpm.cjs")).arg("--version"))
    }
    fn pnpm(&self, args: &[&str]) -> Result<()> {
        // Inherit the enclosing builder's bounded log transport; never buffer
        // untrusted lifecycle/build output in this process.
        let status = self.command().arg(self.tools.join("pnpm.cjs")).args(args).status()?;
        if !status.success() {
            return Err(format!("pnpm failed: {status}").into());
        }
        Ok(())
    }
}
fn invoke(command: &mut Command) -> Result<String> {
    let result = command.output()?;
    if !result.status.success() {
        return Err(format!(
            "pnpm/Node failed: {}\n{}\n{}",
            result.status,
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        )
        .into());
    }
    Ok(String::from_utf8(result.stdout)?.trim().to_string())
}
/// Every installed workspace link must resolve inside the installed workspace.
fn validate_installed_links(root: &Path, path: &Path) -> Result<()> {
    let mut pending = Vec::with_capacity(STACK_CAPACITY);
    pending.push(directory_entries(path)?);
    while !pending.is_empty() {
        let next = pending.last_mut().and_then(|frame| frame.next());
        let Some(entry) = next else {
            pending.pop();
            continue;
        };
        let metadata = fs::symlink_metadata(entry.path())?;
        if metadata.file_type().is_symlink() {
            check_installed_link(root, &entry.path())?;
        } else if metadata.is_dir() {
            pending.push(directory_entries(&entry.path())?);
        }
    }
    Ok(())
}

/// A linked workspace must resolve inside the installed workspace root.
fn check_installed_link(root: &Path, path: &Path) -> Result<()> {
    let resolved = fs::canonicalize(path)?;
    if !resolved.starts_with(root) {
        return Err("installed workspace link escapes source".into());
    }
    Ok(())
}

/// Read one directory into the frame of an explicit depth first traversal.
fn directory_entries(path: &Path) -> Result<std::vec::IntoIter<fs::DirEntry>> {
    Ok(fs::read_dir(path)?.collect::<std::io::Result<Vec<_>>>()?.into_iter())
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    use super::*;

    /// A tree with nested directories, an executable file and two symlinks.
    fn fixture(root: &Path) {
        fs::create_dir_all(root.join("a/b/c")).unwrap();
        fs::create_dir_all(root.join("empty")).unwrap();
        fs::write(root.join("a/b/c/deep.txt"), "deep").unwrap();
        fs::write(root.join("top.txt"), "top").unwrap();
        fs::write(root.join("a/tool"), "tool").unwrap();
        fs::set_permissions(root.join("a/tool"), fs::Permissions::from_mode(0o755)).unwrap();
        std::os::unix::fs::symlink("../top.txt", root.join("a/b/link")).unwrap();
        std::os::unix::fs::symlink("c", root.join("a/dirlink")).unwrap();
    }

    /// The tree identity is the admitted content identity, so it must not drift.
    #[test]
    fn tree_identity_is_stable() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("tree");
        fs::create_dir(&root).unwrap();
        fixture(&root);
        assert_eq!(
            files::tree(&root).unwrap().as_str(),
            "c3cbd508c766f658b804eb65ef89c2c8ec2aa4c45fb5ce7b8884d873b41968ed"
        );
    }

    /// The executable bit and the empty directory are content of the identity.
    #[test]
    fn tree_identity_tracks_permission_and_empty_directories() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("tree");
        fs::create_dir(&root).unwrap();
        fs::write(root.join("tool"), "tool").unwrap();
        fs::set_permissions(root.join("tool"), fs::Permissions::from_mode(0o644)).unwrap();
        let plain = files::tree(&root).unwrap();
        fs::set_permissions(root.join("tool"), fs::Permissions::from_mode(0o755)).unwrap();
        assert_ne!(plain, files::tree(&root).unwrap());
        let executable = files::tree(&root).unwrap();
        fs::create_dir(root.join("empty")).unwrap();
        assert_ne!(executable, files::tree(&root).unwrap());
    }
}
