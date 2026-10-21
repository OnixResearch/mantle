/* Shared original Perl subprocess continuation. All commands cross the common
 * source callback, so a shell command remains source shell interpretation and
 * every actual generator/compiler/linker is still an independent Exec leaf. */
static int mpc_perl_name(const char *image)
{ return mantle_controller_oneof(mantle_controller_basename(image),"perl miniperl perl-phase"); }
static char *mpc_physical(Env *e,const char *path,int writing)
{
    struct mantle_paths_operand operand;char *logical;
    logical=*path=='/'?dup(path):join(e->cwd,path);
    if(mantle_paths_map(logical,writing,&operand))die("original Perl source path %s: %s",logical,strerror(errno));
    free(logical);return dup(operand.physical);
}
static char *mpc_logical(const char *path)
{
    const char *prefix="/outputs/workspace";size_t n=strlen(prefix);char *physical=NULL,logical[PATH_MAX];struct mantle_paths_operand operand;
    if(!strncmp(path,prefix,n)&&(path[n]=='/'||!path[n]))physical=path[n]?join(workspace_root,path+n+1):dup(workspace_root);
    if(!mantle_paths_logical(physical?physical:path,logical,sizeof logical)){free(physical);return dup(logical);}
    free(physical);if(mantle_paths_map(path,0,&operand))die("original Perl child cwd is outside exact logical roots: %s",path);return dup(operand.logical);
}
static void mpc_close(FILE *f)
{ if(ferror(f)||fclose(f))die("original Perl controller stream close"); }
static void mpc_rewind(FILE *f)
{ if(fflush(f)||fseek(f,0,SEEK_SET))die("original Perl controller stream rewind"); }
static FILE *mpc_stream(void)
{ FILE *f=tmpfile();if(!f)die("original Perl controller stream");return f; }
static void mpc_save_stream(const char *path,FILE *from)
{
    FILE *to=fopen(path,"wb");if(!to)die("original Perl reply stream %s",path);mpc_rewind(from);mp_copied=0;mp_transfer(from,to);mpc_close(to);
}
static void mpc_copy_file(const char *from,const char *to)
{
    FILE *f=fopen(from,"rb");if(!f)die("original Perl request copy");mpc_save_stream(to,f);mpc_close(f);
}
static void mpc_scalar(const char *root,const char *name,long value)
{
    char *path=join(root,name);FILE *f=fopen(path,"wb");if(!f||fprintf(f,"%ld\n",value)<0)die("original Perl actual reply scalar");mpc_close(f);free(path);
}
static long mpc_number(FILE *f,long limit)
{
    char *text=read_record(f),*end;long value;errno=0;value=strtol(text,&end,10);
    if(errno||end==text||*end||value<0||value>limit)die("original Perl request integer");free(text);return value;
}
static FILE *mpc_descriptor_stream(const char *path,long position,int flags,int null_device,const char *member)
{
    FILE *stream,*input;int fd;char proc[64];const char *mode=(flags&O_ACCMODE)==O_RDONLY?"rb":(flags&O_ACCMODE)==O_WRONLY?"wb":"r+b";
    if(null_device){
        fd=open("/dev/null",flags|O_CLOEXEC);if(fd<0)die("original Perl null descriptor");
        stream=fdopen(fd,mode);
        if(!stream)die("original Perl null descriptor stream");
    }else if(*member){
        fd=mantle_descriptor_open_workspace(workspace_root,member,flags);stream=fdopen(fd,mode);if(!stream||mp_null(fd))die("original Perl bound descriptor stream");
        snprintf(proc,sizeof proc,"/proc/self/fd/%d",fd);if(!files_equal(proc,path))die("original Perl descriptor workspace bytes differ");
    }else{
        stream=mpc_stream();input=fopen(path,"rb");if(!input)die("original Perl descriptor snapshot");
        mp_transfer(input,stream);mpc_close(input);
        fd=fcntl(fileno(stream),F_GETFL);if(fd<0||fcntl(fileno(stream),F_SETFL,(fd&~(O_APPEND|O_NONBLOCK))|(flags&(O_APPEND|O_NONBLOCK))))die("original Perl detached descriptor flags");
    }
    if(fseek(stream,position,SEEK_SET))die("original Perl descriptor initial offset");return stream;
}
static void mpc_environment_free(Env *environment)
{
    int i,j;for(i=0;i<MP_DESCRIPTORS;i++)if(environment->fd[i]){
        for(j=0;j<i;j++)if(environment->fd[j]==environment->fd[i])break;
        if(j==i)mpc_close(environment->fd[i]);
    }
    clearvars(environment);free(environment->cwd);
}
static Env mpc_phase_environment(Env *parent,char **members,int checkpoint)
{
    Env phase=clone(parent);char path[64];long position;int i,j;
    mp_copied=0;
    for(i=0;i<MP_DESCRIPTORS;i++)if(parent->fd[i]){
        for(j=0;j<i;j++)if(parent->fd[j]==parent->fd[i])break;
        if(j<i){phase.fd[i]=phase.fd[j];if(checkpoint)members[i]=members[j];continue;}
        if(fflush(parent->fd[i])||(position=ftell(parent->fd[i]))<0)die("original Perl parent OFD checkpoint");
        snprintf(path,sizeof path,"/proc/self/fd/%d",fileno(parent->fd[i]));
        if(checkpoint)members[i]=mantle_descriptor_workspace_fd(fileno(parent->fd[i]),workspace_root);
        phase.fd[i]=mpc_descriptor_stream(path,position,parent->fd_flags[i],mp_null(fileno(parent->fd[i])),checkpoint?"":members[i]);
    }
    phase.in=phase.fd[0];phase.out=phase.fd[1];phase.err=phase.fd[2];return phase;
}
static void mpc_commit_phase(Env *parent,Env *phase)
{
    char path[64];long position;int i,j;
    for(i=0;i<MP_DESCRIPTORS;i++)if(parent->fd[i]){
        for(j=0;j<i;j++)if(parent->fd[j]==parent->fd[i])break;if(j<i)continue;
        if(!phase->fd[i]||fflush(phase->fd[i])||fflush(parent->fd[i])||(position=ftell(phase->fd[i]))<0)die("original Perl final OFD checkpoint");
        snprintf(path,sizeof path,"/proc/self/fd/%d",fileno(phase->fd[i]));
        mp_restore_descriptor(fileno(parent->fd[i]),phase->fd_flags[i],position,mp_null(fileno(phase->fd[i])),path);
        if(fseek(parent->fd[i],position,SEEK_SET))die("original Perl final descriptor cursor");
        for(j=i;j<MP_DESCRIPTORS;j++)if(parent->fd[j]==parent->fd[i])parent->fd_flags[j]=phase->fd_flags[i];
    }
}
struct mpc_request {
    Env env;Args command;char *operation,*original_zero;mp_descriptors descriptors;
    FILE *inherited[MP_DESCRIPTORS];int payload;
};
static void mpc_read_request(Env *parent,const char *path,const char *root,struct mpc_request *request)
{
    FILE *f=fopen(path,"rb");char *magic,*image,*cwd,*entry,*equal,*snapshot;long count;int i,alias;size_t j;
    if(!f)die("original Perl child request");memset(request,0,sizeof *request);request->env.zero=parent->zero;
    magic=read_record(f);if(strcmp(magic,"MANTLE-NATIVE-PERL-CHILD-2"))die("original Perl request version");free(magic);
    request->operation=read_record(f);if(!mantle_controller_oneof(request->operation,"system readpipe writepipe exec"))die("original Perl request operation");
    image=read_record(f);count=mpc_number(f,16384);if(!count)die("original Perl empty actual argv");
    for(i=0;i<count;i++){entry=read_record(f);if(strlen(entry)>65536)die("original Perl argument bound");arg(&request->command,entry);}
    request->original_zero=request->command.v[0];request->command.v[0]=image;request->command.argv0=request->original_zero;
    cwd=read_record(f);request->env.cwd=mpc_logical(cwd);free(cwd);
    request->env.creation_mask=(mode_t)mpc_number(f,0777);request->env.mask_valid=1;
    count=mpc_number(f,65536);
    for(i=0;i<count;i++){
        entry=read_record(f);equal=strchr(entry,'=');if(!equal||equal==entry)die("original Perl environment record");*equal=0;
        for(j=0;entry[j];j++)if(!((entry[j]>='a'&&entry[j]<='z')||(entry[j]>='A'&&entry[j]<='Z')||(entry[j]>='0'&&entry[j]<='9')||entry[j]=='_'))die("original Perl environment key");
        if(variable(&request->env,entry))die("duplicate actual Perl child environment key");set(&request->env,entry,equal+1);variable(&request->env,entry)->exported=1;free(entry);
    }
    mp_copied=0;
    for(i=0;i<MP_DESCRIPTORS;i++){
        alias=(int)mp_read_number(f,-1,i);request->descriptors.aliases[i]=alias;if(alias<0)continue;
        if(alias<i){
            if(request->descriptors.aliases[alias]!=alias||!request->env.fd[alias])die("original Perl descriptor alias primary");
            request->env.fd[i]=request->env.fd[alias];request->descriptors.flags[i]=request->descriptors.flags[alias];
            request->descriptors.positions[i]=request->descriptors.positions[alias];request->descriptors.nulls[i]=request->descriptors.nulls[alias];
            request->descriptors.members[i]=request->descriptors.members[alias];
        }else{
            request->descriptors.positions[i]=mp_read_number(f,0,LONG_MAX);request->descriptors.flags[i]=(int)mp_read_number(f,0,O_ACCMODE|O_APPEND|O_NONBLOCK);
            if((request->descriptors.flags[i]&~(O_ACCMODE|O_APPEND|O_NONBLOCK))||(request->descriptors.flags[i]&O_ACCMODE)==O_ACCMODE)die("original Perl descriptor status flags");
            request->descriptors.nulls[i]=(int)mp_read_number(f,0,1);request->descriptors.members[i]=mp_read_member(f);
            if(request->descriptors.nulls[i]&&*request->descriptors.members[i])die("original Perl null workspace binding");snapshot=mp_descriptor_path(root,"",i);
            request->env.fd[i]=mpc_descriptor_stream(snapshot,request->descriptors.positions[i],request->descriptors.flags[i],request->descriptors.nulls[i],request->descriptors.members[i]);free(snapshot);
        }
        request->env.fd_flags[i]=request->descriptors.flags[i];request->descriptors.cloexec[i]=(int)mp_read_number(f,0,1);
        request->env.fd_cloexec[i]=request->descriptors.cloexec[i];request->inherited[i]=request->env.fd[i];
    }
    request->payload=(int)mpc_number(f,1);if(request->payload!=!strcmp(request->operation,"writepipe"))die("original Perl pipe payload binding");
    if(fgetc(f)!=EOF)die("trailing original Perl child request");mpc_close(f);
    for(i=0;i<MP_DESCRIPTORS;i++)if(request->env.fd_cloexec[i])request->env.fd[i]=NULL;
    request->env.in=request->env.fd[0];request->env.out=request->env.fd[1];request->env.err=request->env.fd[2];
}
static void mpc_request_free(struct mpc_request *request)
{
    int i,j;for(i=0;i<MP_DESCRIPTORS;i++)if(request->inherited[i]){
        for(j=0;j<i;j++)if(request->inherited[j]==request->inherited[i])break;
        if(j==i)mpc_close(request->inherited[i]);
    }
    for(i=0;i<MP_DESCRIPTORS;i++)if(request->descriptors.aliases[i]==i)free(request->descriptors.members[i]);
    clearvars(&request->env);free(request->env.cwd);native_gcc_args_free(&request->command);free(request->original_zero);free(request->operation);
}
static void mpc_save_descriptors(const char *root,const char *reply,struct mpc_request *request)
{
    mp_descriptors result=request->descriptors;char *from,*to;FILE *table;long position;int i,j;
    mp_copied=0;
    for(i=0;i<MP_DESCRIPTORS;i++)if(result.aliases[i]==i){
        if(fflush(request->inherited[i])||(position=ftell(request->inherited[i]))<0)die("original Perl child OFD completion");result.positions[i]=position;
        for(j=0;j<MP_DESCRIPTORS;j++)if(request->env.fd[j]==request->inherited[i]&&!request->env.fd_cloexec[j]){result.flags[i]=request->env.fd_flags[j];break;}
        if(!result.nulls[i]){
            from=mp_descriptor_path(root,"",i);to=mp_descriptor_path(reply,"input-",i);mpc_copy_file(from,to);free(from);free(to);
            to=mp_descriptor_path(reply,"",i);mp_snapshot(fileno(request->inherited[i]),to);free(to);
        }
    }
    to=join(reply,"descriptors");table=fopen(to,"wb");if(!table)die("original Perl full descriptor reply");mp_descriptor_rows(table,&result);mpc_close(table);free(to);
}
static int mantle_native_process_controller_external(Env *e,Args *a)
{
    const char *base="/tmp/.mantle-perl-children";char ordinal[64],*root,*physical,*baseline,*replies,*request_path,*stdin_path,*canonical_root,*reply,*path;
    FILE *pipe_stream;Env initial,phase;char *members[MP_DESCRIPTORS];struct mpc_request request;struct stat request_stat;int status,child_status,slot,prior;unsigned long index;
    if(!a->n)return -1;
    physical=mpc_physical(e,base,1);mkdir_checked(physical);canonical_root=realpath(physical,NULL);if(!canonical_root)die("original native process private root");free(physical);
    snprintf(ordinal,sizeof ordinal,"%lu",request_ordinal);root=join(base,ordinal);
    physical=mpc_physical(e,root,1);remove_tree(physical);mkdir_checked(physical);
    baseline=join(physical,"baseline");replies=join(root,"replies");path=join(physical,"replies");mkdir_checked(path);free(path);
    memset(members,0,sizeof members);initial=mpc_phase_environment(e,members,1);
    request_path=join(physical,"request");stdin_path=join(physical,"stdin");status=-1;
    for(index=0;index<1000000;index++){
        if(index)mp_copy_workspace(baseline,workspace_root,canonical_root,1);
        if(unlink(request_path)&&errno!=ENOENT)die("original Perl previous request cleanup");
        if(unlink(stdin_path)&&errno!=ENOENT)die("original Perl previous stdin cleanup");
        phase=mpc_phase_environment(&initial,members,0);
        native_gcc_export(&phase,"MANTLE_PERL_PHASE_ROOT",root);native_gcc_export(&phase,"MANTLE_PERL_PHASE_REPLIES",replies);
        native_gcc_export(&phase,"MANTLE_PERL_PHASE_WORKSPACE","");status=mantle_shell_external(&phase,a);
        if(stat(request_path,&request_stat)){
            if(errno!=ENOENT)die("original Perl fresh request lookup");mpc_commit_phase(e,&phase);mpc_environment_free(&phase);break;
        }
        mpc_environment_free(&phase);if(status!=86)die("original Perl child request lacks its actual capture exit");
        if(!index){
            /* The common boundary already retained the exact pre-Perl tree.
             * Adopt that immutable copy only when a real child was observed. */
            path=join(output_root,"request/workspace");
            if(!mp_inside(canonical_root,workspace_root)||canonical_root[strlen(workspace_root)]!='/')die("native private root is outside its workspace");
            reply=join(path,canonical_root+strlen(workspace_root)+1);mp_remove(reply,NULL);free(reply);
            if(rename(path,baseline))die("original native process precommand workspace snapshot");free(path);
        }
        mpc_read_request(e,request_path,physical,&request);pipe_stream=NULL;
        if(request.payload){
            pipe_stream=fopen(stdin_path,"rb");if(!pipe_stream)die("original Perl write-pipe payload");request.env.in=request.env.fd[0]=pipe_stream;
            request.env.fd_flags[0]=O_RDONLY;request.env.fd_cloexec[0]=0;
        }else if(!strcmp(request.operation,"readpipe")){
            pipe_stream=mpc_stream();request.env.out=request.env.fd[1]=pipe_stream;request.env.fd_flags[1]=O_WRONLY;request.env.fd_cloexec[1]=0;
        }
        child_status=mantle_source_external_hook?mantle_source_external_hook(&request.env,&request.command):mantle_native_external(&request.env,&request.command);
        if(child_status<0||child_status>255)die("original Perl child did not publish a normal exit status");
        snprintf(ordinal,sizeof ordinal,"%lu",index);path=mpc_physical(e,replies,1);reply=join(path,ordinal);free(path);mkdir_checked(reply);
        path=join(reply,"request");mpc_copy_file(request_path,path);free(path);path=join(reply,"workspace");mp_copy_workspace(workspace_root,path,canonical_root,0);free(path);
        mpc_save_descriptors(physical,reply,&request);
        if(!strcmp(request.operation,"readpipe")){path=join(reply,"pipe-stdout");mpc_save_stream(path,pipe_stream);free(path);}
        mpc_scalar(reply,"status",child_status);free(reply);if(pipe_stream)mpc_close(pipe_stream);mpc_request_free(&request);
    }
    if(index==1000000)die("original Perl child continuation bound");
    for(slot=0;slot<MP_DESCRIPTORS;slot++)if(initial.fd[slot]){for(prior=0;prior<slot;prior++)if(initial.fd[prior]==initial.fd[slot])break;if(prior==slot)free(members[slot]);}
    mpc_environment_free(&initial);
    remove_tree(physical);free(physical);free(canonical_root);free(root);free(baseline);free(replies);free(request_path);free(stdin_path);return status;
}
static int mantle_perl_controller_external(Env *e,Args *a)
{
    if(!a->n||!mpc_perl_name(a->v[0]))return -1;
    return mantle_native_process_controller_external(e,a);
}
static int mantle_perl_controller_prepare(const char *slot,Request *request,mantle_controller_builder *b)
{
    Args *a=&request->command;int script=0,evaluate=0,search=0,options=1;int i;const char *s;(void)slot;
    if(!a->n||!mpc_perl_name(a->v[0]))return 0;
    for(i=1;i<a->n;i++){
        s=a->v[i];if(script){mantle_controller_argument_data(b,s);continue;}
        if(options&&!strcmp(s,"--")){mantle_controller_argument_literal(b,s);options=0;continue;}
        if(options&&(!strcmp(s,"-e")||!strcmp(s,"-E"))){mantle_controller_argument_literal(b,s);if(++i==a->n)break;mantle_controller_argument_data(b,a->v[i]);evaluate=1;continue;}
        if(options&&!strcmp(s,"-I")){mantle_controller_argument_literal(b,s);if(++i==a->n)break;mantle_controller_argument_path(b,a->v[i],0);continue;}
        if(options&&!strncmp(s,"-I",2)&&s[2]){mantle_controller_option_path(b,s,"-I",0);continue;}
        if(options&&!strcmp(s,"-S")){search=1;mantle_controller_argument_literal(b,s);continue;}
        if(options&&s[0]=='-'&&s[1]){if(!strncmp(s,"-e",2)||!strncmp(s,"-E",2))evaluate=1;mantle_controller_argument_data(b,s);continue;}
        if(evaluate||search||!strcmp(s,"-"))mantle_controller_argument_data(b,s);else mantle_controller_argument_path(b,s,0);script=1;
    }
    return 1;
}
static char *mantle_perl_controller_environment(mantle_controller_builder *b,const char *key,const char *value)
{
    if(!strcmp(key,"MANTLE_PERL_PHASE_WORKSPACE"))return mantle_controller_named_word(b,1,"workspace","");
    if(mantle_controller_oneof(key,"MANTLE_PERL_PHASE_ROOT MANTLE_PERL_PHASE_REPLIES")){
        mantle_controller_word_begin(b);mantle_controller_path(b,value,1);return mantle_controller_take_word(b);
    }
    if(mantle_controller_oneof(key,"PERL5LIB PERLLIB")){
        const char *p=value,*end;mantle_controller_word_begin(b);
        for(;;){end=strchr(p,':');if(end==p||(!end&&!*p))mantle_controller_literal(b,"");else{char *path=end?slice(p,(size_t)(end-p)):dup(p);mantle_controller_path(b,path,0);free(path);}if(!end)break;mantle_controller_literal(b,":");p=end+1;}
        return mantle_controller_take_word(b);
    }
    return NULL;
}
