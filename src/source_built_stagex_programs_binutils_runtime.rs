//! Original binutils configure, Make, install and smoke recipe lowering.
//!
//! The controller interprets source scripts; the immutable catalog below is the
//! only authority for a real child image. Configure answers and Make output are
//! never computed by the recipe author.
use std::collections::BTreeMap;

use super::Author;
use super::FILE_BYTES_MAX;
use super::Files;
use super::GENERATORS;
use super::HELPERS;
use super::NativeHelpers;
use super::TREE_BYTES_MAX;
use super::error;
use crate::errors::RunError;
use crate::source_built_program_authoring::AuthoringRequest;
use crate::source_built_recipe::ArtifactKind;
use crate::source_built_recipe::ArtifactRef;
use crate::source_built_recipe::RecipeExitPolicy;
use crate::source_built_recipe::RecipeOperation;
use crate::source_built_recipe::RecipeOutput;
use crate::source_built_recipe::RecipeStep;
use crate::source_built_recipe::RecipeWord;
use crate::source_built_stagex_programs::controller::ControllerExecutable;
use crate::source_built_stagex_programs::controller::ControllerSlot;
use crate::source_built_stagex_programs::controller::ControllerSpec;
use crate::source_built_stagex_programs::input;
use crate::source_built_stagex_programs::literal;
use crate::source_built_stagex_programs::output;
use crate::stagex_transition::BINUTILS_COMPONENT_BUILD_STAGE_ID as COMPONENTS;
use crate::stagex_transition::BINUTILS_INSTALL_SMOKE_STAGE_ID as SMOKE;
use crate::stagex_transition::BINUTILS_INSTALL_STAGE_ID as INSTALL;

const ORIGINAL: &str = include_str!("stagex_binutils.rs");
const ROOT: &str = "/mantle/stagex/transition";
const RUNTIME: &str = "/mantle/stagex/transition/binutils-stage/runtime";
const RELATIVE: &str = "transition/binutils-stage/runtime";
const PREFIX: &str = "/mantle/stagex/binutils-probe-output";
const INSTALLED: &str = "transition/binutils-stage/runtime/install-destdir/mantle/stagex/binutils-probe-output";

fn netstring(to: &mut String, value: &str) {
    to.push_str(&format!("{}:{},", value.len(), value));
}
fn quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
fn dependency(stage: &str, member: &str) -> String {
    format!("{ROOT}/{stage}-stage/runtime/output/{member}")
}
fn relative(path: &str) -> Result<String, RunError> {
    path.strip_prefix("/mantle/stagex/")
        .map(str::to_owned)
        .ok_or_else(|| error("binutils initial path escaped its explicit logical root"))
}
fn strings(source: &str) -> Result<Vec<String>, RunError> {
    let mut result = Vec::new();
    let mut position = 0;
    while let Some(start) = source[position..].find('"') {
        let start = position + start;
        let mut end = start + 1;
        let mut escaped = false;
        for byte in source.as_bytes()[end..].iter() {
            end += 1;
            if *byte == b'"' && !escaped {
                break;
            }
            escaped = *byte == b'\\' && !escaped;
        }
        result.push(
            serde_json::from_str(&source[start..end]).map_err(|e| error(format!("binutils source literal: {e}")))?,
        );
        position = end;
    }
    Ok(result)
}
fn array(name: &str) -> Result<Vec<String>, RunError> {
    let start = ORIGINAL
        .split_once(&format!("const {name}:"))
        .ok_or_else(|| error(format!("missing original binutils inventory {name}")))?
        .1;
    let body = start.split_once(" = ").ok_or_else(|| error("original binutils array value"))?.1;
    let body = body
        .split_once("]; ")
        .map(|p| p.0)
        .or_else(|| body.split_once("];\n").map(|p| p.0))
        .ok_or_else(|| error("unterminated original binutils inventory"))?;
    strings(body)
}
fn bytes(name: &str) -> Result<String, RunError> {
    let start = ORIGINAL
        .split_once(&format!("const {name}:"))
        .ok_or_else(|| error(format!("missing original binutils smoke source {name}")))?
        .1;
    let body = start.split_once(" =").ok_or_else(|| error("original binutils source value"))?.1;
    let first = body.find('"').ok_or_else(|| error("missing binutils smoke literal"))?;
    let mut end = first + 1;
    let mut escaped = false;
    for byte in body.as_bytes()[end..].iter() {
        end += 1;
        if *byte == b'"' && !escaped {
            break;
        }
        escaped = *byte == b'\\' && !escaped;
    }
    serde_json::from_str(&body[first..end]).map_err(|e| error(format!("original binutils smoke literal: {e}")))
}
fn authored(author: &mut Author<'_>, name: &str, text: String) -> Result<ArtifactRef, RunError> {
    author.builder.authored(&format!("helpers/binutils-runtime/{name}"), text)
}
fn project(
    author: &mut Author<'_>,
    native: &NativeHelpers,
    tool: &str,
    id: &str,
    tree: ArtifactRef,
    path: &str,
    kind: ArtifactKind,
) -> Result<ArtifactRef, RunError> {
    author.exec(
        tool,
        id,
        native.transform.clone(),
        Files::from([("tree".into(), tree)]),
        vec![
            literal(if kind == ArtifactKind::Tree { "tree" } else { "copy" }),
            input("tree", path),
            output("result", ""),
        ],
        kind,
    )
}
fn installed(
    author: &mut Author<'_>,
    controller: &ArtifactRef,
    tree: &ArtifactRef,
    generated: &ArtifactRef,
    tool: &str,
    id: &str,
    path: &str,
) -> Result<ArtifactRef, RunError> {
    author.exec(
        tool,
        id,
        controller.clone(),
        Files::from([("tree".into(), tree.clone()), ("catalog".into(), generated.clone())]),
        vec![
            literal("--mantle-native-installed"),
            input("tree", path),
            input("catalog", ""),
            output("result", ""),
        ],
        ArtifactKind::Executable,
    )
}

fn utility_variant(author: &mut Author<'_>, native: &NativeHelpers) -> Result<ArtifactRef, RunError> {
    let source = author.builder.source("stagex-configure-utility-source")?;
    let headers = author.builder.artifact("musl-native-headers")?;
    let compiler = author.builder.artifact("tcc-musl-v2")?;
    let object = author.exec(
        HELPERS,
        "binutils-configure-utility-command-compile",
        compiler,
        Files::from([("source".into(), source), ("headers".into(), headers)]),
        vec![
            literal("-c"),
            literal("-I"),
            input("headers", ""),
            input("source", ""),
            literal("-o"),
            output("result", ""),
        ],
        ArtifactKind::Object,
    )?;
    author.link(
        HELPERS,
        "binutils-configure-utility-command-link",
        &[object, native.runtime.clone(), native.assembly.clone()],
        false,
    )
}

