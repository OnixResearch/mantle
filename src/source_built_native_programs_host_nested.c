/* Source continuations for nested original shell and GNU Make commands.
 * Appended after the common shell/controller and native GCC source, in one TU.
 * There is no process launcher here. Every Make invocation below crosses the
 * same immutable request/result boundary as its real compiler and utility leaves.
 */
#define MANTLE_NATIVE_CONTROL_DIRECTORY ".mantle-native-controllers"

static const char *mantle_native_basename(const char *path) {
    const char *slash = strrchr(path, '/');
    return slash ? slash + 1 : path;
}
static int mantle_native_shell_name(const char *name) {
    return !strcmp(name,"sh") || !strcmp(name,"bash") || !strcmp(name,"bash-full") || !strcmp(name,"dash");
}
static void mantle_native_env_free(Env *e, int close_streams) {
    if (close_streams) for (int i=0;i<64;i++) {
        int first=e->fd[i]!=NULL;
        for (int j=0;j<i;j++) if(e->fd[i]==e->fd[j]) first=0;
        if(first && fclose(e->fd[i])) die("nested source stream close");
    }
    clearvars(e); free(e->cwd);
}
/* A shell child inherits exported values and open descriptors, not its parent's
 * shell functions, readonly attributes, control flow, options or EXIT trap. */
