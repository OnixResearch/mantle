/* Prepare status 1 means genuinely unselected; start/resume status 1 means
 * actual source completion. Protocol failures remain distinct status 2. */
static void mantle_controller_die(const char *format,...) {
 va_list arguments;va_start(arguments,format);vfprintf(stderr,format,arguments);va_end(arguments);fputc('\n',stderr);exit(2);
}
#define die mantle_controller_die

/* Appended after the shared shell Request protocol and logical path mapper.
 * This module constructs data, never executes an original command. */
typedef struct mantle_controller_binding {
 const char *name,*path; int used;
} mantle_controller_binding;
typedef struct mantle_controller_builder {
 Request *request;
 FILE *data;
 unsigned long long data_offset;
 Args words;
 Buf word;
 int in_word,parts;
 mantle_controller_binding *bindings;
 int binding_count;
 int private_descriptors;
} mantle_controller_builder;
static int (*mantle_controller_prepare_hook)(const char *,Request *,mantle_controller_builder *);
static char *(*mantle_controller_environment_hook)(mantle_controller_builder *,const char *,const char *);
static void (*mantle_controller_source_scope_resume_hook)(Env *);
static void (*mantle_controller_process_attributes_hook)(mantle_controller_builder *,Buf *,int *);

static void mantle_controller_json(Buf *b,const char *s) {
 putsb(b,"\"");
 for(const unsigned char *p=(const unsigned char *)s;*p;p++) {
  char escaped[7];
  switch(*p) {
   case '"':putsb(b,"\\\"");break; case '\\':putsb(b,"\\\\");break;
   case '\b':putsb(b,"\\b");break; case '\f':putsb(b,"\\f");break;
   case '\n':putsb(b,"\\n");break; case '\r':putsb(b,"\\r");break;
   case '\t':putsb(b,"\\t");break;
   default:if(*p<32){snprintf(escaped,sizeof escaped,"\\u%04x",*p);putsb(b,escaped);}else putn(b,(const char *)p,1);
  }
 }
 putsb(b,"\"");
}
static void mantle_controller_utf8(const char *s) {
 const unsigned char *p=(const unsigned char *)s;
 while(*p) {
  unsigned long value;int n;
  if(*p<128){p++;continue;}
  if(*p>=0xc2&&*p<=0xdf){value=*p&31;n=1;}
  else if(*p>=0xe0&&*p<=0xef){value=*p&15;n=2;}
  else if(*p>=0xf0&&*p<=0xf4){value=*p&7;n=3;}
  else die("controller source data is not UTF-8");
  p++;for(int i=0;i<n;i++){if((*p&0xc0)!=0x80)die("controller source data is not UTF-8");value=(value<<6)|(*p++&63);}
  if((n==2&&value<0x800)||(n==3&&value<0x10000)||value>0x10ffff||(value>=0xd800&&value<=0xdfff))die("controller source data is not UTF-8");
 }
}
static void mantle_controller_relative(const char *p) {
 if(*p=='/'||strchr(p,'\\'))die("controller nonrelative typed operand: %s",p);
 while(*p){const char *q=p;while(*p&&*p!='/')p++;size_t n=p-q;if(!n||(n==1&&*q=='.')||(n==2&&q[0]=='.'&&q[1]=='.'))die("controller noncanonical typed operand");if(*p&&!*++p)die("controller trailing operand separator");}
}
static void mantle_controller_word_begin(mantle_controller_builder *b) {
 if(b->in_word)die("controller nested word");memset(&b->word,0,sizeof b->word);putsb(&b->word,"{\"parts\":[");b->parts=0;b->in_word=1;
}
static void mantle_controller_part(mantle_controller_builder *b) {
 if(!b->in_word)die("controller operand outside word");if(b->parts++)putsb(&b->word,",");
}
static void mantle_controller_literal(mantle_controller_builder *b,const char *s) {
 if(strchr(s,'/')||strchr(s,'\\'))die("controller untyped literal path: %s",s);
 mantle_controller_utf8(s);mantle_controller_part(b);putsb(&b->word,"{\"kind\":\"literal\",\"text\":");mantle_controller_json(&b->word,s);putsb(&b->word,"}");
}
static void mantle_controller_data(mantle_controller_builder *b,const char *s) {
 mantle_controller_utf8(s);size_t n=strlen(s);if(n>65536||b->data_offset+n>64UL*1024*1024)die("controller source frame bound");
 if(fwrite(s,1,n,b->data)!=n)die("controller source frame write");
 mantle_controller_part(b);char numbers[160];snprintf(numbers,sizeof numbers,"{\"kind\":\"text_slice\",\"name\":\"data\",\"offset_bytes\":%llu,\"length_bytes\":%lu}",b->data_offset,(unsigned long)n);putsb(&b->word,numbers);b->data_offset+=n;
}
static void mantle_controller_typed(mantle_controller_builder *b,const char *kind,const char *key,const char *name,const char *path) {
 mantle_controller_relative(path);mantle_controller_utf8(name);mantle_controller_part(b);putsb(&b->word,"{\"kind\":");mantle_controller_json(&b->word,kind);putsb(&b->word,",");mantle_controller_json(&b->word,key);putsb(&b->word,":");mantle_controller_json(&b->word,name);putsb(&b->word,",\"path\":");mantle_controller_json(&b->word,path);putsb(&b->word,"}");
}
static void mantle_controller_input(mantle_controller_builder *b,const char *name,const char *path) {
 int found=0;for(int i=0;i<b->binding_count;i++)if(!strcmp(name,b->bindings[i].name)){b->bindings[i].used=1;found=1;break;}
 if(!found&&strcmp(name,"times"))die("controller unknown command input: %s",name);
 mantle_controller_typed(b,"input","name",name,path);
}
static void mantle_controller_output(mantle_controller_builder *b,const char *name,const char *path) {
 if(strcmp(name,"workspace")&&strcmp(name,"times")&&strcmp(name,"generated")&&strcmp(name,"descriptors")&&strcmp(name,"descriptor-files"))die("controller unknown mutable output: %s",name);
 mantle_controller_typed(b,"output","name",name,path);
}
static void mantle_controller_runtime(mantle_controller_builder *b,const char *mount,const char *path) {
 mantle_controller_typed(b,"runtime","mount",mount,path);
}
static void mantle_controller_path(mantle_controller_builder *b,const char *path,int writing) {
 struct mantle_paths_operand p;
 if(mantle_paths_map(path,writing,&p)){
  /* Reverse only an already-classified child path through this request's
   * exact Produced roots; opaque source data never enters this conversion. */
  char physical[PATH_MAX],logical[PATH_MAX];const char prefix[]="/outputs/workspace";size_t n=sizeof prefix-1;const char *candidate=path;
  if(!strncmp(path,prefix,n)&&(!path[n]||path[n]=='/')){const char *relative=path+n;if(*relative=='/')relative++;if(mantle_paths_join(physical,mantle_paths_workspace,relative))die("controller canonical workspace path bound");candidate=physical;}
  if(mantle_paths_logical(candidate,logical,sizeof logical)||mantle_paths_map(logical,writing,&p))die("controller rejects original file operand %s: %s",path,strerror(errno));
 }
 if(p.kind==MANTLE_PATHS_WORKSPACE)mantle_controller_typed(b,"workspace","mount",p.logical_root,p.relative);
 else if(p.kind==MANTLE_PATHS_RUNTIME)mantle_controller_runtime(b,p.runtime_root,p.relative);
 else if(p.kind==MANTLE_PATHS_INPUT)mantle_controller_input(b,p.input_name,p.relative);
 else die("controller unknown file operand class");
}
static void mantle_controller_word_end(mantle_controller_builder *b) {
 if(!b->in_word||!b->parts)die("controller empty operand word");putsb(&b->word,"]}");arg(&b->words,done(&b->word));b->in_word=0;
}
static char *mantle_controller_take_word(mantle_controller_builder *b) {if(b->in_word)mantle_controller_word_end(b);if(!b->words.n)die("controller missing completed native environment word");return b->words.v[--b->words.n];}
static void mantle_controller_argument_data(mantle_controller_builder *b,const char *text) {mantle_controller_word_begin(b);mantle_controller_data(b,text);mantle_controller_word_end(b);}
static void mantle_controller_argument_path(mantle_controller_builder *b,const char *path,int writing) {mantle_controller_word_begin(b);mantle_controller_path(b,path,writing);mantle_controller_word_end(b);}
static void mantle_controller_cwd_relative(mantle_controller_builder *b,const char *path) {
 const char *relative=!strncmp(path,"./",2)?path+2:path;if(!*relative)die("controller empty cwd-relative operand");mantle_controller_relative(relative);
 struct mantle_paths_operand target,cwd;if(mantle_paths_map(path,0,&target)||mantle_paths_map(b->request->env.cwd,0,&cwd)||strcmp(target.logical_root,cwd.logical_root))die("controller cwd-relative operand escapes typed cwd root");
 mantle_controller_word_begin(b);mantle_controller_part(b);putsb(&b->word,"{\"kind\":\"cwd_relative\",\"path\":");mantle_controller_json(&b->word,path);putsb(&b->word,"}");mantle_controller_word_end(b);
}
static void mantle_controller_argument_literal(mantle_controller_builder *b,const char *text) {mantle_controller_word_begin(b);mantle_controller_literal(b,text);mantle_controller_word_end(b);}
static char *mantle_controller_data_word(mantle_controller_builder *b,const char *s) {int n=b->words.n;mantle_controller_argument_data(b,s);char *word=b->words.v[n];b->words.n=n;return word;}
static char *mantle_controller_named_word(mantle_controller_builder *b,int output,const char *name,const char *path) {int n=b->words.n;mantle_controller_word_begin(b);if(output)mantle_controller_output(b,name,path);else mantle_controller_input(b,name,path);mantle_controller_word_end(b);char *word=b->words.v[n];b->words.n=n;return word;}
static const char *mantle_controller_basename(const char *s) {const char *p=strrchr(s,'/');return p?p+1:s;}
static int mantle_controller_oneof(const char *name,const char *list) {
 size_t n=strlen(name);for(const char *p=list;*p;){const char *q=strchr(p,' ');size_t m=q?(size_t)(q-p):strlen(p);if(m==n&&!memcmp(p,name,n))return 1;if(!q)break;p=q+1;}return 0;
}
static void mantle_controller_option_path(mantle_controller_builder *b,const char *arg,const char *prefix,int writing) {
 mantle_controller_word_begin(b);mantle_controller_literal(b,prefix);mantle_controller_path(b,arg+strlen(prefix),writing);mantle_controller_word_end(b);
}
/* Option grammars belong to the source command, not to the shape of its bytes.
 * Data containing a slash is still data; a path without one is still a path. */