fn wrapper() -> Result<String, RunError> {
    let mut script = super::dispatch::tcc_wrapper_template()?.to_owned();
    for (marker, value) in [
        ("@TCC@", dependency("tcc-musl-v2", "bin/tcc-0.9.27-musl-v2")),
        ("@MUSL_INCLUDE@", dependency("musl-native", "include")),
        ("@MUSL_LIB@", dependency("musl-native", "lib")),
        ("@RUNTIME@", format!("{RUNTIME}/binutils-runtime.o")),
        ("@RUNTIME_ASM@", format!("{RUNTIME}/binutils-runtime-asm.o")),
        ("@CANONICALIZER@", format!("{RUNTIME}/stagex-elf-local-symbol-canonicalizer")),
        ("@CHMOD@", dependency("coreutils", "bin/chmod")),
        ("@CAT@", dependency("coreutils", "bin/cat")),
        ("@RM@", dependency("coreutils", "bin/rm")),
        ("@WC@", dependency("coreutils", "bin/wc")),
        ("@GREP@", dependency("grep", "bin/grep")),
        ("@SED@", dependency("sed", "bin/sed")),
        ("@SOURCE_BYTES_MAX@", "65536".into()),
        ("@INVOCATION_COUNT_MAX@", "4096".into()),
    ] {
        if !script.contains(marker) {
            return Err(error(format!("binutils wrapper binding drift {marker}")));
        }
        script = script.replace(marker, &value);
    }
    Ok(script)
}

fn environment() -> BTreeMap<String, String> {
    let bash = dependency("bash-full", "bin/bash-full");
    let compiler = format!("{bash} {RUNTIME}/binutils-tcc-wrapper.sh");
    let mut env = BTreeMap::from([
        ("PATH".into(), format!("{RUNTIME}/tools")),
        ("CONFIG_SHELL".into(), bash.clone()),
        ("SHELL".into(), bash.clone()),
        ("CC".into(), compiler.clone()),
        ("CC_FOR_BUILD".into(), compiler.clone()),
        ("CPP".into(), format!("{compiler} -E")),
        ("AR".into(), format!("{bash} {RUNTIME}/stagex-binutils-ar-runner.sh")),
        ("RANLIB".into(), "true".into()),
        ("LD".into(), "true".into()),
        ("MAKEINFO".into(), "true".into()),
        ("AWK".into(), dependency("gawk", "bin/gawk")),
        ("M4".into(), dependency("m4", "bin/m4")),
        (
            "CFLAGS".into(),
            format!("-I{} {}", dependency("musl-native", "include"), crate::stagex_binutils::source_owned_cflags()),
        ),
        (
            "CFLAGS_FOR_BUILD".into(),
            format!("-I{} {}", dependency("musl-native", "include"), crate::stagex_binutils::source_owned_cflags()),
        ),
        ("LDFLAGS".into(), format!("-static -L{}", dependency("musl-native", "lib"))),
        ("MANTLE_STAGE_X_FILE".into(), format!("{RUNTIME}/tools/file")),
        ("MANTLE_STAGE_X_SED_BRIDGE_BASH".into(), bash),
        ("MANTLE_STAGE_X_SED_BRIDGE_SCRIPT".into(), format!("{RUNTIME}/stagex-sed-regular-file-bridge.sh")),
        ("MANTLE_STAGE_X_SED_BRIDGE_TARGET".into(), dependency("sed", "bin/sed")),
        ("MANTLE_STAGE_X_SED_BRIDGE_LAUNCHER".into(), format!("{RUNTIME}/tools/sed")),
        ("MANTLE_STAGE_X_YLWRAP_SED".into(), format!("{RUNTIME}/stagex-ylwrap-sed-runner")),
        ("MANTLE_STAGE_X_SED_BRIDGE_EMIT".into(), format!("{RUNTIME}/tools/emit")),
        ("MANTLE_STAGE_X_SED_SPOOL_ROOT".into(), format!("{RUNTIME}/sed-spool")),
        ("MANTLE_STAGE_X_SED_BRIDGE_AUDIT".into(), format!("{RUNTIME}/sed-spool/audit.tsv")),
        ("MANTLE_STAGE_X_AR_AUDIT".into(), format!("{RUNTIME}/ar-audit.tsv")),
        ("MANTLE_STAGE_X_AR_COUNT".into(), format!("{RUNTIME}/ar-count.txt")),
        ("MANTLE_STAGE_X_AR_SCRATCH".into(), format!("{RUNTIME}/ar-scratch")),
        ("MANTLE_BINUTILS_CONFIGURE_PROBE_AUDIT".into(), format!("{RUNTIME}/preprocess.audit")),
        ("MANTLE_BINUTILS_CONFIGURE_PROBE_COUNT".into(), format!("{RUNTIME}/preprocess.count")),
        ("MANTLE_STAGE_X_BISON".into(), dependency("bison", "bin/bison")),
        ("MANTLE_STAGE_X_FLEX".into(), dependency("flex", "bin/flex")),
        ("MANTLE_CONTROLLER_ROOTS".into(), "source-owned-manifest".into()),
    ]);
    for (name, command) in [
        ("CAT", "cat"),
        ("HEAD", "head"),
        ("MKDIR", "mkdir"),
        ("RM", "rm"),
        ("RMDIR", "rmdir"),
        ("WC", "wc"),
    ] {
        env.insert(format!("MANTLE_STAGE_X_SED_BRIDGE_{name}"), dependency("coreutils", &format!("bin/{command}")));
    }
    for (name, command) in [
        ("BASENAME", "basename"),
        ("CAT", "cat"),
        ("MV", "mv"),
        ("RM", "rm"),
        ("WC", "wc"),
    ] {
        env.insert(format!("MANTLE_STAGE_X_AR_{name}"), dependency("coreutils", &format!("bin/{command}")));
    }
    env
}

fn smoke_inputs(author: &mut Author<'_>, initial: &mut Files) -> Result<(), RunError> {
    for (path, text) in [
        (
            "elf-canonicalizer-smoke.c",
            "static const char value[] = \"anonymous-symbol\"; int value_at(unsigned i) { return value[i]; }\n".into(),
        ),
        ("elf-canonicalizer-malformed.o", "not-an-elf\n".into()),
        ("sed-bridge-smoke/input.txt", bytes("SED_BRIDGE_SMOKE_INPUT")?),
        ("sed-bridge-smoke/first.sed", bytes("SED_BRIDGE_FIRST_PROGRAM")?),
        ("sed-bridge-smoke/second.sed", bytes("SED_BRIDGE_SECOND_PROGRAM")?),
        ("sed-bridge-smoke/expected.txt", bytes("SED_BRIDGE_SMOKE_EXPECTED")?),
        ("ylwrap-sed-smokes/ylwrap2/y.tab.c", "x".repeat(8 * 1024 * 1024 + 1)),
    ] {
        initial.insert(format!("{RELATIVE}/{path}"), authored(author, path, text)?);
    }
    for label in [
        "missing-authority",
        "wrong-directory",
        "non-conftest",
        "unknown-class",
        "oversized-source",
        "exhausted-budget",
    ] {
        let name = if label == "non-conftest" {
            "probe.c"
        } else {
            "conftest.c"
        };
        let text = if label == "oversized-source" {
            "x".repeat(65537)
        } else {
            "int mantle_probe;\n".into()
        };
        initial.insert(
            format!("{RELATIVE}/preprocess-negative-{label}/{name}"),
            authored(author, &format!("preprocess-negative-{label}/{name}"), text)?,
        );
    }
    Ok(())
}

