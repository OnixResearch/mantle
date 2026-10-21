//! Genuine auxiliary BusyBox compiled from the admitted original source.
//! The bounded bootstrap profile is not the selected final BusyBox derivation.
use std::collections::BTreeMap;

use super::MAKE_CONTROL_HEADER_MEMBER;
use super::MAKE_CONTROL_MEMBER;
use super::MAKE_RECIPE_MEMBER;
use super::archive;
use super::archive_root;
use super::fail;
use super::input;
use super::lit;
use super::original_block;
use super::output;
use super::produced;
use super::root::OriginalShellNode;
use super::root::lower_original_shell_node;
use super::source;
use crate::errors::RunError;
use crate::source_built_native_programs::NativeNodeRequest;
use crate::source_built_recipe::ArtifactKind;
use crate::source_built_recipe::ArtifactRef;
use crate::source_built_recipe::RecipeExitPolicy;
use crate::source_built_recipe::RecipeOperation;
use crate::source_built_recipe::RecipeOutput;
use crate::source_built_recipe::RecipeStep;
use crate::source_built_recipe::RecipeWord;
use crate::source_built_recipe::SourceRecipe;
use crate::source_built_recipe::WordPart;
type Files = BTreeMap<String, ArtifactRef>;
fn predecessor(request: &NativeNodeRequest<'_, '_>, key: &str) -> Result<ArtifactRef, RunError> {
    match request.request.predecessor_exports.get(key) {
        Some(reference @ ArtifactRef::Produced { .. }) => Ok(reference.clone()),
        _ => Err(fail(format!("missing actual bootstrap predecessor {key}"))),
    }
}
fn flag(prefix: &str, name: &str, path: &str) -> RecipeWord {
    RecipeWord {
        parts: vec![WordPart::Literal { text: prefix.into() }, WordPart::Input {
            name: name.into(),
            path: path.into(),
        }],
    }
}
fn add(
    recipe: &mut SourceRecipe,
    id: String,
    inputs: Files,
    kind: ArtifactKind,
    arguments: Vec<RecipeWord>,
    executable: &str,
    limit: u64,
) -> ArtifactRef {
    recipe.steps.push(RecipeStep {
        id: id.clone(),
        inputs,
        outputs: BTreeMap::from([("result".into(), RecipeOutput { kind, bytes_max: limit })]),
        operation: RecipeOperation::Exec {
            executable_input: executable.into(),
            arguments,
            environment: BTreeMap::new(),
            runtime_mounts: BTreeMap::new(),
            cwd_input: None,
            cwd_output: None,
            output_seeds: BTreeMap::new(),
            command_state: None,
            guard: None,
            stdin_input: None,
            merge_stderr: false,
            invocation_input: None,
            exit_policy: RecipeExitPolicy::Exact { code: 0 },
        },
    });
    produced(&id, "result")
}
fn tree(recipe: &mut SourceRecipe, id: String, entries: Files, limit: u64) -> ArtifactRef {
    let mut inputs = Files::new();
    let mut paths = BTreeMap::new();
    for (index, (path, reference)) in entries.into_iter().enumerate() {
        let name = format!("entry-{index}");
        inputs.insert(name.clone(), reference);
        paths.insert(path, name);
    }
    recipe.steps.push(RecipeStep {
        id: id.clone(),
        inputs,
        outputs: BTreeMap::from([("result".into(), RecipeOutput {
            kind: ArtifactKind::Tree,
            bytes_max: limit,
        })]),
        operation: RecipeOperation::AssembleTree {
            output: "result".into(),
            entries: paths,
        },
    });
    produced(&id, "result")
}
fn tcc(
    request: &NativeNodeRequest<'_, '_>,
    recipe: &mut SourceRecipe,
    id: String,
    unit: Option<(ArtifactRef, &str)>,
    objects: &[ArtifactRef],
    includes: &[&str],
) -> Result<ArtifactRef, RunError> {
    let mut inputs = Files::from([
        ("compiler".into(), predecessor(request, "stagex-provider-publication:tcc")?),
        ("headers".into(), predecessor(request, "stagex-provider-publication:musl.headers")?),
        ("libraries".into(), predecessor(request, "stagex-provider-publication:musl.libraries")?),
        ("runtime".into(), predecessor(request, "stagex-provider-publication:tcc.runtime")?),
    ]);
    let mut args = vec![
        flag("-B", "runtime", ""),
        lit("-nostdinc"),
        flag("-I", "runtime", "include"),
        flag("-I", "headers", ""),
        lit("-D_GNU_SOURCE"),
        lit("-DHAVE_CONFIG_H"),
        lit("-include"),
        lit("stddef.h"),
    ];
    let compile = unit.is_some();
    if let Some((reference, path)) = unit {
        inputs.insert("source".into(), reference);
        for directory in includes {
            args.push(flag("-I", "source", directory));
        }
        args.extend([lit("-c"), input("source", path)]);
    } else {
        args.extend([
            lit("-nostdlib"),
            lit("-static"),
            input("libraries", "crt1.o"),
            input("libraries", "crti.o"),
        ]);
        for (index, reference) in objects.iter().enumerate() {
            let name = format!("object-{index}");
            inputs.insert(name.clone(), reference.clone());
            args.push(input(&name, ""));
        }
        args.extend([
            input("libraries", "libc.a"),
            input("runtime", "libtcc1.a"),
            input("libraries", "crtn.o"),
        ]);
    }
    args.extend([lit("-o"), output("result")]);
    Ok(add(
        recipe,
        id,
        inputs,
        if compile {
            ArtifactKind::Object
        } else {
            ArtifactKind::Executable
        },
        args,
        "compiler",
        request.request.proof.resource_bounds.disk_bytes_max.min(64 * 1024 * 1024),
    ))
}
fn append_make(
    request: &NativeNodeRequest<'_, '_>,
    recipe: &mut SourceRecipe,
    projector: &ArtifactRef,
    files: &mut BTreeMap<String, String>,
) -> Result<ArtifactRef, RunError> {
    let archive = archive(request, "-make-4.4.1-full-source-src")?;
    let (_, root) = archive_root(archive, "make-4.4.1", "src/main.c")?;
    files.insert(MAKE_RECIPE_MEMBER.into(), include_str!("../bootstrap/make-4.4.1-gcc10.ncl").into());
    files.insert(
        MAKE_CONTROL_HEADER_MEMBER.into(),
        include_str!("source_built_native_programs_host_make_control.h").into(),
    );
    let mut control = crate::source_built_stagex_programs::shellvm::library_source()?;
    control.push_str(&include_str!("source_built_native_programs_host_make_control.c").replace(
        "#include \"mantle_make_control.h\"",
        include_str!("source_built_native_programs_host_make_control.h"),
    ));
    files.insert(MAKE_CONTROL_MEMBER.into(), control);
    let base = format!("{}.host.bootstrap-make", request.id);
    let limit = request.request.proof.resource_bounds.disk_bytes_max;
    let configured = add(
        recipe,
        format!("{base}.source"),
        Files::from([
            ("tool".into(), projector.clone()),
            ("archive".into(), archive.artifact.clone()),
            ("recipe".into(), source(MAKE_RECIPE_MEMBER)),
            ("control".into(), source(MAKE_CONTROL_HEADER_MEMBER)),
            ("unit".into(), source(MAKE_CONTROL_MEMBER)),
        ]),
        ArtifactKind::Tree,
        vec![
            lit("make-source"),
            input("archive", &root),
            input("recipe", ""),
            input("control", ""),
            input("unit", ""),
            output("result"),
        ],
        "tool",
        limit,
    );
    let mut sources = original_block(include_str!("../bootstrap/make-4.4.1-gcc10.ncl"), "SRCS=\"", "\"\n")?
        .split_whitespace()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    sources.sort();
    let mut objects = Vec::new();
    for (index, path) in sources.iter().enumerate() {
        objects.push(tcc(request, recipe, format!("{base}.unit-{index}"), Some((configured.clone(), path)), &[], &[
            "src", "lib", "",
        ])?);
    }
    let stubs = add(
        recipe,
        format!("{base}.stubs"),
        Files::from([
            ("tool".into(), projector.clone()),
            ("recipe".into(), source(MAKE_RECIPE_MEMBER)),
        ]),
        ArtifactKind::Text,
        vec![
            lit("block"),
            input("recipe", ""),
            lit("<<'STUBEOF'\n"),
            lit("\nSTUBEOF"),
            output("result"),
        ],
        "tool",
        limit,
    );
    // These two units do not include config.h; their flags retain the original
    // stub/control semantics rather than inventing a configured source tree.
    objects.push(tcc(request, recipe, format!("{base}.stub-object"), Some((stubs, "")), &[], &[])?);
    objects.push(tcc(
        request,
        recipe,
        format!("{base}.control-object"),
        Some((source(MAKE_CONTROL_MEMBER), "")),
        &[],
        &[],
    )?);
    tcc(request, recipe, format!("{base}.link"), None, &objects, &[])
}
fn append_usage_compressor(
    request: &NativeNodeRequest<'_, '_>,
    recipe: &mut SourceRecipe,
    files: &mut BTreeMap<String, String>,
) -> Result<ArtifactRef, RunError> {
    const DRIVER: &str = "source-built/native/host-bzip2.c";
    const HEADER: &str = "source-built/native/host-bzip2-libc.h";
    files.insert(DRIVER.into(), include_str!("source_built_native_programs_host_bzip2.c").into());
    files.insert(HEADER.into(), include_str!("source_built_native_programs_host_bzip2_libc.h").into());
    let archive = archive(request, "-busybox-1.37.0-full-source-src")?;
    let (_, member) = archive_root(archive, "busybox-1.37.0", "Makefile")?;
    let base = format!("{}.host.bootstrap-bzip2", request.id);
    let combined = tree(
        recipe,
        format!("{base}.source"),
        Files::from([
            ("busybox".into(), archive.artifact.clone()),
            ("driver.c".into(), source(DRIVER)),
            ("libbb.h".into(), source(HEADER)),
        ]),
        request.request.proof.resource_bounds.disk_bytes_max,
    );
    let root = if member.is_empty() {
        "busybox".into()
    } else {
        format!("busybox/{member}")
    };
    let object = tcc(request, recipe, format!("{base}.compile"), Some((combined, "driver.c")), &[], &["", &root])?;
    tcc(request, recipe, format!("{base}.link"), None, &[object], &[])
}
const COMPRESSION_PROFILE: &str = "CONFIG_BZIP2=y\nCONFIG_BZIP2_SMALL=8\nCONFIG_BUNZIP2=y\nCONFIG_BZCAT=y\nCONFIG_FEATURE_BZIP2_DECOMPRESS=y\nCONFIG_GZIP=y\nCONFIG_GUNZIP=y\nCONFIG_ZCAT=y\n";
const PROFILE: &str = "CONFIG_STATIC=y\nCONFIG_DESKTOP=y\nCONFIG_LONG_OPTS=y\nCONFIG_FEATURE_VERBOSE_USAGE=y\nCONFIG_FEATURE_PREFER_APPLETS=y\nCONFIG_ASH=y\nCONFIG_SH_IS_ASH=y\nCONFIG_ASH_BASH_COMPAT=y\nCONFIG_ASH_JOB_CONTROL=y\nCONFIG_ASH_ALIAS=y\nCONFIG_ASH_MATH_SUPPORT=y\nCONFIG_ASH_MATH_SUPPORT_64=y\nCONFIG_ASH_ECHO=y\nCONFIG_ASH_PRINTF=y\nCONFIG_ASH_TEST=y\nCONFIG_ASH_GETOPTS=y\nCONFIG_ASH_CMDCMD=y\nCONFIG_AWK=y\nCONFIG_FEATURE_AWK_LIBM=y\nCONFIG_FEATURE_AWK_GNU_EXTENSIONS=y\nCONFIG_SED=y\nCONFIG_GREP=y\nCONFIG_EGREP=y\nCONFIG_FGREP=y\nCONFIG_FEATURE_GREP_CONTEXT=y\nCONFIG_FIND=y\nCONFIG_FEATURE_FIND_PRINT0=y\nCONFIG_FEATURE_FIND_TYPE=y\nCONFIG_FEATURE_FIND_NAME=y\nCONFIG_FEATURE_FIND_PATH=y\nCONFIG_FEATURE_FIND_REGEX=y\nCONFIG_FEATURE_FIND_PAREN=y\nCONFIG_FEATURE_FIND_NOT=y\nCONFIG_FEATURE_FIND_PRUNE=y\nCONFIG_FEATURE_FIND_EXEC=y\nCONFIG_XARGS=y\nCONFIG_FEATURE_XARGS_SUPPORT_QUOTES=y\nCONFIG_FEATURE_XARGS_SUPPORT_ZERO_TERM=y\nCONFIG_FEATURE_XARGS_SUPPORT_REPL_STR=y\nCONFIG_BASENAME=y\nCONFIG_CAT=y\nCONFIG_CHMOD=y\nCONFIG_CMP=y\nCONFIG_COMM=y\nCONFIG_CP=y\nCONFIG_FEATURE_CP_LONG_OPTIONS=y\nCONFIG_CUT=y\nCONFIG_DATE=y\nCONFIG_DD=y\nCONFIG_DIRNAME=y\nCONFIG_ECHO=y\nCONFIG_ENV=y\nCONFIG_EXPR=y\nCONFIG_EXPR_MATH_SUPPORT_64=y\nCONFIG_FALSE=y\nCONFIG_HEAD=y\nCONFIG_FEATURE_FANCY_HEAD=y\nCONFIG_HOSTNAME=y\nCONFIG_ID=y\nCONFIG_INSTALL=y\nCONFIG_LN=y\nCONFIG_LS=y\nCONFIG_FEATURE_LS_FILETYPES=y\nCONFIG_FEATURE_LS_SORTFILES=y\nCONFIG_FEATURE_LS_TIMESTAMPS=y\nCONFIG_MKDIR=y\nCONFIG_MKTEMP=y\nCONFIG_MV=y\nCONFIG_OD=y\nCONFIG_PASTE=y\nCONFIG_PRINTENV=y\nCONFIG_PRINTF=y\nCONFIG_PWD=y\nCONFIG_READLINK=y\nCONFIG_FEATURE_READLINK_FOLLOW=y\nCONFIG_REALPATH=y\nCONFIG_RM=y\nCONFIG_RMDIR=y\nCONFIG_SLEEP=y\nCONFIG_FEATURE_FANCY_SLEEP=y\nCONFIG_SORT=y\nCONFIG_FEATURE_SORT_BIG=y\nCONFIG_SPLIT=y\nCONFIG_TAIL=y\nCONFIG_FEATURE_FANCY_TAIL=y\nCONFIG_TEE=y\nCONFIG_TEST=y\nCONFIG_TEST1=y\nCONFIG_TEST2=y\nCONFIG_TOUCH=y\nCONFIG_FEATURE_TOUCH_NODEREF=y\nCONFIG_TR=y\nCONFIG_FEATURE_TR_CLASSES=y\nCONFIG_FEATURE_TR_EQUIV=y\nCONFIG_TRUE=y\nCONFIG_UNAME=y\nCONFIG_UNIQ=y\nCONFIG_WC=y\nCONFIG_WHICH=y\nCONFIG_WHOAMI=y\nCONFIG_TIMEOUT=y\nCONFIG_FEATURE_INSTALLER=y\n";
/// Attach a real source build, returning its exact original consumer prerequisites.
pub(crate) fn append(
    request: &NativeNodeRequest<'_, '_>,
    preparation: &mut SourceRecipe,
    controller: &ArtifactRef,
    projector: &ArtifactRef,
    tools: &[(String, ArtifactRef)],
    files: &mut BTreeMap<String, String>,
) -> Result<(ArtifactRef, ArtifactRef), RunError> {
    let source_archive = archive(request, "-busybox-1.37.0-full-source-src")?;
    let (_, member) = archive_root(source_archive, "busybox-1.37.0", "Makefile")?;
    let make = append_make(request, preparation, projector, files)?;
    let compressor = append_usage_compressor(request, preparation, files)?;
    let mut entries = Files::new();
    for (key, reference) in request.request.predecessor_exports {
        if let Some(path) = key.strip_prefix("stagex-transition:member:")
            && path.contains("/bin/")
        {
            let name = path.rsplit('/').next().unwrap_or(path);
            entries.entry(name.into()).or_insert_with(|| reference.clone());
        }
    }
    for (name, reference) in [
        ("make", make.clone()),
        ("bzip2", compressor.clone()),
        ("tcc", predecessor(request, "stagex-transition:phase-tool:tcc-musl-v2")?),
    ] {
        entries.insert(name.into(), reference);
    }
    let base = format!("{}.host.bootstrap-busybox", request.id);
    let limit = request.request.proof.resource_bounds.disk_bytes_max;
    let tools_tree = tree(preparation, format!("{base}.tools"), entries, limit);
    let mut command_inputs = Files::from([
        ("bootstrap-tools".into(), tools_tree),
        ("bootstrap-source".into(), source_archive.artifact.clone()),
    ]);
    let mut runtime_mounts = BTreeMap::from([("/__mantle_native_bootstrap_tools".into(), "bootstrap-tools".into())]);
    let source_is_direct = request.dependencies.iter().any(|(path, program)| {
        program
            .outputs
            .keys()
            .any(|name| request.logical_outputs.get(&format!("{path}:{name}")) == Some(&source_archive.original_output))
    });
    if !source_is_direct {
        runtime_mounts.insert(source_archive.original_output.clone(), "bootstrap-source".into());
    } else {
        command_inputs.remove("bootstrap-source");
    }
    for (key, role) in [
        ("bootstrap-headers", "musl.headers"),
        ("bootstrap-libraries", "musl.libraries"),
        ("bootstrap-runtime", "tcc.runtime"),
    ] {
        command_inputs.insert(key.into(), predecessor(request, &format!("stagex-provider-publication:{role}"))?);
        runtime_mounts.insert(format!("/__mantle_native_{key}"), key.into());
    }
    let root = if member.is_empty() {
        source_archive.original_output.clone()
    } else {
        format!("{}/{member}", source_archive.original_output)
    };
    let mut script = format!(
        r#"set -eu
export PATH=/__mantle_native_bootstrap_tools
WORK=/tmp/mantle-native-bootstrap-busybox
mkdir -p "$WORK/src"
cp -R '{root}/.' "$WORK/src/"
chmod -R u+w "$WORK/src"
cd "$WORK/src"
cat >"$WORK/profile" <<'MANTLE_PROFILE'
{COMPRESSION_PROFILE}{PROFILE}MANTLE_PROFILE
cat >"$WORK/tcc" <<'MANTLE_TCC'
#!/bin/sh
link=yes
for option in "$@"; do
 case "$option" in -c|-E|-S|-v|-dumpversion|-dumpmachine|-print-*) link=no ;; esac
done
if test "$link" = no; then
 exec /__mantle_native_bootstrap_tools/tcc -B/__mantle_native_bootstrap-runtime -nostdinc -I/__mantle_native_bootstrap-runtime/include -I/__mantle_native_bootstrap-headers "$@"
fi
exec /__mantle_native_bootstrap_tools/tcc -B/__mantle_native_bootstrap-runtime -nostdinc -I/__mantle_native_bootstrap-runtime/include -I/__mantle_native_bootstrap-headers -L/__mantle_native_bootstrap-libraries -nostdlib -static /__mantle_native_bootstrap-libraries/crt1.o /__mantle_native_bootstrap-libraries/crti.o "$@" /__mantle_native_bootstrap-libraries/libc.a /__mantle_native_bootstrap-runtime/libtcc1.a /__mantle_native_bootstrap-libraries/crtn.o
MANTLE_TCC
chmod 755 "$WORK/tcc"
export CC="$WORK/tcc" HOSTCC="$WORK/tcc" AR=/__mantle_native_bootstrap_tools/ar
export CONFIG_SHELL=/bin/sh SHELL=/bin/sh LC_ALL=C SOURCE_DATE_EPOCH=1 KCONFIG_NOTIMESTAMP=1
KCONFIG_ALLCONFIG="$WORK/profile" make allnoconfig CC="$CC" HOSTCC="$HOSTCC"
make -j1 CC="$CC" HOSTCC="$HOSTCC" HOSTLDFLAGS=-static
test -x busybox
mkdir -p "$out"
cp busybox "$out/busybox"
chmod 755 "$out/busybox"
"#
    );
    // Source scripts, not guessed configured headers, choose and generate the
    // exact selected object graph. Original conf handles profile dependencies.
    script.push_str("\"$out/busybox\" --list >\"$WORK/applets\"\n");
    let mut inventory = tools.to_vec();
    inventory.push(("make".into(), make));
    inventory.push(("bzip2".into(), compressor));
    inventory.push(("tcc".into(), predecessor(request, "stagex-transition:phase-tool:tcc-musl-v2")?));
    let remaining = crate::source_built_recipe::STEP_COUNT_MAX
        .checked_sub(preparation.steps.len())
        .ok_or_else(|| fail("bootstrap preparation exceeds admitted declaration capacity"))?;
    let commands_max = crate::source_built_recipe::controller::reserve_native_commands(
        request.reservations,
        &format!("{}.host.original.bootstrap-busybox.controller", request.id),
        source_archive.native_source_files,
        remaining,
    )?;
    let mut program = lower_original_shell_node(request, OriginalShellNode {
        namespace: ".bootstrap-busybox".into(),
        script: Some(script),
        controller: controller.clone(),
        projector: projector.clone(),
        tools: inventory,
        command_inputs,
        runtime_mounts,
        source_overrides: BTreeMap::new(),
        members: BTreeMap::from([("out".into(), BTreeMap::from([("busybox".into(), ArtifactKind::Executable)]))]),
        phase_members: BTreeMap::new(),
        steps: std::mem::take(&mut preparation.steps),
        timeout_groups: std::mem::take(&mut preparation.timeout_groups),
        files: std::mem::take(files),
        commands_max,
    })?;
    let image = program
        .members
        .get("out")
        .and_then(|m| m.get("busybox"))
        .cloned()
        .ok_or_else(|| fail("bootstrap original BusyBox image missing"))?;
    let root = program
        .outputs
        .get("out")
        .cloned()
        .ok_or_else(|| fail("bootstrap original BusyBox namespace missing"))?;
    preparation.steps.append(&mut program.steps);
    preparation.timeout_groups.append(&mut program.timeout_groups);
    files.append(&mut program.files);
    Ok((image, root))
}
