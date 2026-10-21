/* Original Bison/Flex subprocess continuations, composed after the shared
 * source controller and native GCC hook. Every mantle_shell_external call is
 * one independently authenticated Produced command, never a process launch. */
struct final_gnu_capability { const char *image; int kind; };
static const struct final_gnu_capability final_gnu_capabilities[] = {
    FINAL_GNU_CAPABILITIES
    {NULL,0}
};
static char *final_gnu_physical(Env *e,const char *path,int writing)
{
    static const char output[]="/outputs/workspace/";
    if(!strncmp(path,output,sizeof output-1)) {
        mantle_controller_relative(path+sizeof output-1);
        return join(workspace_root,path+sizeof output-1);
    }
    char *logical=*path=='/'?dup(path):join(e->cwd,path);
    struct mantle_paths_operand mapped;
    if(mantle_paths_map(logical,writing,&mapped))die("original GNU source path %s: %s",logical,strerror(errno));
    free(logical);return dup(mapped.physical);
}
static int final_gnu_capability(Env *e,Args *a)
{
    if(!a->n)return 0;
    if(strchr(a->v[0],'/')) {
        char *logical=*a->v[0]=='/'?dup(a->v[0]):join(e->cwd,a->v[0]);
        struct mantle_paths_operand mapped;
        int known=mantle_paths_map(logical,0,&mapped)==0,result=0;
        for(size_t i=0;final_gnu_capabilities[i].image;i++)
            if(!strcmp(logical,final_gnu_capabilities[i].image)||(known&&!strcmp(mapped.logical,final_gnu_capabilities[i].image))) { result=final_gnu_capabilities[i].kind;break; }
        free(logical);return result;
    }
    const char *path=get(e,"PATH");if(!path)return 0;
    for(const char *p=path;;) {
        const char *end=strchr(p,':');size_t n=end?(size_t)(end-p):strlen(p);
        char *directory=n?slice(p,n):dup(e->cwd),*logical=join(directory,a->v[0]);free(directory);
        struct mantle_paths_operand mapped;struct stat st;
        if(!mantle_paths_map(logical,0,&mapped)&&!stat(mapped.physical,&st)&&S_ISREG(st.st_mode)&&(st.st_mode&0111)) {
            int result=0;
            for(size_t i=0;final_gnu_capabilities[i].image;i++)if(!strcmp(logical,final_gnu_capabilities[i].image)||!strcmp(mapped.logical,final_gnu_capabilities[i].image)){result=final_gnu_capabilities[i].kind;break;}
            free(logical);return result;
        }
        free(logical);if(!end)break;p=end+1;
    }
    return 0;
}
static FILE *final_gnu_stream(void) { FILE *f=tmpfile();if(!f)die("original GNU phase stream");return f; }
static void final_gnu_rewind(FILE *f) { if(fflush(f)||fseek(f,0,SEEK_SET))die("original GNU phase stream rewind"); }
static void final_gnu_emit(FILE *from,FILE *to) { final_gnu_rewind(from);transfer(from,to); }
static void final_gnu_close(FILE *f) { if(fclose(f))die("original GNU phase stream close"); }
static void final_gnu_environment_free(Env *e) { clearvars(e);free(e->cwd); }
static char *final_gnu_root(Env *e)
{
    const char *base="/tmp/.mantle-final-generators";
    char *physical=final_gnu_physical(e,base,1);mkdir_checked(physical);free(physical);
    char ordinal[64];snprintf(ordinal,sizeof ordinal,"%lu",request_ordinal);
    char *root=join(base,ordinal);physical=final_gnu_physical(e,root,1);remove_tree(physical);mkdir_checked(physical);free(physical);return root;
}
static void final_gnu_remove(Env *e,char *root) { char *physical=final_gnu_physical(e,root,1);remove_tree(physical);free(physical);free(root); }
static char *final_gnu_member(const char *root,const char *name) { return join(root,name); }
static void final_gnu_save_stream(Env *e,const char *path,FILE *stream)
{
    char *physical=final_gnu_physical(e,path,1);FILE *out=fopen(physical,"wb");if(!out)die("original GNU phase output %s",path);
    final_gnu_emit(stream,out);final_gnu_close(out);free(physical);
}
static void final_gnu_save_status(Env *e,const char *path,int status)
{
    if(status<0||status>255)die("original GNU child normal exit status");
    char *physical=final_gnu_physical(e,path,1);FILE *out=fopen(physical,"wb");if(!out||fprintf(out,"%d\n",status)<0)die("original GNU actual child status");final_gnu_close(out);free(physical);
}
static char *final_gnu_load(Env *e,const char *path) { char *physical=final_gnu_physical(e,path,0),*text=load(physical);free(physical);return text; }
static void final_gnu_response(Env *e,const char *response,const char *image,Args *command)
{
    char *selected=final_gnu_load(e,image),*end=strchr(selected,'\n');
    if(!end||end==selected||end[1])die("original Bison selected child image boundary");*end=0;arg(command,selected);
    char *text=final_gnu_load(e,response),*cursor=text;
    while(*cursor) {
        end=strchr(cursor,'\n');if(!end)die("original Bison unterminated m4 response");*end=0;
        size_t n=native_gcc_phase_split(cursor);if(n!=1)die("original Bison response argument boundary");
        arg(command,dup(native_gcc_phase_arguments[0]));cursor=end+1;
    }
    free(text);
}
static int final_gnu_bison(Env *e,Args *a)
{
    char *root=final_gnu_root(e),*response=final_gnu_member(root,"response"),*image=final_gnu_member(root,"image");
    char *result=final_gnu_member(root,"result"),*status_file=final_gnu_member(root,"status"),*diagnostic=final_gnu_member(root,"diagnostic");
    FILE *muscles=final_gnu_stream(),*errors=final_gnu_stream();
    long input_position=e->in?ftell(e->in):0;if(input_position<0)die("original Bison input position");
    Env capture=clone(e);capture.out=capture.fd[1]=muscles;capture.err=capture.fd[2]=errors;
    capture.fd_flags[1]=capture.fd_flags[2]=O_WRONLY;capture.fd_cloexec[1]=capture.fd_cloexec[2]=0;
    native_gcc_export(&capture,"MANTLE_BISON_MODE","capture");native_gcc_export(&capture,"MANTLE_BISON_RESPONSE",response);
    native_gcc_export(&capture,"MANTLE_BISON_IMAGE",image);native_gcc_export(&capture,"MANTLE_PHASE_CWD",e->cwd);
    int status=mantle_shell_external(&capture,a);final_gnu_environment_free(&capture);
    if(status!=86) { final_gnu_emit(muscles,e->out);final_gnu_emit(errors,e->err); }
    else {
        Args child={0};final_gnu_response(e,response,image,&child);final_gnu_rewind(muscles);
        FILE *expanded=final_gnu_stream(),*child_errors=final_gnu_stream();Env macro=clone(e);
        macro.in=macro.fd[0]=muscles;macro.out=macro.fd[1]=expanded;macro.err=macro.fd[2]=child_errors;
        macro.fd_flags[0]=O_RDONLY;macro.fd_flags[1]=macro.fd_flags[2]=O_WRONLY;
        macro.fd_cloexec[0]=macro.fd_cloexec[1]=macro.fd_cloexec[2]=0;
        int child_status=mantle_source_external_hook?mantle_source_external_hook(&macro,&child):mantle_native_external(&macro,&child);final_gnu_environment_free(&macro);native_gcc_args_free(&child);
        final_gnu_save_stream(e,result,expanded);final_gnu_save_stream(e,diagnostic,child_errors);final_gnu_save_status(e,status_file,child_status);
        final_gnu_close(expanded);final_gnu_close(child_errors);
        if(e->in&&fseek(e->in,input_position,SEEK_SET))die("original Bison replay input");
        Env replay=clone(e);native_gcc_export(&replay,"MANTLE_BISON_MODE","replay");
        native_gcc_export(&replay,"MANTLE_BISON_RESPONSE",response);native_gcc_export(&replay,"MANTLE_BISON_IMAGE",image);
        native_gcc_export(&replay,"MANTLE_BISON_RESULT",result);native_gcc_export(&replay,"MANTLE_BISON_STATUS",status_file);
        native_gcc_export(&replay,"MANTLE_BISON_STDERR",diagnostic);native_gcc_export(&replay,"MANTLE_PHASE_CWD",e->cwd);
        status=mantle_shell_external(&replay,a);final_gnu_environment_free(&replay);
        if(status==86)die("original Bison replay yielded a second unexpanded child");
    }
    final_gnu_close(muscles);final_gnu_close(errors);free(response);free(image);free(result);free(status_file);free(diagnostic);final_gnu_remove(e,root);return status;
}
static char *final_gnu_state_line(FILE *f)
{
    Buf line={0};int c;while((c=fgetc(f))!=EOF&&c!='\n'){if(c==0||line.n>=4095)die("original Flex state line bound");char byte=(char)c;putn(&line,&byte,1);}
    if(c!='\n')die("original Flex state line terminator");return done(&line);
}
static int final_gnu_filter(Env *e,const char *image,const char *mode,const char *state,FILE *in,FILE *out,const char *header)
{
    Env phase=clone(e);phase.in=phase.fd[0]=in;phase.out=phase.fd[1]=out;
    phase.fd_flags[0]=O_RDONLY;phase.fd_flags[1]=O_WRONLY;phase.fd_cloexec[0]=phase.fd_cloexec[1]=0;
    if(header)native_gcc_export(&phase,"MANTLE_FLEX_HEADER",header);
    Args args={0};arg(&args,dup(image));arg(&args,dup(mode));arg(&args,dup(state));
    final_gnu_rewind(in);int status=mantle_shell_external(&phase,&args);native_gcc_args_free(&args);final_gnu_environment_free(&phase);return status;
}
static int final_gnu_m4(Env *e,Args *command,FILE *in,FILE *out)
{
    Env phase=clone(e);phase.in=phase.fd[0]=in;phase.out=phase.fd[1]=out;
    phase.fd_flags[0]=O_RDONLY;phase.fd_flags[1]=O_WRONLY;phase.fd_cloexec[0]=phase.fd_cloexec[1]=0;
    final_gnu_rewind(in);
    int status=mantle_source_external_hook?mantle_source_external_hook(&phase,command):mantle_native_external(&phase,command);final_gnu_environment_free(&phase);return status;
}
static int final_gnu_flex(Env *e,Args *a)
{
    char *root=final_gnu_root(e),*state_path=final_gnu_member(root,"state"),*header_path=final_gnu_member(root,"header-m4");
    char *response_path=final_gnu_member(root,"response");
    FILE *raw=final_gnu_stream(),*errors=final_gnu_stream();Env capture=clone(e);
    capture.out=capture.fd[1]=raw;capture.err=capture.fd[2]=errors;
    capture.fd_flags[1]=capture.fd_flags[2]=O_WRONLY;capture.fd_cloexec[1]=capture.fd_cloexec[2]=0;
    native_gcc_export(&capture,"MANTLE_FLEX_STATE",state_path);native_gcc_export(&capture,"MANTLE_PHASE_CWD",e->cwd);
    native_gcc_export(&capture,"MANTLE_FLEX_RESPONSE",response_path);
    int status=mantle_shell_external(&capture,a);final_gnu_environment_free(&capture);
    char *physical=final_gnu_physical(e,state_path,0);struct stat st;
    if(status||stat(physical,&st)) {
        if(!status&&errno!=ENOENT)die("original Flex phase state stat");
        final_gnu_emit(raw,e->out);final_gnu_emit(errors,e->err);free(physical);
    } else {
        FILE *state=fopen(physical,"rb");if(!state)die("original Flex phase state");
        char *prefix=final_gnu_state_line(state),*output=final_gnu_state_line(state),*header=final_gnu_state_line(state),*linedirs=final_gnu_state_line(state);
        if(fgetc(state)!=EOF||ferror(state))die("original Flex trailing phase state");final_gnu_close(state);free(physical);free(prefix);free(linedirs);
        if(*output) { physical=final_gnu_physical(e,output,0);FILE *file=fopen(physical,"rb");if(!file)die("original Flex raw scanner");if(ftruncate(fileno(raw),0)||fseek(raw,0,SEEK_SET))die("original Flex raw stream reset");transfer(file,raw);final_gnu_close(file);free(physical); }
        FILE *code=final_gnu_stream();status=final_gnu_filter(e,a->v[0],"--mantle-tee",state_path,raw,code,header_path);
        final_gnu_emit(errors,e->err);
        physical=final_gnu_physical(e,response_path,0);FILE *response=fopen(physical,"rb");if(!response)die("original Flex child response");free(physical);
        char *count_text=final_gnu_state_line(response),*end;errno=0;long count=strtol(count_text,&end,10);
        if(errno||*end||count<1||count>16384)die("original Flex child argument count");free(count_text);
        Args macro={0};for(long i=0;i<count;i++){char *word=read_record(response);if(strlen(word)>65536)die("original Flex child word bound");arg(&macro,word);}
        if(fgetc(response)!=EOF||ferror(response))die("original Flex trailing child response");final_gnu_close(response);
        for(int part=0;!status&&part<(*header?2:1);part++) {
            FILE *stream=code;
            if(part){physical=final_gnu_physical(e,header_path,0);stream=fopen(physical,"rb");if(!stream)die("original Flex header macro stream");free(physical);}
            FILE *expanded=final_gnu_stream(),*filtered=final_gnu_stream();status=final_gnu_m4(e,&macro,stream,expanded);
            if(!status)status=final_gnu_filter(e,a->v[0],"--mantle-linedirs",state_path,expanded,filtered,NULL);
            if(!status){const char *target=part?header:output;if(*target)final_gnu_save_stream(e,target,filtered);else final_gnu_emit(filtered,e->out);}
            final_gnu_close(expanded);final_gnu_close(filtered);if(part)final_gnu_close(stream);
        }
        if(status)status=1; /* Original flex_main converts any filter child failure to 1. */
        native_gcc_args_free(&macro);final_gnu_close(code);free(output);free(header);
    }
    final_gnu_close(raw);final_gnu_close(errors);free(state_path);free(header_path);free(response_path);final_gnu_remove(e,root);return status;
}
static int final_gnu_external(Env *e,Args *a)
{
    int kind=final_gnu_capability(e,a);if(kind==1)return final_gnu_bison(e,a);if(kind==2)return final_gnu_flex(e,a);if(kind==3)return mantle_native_process_controller_external(e,a);return -1;
}
static int final_gnu_m4_prepare(Args *a,mantle_controller_builder *b)
{
    int options=1;
    for(int i=1;i<a->n;i++) {
        const char *s=a->v[i];if(options&&!strcmp(s,"--")){mantle_controller_argument_literal(b,s);options=0;continue;}
        if(options&&mantle_controller_oneof(s,"-I --include -F --freeze-state -R --reload-state --debugfile --error-output")) {
            int writing=mantle_controller_oneof(s,"-F --freeze-state --debugfile --error-output");mantle_controller_argument_literal(b,s);
            if(++i==a->n)die("original m4 missing file operand");mantle_controller_argument_path(b,a->v[i],writing);continue;
        }
        if(options&&s[0]=='-'&&s[1]) {
            if(strchr("IFR",s[1])&&s[2]){char flag[3]={'-',s[1],0};mantle_controller_option_path(b,s,flag,s[1]=='F');continue;}
            static const char *const paths[]={"--include=","--freeze-state=","--reload-state=","--debugfile=","--error-output="};
            int handled=0;for(size_t p=0;p<sizeof paths/sizeof paths[0];p++)if(!strncmp(s,paths[p],strlen(paths[p]))){mantle_controller_option_path(b,s,paths[p],p==1||p>=3);handled=1;break;}if(handled)continue;
            if(mantle_controller_oneof(s,"-D --define -U --undefine -t --trace --arglength --nesting-limit --word-regexp")){mantle_controller_argument_literal(b,s);if(++i==a->n)die("original m4 missing option data");mantle_controller_argument_data(b,a->v[i]);continue;}
            mantle_controller_argument_data(b,s);continue;
        }
        if(!strcmp(s,"-"))mantle_controller_argument_literal(b,s);else mantle_controller_argument_path(b,s,0);
    }
    return 1;
}
static int final_gnu_prepare(const char *slot,Request *request,mantle_controller_builder *b)
{
    (void)slot;Args *a=&request->command;if(!a->n)return 0;const char *name=mantle_controller_basename(a->v[0]);
    int bison=!strcmp(name,"bison"),flex=mantle_controller_oneof(name,"flex lex flex++"),m4=!strcmp(name,"m4");
    if(!bison&&!flex&&!m4)return final_gnu_generated_prepare(a,b);
    if(m4)return final_gnu_m4_prepare(a,b);
    if(flex&&a->n==3&&mantle_controller_oneof(a->v[1],"--mantle-tee --mantle-linedirs")){mantle_controller_argument_literal(b,a->v[1]);mantle_controller_argument_path(b,a->v[2],0);return 1;}
    int options=1;
    for(int i=1;i<a->n;i++) {
        const char *s=a->v[i];if(options&&!strcmp(s,"--")){mantle_controller_argument_literal(b,s);options=0;continue;}
        if(options&&mantle_controller_oneof(s,"-o --output --outfile --defines --header-file --tables-file -I --include -S --skeleton --skel")) {
            int writing=mantle_controller_oneof(s,"-o --output --outfile --defines --header-file --tables-file");mantle_controller_argument_literal(b,s);
            if(++i==a->n)die("original generator missing file operand");mantle_controller_argument_path(b,a->v[i],writing);continue;
        }
        if(options&&s[0]=='-'&&s[1]) {
            if(strchr("oIS",s[1])&&s[2]){char flag[3]={'-',s[1],0};mantle_controller_option_path(b,s,flag,s[1]=='o');continue;}
            static const char *const paths[]={"--output=","--outfile=","--defines=","--header-file=","--tables-file=","--include=","--skeleton=","--skel="};
            int handled=0;for(size_t p=0;p<sizeof paths/sizeof paths[0];p++)if(!strncmp(s,paths[p],strlen(paths[p]))){mantle_controller_option_path(b,s,paths[p],p<5);handled=1;break;}if(handled)continue;
            if(mantle_controller_oneof(s,"-D -U -b --file-prefix -p --name-prefix")||(flex&&mantle_controller_oneof(s,"-P --prefix"))){mantle_controller_argument_literal(b,s);if(++i==a->n)die("original generator missing option data");mantle_controller_argument_data(b,a->v[i]);continue;}
            mantle_controller_argument_data(b,s);continue;
        }
        if(!strcmp(s,"-"))mantle_controller_argument_literal(b,s);else mantle_controller_argument_path(b,s,0);
    }
    return 1;
}
static char *final_gnu_environment(mantle_controller_builder *b,const char *key,const char *value)
{
    if(!mantle_controller_oneof(key,"M4 BISON_PKGDATADIR AC_MACRODIR AUTOM4TE AUTOCONF autom4te_perllibdir MANTLE_PHASE_CWD MANTLE_BISON_RESPONSE MANTLE_BISON_IMAGE MANTLE_BISON_RESULT MANTLE_BISON_STATUS MANTLE_BISON_STDERR MANTLE_FLEX_STATE MANTLE_FLEX_HEADER MANTLE_FLEX_RESPONSE"))return NULL;
    int writing=mantle_controller_oneof(key,"MANTLE_PHASE_CWD MANTLE_FLEX_STATE MANTLE_FLEX_HEADER MANTLE_FLEX_RESPONSE");
    if(mantle_controller_oneof(key,"MANTLE_BISON_RESPONSE MANTLE_BISON_IMAGE")){const char *mode=get(&b->request->env,"MANTLE_BISON_MODE");writing=mode&&!strcmp(mode,"capture");}
    mantle_controller_word_begin(b);if(*value)mantle_controller_path(b,value,writing);else mantle_controller_literal(b,"");return mantle_controller_take_word(b);
}