static int mantle_controller_standard(mantle_controller_builder *b) {
 Args *a=&b->request->command;const char *name=mantle_controller_basename(a->v[0]);
 if(a->n>1&&!strcmp(a->v[1],"--mantle-source-control")) {
  if(a->n<7||!mantle_controller_oneof(name,"make gmake"))die("controller invalid original Make control request");
  mantle_controller_argument_literal(b,a->v[1]);for(int i=2;i<=5;i++){if(!strcmp(a->v[i],"-")&&i!=2&&i!=5)mantle_controller_argument_literal(b,"-");else mantle_controller_argument_path(b,a->v[i],i==5);}mantle_controller_argument_data(b,a->v[6]);
  for(int i=7;i<a->n;i++){const char *s=a->v[i];if(mantle_controller_oneof(s,"-C --directory -f --file --makefile -I --include-dir -o --old-file --assume-old -W --what-if --new-file --assume-new")){mantle_controller_argument_literal(b,s);if(++i==a->n)die("controller original Make option missing path");mantle_controller_argument_path(b,a->v[i],0);}else if(!strncmp(s,"--directory=",12))mantle_controller_option_path(b,s,"--directory=",0);else if(!strncmp(s,"--file=",7))mantle_controller_option_path(b,s,"--file=",0);else if(!strncmp(s,"--makefile=",11))mantle_controller_option_path(b,s,"--makefile=",0);else if(!strncmp(s,"--include-dir=",14))mantle_controller_option_path(b,s,"--include-dir=",0);else if(s[0]=='-'&&strchr("CfIoW",s[1])&&s[2]){char prefix[3]={s[0],s[1],0};mantle_controller_option_path(b,s,prefix,0);}else mantle_controller_argument_data(b,s);}
  return 1;
 }
 if(mantle_controller_oneof(name,"tr expr printf echo basename dirname uname arch nproc true false yes sleep seq")) {
  for(int i=1;i<a->n;i++)mantle_controller_argument_data(b,a->v[i]);return 1;
 }
 if(!strcmp(name,"sed")) {
  int script=0,options=1;
  for(int i=1;i<a->n;i++){const char *s=a->v[i];
   if(options&&!strcmp(s,"--")){options=0;mantle_controller_argument_literal(b,s);continue;}
   if(options&&(!strcmp(s,"-e")||!strcmp(s,"--expression"))){mantle_controller_argument_literal(b,s);if(++i==a->n)die("controller sed missing expression");mantle_controller_argument_data(b,a->v[i]);script=1;continue;}
   if(options&&(!strcmp(s,"-f")||!strcmp(s,"--file"))){mantle_controller_argument_literal(b,s);if(++i==a->n)die("controller sed missing script file");mantle_controller_argument_path(b,a->v[i],0);script=1;continue;}
   if(options&&!strncmp(s,"--expression=",13)){mantle_controller_argument_data(b,s);script=1;continue;}
   if(options&&!strncmp(s,"--file=",7)){mantle_controller_option_path(b,s,"--file=",0);script=1;continue;}
   if(options&&!strncmp(s,"-e",2)&&s[2]){mantle_controller_argument_data(b,s);script=1;continue;}
   if(options&&!strncmp(s,"-f",2)&&s[2]){mantle_controller_option_path(b,s,"-f",0);script=1;continue;}
   if(options&&s[0]=='-'&&s[1]){if(strspn(s+1,"nErzu")!=strlen(s+1)&&strncmp(s,"-i",2)&&strncmp(s,"--in-place",10)&&strcmp(s,"--posix"))die("controller unsupported original sed option: %s",s);mantle_controller_argument_data(b,s);continue;}
   if(!script){mantle_controller_argument_data(b,s);script=1;}else if(!strcmp(s,"-"))mantle_controller_argument_literal(b,s);else mantle_controller_argument_path(b,s,0);
  }return 1;
 }
 if(mantle_controller_oneof(name,"awk gawk mawk")) {
  int program=0,options=1;
  for(int i=1;i<a->n;i++){const char *s=a->v[i];
   if(options&&!strcmp(s,"--")){options=0;mantle_controller_argument_literal(b,s);continue;}
   if(options&&(!strcmp(s,"-f")||!strcmp(s,"--file"))){mantle_controller_argument_literal(b,s);if(++i==a->n)die("controller awk missing program file");mantle_controller_argument_path(b,a->v[i],0);program=1;continue;}
   if(options&&(!strcmp(s,"-v")||!strcmp(s,"-F")||!strcmp(s,"-e")||!strcmp(s,"--source"))){mantle_controller_argument_literal(b,s);if(++i==a->n)die("controller awk missing option data");mantle_controller_argument_data(b,a->v[i]);if(!strcmp(s,"-e")||!strcmp(s,"--source"))program=1;continue;}
   if(options&&!strncmp(s,"-f",2)&&s[2]){mantle_controller_option_path(b,s,"-f",0);program=1;continue;}
   if(options&&s[0]=='-'&&s[1]){if(strncmp(s,"-v",2)&&strncmp(s,"-F",2)&&strcmp(s,"--posix")&&strcmp(s,"--traditional"))die("controller unsupported original awk option: %s",s);mantle_controller_argument_data(b,s);continue;}
   if(!program){mantle_controller_argument_data(b,s);program=1;}else if(strchr(s,'=')||!strcmp(s,"-"))mantle_controller_argument_data(b,s);else mantle_controller_argument_path(b,s,0);
  }return 1;
 }
 if(mantle_controller_oneof(name,"grep egrep fgrep")) {
  int pattern=0,options=1;
  for(int i=1;i<a->n;i++){const char *s=a->v[i];if(options&&!strcmp(s,"--")){options=0;mantle_controller_argument_literal(b,s);continue;}
   if(options&&(!strcmp(s,"-e")||!strcmp(s,"--regexp")||!strcmp(s,"-f")||!strcmp(s,"--file"))){int file=!strcmp(s,"-f")||!strcmp(s,"--file");mantle_controller_argument_literal(b,s);if(++i==a->n)die("controller grep missing option operand");if(file)mantle_controller_argument_path(b,a->v[i],0);else mantle_controller_argument_data(b,a->v[i]);pattern=1;continue;}
   if(options&&mantle_controller_oneof(s,"-A -B -C -m --after-context --before-context --context --max-count")){mantle_controller_argument_literal(b,s);if(++i==a->n)die("controller grep missing numeric option operand");mantle_controller_argument_data(b,a->v[i]);continue;}
   if(options&&!strncmp(s,"--file=",7)){mantle_controller_option_path(b,s,"--file=",0);pattern=1;continue;}
   if(options&&s[0]=='-'&&s[1]){if(!strncmp(s,"-e",2)||!strncmp(s,"--regexp=",9))pattern=1;else if(!strncmp(s,"-f",2)){mantle_controller_option_path(b,s,"-f",0);pattern=1;continue;}else if(strspn(s+1,"EFGivwxclLqnsrRHoahIZz0123456789ABCm")!=strlen(s+1))die("controller unsupported original grep option: %s",s);mantle_controller_argument_data(b,s);continue;}
   if(!pattern){mantle_controller_argument_data(b,s);pattern=1;}else if(!strcmp(s,"-"))mantle_controller_argument_literal(b,s);else mantle_controller_argument_path(b,s,0);
  }return 1;
 }
 if(mantle_controller_oneof(name,"cat cp mv rm mkdir rmdir touch chmod chown ln readlink realpath install ls test [ cmp diff head tail cut sort uniq wc tee od cksum sha256sum md5sum expand unexpand paste join comm truncate du df find")) {
  int options=1,mode=(!strcmp(name,"chmod")||!strcmp(name,"chown")),link_symbolic=0;
  for(int i=1;i<a->n;i++){const char *s=a->v[i];
   if(mode&&!strcmp(name,"chmod")&&s[0]=='-'&&s[1]&&strspn(s+1,"rwxXstugoam,=+-")==strlen(s+1)){mantle_controller_argument_data(b,s);mode=0;continue;}
   if(options&&!strcmp(s,"--")){options=0;mantle_controller_argument_literal(b,s);continue;}
   if(options&&s[0]=='-'&&s[1]) {
    if(!strcmp(name,"ln")&&strchr(s,'s'))link_symbolic=1;
    int path_option=(mantle_controller_oneof(name,"cp mv ln install")&&mantle_controller_oneof(s,"-t --target-directory"))||(!strcmp(name,"sort")&&mantle_controller_oneof(s,"-o --output -T --temporary-directory --files0-from"))||(mantle_controller_oneof(name,"chmod chown touch")&&!strcmp(s,"--reference"));
    int data_option=(mantle_controller_oneof(name,"mkdir install")&&mantle_controller_oneof(s,"-m --mode"))||(mantle_controller_oneof(name,"cp mv ln install")&&mantle_controller_oneof(s,"-S --suffix"))||(mantle_controller_oneof(name,"head tail")&&mantle_controller_oneof(s,"-n --lines -c --bytes"))||(!strcmp(name,"cut")&&mantle_controller_oneof(s,"-b --bytes -c --characters -d --delimiter -f --fields"))||(!strcmp(name,"sort")&&mantle_controller_oneof(s,"-k --key -t --field-separator -S --buffer-size"))||(!strcmp(name,"uniq")&&mantle_controller_oneof(s,"-f --skip-fields -s --skip-chars -w --check-chars"))||(!strcmp(name,"touch")&&mantle_controller_oneof(s,"-d --date -t"))||(!strcmp(name,"truncate")&&mantle_controller_oneof(s,"-s --size"));
    if(path_option){mantle_controller_argument_literal(b,s);if(++i==a->n)die("controller missing original path option operand");mantle_controller_argument_path(b,a->v[i],strcmp(s,"--reference")&&strcmp(s,"--files0-from"));continue;}
    if(data_option){mantle_controller_argument_literal(b,s);if(++i==a->n)die("controller missing original option data");mantle_controller_argument_data(b,a->v[i]);continue;}
    if(!strncmp(s,"--target-directory=",19)){mantle_controller_option_path(b,s,"--target-directory=",1);continue;}
    if(!strncmp(s,"--reference=",12)){mantle_controller_option_path(b,s,"--reference=",0);continue;}
    if(!strcmp(name,"sort")&&!strncmp(s,"--output=",9)){mantle_controller_option_path(b,s,"--output=",1);continue;}
    if(!strcmp(name,"sort")&&!strncmp(s,"--temporary-directory=",22)){mantle_controller_option_path(b,s,"--temporary-directory=",1);continue;}
    if(!strcmp(name,"sort")&&!strncmp(s,"--files0-from=",14)){mantle_controller_option_path(b,s,"--files0-from=",0);continue;}
    if(!strcmp(name,"sort")&&s[0]=='-'&&(s[1]=='o'||s[1]=='T')&&s[2]){char prefix[3]={'-',s[1],0};mantle_controller_option_path(b,s,prefix,1);continue;}
    if(mantle_controller_oneof(name,"cp mv ln install")&&!strncmp(s,"-t",2)&&s[2]){mantle_controller_option_path(b,s,"-t",1);continue;}
    if(!strcmp(name,"touch")&&!strcmp(s,"-r")){mantle_controller_argument_literal(b,s);if(++i==a->n)die("controller touch missing reference file");mantle_controller_argument_path(b,a->v[i],0);continue;}
    if(!strcmp(name,"touch")&&!strncmp(s,"-r",2)&&s[2]){mantle_controller_option_path(b,s,"-r",0);continue;}
    if((!strcmp(name,"cut")&&s[0]=='-'&&strchr("bcdf",s[1])&&s[2])||(!strcmp(name,"sort")&&s[0]=='-'&&strchr("ktS",s[1])&&s[2])||(mantle_controller_oneof(name,"head tail")&&s[0]=='-'&&strchr("nc",s[1])&&s[2])||(mantle_controller_oneof(name,"mkdir install")&&!strncmp(s,"-m",2)&&s[2])||!strncmp(s,"--delimiter=",12)||!strncmp(s,"--field-separator=",18)){mantle_controller_argument_data(b,s);continue;}
    if(strchr(s,'/')||strchr(s,'\\'))die("controller unsupported original path-bearing option %s for %s",s,name);
    mantle_controller_argument_literal(b,s);continue;
   }
   if(mode){mantle_controller_argument_data(b,s);mode=0;continue;}
   if((link_symbolic&&i==a->n-2)||!strcmp(s,"-")||(!strcmp(name,"[")&&!strcmp(s,"]"))){mantle_controller_argument_data(b,s);continue;}
   if(!strcmp(name,"test")||!strcmp(name,"[")){die("controller original test operand requires source expression extension: %s",s);}
   if(!strcmp(name,"find")){die("controller original find expression requires source option extension: %s",s);}
   mantle_controller_argument_path(b,s,mantle_controller_oneof(name,"cp mv rm mkdir rmdir touch chmod chown ln install tee truncate"));
  }return 1;
 }
 return 0;
}

