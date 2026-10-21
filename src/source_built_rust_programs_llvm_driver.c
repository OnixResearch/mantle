/* Source option grammar shared by actual CMake and its LLVM generators.
 * This code only types operands; original CMake/TableGen choose every command. */
static int llvm_native_prepare(const char *slot,Request *r,mantle_controller_builder *b){
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
 if(!strcmp(key,"MANTLE_CONTROLLER_ROOTS"))return mantle_controller_named_word(b,0,"command-roots","");
 if(mantle_controller_oneof(key,"MANTLE_CAIRN_CMAKE_PHASE MANTLE_CAIRN_CMAKE_HISTORY MANTLE_CAIRN_CMAKE_WORKSPACE MANTLE_CAIRN_CMAKE_OBSERVATIONS")){const char root[]="/outputs/workspace";size_t n=sizeof root-1;if(strncmp(value,root,n)||(value[n]&&value[n]!='/'))die("CMake continuation metadata escaped declared output");const char *relative=value+n;if(*relative=='/')++relative;return mantle_controller_named_word(b,1,"workspace",relative);}
 if(mantle_controller_oneof(key,"MANTLE_CMAKE_PROJECT MANTLE_LLVM_CMAKE PYTHONHOME")){mantle_controller_word_begin(b);mantle_controller_path(b,value,!strcmp(key,"MANTLE_CMAKE_PROJECT"));return mantle_controller_take_word(b);}
 return native_gcc_environment(b,key,value);
}
/* The captured command enters the SAME evaluator and descriptor decoder as an
 * original shell command. No reconstructed shell text or second wire format. */
static int llvm_native_source_entry(Env *e,const char *program){
 const char *captured=getenv("MANTLE_NATIVE_CAPTURED_REQUEST");
 if(!captured)return -1;
 (void)program;Request request;read_request_data_at(captured,&request,workspace_root);
 if(!request.active)die("captured native command is inactive");
 for(int i=0;i<64;i++)if(e->fd[i]){FILE *stream=e->fd[i];for(int j=i;j<64;j++)if(e->fd[j]==stream)e->fd[j]=NULL;if(fclose(stream))die("close initial native descriptor");}
 clearvars(e);free(e->cwd);free(e->zero);
 *e=request.env;parsing_source=e->zero;
 if(mantle_paths_ready?mantle_paths_chdir(e->cwd):chdir(e->cwd))die("captured native command cwd");
 int status=external(e,&request.command);request.env=*e;
 char *result=join(output_root,"native-result");write_result(result,&request,status,NULL,NULL);free(result);
 return status;
}