fn helper_smokes() -> String {
    let mut script=r###"
@TCC@ -c elf-canonicalizer-smoke.c -o elf-canonicalizer-smoke-a.o
@TCC@ -c elf-canonicalizer-smoke.c -o elf-canonicalizer-smoke-b.o
@CANON@ "$PWD/elf-canonicalizer-smoke-a.o"
@CANON@ "$PWD/elf-canonicalizer-smoke-b.o"
@CMP@ elf-canonicalizer-smoke-a.o elf-canonicalizer-smoke-b.o
set +e
@CANON@ "$PWD/elf-canonicalizer-malformed.o"
rejected=$?
set -e
test "$rejected" -ne 0
for label in missing-authority wrong-directory non-conftest unknown-class oversized-source exhausted-budget; do
  cd @RUNTIME@/preprocess-negative-$label
  source_name=conftest.c
  export MANTLE_BINUTILS_CONFIGURE_PROBE_DIR=$PWD
  export MANTLE_BINUTILS_CONFIGURE_PROBE_CLASS=intl
  export MANTLE_BINUTILS_CONFIGURE_PROBE_AUDIT=$PWD/audit.tsv
  export MANTLE_BINUTILS_CONFIGURE_PROBE_COUNT=$PWD/count.txt
  case "$label" in
    missing-authority) unset MANTLE_BINUTILS_CONFIGURE_PROBE_DIR MANTLE_BINUTILS_CONFIGURE_PROBE_CLASS MANTLE_BINUTILS_CONFIGURE_PROBE_AUDIT MANTLE_BINUTILS_CONFIGURE_PROBE_COUNT; reason=missing-authority-directory ;;
    wrong-directory) export MANTLE_BINUTILS_CONFIGURE_PROBE_DIR=$PWD/declared-directory; reason=current-directory-mismatch ;;
    non-conftest) source_name=probe.c; reason=non-conftest-source ;;
    unknown-class) export MANTLE_BINUTILS_CONFIGURE_PROBE_CLASS=unknown; reason=unknown-configure-class ;;
    oversized-source) reason=source-too-large ;;
    exhausted-budget) printf '4096\n' > count.txt; reason=invocation-budget-exhausted ;;
  esac
  expect_status 1 @BASH@ @RUNTIME@/binutils-tcc-wrapper.sh -E "$source_name" > stdout.txt 2> stderr.txt
  @GREP@ -q "$reason" stderr.txt
  test ! -e audit.tsv