static char *mantle_controller_absolute(const char *path) {
 if(*path=='/')return dup(path);char *cwd=getcwd(NULL,0);if(!cwd)die("controller cwd");char *p=join(cwd,path);free(cwd);return p;
}
static int mantle_controller_exists(const char *root,const char *name) {char *p=join(root,name);struct stat st;int rc=lstat(p,&st);free(p);if(rc&&errno!=ENOENT)die("controller stat %s",name);return !rc;}
static unsigned long mantle_controller_number(const char *s,unsigned long maximum) {char *end;errno=0;unsigned long n=strtoul(s,&end,10);if(errno||!*s||*end||*s=='-'||n>maximum)die("controller invalid bounded integer: %s",s);return n;}
static int mantle_controller_status(const char *root,const char *name) {
 char *p=join(root,name),*s=load(p);free(p);size_t n=strlen(s);if(!n||s[n-1]!='\n')die("controller noncanonical executor status");s[--n]=0;unsigned long status=mantle_controller_number(s,255);char canonical[16];snprintf(canonical,sizeof canonical,"%lu",status);if(strcmp(s,canonical))die("controller noncanonical executor status");free(s);return (int)status;
}
static char *mantle_controller_request_cwd(const char *request) {
 FILE *f=open_record(request,"request","rb");char *magic=read_record(f);if(strcmp(magic,"MANTLE-SHELL-2"))die("controller request version");free(magic);(void)read_integer(f);char *cwd=read_record(f);fclose(f);return cwd;
}
static char *mantle_controller_scratch(void) {char *cwd=getcwd(NULL,0),*pattern=join(cwd,".mantle-controller-XXXXXX");free(cwd);if(!mkdtemp(pattern))die("controller scratch directory");return pattern;}
/* Source PATH lookup grants no execution authority: the bytes must subsequently
 * match an individually Produced image or an executed generated catalog entry. */
