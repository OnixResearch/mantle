/* Original CMake evaluator continuation, composed with the shared source shell.
 * Only mantle_shell_external crosses a native request boundary. This evaluator
 * never spawns a process and never supplies an authored compiler/probe result. */
static unsigned long llvm_cmake_ordinal;
static char *llvm_runtime_path(const char *text) {
    const char *old="/outputs/workspace";size_t n=strlen(old);Buf b={0};
    for(const char *p=text;*p;){
        if(!strncmp(p,old,n)&&(!p[n]||p[n]=='/'||p[n]==' '||p[n]=='\"'||p[n]=='\'')){
            const char *relative=p+n;if(*relative=='/')++relative;size_t used=0;const char *logical=NULL;
            for(size_t i=0;i<mantle_paths_count;++i){struct mantle_paths_root *root=&mantle_paths_roots[i];size_t k=strlen(root->target);if(root->kind==MANTLE_PATHS_WORKSPACE&&!strncmp(relative,root->target,k)&&(!relative[k]||relative[k]=='/'||relative[k]==' '||relative[k]=='\"'||relative[k]=='\'')){logical=root->logical;used=k;break;}}
            if(logical){putsb(&b,logical);p=relative+used;}else{putsb(&b,workspace_root);p+=n;}
        }else putn(&b,p++,1);
    }
    return done(&b);
}
static uint32_t llvm_u32(FILE *f) {
    unsigned char b[4];
    if(fread(b,1,4,f)!=4)die("truncated original CMake request integer");
    return (uint32_t)b[0]|(uint32_t)b[1]<<8|(uint32_t)b[2]<<16|(uint32_t)b[3]<<24;
}
static char *llvm_bytes(FILE *f) {
    uint32_t n=llvm_u32(f);if(n>64UL*1024*1024)die("original CMake request string bound");
    char *s=alloc((size_t)n+1);if(fread(s,1,n,f)!=n||memchr(s,0,n))die("invalid original CMake request string");
    s[n]=0;char *relocated=llvm_runtime_path(s);free(s);return relocated;
}
static Args llvm_vector(FILE *f) {
    uint32_t n=llvm_u32(f);if(n>65536)die("original CMake request vector bound");
    Args values={0};while(n--)arg(&values,llvm_bytes(f));return values;
}
static void llvm_put32(FILE *f, uint32_t value) {
    unsigned char b[4]; for (unsigned i=0;i<4;++i)b[i]=(unsigned char)(value>>(8*i));
    if(fwrite(b,1,4,f)!=4)die("CMake observation integer write");
}
static void llvm_putbytes(FILE *f, const void *bytes, size_t n) {
    if(n>64UL*1024*1024)die("CMake observation record bound");
    llvm_put32(f,(uint32_t)n);if(fwrite(bytes,1,n,f)!=n)die("CMake observation bytes write");
}
static void llvm_putstring(FILE *f,const char *s){llvm_putbytes(f,s,strlen(s));}
static void llvm_putstream(FILE *f,FILE *stream){
    if(fflush(stream)||fseek(stream,0,SEEK_END))die("CMake stream extent");
    long n=ftell(stream);if(n<0||n>64L*1024*1024||fseek(stream,0,SEEK_SET))die("CMake stream bound");
    llvm_put32(f,(uint32_t)n);transfer(stream,f);
}
static void llvm_export(Env *e,const char *key,const char *value){char *owned=NULL;size_t n=strlen(workspace_root);if(!strncmp(value,workspace_root,n)&&(!value[n]||value[n]=='/')){Buf b={0};putsb(&b,"/outputs/workspace");putsb(&b,value+n);owned=done(&b);}set(e,key,owned?owned:value);variable(e,key)->exported=1;free(owned);}
static void llvm_private_streams(Env *e) {
    memset(e->fd,0,sizeof e->fd);memset(e->fd_flags,0,sizeof e->fd_flags);memset(e->fd_cloexec,0,sizeof e->fd_cloexec);
    for(int i=0;i<3;++i){e->fd[i]=tmpfile();if(!e->fd[i])die("CMake observation stream");e->fd_flags[i]=i?O_WRONLY:O_RDONLY;}
    e->in=e->fd[0];e->out=e->fd[1];e->err=e->fd[2];
}
static void llvm_close_streams(Env *e){for(int i=0;i<3;++i)if(fclose(e->fd[i]))die("CMake observation stream close");}
static Args llvm_names(const char *path){
    Args names={0};DIR *d=opendir(path);if(!d)die("CMake effect directory %s",path);struct dirent *entry;
    while((entry=readdir(d)))if(strcmp(entry->d_name,".")&&strcmp(entry->d_name,".."))arg(&names,dup(entry->d_name));
    if(closedir(d))die("CMake effect directory close");
    for(int i=1;i<names.n;++i){char *p=names.v[i];int j=i;while(j&&strcmp(names.v[j-1],p)>0){names.v[j]=names.v[j-1];--j;}names.v[j]=p;}
    return names;
}
static void llvm_free_args(Args *a){for(int i=0;i<a->n;++i)free(a->v[i]);free(a->v);memset(a,0,sizeof *a);}
static void llvm_project_copy(const char *from,const char *to){
    mkdir_checked(to);Args names=llvm_names(from);
    for(int i=0;i<names.n;++i)if(strcmp(names.v[i],".mantle-llvm")){char *a=join(from,names.v[i]),*b=join(to,names.v[i]);copy_tree(a,b);free(a);free(b);}
    llvm_free_args(&names);
}
static void llvm_project_restore(const char *from,const char *to){
    Args names=llvm_names(to);
    for(int i=0;i<names.n;++i)if(strcmp(names.v[i],".mantle-llvm")){char *a=join(to,names.v[i]);remove_tree(a);free(a);}
    llvm_free_args(&names);llvm_project_copy(from,to);
}
static char *llvm_link(const char *path,const struct stat *st){
    size_t n=st->st_size>0?(size_t)st->st_size+1:4096;char *p=alloc(n+1);ssize_t got=readlink(path,p,n);
    if(got<0||(size_t)got==n)die("CMake original link bound");p[got]=0;return p;
}
struct LlvmEffects {FILE *records;const char *history;unsigned ordinal;uint32_t count;};
static void llvm_effect(struct LlvmEffects *effects,unsigned tag,const char *relative,const char *actual,const struct stat *st){
    if(++effects->count>65536)die("CMake effect cardinality bound");
    llvm_put32(effects->records,tag);llvm_putstring(effects->records,relative);
    if(tag==1){
        char name[80];snprintf(name,sizeof name,"effects/%u/%u",effects->ordinal,effects->count);
        char *target=join(effects->history,name);char *parent=dup(target),*slash=strrchr(parent,'/');if(!slash)die("CMake effect parent");*slash=0;
        mkdir_checked(parent);binary_copy(actual,target);free(target);free(parent);llvm_putstring(effects->records,name);
    }else if(tag==3){char *target=llvm_link(actual,st);llvm_putstring(effects->records,target);free(target);}
    else llvm_putstring(effects->records,"");
    llvm_put32(effects->records,st?(uint32_t)(st->st_mode&0777):0);
    uint64_t seconds=st?(uint64_t)st->st_mtim.tv_sec:0;
    llvm_put32(effects->records,(uint32_t)seconds);llvm_put32(effects->records,(uint32_t)(seconds>>32));
    llvm_put32(effects->records,st?(uint32_t)st->st_mtim.tv_nsec:0);
}
static void llvm_delete_effects(struct LlvmEffects *effects,const char *old,const char *relative){
    struct stat st;if(lstat(old,&st))die("CMake deletion observation");
    if(S_ISDIR(st.st_mode)){Args names=llvm_names(old);for(int i=0;i<names.n;++i){char *p=join(old,names.v[i]),*r=join(relative,names.v[i]);llvm_delete_effects(effects,p,r);free(p);free(r);}llvm_free_args(&names);}
    llvm_effect(effects,0,relative,NULL,NULL);
}
static void llvm_effects(struct LlvmEffects *effects,const char *before,const char *after,const char *relative){
    struct stat a,b;int have_a=!lstat(before,&a);
    if(!have_a&&errno!=ENOENT)die("CMake before effect stat");
    int have_b=!lstat(after,&b);if(!have_b&&errno!=ENOENT)die("CMake after effect stat");
    if(!have_b){if(have_a&&*relative)llvm_delete_effects(effects,before,relative);return;}
    if(have_a&&((a.st_mode&S_IFMT)!=(b.st_mode&S_IFMT))){if(*relative)llvm_delete_effects(effects,before,relative);have_a=0;}
    if(S_ISDIR(b.st_mode)){
        int changed=!have_a||(a.st_mode&0777)!=(b.st_mode&0777)||a.st_mtim.tv_sec!=b.st_mtim.tv_sec||a.st_mtim.tv_nsec!=b.st_mtim.tv_nsec;
        if(!have_a&&*relative)llvm_effect(effects,2,relative,after,&b);
        Args names=llvm_names(after);for(int i=0;i<names.n;++i){if(!*relative&&!strcmp(names.v[i],".mantle-llvm"))continue;char *p=join(before,names.v[i]),*q=join(after,names.v[i]),*r=*relative?join(relative,names.v[i]):dup(names.v[i]);llvm_effects(effects,p,q,r);free(p);free(q);free(r);}llvm_free_args(&names);
        if(have_a){names=llvm_names(before);for(int i=0;i<names.n;++i){if(!*relative&&!strcmp(names.v[i],".mantle-llvm"))continue;char *p=join(before,names.v[i]),*q=join(after,names.v[i]);struct stat st;if(lstat(q,&st)){if(errno!=ENOENT)die("CMake deleted member stat");char *r=*relative?join(relative,names.v[i]):dup(names.v[i]);llvm_delete_effects(effects,p,r);free(r);}free(p);free(q);}llvm_free_args(&names);}
        if(changed&&*relative)llvm_effect(effects,2,relative,after,&b);
    }else if(S_ISREG(b.st_mode)){
        if(!have_a||a.st_size!=b.st_size||(a.st_mode&0777)!=(b.st_mode&0777)||a.st_mtim.tv_sec!=b.st_mtim.tv_sec||a.st_mtim.tv_nsec!=b.st_mtim.tv_nsec||!files_equal(before,after))llvm_effect(effects,1,relative,after,&b);
    }else if(S_ISLNK(b.st_mode)){
        char *right=llvm_link(after,&b),*left=have_a?llvm_link(before,&a):NULL;
        if(!left||strcmp(left,right)||a.st_mtim.tv_sec!=b.st_mtim.tv_sec||a.st_mtim.tv_nsec!=b.st_mtim.tv_nsec)llvm_effect(effects,3,relative,after,&b);free(left);free(right);
    }else die("CMake produced non-file filesystem effect");
}
static void llvm_pending_environment(FILE *frame,Env *e){
    clearvars(e);e->vars=NULL;uint32_t n=llvm_u32(frame);if(n>65536)die("CMake original environment bound");
    while(n--){char *entry=llvm_bytes(frame),*equal=strchr(entry,'=');if(!equal||equal==entry)die("CMake original environment entry");*equal=0;llvm_export(e,entry,equal+1);free(entry);}
}
static void llvm_frame_copy(FILE *from,FILE *to,size_t n){char b[65536];while(n){size_t want=n<sizeof b?n:sizeof b;if(fread(b,1,want,from)!=want||fwrite(b,1,want,to)!=want)die("CMake observed bytes copy");n-=want;}}
static void llvm_acknowledge(Env *outer,const char *pending,const char *phase,const char *history,const char *work,unsigned ordinal){
    FILE *f=fopen(pending,"rb");char magic[8];if(!f||fread(magic,1,8,f)!=8||memcmp(magic,"MCCMQ01\n",8)||llvm_u32(f)!=ordinal)die("CMake pending request ordinal/version");
    uint32_t extent=llvm_u32(f);long begin=ftell(f);if(begin<0||extent>64UL*1024*1024)die("CMake request extent");
    char *kind=llvm_bytes(f),*directory=llvm_bytes(f);uint32_t count=llvm_u32(f);if(!count||count>64)die("CMake original process chain bound");
    Args *commands=alloc(count*sizeof *commands);for(uint32_t i=0;i<count;++i)commands[i]=llvm_vector(f);
    uint32_t input_length=llvm_u32(f);FILE *input=tmpfile();if(!input)die("CMake original stdin");llvm_frame_copy(f,input,input_length);rewind(input);
    Env e=clone(outer);llvm_pending_environment(f,&e);free(e.cwd);e.cwd=directory;
    if(ftell(f)-begin!=extent||fgetc(f)!=EOF)die("CMake original request trailing bytes");
    char *base=join(phase,"effect-base");remove_tree(base);llvm_project_copy(work,base);
    FILE *out=tmpfile(),*err=tmpfile();if(!out||!err)die("CMake original process streams");
    int merged=strstr(kind,":merged")!=NULL;uint32_t *statuses=alloc(count*sizeof *statuses);uint32_t input_consumed=0;
    for(uint32_t i=0;i<count;++i){
        FILE *next=tmpfile();if(!next)die("CMake pipeline stream");
        memset(e.fd,0,sizeof e.fd);memset(e.fd_flags,0,sizeof e.fd_flags);memset(e.fd_cloexec,0,sizeof e.fd_cloexec);e.fd[0]=input;e.fd[1]=next;e.fd[2]=merged?next:err;e.fd_flags[0]=O_RDONLY;e.fd_flags[1]=e.fd_flags[2]=O_WRONLY;e.in=input;e.out=next;e.err=e.fd[2];
        if(mantle_paths_chdir(e.cwd))die("CMake original command cwd");int status=external(&e,&commands[i]);if(status<0||status>255)die("CMake command has no normal native status");statuses[i]=(uint32_t)status;
        if(!i){long consumed=ftell(input);if(consumed<0||(uint64_t)consumed>input_length)die("CMake actual child stdin position");input_consumed=(uint32_t)consumed;}
        if(fflush(next)||fseek(next,0,SEEK_SET))die("CMake pipeline rewind");if(fclose(input))die("CMake pipeline close");input=next;
    }
    transfer(input,out);if(fclose(input))die("CMake final pipeline close");
    FILE *effects_file=tmpfile();if(!effects_file)die("CMake effect records");struct LlvmEffects effects={effects_file,history,ordinal,0};
    char *effect_root=join(history,"effects");mkdir_checked(effect_root);free(effect_root);char number[32];snprintf(number,sizeof number,"%u",ordinal);char *blobroot=join(history,"effects"),*blobs=join(blobroot,number);mkdir_checked(blobs);free(blobroot);free(blobs);
    llvm_effects(&effects,base,work,"");
    char *path=join(history,number);FILE *record_file=fopen(path,"wb");if(!record_file||fwrite("MCCMR03\n",1,8,record_file)!=8)die("CMake actual observation record");
    if(fseek(f,begin,SEEK_SET))die("CMake original request rewind");llvm_put32(record_file,extent);llvm_frame_copy(f,record_file,extent);llvm_put32(record_file,count);
    for(uint32_t i=0;i<count;++i)llvm_put32(record_file,statuses[i]);llvm_putstream(record_file,out);llvm_putstream(record_file,err);llvm_put32(record_file,input_consumed);llvm_put32(record_file,effects.count);rewind(effects_file);transfer(effects_file,record_file);
    if(fclose(record_file)||fclose(f)||fclose(out)||fclose(err)||fclose(effects_file))die("CMake observation close");
    for(uint32_t i=0;i<count;++i)llvm_free_args(&commands[i]);free(commands);free(statuses);free(kind);free(path);clearvars(&e);free(e.cwd);remove_tree(base);free(base);
    if(mantle_paths_chdir(outer->cwd))die("CMake parent cwd restore");
}
static int llvm_cmake(Env *outer,Args *command){
    char number[32];snprintf(number,sizeof number,"%lu",llvm_cmake_ordinal++);
    char *auxiliary=join(workspace_root,".mantle-llvm"),*call=join(auxiliary,number);mkdir_checked(auxiliary);mkdir_checked(call);
    const char *project=get(outer,"MANTLE_CMAKE_PROJECT");struct mantle_paths_operand project_path;char *work;if(project){if(mantle_paths_map(project,1,&project_path))die("CMake original project root");work=dup(project_path.physical);}else work=dup(workspace_root);char *history=join(call,"history"),*base=join(call,"base"),*phase=join(call,"phase");
    size_t root_length=strlen(workspace_root);if(strncmp(work,workspace_root,root_length)||(work[root_length]&&work[root_length]!='/'))die("CMake project is outside declared workspace");
    char *observations=join(call,"observations");mkdir_checked(observations);
    mkdir_checked(history);text_file(history,"format","MCCMH03\n");llvm_project_copy(work,base);
    unsigned ordinal=0;int status;
    for(;;){
        llvm_project_restore(base,work);remove_tree(phase);mkdir_checked(phase);
        Env e=clone(outer);llvm_private_streams(&e);
        long input_position=ftell(outer->in);if(input_position<0)die("CMake parent input position");
        transfer(outer->in,e.in);if(fseek(outer->in,input_position,SEEK_SET)||fseek(e.in,0,SEEK_SET))die("CMake original input rewind");
        llvm_export(&e,"MANTLE_CAIRN_CMAKE_PHASE",phase);llvm_export(&e,"MANTLE_CAIRN_CMAKE_HISTORY",history);llvm_export(&e,"MANTLE_CAIRN_CMAKE_WORKSPACE",work);
        llvm_export(&e,"MANTLE_CAIRN_CMAKE_OBSERVATIONS",observations);
        status=mantle_shell_external(&e,command);
        if(status!=75){rewind(e.out);transfer(e.out,outer->out);rewind(e.err);transfer(e.err,outer->err);llvm_close_streams(&e);clearvars(&e);free(e.cwd);break;}
        llvm_close_streams(&e);clearvars(&e);free(e.cwd);
        char *pending=join(phase,"request");llvm_acknowledge(outer,pending,phase,history,work,ordinal++);free(pending);
    }
    free(auxiliary);free(call);free(history);free(base);free(phase);free(work);free(observations);
    if(mantle_paths_chdir(outer->cwd))die("CMake outer cwd restore");return status;
}