static Env mantle_native_shell_environment(Env *parent) {
    Env e=clone(parent); e.funs=NULL; e.flow=e.depth=e.status=e.suppress=0;
    e.errexit=e.nounset=e.noglob=e.allexport=e.noclobber=e.substituted=0;
    memset(e.traps,0,sizeof e.traps);
    for(int i=0;i<64;i++)if(e.fd_cloexec[i]){e.fd[i]=NULL;e.fd_flags[i]=0;e.fd_cloexec[i]=0;}
    e.in=e.fd[0];e.out=e.fd[1];e.err=e.fd[2];
    Var **next=&e.vars;
    while(*next) {
        Var *v=*next;
        if(!v->exported) { *next=v->next;free(v->key);free(v->value);free(v); }
        else { v->readonly=0; next=&v->next; }
    }
    return e;
}
static int mantle_native_shell(Env *parent,Args *a) {
    if(!a->n) return -1;
    int first=1;
    const char *name=mantle_native_basename(a->v[0]);
    if(!strcmp(name,"busybox") && a->n>1 && mantle_native_shell_name(a->v[1])) first=2;
    else if(!mantle_native_shell_name(name)) return -1;
    Env e=mantle_native_shell_environment(parent);
    const char *shell_zero=first==2?a->v[1]:(a->argv0?a->argv0:a->v[0]);
    int command=0,standard_input=0;
    while(first<a->n && a->v[first][0]=='-' && a->v[first][1]) {
        const char *option=a->v[first++];
        if(!strcmp(option,"--")) break;
        for(const char *p=option+1;*p;p++) {
            if(*p=='c') command=1;
            else if(*p=='s') standard_input=1;
            else if(*p=='e') e.errexit=1;
            else if(*p=='u') e.nounset=1;
            else if(*p=='f') e.noglob=1;
            else if(*p!='x' && *p!='v') die("unsupported original shell option %s",option);
        }
        if(command) break;
    }
    char *source=NULL,*previous_source=parsing_source;
    if(command) {
        if(first==a->n) { mantle_native_env_free(&e,0); return 2; }
        source=ld_string_duplicate(a->v[first++]);
        e.zero=first<a->n?a->v[first++]:shell_zero;
    } else if(!standard_input && first<a->n) {
        e.zero=a->v[first++]; source=mantle_source_load(e.zero);
    } else {
        e.zero=shell_zero; Buf text={0}; char bytes[8192]; size_t n;
        while(e.in && (n=fread(bytes,1,sizeof bytes,e.in))) putn(&text,bytes,n);
        if(e.in && ferror(e.in)) die("original shell stdin");
        source=done(&text);
    }
    e.args=a->v+first; e.argc=a->n-first; parsing_source=e.zero;
    int status=script(&e,source);
    if(e.traps[0]) {
        char *trap=e.traps[0];e.traps[0]=NULL;e.flow=0;int saved=status;
        script(&e,trap);status=e.flow==4?e.status:saved;
    }
    parsing_source=previous_source; free(source);
    for(int i=0;i<64;i++)if(e.fd[i]) {
        int owned=1;for(int j=0;j<64;j++)if(e.fd[i]==parent->fd[j])owned=0;
        for(int j=0;j<i;j++)if(e.fd[i]==e.fd[j])owned=0;
        if(owned&&fclose(e.fd[i]))die("close original shell child descriptor");
    }
    mantle_native_env_free(&e,0);
    if(mantle_paths_chdir(parent->cwd)) die("restore nested shell logical cwd");
    return status;
}
static void mantle_native_copy(const char *from,const char *to) {
    struct stat st;if(lstat(from,&st))die("nested workspace stat %s",from);
    if(S_ISDIR(st.st_mode)) {
        mkdir_checked(to);DIR *d=opendir(from);if(!d)die("nested workspace directory");
        struct dirent *entry;
        while((entry=readdir(d))) {
            if(!strcmp(entry->d_name,".")||!strcmp(entry->d_name,"..")||!strcmp(entry->d_name,MANTLE_NATIVE_CONTROL_DIRECTORY))continue;
            char *a=join(from,entry->d_name),*b=join(to,entry->d_name);
            struct stat prior;if(!lstat(b,&prior)&&!S_ISDIR(prior.st_mode)&&S_ISDIR(st.st_mode))remove_tree(b);
            mantle_native_copy(a,b);free(a);free(b);
        }
        if(closedir(d))die("nested workspace close");
    } else copy_tree(from,to);
    struct timespec times[2]={{0,UTIME_OMIT},st.st_mtim};
    if(utimensat(AT_FDCWD,to,times,AT_SYMLINK_NOFOLLOW))die("nested workspace time");
}
static char *mantle_native_logical(const char *physical) {
    char logical[PATH_MAX];
    if(mantle_paths_logical(physical,logical,sizeof logical))die("nested control path leaves declared logical roots");
    return ld_string_duplicate(logical);
}
static int mantle_native_make(Env *parent,Args *a) {
    if(!a->n)return -1;
    const char *name=mantle_native_basename(a->v[0]);
    if(strcmp(name,"make")&&strcmp(name,"gmake"))return -1;
    char *workspace=workspace_root;
    const char *manifest=getenv("MANTLE_CONTROLLER_ROOTS");
    if(!manifest)die("nested Make requires Produced logical roots");
    struct mantle_paths_operand location;
    if(mantle_paths_map(parent->cwd,1,&location)||location.kind!=MANTLE_PATHS_WORKSPACE)die("original Make cwd leaves mutable source roots");
    char *logical_controls=join(location.logical_root,MANTLE_NATIVE_CONTROL_DIRECTORY);
    struct mantle_paths_operand control_location;
    if(mantle_paths_map(logical_controls,1,&control_location)||control_location.kind!=MANTLE_PATHS_WORKSPACE)die("nested Make control root");
    free(logical_controls);
    char *controls=ld_string_duplicate(control_location.physical);mkdir_checked(controls);
    char ordinal[64];snprintf(ordinal,sizeof ordinal,"%lu",request_ordinal);
    char *root=join(controls,ordinal);remove_tree(root);mkdir_checked(root);
    /* Capture outside the workspace first: a request contains a workspace
     * snapshot and must never recursively snapshot its own control tree. */
    char *temporary=join(output_root,"native-make-call");remove_tree(temporary);
    write_request(temporary,parent,a);
    char *snapshot=join(temporary,"workspace");remove_tree(snapshot);mantle_native_copy(workspace,snapshot);free(snapshot);
    char *initial=join(root,"initial-request");copy_tree(temporary,initial);remove_tree(temporary);free(temporary);
    Env process=clone(parent);
    /* Only this administrative transport has private stdio. All original
     * descriptors, including source FD5 and aliases, live in initial-request. */
    process.in=process.fd[0]=tmpfile();process.out=process.fd[1]=tmpfile();process.err=process.fd[2]=tmpfile();
    for(int i=0;i<64;i++){process.fd_flags[i]=i==0?O_RDONLY:i<3?O_WRONLY:0;process.fd_cloexec[i]=0;if(i>=3)process.fd[i]=NULL;}
    if(!process.in||!process.out||!process.err)die("nested Make private transport streams");
    char *previous=ld_string_duplicate("-"),*result=ld_string_duplicate("-");
    int status;
    for(unsigned long iteration=0;;iteration++) {
        char component[64];snprintf(component,sizeof component,"checkpoint-%lu",iteration);char *checkpoint=join(root,component);
        char *initial_arg=mantle_native_logical(initial),*previous_arg=!strcmp(previous,"-")?ld_string_duplicate("-"):mantle_native_logical(previous);
        char *result_arg=!strcmp(result,"-")?ld_string_duplicate("-"):mantle_native_logical(result),*checkpoint_arg=mantle_native_logical(checkpoint);
        Args run={0};arg(&run,a->v[0]);arg(&run,"--mantle-source-control");arg(&run,initial_arg);arg(&run,previous_arg);arg(&run,result_arg);arg(&run,checkpoint_arg);
        arg(&run,a->argv0?a->argv0:a->v[0]);for(int i=1;i<a->n;i++)arg(&run,a->v[i]);
        int actual=mantle_shell_external(&process,&run);
        free(run.v);free(initial_arg);free(previous_arg);free(result_arg);free(checkpoint_arg);
        if(actual!=0&&actual!=1)die("original Make source controller failed before an authenticated source outcome");
        if(text_is(checkpoint,"state","done\n")) {
            if(actual!=1)die("completed Make lacks actual completion predicate");
            /* Source-result is inside the control tree being replaced. Keep
             * both immutable acknowledgements outside that tree for restore. */
            char *call=join(output_root,"native-make-return-call"),*answer=join(output_root,"native-make-return-result");
            remove_tree(call);remove_tree(answer);copy_tree(initial,call);
            char *source_result=join(checkpoint,"source-result");copy_tree(source_result,answer);free(source_result);
            status=restore_result(call,answer,parent);
            remove_tree(call);remove_tree(answer);free(call);free(answer);
            free(checkpoint);break;
        }
        if(actual||!text_is(checkpoint,"state","paused\n"))die("unknown original Make continuation state");
        snprintf(component,sizeof component,"bridge-%lu",iteration);char *bridge=join(root,component);mkdir_checked(bridge);
        if(chdir(bridge))die("nested Make request bridge cwd");
        Request request;char *request_path=join(checkpoint,"request");read_request_data(request_path,&request);free(request_path);
        if(!request.active)die("paused original Make has no request");
        workspace_root=request.workspace;
        if(mantle_paths_init(workspace_root,manifest,request.env.cwd)||mantle_paths_chdir(request.env.cwd))die("nested Make child logical workspace");
        int child_status=external(&request.env,&request.command);
        char *acknowledged=join(bridge,"result");write_result(acknowledged,&request,child_status,NULL,NULL);
        mantle_native_env_free(&request.env,1);
        workspace_root=workspace;
        if(mantle_paths_init(workspace,manifest,parent->cwd)||mantle_paths_chdir(parent->cwd))die("restore original Make logical workspace");
        free(previous);previous=checkpoint;free(result);result=acknowledged;free(bridge);
        if(iteration==100000UL)die("nested Make continuation bound");
    }
    free(previous);free(result);mantle_native_env_free(&process,1);
    if(mantle_paths_chdir(parent->cwd))die("restore source command logical cwd");
    remove_tree(root);free(initial);free(root);free(controls);
    return status;
}
static char *mantle_native_script_path(Env *e,const char *name) {
    struct mantle_paths_operand p;
    if(strchr(name,'/'))return mantle_paths_map(name,0,&p)?NULL:ld_string_duplicate(name);
    const char *path=get(e,"PATH");if(!path)return NULL;
    for(const char *s=path;;){const char *end=strchr(s,':');size_t n=end?(size_t)(end-s):strlen(s);char *directory=alloc(n+1);memcpy(directory,s,n);char *candidate=n?join(directory,name):ld_string_duplicate(name);free(directory);struct stat st;
        if(!mantle_paths_map(candidate,0,&p)&&!stat(p.physical,&st)&&S_ISREG(st.st_mode)&&(st.st_mode&0111))return candidate;
        free(candidate);if(!end)break;s=end+1;
    }
    return NULL;
}
/* The bytes, including interpreter and optional argument, come from the actual
 * source executable. In particular generated config.status and compiler shell
 * wrappers never become opaque executable leaves. */