static char *mantle_controller_resolve(Request *r) {
 const char *name=r->command.v[0];struct mantle_paths_operand p;
 if(strchr(name,'/')){if(mantle_paths_map(name,0,&p))die("controller original executable path unavailable: %s",name);return dup(p.physical);}
 const char *path=get(&r->env,"PATH");if(!path)die("controller original command has no source PATH: %s",name);
 for(const char *s=path;;){const char *end=strchr(s,':');size_t n=end?(size_t)(end-s):strlen(s);char *directory=alloc(n+1);memcpy(directory,s,n);char *candidate=n?join(directory,name):dup(name);free(directory);if(!mantle_paths_map(candidate,0,&p)){struct stat st;if(!stat(p.physical,&st)&&S_ISREG(st.st_mode)&&(st.st_mode&0111)){free(candidate);return dup(p.physical);}}free(candidate);if(!end)break;s=end+1;}
 die("controller original source PATH does not resolve command: %s",name);return NULL;
}
static void mantle_controller_load(const char *checkpoint,Request *r,const char *roots,char **scratch) {
 char *request=join(checkpoint,"request");*scratch=mantle_controller_scratch();if(chdir(*scratch))die("controller scratch cwd");
 if(!roots&&mantle_controller_exists(checkpoint,"controller-roots"))roots=join(checkpoint,"controller-roots");
 /* Materialize the authentic snapshot before opening any logical root anchor.
  * Decoding does not enter cwd; initialization enters its original logical cwd. */
 read_request_data(request,r);free(request);
 if(r->active){if(!roots)die("controller active request requires original logical roots");if(*r->env.cwd!='/')die("controller request cwd must retain original logical absolute path");if(mantle_paths_init(r->workspace,roots,r->env.cwd))die("controller logical roots: %s",strerror(errno));}
 if(r->active&&mantle_controller_exists(checkpoint,"times")){char *times=join(checkpoint,"times");mantle_workspace_restore(r->workspace,times);free(times);}
}
static void mantle_controller_empty_file(const char *path) {FILE *f=fopen(path,"wb");if(!f||fclose(f))die("controller empty protocol output");}
static void mantle_controller_stdin(Request *r,const char *path) {FILE *out=fopen(path,"wb");if(!out)die("controller stdin snapshot");if(r->env.fd[0]){long at=ftell(r->env.fd[0]);if(at<0)die("controller stdin position");char proc[64];snprintf(proc,sizeof proc,"/proc/self/fd/%d",fileno(r->env.fd[0]));FILE *input=fopen(proc,"rb");if(!input||fseek(input,at,SEEK_SET))die("controller stdin observation");transfer(input,out);if(fclose(input))die("controller stdin observation close");}if(fclose(out))die("controller stdin close");}
static char *mantle_controller_catalog_key(Request *r) {struct mantle_paths_operand p;if(mantle_paths_map(r->command.v[0],0,&p)||p.kind!=MANTLE_PATHS_WORKSPACE)return NULL;return dup(p.workspace_relative);}
static char *mantle_controller_catalog_producer(const char *checkpoint,const char *key) {
 if(!key||!mantle_controller_exists(checkpoint,"generated-catalog"))return NULL;FILE *f=open_record(checkpoint,"generated-catalog","rb");char *magic=read_record(f);if(strcmp(magic,"MANTLE-GENERATED-CATALOG-1"))die("controller generated catalog version");free(magic);char *found=NULL;
 for(;;){int c=fgetc(f);if(c==EOF)break;ungetc(c,f);char *path=read_record(f),*producer=read_record(f);mantle_controller_relative(path);if(!strcmp(path,key)){if(found)die("controller duplicate generated authority");found=dup(producer);}free(path);free(producer);}if(ferror(f)||fclose(f))die("controller generated catalog read");return found;
}
static int mantle_controller_identity(const char *checkpoint,Request *r,int argc,char **argv) {
 if(argc<1)die("controller missing original image identity");const char *mode=argv[0],*name=mantle_controller_basename(r->command.v[0]);
 if(!strcmp(mode,"fixed-image")||!strcmp(mode,"adapted-image")){if(argc!=3)die("controller fixed image identity layout");if(strcmp(name,argv[1]))return 0;char *actual=mantle_controller_resolve(r);int same=files_equal(actual,argv[2]);free(actual);return same;}
 if(!strcmp(mode,"inventory")){if(argc<2)die("controller inventory identity layout");unsigned long n=mantle_controller_number(argv[1],65536);if(argc!=2+2*(int)n)die("controller inventory image count");char *actual=NULL;for(unsigned long i=0;i<n;i++)if(!strcmp(name,argv[2+2*i])){if(!actual)actual=mantle_controller_resolve(r);if(files_equal(actual,argv[3+2*i])){free(actual);return 1;}}free(actual);return 0;}
 if(!strcmp(mode,"generated")){if(argc<2)die("controller generated identity layout");unsigned long n=mantle_controller_number(argv[1],65536);if(argc!=2+(int)n)die("controller generated producer count");char *key=mantle_controller_catalog_key(r),*producer=mantle_controller_catalog_producer(checkpoint,key);int selected=0;for(unsigned long i=0;producer&&i<n;i++)if(!strcmp(producer,argv[2+i]))selected=1;if(selected){char *tree=join(checkpoint,"generated"),*image=join(tree,key),*actual=mantle_controller_resolve(r);if(!files_equal(image,actual))die("controller generated executable changed since protected producer");free(tree);free(image);free(actual);}free(key);free(producer);return selected;}
 die("controller unknown original image identity: %s",mode);return 0;
}

