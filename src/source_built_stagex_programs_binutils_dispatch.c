/* Original StageX binutils leaf option grammars, composed after the shared
 * source shell, native GCC grammar and original GNU generator continuations.
 * Selection is still exact immutable image identity, never this basename. */
static int stagex_binutils_relative_name(const char *name)
{
    return name&&*name&&strlen(name)<=255&&name[0]!='/'&&
        !strchr(name,'/')&&strcmp(name,"..");
}
static int stagex_binutils_utility_prepare(Request *r,mantle_controller_builder *b)
{
    Args *a=&r->command;const char *name=mantle_controller_basename(a->v[0]);
    if(strcmp(name,"file")&&strcmp(name,"emit"))return 0;
    int operand=a->n==2?1:(!strcmp(name,"file")&&a->n==3&&!strcmp(a->v[1],"-L")?2:0);
    if(operand&&strcmp(a->v[operand],"--mantle-self-test")&&stagex_binutils_relative_name(a->v[operand])) {
        for(int i=1;i<a->n;i++)if(i==operand)mantle_controller_cwd_relative(b,a->v[i]);else mantle_controller_argument_data(b,a->v[i]);
    } else {
        /* Rejected argument shapes never perform I/O in the original helper.
         * Preserve their bytes so its real authority/status checks still run. */
        for(int i=1;i<a->n;i++)mantle_controller_argument_data(b,a->v[i]);
    }
    return 1;
}
static int stagex_binutils_prepare(const char *slot,Request *r,mantle_controller_builder *b)
{
    Args *a=&r->command;if(!a->n)return 0;
    const char *name=mantle_controller_basename(a->v[0]);
    if(!strcmp(name,"tcc-0.9.27-musl-v2")) {
        /* Reuse only the option grammar, not GCC's driver process expansion.
         * Restore the genuine argv0 before immutable invocation metadata. */
        char *original=a->v[0];a->v[0]=(char *)"tcc";
        int handled=native_gcc_prepare(slot,r,b);a->v[0]=original;return handled;
    }
    if(mantle_controller_oneof(name,"stagex-elf-local-symbol-canonicalizer binutils-elf-symbol-canonicalizer")) {
        if(a->n!=2)die("original binutils canonicalizer argument count");
        native_gcc_generated_environment(&r->env);
        mantle_controller_argument_path(b,a->v[1],1);return 1;
    }
    if(!strcmp(name,"stagex-ylwrap-sed-runner")) {
        if(a->n==2&&!strcmp(a->v[1],"--mantle-self-test")){mantle_controller_argument_data(b,a->v[1]);return 1;}
        /* The four exact programs remain opaque source data. The original
         * compiled runner, not the transport, validates their restricted DSL. */
        for(int i=1;i<a->n;i++) {
            if(i==9&&stagex_binutils_relative_name(a->v[i]))mantle_controller_cwd_relative(b,a->v[i]);
            else mantle_controller_argument_data(b,a->v[i]);
        }
        return 1;
    }
    if(stagex_binutils_utility_prepare(r,b))return 1;
    if(mantle_controller_oneof(slot,"chew generated native-generated")) {
        if(!strcmp(name,"chew")) {
            int first=1;
            if(a->n==4&&!strcmp(a->v[first],"-i")){mantle_controller_argument_literal(b,a->v[first]);first++;}
            if(a->n!=first+2||strcmp(a->v[first],"-f"))die("original BFD chew grammar");
            mantle_controller_argument_literal(b,"-f");mantle_controller_argument_path(b,a->v[first+1],0);return 1;
        }
        if(!strcmp(name,"i386-gen")) {
            if(a->n!=3||strcmp(a->v[1],"--srcdir"))die("original i386 generator grammar");
            mantle_controller_argument_literal(b,"--srcdir");mantle_controller_argument_path(b,a->v[2],0);return 1;
        }
        if(!strcmp(name,"sysinfo")) {
            if(a->n!=2||!mantle_controller_oneof(a->v[1],"-c -i -g -d"))die("original sysinfo generator grammar");
            mantle_controller_argument_literal(b,a->v[1]);return 1;
        }
        if(!strcmp(name,"gen-aout")) {
            if(a->n!=2||strcmp(a->v[1],"host"))die("original aout generator grammar");
            mantle_controller_argument_literal(b,"host");return 1;
        }
        /* Original configure executable probes take no arguments. Their ELF
         * bytes must already match an actual earlier compiler product. */
        if(a->n==1)return 1;
    }
    return 0;
}
static char *stagex_binutils_environment(mantle_controller_builder *b,const char *key,const char *value)
{
    /* These are original path-valued environment entries, not arbitrary strings
     * reclassified because their spelling happens to contain a slash. Scripts,
     * compiler command strings, regexes and configuration prefixes stay Data. */
    int writable=mantle_controller_oneof(key,"MANTLE_BINUTILS_CONFIGURE_PROBE_DIR MANTLE_BINUTILS_CONFIGURE_PROBE_AUDIT MANTLE_BINUTILS_CONFIGURE_PROBE_COUNT MANTLE_STAGE_X_SED_SPOOL_ROOT MANTLE_STAGE_X_SED_BRIDGE_AUDIT MANTLE_STAGE_X_AR_AUDIT MANTLE_STAGE_X_AR_COUNT MANTLE_STAGE_X_AR_SCRATCH");
    int readonly=mantle_controller_oneof(key,"MANTLE_STAGE_X_FILE MANTLE_STAGE_X_SED_BRIDGE_BASH MANTLE_STAGE_X_SED_BRIDGE_SCRIPT MANTLE_STAGE_X_SED_BRIDGE_TARGET MANTLE_STAGE_X_SED_BRIDGE_LAUNCHER MANTLE_STAGE_X_YLWRAP_SED MANTLE_STAGE_X_SED_BRIDGE_CAT MANTLE_STAGE_X_SED_BRIDGE_HEAD MANTLE_STAGE_X_SED_BRIDGE_MKDIR MANTLE_STAGE_X_SED_BRIDGE_RM MANTLE_STAGE_X_SED_BRIDGE_RMDIR MANTLE_STAGE_X_SED_BRIDGE_WC MANTLE_STAGE_X_SED_BRIDGE_EMIT MANTLE_STAGE_X_AR_BASENAME MANTLE_STAGE_X_AR_CAT MANTLE_STAGE_X_AR_MV MANTLE_STAGE_X_AR_RM MANTLE_STAGE_X_AR_WC");
    if(!writable&&!readonly)return NULL;
    mantle_controller_word_begin(b);if(*value)mantle_controller_path(b,value,writable);else mantle_controller_literal(b,"");return mantle_controller_take_word(b);
}

