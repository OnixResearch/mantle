/* Adapt only original M4 1.4.7 response and process-boundary entries.
 * Ordinary macro expansion, builtin validation and status decoding stay original. */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
static char source[1024u*1024u+1];
static char entry[64u*1024u+1];
static void fail(const char *message){fprintf(stderr,"original M4 source entry: %s\n",message);abort();}
static size_t load(const char *path,char *data,size_t cap){FILE *f=fopen(path,"rb");size_t n;if(!f)fail("opening source");n=fread(data,1,cap,f);if(ferror(f)||n==cap||fclose(f))fail("reading bounded source");data[n]=0;if(memchr(data,0,n))fail("NUL in source");return n;}
static void emit(FILE *out,const char *data,size_t size){if(fwrite(data,1,size,out)!=size)fail("writing source");}
int main(int argc,char **argv){
 const char *signature="main (int argc, char *const *argv, char *const *envp)";char *at;size_t n,m;FILE *out;
 if(argc!=5)fail("usage: helper MODE ORIGINAL ENTRY OUTPUT");n=load(argv[2],source,sizeof source-1);m=load(argv[3],entry,sizeof entry-1);
 if(!strcmp(argv[1],"builtin")){
  const char *before[]={"#include \"m4.h\"\n","  sysval = system (ARG (1));\n","  pin = popen (ARG (1), \"r\");\n","      sysval = pclose (pin);\n"};
  const char *after[]={"#include \"m4.h\"\nstatic int mantle_m4_child_system(const char *);\nstatic FILE *mantle_m4_child_popen(const char *,const char *);\nstatic int mantle_m4_child_pclose(FILE *);\n","  sysval = mantle_m4_child_system (ARG (1));\n","  pin = mantle_m4_child_popen (ARG (1), \"r\");\n","      sysval = mantle_m4_child_pclose (pin);\n"};
  char *positions[4];const char *cursor=source;size_t i;
  for(i=0;i<4;i++){positions[i]=strstr(source,before[i]);if(!positions[i]||strstr(positions[i]+strlen(before[i]),before[i])||(i&&positions[i]<=positions[i-1]))fail("original M4 builtin selector differs");}
  out=fopen(argv[4],"wbx");if(!out)fail("creating builtin source");
  for(i=0;i<4;i++){emit(out,cursor,(size_t)(positions[i]-cursor));emit(out,after[i],strlen(after[i]));cursor=positions[i]+strlen(before[i]);}
  emit(out,cursor,n-(size_t)(cursor-source));emit(out,"\n",1);emit(out,entry,m);emit(out,"\n",1);
  if(fclose(out))fail("closing builtin source");return 0;
 }
 if(strcmp(argv[1],"main"))fail("unknown original source mode");
 at=strstr(source,signature);if(!at||strstr(at+1,signature))fail("original M4 1.4.7 main ABI differs");
 out=fopen(argv[4],"wbx");if(!out)fail("creating source");
 emit(out,"#define main mantle_m4_original_main\n",sizeof "#define main mantle_m4_original_main\n"-1);emit(out,source,n);
 {
 const char *prefix="\n#undef main\n#define main mantle_m4_response_main\n#define fail mantle_m4_response_fail\n#define space mantle_m4_response_space\n#define phase_cwd mantle_m4_response_cwd\n#define response mantle_m4_response_bytes\n#define arguments mantle_m4_response_arguments\n";
 const char *suffix="\n#undef main\n#undef fail\n#undef space\n#undef phase_cwd\n#undef response\n#undef arguments\nint main(int argc,char **argv,char **envp)\n{\n if(getenv(\"MANTLE_PHASE_CWD\") && argc==2 && argv[1][0]=='@' && argv[1][1])\n  return mantle_m4_response_main(argc,argv);\n return mantle_m4_original_main(argc,argv,envp);\n}\n";
 emit(out,prefix,strlen(prefix));emit(out,entry,m);emit(out,suffix,strlen(suffix));
 }
 if(fclose(out))fail("closing source");return 0;
}
