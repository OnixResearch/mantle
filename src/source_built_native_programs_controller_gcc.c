/* Composed after the shared source shell and controller protocol. GCC itself
 * selects every compiler/assembler/collect2 argument through its original
 * -### path. This code neither translates driver flags nor launches a process. */
static const char *native_gcc_tool_name(const char *image)
{
    const char *name=mantle_controller_basename(image);
    const char *const targets[]={"x86_64-linux-musl-","x86_64-unknown-linux-musl-"};
    for(size_t i=0;i<sizeof targets/sizeof targets[0];i++){size_t n=strlen(targets[i]);if(!strncmp(name,targets[i],n))return name+n;}
    return name;
}
static int native_gcc_driver_name(const char *name)
{
    size_t n=strlen(name);
    return mantle_controller_oneof(name,"gcc g++ cc c++ cpp xgcc xg++ gcc.real g++.real cpp.real gcc-seed gxx-seed") ||
        (n>4 && (!strcmp(name+n-4,"-gcc") || !strcmp(name+n-4,"-g++")));
}
static void native_gcc_export(Env *e,const char *key,const char *value)
{
    Var *v=variable(e,key);
    /* Derived child environment is not a shell assignment to a readonly var. */
    if(v)v->readonly=0;
    set(e,key,value);variable(e,key)->exported=1;
}
/* Offset/content already share the FILE-backed original OFD. Status flags are
 * logical metadata and share with aliases too; descriptor CLOEXEC does not. */
static void native_gcc_descriptor_flags(Env *parent,const Env *child)
{
    for(int i=0;i<64;i++)if(parent->fd[i])for(int j=0;j<64;j++)if(parent->fd[i]==child->fd[j]){
        parent->fd_flags[i]=(parent->fd_flags[i]&O_ACCMODE)|(child->fd_flags[j]&(O_APPEND|O_NONBLOCK));break;
    }
}
static char *native_gcc_stream(FILE *f)
{
    if(fflush(f)||fseek(f,0,SEEK_END))die("native GCC log seek");
    long n=ftell(f);if(n<0 || n>16L*1024*1024)die("native GCC actual log bound");
    if(fseek(f,0,SEEK_SET))die("native GCC log rewind");
    char *text=alloc((size_t)n+1);
    if(fread(text,1,(size_t)n,f)!=(size_t)n||ferror(f)||memchr(text,0,(size_t)n))die("native GCC actual log read");
    text[n]=0;return text;
}
/* Rehydrate only actual GCC path operands through exact authenticated roots.
 * Mutable leaf outputs use the canonical /outputs/workspace namespace; readonly
 * operands use their declared physical Input/Runtime root, never host prefixes. */
