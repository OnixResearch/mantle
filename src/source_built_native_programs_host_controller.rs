//! One source controller convention for native original shell nodes.
use crate::errors::RunError;

pub(crate) struct ControllerHooks<'a> {
    pub source: &'a str,
    pub external: &'a str,
    pub prepare: &'a str,
    pub environment: &'a str,
}
fn identifier(name: &str) -> bool {
    let mut bytes = name.bytes();
    matches!(bytes.next(), Some(b'a'..=b'z' | b'A'..=b'Z' | b'_'))
        && bytes.all(|b| b.is_ascii_alphanumeric() || b == b'_')
}
/// Source extensions share Env/Args/Request and the canonical operand builder;
/// they do not select a second scheduler or invent another request wire format.
pub(crate) fn controller_library_source(extensions: &[ControllerHooks<'_>]) -> Result<String, RunError> {
    let mut source = crate::source_built_stagex_programs::shellvm::library_source()?;
    source.push_str(&crate::source_built_native_programs::controller_gcc_source()?);
    source.push_str(include_str!("source_built_native_programs_host_nested.c"));
    source.push_str(include_str!("source_built_native_programs_host_timeout.c"));
    source.push_str(include_str!("source_built_rust_programs_llvm_controller.c"));
    source.push_str(&crate::source_built_rust_programs::llvm::controller_hooks_source());
    source.push_str(&crate::source_built_native_programs::final_gnu::perl_controller_source());
    for extension in extensions {
        if ![extension.external, extension.prepare, extension.environment].into_iter().all(identifier) {
            return Err(super::fail("invalid native controller source callback identifier"));
        }
        source.push_str(extension.source);
    }
    source.push_str("\nstatic int host_source_external(Env *e,Args *a){int status;\n");
    for extension in extensions {
        source.push_str(&format!("status={}(e,a);if(status>=0)return status;\n", extension.external));
    }
    source.push_str("status=mantle_perl_controller_external(e,a);if(status>=0)return status;\nif(a->n&&!strcmp(mantle_native_basename(a->v[0]),\"cmake\"))return llvm_cmake(e,a);\nreturn mantle_native_external(e,a);}\n");
    source.push_str("static int host_source_prepare(const char *slot,Request *r,mantle_controller_builder *b){int status;\nif(r->command.n>1&&!strcmp(r->command.v[1],\"--mantle-source-control\")&&(!strcmp(mantle_native_basename(r->command.v[0]),\"make\")||!strcmp(mantle_native_basename(r->command.v[0]),\"gmake\"))){b->private_descriptors=1;return mantle_controller_standard(b);}\n");
    for extension in extensions {
        source.push_str(&format!("status={}(slot,r,b);if(status)return status;\n", extension.prepare));
    }
    source.push_str("status=mantle_perl_controller_prepare(slot,r,b);if(status)return status;\nreturn llvm_native_prepare(slot,r,b);}\n");
    source.push_str("static char *host_source_environment(mantle_controller_builder *b,const char *key,const char *value){char *word;\nif(!strcmp(key,\"MANTLE_CONTROLLER_ROOTS\"))return mantle_controller_named_word(b,0,\"command-roots\",\"\");\n");
    for extension in extensions {
        source.push_str(&format!("word={}(b,key,value);if(word)return word;\n", extension.environment));
    }
    source.push_str("word=mantle_perl_controller_environment(b,key,value);if(word)return word;\nreturn llvm_native_environment(b,key,value);}\nstatic void mantle_native_register_hooks(void){mantle_source_external_hook=host_source_external;mantle_source_entry_hook=llvm_native_source_entry;mantle_controller_prepare_hook=host_source_prepare;mantle_controller_environment_hook=host_source_environment;mantle_controller_process_attributes_hook=mantle_native_timeout_attributes;mantle_controller_source_scope_resume_hook=mantle_native_timeout_resume;}\n");
    Ok(source)
}
pub(crate) fn controller_source(extensions: &[ControllerHooks<'_>]) -> Result<String, RunError> {
    let mut source = controller_library_source(extensions)?;
    source.push_str("\nstatic int host_source_run(int argc,char **argv){if(argc>1&&!strcmp(argv[1],\"--mantle-native-installed\"))return mantle_native_installed(argc,argv);if(argc>1&&!strcmp(argv[1],\"--mantle-source-shell\"))return mantle_shell_run(argc-1,argv+1);die(\"native source controller expects explicit original shell entry\");return 2;}\nint main(int argc,char **argv){mantle_native_register_hooks();int status=mantle_controller_dispatch(argc,argv,host_source_run);return status>=0?status:host_source_run(argc,argv);}\n");
    Ok(source)
}
