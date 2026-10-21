//! Source-owned lowering of the bounded utilities and the TinyCC/musl bridge.
//! Original recipe constants remain the command-list authority. No configured
//! source tree or observed command output is consulted during authoring.
use std::collections::BTreeMap;
use std::collections::BTreeSet;

use crate::errors::RunError;
use crate::source_built_program_authoring::AuthoringRequest;
use crate::source_built_recipe::ArtifactKind;
use crate::source_built_recipe::ArtifactRef;
use crate::source_built_recipe::RecipeOperation;
use crate::source_built_recipe::RecipeOutput;
use crate::source_built_recipe::RecipeStep;
use crate::source_built_recipe::RecipeWord;
use crate::source_built_recipe::error;
use crate::source_built_stagex_programs::TransitionBuilder;
use crate::source_built_stagex_programs::input;
use crate::source_built_stagex_programs::literal;
use crate::source_built_stagex_programs::output;

const LIMIT: u64 = 512 * 1024 * 1024;
const MAKE: &str = include_str!("stagex_make.rs");
const PATCH: &str = include_str!("stagex_gnu_patch.rs");
const GZIP: &str = include_str!("stagex_gzip.rs");
const TAR: &str = include_str!("stagex_tar.rs");
const SED: &str = include_str!("stagex_sed.rs");
const BZIP: &str = include_str!("stagex_bzip2.rs");
const CORE: &str = include_str!("stagex_coreutils.rs");
const OYACC: &str = include_str!("stagex_oyacc.rs");
const BASH: &str = include_str!("stagex_bash.rs");
const MUSL: &str = include_str!("stagex_musl.rs");
const NATIVE: &str = include_str!("stagex_musl_native.rs");
const V2: &str = include_str!("stagex_tcc_musl_v2.rs");
const SELFHOST: &str = include_str!("stagex_tcc_selfhost.rs");

// Read source literals, not evaluated Rust expressions. The accepted constants
// are bounded lists of strings and records; changes outside that grammar fail
// authoring rather than silently selecting a different source closure.
fn strings(text: &str) -> Result<Vec<String>, RunError> {
    let mut result = Vec::new();
    let mut at = 0;
    while let Some(start) = text[at..].find('"') {
        let start = at + start;
        let mut end = start + 1;
        let mut escaped = false;
        for byte in text.as_bytes()[end..].iter() {
            end += 1;
            if *byte == b'"' && !escaped {
                break;
            }
            escaped = *byte == b'\\' && !escaped;
        }
        let value: String = serde_json::from_str(&text[start..end])
            .map_err(|e| error(format!("original StageX string literal: {e}")))?;
        result.push(value);
        at = end;
    }
    Ok(result)
}
fn constant<'a>(source: &'a str, name: &str) -> Result<&'a str, RunError> {
    let marker = format!("const {name}:");
    let tail = source.split_once(&marker).ok_or_else(|| error(format!("missing original constant {name}")))?.1;
    let value = tail.split_once(" = ").ok_or_else(|| error(format!("missing original value {name}")))?.1;
    if value.starts_with("br#\"") {
        let end = value.find("\"#;").ok_or_else(|| error("unterminated raw source constant"))?;
        return Ok(&value[..end + 2]);
    }
    let mut quote = false;
    let mut escaped = false;
    let mut depth = 0_i32;
    for (at, ch) in value.char_indices() {
        if quote {
            if ch == '"' && !escaped {
                quote = false;
            }
            escaped = ch == '\\' && !escaped;
        } else {
            match ch {
                '"' => quote = true,
                '[' | '{' | '(' => depth += 1,
                ']' | '}' | ')' => depth -= 1,
                ';' if depth == 0 => return Ok(&value[..at]),
                _ => {}
            }
        }
    }
    Err(error(format!("unterminated original constant {name}")))
}
fn list(source: &str, name: &str) -> Result<Vec<String>, RunError> {
    strings(constant(source, name)?)
}
fn bytes_constant(source: &str, name: &str) -> Result<String, RunError> {
    let text = constant(source, name)?;
    if let Some(raw) = text.strip_prefix("br#\"").and_then(|s| s.strip_suffix("\"#")) {
        return Ok(raw.to_string());
    }
    strings(text)?.into_iter().next().ok_or_else(|| error(format!("missing literal {name}")))
}
fn records(source: &str, name: &str, record: &str) -> Result<Vec<Vec<String>>, RunError> {
    constant(source, name)?
        .split(&format!("{record} {{"))
        .skip(1)
        .map(|s| strings(s.split_once('}').ok_or_else(|| error("unterminated original record"))?.0))
        .collect()
}
fn words(values: &[String]) -> Vec<RecipeWord> {
    values.iter().map(|s| literal(s)).collect()
}
fn step(
    b: &mut TransitionBuilder,
    tool: &str,
    id: &str,
    executable: ArtifactRef,
    mut inputs: BTreeMap<String, ArtifactRef>,
    arguments: Vec<RecipeWord>,
    kinds: &[(&str, ArtifactKind)],
    cwd: Option<&str>,
    stdin: Option<&str>,
    exit: i32,
) -> Result<BTreeMap<String, ArtifactRef>, RunError> {
    inputs.insert("exe".into(), executable);
    b.add(tool, RecipeStep {
        id: id.into(),
        inputs,
        outputs: [("stdout", ArtifactKind::Text), ("stderr", ArtifactKind::Text)]
            .iter()
            .chain(kinds.iter())
            .map(|(name, kind)| {
                ((*name).into(), RecipeOutput {
                    kind: *kind,
                    bytes_max: LIMIT,
                })
            })
            .collect(),
        operation: RecipeOperation::Exec {
            command_state: None,
            executable_input: "exe".into(),
            invocation_input: None,
            arguments,
            environment: BTreeMap::new(),
            runtime_mounts: BTreeMap::new(),
            output_seeds: BTreeMap::new(),
            guard: None,
            merge_stderr: false,
            cwd_input: cwd.map(str::to_string),
            cwd_output: None,
            stdin_input: stdin.map(str::to_string),
            exit_policy: crate::source_built_recipe::RecipeExitPolicy::Exact { code: exit },
        },
    })
}
fn one(map: BTreeMap<String, ArtifactRef>, name: &str) -> Result<ArtifactRef, RunError> {
    map.get(name).cloned().ok_or_else(|| error(format!("missing authored output {name}")))
}
fn rejected(
    b: &mut TransitionBuilder,
    tool: &str,
    id: &str,
    executable: ArtifactRef,
    mut inputs: BTreeMap<String, ArtifactRef>,
    arguments: Vec<RecipeWord>,
    stdin: Option<&str>,
) -> Result<ArtifactRef, RunError> {
    inputs.insert("exe".into(), executable);
    one(
        b.add(tool, RecipeStep {
            id: id.into(),
            inputs,
            outputs: [
                ("stdout", ArtifactKind::Text),
                ("stderr", ArtifactKind::Text),
                ("exit-status", ArtifactKind::Text),
                ("work", ArtifactKind::Tree),
            ]
            .into_iter()
            .map(|(name, kind)| (name.into(), RecipeOutput { kind, bytes_max: LIMIT }))
            .collect(),
            operation: RecipeOperation::Exec {
                command_state: None,
                executable_input: "exe".into(),
                invocation_input: None,
                arguments,
                environment: BTreeMap::new(),
                runtime_mounts: BTreeMap::new(),
                output_seeds: BTreeMap::new(),
                guard: None,
                merge_stderr: false,
                cwd_input: None,
                cwd_output: Some("work".into()),
                stdin_input: stdin.map(str::to_string),
                exit_policy: crate::source_built_recipe::RecipeExitPolicy::OneOf { codes: vec![1, 2, 255] },
            },
        })?,
        "exit-status",
    )
}
fn artifact_suffix(b: &TransitionBuilder, reference: &ArtifactRef) -> Result<&'static str, RunError> {
    match produced_kind(b, reference)? {
        ArtifactKind::Archive => Ok("a"),
        ArtifactKind::Object => Ok("o"),
        _ => Err(error("compiler link input is not an individually produced object/archive")),
    }
}
fn produced_kind(b: &TransitionBuilder, reference: &ArtifactRef) -> Result<ArtifactKind, RunError> {
    let ArtifactRef::Produced { action_id, output_id } = reference else {
        return Err(error("readiness input is not a produced original artifact"));
    };
    let id = action_id
        .strip_prefix("stagex-transition/")
        .ok_or_else(|| error("original produced artifact has another stage"))?;
    b.recipe
        .steps
        .iter()
        .find(|s| s.id == id)
        .and_then(|s| s.outputs.get(output_id))
        .map(|o| o.kind)
        .ok_or_else(|| error("unknown original produced artifact kind"))
}
type CompileSpecs = Vec<(String, Vec<String>)>;
struct CommandGroup {
    tool: String,
    source: ArtifactRef,
    support: CompileSpecs,
    commands: Vec<(String, CompileSpecs)>,
}
struct Middle<'a> {
    b: &'a mut TransitionBuilder,
    helper: ArtifactRef,
    headers: ArtifactRef,
    command_groups: Vec<CommandGroup>,
}
impl Middle<'_> {
    fn transform(
        &mut self,
        tool: &str,
        id: &str,
        source: ArtifactRef,
        mut args: Vec<RecipeWord>,
        extras: BTreeMap<String, ArtifactRef>,
        kind: ArtifactKind,
    ) -> Result<ArtifactRef, RunError> {
        let mut inputs = extras;
        inputs.insert("source".into(), source);
        for (i, word) in args.iter_mut().enumerate() {
            if let [crate::source_built_recipe::WordPart::Literal { text }] = word.parts.as_slice()
                && (text.contains('/') || text.contains('\\'))
            {
                let name = format!("operand{i}");
                let reference = self.b.authored(&format!("instructions/{id}-{i}.txt"), text.clone())?;
                inputs.insert(name.clone(), reference);
                *word = input(&name, "");
            }
        }
        one(
            step(self.b, tool, id, self.helper.clone(), inputs, args, &[("result", kind)], None, None, 0)?,
            "result",
        )
    }
    fn tree(&mut self, tool: &str, id: &str, source: ArtifactRef) -> Result<ArtifactRef, RunError> {
        self.transform(
            tool,
            id,
            source,
            vec![literal("tree"), input("source", ""), output("result", "")],
            BTreeMap::new(),
            ArtifactKind::Tree,
        )
    }
    fn copy(
        &mut self,
        tool: &str,
        id: &str,
        source: ArtifactRef,
        path: &str,
        kind: ArtifactKind,
    ) -> Result<ArtifactRef, RunError> {
        self.transform(
            tool,
            id,
            source,
            vec![literal("copy"), input("source", path), output("result", "")],
            BTreeMap::new(),
            kind,
        )
    }
    fn replace(
        &mut self,
        tool: &str,
        id: &str,
        source: ArtifactRef,
        path: &str,
        old: &str,
        new: &str,
    ) -> Result<ArtifactRef, RunError> {
        self.transform(
            tool,
            id,
            source,
            vec![
                literal("tree-replace"),
                input("source", ""),
                output("result", ""),
                literal(path),
                literal(old),
                literal(new),
            ],
            BTreeMap::new(),
            ArtifactKind::Tree,
        )
    }
    fn install(
        &mut self,
        tool: &str,
        id: &str,
        source: ArtifactRef,
        file: ArtifactRef,
        path: &str,
    ) -> Result<ArtifactRef, RunError> {
        self.transform(
            tool,
            id,
            source,
            vec![
                literal("tree-install"),
                input("source", ""),
                output("result", ""),
                input("file", ""),
                literal(path),
            ],
            BTreeMap::from([("file".into(), file)]),
            ArtifactKind::Tree,
        )
    }
    fn text(
        &mut self,
        tool: &str,
        id: &str,
        source: ArtifactRef,
        path: &str,
        text: &str,
    ) -> Result<ArtifactRef, RunError> {
        self.transform(
            tool,
            id,
            source,
            vec![
                literal("tree-text"),
                input("source", ""),
                output("result", ""),
                literal(path),
                literal(text),
            ],
            BTreeMap::new(),
            ArtifactKind::Tree,
        )
    }
    fn compile(
        &mut self,
        tool: &str,
        id: &str,
        compiler: &str,
        source: ArtifactRef,
        path: &str,
        flags: &[String],
        headers: Option<ArtifactRef>,
    ) -> Result<ArtifactRef, RunError> {
        let mut inputs = BTreeMap::from([("source".into(), source.clone())]);
        let mut args = Vec::new();
        let mut definitions = String::new();
        if let Some(headers) = headers {
            inputs.insert("headers".into(), headers);
            args.extend([literal("-I"), input("headers", "")]);
        }
        for flag in flags {
            if let Some(path) = flag.strip_prefix("-I") {
                let path = path.strip_prefix("./").unwrap_or(path);
                let path = if path == "." { "" } else { path };
                if path.is_empty() || !path.starts_with('-') {
                    args.extend([literal("-I"), input("source", path)]);
                }
            } else if (flag.contains('/') || flag.contains('\\')) && flag.starts_with("-D") {
                definitions.push_str(&flag[2..]);
                definitions.push('\n');
            } else {
                args.push(literal(flag));
            }
        }
        if !definitions.is_empty() {
            let instructions = self.b.authored(&format!("instructions/{id}-defines.txt"), definitions)?;
            let header = self.transform(
                tool,
                &format!("{id}-define-header"),
                instructions,
                vec![literal("defines"), input("source", ""), output("result", "")],
                BTreeMap::new(),
                ArtifactKind::Text,
            )?;
            inputs.insert("defines.h".into(), header);
            args.extend([literal("-include"), input("defines.h", "")]);
        }
        if path.ends_with(".s") && compiler == "tcc-musl-v2" {
            let inline = self.transform(
                tool,
                &format!("{id}-inline-assembly"),
                source,
                vec![literal("inline"), input("source", path), output("result", "")],
                BTreeMap::new(),
                ArtifactKind::Text,
            )?;
            inputs.insert("inline.c".into(), inline);
            args.extend([literal("-c"), input("inline.c", "")]);
        } else {
            args.extend([literal("-c"), input("source", path)]);
        }
        args.extend([literal("-o"), output("object.o", "")]);
        let exe = self.b.artifact(compiler)?;
        one(
            step(self.b, tool, id, exe, inputs, args, &[("object.o", ArtifactKind::Object)], Some("source"), None, 0)?,
            "object.o",
        )
    }
    fn link(
        &mut self,
        tool: &str,
        id: &str,
        compiler: &str,
        objects: &[ArtifactRef],
        extras: &[(&str, ArtifactRef)],
    ) -> Result<ArtifactRef, RunError> {
        let mut inputs = BTreeMap::new();
        let mut args = vec![
            literal("-static"),
            literal("-nostdlib"),
            literal("-o"),
            output("binary", ""),
        ];
        for (i, object) in objects.iter().enumerate() {
            let suffix = artifact_suffix(self.b, object)?;
            let name = format!("object{i}.{suffix}");
            inputs.insert(name.clone(), object.clone());
            args.push(input(&name, ""));
        }
        for (name, reference) in extras {
            let name = format!("{name}.{}", artifact_suffix(self.b, reference)?);
            inputs.insert(name.clone(), reference.clone());
            args.push(input(&name, ""));
        }
        let exe = self.b.artifact(compiler)?;
        one(
            step(self.b, tool, id, exe, inputs, args, &[("binary", ArtifactKind::Executable)], None, None, 0)?,
            "binary",
        )
    }
    fn mes_link(
        &mut self,
        tool: &str,
        id: &str,
        objects: &[ArtifactRef],
        getopt: bool,
    ) -> Result<ArtifactRef, RunError> {
        let mut extras = Vec::new();
        if getopt {
            extras.push(("getopt", self.b.artifact("tinycc-libgetopt")?));
        }
        extras.extend([
            ("crt1", self.b.artifact("tinycc-crt1")?),
            ("libc", self.b.artifact("tinycc27-libc")?),
            ("runtime", self.b.artifact("tinycc-libtcc1")?),
        ]);
        self.link(tool, id, "tinycc-0.9.27", objects, &extras)
    }
    fn archive(
        &mut self,
        tool: &str,
        id: &str,
        compiler: &str,
        objects: &[(String, ArtifactRef)],
        mode: &str,
    ) -> Result<ArtifactRef, RunError> {
        let mut inputs = BTreeMap::new();
        let mut args = vec![literal("-ar"), literal(mode), output("archive", "")];
        for (name, reference) in objects {
            inputs.insert(name.clone(), reference.clone());
            args.push(input(name, ""));
        }
        let exe = self.b.artifact(compiler)?;
        one(
            step(self.b, tool, id, exe, inputs, args, &[("archive", ArtifactKind::Archive)], None, None, 0)?,
            "archive",
        )
    }
    fn publish(&mut self, tool: &str, id: &str, reference: ArtifactRef) -> Result<(), RunError> {
        let mut required = self.b.original_inputs(tool)?;
        required.extend(self.b.predecessors(tool)?);
        let mut inputs = BTreeMap::new();
        let kind = produced_kind(self.b, &reference)?;
        let mut arguments = vec![
            literal(if kind == ArtifactKind::Tree {
                "ready-tree"
            } else {
                "ready-file"
            }),
            input("source", ""),
            output("result", ""),
        ];
        for (_, dependency) in required {
            if !matches!(dependency, ArtifactRef::Produced { .. }) {
                continue;
            }
            let dependency_kind = produced_kind(self.b, &dependency)?;
            let name = format!("ready{}", inputs.len());
            let expected = match dependency_kind {
                ArtifactKind::Tree => "tree",
                ArtifactKind::Executable | ArtifactKind::Object => "elf",
                ArtifactKind::Archive => "archive",
                ArtifactKind::Text => "text",
            };
            arguments.extend([literal(expected), input(&name, "")]);
            inputs.insert(name, dependency);
        }
        let reference = if inputs.is_empty() {
            reference
        } else {
            self.transform(tool, &format!("{id}-original-readiness"), reference, arguments, inputs, kind)?
        };
        let member = if id == "make-3.82" {
            Some("make-stage/runtime/output/bin/make".to_string())
        } else if id == "make-recipe-runner" {
            Some("make-stage/runtime/output/bin/make-recipe-runner".to_string())
        } else if id == "sed-4.0.9" {
            Some("sed-stage/runtime/output/bin/sed".to_string())
        } else if tool == "coreutils-build-exec" && id.starts_with("coreutils-") {
            Some(format!(
                "coreutils-stage/runtime/output/bin/{}",
                if id == "coreutils-bracket" { "[" } else { &id[10..] }
            ))
        } else if id == "bash-2.05b" {
            Some("bash-stage/runtime/output/bin/bash".to_string())
        } else if id == "sh-2.05b" {
            Some("bash-stage/runtime/output/bin/sh".to_string())
        } else {
            None
        };
        if let Some(member) = member {
            self.b.artifacts.insert(format!("member:{member}"), reference.clone());
        }
        self.b.publish(tool, id, reference)
    }
    fn build_set(
        &mut self,
        tool: &str,
        source: ArtifactRef,
        specs: &[(String, Vec<String>)],
        getopt: bool,
    ) -> Result<ArtifactRef, RunError> {
        let command = match tool {
            "gnu-patch-build-exec" => Some("gnu-patch-2.5.9"),
            "gzip-build-exec" => Some("gzip-1.2.4"),
            "tar-build-exec" => Some("tar-1.12"),
            "sed-build-exec" => Some("sed-4.0.9"),
            "oyacc-build-exec" => Some("oyacc-6.6"),
            _ => None,
        };
        if let Some(id) = command {
            self.command_groups.push(CommandGroup {
                tool: tool.into(),
                source: source.clone(),
                support: Vec::new(),
                commands: vec![(id.into(), specs.to_vec())],
            });
        }
        let mut objects = Vec::new();
        for (i, (path, flags)) in specs.iter().enumerate() {
            objects.push(self.compile(
                tool,
                &format!("{tool}-compile-{i:04}"),
                "tinycc-0.9.27",
                source.clone(),
                path,
                flags,
                Some(self.headers.clone()),
            )?);
        }
        self.mes_link(tool, &format!("{tool}-link"), &objects, getopt)
    }
}