static char *native_gcc_logical_word(const char *word)
{
    const char *path=NULL;
    if(word[0]=='/')path=word;
    else if(word[0]=='-'&&strchr("ILBo",word[1])&&word[2]=='/')path=word+2;
    else if(word[0]=='-'&&strncmp(word,"-D",2)&&strncmp(word,"-U",2)&&strncmp(word,"-fmacro-prefix-map=",19)&&strncmp(word,"-fdebug-prefix-map=",19)&&strncmp(word,"-ffile-prefix-map=",18)){
        const char *equal=strchr(word,'=');if(equal&&equal[1]=='/')path=equal+1;
    }
    const char root[]="/outputs/workspace";size_t root_length=sizeof root-1;
    if(!path)return NULL;
    if(strncmp(path,root,root_length)||(path[root_length]&&path[root_length]!='/')){
        const struct mantle_paths_root *selected=NULL;size_t selected_length=0;
        for(size_t i=0;i<mantle_paths_count;i++){
            const struct mantle_paths_root *mapping=&mantle_paths_roots[i];size_t n=strlen(mapping->physical);
            if(n>selected_length&&!strncmp(path,mapping->physical,n)&&(!path[n]||path[n]=='/')){selected=mapping;selected_length=n;}
        }
        if(!selected||!strcmp(selected->physical,selected->logical))return NULL;
        Buf restored={0};putn(&restored,word,(size_t)(path-word));putsb(&restored,selected->logical);putsb(&restored,path+selected_length);return done(&restored);
    }
    const char *relative=path+root_length;if(*relative=='/')++relative;
    for(size_t i=0;i<mantle_paths_count;i++){
        struct mantle_paths_root *mapping=&mantle_paths_roots[i];size_t n=strlen(mapping->target);
        if(mapping->kind!=MANTLE_PATHS_WORKSPACE||strncmp(relative,mapping->target,n)||(relative[n]&&relative[n]!='/'))continue;
        Buf restored={0};putn(&restored,word,(size_t)(path-word));putsb(&restored,mapping->logical);putsb(&restored,relative+n);return done(&restored);
    }
    die("native GCC emitted an unmapped canonical workspace operand: %s",word);return NULL;
}
static Args native_gcc_actual_arguments(size_t count)
{
    Args result={alloc(count*sizeof(char *)+count),(int)count};
    unsigned char *owned=(unsigned char *)(result.v+count);
    for(size_t i=0;i<count;i++){char *logical=native_gcc_logical_word(native_gcc_phase_arguments[i]);owned[i]=logical!=NULL;result.v[i]=logical?logical:native_gcc_phase_arguments[i];}
    return result;
}
static void native_gcc_actual_arguments_free(Args *a)
{
    unsigned char *owned=(unsigned char *)(a->v+a->n);for(int i=0;i<a->n;i++)if(owned[i])free(a->v[i]);free(a->v);
}
static char *native_gcc_logical_environment(const char *key,const char *value)
{
    if(!strcmp(key,"COLLECT_GCC_OPTIONS"))return NULL;
    if(strcmp(key,"COMPILER_PATH")&&strcmp(key,"LIBRARY_PATH"))return native_gcc_logical_word(value);
    Buf result={0};int changed=0;
    for(const char *p=value;;){const char *end=strchr(p,':');size_t n=end?(size_t)(end-p):strlen(p);char *part=slice(p,n),*logical=native_gcc_logical_word(part);putsb(&result,logical?logical:part);changed|=logical!=NULL;free(logical);free(part);if(!end)break;putsb(&result,":");p=end+1;}
    if(!changed){free(result.s);return NULL;}return done(&result);
}
static int native_gcc_metadata(Env *e,const char *line)
{
    static const char *const keys[]={"COLLECT_GCC","COLLECT_GCC_OPTIONS","COLLECT_LTO_WRAPPER","COMPILER_PATH","LIBRARY_PATH"};
    for(size_t i=0;i<sizeof keys/sizeof keys[0];i++){
        size_t n=strlen(keys[i]);
        if(!strncmp(line,keys[i],n)&&line[n]=='='){
            char *logical=native_gcc_logical_environment(keys[i],line+n+1);
            native_gcc_export(e,keys[i],logical?logical:line+n+1);free(logical);return 1;
        }
    }
    return native_gcc_phase_metadata(line);
}
static int native_gcc_is_leaf(const char *image)
{
    return mantle_controller_oneof(native_gcc_tool_name(image),"cc1 cc1plus as ld ld.bfd ld.gold collect2");
}
/* Only a complete, actual first-link query reaches this boundary. Original
 * shell linker adapters are interpreted by the composing native shell hook. */