static int mantle_native_script(Env *e,Args *a) {
    if(!a->n)return -1;char *path=mantle_native_script_path(e,a->v[0]);if(!path)return -1;
    struct mantle_paths_operand p;if(mantle_paths_map(path,0,&p))die("source executable path changed");
    FILE *f=fopen(p.physical,"rb");if(!f)die("source executable read");char line[4096];
    if(!fgets(line,sizeof line,f)){if(ferror(f))die("source executable first line");if(fclose(f))die("source executable close");free(path);return -1;}
    if(fclose(f))die("source executable close");if(strncmp(line,"#!",2)){free(path);return -1;}
    char *interpreter=line+2;while(*interpreter==' '||*interpreter=='\t')interpreter++;
    char *end=interpreter;while(*end&&*end!=' '&&*end!='\t'&&*end!='\n'&&*end!='\r')end++;
    char *option=end;if(*option)*option++=0;while(*option==' '||*option=='\t')option++;
    char *tail=option+strlen(option);while(tail>option&&(tail[-1]=='\n'||tail[-1]=='\r'||tail[-1]==' '||tail[-1]=='\t'))*--tail=0;
    if(!*interpreter)die("empty original source interpreter");Args command={0};arg(&command,interpreter);if(*option)arg(&command,option);arg(&command,path);
    for(int i=1;i<a->n;i++)arg(&command,a->v[i]);int status=external(e,&command);free(command.v);free(path);return status;
}
static int mantle_native_env(Env *parent,Args *a) {
    if(!a->n)return -1;int first=1;const char *name=mantle_native_basename(a->v[0]);
    if(!strcmp(name,"busybox")&&a->n>1&&!strcmp(a->v[1],"env"))first=2;else if(strcmp(name,"env"))return -1;
    Env e=mantle_native_shell_environment(parent);
    while(first<a->n){const char *word=a->v[first];if(!strcmp(word,"--")){first++;break;}
        if(!strcmp(word,"-")||!strcmp(word,"-i")||!strcmp(word,"--ignore-environment")){clearvars(&e);first++;continue;}
        const char *remove=NULL;if(!strncmp(word,"--unset=",8))remove=word+8;else if(!strcmp(word,"-u")){if(++first==a->n){mantle_native_env_free(&e,0);return -1;}remove=a->v[first];}
        if(remove){Var **v=&e.vars;while(*v){if(!strcmp((*v)->key,remove)){Var *old=*v;*v=old->next;free(old->key);free(old->value);free(old);break;}v=&(*v)->next;}first++;continue;}
        if(word[0]=='-'){mantle_native_env_free(&e,0);return -1;}break;
    }
    while(first<a->n){const char *equals=strchr(a->v[first],'=');if(!equals)break;size_t n=(size_t)(equals-a->v[first]);char *key=alloc(n+1);memcpy(key,a->v[first],n);set(&e,key,equals+1);variable(&e,key)->exported=1;free(key);first++;}
    if(first==a->n){mantle_native_env_free(&e,0);return -1;}
    Args command={.v=a->v+first,.n=a->n-first};int status=external(&e,&command);mantle_native_env_free(&e,0);
    if(mantle_paths_chdir(parent->cwd))die("restore source environment wrapper cwd");return status;
}
static int mantle_native_timeout(Env *,Args *);
static int mantle_native_external(Env *e,Args *a) {
    int timed=mantle_native_timeout(e,a);if(timed>=0)return timed;
    int status=mantle_native_shell(e,a);if(status>=0)return status;
    status=mantle_native_env(e,a);if(status>=0)return status;
    status=mantle_native_script(e,a);if(status>=0)return status;
    status=mantle_native_make(e,a);if(status>=0)return status;
    const char *linker=get(e,"LD");
    if(linker&&*linker&&!get(e,"MANTLE_NATIVE_LINKER")){set(e,"MANTLE_NATIVE_LINKER",linker);variable(e,"MANTLE_NATIVE_LINKER")->exported=1;}
    status=native_gcc_external(e,a);if(status>=0)return status;
    return mantle_shell_external(e,a);
}

