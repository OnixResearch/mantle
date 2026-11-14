//! Native-script graph admission and opt-in protected C-driver regression.
//! Bounded C protocol producers are fixtures, not an original Rust compiler,
//! Cargo build-script, native provider, or fixed-point proof.
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use super::*;
use crate::source_built_fixed_point::ProofHermeticityMode;
use crate::source_built_fixed_point::SourceAuthorityInput;
use crate::source_built_fixed_point::SourceAuthorityRole;
use crate::source_built_fixed_point::SourceBuiltFixedPointPlan;
use crate::source_built_fixed_point::SourceBuiltFixedPointPolicies;
use crate::source_built_fixed_point::SourceBuiltFixedPointResourceBounds;
use crate::source_built_fixed_point::SourceContentKind;
use crate::source_built_fixed_point_action_trust::ActionObservationState;
use crate::source_built_fixed_point_action_trust::ExecutionEventKind;
use crate::source_built_fixed_point_planning::InitialOutputAuthorityState;
use crate::source_built_fixed_point_planning::SourceBuiltFixedPointPlanInput;
use crate::source_built_fixed_point_planning::plan_source_built_fixed_point;
use crate::source_built_recipe::RecipePaths;
use crate::source_built_recipe::digest;
use crate::source_built_recipe::prepare;
use crate::source_built_recipe::run;
use crate::source_built_stagex_programs::controller::ControllerExecutable;
use crate::source_built_stagex_programs::controller::ControllerResult;
use crate::source_built_stagex_programs::controller::ControllerSlot;
use crate::source_built_stagex_programs::controller::ControllerSpec;
use crate::source_built_stagex_programs::controller::append_controller;

const NATIVE_SOURCE: &str = r#"
#define _GNU_SOURCE 1
#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
int main(int argc,char **argv){
 assert(argc==2&&!strcmp(argv[0],"/inputs/images/llvm-config"));
 int n=atoi(argv[1]);assert(n>=0&&n<2);
 assert(getenv("SMOKE_VALUE")&&!strcmp(getenv("SMOKE_VALUE"),"value beta"));
 char cwd[256];assert(getcwd(cwd,sizeof cwd)&&!strcmp(cwd,"/tmp/package"));
 char expected[64],actual[64];int size=snprintf(expected,sizeof expected,"input-%d\n",n);
 assert(fread(actual,1,sizeof actual,stdin)==(size_t)size&&!memcmp(actual,expected,size));
 char side[128];snprintf(side,sizeof side,"/tmp/native-side-%d",n);
 FILE *f=fopen(side,"wb");assert(f&&fwrite(expected,1,size,f)==(size_t)size&&!fclose(f));
 printf("native-%d",n);assert(fwrite("\0\xff\n",1,3,stdout)==3);
 fprintf(stderr,"native diagnostic %d\n",n);
 return n==0?23:0;
}
"#;