static int native_gcc_link_query(Env *driver,Args *collect)
{
    const char *linker=get(driver,"MANTLE_NATIVE_COLLECT2_LINKER");
    if(!linker)linker=get(driver,"MANTLE_GCC_LINKER");
    if(!linker)linker=get(driver,"MANTLE_NATIVE_LINKER");
    if(!linker||!*linker)die("native collect2 lacks original exact linker binding");
    Env query=clone(driver);FILE *capture=tmpfile();if(!capture)die("native collect2 query stream");
    query.out=query.fd[1]=capture;
    query.fd_flags[1]=O_WRONLY;query.fd_cloexec[1]=0;
    native_gcc_export(&query,"MANTLE_COLLECT2_PHASE_QUERY","1");
    native_gcc_export(&query,"MANTLE_COLLECT2_PHASE_LINKER",linker);
    native_gcc_export(&query,"MANTLE_COLLECT2_PHASE_DRIVER_ENV","1");
    int status=mantle_shell_external(&query,collect);
    char *text=native_gcc_stream(capture);
    native_gcc_descriptor_flags(driver,&query);
    if(fclose(capture))die("native collect2 query close");
    clearvars(&query);free(query.cwd);
    if(status){free(text);return status;}
    char *end=strchr(text,'\n');
    if(!end||end[1])die("native collect2 query must contain exactly one original link");
    *end=0;size_t n=native_gcc_phase_split(text);
    if(!n)die("native collect2 omitted its original linker");
    Args actual=native_gcc_actual_arguments(n);
    if(strcmp(actual.v[0],linker))die("native collect2 changed its declared original linker");
    /* Reenter only the source shell dispatcher: a genuine wrapper script is
     * interpreted, while an ELF link crosses the protected raw boundary. */
    status=external(driver,&actual);
    native_gcc_actual_arguments_free(&actual);free(text);return status;
}
static int native_gcc_external(Env *e,Args *a)
{
    if(!a->n||!native_gcc_driver_name(mantle_controller_basename(a->v[0])))return -1;
    int verbose=0,explicit_query=0;
    for(int i=1;i<a->n;i++){
        if(!strcmp(a->v[i],"-###"))explicit_query=1;
        if(!strcmp(a->v[i],"-v"))verbose=1;
    }
    if(explicit_query)return mantle_shell_external(e,a);
    Args request={.v=alloc(((size_t)a->n+1)*sizeof *request.v),.n=a->n+1,.argv0=a->argv0};
    memcpy(request.v,a->v,(size_t)a->n*sizeof *request.v);request.v[a->n]=(char *)"-###";
    Env driver=clone(e);FILE *capture=tmpfile();if(!capture)die("native GCC query stream");
    driver.err=driver.fd[2]=capture;
    driver.fd_flags[2]=O_WRONLY;driver.fd_cloexec[2]=0;
    int status=mantle_shell_external(&driver,&request);
    free(request.v);
    char *text=native_gcc_stream(capture);if(fclose(capture))die("native GCC query close");
    driver.err=e->err;driver.fd[2]=e->fd[2];
    driver.fd_flags[2]=e->fd_flags[2];driver.fd_cloexec[2]=e->fd_cloexec[2];
    char *cursor=text;
    while(*cursor){
        char *line=cursor,*end=strchr(cursor,'\n');if(!end)die("native GCC unterminated actual phase log");
        *end=0;cursor=end+1;
        if(native_gcc_metadata(&driver,line)){if(verbose&&*line&&e->err)fprintf(e->err,"%s\n",line);continue;}
        if(*line!=' '&&*line!='\t'){
            if(e->err)fprintf(e->err,"%s\n",line);continue;
        }
        if(status)continue; /* A rejected driver query never runs a child. */
        if(verbose&&e->err)fprintf(e->err,"%s\n",line);
        size_t n=native_gcc_phase_split(line);
        if(!n||!native_gcc_is_leaf(native_gcc_phase_arguments[0]))die("native GCC selected an unauthored original child phase");
        Args actual=native_gcc_actual_arguments(n);
        const char *name=native_gcc_tool_name(actual.v[0]);
        int child=!strcmp(name,"collect2")?native_gcc_link_query(&driver,&actual):mantle_shell_external(&driver,&actual);
        native_gcc_actual_arguments_free(&actual);
        if(child){status=child;break;}
    }
    native_gcc_descriptor_flags(e,&driver);
    free(text);clearvars(&driver);free(driver.cwd);return status;
}
static void native_gcc_directory_separator(mantle_controller_builder *b)
{
    mantle_controller_part(b);putsb(&b->word,"{\"kind\":\"directory_separator\"}");
}
static void native_gcc_path(mantle_controller_builder *b,const char *path,int writing)
{
    size_t n=strlen(path);int directory=n&&path[n-1]=='/';
    if(directory){char *clean=dup(path);while(n&&clean[n-1]=='/')clean[--n]=0;if(!n)die("native compiler root directory operand");mantle_controller_path(b,clean,writing);free(clean);native_gcc_directory_separator(b);}
    else mantle_controller_path(b,path,writing);
}
static void native_gcc_argument_path(mantle_controller_builder *b,const char *path,int writing)
{
    mantle_controller_word_begin(b);native_gcc_path(b,path,writing);mantle_controller_word_end(b);
}
static void native_gcc_joined_path(mantle_controller_builder *b,const char *word,size_t prefix,int writing)
{
    char *flag=slice(word,prefix);mantle_controller_word_begin(b);mantle_controller_literal(b,flag);native_gcc_path(b,word+prefix,writing);mantle_controller_word_end(b);free(flag);
}
static void native_gcc_generated_record(Buf *map,const char *value)
{
    char length[32];snprintf(length,sizeof length,"%lu:",(unsigned long)strlen(value));putsb(map,length);putsb(map,value);putsb(map,",");
}
static void native_gcc_generated_environment(Env *environment)
{
    Buf map={0};native_gcc_generated_record(&map,"MANTLE-NATIVE-GENERATED-ROOTS-1");
    for(size_t i=0;i<mantle_paths_count;i++)if(mantle_paths_roots[i].kind==MANTLE_PATHS_WORKSPACE){
        native_gcc_generated_record(&map,mantle_paths_roots[i].logical);
        native_gcc_generated_record(&map,mantle_paths_roots[i].target);
    }
    native_gcc_export(environment,"MANTLE_NATIVE_GENERATED_ROOT","generated");
    native_gcc_export(environment,"MANTLE_NATIVE_GENERATED_MAP",done(&map));
    free(map.s);
}
static char *native_gcc_environment(mantle_controller_builder *b,const char *key,const char *value)
{
    if(!strcmp(key,"MANTLE_NATIVE_GENERATED_ROOT")){
        if(strcmp(value,"generated"))die("native genuine writer root metadata collision");
        return mantle_controller_named_word(b,1,"generated","");
    }
    int list=mantle_controller_oneof(key,"COMPILER_PATH LIBRARY_PATH GCC_EXEC_PREFIX_PATH CPATH C_INCLUDE_PATH CPLUS_INCLUDE_PATH OBJC_INCLUDE_PATH PATH LD_LIBRARY_PATH PYTHONPATH PERL5LIB");
    int single=mantle_controller_oneof(key,"COLLECT_GCC COLLECT_LTO_WRAPPER GCC_EXEC_PREFIX TMPDIR PYTHONHOME MANTLE_GCC_ASSEMBLER MANTLE_GCC_LINKER MANTLE_GCC_LINK_LIBC MANTLE_GCC_LINK_LIBM MANTLE_GCC_LINK_LIBGCC MANTLE_GCC_LINK_CRT MANTLE_NATIVE_LINKER MANTLE_NATIVE_COLLECT2_LINKER MANTLE_COLLECT2_PHASE_LINKER MANTLE_COLLECT2_PHASE_DRIVER_LOG");
    if(!list&&!single)return NULL;
    mantle_controller_word_begin(b);
    if(single){if(*value)native_gcc_path(b,value,0);else mantle_controller_literal(b,"");}
    else for(const char *p=value;;){const char *end=strchr(p,':');size_t n=end?(size_t)(end-p):strlen(p);if(n){char *part=slice(p,n);native_gcc_path(b,part,0);free(part);}else mantle_controller_literal(b,"");if(!end)break;mantle_controller_literal(b,":");p=end+1;}
    return mantle_controller_take_word(b);
}
/* Source option grammar for already-selected leaf commands. Driver arguments
 * remain driver arguments; this never chooses a cc1/as/ld flag. */
