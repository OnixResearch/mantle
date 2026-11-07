/* Rust Std's original MANTLE-SHELL-2 requests use the common source evaluator.
 * Replay records are immutable inputs to the script; only actual final script
 * streams escape this driver. There is no subprocess launcher here. */
#define RUST_SCRIPT_HISTORY ".mantle-rust-controller"
static unsigned long rust_script_native_begin,rust_script_native_max;
static void (*rust_script_suspend_prior)(Env *);
static void rust_script_native_suspend(Env *e){
 if(request_ordinal<rust_script_native_begin||request_ordinal-rust_script_native_begin>=rust_script_native_max)die("original Rust request exhausted native command reservation");
 if(rust_script_suspend_prior)rust_script_suspend_prior(e);
}
static unsigned long rust_script_number(const char *text,int positive){
 if(!text||!*text)die("missing original Rust source reservation");
 unsigned long n=0;for(const unsigned char *p=(const unsigned char *)text;*p;p++){if(*p<'0'||*p>'9'||n>(ULONG_MAX-(*p-'0'))/10)die("invalid original Rust source reservation");n=n*10+*p-'0';}
 if(positive&&!n)die("empty original Rust native command reservation");return n;
}
static void rust_script_move(const char *from,const char *to){if(rename(from,to))die("original Rust continuation rename: %s",strerror(errno));}
static int rust_script_exists(const char *path){struct stat st;if(!lstat(path,&st))return 1;if(errno!=ENOENT)die("original Rust continuation lookup");return 0;}
static char *rust_script_physical(const char *logical,int writing){
 struct mantle_paths_operand path;if(mantle_paths_map(logical,writing,&path)||path.kind!=MANTLE_PATHS_WORKSPACE||strcmp(path.logical_root,"/tmp"))die("original Rust output escaped temporary workspace");return dup(path.physical);
}
static void rust_script_forget(Env *e,const char *key){
 Var **next=&e->vars;while(*next){Var *v=*next;if(!strcmp(v->key,key)){*next=v->next;free(v->key);free(v->value);free(v);return;}next=&v->next;}
}
static void rust_script_request_free(Request *r){
 mantle_native_env_free(&r->env,1);free(r->env.zero);for(int i=0;i<r->env.argc;i++)free(r->env.args[i]);free(r->env.args);
 for(int i=0;i<32;i++)free(r->env.traps[i]);
 for(int i=0;i<64;i++)if(r->aliases[i]==i)free(r->descriptor_workspace[i]);
 free(r->command.argv0);llvm_free_args(&r->command);free(r->root);free(r->workspace);
}
/* The readonly input has a writable workspace duplicate. Keep exact content,
 * names, kinds and modes outside that workspace across every script leaf. */