// This program performs only filesystem and text transformations. It contains
// no exec, spawn, system, shell, configure, or compiler-driver entry point.
const HELPER: &str = r###"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <dirent.h>
#include <unistd.h>
static void die(void) { fputs("StageX source transformation failed\n", stderr); exit(1); }
static char *join(const char *a,const char *b) { char *p=malloc(strlen(a)+strlen(b)+2); if(!p)die(); sprintf(p,"%s/%s",a,b); return p; }
static void parents(char *p) { char *q=p+1; for(;*q;q++)if(*q=='/'){*q=0;mkdir(p,0700);*q='/';} }
static char *load(const char *p) { FILE *f=fopen(p,"rb"); long n; char *s; if(!f)die(); if(fseek(f,0,SEEK_END))die(); n=ftell(f); if(n<0||n>67108864)die(); rewind(f); s=malloc(n+1);if(!s)die();if(fread(s,1,n,f)!=(size_t)n)die();s[n]=0;fclose(f);return s; }
static void save(const char *p,const char *s) { FILE *f; char *q=strdup(p);parents(q);free(q);f=fopen(p,"wb");if(!f)die();if(fwrite(s,1,strlen(s),f)!=strlen(s)||fclose(f))die(); }
static void copy(const char *s,const char *d) { FILE *a,*b;size_t n;char buf[8192];char *q=strdup(d);parents(q);free(q);a=fopen(s,"rb");b=fopen(d,"wb");if(!a||!b)die();while((n=fread(buf,1,sizeof(buf),a)))if(fwrite(buf,1,n,b)!=n)die();if(ferror(a)||fclose(a)||fclose(b))die();chmod(d,0700); }
static void tree(const char *s,const char *d) { DIR *dir;struct dirent *e;struct stat st;char *a,*b;mkdir(d,0700);dir=opendir(s);if(!dir)die();while((e=readdir(dir))){if(!strcmp(e->d_name,".")||!strcmp(e->d_name,".."))continue;a=join(s,e->d_name);b=join(d,e->d_name);if(stat(a,&st))die();if(S_ISDIR(st.st_mode))tree(a,b);else if(S_ISREG(st.st_mode))copy(a,b);else die();free(a);free(b);}closedir(dir); }
static void remove_path(const char *p) { struct stat st;DIR *d;struct dirent *e;char *q;if(lstat(p,&st))return;if(S_ISDIR(st.st_mode)){d=opendir(p);if(!d)die();while((e=readdir(d)))if(strcmp(e->d_name,".")&&strcmp(e->d_name,"..")){q=join(p,e->d_name);remove_path(q);free(q);}closedir(d);if(rmdir(p))die();}else if(unlink(p))die(); }
static char *replace(const char *s,const char *old,const char *new) { size_t n=0,ol=strlen(old),nl=strlen(new);const char *p=s,*q;char *out,*w;if(!ol)die();while((q=strstr(p,old))){n++;p=q+ol;}if(!n)die();out=malloc(strlen(s)+n*nl+1);if(!out)die();w=out;p=s;while((q=strstr(p,old))){memcpy(w,p,q-p);w+=q-p;memcpy(w,new,nl);w+=nl;p=q+ol;}strcpy(w,p);return out; }
static void rewrite(const char *p,const char *old,const char *new) { char *s=load(p),*t=replace(s,old,new);save(p,t);free(s);free(t); }
static void range(const char *p,const char *start,const char *end,const char *value) { char *s=load(p),*a=strstr(s,start),*b,*t;if(!a)die();b=strstr(a,end);if(!b)die();b+=strlen(end);t=malloc(strlen(s)+strlen(value)+1);if(!t)die();memcpy(t,s,a-s);strcpy(t+(a-s),value);strcat(t,b);save(p,t);free(s);free(t); }
static void lines(const char *p,const char *pattern,int exact) { char *s=load(p),*q=s,*e,*a,*b;int drop;size_t length=strlen(pattern);FILE *f=fopen(p,"wb");if(!f)die();while(*q){e=strchr(q,'\n');if(!e)e=q+strlen(q);else e++;a=q;b=e;if(exact){while(a<b&&(*a==' '||*a=='\t'||*a=='\r'||*a=='\n'))a++;while(b>a&&(b[-1]==' '||b[-1]=='\t'||b[-1]=='\r'||b[-1]=='\n'))b--;}drop=exact?((size_t)(b-a)==length&&!memcmp(a,pattern,length)):!strncmp(q,pattern,length);if(!drop&&fwrite(q,1,e-q,f)!=(size_t)(e-q))die();q=e;}if(fclose(f))die();free(s); }
static void heredoc(const char *s,const char *p,const char *marker,const char *end) { char *text=load(s),*a=strstr(text,marker),*b;FILE *f;char *q;if(!a)die();a+=strlen(marker);b=strstr(a,end);if(!b||strstr(b+strlen(end),marker))die();q=strdup(p);parents(q);free(q);f=fopen(p,"wb");if(!f||fwrite(a,1,b-a+1,f)!=(size_t)(b-a+1)||fclose(f))die();free(text); }
static void headers(const char *root,const char *dest) { char *a,*b;const char *dirs[]={"include","arch/generic/bits","arch/x86_64/bits","obj/include/bits"};int i;for(i=0;i<4;i++){a=join(root,dirs[i]);b=i?join(dest,"bits"):strdup(dest);tree(a,b);free(a);free(b);} }
static void alltypes(FILE *f,char *s) { char *line=s,*end,*rest,*name,*definition;int more;while(*line){end=strchr(line,'\n');more=end!=0;if(end)*end=0;else end=line+strlen(line);if(!strncmp(line,"TYPEDEF ",8)){rest=line+8;name=strrchr(rest,' ');if(!name)die();*name++=0;if(!*name||name[strlen(name)-1]!=';')die();name[strlen(name)-1]=0;fprintf(f,"#if defined(__NEED_%s) && !defined(__DEFINED_%s)\ntypedef %s %s;\n#define __DEFINED_%s\n#endif\n\n",name,name,rest,name,name);}else if(!strncmp(line,"STRUCT ",7)||!strncmp(line,"UNION ",6)){int st=!strncmp(line,"STRUCT ",7);name=line+(st?7:6);definition=strchr(name,' ');if(!definition)die();*definition++=0;definition[strlen(definition)-1]=0;fprintf(f,"#if defined(__NEED_%s_%s) && !defined(__DEFINED_%s_%s)\n%s %s %s;\n#define __DEFINED_%s_%s\n#endif\n\n",st?"struct":"union",name,st?"struct":"union",name,st?"struct":"union",name,definition,st?"struct":"union",name);}else fprintf(f,"%s\n",line);if(!more)break;line=end+1;} }
static void generate(const char *root) { char *a=join(root,"arch/x86_64/bits/alltypes.h.in"),*b=join(root,"include/alltypes.h.in"),*d=join(root,"obj/include/bits/alltypes.h"),*s=load(a),*t=load(b),*q=strdup(d);FILE *f;parents(q);free(q);f=fopen(d,"wb");if(!f)die();alltypes(f,s);alltypes(f,t);fclose(f);free(a);free(b);free(d);free(s);free(t);a=join(root,"arch/x86_64/bits/syscall.h.in");d=join(root,"obj/include/bits/syscall.h");s=load(a);save(d,s);f=fopen(d,"ab");if(!f)die();q=s;while(*q){char *e=strchr(q,'\n'),*nr;if(!e)e=q+strlen(q);nr=strstr(q,"__NR_");if(nr&&nr<e){fwrite(q,1,nr-q,f);fputs("SYS_",f);fwrite(nr+5,1,e-(nr+5),f);fputc('\n',f);}q=*e?e+1:e;}fclose(f);free(a);free(d);free(s);d=join(root,"obj/src/internal/version.h");save(d,"#define VERSION \"1.1.24\"\n");free(d); }
static void add(char *out,const char *start,size_t n) { size_t len=strlen(out);memcpy(out+len,start,n);out[len+n]=0; }
static void patch_hunk(const char *path,const char *old,const char *new) { char *text,*next;if(!path)die();if(!*old){save(path,new);return;}text=load(path);next=replace(text,old,new);save(path,next);free(text);free(next); }
static void patch(const char *root,const char *file,int strip) {
    char *s=load(file),*q=s,*end,*path=0,*old=calloc(strlen(s)+1,1),*new=calloc(strlen(s)+1,1),*p;int hunk=0;
    if(!old||!new)die();
    while(*q){
        end=strchr(q,'\n');if(!end)end=q+strlen(q);else end++;
        if(!strncmp(q,"+++ ",4)||!strncmp(q,"--- ",4)||!strncmp(q,"@@ ",3)||!strncmp(q,"diff ",5)){
            if(hunk){patch_hunk(path,old,new);old[0]=new[0]=0;hunk=0;}
            if(!strncmp(q,"+++ ",4)){
                char *name,*slash,*e;int i;p=malloc(end-q+1);if(!p)die();memcpy(p,q+4,end-q-4);p[end-q-4]=0;e=strpbrk(p,"\t\r\n ");if(e)*e=0;name=p;
                for(i=0;i<strip;i++){slash=strchr(name,'/');if(!slash)die();name=slash+1;}
                if(name[0]=='/'||strstr(name,".."))die();free(path);path=join(root,name);free(p);
            }else if(!strncmp(q,"@@ ",3))hunk=1;
        }else if(hunk){if(q[0]==' '||q[0]=='-')add(old,q+1,end-q-1);if(q[0]==' '||q[0]=='+')add(new,q+1,end-q-1);}
        q=end;
    }
    if(hunk)patch_hunk(path,old,new);free(path);free(old);free(new);free(s);
}
static unsigned char *binary(const char *p,size_t *n) { FILE *f=fopen(p,"rb");long size;unsigned char *b;if(!f||fseek(f,0,SEEK_END))die();size=ftell(f);if(size<0||size>67108864)die();rewind(f);b=malloc(size+1);if(!b)die();if(fread(b,1,size,f)!=(size_t)size||fclose(f))die();*n=size;return b; }
static void binary_save(const char *p,const unsigned char *b,size_t n) { FILE *f=fopen(p,"wb");if(!f||fwrite(b,1,n,f)!=n||fclose(f))die(); }
static unsigned long long le(const unsigned char *b,size_t n,size_t at,int width) { unsigned long long v=0;int i;if(at>n||(size_t)width>n-at)die();for(i=width-1;i>=0;i--)v=(v<<8)|b[at+i];return v; }
static void put_le(unsigned char *b,size_t n,size_t at,unsigned long long value,int width) { int i;if(at>n||(size_t)width>n-at)die();for(i=0;i<width;i++){b[at+i]=value&255;value>>=8;}if(value)die(); }
struct string_entry { size_t old_offset,new_offset,target; };
static struct string_entry *find_entry(struct string_entry *entries,size_t count,size_t offset) { size_t low=0,high=count,mid;while(low<high){mid=low+(high-low)/2;if(entries[mid].old_offset<offset)low=mid+1;else high=mid;}if(low==count||entries[low].old_offset!=offset)die();return entries+low; }
static size_t aligned(size_t value,size_t alignment) { if(!alignment)alignment=1;if(alignment>1048576||(alignment&(alignment-1))||value>67108864-alignment)die();return (value+alignment-1)&~(alignment-1); }
static size_t shifted(size_t offset,size_t start,size_t end,size_t growth) { if(offset<start)return offset;if(offset<end)die();return offset+growth; }
/* Lower elf_local_symbol_core's fixed-width string-table rebuild, including
 * symbol offsets, section alignment and the relocated section-header table. */
