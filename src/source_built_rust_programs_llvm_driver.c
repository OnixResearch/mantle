/* Source option grammar shared by actual CMake and its LLVM generators.
 * This code only types operands; original CMake/TableGen choose every command. */
static int llvm_native_prepare(const char *slot,Request *r,mantle_controller_builder *b){
 if(rust_script_prepare(slot,r,b))return 1;
 if(native_gcc_prepare(slot,r,b))return 1;Args *a=&r->command;const char *name=mantle_controller_basename(a->v[0]);
 if(!strcmp(name,"cmake")){
  const char *operation=NULL;int operand=0;
  for(int i=1;i<a->n;++i){const char *s=a->v[i];
   if(!strcmp(s,"-E")){mantle_controller_argument_literal(b,s);if(++i==a->n)die("original CMake -E operation missing");operation=a->v[i];mantle_controller_argument_literal(b,operation);continue;}
   if(operation){
    int data=mantle_controller_oneof(operation,"echo echo_append cmake_echo_color env capabilities time sleep true false")||s[0]=='-'||strchr(s,'=');
    if(!strcmp(operation,"cmake_depends")&&operand==0)data=1;
    if((!strcmp(operation,"cmake_progress_report")&&operand>0)||(!strcmp(operation,"cmake_progress_start")&&operand>0&&!strcmp(s,"0")))data=1;
    if(!strcmp(operation,"create_symlink")&&operand==0)data=1;
    if(data)mantle_controller_argument_data(b,s);else mantle_controller_argument_path(b,s,1);
    ++operand;continue;
   }
   if(!strcmp(s,"-G")||!strcmp(s,"--generator")){mantle_controller_argument_literal(b,s);if(++i==a->n)die("original CMake generator missing");mantle_controller_argument_data(b,a->v[i]);continue;}
   if(!strcmp(s,"-S")||!strcmp(s,"-B")||!strcmp(s,"-P")||!strcmp(s,"--build")||!strcmp(s,"--install")){mantle_controller_argument_literal(b,s);if(++i==a->n)die("original CMake path missing");mantle_controller_argument_path(b,a->v[i],strcmp(s,"-S")&&strcmp(s,"-P"));continue;}
   if(s[0]=='-'){/* Cache definitions retain their original CMake string grammar. */mantle_controller_argument_data(b,s);continue;}
   mantle_controller_argument_path(b,s,0);
  }
  return 1;
 }
 if(strstr(name,"tblgen")||!strcmp(name,"vt_gen")||!strcmp(name,"llvm-config")){
  for(int i=1;i<a->n;++i){const char *s=a->v[i];
   if(!strcmp(s,"-I")||!strcmp(s,"-o")||!strcmp(s,"-d")||!strcmp(s,"--output")||!strcmp(s,"--depend-file")){mantle_controller_argument_literal(b,s);if(++i==a->n)die("original LLVM generator missing path");mantle_controller_argument_path(b,a->v[i],strcmp(s,"-I")!=0);}
   else if(s[0]=='-'||!strcmp(name,"llvm-config"))mantle_controller_argument_data(b,s);
   else mantle_controller_argument_path(b,s,0);
  }
  return 1;
 }
 return 0;
}
static char *llvm_native_environment(mantle_controller_builder *b,const char *key,const char *value){
 char *rust_word=rust_script_environment(b,key,value);if(rust_word)return rust_word;
 if(!strcmp(key,"MANTLE_CONTROLLER_ROOTS"))return mantle_controller_named_word(b,0,"command-roots","");
 if(mantle_controller_oneof(key,"MANTLE_CAIRN_CMAKE_PHASE MANTLE_CAIRN_CMAKE_HISTORY MANTLE_CAIRN_CMAKE_WORKSPACE MANTLE_CAIRN_CMAKE_OBSERVATIONS")){const char root[]="/outputs/workspace";size_t n=sizeof root-1;if(strncmp(value,root,n)||(value[n]&&value[n]!='/'))die("CMake continuation metadata escaped declared output");const char *relative=value+n;if(*relative=='/')++relative;return mantle_controller_named_word(b,1,"workspace",relative);}
 if(mantle_controller_oneof(key,"MANTLE_CMAKE_PROJECT MANTLE_LLVM_CMAKE PYTHONHOME")){mantle_controller_word_begin(b);mantle_controller_path(b,value,!strcmp(key,"MANTLE_CMAKE_PROJECT"));return mantle_controller_take_word(b);}
 return native_gcc_environment(b,key,value);
}
/* Original Rust scripts and their captured children share this evaluator. */
static int llvm_native_source_entry(Env *e,const char *program){return rust_script_source_entry(e,program);}