// Compiled together with the production controller library. Record encoding,
// descriptor snapshots, tree copies and timestamp capture are shared machinery;
// only the bounded request producer and its replay assertions are fixture code.
const FIXTURE_SOURCE: &str = r#"
static void smoke_expect(int condition,const char *message){if(!condition)die("native smoke: %s",message);}
static void smoke_bytes(const char *path,const void *bytes,size_t size){FILE *f=fopen(path,"wb");if(!f||fwrite(bytes,1,size,f)!=size||fclose(f))die("native smoke write %s",path);}
static void smoke_check(const char *path,const void *bytes,size_t size){FILE *f=fopen(path,"rb");if(!f)die("native smoke read %s",path);unsigned char actual[256];size_t n=fread(actual,1,sizeof actual,f);smoke_expect(!ferror(f)&&!fclose(f)&&n==size&&!memcmp(actual,bytes,size),path);}
static void smoke_executable(const char *from,const char *to){binary_copy(from,to);if(chmod(to,0755))die("native smoke chmod");}
static void smoke_frame(FILE *f,const char *key,const char *value){record(f,key);record(f,value);}
static void smoke_bootstrap(int argc,char **argv){
 smoke_expect(argc==12,"bootstrap arguments");
 smoke_executable("/proc/self/exe",argv[2]);smoke_executable("/proc/self/exe",argv[3]);
 smoke_bytes(argv[4],smoke_native_image,sizeof smoke_native_image);if(chmod(argv[4],0755))die("native image mode");
 mkdir_checked(argv[5]);char *script=join(argv[5],"script"),*native=join(argv[5],"llvm-config");smoke_executable(argv[3],script);
 if(!strcmp(argv[11],"tamper"))smoke_executable(argv[3],native);else smoke_executable(argv[4],native);free(script);free(native);
 const char *entry=argv[6];mkdir_checked(entry);char *initial=join(entry,"initial"),*tmp=join(initial,"tmp");mkdir_checked(initial);mkdir_checked(tmp);
 char *package=join(tmp,"package");mkdir_checked(package);free(package);
 char *rust=join(tmp,"mantle-rust"),*id=join(rust,"smoke"),*out=join(id,"out");mkdir_checked(rust);mkdir_checked(id);mkdir_checked(out);free(rust);free(id);free(out);free(tmp);free(initial);
 text_file(entry,"source.sh",":\n");text_file(entry,"cwd",!strcmp(argv[11],"invalid-cwd")?"tmp/package":"/tmp/package");text_file(entry,"zero","/inputs/images/script");
 FILE *f=open_record(entry,"environment","wb");
 smoke_frame(f,"MANTLE_RUST_SCRIPT_IMAGE","/inputs/images/script");smoke_frame(f,"MANTLE_RUST_SCRIPT_SLOTS",argv[10]);
 smoke_frame(f,"MANTLE_RUST_NATIVE_COMMANDS_MAX","1");smoke_frame(f,"OUT_DIR","/tmp/mantle-rust/smoke/out");
 smoke_frame(f,"MANTLE_CC_REQUEST","/tmp/mantle-rust/smoke/out/.mantle-successor-request");smoke_frame(f,"SMOKE_COUNT",argv[9]);smoke_frame(f,"MANTLE_CONTROLLER_ROOTS","/inputs/command-roots");if(!strcmp(argv[11],"native-bound"))smoke_frame(f,"SMOKE_NATIVE_BOUND","1");if(!strcmp(argv[11],"history"))smoke_frame(f,"SMOKE_HISTORY_TAMPER","1");if(!strcmp(argv[11],"script-failure"))smoke_frame(f,"SMOKE_SCRIPT_FAILURE","1");if(fclose(f))die("smoke environment close");
 f=fopen(argv[7],"wb");if(!f)die("smoke roots");record(f,"MANTLE-CONTROLLER-ROOTS-1");
 for(int i=0;i<3;i++)record(f,(const char *[]){"mutable","/tmp","tmp"}[i]);
 record(f,"input");record(f,"/inputs/images");record(f,"images");if(fclose(f))die("smoke roots close");
 smoke_expect(!strcmp(argv[8],"bounded-c-protocol-only"),"fixture non-claim");
}
static void smoke_capture_times(const char *root){
 FILE *f=open_record(root,"workspace-times","wb");record(f,"MANTLE-WORKSPACE-TIMES-1");
 struct stat st;if(lstat("/tmp",&st))die("smoke tmp metadata");record(f,"");integer(f,st.st_mtim.tv_sec);integer(f,st.st_mtim.tv_nsec);
 unsigned long count=1;mantle_ws_capture_node("/","tmp",f,&count);if(fclose(f))die("smoke times close");
}
static void smoke_restore_times(const char *reply){
 FILE *f=open_record(reply,"workspace-times","rb");char *magic=read_record(f);smoke_expect(!strcmp(magic,"MANTLE-WORKSPACE-TIMES-1"),"reply times version");free(magic);
 while(1){int c=fgetc(f);if(c==EOF)break;ungetc(c,f);char *relative=read_record(f);long long seconds=read_integer(f),nanos=read_integer(f);
  if(*relative){smoke_expect(!strcmp(relative,"tmp")||!strncmp(relative,"tmp/",4),"reply time escaped tmp");smoke_expect(mantle_ws_relative(relative)&&nanos>=0&&nanos<1000000000,"reply time path/value");char *path=join("/",relative);struct timespec times[2]={{.tv_nsec=UTIME_OMIT},{.tv_sec=seconds,.tv_nsec=nanos}};if(utimensat(AT_FDCWD,path,times,AT_SYMLINK_NOFOLLOW))die("smoke restore times");free(path);}free(relative);
 }if(ferror(f)||fclose(f))die("smoke reply times read");
}
static void smoke_request(const char *root,unsigned long n){
 remove_tree(root);mkdir_checked(root);text_file(root,"state","active\n");
 char *cwd=getcwd(NULL,0);if(!cwd)die("smoke request cwd");text_file(root,"cwd",cwd);text_file(root,"zero","/inputs/images/script");
 FILE *streams[3];char input[64],turn[32];int size=snprintf(input,sizeof input,"input-%lu\n",n);snprintf(turn,sizeof turn,"%lu",n);
 for(int i=0;i<3;i++){streams[i]=tmpfile();if(!streams[i])die("smoke descriptor stream");}
 smoke_expect(fwrite(input,1,size,streams[0])==(size_t)size&&!fseek(streams[0],0,SEEK_SET),"request stdin");
 FILE *f=open_record(root,"request","wb");record(f,"MANTLE-SHELL-2");integer(f,n);record(f,cwd);free(cwd);
 record(f,"/inputs/images/script");
 if(getenv("SMOKE_NATIVE_BOUND")){record(f,"/bin/sh");integer(f,3);record(f,"/bin/sh");record(f,"-c");record(f,"/inputs/images/llvm-config 0; /inputs/images/llvm-config 1");}
 else{record(f,"/inputs/images/llvm-config");integer(f,2);record(f,"/inputs/images/llvm-config");record(f,turn);}integer(f,0);
 integer(f,2);record(f,"SMOKE_VALUE");integer(f,1);record(f,"value beta");integer(f,1);integer(f,0);
 record(f,"MANTLE_CONTROLLER_ROOTS");integer(f,1);record(f,"/inputs/command-roots");integer(f,1);integer(f,0);
 for(int i=0;i<6;i++)integer(f,0);mode_t mask=umask(0);umask(mask);integer(f,mask);for(int i=0;i<32;i++)record(f,"");
 for(int i=0;i<64;i++){
  if(i>=3){integer(f,-1);continue;}integer(f,i);char name[32];snprintf(name,sizeof name,"descriptor-%d",i);char *path=join(root,name);long position;stream_file(streams[i],path,&position);free(path);
  integer(f,position);integer(f,mantle_descriptor_flags(fileno(streams[i])));integer(f,0);record(f,"");integer(f,0);
 }if(fclose(f))die("smoke request close");for(int i=0;i<3;i++)fclose(streams[i]);smoke_capture_times(root);
}
static int smoke_script(void){
 smoke_expect(!getenv("MANTLE_RUST_SCRIPT_IMAGE")&&!getenv("MANTLE_RUST_SCRIPT_SLOTS")&&!getenv("MANTLE_RUST_NATIVE_COMMANDS_MAX"),"driver metadata leaked to script");
 const char *root=getenv("MANTLE_CC_REQUEST"),*history=getenv("MANTLE_CC_REPLIES"),*out=getenv("OUT_DIR");smoke_expect(root&&history&&out,"script environment");
 smoke_expect(!strcmp(getenv("MANTLE_CC_PHASE_STATE"),out),"mutable phase state");
 if(getenv("SMOKE_SCRIPT_FAILURE"))return 1;
 char *random=join(out,".mantle-cc-random");mkdir_checked(random);text_file(random,".process-index","0\n");
 unsigned long count=strtoul(getenv("SMOKE_COUNT"),NULL,10);smoke_expect(count<=2,"fixture command bound");
 for(unsigned long n=0;n<count;n++){
  char number[32];snprintf(number,sizeof number,"%lu\n",n+1);text_file(random,".process-index",number);smoke_request(root,n);
  snprintf(number,sizeof number,"%lu",n);char *reply=join(history,number),*saved=join(reply,"request"),*expected=join(saved,"request"),*actual=join(root,"request");
  struct stat st;if(stat(expected,&st)){smoke_expect(errno==ENOENT,"reply lookup");printf("discarded script attempt %lu\n",n);free(reply);free(saved);free(expected);free(actual);free(random);return 75;}
  smoke_expect(files_equal(actual,expected),"reply did not acknowledge actual request");free(actual);free(expected);
  for(int i=0;i<3;i++){char name[32];snprintf(name,sizeof name,"descriptor-%d",i);actual=join(root,name);expected=join(saved,name);smoke_expect(files_equal(actual,expected),"reply did not preserve original descriptor");free(actual);free(expected);}free(saved);
  char *status=join(reply,"status");smoke_check(status,n==0?"23\n":"0\n",n==0?3:2);free(status);
  char bytes[128];int size=snprintf(bytes,sizeof bytes,"native-%lu",n);bytes[size++]=0;bytes[size++]=(char)255;bytes[size++]='\n';char *stream=join(reply,"descriptor-1");smoke_check(stream,bytes,size);free(stream);
  size=snprintf(bytes,sizeof bytes,"native diagnostic %lu\n",n);stream=join(reply,"descriptor-2");smoke_check(stream,bytes,size);free(stream);
  char *workspace=join(reply,"workspace");synchronize_tree(workspace,"/tmp");free(workspace);smoke_restore_times(reply);
  snprintf(number,sizeof number,"captured-%lu",n);char *captured=join("/tmp",number);mkdir_checked(captured);saved=join(reply,"request");char *request_copy=join(captured,"request");copy_tree(saved,request_copy);free(saved);free(request_copy);status=join(reply,"status");char *status_copy=join(captured,"status");binary_copy(status,status_copy);free(status);free(status_copy);free(captured);free(reply);
  char side[128];snprintf(side,sizeof side,"/tmp/native-side-%lu",n);size=snprintf(bytes,sizeof bytes,"input-%lu\n",n);smoke_check(side,bytes,size);
  if(getenv("SMOKE_HISTORY_TAMPER")){smoke_bytes("/outputs/workspace/.mantle-rust-controller/history/0/status","0\n",2);free(random);return 0;}
 }
 free(random);smoke_bytes("/tmp/final.bin","script-complete\0\xfe\n",18);smoke_expect(fwrite("script-complete\0\xfe\n",1,18,stdout)==18,"final stdout");fputs("script diagnostic\n",stderr);return 0;
}
int main(int argc,char **argv){
 if(argc>1&&!strcmp(argv[1],"--smoke-bootstrap")){smoke_bootstrap(argc,argv);return 0;}
 if(!strcmp(argv[0],"/inputs/images/script"))return smoke_script();
 return smoke_controller_main(argc,argv);
}
"#;