done
cd @RUNTIME@
unset MANTLE_BINUTILS_CONFIGURE_PROBE_DIR MANTLE_BINUTILS_CONFIGURE_PROBE_CLASS
export MANTLE_BINUTILS_CONFIGURE_PROBE_AUDIT=@RUNTIME@/preprocess.audit
export MANTLE_BINUTILS_CONFIGURE_PROBE_COUNT=@RUNTIME@/preprocess.count
@MKDIR@ -p ar-runner-smokes/scratch
cd ar-runner-smokes
printf abc > a.o
printf long-payload > a-very-long-member-name.o
printf occupied > existing.a
@LN@ -s a.o link.o
(
 export MANTLE_STAGE_X_AR_AUDIT=$PWD/audit.tsv MANTLE_STAGE_X_AR_COUNT=$PWD/count.txt MANTLE_STAGE_X_AR_SCRATCH=$PWD/scratch
 @BASH@ @RUNTIME@/stagex-binutils-ar-runner.sh rc libtest.a a.o a-very-long-member-name.o
 expect_status 125 @BASH@ @RUNTIME@/stagex-binutils-ar-runner.sh rc empty.a
 expect_status 125 @BASH@ @RUNTIME@/stagex-binutils-ar-runner.sh rc existing.a a.o
 expect_status 125 @BASH@ @RUNTIME@/stagex-binutils-ar-runner.sh t rejected.a a.o
 expect_status 125 @BASH@ @RUNTIME@/stagex-binutils-ar-runner.sh rc ../escape.a a.o
 expect_status 125 @BASH@ @RUNTIME@/stagex-binutils-ar-runner.sh rc symlink.a link.o
)
cd @RUNTIME@
@LAUNCHER@ --mantle-self-test
(
 unset MANTLE_STAGE_X_SED_BRIDGE_BASH MANTLE_STAGE_X_SED_BRIDGE_SCRIPT MANTLE_STAGE_X_SED_BRIDGE_TARGET MANTLE_STAGE_X_YLWRAP_SED MANTLE_STAGE_X_SED_BRIDGE_CAT MANTLE_STAGE_X_SED_BRIDGE_HEAD MANTLE_STAGE_X_SED_BRIDGE_MKDIR MANTLE_STAGE_X_SED_BRIDGE_RM MANTLE_STAGE_X_SED_BRIDGE_RMDIR MANTLE_STAGE_X_SED_BRIDGE_WC MANTLE_STAGE_X_SED_BRIDGE_EMIT MANTLE_STAGE_X_SED_SPOOL_ROOT MANTLE_STAGE_X_SED_BRIDGE_AUDIT
 expect_status 125 @LAUNCHER@ s/x/y/ < /dev/null
)
@YLWRAP@ --mantle-self-test
@MKDIR@ -p ylwrap-sed-smokes/ylwrap1
cd ylwrap-sed-smokes/ylwrap1
path_program='s|@RUNTIME@/ylwrap-sed-smokes/||'
names='s|y\.tab\.c|arparse.c|g;s|y\.tab\.h|arparse.h|g;s|y\.output|arparse.output|g;'
guards='s|Y_TAB_C|ARPARSE_C|g;s|Y_TAB_H|ARPARSE_H|g;s|Y_OUTPUT|ARPARSE_OUTPUT|g;'
ylrun() { @YLWRAP@ -e '/^#/!b' -e "$path_program" -e "$names" -e "$guards" "$1"; }
printf '#line 1 "%s/arparse.y"\n#include "y.tab.h"\n#ifndef Y_TAB_C\nbody y.tab.c\n' @RUNTIME@/ylwrap-sed-smokes > y.tab.c
ylrun y.tab.c > positive.txt
printf '#line 1 "arparse.y"\n#include "arparse.h"\n#ifndef ARPARSE_C\nbody y.tab.c\n' > expected.txt
@CMP@ positive.txt expected.txt
(
 path_program='s|/substituted/source/||'
 expect_status 125 ylrun y.tab.c > wrong-path.txt
 test ! -s wrong-path.txt
)
@LN@ -s y.tab.c lex.yy.c
(
 names='s|lex\.yy\.c|arlex.c|g;'
 guards='s|LEX_YY_C|ARLEX_C|g;'
 expect_status 125 ylrun lex.yy.c > symlink.txt
 test ! -s symlink.txt
)
cd ..
expect_status 125 ylrun y.tab.c > wrong-directory.txt
test ! -s wrong-directory.txt
cd ylwrap2
expect_status 125 ylrun y.tab.c > oversized.txt
test ! -s oversized.txt
cd @RUNTIME@/sed-bridge-smoke
@LAUNCHER@ -n -f first.sed < input.txt | @LAUNCHER@ -f second.sed > positive.stdout.txt
@CMP@ positive.stdout.txt expected.txt
expect_status 125 @LAUNCHER@ s/LTLIBOBJS/rejected/ "$PWD/input.txt" < input.txt > negative.stdout.txt 2> negative.stderr.txt
@GREP@ -q mixed-input-authority negative.stderr.txt
cd @RUNTIME@
"###.to_owned();
    for (marker, value) in [
        ("@TCC@", dependency("tcc-musl-v2", "bin/tcc-0.9.27-musl-v2")),
        ("@CANON@", format!("{RUNTIME}/stagex-elf-local-symbol-canonicalizer")),
        ("@BASH@", dependency("bash-full", "bin/bash-full")),
        ("@CMP@", dependency("diffutils", "bin/cmp")),
        ("@GREP@", dependency("grep", "bin/grep")),
        ("@MKDIR@", dependency("coreutils", "bin/mkdir")),
        ("@LN@", dependency("coreutils", "bin/ln")),
        ("@YLWRAP@", format!("{RUNTIME}/stagex-ylwrap-sed-runner")),
        ("@LAUNCHER@", format!("{RUNTIME}/stagex-sed-bridge-launcher")),
        ("@RUNTIME@", RUNTIME.into()),
    ] {
        script = script.replace(marker, &value);
    }
    script
}
fn driver(compiler_runtime_root: &str) -> Result<String, RunError> {
    let mut script = format!("set -eu\ncd {}\n", quote(RUNTIME));
    script.push_str("expect_status() { expected=$1; shift; set +e; \"$@\"; actual=$?; set -e; test \"$actual\" -eq \"$expected\"; }\n");
    script.push_str(&format!(
        "{} -p {}/sed-spool {}/ar-scratch {}/install-destdir\n",
        dependency("coreutils", "bin/mkdir"),
        RUNTIME,
        RUNTIME,
        RUNTIME
    ));
    script.push_str(&format!(
        "{} -s {} {RUNTIME}/runtime\n",
        dependency("coreutils", "bin/ln"),
        quote(compiler_runtime_root)
    ));
    for class in array("CONFIGURE_CLASSES")? {
        script.push_str(&format!(
            "{} -s {} {RUNTIME}/source/{class}/runtime\n",
            dependency("coreutils", "bin/ln"),
            quote(compiler_runtime_root)
        ));
    }
    script.push_str(&helper_smokes());
    script.push_str(&format!("{RUNTIME}/tools/sleep 0\nexpect_status 125 {RUNTIME}/tools/sleep 11\n{RUNTIME}/tools/file stagex-configure-utility > configure-file-positive.stdout.txt\n"));
    script.push_str(&format!(
        "{} -q 'ELF 64-bit LSB' configure-file-positive.stdout.txt\n",
        dependency("grep", "bin/grep")
    ));
    script.push_str(&format!(
        "expect_status 125 {RUNTIME}/tools/file /usr/bin/file\nexpect_status 125 {RUNTIME}/tools/file nested/conftest\n"
    ));
    script.push_str(&format!("{} -s {RUNTIME}/stagex-configure-utility configure-file-symlink\nexpect_status 125 {RUNTIME}/tools/file configure-file-symlink\n",dependency("coreutils","bin/ln")));
    script.push_str(&format!("{} {RUNTIME}/source/config.sub sun4 > config-sub-sun4.stdout.txt\ntest \"$({} config-sub-sun4.stdout.txt)\" = sparc-sun-sunos4.1.1\n",dependency("bash-full","bin/bash-full"),dependency("coreutils","bin/cat")));
    // These are the original configure CLI, not cached substitutions.
    let configure = ORIGINAL
        .split_once("fn configure_arguments() -> Vec<String> {")
        .ok_or_else(|| error("original configure CLI disappeared"))?
        .1;
    let configure = configure.split_once(".into_iter()").ok_or_else(|| error("original configure CLI terminator"))?.0;
    let arguments = strings(configure)?;
    let arguments = arguments.into_iter().map(|s| quote(&s)).collect::<Vec<_>>();
    let mut configure_args = arguments;
    let prefix_position = configure_args
        .iter()
        .position(|s| s == "'--with-sysroot='")
        .ok_or_else(|| error("original configure prefix insertion point"))?;
    configure_args.splice(prefix_position..prefix_position, [
        quote(&format!("--prefix={PREFIX}")),
        quote(&format!("--libdir={PREFIX}/lib")),
    ]);
    let cli = configure_args.join(" ");
    for class in array("CONFIGURE_CLASSES")? {
        script.push_str(&format!("cd {RUNTIME}/source/{class}\nexport MANTLE_BINUTILS_CONFIGURE_PROBE_DIR={RUNTIME}/source/{class}\nexport MANTLE_BINUTILS_CONFIGURE_PROBE_CLASS={class}\n{} {cli} > {RUNTIME}/{class}.stdout.txt 2> {RUNTIME}/{class}.stderr.txt\n",dependency("bash-full","bin/bash-full")));
    }
    script.push_str(&format!(
        "cd {RUNTIME}/source\nexport BISON={}\nexport FLEX={}\nexport BISON_PKGDATADIR={}\n",
        dependency("bison", "bin/bison"),
        dependency("flex", "bin/flex"),
        dependency("bison", "share/bison")
    ));
    script.push_str("unset LD MANTLE_BINUTILS_CONFIGURE_PROBE_DIR MANTLE_BINUTILS_CONFIGURE_PROBE_CLASS\n");
    let make = dependency("make", "bin/make");
    let generation = |directory: &str, targets: &str, label: &str| {
        format!(
            "{make} -j1 -C {RUNTIME}/source/{directory} {targets} \"CC=$CC\" \"CC_FOR_BUILD=$CC_FOR_BUILD\" \"CFLAGS_FOR_BUILD=$CFLAGS_FOR_BUILD\" \"AR=$AR\" \"SHELL=$SHELL\" RANLIB=true MAKEINFO=true > {RUNTIME}/{label}.stdout.txt 2> {RUNTIME}/{label}.stderr.txt\n"
        )
    };
    script.push_str(&generation("bfd", "headers", "make-bfd-headers"));
    script.push_str(&generation("intl", "plural.c", "make-intl-plural"));
    script.push_str(&format!("{} {RUNTIME}/bfd-configure.authenticated {RUNTIME}/source/bfd/configure\ncd {RUNTIME}/source/bfd\nexport MANTLE_BINUTILS_CONFIGURE_PROBE_DIR={RUNTIME}/source/bfd\nexport MANTLE_BINUTILS_CONFIGURE_PROBE_CLASS=bfd\nLD=true {} {cli} > {RUNTIME}/bfd-second.stdout.txt 2> {RUNTIME}/bfd-second.stderr.txt\nunset MANTLE_BINUTILS_CONFIGURE_PROBE_DIR MANTLE_BINUTILS_CONFIGURE_PROBE_CLASS\ncd {RUNTIME}/source\n",dependency("coreutils","bin/cp"),dependency("bash-full","bin/bash-full")));
    script.push_str(&generation(
        "binutils",
        &array("BINUTILS_GENERATED_TARGETS")?.iter().map(|s| quote(s)).collect::<Vec<_>>().join(" "),
        "make-binutils-generated",
    ));
    script.push_str(&generation(
        "ld",
        &array("LD_GENERATED_TARGETS")?.iter().map(|s| quote(s)).collect::<Vec<_>>().join(" "),
        "make-ld-generated",
    ));
    for component in ["libiberty", "zlib", "bfd", "opcodes", "binutils", "gas", "gprof", "ld"] {
        script.push_str(&format!("{make} -j1 -C {RUNTIME}/source/{component} tooldir={PREFIX} \"CC=$CC\" \"AR=$AR\" \"SHELL=$SHELL\" RANLIB=true MAKEINFO=true CPPFLAGS=-DPLUGIN_LITTLE_ENDIAN \"CFLAGS=$CFLAGS\" > {RUNTIME}/make-{component}-build.stdout.txt 2> {RUNTIME}/make-{component}-build.stderr.txt\n"));
    }
    for component in ["libiberty", "zlib", "bfd", "opcodes", "binutils", "gas", "gprof", "ld"] {
        script.push_str(&format!("{make} -j1 -C {RUNTIME}/source/{component} tooldir={PREFIX} DESTDIR={RUNTIME}/install-destdir prefix={PREFIX} \"SHELL=$SHELL\" MAKEINFO=true install > {RUNTIME}/make-{component}-install.stdout.txt 2> {RUNTIME}/make-{component}-install.stderr.txt\n"));
    }
    for (tool, _) in crate::stagex_binutils::BINUTILS_REQUIRED_TOOLS {
        script.push_str(&format!("test -x {RUNTIME}/install-destdir{PREFIX}/bin/{tool}\n{} -s {PREFIX}/bin/{tool} {RUNTIME}/install-destdir{PREFIX}/bin/x86_64-unknown-linux-gnu-{tool}\n",dependency("coreutils","bin/ln")));
    }
    Ok(script)
}

