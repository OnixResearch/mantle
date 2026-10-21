#define _POSIX_C_SOURCE 200809L
/* Processless materialization of the selected original musl-host Rust repairs.
 * Input trees are immutable; the entire archive tree is copied before edits.
 * No compiler, package manager, shell or build-system process is launched. */
#include <dirent.h>
#include <errno.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <unistd.h>

static void fail(const char *what) { fprintf(stderr,"rust source normalization: %s (%s)\n",what,strerror(errno)); exit(2); }
static void *allocate(size_t n) { void *p=malloc(n?n:1); if(!p) fail("allocation"); return p; }
static char *join(const char *a,const char *b) { size_t n=strlen(a)+strlen(b)+2; char *p=allocate(n); snprintf(p,n,"%s/%s",a,b); return p; }
static char *read_text(const char *p) {
    FILE *f=fopen(p,"rb"); if(!f) fail(p);
    if(fseek(f,0,SEEK_END)) fail(p); long n=ftell(f);
    if(n<0 || n>64*1024*1024) fail("source text exceeds bound");
    rewind(f); char *s=allocate((size_t)n+1);
    if(fread(s,1,(size_t)n,f)!=(size_t)n || fclose(f)) fail(p);
    if(memchr(s,0,(size_t)n)) fail("NUL in source text"); s[n]=0; return s;
}
static void write_text(const char *p,const char *s) {
    int fd=open(p,O_WRONLY|O_CREAT|O_TRUNC|O_NOFOLLOW,0644); if(fd<0) fail(p);
    size_t n=strlen(s); while(n) { ssize_t k=write(fd,s,n); if(k<=0) fail(p); s+=k; n-=(size_t)k; }
    if(close(fd)) fail(p);
}
static void copy_tree(const char *src,const char *dst) {
    struct stat st; if(lstat(src,&st)) fail(src);
    if(S_ISDIR(st.st_mode)) {
        if(mkdir(dst,0755) && errno!=EEXIST) fail(dst);
        DIR *d=opendir(src); if(!d) fail(src); struct dirent *e;
        while((e=readdir(d))) { if(!strcmp(e->d_name,".")||!strcmp(e->d_name,"..")) continue;
            char *a=join(src,e->d_name),*b=join(dst,e->d_name); copy_tree(a,b); free(a); free(b); }
        if(closedir(d)) fail(src);
    } else if(S_ISREG(st.st_mode)) {
        int in=open(src,O_RDONLY|O_NOFOLLOW),out=open(dst,O_WRONLY|O_CREAT|O_EXCL,st.st_mode&0777);
        if(in<0||out<0) fail(src); char buf[65536]; ssize_t n;
        while((n=read(in,buf,sizeof buf))>0) { ssize_t pos=0; while(pos<n) { ssize_t k=write(out,buf+pos,(size_t)(n-pos)); if(k<=0) fail(dst); pos+=k; } }
        if(n<0||close(in)||close(out)) fail(src);
    } else if(S_ISLNK(st.st_mode)) {
        char target[4096]; ssize_t n=readlink(src,target,sizeof target-1); if(n<0||n==(ssize_t)sizeof(target)-1) fail(src);
        target[n]=0; if(symlink(target,dst)) fail(dst);
    } else fail("unsupported source tree entry");
}
/* Exact line/block anchors, not fuzzy patch matching or already-patched fallbacks. */
static char *replace(char *s,const char *old,const char *new,const char *label) {
    char *at=strstr(s,old); if(!at || strstr(at+strlen(old),old)) fail(label);
    size_t prefix=(size_t)(at-s),a=strlen(old),b=strlen(new),suffix=strlen(at+a);
    char *r=allocate(prefix+b+suffix+1); memcpy(r,s,prefix); memcpy(r+prefix,new,b); memcpy(r+prefix+b,at+a,suffix+1); free(s); return r;
}
static void edit(const char *root,const char *file,const char *old,const char *new) {
    char *p=join(root,file),*s=read_text(p); s=replace(s,old,new,p); write_text(p,s); free(s); free(p);
}
static void append_workspace(const char *root,const char *file,int optional) {
    char *p=join(root,file); struct stat st;
    if(lstat(p,&st)) { if(optional && errno==ENOENT) { free(p); return; } fail(p); }
    char *s=read_text(p); if(strncmp(s,"[workspace]\n",12)&&!strstr(s,"\n[workspace]\n")) {
        size_t n=strlen(s); char *r=allocate(n+15); memcpy(r,s,n); strcpy(r+n,"\n[workspace]\n"); write_text(p,r); free(r);
    } free(s); free(p);
}
static char *normalize_patch(char *s) {
    size_t n=strlen(s); char *r=allocate(n+1),*w=r; int skip=0,errors=0,sty=0;
    for(char *p=s;*p;) { char *end=strchr(p,'\n'); size_t len=end?(size_t)(end-p)+1:strlen(p);
        if(!strncmp(p,"--- ",4)) {
            skip=0;
            if(len==strlen("--- compiler/rustc_errors/src/lib.rs\n")&&!memcmp(p,"--- compiler/rustc_errors/src/lib.rs\n",len)) { skip=1; errors++; }
            if(len==strlen("--- compiler/rustc_middle/src/ty/sty.rs\n")&&!memcmp(p,"--- compiler/rustc_middle/src/ty/sty.rs\n",len)) { skip=1; sty++; }
        }
        if(!skip) { memcpy(w,p,len); w+=len; } p+=len;
    }
    if(errors!=1||sty!=1) fail("exact BusyBox-incompatible patch section set"); *w=0; free(s); return r;
}
static char *take_line(char **cursor) {
    if(!**cursor) return NULL; char *p=*cursor,*e=strchr(p,'\n');
    if(!e) fail("unterminated source patch line"); *e=0; *cursor=e+1; return p;
}
static void push(char **s,size_t *n,const char *line) {
    size_t len=strlen(line); char *r=realloc(*s,*n+len+2); if(!r) fail("patch allocation");
    *s=r; memcpy(r+*n,line,len); *n+=len; r[(*n)++]='\n'; r[*n]=0;
}
/* Original mrustc unified patch, with exact unique complete hunk context.
 * Original line numbers are authenticated syntax, never a fuzzy search hint. */