fn compile(cc: &Path, source: &Path, binary: &Path) {
    let output = std::process::Command::new(cc)
        .args(["-std=c99", "-O0", "-static"])
        .arg(source)
        .arg("-o")
        .arg(binary)
        .output()
        .unwrap();
    assert!(output.status.success(), "fixture compilation failed: {}", String::from_utf8_lossy(&output.stderr));
}

fn fixture_binary(root: &Path, cc: &Path) -> PathBuf {
    let native_source = root.join("native.c");
    let native = root.join("native");
    fs::write(&native_source, NATIVE_SOURCE).unwrap();
    compile(cc, &native_source, &native);
    let image = fs::read(&native).unwrap();
    let image = image.iter().map(u8::to_string).collect::<Vec<_>>().join(",");
    let controller = crate::source_built_native_programs::host::controller_source(&[]).unwrap();
    let source = root.join("native-script-fixture.c");
    fs::write(&source, format!(
        "#define main smoke_controller_main\n{controller}\n#undef main\nstatic const unsigned char smoke_native_image[]={{ {image} }};\n{FIXTURE_SOURCE}"
    )).unwrap();
    let binary = root.join("native-script-fixture");
    compile(cc, &source, &binary);
    binary
}

fn reference(step: &str, output: &str) -> ArtifactRef {
    ArtifactRef::Produced {
        action_id: format!("stagex-transition/{step}"),
        output_id: output.into(),
    }
}