pub(super) fn extend(
    author: &mut Author<'_>,
    native: &NativeHelpers,
    sources: &mut Files,
    request: &AuthoringRequest<'_>,
) -> Result<(), RunError> {
    let bison = dependency("bison", "bin/bison");
    let flex = dependency("flex", "bin/flex");
    let source = authored(author, "controller.c", super::dispatch::controller_source(&bison, &flex)?)?;
    let object = author.compile(GENERATORS, "binutils-original-controller-compile", source, "")?;
    let controller = author.link(
        GENERATORS,
        "binutils-original-controller-link",
        &[object, native.runtime.clone(), native.assembly.clone()],
        false,
    )?;
    let utility = utility_variant(author, native)?;
    let empty = author.exec(
        GENERATORS,
        "binutils-original-empty-tree",
        author.builder.artifact("coreutils-mkdir")?,
        Files::new(),
        vec![literal("-p"), output("result", "")],
        ArtifactKind::Tree,
    )?;
    let mut initial = Files::from([
        ("tmp".into(), empty.clone()),
        (format!("{RELATIVE}/sed-spool"), empty.clone()),
        (format!("{RELATIVE}/ar-scratch"), empty.clone()),
    ]);
    smoke_inputs(author, &mut initial)?;
    let source_tree = author.tree(GENERATORS, "binutils-original-runtime-source", sources.clone())?;
    initial.insert(format!("{RELATIVE}/source"), source_tree);
    initial.insert(format!("{RELATIVE}/source/bfd/doc/chew"), author.builder.artifact("binutils-bfd-chew")?);
    initial.insert(
        format!("{RELATIVE}/bfd-configure.authenticated"),
        author.builder.artifact("binutils-bfd-configure-authenticated")?,
    );
    initial.insert(format!("{RELATIVE}/binutils-runtime.o"), native.runtime.clone());
    initial.insert(format!("{RELATIVE}/binutils-runtime-asm.o"), native.assembly.clone());
    initial.insert(
        format!("{RELATIVE}/binutils-tcc-wrapper.sh"),
        authored(author, "binutils-tcc-wrapper.sh", wrapper()?)?,
    );
    for (name, id) in [
        ("stagex-sed-regular-file-bridge.sh", "stagex-sed-regular-file-bridge-source"),
        ("stagex-binutils-ar-runner.sh", "stagex-binutils-ar-runner-source"),
    ] {
        initial.insert(format!("{RELATIVE}/{name}"), author.builder.source(id)?);
    }
    let mut slots = Vec::new();
    let command_inputs = Files::new();
    let slot = |id: &str, raw: ArtifactRef, executable: ArtifactRef, generated_outputs: bool| ControllerSlot {
        id: id.into(),
        executable: ControllerExecutable::Adapted {
            executable,
            original_image: raw,
        },
        inputs: command_inputs.clone(),
        runtime_mounts: BTreeMap::new(),
        exit_policy: RecipeExitPolicy::Normal,
        merge_stderr: false,
        generated_outputs,
    };
    for name in array("COREUTILS_TOOL_NAMES")? {
        let id = format!("coreutils-{name}");
        let raw = author.builder.artifact(&id)?;
        initial.insert(relative(&dependency("coreutils", &format!("bin/{name}")))?, raw.clone());
        initial.insert(format!("{RELATIVE}/tools/{name}"), raw.clone());
        slots.push(slot(&name, raw, author.builder.artifact(&format!("phase-tool:{id}"))?, false));
    }
    for (name, stage, path, id) in [
        ("bash-full", "bash-full", "bin/bash-full", "bash-2.05b-full"),
        ("tcc-0.9.27-musl-v2", "tcc-musl-v2", "bin/tcc-0.9.27-musl-v2", "tcc-musl-v2"),
        ("sed", "sed", "bin/sed", "sed-4.0.9"),
        ("grep", "grep", "bin/grep", "grep-2.4-runner"),
        ("diff", "diffutils", "bin/diff", "diffutils-diff-2.7"),
        ("cmp", "diffutils", "bin/cmp", "diffutils-cmp-2.7"),
        ("gawk", "gawk", "bin/gawk", "gawk-3.0.4"),
        ("m4", "m4", "bin/m4", "m4-1.4.7"),
        ("bison", "bison", "bin/bison", "bison-2.3"),
        ("flex", "flex", "bin/flex", "flex-2.6.4"),
        ("make", "make", "bin/make", "make-3.82"),
    ] {
        let raw = author.builder.artifact(id)?;
        initial.insert(relative(&dependency(stage, path))?, raw.clone());
        let aliases: &[&str] = match name {
            "bash-full" => &["sh", "bash"],
            "tcc-0.9.27-musl-v2" => &["cc"],
            "grep" => &["grep", "egrep", "fgrep"],
            "gawk" => &["awk", "gawk"],
            "sed" => &[],
            _ => &[name],
        };
        for alias in aliases {
            initial.insert(format!("{RELATIVE}/tools/{alias}"), raw.clone());
        }
        if name != "bash-full" {
            let controlled = author.builder.artifact(&format!("phase-tool:{id}"))?;
            slots.push(slot(name, raw.clone(), controlled.clone(), name == "tcc-0.9.27-musl-v2"));
            for alias in aliases {
                if *alias != name {
                    slots.push(slot(alias, raw.clone(), controlled.clone(), name == "tcc-0.9.27-musl-v2"));
                }
            }
        }
    }
    for (name, id) in [
        ("stagex-sed-bridge-launcher", "binutils-sed-bridge-launcher"),
        ("stagex-configure-utility", "binutils-configure-utility"),
        ("stagex-ylwrap-sed-runner", "binutils-ylwrap-runner"),
        ("stagex-elf-local-symbol-canonicalizer", "binutils-elf-symbol-canonicalizer"),
    ] {
        let raw = author.builder.artifact(id)?;
        initial.insert(format!("{RELATIVE}/{name}"), raw.clone());
        if name == "stagex-configure-utility" {
            for alias in ["sleep", "file", "emit"] {
                initial.insert(format!("{RELATIVE}/tools/{alias}"), raw.clone());
                slots.push(slot(alias, raw.clone(), utility.clone(), false));
            }
        }
        if name == "stagex-sed-bridge-launcher" {
            initial.insert(format!("{RELATIVE}/tools/sed"), raw.clone());
            slots.push(slot(name, raw.clone(), raw.clone(), false));
        }
        if name == "stagex-ylwrap-sed-runner" {
            let source = author.builder.source("stagex-ylwrap-sed-runner-source")?;
            let object = author.exec(
                HELPERS,
                "binutils-ylwrap-command-compile",
                author.builder.artifact("tcc-musl-v2")?,
                Files::from([
                    ("source".into(), source),
                    ("headers".into(), author.builder.artifact("musl-native-headers")?),
                ]),
                vec![
                    literal("-c"),
                    literal("-I"),
                    input("headers", ""),
                    input("source", ""),
                    literal("-o"),
                    output("result", ""),
                ],
                ArtifactKind::Object,
            )?;
            let controlled = author.link(
                HELPERS,
                "binutils-ylwrap-command-link",
                &[object, native.runtime.clone(), native.assembly.clone()],
                false,
            )?;
            slots.push(slot(name, raw.clone(), controlled, false));
        }
        if name == "stagex-elf-local-symbol-canonicalizer" {
            slots.push(slot(name, raw, author.builder.artifact("phase-tool:binutils-elf-symbol-canonicalizer")?, true));
        }
    }
    let chew_source = author.builder.fetcher(GENERATORS, super::SOURCE)?;
    let chew_object = author.exec(
        GENERATORS,
        "binutils-bfd-chew-command-compile",
        author.builder.artifact("tcc-musl-v2")?,
        Files::from([
            ("source".into(), chew_source),
            ("headers".into(), author.builder.artifact("musl-native-headers")?),
        ]),
        vec![
            literal("-c"),
            literal("-I"),
            input("headers", ""),
            super::flag("-I", input("source", "include")),
            literal("-D_GNU_SOURCE"),
            literal("-DBUILDFIXED=1"),
            literal("-DDYNAMIC_CRC_TABLE=1"),
            input("source", "bfd/doc/chew.c"),
            literal("-o"),
            output("result", ""),
        ],
        ArtifactKind::Object,
    )?;
    let chew = author.link(
        GENERATORS,
        "binutils-bfd-chew-command-link",
        &[chew_object, native.runtime.clone(), native.assembly.clone()],
        false,
    )?;
    slots.push(slot("chew", author.builder.artifact("binutils-bfd-chew")?, chew, false));
    slots.push(ControllerSlot {
        id: "generated".into(),
        executable: ControllerExecutable::RequestOutput {
            producer_slots: vec![
                "tcc-0.9.27-musl-v2".into(),
                "cc".into(),
                "stagex-elf-local-symbol-canonicalizer".into(),
            ],
        },
        inputs: Files::new(),
        runtime_mounts: BTreeMap::new(),
        exit_policy: RecipeExitPolicy::Normal,
        merge_stderr: false,
        generated_outputs: false,
    });
    initial.insert(relative(&dependency("musl-native", "include"))?, author.builder.artifact("musl-native-headers")?);
    initial
        .insert(relative(&dependency("musl-native", "lib"))?, author.builder.artifact("musl-native-libraries-tree")?);
    initial.insert(relative(&dependency("bison", "share/bison"))?, author.builder.artifact("bison-2.3-runtime-data")?);
    // Preserve the original relative TinyCC runtime suffix at each actual
    // compiler cwd. No argv flag or configured path is substituted.
    let compiler_runtime = author.tree(
        GENERATORS,
        "binutils-original-relative-compiler-runtime",
        Files::from([
            ("tcc-musl-v2/lib/tcc/libtcc1.a".into(), author.builder.artifact("tcc-musl-v2-libtcc1")?),
            ("tcc-musl-v2/lib/tcc/include".into(), author.builder.artifact("tcc-musl-v2-header-tree")?),
            ("musl-1.1.24-pass2/include".into(), author.builder.artifact("musl-pass2-headers")?),
            ("musl-1.1.24-pass2/lib".into(), author.builder.artifact("musl-pass2-libraries-tree")?),
        ]),
    )?;
    let compiler_runtime_root = format!("{}/stagex-binutils-original-runtime", request.proof.logical_store_prefix);
    for slot in &mut slots {
        slot.inputs.insert("original-compiler-runtime".into(), compiler_runtime.clone());
        slot.runtime_mounts.insert(compiler_runtime_root.clone(), "original-compiler-runtime".into());
    }
    let initial = author.tree(GENERATORS, "binutils-original-initial-workspace", initial)?;
    let mut roots = String::new();
    for value in [
        "MANTLE-CONTROLLER-ROOTS-1",
        "mutable",
        "/mantle/stagex",
        "stagex",
        "mutable",
        "/tmp",
        "tmp",
    ] {
        netstring(&mut roots, value);
    }
    for value in [
        "runtime",
        compiler_runtime_root.as_str(),
        compiler_runtime_root.as_str(),
    ] {
        netstring(&mut roots, value);
    }
    // All initial entries above are beneath this one carrier directory.
    let initial = author.tree(
        GENERATORS,
        "binutils-original-logical-workspace",
        Files::from([("stagex".into(), initial.clone()), ("tmp".into(), empty)]),
    )?;
    let roots = authored(author, "roots.netstrings", roots)?;
    for slot in &mut slots {
        slot.inputs.insert("command-roots".into(), roots.clone());
    }
    let cwd = authored(author, "cwd.txt", format!("{RUNTIME}\n"))?;
    let zero = authored(author, "zero.txt", format!("{}\n", dependency("bash-full", "bin/bash-full")))?;
    let mut env = String::new();
    for (key, value) in environment() {
        netstring(&mut env, &key);
        netstring(&mut env, &value);
    }
    let env = authored(author, "environment.netstrings", env)?;
    let driver = authored(author, "recipe.sh", driver(&compiler_runtime_root)?)?;
    let scripts =
        author.tree(GENERATORS, "binutils-original-driver-source", Files::from([("recipe.sh".into(), driver)]))?;
    let variant_slots = slots.clone();
    let inputs = Files::from([
        ("scripts".into(), scripts),
        ("initial-environment".into(), env.clone()),
        ("initial-workspace".into(), initial),
        ("controller-roots".into(), roots.clone()),
        ("initial-cwd".into(), cwd),
        ("original-zero".into(), zero),
    ]);
    let result = author.builder.register_controller(GENERATORS, ControllerSpec {
        id: "binutils-original-runtime".into(),
        controller: controller.clone(),
        inputs,
        controller_runtime_mounts: BTreeMap::new(),
        start_arguments: vec![
            literal("--mantle-source-shell"),
            literal("run"),
            input("scripts", ""),
            literal("recipe.sh"),
            input("initial-environment", ""),
            literal("-"),
            literal("-"),
            input("initial-workspace", ""),
            output("checkpoint", ""),
        ],
        environment: BTreeMap::from([
            ("MANTLE_CONTROLLER_ROOTS".into(), input("controller-roots", "")),
            ("MANTLE_CONTROLLER_INITIAL_CWD".into(), input("initial-cwd", "")),
            ("MANTLE_SHELL_ORIGINAL_ZERO".into(), input("original-zero", "")),
        ]),
        restart_indices: [5, 6, 8],
        slots,
        commands_max: crate::stagex_binutils::BINUTILS_PROTECTED_EXEC_EVENT_COUNT_BOUNDS[1],
        exit_policy: RecipeExitPolicy::Exact { code: 0 },
        file_bytes_max: FILE_BYTES_MAX,
        tree_bytes_max: TREE_BYTES_MAX.min(request.proof.resource_bounds.disk_bytes_max),
    })?;
    let workspace = result.workspace.clone();
    author
        .builder
        .publish(GENERATORS, "binutils-configure-probe-executables", result.generated.clone())?;
    for (id, path, kind) in [
        ("binutils-libiberty-archive", "source/libiberty/libiberty.a", ArtifactKind::Archive),
        ("binutils-zlib-archive", "source/zlib/libz.a", ArtifactKind::Archive),
        ("binutils-bfd-archive", "source/bfd/.libs/libbfd.a", ArtifactKind::Archive),
        ("binutils-opcodes-archive", "source/opcodes/.libs/libopcodes.a", ArtifactKind::Archive),
        ("binutils-size-build-output", "source/binutils/size", ArtifactKind::Executable),
        ("binutils-as-build-output", "source/gas/as-new", ArtifactKind::Executable),
        ("binutils-gprof-build-output", "source/gprof/gprof", ArtifactKind::Executable),
        ("binutils-ld-build-output", "source/ld/ld-new", ArtifactKind::Executable),
    ] {
        let path = format!("stagex/{RELATIVE}/{path}");
        let value = if kind == ArtifactKind::Executable {
            installed(
                author,
                &controller,
                &workspace,
                &result.generated,
                COMPONENTS,
                &format!("binutils-original-publish-{id}"),
                &path,
            )?
        } else {
            project(
                author,
                native,
                COMPONENTS,
                &format!("binutils-original-publish-{id}"),
                workspace.clone(),
                &path,
                kind,
            )?
        };
        author.builder.publish(COMPONENTS, id, value)?;
    }
    let mut generated = Files::new();
    for path in crate::stagex_binutils::BINUTILS_GENERATED_SOURCE_OUTPUTS.iter().copied().chain([
        "opcodes/i386-init.h",
        "opcodes/i386-tbl.h",
        "bfd/bfd.h",
        "bfd/bfd-in3.h",
        "binutils/sysroff.h",
        "ld/ldemul-list.h",
    ]) {
        let reference = project(
            author,
            native,
            COMPONENTS,
            &format!("binutils-original-source-{}", generated.len()),
            workspace.clone(),
            &format!("stagex/{RELATIVE}/source/{path}"),
            ArtifactKind::Text,
        )?;
        generated.insert(path.into(), reference);
    }
    let generated = author.tree(COMPONENTS, "binutils-original-generated-source-products", generated)?;
    author.builder.publish(COMPONENTS, "binutils-generated-sources", generated)?;
    let mut headers = Files::new();
    for (name, path) in [
        ("bfd.h", "bfd/bfd.h"),
        ("ansidecl.h", "include/ansidecl.h"),
        ("symcat.h", "include/symcat.h"),
    ] {
        headers.insert(
            name.into(),
            project(
                author,
                native,
                INSTALL,
                &format!("binutils-original-header-{name}"),
                workspace.clone(),
                &format!("stagex/{RELATIVE}/source/{path}"),
                ArtifactKind::Text,
            )?,
        );
    }
    let headers = author.tree(INSTALL, "binutils-original-publication-headers", headers)?;
    author.builder.artifacts.insert("publication-binutils-headers".into(), headers);
    let ldscripts = project(
        author,
        native,
        INSTALL,
        "binutils-original-publication-ldscripts",
        workspace.clone(),
        &format!("stagex/{INSTALLED}/lib/ldscripts"),
        ArtifactKind::Tree,
    )?;
    author.builder.artifacts.insert("publication-binutils-ldscripts".into(), ldscripts);
    let mut tools = Files::new();
    for (name, _) in crate::stagex_binutils::BINUTILS_REQUIRED_TOOLS {
        let id = format!("binutils-installed-{name}");
        let executable = installed(
            author,
            &controller,
            &workspace,
            &result.generated,
            INSTALL,
            &format!("binutils-original-publish-{id}"),
            &format!("stagex/{INSTALLED}/bin/{name}"),
        )?;
        author.builder.publish(INSTALL, &id, executable.clone())?;
        tools.insert(name.into(), executable);
    }
    let writer =
        super::ld_writer_variant(author, native, result.workspace.clone(), controller, variant_slots, roots, env)?;
    author.builder.artifacts.insert("phase-tool:binutils-ld".into(), writer);
    let checks = authored(author, "checks.c", CHECKS.into())?;
    let checks = author.compile(SMOKE, "binutils-original-checks-compile", checks, "")?;
    let checks = author.link(
        SMOKE,
        "binutils-original-checks-link",
        &[checks, native.runtime.clone(), native.assembly.clone()],
        false,
    )?;
    smokes(author, native, &checks, &tools)
}