static void canonical(const char *src,const char *dest) {
    size_t n,sections,count,i,j,k,header,symbol_section=0,string_section,strings,size,symbols,bytes,entry_count=0,at,length,used=0,boundary,alignment=1,end,growth,new_sections,new_end,offset,section_size,section_alignment;
    unsigned char *b=binary(src,&n),*table,*out;struct string_entry *entries,*entry;
    if(n<64||memcmp(b,"\177ELF\2\1",6)||le(b,n,16,2)!=1||le(b,n,18,2)!=62||le(b,n,32,8)!=0||le(b,n,56,2)!=0)die();
    sections=le(b,n,40,8);count=le(b,n,60,2);
    if(!count||count>4096||le(b,n,58,2)!=64||sections>n||count>(n-sections)/64)die();
    for(i=0;i<count;i++){header=sections+i*64;if(le(b,n,header+4,4)==2){if(symbol_section)die();symbol_section=i;}}
    if(!symbol_section)die();header=sections+symbol_section*64;
    symbols=le(b,n,header+24,8);bytes=le(b,n,header+32,8);string_section=le(b,n,header+40,4);
    if(string_section>=count||le(b,n,header+56,8)!=24||bytes%24||symbols>n||bytes>n-symbols)die();
    header=sections+string_section*64;if(le(b,n,header+4,4)!=3)die();strings=le(b,n,header+24,8);size=le(b,n,header+32,8);
    if(!size||strings>n||size>n-strings||b[strings])die();
    for(at=0;at<size;at++){if(!b[strings+at])entry_count++;}if(b[strings+size-1])die();
    entries=calloc(entry_count,sizeof(*entries));if(!entries)die();
    for(at=0,i=0;at<size;i++){entries[i].old_offset=at;entries[i].target=(size_t)-1;while(at<size&&b[strings+at])at++;if(at==size)die();at++;}
    for(j=0;j<bytes/24;j++){
        offset=le(b,n,symbols+j*24,4);entry=find_entry(entries,entry_count,offset);k=offset;while(k<size&&b[strings+k])k++;
        if((b[symbols+j*24+4]>>4)!=0||k-offset<3||b[strings+offset]!='L'||b[strings+offset+1]!='.')continue;
        for(at=offset+2;at<k;at++)if(b[strings+at]<'0'||b[strings+at]>'9')break;
        if(at!=k)continue;if(k-offset>12||entry->target!=(size_t)-1)die();entry->target=j;
    }
    for(j=0;j<bytes/24;j++){entry=find_entry(entries,entry_count,le(b,n,symbols+j*24,4));if(entry->target!=(size_t)-1&&(b[symbols+j*24+4]>>4)!=0)die();}
    for(i=0;i<entry_count;i++){at=entries[i].old_offset;k=at;while(b[strings+k])k++;length=entries[i].target==(size_t)-1?k-at:12;if(used>67108864-length-1)die();entries[i].new_offset=used;used+=length+1;}
    table=calloc(used,1);if(!table)die();
    for(i=0;i<entry_count;i++){
        entry=entries+i;at=entry->new_offset;
        if(entry->target==(size_t)-1){k=entry->old_offset;while(b[strings+k])table[at++]=b[strings+k++];}
        else {size_t value=entry->target;table[at]='L';table[at+1]='.';memset(table+at+2,'0',10);k=at+12;do{if(k<=at+2)die();table[--k]='0'+value%10;value/=10;}while(value);}
    }
    boundary=n;if(sections>strings&&sections<=boundary){boundary=sections;alignment=8;}
    for(i=0;i<count;i++){
        header=sections+i*64;offset=le(b,n,header+24,8);section_size=le(b,n,header+32,8);
        if(i==string_section||le(b,n,header+4,4)==8||!section_size||offset<=strings)continue;
        section_alignment=le(b,n,header+48,8);aligned(0,section_alignment);if(!section_alignment)section_alignment=1;
        if(offset<boundary){boundary=offset;alignment=section_alignment;}else if(offset==boundary&&section_alignment>alignment)alignment=section_alignment;
    }
    end=strings+size;if(boundary<end||boundary!=aligned(end,alignment))die();for(at=end;at<boundary;at++)if(b[at])die();
    new_end=aligned(strings+used,alignment);if(used<size||new_end<boundary)die();growth=new_end-boundary;if(n>67108864-growth)die();
    for(j=0;j<bytes/24;j++){entry=find_entry(entries,entry_count,le(b,n,symbols+j*24,4));put_le(b,n,symbols+j*24,entry->new_offset,4);}
    out=calloc(n+growth,1);if(!out)die();memcpy(out,b,strings);memcpy(out+strings,table,used);memcpy(out+new_end,b+boundary,n-boundary);
    new_sections=shifted(sections,strings,boundary,growth);put_le(out,n+growth,40,new_sections,8);
    for(i=0;i<count;i++){
        header=sections+i*64;offset=le(b,n,header+24,8);section_size=le(b,n,header+32,8);
        if(offset&&i!=string_section)offset=shifted(offset,strings,boundary,growth);
        if(i==string_section)section_size=used;
        put_le(out,n+growth,new_sections+i*64+24,offset,8);put_le(out,n+growth,new_sections+i*64+32,section_size,8);
    }
    binary_save(dest,out,n+growth);free(out);free(table);free(entries);free(b);
}
static void inline_source(const char *src,const char *dest) { char *s=load(src),*p=s,*end,*q;FILE *f=fopen(dest,"wb");if(!f)die();fputs("static void force_text_section(void) {}\n__asm__(\n",f);while(*p){end=strchr(p,'\n');if(!end)end=p+strlen(p);if(end-p!=5||strncmp(p,".text",5)){fputc('"',f);for(q=p;q<end;q++){if(*q=='"'||*q=='\\')fputc('\\',f);fputc(*q,f);}fputs("\\n\"\n",f);}p=*end?end+1:end;}fputs(");\n",f);if(fclose(f))die();free(s); }
static void normalize_ar(const char *src,const char *dest) { size_t n,at=8,size;unsigned char *b=binary(src,&n);if(n<8||memcmp(b,"!<arch>\n",8))die();while(at<n){char digits[11];if(n-at<60||memcmp(b+at+58,"`\n",2))die();memcpy(digits,b+at+48,10);digits[10]=0;size=strtoul(digits,0,10);if(size>n-at-60)die();memcpy(b+at+16,"0           0     0     100644  ",32);at+=60+size+(size%2);}if(at!=n)die();binary_save(dest,b,n);free(b); }
static void source_contract(const char *src,const char *dest) { DIR *d=opendir(src);struct dirent *e;char *a,*b,*dot;struct stat st;int count=0;if(!d)die();mkdir(dest,0700);while((e=readdir(d))){dot=strrchr(e->d_name,'.');if(!dot||!(strcmp(dot,".c")==0||strcmp(dot,".h")==0||strcmp(dot,".def")==0))continue;a=join(src,e->d_name);b=join(dest,e->d_name);if(stat(a,&st))die();if(S_ISREG(st.st_mode)){copy(a,b);count++;}free(a);free(b);}closedir(d);if(!count||count>128)die();a=join(src,"include");b=join(dest,"include");tree(a,b);free(a);free(b); }
static size_t ready_tree(const char *path) { DIR *d=opendir(path);struct dirent *e;struct stat st;char *p;size_t count=0,n;unsigned char *b;if(!d)die();while((e=readdir(d))){if(!strcmp(e->d_name,".")||!strcmp(e->d_name,".."))continue;p=join(path,e->d_name);if(lstat(p,&st))die();if(S_ISDIR(st.st_mode))count+=ready_tree(p);else if(S_ISREG(st.st_mode)){b=binary(p,&n);free(b);count++;}else die();free(p);}if(closedir(d))die();return count; }
static void ready(const char *kind,const char *path) { size_t n,i;unsigned char *b;if(!strcmp(kind,"tree")){if(!ready_tree(path))die();return;}b=binary(path,&n);if(!strcmp(kind,"elf")){if(n<4||memcmp(b,"\177ELF",4))die();}else if(!strcmp(kind,"archive")){if(n<8||memcmp(b,"!<arch>\n",8))die();}else if(!strcmp(kind,"text")){for(i=0;i<n;i++)if(!b[i])die();}else die();free(b); }
static void function_replace(const char *path,const char *marker,const char *value) { char *s=load(path),*a=strstr(s,marker),*open,*end,*t;int depth=0;if(!a||(open=strchr(a,'{'))==0)die();for(end=open;*end;end++){if(*end=='{')depth++;else if(*end=='}'&&!--depth){end++;break;}}if(depth)die();t=malloc(strlen(s)+strlen(value)+1);if(!t)die();memcpy(t,s,a-s);strcpy(t+(a-s),value);strcat(t,end);save(path,t);free(s);free(t); }
int main(int argc,char **argv) {
    char *p,*s,*t; FILE *f; int i;
    if(argc<4)die();
    for(i=2;i<argc;i++)if(!strncmp(argv[i],"/inputs/operand",15))argv[i]=load(argv[i]);
    if(!strcmp(argv[1],"defines")){s=load(argv[2]);f=fopen(argv[3],"wb");if(!f)die();p=s;while(*p){char *e=strchr(p,'\n'),*eq;if(!e)e=p+strlen(p);eq=strchr(p,'=');if(!eq||eq>=e)die();fputs("#define ",f);fwrite(p,1,eq-p,f);fputc(' ',f);fwrite(eq+1,1,e-eq-1,f);fputc('\n',f);p=*e?e+1:e;}if(fclose(f))die();free(s);return 0;}
    if(!strcmp(argv[1],"ready-tree")||!strcmp(argv[1],"ready-file")){if(argc%2)die();for(i=4;i<argc;i+=2)ready(argv[i],argv[i+1]);if(!strcmp(argv[1],"ready-tree"))tree(argv[2],argv[3]);else copy(argv[2],argv[3]);return 0;}
    if(!strcmp(argv[1],"gate-text")){if(argc!=7)die();s=load(argv[2]);t=load(argv[5]);if(strcmp(s,argv[4])||(strcmp(t,"1\n")&&strcmp(t,"2\n")&&strcmp(t,"255\n")))die();save(argv[3],argv[6]);free(s);free(t);return 0;}
    if(!strcmp(argv[1],"copy")){copy(argv[2],argv[3]);return 0;}
    if(!strcmp(argv[1],"canonical-elf")){canonical(argv[2],argv[3]);return 0;}
    if(!strcmp(argv[1],"normalize-ar")){normalize_ar(argv[2],argv[3]);return 0;}
    if(!strcmp(argv[1],"inline")){inline_source(argv[2],argv[3]);return 0;}
    if(!strcmp(argv[1],"source-contract")){source_contract(argv[2],argv[3]);return 0;}
    if(!strcmp(argv[1],"text")){save(argv[3],argv[2]);return 0;}
    if(!strcmp(argv[1],"headers")){headers(argv[2],argv[3]);return 0;}
    if(!strcmp(argv[1],"generator-source")){s=load(argv[2]);t=replace(s,"int main(void)","int main(int argc, char **argv)");free(s);s=replace(t,"fopen(\"crc.c\", \"w\")","fopen(argv[1], \"w\")");save(argv[3],s);free(s);free(t);return 0;}
    if(!strcmp(argv[1],"runner-source")){s=load(argv[2]);t=replace(s,"fopen(\"smoke/out\", \"w\")","stdout");free(s);s=replace(t,"write smoke/out smoke-ok","write smoke-out smoke-ok");save(argv[3],s);free(s);free(t);return 0;}
    if(!strcmp(argv[1],"concat")){f=fopen(argv[3],"wb");if(!f)die();for(i=2;i<argc;i++)if(i!=3){s=load(argv[i]);fputs(s,f);free(s);}fclose(f);return 0;}
    tree(argv[2],argv[3]);
    if(!strcmp(argv[1],"tree"))return 0;
    if(argc<5)die();p=join(argv[3],argv[4]);
    if(!strcmp(argv[1],"tree-replace")){if(argc!=7)die();rewrite(p,argv[5],argv[6]);}
    else if(!strcmp(argv[1],"tree-install")){free(p);p=join(argv[3],argv[5]);copy(argv[4],p);}
    else if(!strcmp(argv[1],"tree-text")){if(argc!=6)die();save(p,argv[5]);}
    else if(!strcmp(argv[1],"tree-remove")){for(i=4;i<argc;i++){free(p);p=join(argv[3],argv[i]);remove_path(p);}}
    else if(!strcmp(argv[1],"tree-range")){if(argc!=8)die();range(p,argv[5],argv[6],argv[7]);}
    else if(!strcmp(argv[1],"tree-lines")||!strcmp(argv[1],"tree-declaration")){if(argc!=6)die();lines(p,argv[5],!strcmp(argv[1],"tree-declaration"));}
    else if(!strcmp(argv[1],"tree-heredoc")){if(argc!=8)die();heredoc(argv[5],p,argv[6],argv[7]);}
    else if(!strcmp(argv[1],"tree-validate-helper")){if(argc!=6)die();s=load(p);t=load(argv[5]);if(strcmp(s,t))die();free(s);free(t);}
    else if(!strcmp(argv[1],"tree-empty-config")){if(argc!=6)die();s=load(argv[5]);if(!strstr(s,": > config.h\n"))die();save(p,"");free(s);}
    else if(!strcmp(argv[1],"tree-generate")){generate(argv[3]);}
    else if(!strcmp(argv[1],"tree-patch")){if(argc!=6)die();patch(argv[3],argv[4],atoi(argv[5]));}
    else if(!strcmp(argv[1],"tree-append")){s=load(argv[5]);f=fopen(p,"ab");if(!f)die();if(fputs(s,f)<0||fclose(f))die();free(s);}
    else if(!strcmp(argv[1],"tree-truncate")){char *mark;s=load(p);mark=strstr(s,argv[5]);if(!mark)die();while(mark>s&&mark[-1]!='\n')mark--;*mark=0;t=malloc(strlen(s)+strlen(argv[6])+1);if(!t)die();strcpy(t,s);strcat(t,argv[6]);save(p,t);free(s);free(t);}
    else if(!strcmp(argv[1],"tree-prepend")){s=load(p);t=malloc(strlen(s)+strlen(argv[5])+1);if(!t)die();strcpy(t,argv[5]);strcat(t,s);save(p,t);free(s);free(t);}
    else if(!strcmp(argv[1],"tree-function")){function_replace(p,argv[5],argv[6]);}
    else die();
    free(p);return 0;
}
"###;

