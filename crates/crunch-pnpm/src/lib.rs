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
use serde_json::json;

pub fn execute(request: Request, source: &Path, mirror: &Path, tools: &Path, output: &Path) -> Result<()> {
    let crunch_pnpm_core::AdmittedRequest {
        request,
        identity,
        sources,
    } = crunch_pnpm_core::validate(request)?;
    files::tree(mirror)?;
    if files::tree(source)? != request.source_blake3 || files::tree(tools)? != request.tools_blake3 {
        return Err("source/tool content identity mismatch".into());
    }
    files::reject_links(source)?;
    files::reject_links(mirror)?;
    for runtime in &request.runtime {
        if files::tree(Path::new(&runtime.path))? != runtime.blake3 {
            return Err("runtime identity mismatch".into());
        }
    }
    let mut roots = vec![tools];
    roots.extend(request.runtime.iter().map(|input| Path::new(&input.path)));
    for root in &roots {
        files::tool_links(root, &roots)?;
    }
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
    if shell.node(&[
        "--eval",
        "process.stdout.write(process.versions.node + '\\n' + process.platform + '-' + process.arch)",
    ])? != format!("{NODE_VERSION}\n{}", request.platform)
        || shell.pnpm_version()? != PNPM_VERSION
    {
        return Err("pinned Node/pnpm version/platform mismatch".into());
    }
    // pnpm v9 indexes tarballs by SRI, independent of registry locator. Local archives
    // seed that native index without inventing a resolver or modifying the wanted lock.
    for package in &request.packages {
        let archive = files::child(mirror, &package.archive)?;
        shell.pnpm(&["store", "add", "--offline", archive.to_str().ok_or("non-UTF8 archive")?])?;
    }
    shell.pnpm(&[
        "install",
        "--offline",
        "--frozen-lockfile",
        "--ignore-scripts",
        "--strict-peer-dependencies",
    ])?;
    validate_installed_links(&work, &work)?;
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
    if files::bytes(&work.join("pnpm-lock.yaml"))? != lock_before {
        return Err("install mutated frozen lock".into());
    }
    fs::create_dir(output)?;
    let mut outputs = Vec::new();
    for target in &request.targets {
        for member in target.outputs.iter().chain(target.package_output.iter()) {
            let relative = format!("{}/{member}", target.workspace);
            let from = files::child(&work, &relative)?;
            if !fs::canonicalize(&from)?.starts_with(&work) {
                return Err("declared output escapes workspace".into());
            }
            let digest = files::tree(&from)?;
            files::reject_links(&from)?;
            files::copy(&from, &files::child(output, &relative)?)?;
            outputs.push(json!({"path": relative, "blake3": digest}));
        }
    }
    files::write_json(
        &output.join("provenance.json"),
        &json!({
            "schema": "mantle-pnpm-output-v1", "identity_blake3": identity,
            "inputs": request, "sources": sources, "outputs": outputs,
            "execution": "pnpm-inside-mantle-network-denied-derivation",
            "non_claim": "This helper is not sandbox authority; only the enclosing signed Mantle build receipt establishes sandbox execution and trusted reuse."
        }),
    )?;
    Ok(())
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
fn validate_installed_links(root: &Path, path: &Path) -> Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let metadata = fs::symlink_metadata(entry.path())?;
        if metadata.file_type().is_symlink() {
            let resolved = fs::canonicalize(entry.path())?;
            if !resolved.starts_with(root) {
                return Err("installed workspace link escapes source".into());
            }
        } else if metadata.is_dir() {
            validate_installed_links(root, &entry.path())?;
        }
    }
    Ok(())
}