fn smoke(
    author: &mut Author<'_>,
    owner: &str,
    id: &str,
    executable: ArtifactRef,
    mut inputs: Files,
    arguments: Vec<RecipeWord>,
    outputs: BTreeMap<String, RecipeOutput>,
    policy: RecipeExitPolicy,
) -> Result<Files, RunError> {
    inputs.insert("executable".into(), executable);
    let mut outputs = outputs;
    for name in ["stdout", "stderr"] {
        outputs.insert(name.into(), super::spec(ArtifactKind::Text));
    }
    outputs.insert("exit-status".into(), RecipeOutput {
        kind: ArtifactKind::Text,
        bytes_max: 4,
    });
    author.add(owner, RecipeStep {
        id: id.into(),
        inputs,
        outputs,
        operation: RecipeOperation::Exec {
            command_state: None,
            executable_input: "executable".into(),
            arguments,
            environment: BTreeMap::new(),
            runtime_mounts: BTreeMap::new(),
            cwd_input: None,
            cwd_output: None,
            stdin_input: None,
            merge_stderr: false,
            invocation_input: None,
            exit_policy: policy,
            guard: None,
            output_seeds: BTreeMap::new(),
        },
    })
}
fn smokes(
    author: &mut Author<'_>,
    native: &NativeHelpers,
    checks: &ArtifactRef,
    tools: &Files,
) -> Result<(), RunError> {
    let source = authored(author, "positive.s", bytes("BINUTILS_POSITIVE_ASSEMBLY")?)?;
    let assembled = smoke(
        author,
        INSTALL,
        "binutils-original-assemble-positive",
        tools["as"].clone(),
        Files::from([("source".into(), source)]),
        vec![literal("-o"), output("object", ""), input("source", "")],
        BTreeMap::from([("object".into(), super::spec(ArtifactKind::Object))]),
        RecipeExitPolicy::Exact { code: 0 },
    )?;
    let object = assembled["object"].clone();
    let linked = smoke(
        author,
        INSTALL,
        "binutils-original-link-positive",
        tools["ld"].clone(),
        Files::from([("object".into(), object.clone())]),
        vec![literal("-o"), output("program", ""), input("object", "")],
        BTreeMap::from([("program".into(), super::spec(ArtifactKind::Executable))]),
        RecipeExitPolicy::Exact { code: 0 },
    )?;
    let positive = linked["program"].clone();
    let published = project(
        author,
        native,
        INSTALL,
        "binutils-original-publish-positive-smoke",
        positive.clone(),
        "",
        ArtifactKind::Executable,
    )?;
    author.builder.publish(INSTALL, "binutils-positive-smoke-executable", published)?;
    let executed = smoke(
        author,
        SMOKE,
        "binutils-original-execute-positive",
        positive,
        Files::new(),
        Vec::new(),
        BTreeMap::new(),
        RecipeExitPolicy::Exact { code: 42 },
    )?;
    let object_tree = author.tree(
        SMOKE,
        "binutils-original-smoke-object-tree",
        Files::from([("positive.o".into(), object.clone())]),
    )?;
    let archive = smoke(
        author,
        SMOKE,
        "binutils-original-archive-positive",
        tools["ar"].clone(),
        Files::from([("object".into(), object_tree)]),
        vec![literal("rc"), output("archive", ""), input("object", "positive.o")],
        BTreeMap::from([("archive".into(), super::spec(ArtifactKind::Archive))]),
        RecipeExitPolicy::Exact { code: 0 },
    )?;
    let ranlib = smoke(
        author,
        SMOKE,
        "binutils-original-ranlib-positive",
        tools["ranlib"].clone(),
        Files::from([("seed".into(), archive["archive"].clone())]),
        vec![output("archive", "")],
        BTreeMap::from([("archive".into(), super::spec(ArtifactKind::Archive))]),
        RecipeExitPolicy::Exact { code: 0 },
    )?;
    if let Some(last) = author.builder.recipe.steps.last_mut()
        && let RecipeOperation::Exec { output_seeds, .. } = &mut last.operation
    {
        output_seeds.insert("archive".into(), "seed".into());
    }
    let copied = smoke(
        author,
        SMOKE,
        "binutils-original-objcopy-positive",
        tools["objcopy"].clone(),
        Files::from([("object".into(), object.clone())]),
        vec![input("object", ""), output("object", "")],
        BTreeMap::from([("object".into(), super::spec(ArtifactKind::Object))]),
        RecipeExitPolicy::Exact { code: 0 },
    )?;
    let mut observations = Files::from([("execution-status".into(), executed["exit-status"].clone())]);
    for (tool, id, source, args, marker) in [
        ("ar", "archive-list", ranlib["archive"].clone(), vec![literal("t")], "positive.o"),
        ("nm", "nm-positive", object.clone(), vec![], "_start"),
        ("objdump", "objdump-positive", copied["object"].clone(), vec![literal("-d")], "syscall"),
        ("readelf", "readelf-positive", object, vec![literal("-h")], "ELF64"),
    ] {
        let mut args = args;
        args.push(input("source", ""));
        let result = smoke(
            author,
            SMOKE,
            &format!("binutils-original-{id}"),
            tools[tool].clone(),
            Files::from([("source".into(), source)]),
            args,
            BTreeMap::new(),
            RecipeExitPolicy::Exact { code: 0 },
        )?;
        let checked = author.exec(
            SMOKE,
            &format!("binutils-original-{id}-marker"),
            checks.clone(),
            Files::from([("source".into(), result["stdout"].clone())]),
            vec![
                literal("contains"),
                input("source", ""),
                output("result", ""),
                literal(marker),
            ],
            ArtifactKind::Text,
        )?;
        observations.insert(id.into(), checked);
    }
    for (tool, id, constant, args, has_output) in [
        (
            "as",
            "assemble-negative",
            "BINUTILS_INVALID_ASSEMBLY",
            vec![literal("-o"), output("work", "rejected"), input("source", "")],
            true,
        ),
        (
            "ld",
            "link-negative",
            "BINUTILS_INVALID_OBJECT",
            vec![literal("-o"), output("work", "rejected"), input("source", "")],
            true,
        ),
        ("ar", "archive-negative", "BINUTILS_INVALID_ARCHIVE", vec![literal("t"), input("source", "")], false),
    ] {
        let source = authored(author, &format!("{id}.input"), bytes(constant)?)?;
        let outputs = if has_output {
            BTreeMap::from([("work".into(), super::spec(ArtifactKind::Tree))])
        } else {
            BTreeMap::new()
        };
        let result = smoke(
            author,
            SMOKE,
            &format!("binutils-original-{id}"),
            tools[tool].clone(),
            Files::from([("source".into(), source)]),
            args,
            outputs,
            RecipeExitPolicy::Normal,
        )?;
        if has_output
            && let Some(last) = author.builder.recipe.steps.last_mut()
            && let RecipeOperation::Exec { cwd_output, .. } = &mut last.operation
        {
            *cwd_output = Some("work".into());
        }
        let mut inputs = Files::from([("status".into(), result["exit-status"].clone())]);
        let mut arguments = vec![literal("rejection"), input("status", ""), output("result", "")];
        if has_output {
            inputs.insert("work".into(), result["work"].clone());
            arguments.push(input("work", "rejected"));
        }
        let checked = author.exec(
            SMOKE,
            &format!("binutils-original-{id}-check"),
            checks.clone(),
            inputs,
            arguments,
            ArtifactKind::Text,
        )?;
        observations.insert(id.into(), checked);
    }
    let receipt = author.tree(SMOKE, "binutils-original-runtime-smoke-observations", observations)?;
    author.builder.publish(SMOKE, "binutils-runtime-smoke", receipt)
}