fn indexed_specs(paths: Vec<String>, flags: Vec<String>) -> Vec<(String, Vec<String>)> {
    paths.into_iter().map(|path| (path, flags.clone())).collect()
}
fn named_specs(
    source: &str,
    name: &str,
    prefix: &str,
    flags: Vec<String>,
) -> Result<Vec<(String, Vec<String>)>, RunError> {
    Ok(indexed_specs(list(source, name)?.into_iter().map(|p| format!("{prefix}{p}.c")).collect(), flags))
}
impl Middle<'_> {
    fn smoke_stdout(
        &mut self,
        tool: &str,
        id: &str,
        executable: &str,
        args: &[&str],
        stdin: Option<ArtifactRef>,
        exit: i32,
    ) -> Result<ArtifactRef, RunError> {
        let mut inputs = BTreeMap::new();
        if let Some(source) = stdin {
            inputs.insert("stdin".into(), source);
        }
        let has_stdin = inputs.contains_key("stdin");
        let exe = self.b.artifact(executable)?;
        one(
            step(
                self.b,
                tool,
                id,
                exe,
                inputs,
                args.iter().map(|s| literal(s)).collect(),
                &[("stdout", ArtifactKind::Text)],
                None,
                has_stdin.then_some("stdin"),
                exit,
            )?,
            "stdout",
        )
    }
    fn authored_text(&mut self, path: &str, text: &str) -> Result<ArtifactRef, RunError> {
        self.b.authored(path, text.into())
    }
    fn archive_source(&mut self, tool: &str, id: &str, marker: &str) -> Result<ArtifactRef, RunError> {
        let source = self.b.fetcher(tool, id)?;
        let members = self.b.members(id)?;
        if members.iter().any(|p| p == marker) {
            return Ok(source);
        }
        let suffix = format!("/{marker}");
        let roots = members.iter().filter_map(|p| p.strip_suffix(&suffix)).collect::<BTreeSet<_>>();
        if roots.len() != 1 {
            return Err(error(format!("original {id} lacks a unique source root for {marker}")));
        }
        let root = roots.into_iter().next().ok_or_else(|| error("missing original archive root"))?;
        self.transform(
            tool,
            &format!("{tool}-{id}-source-root"),
            source,
            vec![literal("tree"), input("source", root), output("result", "")],
            BTreeMap::new(),
            ArtifactKind::Tree,
        )
    }
    fn musl_members(&mut self) -> Result<Vec<String>, RunError> {
        let members = self.b.members("musl-1.1.24-source")?;
        if members.iter().any(|p| p == "src/internal/libc.h") {
            return Ok(members);
        }
        Ok(members.into_iter().map(|p| p.strip_prefix("musl-1.1.24/").unwrap_or(&p).to_string()).collect())
    }
    fn source(&mut self, id: &str) -> Result<ArtifactRef, RunError> {
        self.b.source(id)
    }
    fn heredoc(
        &mut self,
        tool: &str,
        id: &str,
        tree: ArtifactRef,
        recipe: ArtifactRef,
        target: &str,
        marker: &str,
        end: &str,
    ) -> Result<ArtifactRef, RunError> {
        self.transform(
            tool,
            id,
            tree,
            vec![
                literal("tree-heredoc"),
                input("source", ""),
                output("result", ""),
                literal(target),
                input("recipe", ""),
                literal(marker),
                literal(end),
            ],
            BTreeMap::from([("recipe".into(), recipe)]),
            ArtifactKind::Tree,
        )
    }
    fn checked_heredoc(
        &mut self,
        tool: &str,
        id: &str,
        tree: ArtifactRef,
        recipe: ArtifactRef,
        helper_id: &str,
        target: &str,
        marker: &str,
        end: &str,
    ) -> Result<ArtifactRef, RunError> {
        let tree = self.heredoc(tool, id, tree, recipe, target, marker, end)?;
        let expected = self.source(helper_id)?;
        self.transform(
            tool,
            &format!("{id}-validate-original"),
            tree,
            vec![
                literal("tree-validate-helper"),
                input("source", ""),
                output("result", ""),
                literal(target),
                input("expected", ""),
            ],
            BTreeMap::from([("expected".into(), expected)]),
            ArtifactKind::Tree,
        )
    }
    fn empty_tcc_config(
        &mut self,
        tool: &str,
        id: &str,
        tree: ArtifactRef,
        recipe_id: &str,
    ) -> Result<ArtifactRef, RunError> {
        let recipe = self.source(recipe_id)?;
        self.transform(
            tool,
            id,
            tree,
            vec![
                literal("tree-empty-config"),
                input("source", ""),
                output("result", ""),
                literal("config.h"),
                input("recipe", ""),
            ],
            BTreeMap::from([("recipe".into(), recipe)]),
            ArtifactKind::Tree,
        )
    }
    fn tcc_source(&mut self, tool: &str, prefix: &str, recipe_id: &str) -> Result<ArtifactRef, RunError> {
        let source = self.archive_source(tool, "tinycc-0.9.27-source", "tcc.c")?;
        let patch = self.source("tinycc-0.9.27-patch-source")?;
        let source = self.patch_tree(tool, &format!("{prefix}-stagex-patch"), source, patch)?;
        let source = self.replace(
            tool,
            &format!("{prefix}-fixed-width"),
            source,
            "elf.h",
            "#include <inttypes.h>\n",
            &bytes_constant(include_str!("stagex_tcc_musl_prep.rs"), "FIXED_WIDTH_ALIAS_INSERTION")?,
        )?;
        self.empty_tcc_config(tool, &format!("{prefix}-empty-config"), source, recipe_id)
    }
    fn range(
        &mut self,
        tool: &str,
        id: &str,
        tree: ArtifactRef,
        path: &str,
        start: &str,
        end: &str,
        replacement: &str,
    ) -> Result<ArtifactRef, RunError> {
        self.transform(
            tool,
            id,
            tree,
            vec![
                literal("tree-range"),
                input("source", ""),
                output("result", ""),
                literal(path),
                literal(start),
                literal(end),
                literal(replacement),
            ],
            BTreeMap::new(),
            ArtifactKind::Tree,
        )
    }
    fn remove(&mut self, tool: &str, id: &str, tree: ArtifactRef, paths: &[String]) -> Result<ArtifactRef, RunError> {
        let mut args = vec![literal("tree-remove"), input("source", ""), output("result", "")];
        args.extend(words(paths));
        self.transform(tool, id, tree, args, BTreeMap::new(), ArtifactKind::Tree)
    }
    fn compile_file(
        &mut self,
        tool: &str,
        id: &str,
        file: ArtifactRef,
        flags: &[String],
    ) -> Result<ArtifactRef, RunError> {
        let exe = self.b.artifact("tinycc-0.9.27")?;
        let mut args = vec![literal("-I"), input("headers", "")];
        args.extend(words(flags));
        args.extend([
            literal("-c"),
            input("source.c", ""),
            literal("-o"),
            output("object.o", ""),
        ]);
        one(
            step(
                self.b,
                tool,
                id,
                exe,
                BTreeMap::from([("source.c".into(), file), ("headers".into(), self.headers.clone())]),
                args,
                &[("object.o", ArtifactKind::Object)],
                None,
                None,
                0,
            )?,
            "object.o",
        )
    }
    fn patch_tree(
        &mut self,
        tool: &str,
        id: &str,
        tree: ArtifactRef,
        patch: ArtifactRef,
    ) -> Result<ArtifactRef, RunError> {
        if tool.starts_with("coreutils-") || tool.starts_with("oyacc-") {
            let exe = self.b.artifact("gnu-patch-2.5.9")?;
            return one(
                self.b.add(tool, RecipeStep {
                    id: id.into(),
                    inputs: BTreeMap::from([("exe".into(), exe), ("source".into(), tree), ("patch".into(), patch)]),
                    outputs: [
                        ("result", ArtifactKind::Tree),
                        ("stdout", ArtifactKind::Text),
                        ("stderr", ArtifactKind::Text),
                    ]
                    .into_iter()
                    .map(|(name, kind)| (name.into(), RecipeOutput { kind, bytes_max: LIMIT }))
                    .collect(),
                    operation: RecipeOperation::Exec {
                        command_state: None,
                        executable_input: "exe".into(),
                        invocation_input: None,
                        arguments: vec![literal("-p1"), literal("-i"), input("patch", "")],
                        environment: BTreeMap::new(),
                        runtime_mounts: BTreeMap::new(),
                        output_seeds: BTreeMap::from([("result".into(), "source".into())]),
                        guard: None,
                        merge_stderr: false,
                        cwd_input: None,
                        cwd_output: Some("result".into()),
                        stdin_input: None,
                        exit_policy: crate::source_built_recipe::RecipeExitPolicy::Exact { code: 0 },
                    },
                })?,
                "result",
            );
        }
        self.transform(
            tool,
            id,
            tree,
            vec![
                literal("tree-patch"),
                input("source", ""),
                output("result", ""),
                input("patch", ""),
                literal("2"),
            ],
            BTreeMap::from([("patch".into(), patch)]),
            ArtifactKind::Tree,
        )
    }
    fn utility_smoke(&mut self, name: &str, executable: &str, source: &str, args: &[&str]) -> Result<(), RunError> {
        let tool = format!("{name}-smoke-exec");
        let data = self.authored_text(&format!("programs/stagex/{name}-smoke.txt"), source)?;
        let reference = if executable == "sed-4.0.9" {
            let script = self.authored_text("programs/stagex/sed-positive.sed", &format!("{}\n", args[0]))?;
            let exe = self.b.artifact(executable)?;
            one(
                step(
                    self.b,
                    &tool,
                    &format!("{name}-positive"),
                    exe,
                    BTreeMap::from([("data".into(), data), ("script".into(), script)]),
                    vec![literal("-f"), input("script", ""), input("data", "")],
                    &[("stdout", ArtifactKind::Text)],
                    None,
                    None,
                    0,
                )?,
                "stdout",
            )?
        } else {
            self.smoke_stdout(&tool, &format!("{name}-positive"), executable, args, Some(data), 0)?
        };
        self.publish(&tool, &format!("{name}-smoke"), reference)
    }
}

fn utilities(m: &mut Middle<'_>) -> Result<(), RunError> {
    let tool = "make-recipe-runner-build-exec";
    let runner = m.source("make-recipe-runner-source")?;
    let runner = m.transform(
        tool,
        "make-runner-output-relocation",
        runner,
        vec![literal("runner-source"), input("source", ""), output("result", "")],
        BTreeMap::new(),
        ArtifactKind::Text,
    )?;
    let object = m.compile_file(tool, "make-recipe-runner-compile", runner, &[])?;
    let binary = m.mes_link(tool, "make-recipe-runner-link", &[object], false)?;
    m.publish(tool, "make-recipe-runner", binary)?;

    let tool = "make-build-exec";
    let source = m.archive_source(tool, "make-3.82-source", "main.c")?;
    let patch = m.source("make-3.82-stagex-patch")?;
    let source = m.patch_tree(tool, "make-source-patch", source, patch)?;
    let source = m.text(tool, "make-original-config", source, "config.h", "")?;
    let common = list(MAKE, "MAKE_COMMON_FLAGS")?;
    let specs = records(MAKE, "MAKE_COMPILE_SPECS", "MakeCompileSpec")?
        .into_iter()
        .map(|record| {
            let mut flags = common.clone();
            flags.extend(record.iter().skip(2).cloned());
            (record[0].clone(), flags)
        })
        .collect::<Vec<_>>();
    m.b.artifacts.insert("make-3.82-configured-source".into(), source.clone());
    let binary = m.build_set(tool, source, &specs, false)?;
    m.publish(tool, "make-3.82", binary)?;
    // The original Make dispatch is expanded, not run as a parent process. Its
    // exact bounded recipe runner is the sole execution leaf for that recipe.
    let runner = m.b.artifact("make-recipe-runner")?;
    let smoke = one(
        step(
            m.b,
            "make-smoke-exec",
            "make-bounded-recipe",
            runner,
            BTreeMap::new(),
            vec![literal("-c"), literal("write smoke-out smoke-ok")],
            &[("stdout", ArtifactKind::Text)],
            None,
            None,
            0,
        )?,
        "stdout",
    )?;
    m.publish("make-smoke-exec", "make-recipe-smoke", smoke)?;

    let tool = "gnu-patch-build-exec";
    let source = m.archive_source(tool, "gnu-patch-2.5.9-source", "patch.c")?;
    let config = bytes_constant(PATCH, "GNU_PATCH_CONFIG_H")?;
    let source = m.text(tool, "gnu-patch-config", source, "config.h", &config)?;
    let source = m.text(tool, "gnu-patch-patchlevel", source, "patchlevel.h", "")?;
    let specs = named_specs(PATCH, "GNU_PATCH_SOURCE_NAMES", "", vec!["-I.".into(), "-DHAVE_CONFIG_H".into()])?;
    let binary = m.build_set(tool, source, &specs, false)?;
    m.publish(tool, "gnu-patch-2.5.9", binary)?;
    let old = m.authored_text("programs/stagex/patch-positive.txt", "old\n")?;
    let diff = m.authored_text(
        "programs/stagex/patch-positive.diff",
        "--- smoke.txt\n+++ smoke.txt\n@@ -1 +1 @@\n-old\n+new\n",
    )?;
    let exe = m.b.artifact("gnu-patch-2.5.9")?;
    let smoke = one(
        step(
            m.b,
            "gnu-patch-smoke-exec",
            "gnu-patch-positive",
            exe,
            BTreeMap::from([("old".into(), old), ("patch".into(), diff)]),
            vec![
                literal("-o"),
                output("result", ""),
                literal("-i"),
                input("patch", ""),
                input("old", ""),
            ],
            &[("result", ArtifactKind::Text)],
            None,
            None,
            0,
        )?,
        "result",
    )?;
    m.publish("gnu-patch-smoke-exec", "gnu-patch-smoke", smoke)?;
    let malformed =
        m.authored_text("programs/stagex/patch-malformed.diff", &bytes_constant(PATCH, "GNU_PATCH_MALFORMED_DIFF")?)?;
    let exe = m.b.artifact("gnu-patch-2.5.9")?;
    rejected(
        m.b,
        "gnu-patch-smoke-exec",
        "gnu-patch-reject-malformed",
        exe,
        BTreeMap::from([("patch".into(), malformed)]),
        vec![literal("-i"), input("patch", "")],
        None,
    )?;

    let tool = "gzip-generator-build-exec";
    let generator = m.source("gzip-makecrc-source")?;
    // The original generator's fixed output pathname is redirected in source,
    // never by a launching shell or by importing a precomputed CRC table.
    let generator_tree = m.transform(
        tool,
        "gzip-generator-source",
        generator,
        vec![literal("generator-source"), input("source", ""), output("result", "")],
        BTreeMap::new(),
        ArtifactKind::Text,
    )?;
    let object = m.compile_file(tool, "gzip-generator-compile", generator_tree, &[])?;
    let binary = m.mes_link(tool, "gzip-generator-link", &[object], false)?;
    m.publish(tool, "gzip-makecrc", binary.clone())?;
    let crc = one(
        step(
            m.b,
            "gzip-generator-run-exec",
            "gzip-generate-crc",
            binary,
            BTreeMap::new(),
            vec![output("crc", "")],
            &[("crc", ArtifactKind::Text)],
            None,
            None,
            0,
        )?,
        "crc",
    )?;
    m.publish("gzip-generator-run-exec", "gzip-crc-table", crc.clone())?;
    let tool = "gzip-build-exec";
    let source = m.archive_source(tool, "gzip-1.2.4-source", "gzip.c")?;
    let source = m.range(tool, "gzip-remove-original-crc", source, "util.c", "ulg crc_32_tab", "};", "")?;
    let source = m.transform(
        tool,
        "gzip-install-generated-crc",
        source,
        vec![
            literal("tree-append"),
            input("source", ""),
            output("result", ""),
            literal("util.c"),
            input("crc", ""),
        ],
        BTreeMap::from([("crc".into(), crc)]),
        ArtifactKind::Tree,
    )?;
    let specs =
        named_specs(GZIP, "GZIP_SOURCE_NAMES", "", vec!["-I.".into(), "-DNO_UTIME".into(), "-Dstrlwr=unused".into()])?;
    let binary = m.build_set(tool, source, &specs, false)?;
    m.publish(tool, "gzip-1.2.4", binary.clone())?;
    let alias = m.copy(tool, "gzip-gunzip-alias", binary, "", ArtifactKind::Executable)?;
    m.publish(tool, "gunzip-1.2.4", alias)?;
    compression_smoke(m, "gzip", "gzip-1.2.4", "gunzip-1.2.4", "gzip smoke\n")?;

    let tool = "tar-build-exec";
    let source = m.archive_source(tool, "tar-1.12-source", "src/tar.c")?;
    let source = m.text(tool, "tar-config", source, "config.h", &bytes_constant(TAR, "TAR_CONFIG_H")?)?;
    let stub = m.source("tar-getdate-stub-source")?;
    let source = m.install(tool, "tar-install-getdate", source, stub, "lib/getdate_stub.c")?;
    let common = list(TAR, "TAR_COMMON_FLAGS")?;
    let mut specs = named_specs(TAR, "TAR_LIB_SOURCES", "lib/", common.clone())?;
    let mut flags = common;
    flags.extend(["-Ilib".into(), "-Isrc".into()]);
    specs.extend(named_specs(TAR, "TAR_SRC_SOURCES", "src/", flags)?);
    let binary = m.build_set(tool, source, &specs, false)?;
    m.publish(tool, "tar-1.12", binary)?;
    let smoke_data = m.authored_text("programs/stagex/tar-positive.txt", "tar112-ok\n")?;
    let smoke_tree = m.assemble("tar-smoke-exec", "tar-positive-tree", &[("positive.txt".into(), smoke_data)])?;
    let exe = m.b.artifact("tar-1.12")?;
    let archive = one(
        step(
            m.b,
            "tar-smoke-exec",
            "tar-create-positive",
            exe.clone(),
            BTreeMap::from([("source".into(), smoke_tree)]),
            vec![
                literal("-cf"),
                output("archive", ""),
                literal("-C"),
                input("source", ""),
                literal("positive.txt"),
            ],
            &[("archive", ArtifactKind::Archive)],
            None,
            None,
            0,
        )?,
        "archive",
    )?;
    let smoke = one(
        step(
            m.b,
            "tar-smoke-exec",
            "tar-extract-positive",
            exe,
            BTreeMap::from([("archive".into(), archive)]),
            vec![literal("-xOf"), input("archive", ""), literal("positive.txt")],
            &[("stdout", ArtifactKind::Text)],
            None,
            None,
            0,
        )?,
        "stdout",
    )?;
    m.publish("tar-smoke-exec", "tar-smoke", smoke)?;
    let malformed =
        m.authored_text("programs/stagex/tar-malformed.txt", &bytes_constant(TAR, "TAR_MALFORMED_INPUT")?)?;
    let exe = m.b.artifact("tar-1.12")?;
    rejected(
        m.b,
        "tar-smoke-exec",
        "tar-reject-malformed",
        exe,
        BTreeMap::from([("archive".into(), malformed)]),
        vec![literal("-tf"), input("archive", "")],
        None,
    )?;

    let tool = "sed-build-exec";
    let source = m.archive_source(tool, "sed-4.0.9-source", "sed/sed.c")?;
    let source = m.text(tool, "sed-config", source, "config.h", "")?;
    let flags = list(SED, "SED_COMMON_FLAGS")?.into_iter().filter(|s| s != "-c").collect::<Vec<_>>();
    let mut specs = named_specs(SED, "SED_LIB_SOURCES", "lib/", flags.clone())?;
    specs.extend(named_specs(SED, "SED_APP_SOURCES", "sed/", flags)?);
    let binary = m.build_set(tool, source, &specs, false)?;
    m.publish(tool, "sed-4.0.9", binary)?;
    m.utility_smoke("sed", "sed-4.0.9", "sed409-input\n", &["s/input/ok/"])?;
    let script = m.authored_text("programs/stagex/sed-malformed.sed", "s/[//\n")?;
    let data = m.authored_text("programs/stagex/sed-malformed-input.txt", "sed409-input\n")?;
    let exe = m.b.artifact("sed-4.0.9")?;
    rejected(
        m.b,
        "sed-smoke-exec",
        "sed-reject-malformed",
        exe,
        BTreeMap::from([("script".into(), script), ("data".into(), data)]),
        vec![literal("-f"), input("script", ""), input("data", "")],
        None,
    )?;

    bzip_core_bash(m)
}