typedef struct mantle_controller_environment {const char *key;char *word;} mantle_controller_environment;
static int mantle_controller_environment_compare(const void *a,const void *b) {return strcmp(((const mantle_controller_environment *)a)->key,((const mantle_controller_environment *)b)->key);}
static int mantle_controller_root_compare(const void *a,const void *b) {return strcmp((*(const struct mantle_paths_root *const *)a)->logical,(*(const struct mantle_paths_root *const *)b)->logical);}
static int mantle_controller_name_compare(const void *a,const void *b) {return strcmp(*(const char *const *)a,*(const char *const *)b);}
static void mantle_controller_readonly_inputs(mantle_controller_builder *b,Buf *json) {
 const char *names[MANTLE_PATHS_MAX_ROOTS];size_t count=0;
 for(size_t i=0;i<mantle_paths_count;i++)if(mantle_paths_roots[i].kind==MANTLE_PATHS_INPUT)names[count++]=mantle_paths_roots[i].target;
 if(!count)return;qsort(names,count,sizeof *names,mantle_controller_name_compare);putsb(json,",\"readonly_input_roots\":[");
 int emitted=0;for(size_t i=0;i<count;i++){if(i&&!strcmp(names[i-1],names[i]))continue;int found=0;for(int j=0;j<b->binding_count;j++)if(!strcmp(names[i],b->bindings[j].name)){b->bindings[j].used=1;found=1;break;}if(!found)die("controller readonly source root has no declared command Tree input");if(emitted++)putsb(json,",");mantle_controller_json(json,names[i]);}putsb(json,"]");
}
static char *mantle_controller_path_word(mantle_controller_builder *b,const char *path) {int n=b->words.n;mantle_controller_argument_path(b,path,0);char *word=b->words.v[n];b->words.n=n;return word;}
static char *mantle_controller_image_word(mantle_controller_builder *b) {
 char *physical=mantle_controller_resolve(b->request),logical[PATH_MAX];
 if(mantle_paths_logical(physical,logical,sizeof logical))die("controller resolved image lacks a declared logical root");free(physical);
 struct mantle_paths_operand image;if(mantle_paths_map(logical,0,&image))die("controller resolved image path cannot be classified");
 /* A mutable generated workspace is not readonly installed-image authority.
  * The runtime still checks this member against the exact Produced image. */
 if(image.kind==MANTLE_PATHS_WORKSPACE)return NULL;
 if(image.kind!=MANTLE_PATHS_INPUT&&image.kind!=MANTLE_PATHS_RUNTIME)die("controller installed image requires readonly source Tree");
 return mantle_controller_path_word(b,logical);
}
static int mantle_controller_descriptor_alias(Request *r,int index) {
 if(!r->env.fd[index]||r->env.fd_cloexec[index])return -1;
 for(int i=0;i<index;i++)if(!r->env.fd_cloexec[i]&&r->env.fd[i]==r->env.fd[index])return i;
 return index;
}
static void mantle_controller_descriptor_files(Request *r,const char *directory) {
 mkdir_checked(directory);
 for(int i=0;i<64;i++)if(mantle_controller_descriptor_alias(r,i)==i&&!mantle_descriptor_null(r->env.fd[i])) {
  char name[32];snprintf(name,sizeof name,"%d",i);char *path=join(directory,name);long at;
  stream_file(r->env.fd[i],path,&at);if(at!=r->positions[i])die("controller source descriptor position drift");free(path);
 }
}
static void mantle_controller_descriptors(mantle_controller_builder *b,Buf *json) {
 putsb(json,",\"descriptors\":{\"slots\":[");
 for(int i=0;i<64;i++) {
  if(i)putsb(json,",");int alias=mantle_controller_descriptor_alias(b->request,i);
  if(alias<0){putsb(json,"null");continue;}
  Env *env=&b->request->env;int flags=env->fd_flags[i];char values[160];
  if(flags!=env->fd_flags[alias]||b->request->positions[i]!=b->request->positions[alias])die("controller source descriptor alias state differs");
  putsb(json,"{\"target\":{");
  if(mantle_descriptor_null(env->fd[i]))putsb(json,"\"kind\":\"null-device\"");
  else {const char *member=b->request->descriptor_workspace[i];char name[32];snprintf(name,sizeof name,"%d",alias);char *word=mantle_controller_named_word(b,1,member&&*member?"workspace":"descriptor-files",member&&*member?member:name);putsb(json,"\"kind\":\"file\",\"path\":");putsb(json,word);free(word);}
  putsb(json,"},\"access\":");mantle_controller_json(json,(flags&O_ACCMODE)==O_RDONLY?"read":(flags&O_ACCMODE)==O_WRONLY?"write":"read-write");
  snprintf(values,sizeof values,",\"flags\":%u,\"offset\":%lu,\"alias\":%d}",(unsigned)(flags&(O_APPEND|O_NONBLOCK)),(unsigned long)b->request->positions[i],alias);putsb(json,values);
 }
 putsb(json,"]}");
}
static void mantle_controller_invocation(mantle_controller_builder *b,const char *path) {
 mantle_controller_environment *env=NULL;size_t count=0;
 for(Var *v=b->request->env.vars;v;v=v->next)if(v->exported&&v->value){
  if(!strcmp(v->key,"MANTLE_STAGE_X_SED_BRIDGE_FSIZE"))continue;
  if(!*v->key||strspn(v->key,"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_")!=strlen(v->key))die("controller invalid source environment key: %s",v->key);
  char *word=mantle_controller_environment_hook?mantle_controller_environment_hook(b,v->key,v->value):NULL;if(!word)word=mantle_controller_data_word(b,v->value);env=realloc(env,(count+1)*sizeof *env);if(!env)die("controller environment allocation");env[count++]=(mantle_controller_environment){v->key,word};
 }
 struct mantle_paths_operand cwd;if(mantle_paths_map(b->request->env.cwd,0,&cwd))die("controller cwd is not a declared logical Tree");
 char *cwd_word=mantle_controller_path_word(b,b->request->env.cwd);
 char *argv0_word=mantle_controller_data_word(b,b->request->command.argv0);
 char *image_word=mantle_controller_image_word(b);
 qsort(env,count,sizeof *env,mantle_controller_environment_compare);Buf json={0};putsb(&json,"{\"arguments\":[");for(int i=0;i<b->words.n;i++){if(i)putsb(&json,",");putsb(&json,b->words.v[i]);}putsb(&json,"],\"environment\":{");for(size_t i=0;i<count;i++){if(i&&!strcmp(env[i-1].key,env[i].key))die("controller duplicate source environment key");if(i)putsb(&json,",");mantle_controller_json(&json,env[i].key);putsb(&json,":");putsb(&json,env[i].word);}putsb(&json,"},\"cwd\":");putsb(&json,cwd_word);
 putsb(&json,",\"argv0\":");putsb(&json,argv0_word);free(argv0_word);
 if(image_word){putsb(&json,",\"executable_path\":");putsb(&json,image_word);free(image_word);}
 const struct mantle_paths_root *roots[MANTLE_PATHS_MAX_ROOTS];size_t root_count=0;for(size_t i=0;i<mantle_paths_count;i++)if(mantle_paths_roots[i].kind==MANTLE_PATHS_WORKSPACE)roots[root_count++]=&mantle_paths_roots[i];if(!root_count)die("controller missing logical workspace roots");qsort(roots,root_count,sizeof *roots,mantle_controller_root_compare);
 putsb(&json,",\"workspace_mounts\":{");for(size_t i=0;i<root_count;i++){if(i)putsb(&json,",");mantle_controller_json(&json,roots[i]->logical);putsb(&json,":{\"output\":\"workspace\",\"path\":");mantle_controller_json(&json,roots[i]->target);putsb(&json,"}");}putsb(&json,"}");
 mantle_controller_readonly_inputs(b,&json);
 Buf attributes={0};int fields=0;
 const char *file_size=get(&b->request->env,"MANTLE_STAGE_X_SED_BRIDGE_FSIZE");if(file_size){if(strcmp(file_size,"8388608"))die("controller original sed bridge file-size scope drift");putsb(&attributes,"\"file_size_bytes_max\":8388608");fields++;}
 if(!b->request->env.mask_valid||b->request->env.creation_mask>0777)die("controller missing actual source creation mask");if(fields++)putsb(&attributes,",");char mask[64];snprintf(mask,sizeof mask,"\"umask\":%u",(unsigned)b->request->env.creation_mask);putsb(&attributes,mask);
 if(mantle_controller_process_attributes_hook)mantle_controller_process_attributes_hook(b,&attributes,&fields);
 if(fields){putsb(&json,",\"process_attributes\":{");putsb(&json,attributes.s);putsb(&json,"}");}free(attributes.s);if(!b->private_descriptors)mantle_controller_descriptors(b,&json);putsb(&json,"}");
 FILE *f=fopen(path,"wb");if(!f||fwrite(json.s,1,json.n,f)!=json.n||fclose(f))die("controller invocation write");free(json.s);free(cwd_word);for(size_t i=0;i<count;i++)free(env[i].word);free(env);
}
static int mantle_controller_prepare(int argc,char **argv) {
 if(argc<12)die("controller prepare usage");char *checkpoint=mantle_controller_absolute(argv[2]);char *outputs[6];for(int i=0;i<6;i++)outputs[i]=mantle_controller_absolute(argv[4+i]);int separator=10;while(separator<argc&&strcmp(argv[separator],"--"))separator++;if(separator==argc)die("controller prepare missing identity separator");
 const char *roots=getenv("MANTLE_CONTROLLER_ROOTS");mantle_controller_binding *bindings=alloc((separator-10)*sizeof *bindings);int binding_count=0,command_inputs=0;
 for(int i=10;i<separator;){if(!strcmp(argv[i],"--command-inputs")){if(command_inputs)die("controller duplicate command input marker");command_inputs=1;i++;continue;}if(!strcmp(argv[i],"--runtime-inputs")){if(!command_inputs||i+1>=separator)die("controller runtime input layout");unsigned long count=mantle_controller_number(argv[i+1],65536);i+=2;if(count>(unsigned long)(separator-i))die("controller runtime input count");for(unsigned long k=0;k<count;k++){int found=0;for(int j=0;j<binding_count;j++)if(!strcmp(bindings[j].name,argv[i+k])){bindings[j].used=1;found=1;break;}if(!found)die("controller runtime input has no actual command binding");}i+=count;continue;}if(i+1>=separator)die("controller command input binding layout");for(int j=0;j<binding_count;j++)if(!strcmp(bindings[j].name,argv[i]))die("controller duplicate command input binding");bindings[binding_count++]=(mantle_controller_binding){argv[i],mantle_controller_absolute(argv[i+1]),!command_inputs};i+=2;}
 if(!command_inputs)die("controller missing command input marker");if(roots)roots=mantle_controller_absolute(roots);else if(mantle_controller_exists(checkpoint,"controller-roots"))roots=join(checkpoint,"controller-roots");Request r;char *scratch;mantle_controller_load(checkpoint,&r,roots,&scratch);
 int selected=r.active&&mantle_controller_identity(checkpoint,&r,argc-separator-1,argv+separator+1);
 mkdir_checked(outputs[1]);mkdir_checked(outputs[5]);
 if(!selected){for(int i=0;i<5;i++)if(i!=1)mantle_controller_empty_file(outputs[i]);return 1;}
 if(!roots)die("controller active request has no Produced logical roots manifest");
 mantle_controller_builder b;memset(&b,0,sizeof b);b.request=&r;b.bindings=bindings;b.binding_count=binding_count;b.data=fopen(outputs[4],"wb");if(!b.data)die("controller data output");
 int handled=mantle_controller_prepare_hook?mantle_controller_prepare_hook(argv[3],&r,&b):0;if(handled<0)die("controller native source prepare rejected original %s in slot %s",r.command.v[0],argv[3]);if(!handled&&!mantle_controller_standard(&b))die("controller unsupported original command %s in slot %s; source prepare extension required",r.command.v[0],argv[3]);if(b.in_word)die("controller native prepare left unfinished word");
 if(b.private_descriptors){if(r.command.n<2||strcmp(r.command.v[1],"--mantle-source-control"))die("private descriptor transport requires original source control mode");for(int i=3;i<64;i++)if(r.env.fd[i])die("private source control transport retains external descriptor authority");}
 copy_tree(r.workspace,outputs[1]);mantle_controller_stdin(&r,outputs[2]);mantle_controller_descriptor_files(&r,outputs[5]);mantle_workspace_capture(r.workspace,outputs[3]);mantle_controller_invocation(&b,outputs[0]);for(int i=0;i<b.binding_count;i++)if(!b.bindings[i].used)die("controller unused declared command input: %s",b.bindings[i].name);if(fclose(b.data))die("controller data close");return 0;
}