fn source_script_recipe(stage: &str, captured: bool) -> SourceRecipe {
    let proof = crate::source_built_fixed_point_action_trust::tests::fixture().0;
    let request = crate::source_built_program_authoring::AuthoringRequest {
        proof: &proof,
        source_roots: &BTreeMap::new(),
        predecessor_exports: &BTreeMap::new(),
        scratch_root: Path::new("/unused"),
        targets: &[],
    };
    let mut recipe = SourceRecipe {
        schema: crate::source_built_recipe::SCHEMA.into(),
        stage_id: stage.into(),
        preparation: RecipePreparation {
            tool: "native-script-stage-fixture".into(),
            provenance: "structural source graph; no compiler or provider execution".into(),
        },
        steps: Vec::new(),
        timeout_groups: Vec::new(),
        roots: Vec::new(),
        exports: BTreeMap::new(),
        semantic_roots: BTreeMap::new(),
    };
    let result = crate::source_built_rust_programs::llvm::append_native_source_script(
        &request,
        &mut recipe,
        reference("prerequisites", "controller"),
        crate::source_built_rust_programs::llvm::NativeSourceScript {
            id: "source-script".into(),
            script: "exec cc source.c\n".into(),
            initial_workspace: reference("prerequisites", "workspace"),
            initial_cwd: if captured { String::new() } else { "/tmp".into() },
            initial_cwd_input: captured.then(|| (reference("prerequisites", "entry"), "cwd".into())),
            original_zero: "/tmp/source-script".into(),
            environment: BTreeMap::new(),
            environment_input: captured.then(|| (reference("prerequisites", "entry"), "environment".into())),
            inputs: BTreeMap::from([("source".into(), reference("prerequisites", "source"))]),
            readonly_inputs: BTreeSet::from(["source".into()]),
            runtime_mounts: BTreeMap::new(),
            inventory: vec![("cc".into(), reference("prerequisites", "cc"))],
            commands_max: 2,
        },
    )
    .unwrap();
    recipe.roots.extend([result.checkpoint, result.workspace]);
    recipe
}