fn bzip_core_bash(m: &mut Middle<'_>) -> Result<(), RunError> {
    let tool = "bzip2-rewrite-exec";
    let mut source = m.archive_source(tool, "bzip2-1.0.8-source", "bzip2.c")?;
    let header = m.source("bzip2-utime-header-source")?;
    source = m.install(tool, "bzip2-utime-header", source, header, "utime.h")?;
    for (i, script) in list(BZIP, "BZIP2_REWRITES")?.chunks_exact(2).enumerate() {
        let exe = m.b.artifact("sed-4.0.9")?;
        let script = m.authored_text(&format!("instructions/bzip2-{i}.sed"), &format!("{}\n", script[0]))?;
        let rewritten = one(
            step(
                m.b,
                tool,
                &format!("bzip2-sed-{i}"),
                exe,
                BTreeMap::from([("source".into(), source.clone()), ("script".into(), script)]),
                vec![literal("-f"), input("script", ""), input("source", "bzip2.c")],
                &[("stdout", ArtifactKind::Text)],
                None,
                None,
                0,
            )?,
            "stdout",
        )?;
        source = m.install(tool, &format!("bzip2-install-rewrite-{i}"), source, rewritten, "bzip2.c")?;
    }
    m.publish(tool, "bzip2-configured-source", source.clone())?;
    let tool = "bzip2-build-exec";
    let flags = vec!["-D_FILE_OFFSET_BITS=64".into(), "-I.".into()];
    m.command_groups.push(CommandGroup {
        tool: tool.into(),
        source: source.clone(),
        support: Vec::new(),
        commands: vec![(
            "bzip2-1.0.8".into(),
            indexed_specs(
                list(BZIP, "BZIP2_LIBRARY_SOURCES")?
                    .into_iter()
                    .map(|name| format!("{name}.c"))
                    .chain(std::iter::once("bzip2.c".into()))
                    .collect(),
                flags.clone(),
            ),
        )],
    });
    m.command_groups.push(CommandGroup {
        tool: tool.into(),
        source: source.clone(),
        support: Vec::new(),
        commands: vec![("bzip2recover-1.0.8".into(), vec![("bzip2recover.c".into(), flags.clone())])],
    });
    let mut library = Vec::new();
    for (i, name) in list(BZIP, "BZIP2_LIBRARY_SOURCES")?.iter().enumerate() {
        library.push(m.compile(
            tool,
            &format!("bzip2-library-{i}"),
            "tinycc-0.9.27",
            source.clone(),
            &format!("{name}.c"),
            &flags,
            Some(m.headers.clone()),
        )?);
    }
    let app =
        m.compile(tool, "bzip2-app", "tinycc-0.9.27", source.clone(), "bzip2.c", &flags, Some(m.headers.clone()))?;
    library.push(app);
    let binary = m.mes_link(tool, "bzip2-link", &library, false)?;
    m.publish(tool, "bzip2-1.0.8", binary)?;
    let object = m.compile(
        tool,
        "bzip2recover-app",
        "tinycc-0.9.27",
        source,
        "bzip2recover.c",
        &flags,
        Some(m.headers.clone()),
    )?;
    let binary = m.mes_link(tool, "bzip2recover-link", &[object], false)?;
    m.publish(tool, "bzip2recover-1.0.8", binary)?;
    compression_smoke(m, "bzip2", "bzip2-1.0.8", "bzip2-1.0.8", "bzip2-108-ok\n")?;

    let tool = "coreutils-patch-exec";
    let mut source = m.archive_source(tool, "coreutils-5.0-source", "src/cat.c")?;
    let header = m.source("coreutils-5.0-config-header-source")?;
    source = m.install(tool, "coreutils-config-header", source, header, "config.h")?;
    for (i, record) in records(CORE, "COREUTILS_PATCHES", "CoreutilsPatchSpec")?.iter().enumerate() {
        let patch = m.source(&record[0])?;
        source = m.patch_tree(tool, &format!("coreutils-patch-{i}"), source, patch)?;
    }
    for (i, (old, new)) in [
        ("lib/fnmatch_.h", "lib/fnmatch.h"),
        ("lib/ftw_.h", "lib/ftw.h"),
        ("lib/search_.h", "lib/search.h"),
    ]
    .iter()
    .enumerate()
    {
        let file = m.copy(tool, &format!("coreutils-header-source-{i}"), source.clone(), old, ArtifactKind::Text)?;
        source = m.install(tool, &format!("coreutils-header-install-{i}"), source, file, new)?;
    }
    m.publish(tool, "coreutils-configured-source", source.clone())?;
    let tool = "coreutils-build-exec";
    let flags = list(CORE, "COMMON_FLAGS")?.into_iter().filter(|s| s != "-c").collect::<Vec<_>>();
    let commands = records(CORE, "UTILITY_SPECS", "UtilitySpec")?
        .into_iter()
        .map(|record| {
            (
                format!("coreutils-{}", record[0]),
                indexed_specs(record.into_iter().skip(1).collect(), flags.clone()),
            )
        })
        .collect();
    m.command_groups.push(CommandGroup {
        tool: tool.into(),
        source: source.clone(),
        support: indexed_specs(list(CORE, "LIBRARY_SOURCES")?, flags.clone()),
        commands,
    });
    let mut objects = Vec::new();
    for (i, path) in list(CORE, "LIBRARY_SOURCES")?.iter().enumerate() {
        objects.push(m.compile(
            tool,
            &format!("coreutils-library-{i:03}"),
            "tinycc-0.9.27",
            source.clone(),
            path,
            &flags,
            Some(m.headers.clone()),
        )?);
    }
    let members = list(CORE, "LIBRARY_SOURCES")?
        .into_iter()
        .zip(objects)
        .map(|(path, object)| {
            (format!("{}.o", path.rsplit('/').next().unwrap_or(&path).trim_end_matches(".c")), object)
        })
        .collect::<Vec<_>>();
    let archive = m.archive(tool, "coreutils-support-archive", "tinycc-0.9.27", &members, "cr")?;
    for (i, record) in records(CORE, "UTILITY_SPECS", "UtilitySpec")?.iter().enumerate() {
        let name = &record[0];
        let mut objects = Vec::new();
        for (j, path) in record.iter().skip(1).enumerate() {
            objects.push(m.compile(
                tool,
                &format!("coreutils-{name}-{i}-{j}"),
                "tinycc-0.9.27",
                source.clone(),
                path,
                &flags,
                Some(m.headers.clone()),
            )?);
        }
        objects.push(archive.clone());
        let binary = m.mes_link(tool, &format!("coreutils-{name}-link"), &objects, false)?;
        m.publish(tool, &format!("coreutils-{name}"), binary)?;
    }
    let test = m.b.artifact("coreutils-test")?;
    let bracket = m.copy(tool, "coreutils-bracket-alias", test, "", ArtifactKind::Executable)?;
    m.publish(tool, "coreutils-bracket", bracket)?;
    let smoke_tool = "coreutils-smoke-exec";
    let data =
        m.authored_text("programs/stagex/coreutils-positive.txt", &bytes_constant(CORE, "COREUTILS_SMOKE_INPUT")?)?;
    let exe = m.b.artifact("coreutils-mkdir")?;
    let directory = one(
        step(
            m.b,
            smoke_tool,
            "coreutils-mkdir-positive",
            exe,
            BTreeMap::new(),
            vec![literal("-p"), output("tree", "dir")],
            &[("tree", ArtifactKind::Tree)],
            None,
            None,
            0,
        )?,
        "tree",
    )?;
    m.smoke_stdout(smoke_tool, "coreutils-echo-positive", "coreutils-echo", &["coreutils-smoke"], None, 0)?;
    let exe = m.b.artifact("coreutils-cp")?;
    let copied = one(
        step(
            m.b,
            smoke_tool,
            "coreutils-copy-positive",
            exe,
            BTreeMap::from([("data".into(), data)]),
            vec![input("data", ""), output("copied", "")],
            &[("copied", ArtifactKind::Text)],
            None,
            None,
            0,
        )?,
        "copied",
    )?;
    let exe = m.b.artifact("coreutils-cat")?;
    let smoke = one(
        step(
            m.b,
            smoke_tool,
            "coreutils-cat-positive",
            exe,
            BTreeMap::from([("data".into(), copied.clone())]),
            vec![input("data", "")],
            &[("stdout", ArtifactKind::Text)],
            None,
            None,
            0,
        )?,
        "stdout",
    )?;
    let exe = m.b.artifact("coreutils-test")?;
    step(
        m.b,
        smoke_tool,
        "coreutils-test-positive",
        exe,
        BTreeMap::from([("data".into(), copied)]),
        vec![literal("-f"), input("data", "")],
        &[],
        None,
        None,
        0,
    )?;
    let exe = m.b.artifact("coreutils-cp")?;
    rejected(
        m.b,
        smoke_tool,
        "coreutils-reject-missing",
        exe,
        BTreeMap::from([("directory".into(), directory)]),
        vec![input("directory", "missing"), output("work", "rejected")],
        None,
    )?;
    m.publish(smoke_tool, "coreutils-smoke", smoke)?;

    let tool = "oyacc-patch-exec";
    let mut source = m.archive_source(tool, "oyacc-6.6-source", "main.c")?;
    let empty = m.source("oyacc-empty-config-source")?;
    source = m.install(tool, "oyacc-empty-config", source, empty, "config.h")?;
    for (i, name) in ["oyacc-meslibc-patch-source", "oyacc-tcc-patch-source"].iter().enumerate() {
        let patch = m.source(name)?;
        source = m.patch_tree(tool, &format!("oyacc-patch-{i}"), source, patch)?;
    }
    m.publish(tool, "oyacc-configured-source", source.clone())?;
    let specs = named_specs(OYACC, "OYACC_SOURCES", "", vec!["-D__dead=".into(), "-D__unused=".into()])?;
    let binary = m.build_set("oyacc-build-exec", source, &specs, false)?;
    m.publish("oyacc-build-exec", "oyacc-6.6", binary)?;
    let grammar = m.source("oyacc-smoke-grammar-source")?;
    let exe = m.b.artifact("oyacc-6.6")?;
    let generated = one(
        step(
            m.b,
            "oyacc-smoke-exec",
            "oyacc-generate-parser",
            exe,
            BTreeMap::from([("grammar".into(), grammar)]),
            vec![literal("-o"), output("parser", ""), input("grammar", "")],
            &[("parser", ArtifactKind::Text)],
            None,
            None,
            0,
        )?,
        "parser",
    )?;
    m.publish("oyacc-smoke-exec", "oyacc-smoke-output", generated)?;
    let malformed = m.source("oyacc-malformed-grammar-source")?;
    let exe = m.b.artifact("oyacc-6.6")?;
    rejected(
        m.b,
        "oyacc-smoke-exec",
        "oyacc-reject-malformed",
        exe,
        BTreeMap::from([("grammar".into(), malformed)]),
        vec![input("grammar", "")],
        None,
    )?;

    let tool = "bash-build-exec";
    let mut source = m.archive_source(tool, "bash-2.05b-source", "shell.c")?;
    let recipe = m.source("bash-2.05b-recipe-source")?;
    for (i, helper) in records(BASH, "BASH_HELPERS", "BashHelperSpec")?.iter().enumerate() {
        source = m.checked_heredoc(
            tool,
            &format!("bash-helper-{i}"),
            source,
            recipe.clone(),
            &helper[0],
            &helper[2],
            &format!("<< '{}'\n", helper[1]),
            &format!("\n{}\n", helper[1]),
        )?;
    }
    m.publish(tool, "bash-configured-source", source.clone())?;
    let flags = vec![
        "-I.".into(),
        "-Iinclude".into(),
        "-Ilib".into(),
        "-Ibuiltins".into(),
        "-DHAVE_CONFIG_H".into(),
    ];
    let mut paths = list(BASH, "CORE_SOURCES")?;
    paths.extend(["builtins/bootstrap_stubs".into(), "lib/readline/readline_stub".into()]);
    paths.extend(list(BASH, "GLOB_SOURCES")?);
    paths.push("lib/tilde/tilde".into());
    paths.extend(list(BASH, "SH_LIBRARY_SOURCES")?);
    paths.extend(list(BASH, "BUILTIN_SOURCES")?);
    let specs = indexed_specs(paths.into_iter().map(|p| format!("{p}.c")).collect(), flags);
    let binary = m.build_set(tool, source, &specs, true)?;
    m.publish(tool, "bash-2.05b", binary.clone())?;
    let alias = m.copy(tool, "bash-sh-alias", binary, "", ArtifactKind::Executable)?;
    m.publish(tool, "sh-2.05b", alias)?;
    let smoke =
        m.smoke_stdout("bash-smoke-exec", "bash-builtin-positive", "bash-2.05b", &["-c", "echo bash-ok"], None, 0)?;
    m.publish("bash-smoke-exec", "bash-smoke-output", smoke)?;
    let exe = m.b.artifact("bash-2.05b")?;
    rejected(
        m.b,
        "bash-smoke-exec",
        "bash-reject-malformed",
        exe,
        BTreeMap::new(),
        vec![literal("-c"), literal("if")],
        None,
    )?;
    Ok(())
}

fn compression_smoke(
    m: &mut Middle<'_>,
    name: &str,
    compressor: &str,
    decompressor: &str,
    text: &str,
) -> Result<(), RunError> {
    let tool = format!("{name}-smoke-exec");
    let original = if name == "gzip" { GZIP } else { BZIP };
    let constant = if name == "gzip" {
        "GZIP_MALFORMED_INPUT"
    } else {
        "BZIP2_MALFORMED_INPUT"
    };
    let malformed =
        m.authored_text(&format!("programs/stagex/{name}-malformed.txt"), &bytes_constant(original, constant)?)?;
    let exe = m.b.artifact(decompressor)?;
    rejected(
        m.b,
        &tool,
        &format!("{name}-reject-malformed"),
        exe,
        BTreeMap::from([("stdin".into(), malformed)]),
        vec![literal("-cd")],
        Some("stdin"),
    )?;
    let data = m.authored_text(&format!("programs/stagex/{name}-positive.txt"), text)?;
    let exe = m.b.artifact(compressor)?;
    let packed = one(
        step(
            m.b,
            &tool,
            &format!("{name}-compress-positive"),
            exe,
            BTreeMap::from([("stdin".into(), data)]),
            vec![literal("-c")],
            &[("stdout", ArtifactKind::Archive)],
            None,
            Some("stdin"),
            0,
        )?,
        "stdout",
    )?;
    let unpacked =
        m.smoke_stdout(&tool, &format!("{name}-decompress-positive"), decompressor, &["-cd"], Some(packed), 0)?;
    m.publish(&tool, &format!("{name}-smoke"), unpacked)
}