static void native_gcc_linker_option(mantle_controller_builder *b,const char *word)
{
    int path_next=0,data_next=0;
    mantle_controller_word_begin(b);mantle_controller_literal(b,"-Wl,");
    for(const char *p=word+4;;){
        const char *end=strchr(p,',');size_t n=end?(size_t)(end-p):strlen(p);
        char *part=slice(p,n);
        if(path_next){native_gcc_path(b,part,0);path_next=0;}
        else if(data_next){mantle_controller_data(b,part);data_next=0;}
        else if(mantle_controller_oneof(part,"-L -T --script --version-script -dynamic-linker --dynamic-linker -rpath --rpath -rpath-link --rpath-link --sysroot")){
            mantle_controller_literal(b,part);path_next=1;
        }else if(mantle_controller_oneof(part,"--defsym -z -m -u -e -soname --soname")){
            mantle_controller_literal(b,part);data_next=1;
        }else {
            char *equal=strchr(part,'=');int joined_path=0;
            if(equal){*equal=0;joined_path=mantle_controller_oneof(part,"--script --version-script --dynamic-linker --rpath --rpath-link --sysroot");*equal='=';}
            if(joined_path){
                char saved=equal[1];equal[1]=0;mantle_controller_literal(b,part);equal[1]=saved;native_gcc_path(b,equal+1,0);
            }else if(part[0]=='-'&&part[1]=='L'&&part[2]){mantle_controller_literal(b,"-L");native_gcc_path(b,part+2,0);}
            else if(part[0]!='-'&&strchr(part,'/'))native_gcc_path(b,part,0);
            else if(strchr(part,'/')||strchr(part,'\\'))die("native unclassified original linker option: %s",part);
            else mantle_controller_data(b,part);
        }
        free(part);if(!end)break;mantle_controller_literal(b,",");p=end+1;
    }
    if(path_next||data_next)die("native original joined linker option missing operand");
    mantle_controller_word_end(b);
}
static int native_gcc_prepare(const char *slot,Request *request,mantle_controller_builder *b)
{
    Args *a=&request->command;const char *name=native_gcc_tool_name(a->v[0]);
    int driver=native_gcc_driver_name(name),front=mantle_controller_oneof(name,"cc1 cc1plus"),assembler=!strcmp(name,"as"),link=mantle_controller_oneof(name,"ld ld.bfd ld.gold collect2"),archive=mantle_controller_oneof(name,"ar ranlib nm strip objcopy objdump readelf strings size addr2line c++filt elfedit"),tcc=!strcmp(name,"tcc");
    if(!driver&&!front&&!assembler&&!link&&!archive&&!tcc)return !strcmp(slot,"native-generated")&&a->n==1;
    if((link&&strcmp(name,"collect2"))||tcc)native_gcc_generated_environment(&request->env);
    if(driver){int query=0;for(int i=1;i<a->n;i++)if(!strcmp(a->v[i],"-###"))query=1;if(!query)die("native GCC driver cannot execute an opaque child pipeline");}
    int options=1,archive_operation=!strcmp(name,"ar"),bare_operands=0,explicit_output=0;
    for(int i=1;i<a->n;i++){
        if(!strcmp(a->v[i],"-o"))explicit_output=1;
        if(archive_operation&&mantle_controller_oneof(a->v[i],"--version --help -V -h"))archive_operation=0;
    }
    for(int i=1;i<a->n;i++){
        const char *s=a->v[i];
        if(options&&!strcmp(s,"--")){mantle_controller_argument_literal(b,s);options=0;continue;}
        if(options&&s[0]=='@'){
            if(!s[1])die("native original response file has empty name");
            native_gcc_joined_path(b,s,1,0);continue;
        }
        if(archive_operation){mantle_controller_argument_data(b,s);archive_operation=0;if(++i==a->n)die("native archive lacks original archive path");native_gcc_argument_path(b,a->v[i],1);continue;}
        if(options&&(mantle_controller_oneof(s,"-o -I -L -B -isystem -iquote -idirafter -iprefix -iwithprefix -iwithprefixbefore -isysroot --sysroot -include -imacros -iplugindir -MF -T --script --version-script -dynamic-linker --dynamic-linker -rpath --rpath -rpath-link --rpath-link -plugin --plugin")||((front||assembler)&&mantle_controller_oneof(s,"-MD -MMD")))){
            int writing=mantle_controller_oneof(s,"-o -MF -MD -MMD");
            mantle_controller_argument_literal(b,s);if(++i==a->n)die("native compiler path option missing operand");native_gcc_argument_path(b,a->v[i],writing);continue;
        }
        if(options&&mantle_controller_oneof(s,"-D -U -x -dumpbase -dumpbase-ext -auxbase -auxbase-strip -MT -MQ -imultilib -imultiarch -march -mcpu -mtune -mabi -z -m -u -e -soname --soname --defsym -plugin-opt --plugin-opt")){
            mantle_controller_argument_literal(b,s);if(++i==a->n)die("native compiler data option missing operand");mantle_controller_argument_data(b,a->v[i]);continue;
        }
        if(options&&s[0]=='-'&&s[1]){
            if(driver&&!strncmp(s,"-Wl,",4)){native_gcc_linker_option(b,s);continue;}
            if(!strncmp(s,"-D",2)||!strncmp(s,"-U",2)||!strncmp(s,"-fmacro-prefix-map=",19)||!strncmp(s,"-fdebug-prefix-map=",19)||!strncmp(s,"-ffile-prefix-map=",18)||!strncmp(s,"-frandom-seed=",14)){
                mantle_controller_argument_data(b,s);continue;
            }
            if(strchr("ILB",s[1])&&s[2]){native_gcc_joined_path(b,s,2,0);continue;}
            if(s[1]=='o'&&s[2]){native_gcc_joined_path(b,s,2,1);continue;}
            static const char *const paths[]={"--sysroot=","-isysroot=","-specs=","--script=","--version-script=","--dynamic-linker=","--rpath=","--rpath-link=","-fplugin=","-iplugindir=","-iprefix=","-isystem=","-iquote=","-idirafter="};
            int mapped=0;for(size_t p=0;p<sizeof paths/sizeof paths[0];p++){size_t n=strlen(paths[p]);if(!strncmp(s,paths[p],n)){native_gcc_joined_path(b,s,n,0);mapped=1;break;}}
            if(mapped)continue;
            if(strchr(s,'/')||strchr(s,'\\'))die("native unclassified source compiler path option: %s",s);
            mantle_controller_argument_literal(b,s);continue;
        }
        if(!strcmp(s,"-"))mantle_controller_argument_literal(b,s);
        else {int writing=!strcmp(name,"ranlib")||(!strcmp(name,"strip")&&!explicit_output)||(!strcmp(name,"objcopy")&&bare_operands>0);native_gcc_argument_path(b,s,writing);bare_operands++;}
    }
    return 1;
}