/* ObservePhase uses canonical serde JSON. Parse the actual inventory, rather
 * than looking for a substring that could appear in an output name. */
typedef struct mantle_controller_json_reader {const unsigned char *p;} mantle_controller_json_reader;
static void mantle_controller_expect(mantle_controller_json_reader *j,const char *text) {size_t n=strlen(text);if(strncmp((const char *)j->p,text,n))die("controller malformed phase observation");j->p+=n;}
static char *mantle_controller_json_string(mantle_controller_json_reader *j) {
 if(*j->p++!='"')die("controller phase string");Buf b={0};while(*j->p&&*j->p!='"'){unsigned char c=*j->p++;if(c<32)die("controller phase string control");if(c=='\\'){c=*j->p++;switch(c){case '"':case '\\':case '/':break;case 'b':c='\b';break;case 'f':c='\f';break;case 'n':c='\n';break;case 'r':c='\r';break;case 't':c='\t';break;default:die("controller unsupported escaped phase identifier");}}putn(&b,(const char *)&c,1);}if(*j->p++!='"')die("controller unterminated phase string");return done(&b);
}
static int mantle_controller_observation(const char *root) {
 char *path=join(root,"outcome.json"),*text=load(path);free(path);mantle_controller_json_reader j={(unsigned char *)text};mantle_controller_expect(&j,"{\"kind\":");char *kind=mantle_controller_json_string(&j);int executed=!strcmp(kind,"executed");if(executed){mantle_controller_expect(&j,",\"output_digests\":{");int count=0;char *previous=NULL;while(*j.p!='}'){if(count++)mantle_controller_expect(&j,",");char *name=mantle_controller_json_string(&j);if(previous&&strcmp(previous,name)>=0)die("controller phase output order");free(previous);previous=dup(name);mantle_controller_relative(name);if(strchr(name,'/'))die("controller phase output id");mantle_controller_expect(&j,":");char *digest=mantle_controller_json_string(&j);if(strlen(digest)!=64||strspn(digest,"0123456789abcdef")!=64)die("controller phase output digest");char *outputs=join(root,"outputs"),*member=join(outputs,name);struct stat st;if(lstat(member,&st))die("controller missing actual phase output: %s",name);free(outputs);free(member);free(name);free(digest);}free(previous);mantle_controller_expect(&j,"}}");if(!count)die("controller empty executed inventory");}
 else if(!strcmp(kind,"skipped")){mantle_controller_expect(&j,",\"guard_codes\":[");int count=0,previous=-1;while(*j.p!=']'){if(count++)mantle_controller_expect(&j,",");if(*j.p<'0'||*j.p>'9')die("controller skipped guard code");unsigned long n=0;do{n=n*10+*j.p++-'0';if(n>255)die("controller skipped guard code range");}while(*j.p>='0'&&*j.p<='9');if((int)n<=previous)die("controller skipped guard code order");previous=n;}if(!count||count>8)die("controller skipped guard count");mantle_controller_expect(&j,"]}");if(mantle_controller_exists(root,"outputs"))die("controller skipped phase has outputs");}
 else die("controller unknown phase observation kind: %s",kind);
 if(*j.p)die("controller trailing phase observation bytes");free(kind);free(text);return executed;
}
static void mantle_controller_catalog_copy(const char *checkpoint,const char *output) {
 if(mantle_controller_exists(checkpoint,"generated")){char *from=join(checkpoint,"generated"),*to=join(output,"generated");copy_tree(from,to);free(from);free(to);}
 if(mantle_controller_exists(checkpoint,"generated-catalog")){char *from=join(checkpoint,"generated-catalog"),*to=join(output,"generated-catalog");binary_copy(from,to);free(from);free(to);}
}
static int mantle_controller_string_compare(const void *a,const void *b) {return strcmp(*(char *const *)a,*(char *const *)b);}
static void mantle_controller_catalog_add(const char *source,const char *relative,const char *destination,FILE *catalog,const char *producer) {
 char *path=*relative?join(source,relative):dup(source);struct stat st;if(lstat(path,&st))die("controller generated product stat");
 if(S_ISDIR(st.st_mode)){DIR *d=opendir(path);if(!d)die("controller generated directory");Args names={0};struct dirent *entry;while((entry=readdir(d))){if(!strcmp(entry->d_name,".")||!strcmp(entry->d_name,".."))continue;arg(&names,dup(entry->d_name));}if(closedir(d))die("controller generated directory close");qsort(names.v,names.n,sizeof *names.v,mantle_controller_string_compare);for(int i=0;i<names.n;i++){char *child=*relative?join(relative,names.v[i]):dup(names.v[i]);mantle_controller_catalog_add(source,child,destination,catalog,producer);free(child);free(names.v[i]);}free(names.v);}
 else {if(!S_ISREG(st.st_mode))die("controller isolated generated product is not regular: %s",relative);mantle_controller_relative(relative);char *to=join(destination,relative),*parent=dup(to);for(char *p=parent+1;*p;p++)if(*p=='/'){*p=0;mkdir_checked(parent);*p='/';}free(parent);binary_copy(path,to);if(chmod(to,st.st_mode&0777))die("controller generated product mode");if(st.st_mode&0111){record(catalog,relative);record(catalog,producer);}free(to);}free(path);
}
static void mantle_controller_catalog_update(const char *checkpoint,const char *next,const char *products,const char *producer) {
 mantle_controller_catalog_copy(checkpoint,next);char *directory=join(next,"generated");mkdir_checked(directory);
 /* Rebuild the path->latest real producer catalog; a later producer may replace
  * a prior executable, but a seeded workspace member never enters this tree. */
 char *catalog_path=join(next,"generated-catalog"),*temporary=join(next,"generated-catalog-new");FILE *fresh=fopen(temporary,"wb");if(!fresh)die("controller generated catalog create");record(fresh,"MANTLE-GENERATED-CATALOG-1");
 if(mantle_controller_exists(next,"generated-catalog")){FILE *old=fopen(catalog_path,"rb");if(!old)die("controller generated catalog open");char *magic=read_record(old);if(strcmp(magic,"MANTLE-GENERATED-CATALOG-1"))die("controller generated catalog version");free(magic);for(;;){int c=fgetc(old);if(c==EOF)break;ungetc(c,old);char *key=read_record(old),*owner=read_record(old);mantle_controller_relative(key);char *replacement=join(products,key);struct stat st;if(lstat(replacement,&st)){if(errno!=ENOENT)die("controller generated replacement stat");record(fresh,key);record(fresh,owner);}free(replacement);free(key);free(owner);}if(ferror(old)||fclose(old))die("controller generated catalog close");}
 mantle_controller_catalog_add(products,"",directory,fresh,producer);if(fclose(fresh)||rename(temporary,catalog_path))die("controller generated catalog publish");free(directory);free(catalog_path);free(temporary);
}
static void mantle_controller_acknowledge_descriptors(Request *r,const char *products) {
 FILE *f=open_record(products,"descriptors","rb");char *magic=read_record(f);if(strcmp(magic,"MANTLE-COMMAND-DESCRIPTORS-2"))die("controller actual descriptor metadata version");free(magic);
 long long positions[64],flags[64];
 for(int i=0;i<64;i++) {
  if(read_integer(f)!=i)die("controller actual descriptor metadata order");positions[i]=read_integer(f);flags[i]=read_integer(f);int alias=mantle_controller_descriptor_alias(r,i);
  if(alias<0){if(positions[i]!=-1||flags[i]!=-1)die("controller closed child descriptor unexpectedly inherited");continue;}
  if(positions[i]<0||positions[i]>LONG_MAX||flags[i]<0||(flags[i]&~(O_APPEND|O_NONBLOCK)))die("controller actual descriptor state");
  if(alias<i&&(positions[i]!=positions[alias]||flags[i]!=flags[alias]))die("controller actual aliases diverge");
 }
 if(fgetc(f)!=EOF||ferror(f)||fclose(f))die("controller actual descriptor metadata trailing bytes");
 char *directory=join(products,"descriptor-files");
 for(int i=0;i<64;i++)if(mantle_controller_descriptor_alias(r,i)==i) {
  FILE *stream=r->env.fd[i];
  if(!mantle_descriptor_null(stream)) {
   char name[32],proc[64];snprintf(name,sizeof name,"%d",i);char *path=join(directory,name);
   if(r->descriptor_workspace[i]&&*r->descriptor_workspace[i]){snprintf(proc,sizeof proc,"/proc/self/fd/%d",fileno(stream));if(!files_equal(proc,path))die("controller actual workspace descriptor bytes differ");}
   else{FILE *input=fopen(path,"rb");if(!input||fflush(stream)||fseek(stream,0,SEEK_SET)||ftruncate(fileno(stream),0))die("controller actual descriptor backing restore");transfer(input,stream);if(fclose(input)||fflush(stream))die("controller actual descriptor backing close");}
   free(path);
  }
  int actual=fcntl(fileno(stream),F_GETFL);if(actual<0||fcntl(fileno(stream),F_SETFL,(actual&~(O_APPEND|O_NONBLOCK))|flags[i]))die("controller actual descriptor flags restore");
  if(fseek(stream,(long)positions[i],SEEK_SET))die("controller actual descriptor position restore");
  for(int j=0;j<64;j++)if(r->env.fd[j]==stream){r->positions[j]=positions[i];r->env.fd_flags[j]=(r->env.fd_flags[j]&O_ACCMODE)|flags[i];}
 }
 free(directory);
}
/* The source controller's predicate is not an original command status. */
static int mantle_controller_predicate(const char *checkpoint) {
 if(text_is(checkpoint,"state","paused\n"))return 0;
 if(text_is(checkpoint,"state","done\n"))return 1;
 die("controller callback did not produce paused or completed source state");return 2;
}
static int mantle_controller_start(int argc,char **argv,int (*restart)(int,char **)) {
 if(argc<5||strcmp(argv[3],"--")||!restart)die("controller start usage or callback");
 char *checkpoint=mantle_controller_absolute(argv[2]);int source_argc=argc-3;
 char **source_argv=alloc((source_argc+1)*sizeof *source_argv);source_argv[0]=argv[0];
 for(int i=1;i<source_argc;i++)source_argv[i]=argv[i+3];
 (void)restart(source_argc,source_argv);
 return mantle_controller_predicate(checkpoint);
}
static int mantle_controller_resume(int argc,char **argv,int (*restart)(int,char **)) {
 if(argc<9)die("controller resume usage");char *checkpoint=mantle_controller_absolute(argv[2]),*next=mantle_controller_absolute(argv[3]);unsigned long indices[3];for(int i=0;i<3;i++)indices[i]=mantle_controller_number(argv[4+i],1048576);if(indices[0]==indices[1]||indices[0]==indices[2]||indices[1]==indices[2])die("controller duplicate source restart positions");unsigned long count=mantle_controller_number(argv[7],65536);int separator=8+3*(int)count;if(separator>=argc||strcmp(argv[separator],"--"))die("controller resume observation layout");
 int selected=-1;char *products=NULL;for(unsigned long i=0;i<count;i++){int at=8+3*(int)i;if(strcmp(argv[at+1],"0")&&strcmp(argv[at+1],"1"))die("controller generated observation flag");if(mantle_controller_observation(argv[at+2])){if(selected>=0)die("controller multiple protected commands executed for one original request");selected=i;products=join(argv[at+2],"outputs");}}
 char *request_path=join(checkpoint,"request");int active=text_is(request_path,"state","active\n");if(!active&&!text_is(request_path,"state","not-active\n"))die("controller unknown immutable request state");free(request_path);mkdir_checked(next);
 if(!active){if(selected>=0)die("controller protected phase executed without active original request");if(!text_is(checkpoint,"state","done\n"))die("controller inactive checkpoint is not source completion");copy_tree(checkpoint,next);return 1;}
 if(selected<0)die("controller no protected phase matched active original request");
 const char *roots=getenv("MANTLE_CONTROLLER_ROOTS");char *roots_absolute=roots?mantle_controller_absolute(roots):NULL;Request r;char *scratch;mantle_controller_load(checkpoint,&r,roots_absolute,&scratch);if(!r.active)die("controller lost active request");
 if(mantle_controller_source_scope_resume_hook)mantle_controller_source_scope_resume_hook(&r.env);
 char *workspace=join(products,"workspace"),*times=join(products,"times");synchronize_tree(workspace,r.workspace);mantle_workspace_restore(r.workspace,times);mantle_controller_acknowledge_descriptors(&r,products);char *result=join(scratch,"acknowledged-result");int status=mantle_controller_status(products,"exit-status");write_result(result,&r,status,NULL,NULL);char *result_times=join(result,"times");binary_copy(times,result_times);free(result_times);
 int at=8+3*selected;if(!strcmp(argv[at+1],"1")){char *generated=join(products,"generated");mantle_controller_catalog_update(checkpoint,next,generated,argv[at]);free(generated);}else mantle_controller_catalog_copy(checkpoint,next);
 char *next_times=join(next,"times");binary_copy(times,next_times);free(next_times);
 int source_argc=argc-separator;char **source_argv=alloc((source_argc+1)*sizeof *source_argv);source_argv[0]=argv[0];for(int i=1;i<source_argc;i++)source_argv[i]=argv[separator+i];for(int i=0;i<3;i++)if(indices[i]>=(unsigned long)(source_argc-1))die("controller source restart index outside original CLI");source_argv[indices[0]+1]=checkpoint;source_argv[indices[1]+1]=result;source_argv[indices[2]+1]=next;
 if(!restart)die("controller missing original source restart callback");if(mantle_controller_source_scope_resume_hook)mantle_controller_source_scope_resume_hook(&r.env);(void)restart(source_argc,source_argv);return mantle_controller_predicate(next);
}
static int mantle_controller_executable(int argc,char **argv) {
 if(argc<6)die("controller executable usage");const char *checkpoint=argv[2];char *request=join(checkpoint,"request");if(!text_is(request,"state","active\n"))die("controller executable request inactive");free(request);
 if(!strcmp(argv[5],"inventory")){/* The same immutable request/image comparison used by prepare. */
  if(argc<7)die("controller executable inventory usage");unsigned long count=mantle_controller_number(argv[6],65536);if(argc!=7+2*(int)count)die("controller executable inventory layout");const char *roots=getenv("MANTLE_CONTROLLER_ROOTS");char *roots_absolute=roots?mantle_controller_absolute(roots):NULL,*scratch,*destination=mantle_controller_absolute(argv[4]);Request r;mantle_controller_load(checkpoint,&r,roots_absolute,&scratch);if(!r.active)die("controller executable request inactive");const char *name=mantle_controller_basename(r.command.v[0]);char *actual=mantle_controller_resolve(&r);for(unsigned long i=0;i<count;i++)if(!strcmp(name,argv[7+2*i])&&files_equal(actual,argv[8+2*i])){binary_copy(argv[8+2*i],destination);if(chmod(destination,0755))die("controller inventory image mode");free(actual);return 0;}die("controller executable does not match an individually Produced original inventory image");
 }
 FILE *f;char *request_root=join(checkpoint,"request");f=open_record(request_root,"request","rb");free(request_root);char *magic=read_record(f);if(strcmp(magic,"MANTLE-SHELL-2"))die("controller executable request version");free(magic);(void)read_integer(f);char *cwd=read_record(f);free(read_record(f));free(read_record(f));long long n=read_integer(f);if(n<1)die("controller executable empty request");char *command=read_record(f);fclose(f);
 if(strcmp(argv[5],"generated")||argc<7)die("controller generated executable identity layout");unsigned long allowed_count=mantle_controller_number(argv[6],65536);if(argc!=7+(int)allowed_count)die("controller generated executable producer count");const char *roots=getenv("MANTLE_CONTROLLER_ROOTS");if(!roots&&mantle_controller_exists(checkpoint,"controller-roots"))roots=join(checkpoint,"controller-roots");if(!roots)die("controller generated executable requires logical roots manifest");char *workspace=join(checkpoint,"workspace");if(mantle_paths_init(workspace,roots,cwd))die("controller generated executable logical roots");struct mantle_paths_operand p;if(mantle_paths_map(command,0,&p)||p.kind!=MANTLE_PATHS_WORKSPACE)die("controller executable not an isolated generated path");char *producer=mantle_controller_catalog_producer(checkpoint,p.workspace_relative);int allowed=0;for(int i=7;i<argc;i++)if(producer&&!strcmp(producer,argv[i]))allowed=1;if(!allowed)die("controller generated executable lacks an executed allowed producer");char *directory=join(checkpoint,"generated"),*image=join(directory,p.workspace_relative);if(!files_equal(image,p.physical))die("controller generated executable differs from protected compiler output");binary_copy(image,argv[4]);if(chmod(argv[4],0755))die("controller generated executable mode");free(cwd);free(command);free(producer);free(workspace);free(directory);free(image);return 0;
}
static int mantle_controller_finish(int argc,char **argv) {
 if(argc!=7)die("controller finish usage");const char *checkpoint=argv[2];if(!text_is(checkpoint,"state","done\n"))die("controller original source command bound exhausted before completion");char *workspace=join(checkpoint,"workspace"),*times=join(checkpoint,"times");copy_tree(workspace,argv[3]);if(!mantle_controller_exists(checkpoint,"times"))die("controller source completion missing genuine workspace timestamps");binary_copy(times,argv[4]);mkdir_checked(argv[5]);if(mantle_controller_exists(checkpoint,"generated")){char *generated=join(checkpoint,"generated");copy_tree(generated,argv[5]);free(generated);}copy_tree(checkpoint,argv[6]);free(workspace);free(times);return mantle_controller_status(checkpoint,"exit-status");
}
int mantle_controller_dispatch(int argc,char **argv,int (*restart_callback)(int,char **)) {
 if(argc<2||strncmp(argv[1],"--mantle-controller-",20))return -1;
 if(!strcmp(argv[1],"--mantle-controller-start"))return mantle_controller_start(argc,argv,restart_callback);
 if(!strcmp(argv[1],"--mantle-controller-prepare"))return mantle_controller_prepare(argc,argv);
 if(!strcmp(argv[1],"--mantle-controller-resume"))return mantle_controller_resume(argc,argv,restart_callback);
 if(!strcmp(argv[1],"--mantle-controller-executable"))return mantle_controller_executable(argc,argv);
 if(!strcmp(argv[1],"--mantle-controller-finish"))return mantle_controller_finish(argc,argv);
 die("unknown source controller mode: %s",argv[1]);return 2;
}

#undef die