fn function<'a>(source: &'a str, name: &str) -> Result<&'a str, RunError> {
    let marker = format!("fn {name}(");
    let start = source.find(&marker).ok_or_else(|| error(format!("missing original function {name}")))?;
    let end = source[start..].find("\n}\n").ok_or_else(|| error("unterminated original function"))? + start + 3;
    Ok(&source[start..end])
}
fn calls(source: &str, needle: &str) -> Result<Vec<Vec<String>>, RunError> {
    let mut result = Vec::new();
    let mut tail = source;
    while let Some((_, rest)) = tail.split_once(needle) {
        let mut depth = 1;
        let mut quoted = false;
        let mut escape = false;
        let mut end = None;
        for (i, ch) in rest.char_indices() {
            if quoted {
                if ch == '"' && !escape {
                    quoted = false;
                }
                escape = ch == '\\' && !escape;
            } else {
                match ch {
                    '"' => quoted = true,
                    '(' => depth += 1,
                    ')' => {
                        depth -= 1;
                        if depth == 0 {
                            end = Some(i);
                            break;
                        }
                    }
                    _ => {}
                }
            }
        }
        let end = end.ok_or_else(|| error("unterminated original transformation call"))?;
        result.push(strings(&rest[..end])?);
        tail = &rest[end + 1..];
    }
    Ok(result)
}
fn first_literal_after(source: &str, marker: &str) -> Result<String, RunError> {
    strings(
        source
            .split_once(marker)
            .ok_or_else(|| error(format!("missing original literal marker {marker}")))?
            .1,
    )?
    .into_iter()
    .next()
    .ok_or_else(|| error("missing original source literal"))
}
pub(super) fn tcc_flags(tcc: &str, musl: &str, one_source: bool) -> Vec<String> {
    let prep = tcc.ends_with("tcc-musl-prep");
    let runtime = format!("{tcc}/lib/{}tcc", if prep { "mes/" } else { "" });
    let libraries = if prep {
        format!("{tcc}/lib/mes:{runtime}:{musl}/lib")
    } else {
        format!("{musl}/lib:{runtime}")
    };
    let includes = if one_source {
        format!("{musl}/include")
    } else {
        format!("{runtime}/include:{musl}/include")
    };
    let libgcc = if prep {
        format!("{tcc}/lib/mes/libc.a")
    } else {
        format!("{musl}/lib/libc.a")
    };
    [
        "BOOTSTRAP=1",
        "HAVE_BITFIELD=1",
        "HAVE_FLOAT=1",
        "HAVE_LONG_LONG=1",
        "HAVE_SETJMP=1",
        "TCC_TARGET_X86_64=1",
        "CONFIG_TCCBOOT=1",
        "CONFIG_TCC_STATIC=1",
        "CONFIG_USE_LIBGCC=1",
        "TCC_VERSION=\"0.9.27\"",
        "TCC_LIBTCC1=\"libtcc1.a\"",
        "CONFIG_SYSROOT=\"/\"",
        "CONFIG_TCC_ELFINTERP=\"/lib/ld-musl-x86_64.so.1\"",
    ]
    .into_iter()
    .map(|v| format!("-D{v}"))
    .chain([
        format!("-DCONFIG_TCCDIR=\"{runtime}\""),
        format!("-DCONFIG_TCC_CRTPREFIX=\"{musl}/lib\""),
        format!("-DCONFIG_TCC_LIBPATHS=\"{libraries}\""),
        format!("-DCONFIG_TCC_SYSINCLUDEPATHS=\"{includes}\""),
        format!("-DTCC_LIBGCC=\"{libgcc}\""),
        format!("-DONE_SOURCE={}", u8::from(one_source)),
        "-I.".into(),
    ])
    .collect()
}
impl Middle<'_> {
    fn original_replacements(
        &mut self,
        tool: &str,
        id: &str,
        mut tree: ArtifactRef,
        source: &str,
        path: Option<&str>,
        needle: &str,
    ) -> Result<ArtifactRef, RunError> {
        for (i, values) in calls(source, needle)?.iter().enumerate() {
            let (path, old, new) = match (path, values.as_slice()) {
                (Some(path), [old, new]) => (path, old.as_str(), new.as_str()),
                (_, [path, old, new]) => (path.as_str(), old.as_str(), new.as_str()),
                _ => return Err(error(format!("unsupported original replacement in {id}"))),
            };
            tree = self.replace(tool, &format!("{id}-{i}"), tree, path, old, new)?;
        }
        Ok(tree)
    }
    fn generate_headers(&mut self, tool: &str, id: &str, tree: ArtifactRef) -> Result<ArtifactRef, RunError> {
        self.transform(
            tool,
            id,
            tree,
            vec![
                literal("tree-generate"),
                input("source", ""),
                output("result", ""),
                literal("obj"),
            ],
            BTreeMap::new(),
            ArtifactKind::Tree,
        )
    }
    fn installed_headers(&mut self, tool: &str, id: &str, tree: ArtifactRef) -> Result<ArtifactRef, RunError> {
        self.transform(
            tool,
            id,
            tree,
            vec![literal("headers"), input("source", ""), output("result", "")],
            BTreeMap::new(),
            ArtifactKind::Tree,
        )
    }
    fn assemble(&mut self, tool: &str, id: &str, files: &[(String, ArtifactRef)]) -> Result<ArtifactRef, RunError> {
        // AssembleTree merely binds the already individually produced leaves;
        // all content-changing operations are source-built executable leaves.
        let inputs = files
            .iter()
            .enumerate()
            .map(|(i, (_, reference))| (format!("file{i}"), reference.clone()))
            .collect();
        let entries = files.iter().enumerate().map(|(i, (path, _))| (path.clone(), format!("file{i}"))).collect();
        one(
            self.b.add(tool, RecipeStep {
                id: id.into(),
                inputs,
                outputs: BTreeMap::from([("tree".into(), RecipeOutput {
                    kind: ArtifactKind::Tree,
                    bytes_max: LIMIT,
                })]),
                operation: RecipeOperation::AssembleTree {
                    output: "tree".into(),
                    entries,
                },
            })?,
            "tree",
        )
    }
    fn empty_archive(&mut self, tool: &str, id: &str) -> Result<ArtifactRef, RunError> {
        let exe = self.helper.clone();
        one(
            step(
                self.b,
                tool,
                id,
                exe,
                BTreeMap::new(),
                vec![literal("text"), literal("!<arch>\n"), output("archive", "")],
                &[("archive", ArtifactKind::Archive)],
                None,
                None,
                0,
            )?,
            "archive",
        )
    }
    fn smoke_compile(
        &mut self,
        tool: &str,
        id: &str,
        compiler: &str,
        text: &str,
        headers: ArtifactRef,
    ) -> Result<ArtifactRef, RunError> {
        let file = self.authored_text(&format!("programs/stagex/{id}.c"), text)?;
        let exe = self.b.artifact(compiler)?;
        one(
            step(
                self.b,
                tool,
                id,
                exe,
                BTreeMap::from([("source.c".into(), file), ("headers".into(), headers)]),
                vec![
                    literal("-nostdinc"),
                    literal("-I"),
                    input("headers", ""),
                    literal("-c"),
                    input("source.c", ""),
                    literal("-o"),
                    output("object.o", ""),
                ],
                &[("object.o", ArtifactKind::Object)],
                None,
                None,
                0,
            )?,
            "object.o",
        )
    }
    fn malformed_compile(
        &mut self,
        tool: &str,
        id: &str,
        compiler: &str,
        source: ArtifactRef,
        headers: ArtifactRef,
    ) -> Result<ArtifactRef, RunError> {
        let executable = self.b.artifact(compiler)?;
        rejected(
            self.b,
            tool,
            id,
            executable,
            BTreeMap::from([("source.c".into(), source), ("headers".into(), headers)]),
            vec![
                literal("-nostdinc"),
                literal("-I"),
                input("headers", ""),
                literal("-c"),
                input("source.c", ""),
                literal("-o"),
                output("work", "malformed.o"),
            ],
            None,
        )
    }
    fn musl_link(
        &mut self,
        tool: &str,
        id: &str,
        compiler: &str,
        prefix: &str,
        objects: &[ArtifactRef],
        runtime: &str,
    ) -> Result<ArtifactRef, RunError> {
        let mut ordered = vec![
            self.b.artifact(&format!("{prefix}-crt1"))?,
            self.b.artifact(&format!("{prefix}-crti"))?,
        ];
        ordered.extend_from_slice(objects);
        let extras = [
            ("libc", self.b.artifact(&format!("{prefix}-libc"))?),
            ("runtime", self.b.artifact(runtime)?),
            ("crtn", self.b.artifact(&format!("{prefix}-crtn"))?),
        ];
        self.link(tool, id, compiler, &ordered, &extras)
    }
}

fn bridge(m: &mut Middle<'_>) -> Result<(), RunError> {
    let tool = "tcc-musl-prep-build-exec";
    let source = m.tcc_source(tool, "tcc-musl-prep", "tcc-musl-prep-recipe-source")?;
    m.publish(tool, "tcc-musl-prep-configured-source", source.clone())?;
    let flags = tcc_flags("/stagex/tcc-musl-prep", "/stagex/musl-1.1.24", true);
    let object = m.compile(
        tool,
        "tcc-musl-prep-compile",
        "tinycc-0.9.27",
        source.clone(),
        "tcc.c",
        &flags,
        Some(m.headers.clone()),
    )?;
    let va = m.b.artifact("tinycc27-va-list")?;
    let binary = m.mes_link(tool, "tcc-musl-prep-link", &[object, va], false)?;
    m.publish(tool, "tcc-musl-prep", binary.clone())?;
    let alias = m.copy(tool, "tcc-musl-prep-alias-copy", binary, "", ArtifactKind::Executable)?;
    m.publish(tool, "tcc-musl-prep-alias", alias)?;
    for (suffix, original, kind) in [
        ("libc", "tinycc27-libc", ArtifactKind::Archive),
        ("crt1", "tinycc-crt1", ArtifactKind::Object),
        ("libtcc1", "tinycc-libtcc1", ArtifactKind::Archive),
    ] {
        let original = m.b.artifact(original)?;
        let copy = m.copy(tool, &format!("tcc-musl-prep-copy-{suffix}"), original, "", kind)?;
        m.publish(tool, &format!("tcc-musl-prep-{suffix}"), copy)?;
    }
    let headers = m.tree(tool, "tcc-musl-prep-copy-headers", m.headers.clone())?;
    m.publish(tool, "tcc-musl-prep-headers", headers.clone())?;
    let object = m.smoke_compile(
        "tcc-musl-prep-smoke-exec",
        "tcc-musl-prep-positive",
        "tcc-musl-prep",
        "int bridge_value(void) { return 24; }\n",
        headers,
    )?;
    m.publish("tcc-musl-prep-smoke-exec", "tcc-musl-prep-positive-object", object)?;
    let malformed = m.authored_text("programs/stagex/prep-malformed.c", "int broken( {\n")?;
    m.malformed_compile(
        "tcc-musl-prep-smoke-exec",
        "prep-reject-malformed",
        "tcc-musl-prep",
        malformed,
        m.headers.clone(),
    )?;

    reduced_musl(m, false)?;
    let tool = "tcc-musl-build-exec";
    let source = m.tcc_source(tool, "tcc-musl", "tcc-musl-recipe-source")?;
    m.publish(tool, "tcc-musl-configured-source", source.clone())?;
    let flags = tcc_flags("/stagex/tcc-musl", "/stagex/musl-1.1.24", true);
    let object =
        m.compile(tool, "tcc-musl-compile", "tcc-musl-prep", source.clone(), "tcc.c", &flags, Some(m.headers.clone()))?;
    let extras = [
        ("crt1", m.b.artifact("tcc-musl-prep-crt1")?),
        ("libc", m.b.artifact("tcc-musl-prep-libc")?),
        ("runtime", m.b.artifact("tcc-musl-prep-libtcc1")?),
    ];
    let binary = m.link(tool, "tcc-musl-link", "tcc-musl-prep", &[object], &extras)?;
    m.publish(tool, "tcc-musl", binary.clone())?;
    let alias = m.copy(tool, "tcc-musl-alias-copy", binary, "", ArtifactKind::Executable)?;
    m.publish(tool, "tcc-musl-alias", alias)?;
    let runtime = m.b.artifact("tcc-musl-prep-libtcc1")?;
    let runtime = m.copy(tool, "tcc-musl-runtime-copy", runtime, "", ArtifactKind::Archive)?;
    m.publish(tool, "tcc-musl-libtcc1", runtime)?;
    compiler_smoke(m, "tcc-musl", "musl", "tcc-musl-libtcc1", "int main(void) { return 0; }\n")?;
    reduced_musl(m, true)?;
    v2_and_selfhost(m)
}

fn selected_musl_members(members: &BTreeSet<String>, native: bool) -> Vec<String> {
    let mut base = BTreeMap::new();
    let mut arch = BTreeMap::new();
    for path in members {
        if ![".c", ".s", ".S"].iter().any(|ext| path.ends_with(ext)) {
            continue;
        }
        if !(path.starts_with("src/") || path.starts_with("crt/")) {
            continue;
        }
        if let Some((directory, leaf)) = path.split_once("/x86_64/") {
            arch.insert(format!("{directory}/{leaf}"), path.clone());
        } else if path.matches('/').count() == if path.starts_with("src/") { 2 } else { 1 } {
            base.insert(path.clone(), path.clone());
        }
    }
    for key in arch.keys() {
        base.remove(key);
    }
    let arch_stems =
        arch.keys().filter_map(|p| p.rsplit_once('.').map(|(p, _)| p.to_string())).collect::<BTreeSet<_>>();
    let mut selected = base
        .into_values()
        .filter(|p| !native || !arch_stems.contains(p.rsplit_once('.').map_or(p.as_str(), |(stem, _)| stem)))
        .chain(arch.into_values())
        .collect::<Vec<_>>();
    selected.sort();
    selected
}
fn reduced_musl(m: &mut Middle<'_>, second: bool) -> Result<(), RunError> {
    let prefix = if second { "musl-pass2" } else { "musl" };
    let compiler = if second { "tcc-musl" } else { "tcc-musl-prep" };
    let runtime = if second {
        "tcc-musl-libtcc1"
    } else {
        "tcc-musl-prep-libtcc1"
    };
    let tool = format!("{prefix}-build-exec");
    let mut source = m.archive_source(&tool, "musl-1.1.24-source", "src/internal/libc.h")?;
    let mut removed = list(MUSL, "REMOVED_SOURCE_DIRECTORIES")?;
    removed.extend(["crt/Scrt1.c".into(), "crt/rcrt1.c".into()]);
    source = m.remove(&tool, &format!("{prefix}-remove-sources"), source, &removed)?;
    source = m.replace(
        &tool,
        &format!("{prefix}-va-list"),
        source,
        "arch/x86_64/bits/alltypes.h.in",
        "TYPEDEF __builtin_va_list va_list;",
        "#if defined(__TINYC__)\ntypedef char *__builtin_va_list;\n#endif\nTYPEDEF __builtin_va_list va_list;",
    )?;
    source = m.transform(
        &tool,
        &format!("{prefix}-syscall-cut"),
        source,
        vec![
            literal("tree-truncate"),
            input("source", ""),
            output("result", ""),
            literal("src/internal/syscall.h"),
            literal("fixup legacy"),
            literal("#endif\n"),
        ],
        BTreeMap::new(),
        ArtifactKind::Tree,
    )?;
    let helpers = records(MUSL, if second { "MUSL_PASS2_HELPERS" } else { "MUSL_HELPERS" }, "MuslHelperSpec")?;
    let recipe = m.source(if second {
        "musl-pass2-recipe-source"
    } else {
        "musl-1.1.24-recipe-source"
    })?;
    for (i, helper) in helpers.iter().enumerate() {
        source = m.checked_heredoc(
            &tool,
            &format!("{prefix}-helper-{i}"),
            source,
            recipe.clone(),
            &helper[0],
            &helper[1],
            &format!("cat > {} <<'EOF'\n", helper[1]),
            "\nEOF\n",
        )?;
    }
    source = m.generate_headers(&tool, &format!("{prefix}-generate-headers"), source)?;
    m.publish(&tool, &format!("{prefix}-configured-source"), source.clone())?;
    let mut members = m
        .musl_members()?
        .into_iter()
        .filter(|p| !removed.iter().any(|r| p == r || p.starts_with(&format!("{r}/"))))
        .collect::<BTreeSet<_>>();
    members.extend(helpers.iter().map(|helper| helper[1].clone()));
    let selected = selected_musl_members(&members, false);
    finish_musl(m, prefix, compiler, runtime, source, &selected, false)?;
    let smoke_tool = format!("{prefix}-smoke-exec");
    let headers = m.b.artifact(&format!("{prefix}-headers"))?;
    let object = m.smoke_compile(
        &smoke_tool,
        &format!("{prefix}-smoke-compile"),
        compiler,
        "#include <stddef.h>\nint main(void) { return sizeof(size_t) == 0; }\n",
        headers,
    )?;
    let binary = m.musl_link(&smoke_tool, &format!("{prefix}-smoke-link"), compiler, prefix, &[object], runtime)?;
    m.publish(&smoke_tool, &format!("{prefix}-smoke-binary"), binary)?;
    let malformed = m.authored_text(
        &format!("programs/stagex/{prefix}-malformed.c"),
        &bytes_constant(MUSL, "MALFORMED_SMOKE_SOURCE")?,
    )?;
    let headers = m.b.artifact(&format!("{prefix}-headers"))?;
    m.malformed_compile(&smoke_tool, &format!("{prefix}-reject-malformed"), compiler, malformed, headers)?;
    Ok(())
}
fn finish_musl(
    m: &mut Middle<'_>,
    prefix: &str,
    compiler: &str,
    runtime: &str,
    source: ArtifactRef,
    selected: &[String],
    native: bool,
) -> Result<(), RunError> {
    let tool = format!("{prefix}-build-exec");
    let flags = list(if native { NATIVE } else { MUSL }, "COMMON_COMPILE_FLAGS")?;
    let mut objects = Vec::new();
    let mut libraries = Vec::new();
    for (i, path) in selected.iter().enumerate() {
        let compiler = if native
            && ((path.starts_with("src/") && path.contains("/x86_64/") && path.ends_with(".s"))
                || path.starts_with("src/math/")
                || path == "crt/crt1.c")
        {
            "tcc-musl-v2"
        } else {
            compiler
        };
        let object =
            m.compile(&tool, &format!("{prefix}-compile-{i:04}"), compiler, source.clone(), path, &flags, None)?;
        let object = if native {
            m.canonicalize(&tool, &format!("{prefix}-canonical-{i:04}"), object)?
        } else {
            object
        };
        if path.starts_with("crt/") {
            let name = path
                .rsplit('/')
                .next()
                .and_then(|p| p.rsplit_once('.'))
                .ok_or_else(|| error("invalid original CRT source"))?
                .0;
            let installed = m.copy(&tool, &format!("{prefix}-install-{name}"), object, "", ArtifactKind::Object)?;
            m.publish(&tool, &format!("{prefix}-{name}"), installed.clone())?;
            libraries.push((format!("{name}.o"), installed));
        } else {
            objects.push(object);
        }
    }
    let members = objects
        .into_iter()
        .enumerate()
        .map(|(i, object)| {
            (
                if native {
                    format!("o{i:04}.o")
                } else {
                    format!("libc-{i:04}.o")
                },
                object,
            )
        })
        .collect::<Vec<_>>();
    let libc =
        m.archive(&tool, &format!("{prefix}-archive-libc"), compiler, &members, if native { "rcs" } else { "rc" })?;
    let libc = if native {
        m.transform(
            &tool,
            &format!("{prefix}-normalize-archive"),
            libc,
            vec![literal("normalize-ar"), input("source", ""), output("result", "")],
            BTreeMap::new(),
            ArtifactKind::Archive,
        )?
    } else {
        libc
    };
    m.publish(&tool, &format!("{prefix}-libc"), libc.clone())?;
    libraries.push(("libc.a".into(), libc));
    for name in ["m", "rt", "pthread", "crypt", "util", "xnet", "resolv", "dl"] {
        let archive = m.empty_archive(&tool, &format!("{prefix}-empty-lib{name}"))?;
        m.publish(&tool, &format!("{prefix}-lib{name}"), archive.clone())?;
        libraries.push((format!("lib{name}.a"), archive));
    }
    let headers = m.installed_headers(&tool, &format!("{prefix}-install-headers"), source)?;
    m.publish(&tool, &format!("{prefix}-headers"), headers.clone())?;
    let libraries = m.assemble(&tool, &format!("{prefix}-installed-libraries"), &libraries)?;
    m.b.artifacts.insert(format!("{prefix}-libraries-tree"), libraries.clone());
    if native {
        m.b.artifacts.insert("publication-musl-libraries".into(), libraries);
        m.b.artifacts.insert("publication-musl-headers".into(), headers);
    }
    let _ = runtime;
    Ok(())
}
fn compiler_smoke(m: &mut Middle<'_>, prefix: &str, musl: &str, runtime: &str, text: &str) -> Result<(), RunError> {
    let tool = format!("{prefix}-smoke-exec");
    let headers = m.b.artifact(&format!("{musl}-headers"))?;
    let object = m.smoke_compile(&tool, &format!("{prefix}-positive-compile"), prefix, text, headers)?;
    m.publish(&tool, &format!("{prefix}-positive-object"), object.clone())?;
    let binary = m.musl_link(&tool, &format!("{prefix}-positive-link"), prefix, musl, &[object], runtime)?;
    m.publish(&tool, &format!("{prefix}-positive-binary"), binary)?;
    let malformed = m.authored_text(&format!("programs/stagex/{prefix}-malformed.c"), "int broken( {\n")?;
    let headers = m.b.artifact(&format!("{musl}-headers"))?;
    m.malformed_compile(&tool, &format!("{prefix}-reject-malformed"), prefix, malformed, headers)?;
    Ok(())
}