#[test]
fn native_script_graphs_admit_owning_stage_products_and_reject_cross_stage_inputs() {
    for stage in ["full-source-rust-provider", "mantle-stage1", "mantle-stage2"] {
        for captured in [false, true] {
            let mut recipe = source_script_recipe(stage, captured);
            crate::source_built_recipe::validate_source_recipe(&recipe).unwrap();
            let scripts = recipe.steps.iter_mut().find(|step| step.id == "source-script.scripts").unwrap();
            scripts.inputs.insert("member-0".into(), ArtifactRef::Produced {
                action_id: "stagex-transition/source-script.program".into(),
                output_id: "text".into(),
            });
            assert!(crate::source_built_recipe::validate_source_recipe(&recipe).is_err());
        }
    }
}

#[derive(Clone, Copy)]
enum CaseMode {
    Normal,
    RequestExhaustion,
    WrongImage,
    NativeExhaustion,
    HistoryCorruption,
    ScriptFailure,
    InvalidCwd,
}
#[derive(Clone, Copy)]
struct Case {
    count: u32,
    slots: usize,
    mode: CaseMode,
}

fn fixture_recipe(Case { count, slots, mode }: Case) -> (SourceRecipe, ControllerResult) {
    let disk = 256 * 1024 * 1024;
    let mut recipe = SourceRecipe {
        schema: crate::source_built_recipe::SCHEMA.into(),
        stage_id: "stagex-transition".into(),
        preparation: RecipePreparation {
            tool: "compact-native-protected-smoke".into(),
            provenance: "parent-compiled bounded C protocol fixtures; not original compiler/provider proof".into(),
        },
        steps: Vec::new(),
        timeout_groups: Vec::new(),
        roots: Vec::new(),
        exports: BTreeMap::new(),
        semantic_roots: BTreeMap::new(),
    };
    recipe.steps.push(RecipeStep {
        id: "fixture".into(),
        inputs: BTreeMap::from([("seed".into(), ArtifactRef::Source {
            role: SourceAuthorityRole::StagexSeed,
            path: String::new(),
        })]),
        outputs: [
            ("controller", ArtifactKind::Executable),
            ("script", ArtifactKind::Executable),
            ("native", ArtifactKind::Executable),
            ("images", ArtifactKind::Tree),
            ("entry", ArtifactKind::Tree),
            ("roots", ArtifactKind::Text),
            ("stdout", ArtifactKind::Text),
            ("stderr", ArtifactKind::Text),
        ]
        .into_iter()
        .map(|(name, kind)| (name.into(), RecipeOutput { kind, bytes_max: disk }))
        .collect(),
        operation: RecipeOperation::Exec {
            executable_input: "seed".into(),
            invocation_input: None,
            arguments: vec![
                lit("--smoke-bootstrap"),
                output("controller", ""),
                output("script", ""),
                output("native", ""),
                output("images", ""),
                output("entry", ""),
                output("roots", ""),
                lit("bounded-c-protocol-only"),
                lit(count.to_string()),
                lit(slots.to_string()),
                lit(match mode {
                    CaseMode::Normal | CaseMode::RequestExhaustion => "exact",
                    CaseMode::WrongImage => "tamper",
                    CaseMode::NativeExhaustion => "native-bound",
                    CaseMode::HistoryCorruption => "history",
                    CaseMode::ScriptFailure => "script-failure",
                    CaseMode::InvalidCwd => "invalid-cwd",
                }),
            ],
            environment: BTreeMap::new(),
            runtime_mounts: BTreeMap::new(),
            cwd_input: None,
            cwd_output: None,
            output_seeds: BTreeMap::new(),
            command_state: None,
            guard: None,
            stdin_input: None,
            merge_stderr: false,
            exit_policy: RecipeExitPolicy::Exact { code: 0 },
        },
    });
    let command_inputs = BTreeMap::from([
        ("images".into(), reference("fixture", "images")),
        ("command-roots".into(), reference("fixture", "roots")),
    ]);
    let result = append_controller(&mut recipe, ControllerSpec {
        id: "smoke.native-controller".into(),
        controller: reference("fixture", "controller"),
        inputs: BTreeMap::from([
            ("entry".into(), reference("fixture", "entry")),
            ("controller-roots".into(), reference("fixture", "roots")),
        ]),
        controller_runtime_mounts: BTreeMap::new(),
        start_arguments: vec![
            lit("--mantle-source-shell"),
            lit("run"),
            input("entry", ""),
            lit("source.sh"),
            input("entry", "environment"),
            lit("-"),
            lit("-"),
            input("entry", "initial"),
            output("checkpoint", ""),
        ],
        environment: BTreeMap::from([
            ("MANTLE_CONTROLLER_ROOTS".into(), input("controller-roots", "")),
            ("MANTLE_CONTROLLER_INITIAL_CWD".into(), input("entry", "cwd")),
            ("MANTLE_SHELL_ORIGINAL_ZERO".into(), input("entry", "zero")),
        ]),
        restart_indices: [5, 6, 8],
        slots: [("script", "script", "script"), ("native", "llvm-config", "native")]
            .into_iter()
            .map(|(slot, name, image)| ControllerSlot {
                id: slot.into(),
                executable: ControllerExecutable::Inventory(vec![(name.into(), reference("fixture", image))]),
                inputs: command_inputs.clone(),
                runtime_mounts: BTreeMap::new(),
                exit_policy: RecipeExitPolicy::Normal,
                merge_stderr: false,
                generated_outputs: true,
            })
            .collect(),
        commands_max: 8,
        exit_policy: RecipeExitPolicy::Exact { code: 0 },
        file_bytes_max: 64 * 1024 * 1024,
        tree_bytes_max: disk,
    })
    .unwrap();
    recipe.roots.extend([result.checkpoint.clone(), result.workspace.clone()]);
    (recipe, result)
}