#include <sys/resource.h>
struct stagex_binutils_limit_scope {
    struct rlimit outer;
    struct stagex_binutils_limit_scope *parent;
    int suspended;
};
static struct stagex_binutils_limit_scope *stagex_binutils_limit;
static void stagex_binutils_limit_suspend(Env *e)
{
    struct stagex_binutils_limit_scope *scope=stagex_binutils_limit;
    if(!scope||scope->suspended)return;
    /* Flush original buffered writes before lifting the source limit. The
     * checkpoint's private copying is not an original shell file write. */
    for(int i=0;i<64;i++)if(e->fd[i]) {
        int first=1;for(int j=0;j<i;j++)if(e->fd[j]==e->fd[i])first=0;
        if(first&&fflush(e->fd[i]))die("original sed bridge source stream flush");
    }
    while(scope->parent)scope=scope->parent;
    if(setrlimit(RLIMIT_FSIZE,&scope->outer))die("restore source controller file limit");
    stagex_binutils_limit->suspended=1;
}
static void stagex_binutils_limit_resume(Env *e)
{
    (void)e;
    if(!stagex_binutils_limit||!stagex_binutils_limit->suspended)return;
    struct rlimit limit=stagex_binutils_limit->outer;limit.rlim_cur=MANTLE_SPOOL_BYTES_MAX;
    if(setrlimit(RLIMIT_FSIZE,&limit))die("restore original sed bridge source scope");
    stagex_binutils_limit->suspended=0;
}
static int stagex_binutils_launcher_error(Env *e,const char *message,int status)
{
    if(e->err)fputs(message,e->err);return status;
}
static int stagex_binutils_external(Env *e,Args *a)
{
    if(!a->n)return -1;
    const char *authority="/mantle/stagex/transition/binutils-stage/runtime/tools/sed";
    const char *alias="/mantle/stagex/transition/binutils-stage/runtime/stagex-sed-bridge-launcher";
    const char *binding=get(e,"MANTLE_STAGE_X_SED_BRIDGE_LAUNCHER");
    if(binding&&strcmp(binding,authority))die("original sed launcher binding changed");
    char *image=mantle_native_script_path(e,a->v[0]);
    int selected=image&&(!strcmp(image,authority)||!strcmp(image,alias));
    if(selected) {
        struct mantle_paths_operand actual,original;
        if(mantle_paths_map(image,0,&actual)||mantle_paths_map(authority,0,&original)||!files_equal(actual.physical,original.physical))die("original sed launcher alias image changed");
    }
    free(image);if(!selected)return -1;
    /* bootstrap/stagex-sed-bridge-launcher.c:48-78. Only execve's transport
     * changes: the actual Bash source child is evaluated in a fresh shell Env,
     * and each original external command pauses at its own immutable boundary. */
    if(a->n==MANTLE_SELF_TEST_ARGUMENT_COUNT&&!strcmp(a->v[1],"--mantle-self-test"))return stagex_binutils_launcher_self_test();
    if(a->n<1||a->n>MANTLE_ARGUMENT_COUNT_MAX)return stagex_binutils_launcher_error(e,"stagex-sed-bridge: argument limit exceeded\n",MANTLE_AUTHORITY_FAILURE);
    const char *bash=get(e,"MANTLE_STAGE_X_SED_BRIDGE_BASH"),*source=get(e,"MANTLE_STAGE_X_SED_BRIDGE_SCRIPT");
    if(!stagex_binutils_absolute_path(bash)||!stagex_binutils_absolute_path(source))return stagex_binutils_launcher_error(e,"stagex-sed-bridge: missing absolute launcher authority\n",MANTLE_AUTHORITY_FAILURE);
    struct stagex_binutils_limit_scope scope;scope.parent=stagex_binutils_limit;scope.suspended=0;
    if(getrlimit(RLIMIT_FSIZE,&scope.outer))return stagex_binutils_launcher_error(e,"stagex-sed-bridge: file limit setup failed\n",MANTLE_AUTHORITY_FAILURE);
    /* The controller retains its outer hard ceiling solely so private replay
     * snapshots can be written outside source scope. The typed invocation
     * projects the original hard AND soft ceiling onto every actual raw leaf. */
    struct rlimit limit=scope.outer;limit.rlim_cur=MANTLE_SPOOL_BYTES_MAX;
    if(limit.rlim_max<limit.rlim_cur||setrlimit(RLIMIT_FSIZE,&limit))return stagex_binutils_launcher_error(e,"stagex-sed-bridge: file limit setup failed\n",MANTLE_AUTHORITY_FAILURE);
    void (*suspend_prior)(Env *)=mantle_source_limit_suspend_hook;
    void (*resume_prior)(Env *)=mantle_source_limit_resume_hook;
    stagex_binutils_limit=&scope;mantle_source_limit_suspend_hook=stagex_binutils_limit_suspend;
    mantle_source_limit_resume_hook=stagex_binutils_limit_resume;
    Env child=mantle_native_shell_environment(e);
    char file_limit[32];snprintf(file_limit,sizeof file_limit,"%u",(unsigned)MANTLE_SPOOL_BYTES_MAX);
    native_gcc_export(&child,"MANTLE_STAGE_X_SED_BRIDGE_FSIZE",file_limit);
    Args command={0};arg(&command,bash);arg(&command,source);
    for(int i=1;i<a->n;i++)arg(&command,a->v[i]);
    int status=mantle_native_shell(&child,&command);
    stagex_binutils_limit_suspend(&child);
    if(setrlimit(RLIMIT_FSIZE,&scope.outer))die("restore outer sed launcher source limit");
    mantle_source_limit_suspend_hook=suspend_prior;mantle_source_limit_resume_hook=resume_prior;
    stagex_binutils_limit=scope.parent;
    free(command.v);mantle_native_env_free(&child,0);
    if(status<0)return stagex_binutils_launcher_error(e,"stagex-sed-bridge: exact Bash launch failed\n",MANTLE_EXEC_FAILURE);
    return status;
}