fn v2_and_selfhost(m: &mut Middle<'_>) -> Result<(), RunError> {
    let tool = "tcc-musl-v2-build-exec";
    let source = m.tcc_source(tool, "tcc-musl-v2", "tcc-musl-v2-recipe-source")?;
    let adjustments = function(V2, "apply_v2_codegen_adjustments")?;
    let (codegen, rest) = adjustments
        .split_once("let library =")
        .ok_or_else(|| error("original v2 library boundary missing"))?;
    let (library, tools) = rest.split_once("let tools =").ok_or_else(|| error("original v2 tools boundary missing"))?;
    let source =
        m.original_replacements(tool, "v2-codegen", source, codegen, Some("x86_64-gen.c"), "replace_required_text(")?;
    let source =
        m.original_replacements(tool, "v2-diagnostics", source, library, Some("libtcc.c"), "replace_required_text(")?;
    let source = m.original_replacements(
        tool,
        "v2-archive-format",
        source,
        tools,
        Some("tcctools.c"),
        "replace_required_text(",
    )?;
    m.publish(tool, "tcc-musl-v2-configured-source", source.clone())?;
    let flags = tcc_flags("runtime/tcc-musl-v2", "runtime/musl-1.1.24-pass2", true);
    let headers26 = m.b.artifact("tinycc26-include-tree")?;
    let object = m.compile(
        tool,
        "tcc-musl-v2-compile",
        "tinycc-0.9.26",
        source.clone(),
        "tcc.c",
        &flags,
        Some(headers26.clone()),
    )?;
    let extras = [
        ("crt1", m.b.artifact("tinycc-crt1")?),
        ("libc", m.b.artifact("tinycc-libc")?),
        ("runtime", m.b.artifact("tinycc-libtcc1")?),
    ];
    let binary = m.link(tool, "tcc-musl-v2-link", "tinycc-0.9.26", &[object], &extras)?;
    m.publish(tool, "tcc-musl-v2", binary.clone())?;
    let alias = m.copy(tool, "tcc-musl-v2-alias-copy", binary, "", ArtifactKind::Executable)?;
    m.publish(tool, "tcc-musl-v2-alias", alias)?;
    let recipe = m.source("tcc-musl-v2-recipe-source")?;
    let runtime_source = m.checked_heredoc(
        tool,
        "tcc-musl-v2-runtime-source",
        source.clone(),
        recipe,
        "tcc-musl-v2-stdarg-runtime-source",
        "tcc-stdarg-runtime.c",
        "cat > /tmp/tcc-stdarg-runtime.c <<'EOF'\n",
        "\nEOF\n",
    )?;
    let runtime_object = m.compile(
        tool,
        "tcc-musl-v2-runtime-compile",
        "tinycc-0.9.26",
        runtime_source,
        "tcc-stdarg-runtime.c",
        &[],
        Some(headers26),
    )?;
    let predecessor = m.b.artifact("tcc-musl-prep-libtcc1")?;
    let runtime =
        m.extend_archive(tool, "tcc-musl-v2-runtime-archive", "tinycc-0.9.26", predecessor, &[runtime_object])?;
    m.publish(tool, "tcc-musl-v2-libtcc1", runtime.clone())?;
    let exported = m.transform(
        tool,
        "tcc-musl-v2-export-source",
        source.clone(),
        vec![literal("source-contract"), input("source", ""), output("result", "")],
        BTreeMap::new(),
        ArtifactKind::Tree,
    )?;
    m.publish(tool, "tcc-musl-v2-source-tree", exported.clone())?;
    let headers = m.transform(
        tool,
        "tcc-musl-v2-export-headers",
        source.clone(),
        vec![literal("tree"), input("source", "include"), output("result", "")],
        BTreeMap::new(),
        ArtifactKind::Tree,
    )?;
    m.publish(tool, "tcc-musl-v2-header-tree", headers.clone())?;
    let runtime_tree = m.assemble(tool, "tcc-musl-v2-installed-runtime", &[
        ("libtcc1.a".into(), runtime),
        ("include".into(), headers.clone()),
    ])?;
    m.b.artifacts.insert("publication-tcc-runtime".into(), runtime_tree);
    m.b.artifacts.insert("publication-tcc-source".into(), exported.clone());
    let original_tree = m.archive_source(tool, "tinycc-0.9.27-source", "tcc.c")?;
    let runtime_original =
        m.copy(tool, "publication-original-runtime-source", original_tree, "lib/libtcc1.c", ArtifactKind::Text)?;
    m.b.artifacts.insert("publication-tcc-runtime-source".into(), runtime_original);
    let positive = bytes_constant(V2, "POSITIVE_SOURCE")?;
    let prefix_header = m.copy(tool, "v2-smoke-prefix-header", source, "include/stdarg.h", ArtifactKind::Text)?;
    let positive = m.b.authored("programs/stagex/v2-positive.c", positive)?;
    let malformed = m.authored_text("programs/stagex/v2-malformed.c", &bytes_constant(V2, "MALFORMED_SOURCE")?)?;
    let malformed_tree = m.assemble("tcc-musl-v2-smoke-exec", "v2-malformed-source-tree", &[
        ("tcc-stdarg-prefix.h".into(), prefix_header.clone()),
        ("negative.c".into(), malformed),
    ])?;
    let exe = m.b.artifact("tcc-musl-v2")?;
    rejected(
        m.b,
        "tcc-musl-v2-smoke-exec",
        "v2-reject-malformed",
        exe,
        BTreeMap::from([("source".into(), malformed_tree)]),
        vec![
            literal("-I"),
            input("source", ""),
            literal("-c"),
            input("source", "negative.c"),
            literal("-o"),
            output("work", "negative.o"),
        ],
        None,
    )?;
    let smoke_tree = m.assemble("tcc-musl-v2-smoke-exec", "v2-smoke-source-tree", &[
        ("tcc-stdarg-prefix.h".into(), prefix_header),
        ("positive.c".into(), positive),
    ])?;
    let object = m.compile(
        "tcc-musl-v2-smoke-exec",
        "tcc-musl-v2-positive-compile",
        "tcc-musl-v2",
        smoke_tree,
        "positive.c",
        &["-I.".into()],
        Some(headers),
    )?;
    m.publish("tcc-musl-v2-smoke-exec", "tcc-musl-v2-positive-object", object.clone())?;
    let binary = m.musl_link(
        "tcc-musl-v2-smoke-exec",
        "tcc-musl-v2-positive-link",
        "tcc-musl-v2",
        "musl-pass2",
        &[object],
        "tcc-musl-v2-libtcc1",
    )?;
    m.publish("tcc-musl-v2-smoke-exec", "tcc-musl-v2-positive-binary", binary.clone())?;
    let execution = one(
        step(
            m.b,
            "tcc-musl-v2-execution-exec",
            "tcc-musl-v2-positive-execute",
            binary,
            BTreeMap::new(),
            Vec::new(),
            &[("stdout", ArtifactKind::Text)],
            None,
            None,
            0,
        )?,
        "stdout",
    )?;
    m.publish("tcc-musl-v2-execution-exec", "tcc-musl-v2-positive-execution", execution)?;
    selfhost(m, exported)
}
impl Middle<'_> {
    fn extend_archive(
        &mut self,
        tool: &str,
        id: &str,
        compiler: &str,
        archive: ArtifactRef,
        objects: &[ArtifactRef],
    ) -> Result<ArtifactRef, RunError> {
        let [object] = objects else {
            return Err(error("original variadic runtime must add exactly one object"));
        };
        let exe = self.b.artifact(compiler)?;
        one(
            self.b.add(tool, RecipeStep {
                id: id.into(),
                inputs: BTreeMap::from([
                    ("exe".into(), exe),
                    ("predecessor.a".into(), archive),
                    ("tcc-stdarg-runtime.o".into(), object.clone()),
                ]),
                outputs: [
                    ("archive", ArtifactKind::Archive),
                    ("stdout", ArtifactKind::Text),
                    ("stderr", ArtifactKind::Text),
                ]
                .into_iter()
                .map(|(name, kind)| (name.into(), RecipeOutput { kind, bytes_max: LIMIT }))
                .collect(),
                operation: RecipeOperation::Exec {
                    command_state: None,
                    executable_input: "exe".into(),
                    invocation_input: None,
                    arguments: vec![
                        literal("-ar"),
                        literal("rcs"),
                        output("archive", ""),
                        input("tcc-stdarg-runtime.o", ""),
                    ],
                    environment: BTreeMap::new(),
                    runtime_mounts: BTreeMap::new(),
                    output_seeds: BTreeMap::from([("archive".into(), "predecessor.a".into())]),
                    guard: None,
                    merge_stderr: false,
                    cwd_input: None,
                    cwd_output: None,
                    stdin_input: None,
                    exit_policy: crate::source_built_recipe::RecipeExitPolicy::Exact { code: 0 },
                },
            })?,
            "archive",
        )
    }
    fn canonicalize(&mut self, tool: &str, id: &str, object: ArtifactRef) -> Result<ArtifactRef, RunError> {
        self.transform(
            tool,
            id,
            object,
            vec![literal("canonical-elf"), input("source", ""), output("result", "")],
            BTreeMap::new(),
            ArtifactKind::Object,
        )
    }
}
fn selfhost(m: &mut Middle<'_>, mut source: ArtifactRef) -> Result<(), RunError> {
    let tool = "tcc-musl-selfhost-build-exec";
    source = m.empty_tcc_config(tool, "selfhost-empty-config", source, "tcc-musl-selfhost-recipe-source")?;
    source = m.original_replacements(
        tool,
        "selfhost-main-adjustments",
        source,
        function(SELFHOST, "apply_selfhost_adjustments")?,
        None,
        "replace_required_text(",
    )?;
    source = m.original_replacements(
        tool,
        "selfhost-weak-symbols",
        source,
        function(SELFHOST, "add_weak_archive_symbols")?,
        Some("tcctools.c"),
        "replace_required_text(",
    )?;
    source = m.original_replacements(
        tool,
        "selfhost-got-adjustments",
        source,
        function(SELFHOST, "patch_got_handling")?,
        Some("tccelf.c"),
        "replace_required_text(",
    )?;
    let got = strings(function(SELFHOST, "local_got_replacement")?)?
        .into_iter()
        .next()
        .ok_or_else(|| error("original GOT replacement missing"))?;
    source = m.transform(
        tool,
        "selfhost-local-got-function",
        source,
        vec![
            literal("tree-function"),
            input("source", ""),
            output("result", ""),
            literal("tccelf.c"),
            literal("static void fill_local_got_entries"),
            literal(&got),
        ],
        BTreeMap::new(),
        ArtifactKind::Tree,
    )?;
    let diagnostic = first_literal_after(function(SELFHOST, "restore_variadic_diagnostics")?, "let replacement = ")?;
    source = m.range(
        tool,
        "selfhost-variadic-diagnostics",
        source,
        "libtcc.c",
        "static void crunch_diag_putc",
        "static void strcat_printf",
        &(diagnostic + "static void strcat_printf"),
    )?;
    m.publish(tool, "tcc-musl-selfhost-patched-source", source.clone())?;
    let flags = tcc_flags("output", "../../musl-pass2-stage/runtime/output", false);
    let headers = m.b.artifact("tinycc26-include-tree")?;
    let paths = list(SELFHOST, "COMPILER_SOURCES")?;
    let mut objects = Vec::new();
    for (i, path) in paths.iter().enumerate() {
        objects.push(m.compile(
            tool,
            &format!("selfhost-compile-{i}"),
            "tcc-musl-v2",
            source.clone(),
            path,
            &flags,
            Some(headers.clone()),
        )?);
    }
    let members = paths
        .iter()
        .skip(1)
        .zip(objects.iter().skip(1))
        .map(|(path, object)| (path.replace(".c", ".o"), object.clone()))
        .collect::<Vec<_>>();
    let archive = m.archive(tool, "selfhost-archive-before-canonicalization", "tcc-musl-v2", &members, "rc")?;
    let extras = [
        ("crt1", m.b.artifact("tinycc-crt1")?),
        ("libc", m.b.artifact("tinycc-libc")?),
        ("runtime", m.b.artifact("tinycc-libtcc1")?),
    ];
    let binary = m.link(tool, "selfhost-predecessor-link", "tinycc-0.9.26", &[objects[0].clone(), archive], &extras)?;
    m.publish(tool, "tcc-musl-selfhost", binary.clone())?;
    let alias = m.copy(tool, "selfhost-alias-copy", binary, "", ArtifactKind::Executable)?;
    m.publish(tool, "tcc-musl-selfhost-alias", alias)?;
    let mut retained = Vec::new();
    for (i, (path, object)) in paths.iter().zip(objects).enumerate() {
        let canonical = m.canonicalize(tool, &format!("selfhost-canonical-object-{i}"), object)?;
        retained.push((path.replace(".c", ".o"), canonical));
    }
    let archive = m.archive(tool, "selfhost-retained-canonical-archive", "tcc-musl-v2", &retained[1..], "rc")?;
    m.publish(tool, "tcc-musl-selfhost-libtcc", archive)?;
    m.publish(tool, "tcc-musl-selfhost-main-object", retained[0].1.clone())?;
    let tree = m.assemble(tool, "selfhost-retained-object-tree", &retained)?;
    m.publish(tool, "tcc-musl-selfhost-object-tree", tree)?;
    let smoke = bytes_constant(SELFHOST, "SMOKE_SOURCE")?;
    let headers = m.b.artifact("musl-pass2-headers")?;
    let object = m.smoke_compile(
        "tcc-musl-selfhost-smoke-exec",
        "selfhost-positive-compile",
        "tcc-musl-selfhost",
        &smoke,
        headers,
    )?;
    m.publish("tcc-musl-selfhost-smoke-exec", "tcc-musl-selfhost-smoke-object", object)?;
    native_musl(m)
}