const CHECKS: &str = r#"#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>
#include <sys/stat.h>
static void fail(void){fputs("binutils original smoke observation mismatch\n",stderr);exit(125);}
static char *read_text(const char *path){FILE *f=fopen(path,"rb");long n;char *p;if(!f||fseek(f,0,SEEK_END)||(n=ftell(f))<0||n>67108864||fseek(f,0,SEEK_SET))fail();p=malloc((size_t)n+1);if(!p||fread(p,1,(size_t)n,f)!=(size_t)n||fclose(f)||memchr(p,0,(size_t)n))fail();p[n]=0;return p;}
int main(int argc,char **argv){char *text;FILE *out;if(argc<4)fail();text=read_text(argv[2]);
 if(!strcmp(argv[1],"contains")){if(argc!=5||!strstr(text,argv[4]))fail();}
 else if(!strcmp(argv[1],"rejection")){char *end;long status=strtol(text,&end,10);struct stat st;if(status<1||status>255||strcmp(end,"\n"))fail();if(argc==5&&(!lstat(argv[4],&st)||errno!=ENOENT))fail();if(argc!=4&&argc!=5)fail();}
 else fail();out=fopen(argv[3],"wb");if(!out||fwrite(text,1,strlen(text),out)!=strlen(text)||fclose(out))fail();free(text);return 0;}
"#;