fn sources(root: &Path, seed: &Path, recipe: &SourceRecipe) -> BTreeMap<SourceAuthorityRole, PathBuf> {
    let lineage = root.join("lineage.json");
    fs::write(&lineage, serde_json::to_vec_pretty(recipe).unwrap()).unwrap();
    let mut roots = BTreeMap::from([
        (SourceAuthorityRole::StagexSeed, seed.to_path_buf()),
        (SourceAuthorityRole::StagexLineage, lineage),
    ]);
    for role in [
        SourceAuthorityRole::StagexSourceBundle,
        SourceAuthorityRole::NativeSourceBundle,
    ] {
        let path = root.join(format!("{role:?}.source"));
        fs::write(&path, b"bounded smoke-only inert source\n").unwrap();
        roots.insert(role, path);
    }
    for role in [
        SourceAuthorityRole::RustSourceArchiveSet,
        SourceAuthorityRole::MantleSource,
        SourceAuthorityRole::VendorInputs,
    ] {
        let path = root.join(format!("{role:?}.sources"));
        fs::create_dir(&path).unwrap();
        fs::write(path.join("README"), b"bounded smoke-only inert source\n").unwrap();
        roots.insert(role, path);
    }
    roots
}

fn proof(root: &Path, roots: &BTreeMap<SourceAuthorityRole, PathBuf>) -> SourceBuiltFixedPointPlan {
    let source_inputs = roots
        .iter()
        .map(|(role, path)| {
            let (size_bytes, digest_blake3) = digest(path).unwrap();
            SourceAuthorityInput {
                id: format!("native-smoke-{role:?}"),
                role: *role,
                kind: if path.is_dir() {
                    SourceContentKind::Directory
                } else {
                    SourceContentKind::RegularFile
                },
                digest_blake3,
                size_bytes,
            }
        })
        .collect();
    let policy = |text: &str| blake3::hash(text.as_bytes()).to_hex().to_string();
    let proof = plan_source_built_fixed_point(SourceBuiltFixedPointPlanInput {
        proof_id: format!("native-smoke-{}", root.file_name().unwrap().to_str().unwrap()),
        logical_store_prefix: "/mantle/store".into(),
        source_inputs,
        initial_output_authority: InitialOutputAuthorityState {
            stagex_transition_entries: 0,
            native_provider_entries: 0,
            rust_provider_entries: 0,
            mantle_output_entries: 0,
        },
        policies: SourceBuiltFixedPointPolicies {
            expected_native_provider_digest_blake3: policy("no provider from bounded C smoke"),
            closure_policy_digest_blake3: policy("static declared fixture only"),
            hermeticity_policy_digest_blake3: policy("strict namespace native smoke"),
            protected_execution_policy_digest_blake3: policy("separate script/native image inventory"),
            effect_policy_digest_blake3: policy("actual shared requests, normal statuses, stdout and tmp"),
            normalization_policy_digest_blake3: policy("exact original output bytes"),
            hermeticity_mode: ProofHermeticityMode::Strict,
            live_fetch_allowed: false,
            cargo_invocation_allowed: false,
            ambient_discovery_allowed: false,
            fallback_allowed: false,
            provider_cache_completion_allowed: false,
        },
        resource_bounds: SourceBuiltFixedPointResourceBounds {
            elapsed_seconds_max: 120,
            disk_bytes_max: 256 * 1024 * 1024,
            open_file_descriptors_max: 1024,
            protected_exec_events_max: 4096,
            source_records_max: 64,
        },
    })
    .unwrap();
    fs::write(root.join("proof.json"), serde_json::to_vec_pretty(&proof).unwrap()).unwrap();
    proof
}