fn native_musl(m: &mut Middle<'_>) -> Result<(), RunError> {
    let tool = "musl-native-build-exec";
    let mut source = m.archive_source(tool, "musl-1.1.24-source", "src/internal/libc.h")?;
    let mut removed = list(NATIVE, "REMOVED_DIRECTORIES")?;
    removed.extend(list(NATIVE, "REMOVED_FILES")?);
    let original_members = m.musl_members()?.into_iter().collect::<BTreeSet<_>>();
    // Preserve the declared upstream math files before removing src/math.
    let mut preserved = Vec::new();
    for (i, name) in list(NATIVE, "PRESERVED_MATH_FILES")?.iter().enumerate() {
        let path = format!("src/math/{name}");
        preserved.push((
            path.clone(),
            m.copy(tool, &format!("native-musl-preserve-math-{i}"), source.clone(), &path, ArtifactKind::Text)?,
        ));
    }
    source = m.remove(tool, "native-musl-remove-declared", source, &removed)?;
    for (i, (path, file)) in preserved.iter().enumerate() {
        source = m.install(tool, &format!("native-musl-restore-math-{i}"), source, file.clone(), path)?;
    }
    for (i, line) in list(NATIVE, "HEADER_DECLARATIONS_REMOVED")?.iter().enumerate() {
        for (j, path) in ["include/wchar.h", "include/wctype.h"].iter().enumerate() {
            source = m.transform(
                tool,
                &format!("native-musl-remove-wide-declaration-{i}-{j}"),
                source,
                vec![
                    literal("tree-declaration"),
                    input("source", ""),
                    output("result", ""),
                    literal(path),
                    literal(line),
                ],
                BTreeMap::new(),
                ArtifactKind::Tree,
            )?;
        }
    }
    let recipe = m.source("musl-native-recipe-source")?;
    let helpers = list(NATIVE, "HELPER_TARGETS")?;
    for (i, path) in helpers.iter().enumerate() {
        source = m.heredoc(
            tool,
            &format!("native-musl-helper-{i}"),
            source,
            recipe.clone(),
            path,
            &format!("$BB cat > {path} <<'EOF'\n"),
            "\nEOF\n",
        )?;
    }
    let rewrites = function(NATIVE, "apply_text_rewrites")?;
    source = m.original_replacements(tool, "native-musl-text-rewrite", source, rewrites, None, "replace_required(")?;
    source = m.replace(
        tool,
        "native-musl-tls-declaration",
        source,
        "src/env/__init_tls.c",
        "volatile int __thread_list_lock;",
        "hidden void *__bootstrap_anonymous_mmap(size_t);\nvolatile int __thread_list_lock;",
    )?;
    source = m.range(
        tool,
        "native-musl-tls-mmap",
        source,
        "src/env/__init_tls.c",
        "mem = (void *)__syscall(",
        "0);",
        "mem = __bootstrap_anonymous_mmap(libc.tls_size);",
    )?;
    for (i, tokens) in calls(function(NATIVE, "lower_assembly_constants")?, "replace_tokens(")?.iter().enumerate() {
        let path = tokens.first().ok_or_else(|| error("original assembly token path missing"))?;
        if path.ends_with(".s") {
            source = m.transform(
                tool,
                &format!("native-musl-equ-lines-{i}"),
                source,
                vec![
                    literal("tree-lines"),
                    input("source", ""),
                    output("result", ""),
                    literal(path),
                    literal(".equ "),
                ],
                BTreeMap::new(),
                ArtifactKind::Tree,
            )?;
        }
        for (j, pair) in tokens[1..].chunks_exact(2).enumerate() {
            source =
                m.replace(tool, &format!("native-musl-assembly-token-{i}-{j}"), source, path, &pair[0], &pair[1])?;
        }
    }
    let prefixes = list(NATIVE, "REMOVED_GLOB_PREFIXES")?;
    let direct = [
        "src/thread/pthread_create.c",
        "src/thread/pthread_join.c",
        "src/thread/pthread_detach.c",
        "src/thread/pthread_cancel.c",
        "src/thread/pthread_getspecific.c",
        "src/thread/pthread_setspecific.c",
    ];
    let glob_removed = original_members
        .iter()
        .filter(|path| {
            (path.ends_with(".c") && prefixes.iter().any(|prefix| path.starts_with(prefix)))
                || direct.contains(&path.as_str())
        })
        .cloned()
        .collect::<Vec<_>>();
    source = m.remove(tool, "native-musl-remove-thread-globs", source, &glob_removed)?;
    source = m.generate_headers(tool, "native-musl-generate-headers", source)?;
    let alltypes = function(NATIVE, "rewrite_alltypes_text")?;
    let prefix = first_literal_after(alltypes, "const PREFIX: &str = ")?;
    let va_list = first_literal_after(alltypes, "let va_list = ")?;
    let isoc = first_literal_after(alltypes, "let isoc = ")?;
    source = m.replace(
        tool,
        "native-musl-alltypes-va-list",
        source,
        "obj/include/bits/alltypes.h",
        "typedef __builtin_va_list va_list;",
        &va_list,
    )?;
    source = m.replace(
        tool,
        "native-musl-alltypes-isoc-list",
        source,
        "obj/include/bits/alltypes.h",
        "typedef __builtin_va_list __isoc_va_list;",
        &isoc,
    )?;
    source = m.transform(
        tool,
        "native-musl-alltypes-prefix",
        source,
        vec![
            literal("tree-prepend"),
            input("source", ""),
            output("result", ""),
            literal("obj/include/bits/alltypes.h"),
            literal(&prefix),
        ],
        BTreeMap::new(),
        ArtifactKind::Tree,
    )?;
    m.publish(tool, "musl-native-configured-source", source.clone())?;
    let mut members = original_members
        .into_iter()
        .filter(|p| !removed.iter().any(|r| p == r || p.starts_with(&format!("{r}/"))) && !glob_removed.contains(p))
        .collect::<BTreeSet<_>>();
    members.extend(preserved.into_iter().map(|(path, _)| path));
    members.extend(helpers);
    let selected = selected_musl_members(&members, true);
    finish_musl(m, "musl-native", "tcc-musl-selfhost", "tcc-musl-v2-libtcc1", source, &selected, true)?;
    let smoke_recipe = include_str!("../bootstrap/musl-1.1.24-native.ncl");
    let marker = "\"$WORK/native-musl-smoke.c\"";
    let start = smoke_recipe.find(marker).ok_or_else(|| error("original native musl positive source missing"))?;
    let tail = &smoke_recipe[start..];
    let start = tail.find("<<'EOF'\n").ok_or_else(|| error("original native musl positive heredoc missing"))?
        + "<<'EOF'\n".len();
    let end = tail[start..]
        .find("\nEOF\n")
        .ok_or_else(|| error("original native musl positive source unterminated"))?
        + start
        + 1;
    let text = &tail[start..end];
    let headers = m.b.artifact("musl-native-headers")?;
    let object =
        m.smoke_compile("musl-native-smoke-exec", "native-musl-positive-compile", "tcc-musl-selfhost", text, headers)?;
    let binary = m.musl_link(
        "musl-native-smoke-exec",
        "native-musl-positive-link",
        "tcc-musl-selfhost",
        "musl-native",
        &[object],
        "tcc-musl-v2-libtcc1",
    )?;
    m.publish("musl-native-smoke-exec", "musl-native-smoke-binary", binary.clone())?;
    let execution = one(
        step(
            m.b,
            "musl-native-execution-exec",
            "native-musl-positive-execute",
            binary,
            BTreeMap::new(),
            Vec::new(),
            &[("stdout", ArtifactKind::Text)],
            None,
            None,
            0,
        )?,
        "stdout",
    )?;
    let malformed =
        m.authored_text("programs/stagex/native-musl-malformed.c", &bytes_constant(NATIVE, "MALFORMED_SMOKE_SOURCE")?)?;
    let headers = m.b.artifact("musl-native-headers")?;
    let rejection = m.malformed_compile(
        "musl-native-execution-exec",
        "native-musl-reject-malformed",
        "tcc-musl-v2",
        malformed,
        headers,
    )?;
    let receipt = m.transform(
        "musl-native-execution-exec",
        "native-musl-validation-receipt",
        execution,
        vec![
            literal("gate-text"),
            input("source", ""),
            output("result", ""),
            literal("native-musl-smoke-ok\n"),
            input("rejection", ""),
            literal(&bytes_constant(NATIVE, "VALIDATION_RECEIPT_BYTES")?),
        ],
        BTreeMap::from([("rejection".into(), rejection)]),
        ArtifactKind::Text,
    )?;
    m.publish("musl-native-execution-exec", "musl-native-validation-receipt", receipt)
}
// The phase capability is a separately linked original Make 3.82. The public
// MES-linked Make above is untouched; the controller needs the later, genuine
// musl runtime for seekable request descriptors and replay filesystem state.
fn make_control(m: &mut Middle<'_>) -> Result<(), RunError> {
    let tool = "make-build-exec";
    let mut source = m.b.artifact("make-3.82-configured-source")?;
    source = m.text(tool, "make-control-config", source, "config.h", "")?;
    source = m.text(
        tool,
        "make-control-header",
        source,
        "mantle_make_control.h",
        include_str!("source_built_native_programs_host_make_control.h"),
    )?;
    for (i, path) in ["main.c", "job.c", "function.c", "commands.c"].iter().enumerate() {
        let header = if *path == "main.c" {
            "#include \"make.h\"\n#include \"mantle_make_control.h\"\n#define main mantle_original_make_main"
        } else {
            "#include \"make.h\"\n#include \"mantle_make_control.h\""
        };
        source = m.replace(tool, &format!("make-control-include-{i}"), source, path, "#include \"make.h\"", header)?;
    }
    // 3.82 forks in the callers, unlike 4.4's child_execute_job. Intercept
    // before either vfork, leaving the original parent/reap transitions intact.
    let patches = [
        ("main.c", "  exit (status);", "  mantle_make_exit (status);"),
        ("job.c", "pid = WAIT_NOHANG (&status);", "pid = mantle_make_wait (&status, 1);"),
        ("job.c", "EINTRLOOP(pid, wait (&status));", "EINTRLOOP(pid, mantle_make_wait (&status, 0));"),
        (
            "job.c",
            "child->pid = vfork ();",
            "child->pid = mantle_make_active () ? mantle_make_child (argv, child->environment, child->good_stdin ? 0 : bad_stdin, 1, 2) : vfork ();",
        ),
        (
            "job.c",
            "exec_command (char **argv, char **envp)\n{",
            "exec_command (char **argv, char **envp)\n{\n  if (mantle_make_active ()) mantle_make_replace (argv, envp);",
        ),
        ("function.c", "if (pipe (pipedes) < 0)", "if (mantle_make_pipe (pipedes) < 0)"),
        (
            "function.c",
            "pid = vfork ();",
            "pid = mantle_make_active () ? mantle_make_child (command_argv, envp, 0, pipedes[1], 2) : vfork ();",
        ),
        ("commands.c", "kill (c->pid, SIGTERM)", "mantle_make_cancel (c->pid, SIGTERM)"),
    ];
    for (i, (path, old, new)) in patches.iter().enumerate() {
        source = m.replace(tool, &format!("make-control-source-hook-{i}"), source, path, old, new)?;
    }
    let headers = m.b.artifact("musl-native-headers")?;
    let common = list(MAKE, "MAKE_COMMON_FLAGS")?;
    let mut objects = Vec::new();
    for (i, record) in records(MAKE, "MAKE_COMPILE_SPECS", "MakeCompileSpec")?.iter().enumerate() {
        let mut flags = common.clone();
        flags.extend(record.iter().skip(2).cloned());
        flags.extend(["-D_GNU_SOURCE".into(), "-nostdinc".into()]);
        objects.push(m.compile(
            tool,
            &format!("make-control-compile-{i:04}"),
            "tcc-musl-v2",
            source.clone(),
            &record[0],
            &flags,
            Some(headers.clone()),
        )?);
    }
    let mut bridge = crate::source_built_stagex_programs::shellvm::library_source()?;
    bridge.push_str(&include_str!("source_built_native_programs_host_make_control.c").replace(
        "#include \"mantle_make_control.h\"",
        include_str!("source_built_native_programs_host_make_control.h"),
    ));
    let bridge = m.authored_text("programs/stagex/make-3.82-control.c", &bridge)?;
    let bridge = m.assemble(tool, "make-control-bridge-source", &[("control.c".into(), bridge)])?;
    objects.push(m.compile(
        tool,
        "make-control-bridge-compile",
        "tcc-musl-v2",
        bridge,
        "control.c",
        &["-D_GNU_SOURCE".into(), "-nostdinc".into(), "-std=gnu11".into()],
        Some(headers),
    )?);
    let binary =
        m.musl_link(tool, "make-3.82-control-link", "tcc-musl-v2", "musl-native", &objects, "tcc-musl-v2-libtcc1")?;
    m.b.artifacts.insert("phase-tool:make-3.82".into(), binary);
    Ok(())
}

// Recompile the identical retained original translation units against musl:
// MES's stat/dirent layouts cannot be mixed with the native command runtime.
// Support units are archived once per original package, not once per command.
fn command_variants(m: &mut Middle<'_>) -> Result<(), RunError> {
    let headers = m.b.artifact("musl-native-headers")?;
    for (group_index, group) in std::mem::take(&mut m.command_groups).into_iter().enumerate() {
        let mut support = Vec::new();
        for (i, (path, flags)) in group.support.iter().enumerate() {
            let mut flags = flags.clone();
            flags.extend(["-D_GNU_SOURCE".into(), "-nostdinc".into()]);
            let object = m.compile(
                &group.tool,
                &format!("command-support-{group_index}-{i:04}"),
                "tcc-musl-v2",
                group.source.clone(),
                path,
                &flags,
                Some(headers.clone()),
            )?;
            support.push((format!("unit-{i:04}.o"), object));
        }
        let support = if support.is_empty() {
            None
        } else {
            Some(m.archive(
                &group.tool,
                &format!("command-support-{group_index}-archive"),
                "tcc-musl-v2",
                &support,
                "rcs",
            )?)
        };
        for (name, specs) in group.commands {
            let mut objects = Vec::new();
            for (i, (path, flags)) in specs.iter().enumerate() {
                let mut flags = flags.clone();
                flags.extend(["-D_GNU_SOURCE".into(), "-nostdinc".into()]);
                objects.push(m.compile(
                    &group.tool,
                    &format!("command-{name}-{i:04}"),
                    "tcc-musl-v2",
                    group.source.clone(),
                    path,
                    &flags,
                    Some(headers.clone()),
                )?);
            }
            if let Some(support) = &support {
                objects.push(support.clone());
            }
            let executable = m.musl_link(
                &group.tool,
                &format!("command-{name}-link"),
                "tcc-musl-v2",
                "musl-native",
                &objects,
                "tcc-musl-v2-libtcc1",
            )?;
            // These two original aliases are explicitly argv[0]-selected modes
            // of the same upstream program, not substitutes for other tools.
            if name == "gzip-1.2.4" {
                m.b.artifacts.insert("phase-tool:gunzip-1.2.4".into(), executable.clone());
            }
            if name == "coreutils-test" {
                m.b.artifacts.insert("phase-tool:coreutils-bracket".into(), executable.clone());
            }
            m.b.artifacts.insert(format!("phase-tool:{name}"), executable);
        }
    }
    let tool = "tcc-musl-v2-build-exec";
    let source = m.b.artifact("tcc-musl-v2-configured-source")?;
    let source=m.replace(tool,"tcc-writer-declaration",source,"tccelf.c",
        "static int tcc_write_elf_file(TCCState *s1, const char *filename, int phnum,",
        "extern void mantle_native_generated_created(const char *filename);\nstatic int tcc_write_elf_file(TCCState *s1, const char *filename, int phnum,")?;
    // Original 0.9.27 tccelf.c closes the actual output stream here. Preserve
    // its return policy, but never catalogue bytes after a failed close.
    let source=m.replace(tool,"tcc-successful-writer",source,"tccelf.c",
        "    fclose(f);\n\n    return 0;\n}",
        "    if (fclose(f) == 0 && file_type == TCC_OUTPUT_EXE && s1->output_format == TCC_OUTPUT_FORMAT_ELF)\n        mantle_native_generated_created(filename);\n\n    return 0;\n}")?;
    let mut flags = tcc_flags("runtime/tcc-musl-v2", "runtime/musl-1.1.24-pass2", true);
    flags.extend(["-D_GNU_SOURCE".into(), "-nostdinc".into()]);
    let compiler =
        m.compile(tool, "tcc-native-command-main", "tcc-musl-v2", source, "tcc.c", &flags, Some(headers.clone()))?;
    let publisher = m.authored_text(
        "programs/stagex/tcc-generated-writer.c",
        include_str!("source_built_native_programs_generated.c"),
    )?;
    let publisher = m.assemble(tool, "tcc-generated-writer-source", &[("writer.c".into(), publisher)])?;
    let publisher = m.compile(
        tool,
        "tcc-generated-writer-compile",
        "tcc-musl-v2",
        publisher,
        "writer.c",
        &["-D_GNU_SOURCE".into(), "-nostdinc".into()],
        Some(headers),
    )?;
    let executable = m.musl_link(
        tool,
        "tcc-native-command-link",
        "tcc-musl-v2",
        "musl-native",
        &[compiler, publisher],
        "tcc-musl-v2-libtcc1",
    )?;
    m.b.artifacts.insert("phase-tool:tcc-musl-v2".into(), executable.clone());
    m.b.artifacts.insert("phase-tool:tcc-musl-v2-alias".into(), executable);
    Ok(())
}

pub(super) fn extend(builder: &mut TransitionBuilder, _request: &AuthoringRequest<'_>) -> Result<(), RunError> {
    let headers = builder.artifact("tinycc27-include-tree")?;
    let source = builder.authored("helpers/stagex-middle-source.c", HELPER.into())?;
    let compiler = builder.artifact("tinycc-0.9.27")?;
    let inputs = BTreeMap::from([
        ("source.c".into(), source),
        ("headers".into(), headers.clone()),
        ("crt1.o".into(), builder.artifact("tinycc-crt1")?),
        ("libc.a".into(), builder.artifact("tinycc27-libc")?),
        ("libtcc1.a".into(), builder.artifact("tinycc-libtcc1")?),
    ]);
    let helper = one(
        step(
            builder,
            "make-recipe-runner-build-exec",
            "middle-source-helper-build",
            compiler,
            inputs,
            vec![
                literal("-static"),
                literal("-nostdlib"),
                literal("-I"),
                input("headers", ""),
                literal("-o"),
                output("helper", ""),
                input("source.c", ""),
                input("crt1.o", ""),
                input("libc.a", ""),
                input("libtcc1.a", ""),
            ],
            &[("helper", ArtifactKind::Executable)],
            None,
            None,
            0,
        )?,
        "helper",
    )?;
    let mut middle = Middle {
        b: builder,
        helper,
        headers,
        command_groups: Vec::new(),
    };
    utilities(&mut middle)?;
    bridge(&mut middle)?;
    make_control(&mut middle)?;
    command_variants(&mut middle)
}