static int mantle_native_catalog_contains(const char *directory,const char *installed,const struct stat *wanted,unsigned long *count) {
    if(++*count>262144UL)die("native generated inventory cardinality");
    struct stat st;if(lstat(directory,&st))die("native generated inventory stat");
    if(S_ISDIR(st.st_mode)) {
        DIR *d=opendir(directory);if(!d)die("native generated inventory directory");
        struct dirent *entry;int found=0;
        while(!found&&(entry=readdir(d))) {
            if(!strcmp(entry->d_name,".")||!strcmp(entry->d_name,".."))continue;
            char *path=join(directory,entry->d_name);found=mantle_native_catalog_contains(path,installed,wanted,count);free(path);
        }
        if(closedir(d))die("native generated inventory close");return found;
    }
    return S_ISREG(st.st_mode)&&(st.st_mode&0111)&&st.st_size==wanted->st_size&&files_equal(directory,installed);
}
static int mantle_native_installed(int argc,char **argv) {
    if(argc!=5)die("native installed executable verification layout");
    struct stat st;if(stat(argv[2],&st)||!S_ISREG(st.st_mode)||!(st.st_mode&0111))die("native installed product does not resolve to a regular executable");
    FILE *f=fopen(argv[2],"rb");unsigned char magic[4];
    if(!f||fread(magic,1,4,f)!=4||memcmp(magic,"\177ELF",4))die("native installed image is not an original compiled ELF");
    if(fclose(f))die("native installed image close");
    unsigned long count=0;if(!mantle_native_catalog_contains(argv[3],argv[2],&st,&count))die("native installed image lacks an actual isolated compiler/link product");
    binary_copy(argv[2],argv[4]);if(chmod(argv[4],st.st_mode&0777))die("native installed executable mode");return 0;
}