fn exercise(root: &Path, seed: &Path, sandbox: &Path, case: Case) {
    let Case { count, slots, mode } = case;
    let accepted = matches!(mode, CaseMode::Normal);
    fs::create_dir(root).unwrap();
    let (recipe, products) = fixture_recipe(case);
    let roots = sources(root, seed, &recipe);
    let proof = proof(root, &roots);
    let host = digest(&std::env::current_exe().unwrap()).unwrap().1;
    let stage = prepare(
        &proof,
        "stagex-transition",
        SourceAuthorityRole::StagexLineage,
        "",
        &RecipePaths {
            source_roots: &roots,
            runtime_root: &root.join("runtime"),
            sandbox_binary: sandbox,
        },
        &[],
        &host,
    )
    .unwrap();
    let mut state = ActionObservationState::default();
    let mut events = Vec::new();
    let adapters = [stage.adapter().clone()];
    let result = run(&stage, &mut |event| {
        events.push(event);
        state
            .record_adapters(&proof, &adapters, events.last().expect("actual retained event"))
            .map_err(error)
    });
    fs::write(root.join("events.json"), serde_json::to_vec_pretty(&events).unwrap()).unwrap();
    let executed = |slot: &str, code: i32| {
        events
            .iter()
            .filter(|event| {
                event.kind == ExecutionEventKind::BuildExecution
                    && event.normal_exit_code == Some(code)
                    && crate::source_built_fixed_point_action_trust::controller_action_position(
                        &stage.adapter().controllers[0],
                        &event.action_id,
                    )
                    .unwrap()
                    .is_some_and(|position| {
                        !position.sandbox
                            && position.slot.as_deref() == Some(slot)
                            && position.role
                                == crate::source_built_fixed_point_action_trust::ActionControllerRole::Command
                    })
            })
            .count()
    };
    let script_digest = digest(seed).unwrap().1;
    let native_digest = digest(&seed.parent().unwrap().join("native")).unwrap().1;
    assert_ne!(script_digest, native_digest, "script and native fixtures must be separate images");
    for event in &events {
        if event.kind != ExecutionEventKind::BuildExecution {
            continue;
        }
        let Some(position) = crate::source_built_fixed_point_action_trust::controller_action_position(
            &stage.adapter().controllers[0],
            &event.action_id,
        )
        .unwrap() else {
            continue;
        };
        if position.sandbox
            || position.role != crate::source_built_fixed_point_action_trust::ActionControllerRole::Command
        {
            continue;
        }
        let expected = match position.slot.as_deref() {
            Some("script") => &script_digest,
            Some("native") => &native_digest,
            _ => panic!("unexpected protected fixture command slot"),
        };
        assert_eq!(&event.executable_digest_blake3, expected);
        assert!(events.iter().any(|launch| launch.kind == ExecutionEventKind::ProtectedExec
            && launch.action_id == event.action_id
            && !launch.protected_exec_violation
            && &launch.executable_digest_blake3 == expected));
    }
    if accepted {
        let run =
            result.unwrap_or_else(|failure| panic!("protected native smoke failed at {}: {failure}", root.display()));
        let checkpoint = &run.outputs[&products.checkpoint].0;
        let workspace = &run.outputs[&products.workspace].0;
        assert_eq!(fs::read(checkpoint.join("stdout")).unwrap(), b"script-complete\0\xfe\n");
        assert_eq!(fs::read(checkpoint.join("stderr")).unwrap(), b"script diagnostic\n");
        assert_eq!(fs::read(workspace.join("tmp/final.bin")).unwrap(), b"script-complete\0\xfe\n");
        assert_eq!(fs::read(checkpoint.join("state")).unwrap(), b"done\n");
        assert_eq!(fs::read(checkpoint.join("exit-status")).unwrap(), b"0\n");
        assert!(!workspace.join(".mantle-rust-controller").exists());
        assert_eq!(executed("script", 0), 1);
        assert_eq!(executed("script", 75), count as usize);
        assert_eq!(executed("native", 23), usize::from(count > 0));
        assert_eq!(executed("native", 0), count.saturating_sub(1) as usize);
        assert_eq!(
            state.controller_outcome("stagex-transition/smoke.native-controller").unwrap().completed_turns,
            count * 2 + 1
        );
        for n in 0..count {
            assert_eq!(
                fs::read(workspace.join(format!("tmp/native-side-{n}"))).unwrap(),
                format!("input-{n}\n").as_bytes()
            );
            let reply = workspace.join(format!("tmp/captured-{n}"));
            assert_eq!(fs::read(reply.join("status")).unwrap(), if n == 0 { &b"23\n"[..] } else { &b"0\n"[..] });
            let request = fs::read(reply.join("request/request")).unwrap();
            assert!(request.starts_with(b"14:MANTLE-SHELL-2,"));
            assert_eq!(fs::read(reply.join("request/descriptor-0")).unwrap(), format!("input-{n}\n").as_bytes());
            fs::create_dir(root.join(format!("captured-{n}"))).unwrap();
            fs::copy(reply.join("request/request"), root.join(format!("captured-{n}/request"))).unwrap();
            fs::copy(reply.join("status"), root.join(format!("captured-{n}/status"))).unwrap();
        }
    } else {
        let failure = result.err().expect("invalid native continuation admitted");
        fs::write(root.join("failure.txt"), failure.to_string()).unwrap();
        assert!(state.controller_outcome("stagex-transition/smoke.native-controller").is_err());
        let expected = match mode {
            CaseMode::RequestExhaustion => (0, slots + 1, 1, 0),
            CaseMode::WrongImage => (0, 1, 0, 0),
            CaseMode::NativeExhaustion => (0, 1, 1, 0),
            CaseMode::HistoryCorruption => (1, 1, 1, 0),
            CaseMode::ScriptFailure => (0, 0, 0, 1),
            CaseMode::InvalidCwd => (0, 0, 0, 0),
            CaseMode::Normal => unreachable!(),
        };
        assert_eq!(
            (executed("script", 0), executed("script", 75), executed("native", 23), executed("script", 1)),
            expected
        );
        if matches!(mode, CaseMode::InvalidCwd) {
            assert_eq!(
                fs::read(root.join("runtime/smoke.native-controller/start/outputs/exit-status")).unwrap(),
                b"2\n"
            );
        }
        assert_eq!(executed("native", 0), 0);
    }
    println!(
        "protected native case {}: accepted={accepted}, script75={}, native23={}",
        root.display(),
        executed("script", 75),
        executed("native", 23)
    );
}