static void rust_script_fact_integer(FILE *facts,long long value,int verify){
 if(verify){if(read_integer(facts)!=value)die("original Rust script changed immutable replay history");}else integer(facts,value);
}
static void rust_script_history(FILE *facts,const char *path,int verify,unsigned long *nodes){
 if(++*nodes>262144UL)die("original Rust history exceeds workspace node bound");struct stat st;if(lstat(path,&st))die("original Rust script removed immutable replay history");
 rust_script_fact_integer(facts,st.st_mode&(S_IFMT|0777),verify);
 if(S_ISDIR(st.st_mode)){
  Args names=llvm_names(path);rust_script_fact_integer(facts,names.n,verify);
  for(int i=0;i<names.n;i++){
   if(verify){char *name=read_record(facts);if(strcmp(name,names.v[i]))die("original Rust script changed immutable history names");free(name);}else record(facts,names.v[i]);
   char *child=join(path,names.v[i]);rust_script_history(facts,child,verify,nodes);free(child);
  }llvm_free_args(&names);
 }else if(S_ISREG(st.st_mode)){
  if(st.st_size<0)die("original Rust history file size");rust_script_fact_integer(facts,st.st_size,verify);
  FILE *file=fopen(path,"rb");if(!file)die("original Rust history file");unsigned char bytes[4096],expected[4096];off_t remaining=st.st_size;size_t n;
  while((n=fread(bytes,1,sizeof bytes,file))){
   if((off_t)n>remaining)die("original Rust history changed while observed");remaining-=(off_t)n;
   if(verify){if(fread(expected,1,n,facts)!=n||memcmp(bytes,expected,n))die("original Rust script changed immutable history bytes");}
   else if(fwrite(bytes,1,n,facts)!=n)die("original Rust history fact write");
  }if(ferror(file)||fclose(file)||remaining)die("original Rust history file observation");
 }else if(S_ISLNK(st.st_mode)){
  char *target=llvm_link(path,&st);if(verify){char *expected=read_record(facts);if(strcmp(target,expected))die("original Rust script changed immutable history link");free(expected);}else record(facts,target);free(target);
 }else die("original Rust history member kind");
}
static void rust_script_acknowledge(Env *outer,const char *pending,const char *scratch,const char *auxiliary,const char *parked,unsigned long ordinal,unsigned long bound){
 /* Park the private history before common workspace decoding/snapshots. The
  * captured request is outside that workspace, so no copy can contain itself. */
 rust_script_move(auxiliary,parked);
 char *captured=join(scratch,"captured");remove_tree(captured);copy_tree(pending,captured);
 char *snapshot=join(captured,"workspace"),*temporary=join(snapshot,"tmp"),*tmp=join(workspace_root,"tmp");mkdir_checked(snapshot);copy_tree(tmp,temporary);free(snapshot);free(temporary);free(tmp);
 Request r;read_request_data_at(captured,&r,workspace_root);if(!r.active||r.ordinal!=ordinal)die("original Rust pending request ordinal/state mismatch");
 if(mantle_paths_chdir(r.env.cwd))die("original Rust captured command cwd");
 rust_script_native_begin=request_ordinal;rust_script_native_max=bound;rust_script_suspend_prior=mantle_source_limit_suspend_hook;mantle_source_limit_suspend_hook=rust_script_native_suspend;
 int status=external(&r.env,&r.command);
 mantle_source_limit_suspend_hook=rust_script_suspend_prior;rust_script_suspend_prior=NULL;
 if(status<0||status>255)die("original Rust command has no normal status");
 char *result=join(scratch,"result");remove_tree(result);write_result(result,&r,status,NULL,NULL);
 /* Std installs reply/workspace at /tmp, while the shared timestamp protocol
  * retains its workspace-relative tmp/ prefix and exact descriptor positions. */
 char *whole=join(result,"workspace"),*full=join(scratch,"result-workspace");remove_tree(full);rust_script_move(whole,full);char *part=join(full,"tmp");rust_script_move(part,whole);if(rmdir(full))die("original Rust result contains non-tmp workspace");free(part);free(full);free(whole);
 char *record_path=join(result,"request"),*record_saved=join(scratch,"request-record");rust_script_move(record_path,record_saved);mkdir_checked(record_path);char *record=join(record_path,"request");rust_script_move(record_saved,record);free(record);free(record_saved);
 for(int i=0;i<64;i++)if(r.aliases[i]==i&&!mantle_descriptor_null(r.env.fd[i])){char name[32];snprintf(name,sizeof name,"descriptor-%d",i);char *from=join(captured,name),*to=join(record_path,name);binary_copy(from,to);free(from);free(to);}free(record_path);
 char *exit_path=join(result,"exit-status"),*status_path=join(result,"status");rust_script_move(exit_path,status_path);free(exit_path);free(status_path);
 char number[32];snprintf(number,sizeof number,"%lu",ordinal);char *history=join(parked,"history"),*reply=join(history,number);if(rust_script_exists(reply))die("duplicate original Rust command acknowledgement");rust_script_move(result,reply);free(history);free(reply);free(result);
 rust_script_request_free(&r);remove_tree(captured);free(captured);rust_script_move(parked,auxiliary);
 if(mantle_paths_chdir(outer->cwd))die("original Rust parent cwd restore");
}
static int rust_script_source_entry(Env *outer,const char *program){
 const char *source=get(outer,"MANTLE_RUST_SCRIPT_IMAGE");if(!source)return -1;(void)program;
 char *image=dup(source);unsigned long slots=rust_script_number(get(outer,"MANTLE_RUST_SCRIPT_SLOTS"),0),bound=rust_script_number(get(outer,"MANTLE_RUST_NATIVE_COMMANDS_MAX"),1);
 const char *out=get(outer,"OUT_DIR"),*request=get(outer,"MANTLE_CC_REQUEST");if(!out||!request||strcmp(image,outer->zero))die("original Rust script metadata mismatch");
 char *expected_request=join(out,".mantle-successor-request");if(strcmp(request,expected_request))die("original Rust request path is not its declared output mailbox");free(expected_request);
 char *logical_out=dup(out),*pending=rust_script_physical(request,1),*out_path=rust_script_physical(out,1),*random=join(out_path,".mantle-cc-random");
 rust_script_forget(outer,"MANTLE_RUST_SCRIPT_IMAGE");rust_script_forget(outer,"MANTLE_RUST_SCRIPT_SLOTS");rust_script_forget(outer,"MANTLE_RUST_NATIVE_COMMANDS_MAX");
 char *scratch=join(output_root,"rust-script"),*base=join(scratch,"base"),*saved_random=join(scratch,"random"),*parked=join(scratch,"history"),*tmp=join(workspace_root,"tmp"),*auxiliary=join(workspace_root,RUST_SCRIPT_HISTORY),*history=join(auxiliary,"history");
 if(rust_script_exists(auxiliary))die("original Rust controller private history collision");mkdir_checked(scratch);mkdir_checked(auxiliary);mkdir_checked(history);copy_tree(tmp,base);
 native_gcc_export(outer,"MANTLE_CC_REPLIES","/outputs/workspace/" RUST_SCRIPT_HISTORY "/history");native_gcc_export(outer,"MANTLE_CC_PHASE_STATE",logical_out);
 Args command={.argv0=image};arg(&command,image);unsigned long ordinal=0;
 for(;;){
  remove_tree(saved_random);int have_random=rust_script_exists(random);if(have_random)copy_tree(random,saved_random);
  synchronize_tree(base,tmp);if(have_random){remove_tree(random);copy_tree(saved_random,random);char *index=join(random,".process-index");remove_tree(index);free(index);}
  Env phase=clone(outer);llvm_private_streams(&phase);
  long at=ftell(outer->in);if(at<0)die("original Rust script input position");transfer(outer->in,phase.in);if(fseek(outer->in,at,SEEK_SET)||fseek(phase.in,0,SEEK_SET))die("original Rust script input rewind");
  FILE *facts=tmpfile();if(!facts)die("original Rust history facts");unsigned long nodes=0;rust_script_history(facts,auxiliary,0,&nodes);if(fflush(facts)||fseek(facts,0,SEEK_SET))die("original Rust history facts rewind");
  if(mantle_paths_chdir(phase.cwd))die("original Rust script cwd");int status=mantle_shell_external(&phase,&command);
  nodes=0;rust_script_history(facts,auxiliary,1,&nodes);if(fgetc(facts)!=EOF||ferror(facts)||fclose(facts))die("original Rust history facts differ");
  if(status==0){
   char *index=join(random,".process-index");unsigned long used=0;if(rust_script_exists(index)){char *text=load(index);size_t n=strlen(text);if(!n||text[n-1]!='\n')die("invalid original Rust final process ordinal");text[n-1]=0;used=rust_script_number(text,0);free(text);}free(index);if(used!=ordinal)die("original Rust script left unused acknowledgements");
   rewind(phase.out);transfer(phase.out,outer->out);rewind(phase.err);transfer(phase.err,outer->err);llvm_close_streams(&phase);mantle_native_env_free(&phase,0);break;
  }
  llvm_close_streams(&phase);mantle_native_env_free(&phase,0);
  if(status!=75)die("original Rust script failed with actual exit %d",status);if(ordinal>=slots)die("original Rust script exhausted source request reservation");
  rust_script_acknowledge(outer,pending,scratch,auxiliary,parked,ordinal++,bound);
 }
 remove_tree(scratch);remove_tree(auxiliary);
 free(command.v);free(image);free(logical_out);free(pending);free(out_path);free(random);free(scratch);free(base);free(saved_random);free(parked);free(tmp);free(auxiliary);free(history);
 if(mantle_paths_chdir(outer->cwd))die("original Rust final cwd restore");return 0;
}
/* Rust option grammar classifies operands without choosing flags or results. */
static void rust_script_search_path(mantle_controller_builder *b,const char *word){
 const char *equal=strchr(word,'=');mantle_controller_word_begin(b);
 if(equal){char *prefix=slice(word,(size_t)(equal-word)+1);mantle_controller_literal(b,prefix);free(prefix);word=equal+1;}
 mantle_controller_path(b,word,0);mantle_controller_word_end(b);
}
static void rust_script_emit(mantle_controller_builder *b,const char *word,int joined){
 mantle_controller_word_begin(b);if(joined)mantle_controller_literal(b,"--emit=");
 for(const char *p=word;;){const char *end=strchr(p,',');size_t n=end?(size_t)(end-p):strlen(p);char *part=slice(p,n),*equal=strchr(part,'=');
  if(equal){*equal=0;mantle_controller_literal(b,part);mantle_controller_literal(b,"=");mantle_controller_path(b,equal+1,1);}else mantle_controller_data(b,part);free(part);
  if(!end)break;mantle_controller_literal(b,",");p=end+1;
 }mantle_controller_word_end(b);
}
static int rust_script_prepare(const char *slot,Request *r,mantle_controller_builder *b){
 (void)slot;Args *a=&r->command;
 if(a->n==1&&get(&r->env,"MANTLE_CC_REPLIES")&&r->env.zero&&!strcmp(a->v[0],r->env.zero))return 1;
 const char *compiler=get(&r->env,"RUSTC");if(strcmp(mantle_controller_basename(a->v[0]),"rustc")&&(!compiler||strcmp(a->v[0],compiler)))return 0;
 for(int i=1;i<a->n;i++){const char *s=a->v[i];
  if(mantle_controller_oneof(s,"-o --out-dir --sysroot --extern -L --emit --crate-name --crate-type --edition --target --print --cfg --check-cfg --cap-lints --error-format --json --color -C -Z -A -W -D -F")){
   mantle_controller_argument_literal(b,s);if(++i==a->n)die("original rustc option missing operand");const char *value=a->v[i];
   if(mantle_controller_oneof(s,"-o --out-dir --sysroot"))mantle_controller_argument_path(b,value,strcmp(s,"--sysroot")!=0);
   else if(!strcmp(s,"-L"))rust_script_search_path(b,value);
   else if(!strcmp(s,"--extern")&&strchr(value,'='))rust_script_search_path(b,value);
   else if(!strcmp(s,"--emit"))rust_script_emit(b,value,0);
   else if(!strcmp(s,"--target")&&strchr(value,'/'))mantle_controller_argument_path(b,value,0);
   else if(!strcmp(s,"-C")&&(!strncmp(value,"linker=",7)||!strncmp(value,"incremental=",12))){const char *equal=strchr(value,'=');char *prefix=slice(value,(size_t)(equal-value)+1);mantle_controller_option_path(b,value,prefix,!strncmp(value,"incremental=",12));free(prefix);}
   else mantle_controller_argument_data(b,value);
  }else if(!strncmp(s,"--out-dir=",10))mantle_controller_option_path(b,s,"--out-dir=",1);
  else if(!strncmp(s,"--sysroot=",10))mantle_controller_option_path(b,s,"--sysroot=",0);
  else if(!strncmp(s,"-L",2)&&s[2]){mantle_controller_word_begin(b);mantle_controller_literal(b,"-L");const char *value=s+2,*equal=strchr(value,'=');if(equal){char *prefix=slice(value,(size_t)(equal-value)+1);mantle_controller_literal(b,prefix);free(prefix);value=equal+1;}mantle_controller_path(b,value,0);mantle_controller_word_end(b);}
  else if(!strncmp(s,"--extern=",9)&&strchr(s+9,'=')){mantle_controller_word_begin(b);const char *equal=strchr(s+9,'=');char *prefix=slice(s,(size_t)(equal-s)+1);mantle_controller_literal(b,prefix);free(prefix);mantle_controller_path(b,equal+1,0);mantle_controller_word_end(b);}
  else if(!strncmp(s,"--emit=",7))rust_script_emit(b,s+7,1);
  else if(!strncmp(s,"-Clinker=",9))mantle_controller_option_path(b,s,"-Clinker=",0);
  else if(!strncmp(s,"-Cincremental=",14))mantle_controller_option_path(b,s,"-Cincremental=",1);
  else if(!strncmp(s,"--target=",9)&&strchr(s+9,'/'))mantle_controller_option_path(b,s,"--target=",0);
  else if(mantle_controller_oneof(s,"- -v -V -vV --version --verbose --help -h -g -O --test")||!strncmp(s,"--crate-name=",13)||!strncmp(s,"--crate-type=",13)||!strncmp(s,"--edition=",10)||!strncmp(s,"--target=",9)||!strncmp(s,"--print=",8)||!strncmp(s,"--cfg=",6)||!strncmp(s,"--check-cfg=",12)||!strncmp(s,"--cap-lints=",12)||!strncmp(s,"--error-format=",15)||!strncmp(s,"--json=",7)||!strncmp(s,"--color=",8)||(!strncmp(s,"-C",2)&&s[2])||(!strncmp(s,"-Z",2)&&s[2])||(!strncmp(s,"-A",2)&&s[2])||(!strncmp(s,"-W",2)&&s[2])||(!strncmp(s,"-D",2)&&s[2])||(!strncmp(s,"-F",2)&&s[2]))mantle_controller_argument_data(b,s);
  else if(s[0]=='-')die("unsupported original rustc option: %s",s);
  else mantle_controller_argument_path(b,s,0);
 }return 1;
}
static char *rust_script_environment(mantle_controller_builder *b,const char *key,const char *value){
 if(!strcmp(key,"MANTLE_CC_REPLIES")){
  if(strcmp(value,"/outputs/workspace/" RUST_SCRIPT_HISTORY "/history"))return NULL;
  /* workspace-seed is the controller runtime's reserved immutable input, not
   * an ordinary command binding accepted by mantle_controller_input. */
  mantle_controller_word_begin(b);mantle_controller_typed(b,"input","name","workspace-seed",RUST_SCRIPT_HISTORY "/history");return mantle_controller_take_word(b);
 }
 if(mantle_controller_oneof(key,"MANTLE_CC_PHASE_STATE MANTLE_CC_REQUEST MANTLE_CAIRN_SCRIPT_ENV MANTLE_CAIRN_COMPILE_ENV OUT_DIR CARGO_MANIFEST_DIR CARGO_MANIFEST_PATH RUSTC RUSTDOC LLVM_CONFIG CC CXX AR AS LD NM RANLIB OBJC OBJCXX HOST_CC HOST_CXX HOST_AR RUSTC_LINKER")){
  if(mantle_controller_oneof(key,"RUSTC RUSTDOC LLVM_CONFIG CC CXX AR AS LD NM RANLIB OBJC OBJCXX HOST_CC HOST_CXX HOST_AR RUSTC_LINKER")&&!strchr(value,'/'))return mantle_controller_data_word(b,value);
  mantle_controller_word_begin(b);mantle_controller_path(b,value,mantle_controller_oneof(key,"MANTLE_CC_PHASE_STATE MANTLE_CC_REQUEST OUT_DIR"));return mantle_controller_take_word(b);
 }
 return NULL;
}