static void apply_patch(const char *root,char *patch) {
    char *cursor=patch,*line,*path=NULL; unsigned sections=0,hunks=0;
    while((line=take_line(&cursor))) {
        if(!strncmp(line,"--- ",4)) {
            free(path); path=NULL; const char *rel=line+4;
            if(*rel=='/'||strstr(rel,"..")||strchr(rel,'\t')) fail("unsafe patch member");
            path=join(root,rel); char *next=take_line(&cursor);
            if(!next||strncmp(next,"+++ ",4)||strcmp(next+4,rel)) fail("patch source/destination mismatch"); sections++;
        } else if(!strncmp(line,"@@ ",3)) {
            unsigned ol,oc,nl,nc; int used=0;
            if(!path||sscanf(line,"@@ -%u,%u +%u,%u @@%n",&ol,&oc,&nl,&nc,&used)!=4||!used||!ol||!nl) fail("unsupported unified hunk header");
            char *old=NULL,*new=NULL; size_t a=0,b=0; unsigned x=0,y=0;
            while(x<oc||y<nc) {
                char *body=take_line(&cursor); if(!body) fail("truncated hunk");
                if(*body==' '||*body=='-') { push(&old,&a,body+1); x++; }
                if(*body==' '||*body=='+') { push(&new,&b,body+1); y++; }
                if((*body!=' '&&*body!='-'&&*body!='+')||x>oc||y>nc) fail("hunk count mismatch");
            }
            if(!old||!new) fail("unsupported empty hunk");
            char *s=read_text(path); s=replace(s,old,new,path); write_text(path,s); free(s); free(old); free(new); hunks++;
        } else if(*line && *line!='#') fail("unexpected unified patch syntax");
    }
    if(sections!=3||hunks!=3) fail("Rust 1.90 normalized original patch section count"); free(path);
}
static void driver_rlib(const char *root) {
    edit(root,"compiler/rustc_driver/Cargo.toml","crate-type = [\"dylib\"]\n","crate-type = [\"rlib\"]\n");
}
static void rust190(const char *root,const char *mrustc) {
    char *p=join(mrustc,"mrustc-0.12.0/rustc-1.90.0-src.patch"); char *s=normalize_patch(read_text(p)); free(p);
    apply_patch(root,s); free(s);
    edit(root,"compiler/rustc_errors/src/lib.rs","rustc_data_structures::static_assert_size!(PResult<'_, bool>, 24);\n","#[cfg(not(rust_compiler=\"mrustc\"))]\nrustc_data_structures::static_assert_size!(PResult<'_, bool>, 24);\n");
    edit(root,"compiler/rustc_middle/src/ty/sty.rs","        self.split_last().unwrap().1\n","        (**self).split_last().unwrap().1\n");
    edit(root,"compiler/rustc_session/src/config.rs","        Sysroot { explicit, default: filesearch::default_sysroot() }\n","        match explicit {\n            Some(explicit) => Sysroot { default: explicit.clone(), explicit: Some(explicit) },\n            None => Sysroot { explicit: None, default: filesearch::default_sysroot() },\n        }\n");
    driver_rlib(root);
}
static void successor(const char *root) {
    append_workspace(root,"src/bootstrap/Cargo.toml",0);
    append_workspace(root,"compiler/rustc_codegen_cranelift/Cargo.toml",1);
    append_workspace(root,"compiler/rustc_codegen_gcc/Cargo.toml",1);
    driver_rlib(root);
    edit(root,"compiler/rustc_session/src/filesearch.rs","    from_env_args_next()\n        .unwrap_or_else(|| default_from_rustc_driver_dll().expect(\"Failed finding sysroot\"))\n","    from_env_args_next()\n        .or_else(|| env::var_os(\"RUSTC_SYSROOT\").map(PathBuf::from))\n        .unwrap_or_else(|| default_from_rustc_driver_dll().expect(\"Failed finding sysroot\"))\n");
    char *p=join(root,"src/bootstrap/src/core/build_steps/tool.rs"),*s=read_text(p);
    const char *anchor="        let mut cargo = prepare_tool_cargo(\n"; char *at=strstr(s,anchor);
    if(!at||strstr(at+strlen(anchor),anchor)) fail("ToolBuild cargo construction anchor");
    char *end=strstr(at,"        );\n"); if(!end) fail("ToolBuild cargo construction end"); end+=strlen("        );\n");
    const char *extra="        if self.mode == Mode::ToolRustcPrivate {\n            cargo.append_to_env(\n                \"RUSTC_ADDITIONAL_SYSROOT_PATHS\",\n                builder\n                    .cargo_out(self.build_compiler, Mode::Rustc, target)\n                    .join(\"deps\")\n                    .to_str()\n                    .expect(\"rustc private tool rlib path should be UTF-8\")\n                    .to_owned(),\n                \",\",\n            );\n        }\n";
    size_t prefix=(size_t)(end-s),n=strlen(extra),suffix=strlen(end); char *r=allocate(prefix+n+suffix+1);
    memcpy(r,s,prefix); memcpy(r+prefix,extra,n); memcpy(r+prefix+n,end,suffix+1); write_text(p,r); free(r); free(s); free(p);
}
static void mrustc(const char *root,const char *runtime,const char *metadata,const char *proc_macro) {
    if(!*runtime || runtime[0]!='/' || strpbrk(runtime,"\n\r\"'`\\$") ) fail("unsafe exact runtime directory");
    char *metadata_path=join(root,"src/cairn_metadata.hpp");
    copy_tree(metadata,metadata_path); free(metadata_path);
    edit(root,"src/main.cpp","int main(int argc, char *argv[])\n{\n",
        "#include \"cairn_metadata.hpp\"\n\nint main(int argc, char *argv[])\n{\n    try { cairn_metadata::load(argc, argv); }\n    catch(const std::exception& error) { std::cerr << \"Cairn mrustc metadata: \" << error.what() << std::endl; return 1; }\n");
    edit(root,"src/trans/codegen_c.cpp","#include \"codegen.hpp\"\n","#include \"codegen.hpp\"\nextern \"C\" { extern char **environ; }\n");
    edit(root,"src/trans/codegen_c.cpp","                ::std::ofstream(opt.build_command_file) << cmd_ss.str() << ::std::endl;\n",
        "                ::std::ofstream(opt.build_command_file) << cmd_ss.str() << ::std::endl;\n"
        "                ::std::ofstream cairn_environment(m_outfile_path + \"-environment\", ::std::ios::binary);\n"
        "                ASSERT_BUG(Span(), cairn_environment.is_open(), \"Cannot capture deferred compiler environment\");\n"
        "                for(char **entry = ::environ; *entry; ++entry) {\n"
        "                    const char *equal = strchr(*entry, '=');\n"
        "                    ASSERT_BUG(Span(), equal && equal != *entry, \"Invalid inherited native compiler environment\");\n"
        "                    const size_t key_size = equal - *entry, value_size = strlen(equal + 1);\n"
        "                    cairn_environment << key_size << ':'; cairn_environment.write(*entry, key_size); cairn_environment << ',' << value_size << ':'; cairn_environment.write(equal + 1, value_size); cairn_environment << ',';\n"
        "                }\n"
        "                cairn_environment.close();\n"
        "                ASSERT_BUG(Span(), !cairn_environment.fail(), \"Cannot write deferred compiler environment\");\n");
    char *macro_path=join(root,"src/expand/cairn_proc_macro.hpp");
    copy_tree(proc_macro,macro_path); free(macro_path);
    edit(root,"src/expand/proc_macro.cpp","#include \"proc_macro.hpp\"\n","#include \"proc_macro.hpp\"\n#include \"cairn_proc_macro.hpp\"\n");
    edit(root,"src/expand/proc_macro.cpp","    struct Handles\n    {\n","    std::shared_ptr<CairnProcMacro> m_cairn;\n    struct Handles\n    {\n");
    edit(root,"src/expand/proc_macro.cpp","    bool check_good();\n","    void cairn_finish() { if(m_cairn) m_cairn->finish(); }\n    bool check_good();\n");
    edit(root,"src/expand/proc_macro.cpp","    m_edition(edition)\n{\n",
        "    m_edition(edition)\n{\n    if(getenv(\"MRUSTC_CAIRN_MACRO_HISTORY\")) {\n        m_cairn.reset(new CairnProcMacro(executable, proc_macro_desc.name.c_str()));\n        handles.child_pid = 0; handles.child_stdin = -1; handles.child_stdout = -1;\n        this->send_span_def(1, sp);\n        return;\n    }\n");
    edit(root,"src/expand/proc_macro.cpp","    if( !pmi.check_good() )\n        return ::std::unique_ptr<TokenStream>();\n","");
    edit(root,"src/expand/proc_macro.cpp","    // 3. Return boxed invocation instance\n",
        "    pmi.cairn_finish();\n    if( !pmi.check_good() )\n        return ::std::unique_ptr<TokenStream>();\n    // 3. Return boxed invocation instance\n");
    edit(root,"src/expand/proc_macro.cpp","bool ProcMacroInv::check_good()\n{\n",
        "bool ProcMacroInv::check_good()\n{\n    if(m_cairn) return m_cairn->check_good();\n");
    edit(root,"src/expand/proc_macro.cpp","void ProcMacroInv::send_bytes_raw(const void* val, size_t size)\n{\n",
        "void ProcMacroInv::send_bytes_raw(const void* val, size_t size)\n{\n    if(m_cairn) { m_cairn->send(val, size); return; }\n");
    edit(root,"src/expand/proc_macro.cpp","void ProcMacroInv::recv_bytes_raw(void* out_void, size_t len)\n{\n",
        "void ProcMacroInv::recv_bytes_raw(void* out_void, size_t len)\n{\n    if(m_cairn) { m_cairn->receive(out_void, len); return; }\n");
    char *p=join(root,"Makefile"),*s=read_text(p),*at=strstr(s,"-D VERSION_GIT_FULLHASH=");
    if(!at||strstr(at+1,"-D VERSION_GIT_FULLHASH=")) fail("authenticated version metadata rule");
    char *start=at; while(start>s&&start[-1]!='\n') start--; char *end=strchr(at,'\n');
    if(!end||!strstr(at,"-D VERSION_GIT_ISDIRTY=")) fail("authenticated version metadata end");
    char save=end[1]; end[1]=0; char *old=strdup(start); if(!old) fail("metadata allocation"); end[1]=save;
    s=replace(s,old,"\t$V$(CXX) -o $@ -c $< $(CXXFLAGS) $(CPPFLAGS) -MMD -MP -MF $@.dep -D VERSION_GIT_FULLHASH=\\\"authenticated-mrustc-0.12.0\\\" -D VERSION_GIT_BRANCH=\\\"v0.12.0\\\" -D VERSION_GIT_SHORTHASH=\\\"v0.12.0\\\" -D VERSION_BUILDTIME=\\\"1970-01-01T00:00:00Z\\\" -D VERSION_GIT_ISDIRTY=0\n","metadata rule");
    write_text(p,s); free(old); free(s); free(p);
    edit(root,"src/hir_typeck/expr_cs.cpp","                                context.possible_ivar_vals[te->index].force_disable = true;\n","                                if(te->index >= context.possible_ivar_vals.size()) {\n                                    context.possible_ivar_vals.resize(te->index + 1);\n                                }\n                                context.possible_ivar_vals[te->index].force_disable = true;\n");
    p=join(root,"Cargo.toml"); write_text(p,"[workspace]\nmembers = [\"lib/libproc_macro\"]\nresolver = \"2\"\n"); free(p);
    const char *out="    auto out_dir = parent.get_output_dir(m_is_for_host).to_absolute() / parent.get_build_script_out(m_manifest);\n";
    edit(root,"tools/minicargo/build.cpp",out,"    auto out_dir = parent.get_output_dir(m_is_for_host).to_absolute() / parent.get_build_script_out(m_manifest);\n    if( m_manifest.build_script() != \"\" ) {\n        out_dir = parent.get_output_dir(true).to_absolute() / parent.get_build_script_out(m_manifest);\n    }\n");
    edit(root,"tools/minicargo/build.cpp","        args.push_back(\"force-unstable-if-unmarked\");\n","        args.push_back(\"force-unstable-if-unmarked\");\n        // Mantle: keep static musl first-stage rustc single-threaded.\n        args.push_back(\"-Z\");\n        args.push_back(\"threads=1\");\n");
    edit(root,"minicargo.mk","LLVM_CMAKE_OPTS += LLVM_ENABLE_ZLIB=OFF LLVM_ENABLE_TERMINFO=OFF LLVM_ENABLE_LIBEDIT=OFF WITH_POLLY=OFF\n","LLVM_CMAKE_OPTS += LLVM_ENABLE_ZLIB=OFF LLVM_ENABLE_ZSTD=OFF LLVM_ENABLE_TERMINFO=OFF LLVM_ENABLE_LIBEDIT=OFF WITH_POLLY=OFF LLVM_ENABLE_BACKTRACES=OFF CMAKE_DISABLE_FIND_PACKAGE_Backtrace=ON CMAKE_DISABLE_FIND_PACKAGE_zstd=ON LLVM_TOOL_LTO_BUILD=OFF LLVM_BUILD_TOOLS=OFF\n");
    edit(root,"minicargo.mk","\t$Vcd $(RUSTCSRC)build && $(MAKE) -j $(PARLEVEL)\n",
        "\t$Vcd $(RUSTCSRC)build && $(MAKE) -j $(PARLEVEL) llvm-headers vt_gen llvm-config "
        "LLVMBinaryFormat LLVMMC LLVMAArch64Info LLVMBitstreamReader LLVMRemarks LLVMCore "
        "LLVMAArch64Utils LLVMCodeGenTypes LLVMAArch64Desc LLVMBitReader LLVMAsmParser "
        "LLVMIRReader LLVMMCParser LLVMTextAPI LLVMObject LLVMDebugInfoDWARF "
        "LLVMDebugInfoCodeView LLVMDebugInfoMSF LLVMDebugInfoPDB LLVMDebugInfoBTF "
        "LLVMSymbolize LLVMProfileData LLVMAnalysis LLVMBitWriter LLVMCGData "
        "LLVMTransformUtils LLVMObjCARCOpts LLVMAggressiveInstCombine LLVMInstCombine "
        "LLVMScalarOpts LLVMTarget LLVMCodeGen LLVMAsmPrinter LLVMCFGuard "
        "LLVMSelectionDAG LLVMGlobalISel LLVMSandboxIR LLVMVectorize LLVMAArch64CodeGen "
        "LLVMAArch64AsmParser LLVMMCDisassembler LLVMAArch64Disassembler LLVMARMInfo "
        "LLVMARMUtils LLVMARMDesc LLVMFrontendOffloading LLVMFrontendAtomic LLVMFrontendOpenMP "
        "LLVMLinker LLVMInstrumentation LLVMipo LLVMARMCodeGen LLVMARMAsmParser "
        "LLVMARMDisassembler LLVMCoverage LLVMExtensions LLVMCoroutines LLVMHipStdPar "
        "LLVMIRPrinter LLVMPasses LLVMLTO LLVMX86Info LLVMX86Desc LLVMX86CodeGen "
        "LLVMX86AsmParser LLVMX86Disassembler LLVMMCA LLVMX86TargetMCA\n");
    p=join(root,"rustc-1.90.0-src.patch"); s=normalize_patch(read_text(p)); write_text(p,s); free(s); free(p);
    p=join(root,"run_rustc/Makefile"); s=read_text(p);
    size_t capacity=strlen(runtime)*4+4096; char *line=allocate(capacity);
    snprintf(line,capacity,"RUSTC_ENV_VARS += LD_LIBRARY_PATH=%s:$(abspath $(PREFIX_2)lib):$(abspath $(LIBDIR))\n",runtime);
    s=replace(s,"RUSTC_ENV_VARS += LD_LIBRARY_PATH=$(abspath $(LIBDIR))\n",line,"run_rustc runtime");
    s=replace(s,"CARGO_ENV_STAGE2_STD := CARGO_TARGET_DIR=$(OUTDIR)build-std2 RUSTC=$(abspath rustc_proxy.sh) PROXY_RUSTC=$(abspath $(BINDIR_2)rustc) PROXY_MRUSTC=$(abspath $(BINDIR_S)rustc) $(CARGO_ENV)\n","CARGO_ENV_STAGE2_STD := $(RUSTC_ENV_VARS) CARGO_TARGET_DIR=$(OUTDIR)build-std2 RUSTC=$(abspath rustc_proxy.sh) PROXY_RUSTC=$(abspath $(BINDIR_2)rustc) PROXY_MRUSTC=$(abspath $(BINDIR_S)rustc) $(CARGO_ENV)\n","stage2 Cargo runtime");
    s=replace(s,"CARGO_ENV_RUSTC := CARGO_TARGET_DIR=$(OUTDIR)build-rustc RUSTC=$(abspath rustc_proxy.sh) PROXY_RUSTC=$(abspath $(BINDIR_2)rustc) PROXY_MRUSTC=$(abspath $(BINDIR_S)rustc) $(CARGO_ENV)\n","CARGO_ENV_RUSTC := CARGO_TARGET_DIR=$(OUTDIR)build-rustc RUSTC=$(abspath rustc_proxy.sh) PROXY_RUSTC=$(abspath $(BINDIR_2)rustc) PROXY_MRUSTC=$(abspath $(BINDIR_2)rustc) $(CARGO_ENV)\n","final Cargo host prefix");
    const char *rules[]={"$(BINDIR_S)rustc: ../output$(OUTDIR_SUF)/rustc\n","$(BINDIR_2)rustc: ../output$(OUTDIR_SUF)/rustc\n"};
    const char *prefixes[]={"PREFIX_S","PREFIX_2"};
    for(unsigned i=0;i<2;i++) {
        char *rule=strstr(s,rules[i]); if(!rule||strstr(rule+1,rules[i])) fail("run_rustc compiler copy rule");
        char *copy=strstr(rule,"\tcp $< $@\n"); if(!copy) fail("run_rustc copy recipe");
        /* Fail if a different target intervenes before the copy recipe. */
        for(char *scan=rule+strlen(rules[i]);scan<copy;scan++) if(*scan=='\n' && scan+1<copy && scan[1] && scan[1]!='\t' && scan[1]!='#' && scan[1]!='\n') fail("run_rustc copy rule boundary");
        size_t n=(size_t)(copy-rule)+strlen("\tcp $< $@\n"); char *block=allocate(n+1); memcpy(block,rule,n); block[n]=0;
        char *replacement=allocate(n+capacity); memcpy(replacement,rule,n-strlen("\tcp $< $@\n"));
        snprintf(replacement+n-strlen("\tcp $< $@\n"),capacity,"\tln -sf \"$(abspath $<)\" \"$@.bin\"\n\tprintf '#!/bin/sh\\nLD_LIBRARY_PATH=\"%s:$(abspath $(PREFIX_2)lib):$(abspath $(LIBDIR))\" \"%s/libc.so\" \"$$0.bin\" --sysroot \"$(abspath $(%s))\" \"$$@\"\\n' >$@\n\tchmod +x $@\n",runtime,runtime,prefixes[i]);
        s=replace(s,block,replacement,"run_rustc static sysroot recipe"); free(block); free(replacement);
    }
    s=replace(s,"\t$Vprintf '#!/bin/sh\\nd=$$(dirname $$0)\\nLD_LIBRARY_PATH=\"$(abspath $(OUTDIR)prefix/lib):$(abspath $(LIBDIR))\" $$d/rustc_binary \"$$@\"' >$@\n","\t$Vln -sf rustc_binary $(BINDIR)rustc_binary.sysroot\n\t$Vprintf '#!/bin/sh\\nd=$$(dirname $$0)\\nLD_LIBRARY_PATH=\"$(abspath $(OUTDIR)prefix/lib):$(abspath $(LIBDIR))\" $$d/rustc_binary.sysroot --sysroot \"$$d/..\" \"$$@\"' >$@\n","run_rustc final sysroot wrapper");
    s=replace(s,"\t$VTMPDIR=$(abspath $(PREFIX)tmp) $(CARGO_ENV_RUSTC) $(BINDIR_S)cargo build $(CARGO_FLAGS) --manifest-path $(RUST_SRC_CARGO)Cargo.toml\n","\t$VTMPDIR=$(abspath $(PREFIX)tmp) $(CARGO_ENV_RUSTC) $(BINDIR_S)cargo build $(CARGO_FLAGS) --manifest-path $(RUST_SRC_CARGO)Cargo.toml --features all-static\n","run_rustc Cargo all-static");
    write_text(p,s); free(s); free(p); free(line);
}
int main(int argc,char **argv) {
    if(argc!=6 && argc!=8) fail("usage: transformer mode version input-tree output-tree policy-input [mrustc-metadata-source mrustc-proc-macro-source]");
    const char *mode=argv[1],*version=argv[2]; char top[128];
    if(!strcmp(mode,"mrustc") && !strcmp(version,"0.12.0")) strcpy(top,"mrustc-0.12.0");
    else if(!strcmp(mode,"first-rust") && !strcmp(version,"1.90.0")) strcpy(top,"rustc-1.90.0-src");
    else if(!strcmp(mode,"successor") && (!strcmp(version,"1.91.1")||!strcmp(version,"1.92.0")||!strcmp(version,"1.93.1")||!strcmp(version,"1.94.1"))) snprintf(top,sizeof top,"rustc-%s-src",version);
    else fail("unsupported original route source/version");
    copy_tree(argv[3],argv[4]); char *root=join(argv[4],top);
    if(!strcmp(mode,"mrustc")) { if(argc!=8) fail("mrustc phase sources missing"); mrustc(root,argv[5],argv[6],argv[7]); }
    else if(!strcmp(mode,"first-rust")) rust190(root,argv[5]);
    else successor(root);
    free(root); return 0;
}