#[test]
#[ignore = "requires explicit static C compiler, reviewed bwrap and fresh evidence root"]
fn compact_native_protected_smoke() {
    let root = PathBuf::from(std::env::var_os("MANTLE_NATIVE_SMOKE_ROOT").expect("fresh absolute smoke evidence root"));
    let cc = PathBuf::from(std::env::var_os("MANTLE_NATIVE_SMOKE_CC").expect("explicit pinned static C compiler"));
    let sandbox = PathBuf::from(std::env::var_os("MANTLE_PROTECTED_BWRAP").expect("explicit reviewed bwrap"));
    assert!(root.is_absolute() && cc.is_absolute() && sandbox.is_absolute());
    fs::create_dir(&root).expect("smoke root must be fresh");
    let seed = fixture_binary(&root, &cc);
    for (name, case) in [
        ("early", Case {
            count: 0,
            slots: 0,
            mode: CaseMode::Normal,
        }),
        ("nonzero-replay", Case {
            count: 2,
            slots: 2,
            mode: CaseMode::Normal,
        }),
        ("exhausted", Case {
            count: 2,
            slots: 1,
            mode: CaseMode::RequestExhaustion,
        }),
        ("wrong-image", Case {
            count: 1,
            slots: 1,
            mode: CaseMode::WrongImage,
        }),
        ("native-bound", Case {
            count: 1,
            slots: 1,
            mode: CaseMode::NativeExhaustion,
        }),
        ("history-corruption", Case {
            count: 1,
            slots: 1,
            mode: CaseMode::HistoryCorruption,
        }),
        ("script-failure", Case {
            count: 0,
            slots: 0,
            mode: CaseMode::ScriptFailure,
        }),
        ("invalid-cwd", Case {
            count: 0,
            slots: 0,
            mode: CaseMode::InvalidCwd,
        }),
    ] {
        exercise(&root.join(name), &seed, &sandbox, case);
    }
}
