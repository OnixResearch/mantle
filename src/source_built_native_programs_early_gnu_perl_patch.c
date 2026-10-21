/* Source-only Perl 5.6.2/5.10 native process seam. The interpreter's original
 * do_exec3/do_aexec5 choose every image and argument; no script is translated. */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#define MAX_SOURCE (4u*1024u*1024u)
static char source[MAX_SOURCE+1],header[64u*1024u+1],bridge[MAX_SOURCE+1];
struct change { char *at; const char *text; size_t size; };
static struct change changes[8];static size_t change_count;
static void fail(const char *why){fprintf(stderr,"original Perl process source: %s\n",why);exit(1);}
static size_t load(const char *path,char *data,size_t cap){FILE *f=fopen(path,"rb");size_t n;if(!f)fail("opening source");n=fread(data,1,cap,f);if(ferror(f)||n==cap||fclose(f))fail("reading bounded source");data[n]=0;if(memchr(data,0,n))fail("NUL in source");return n;}
static char *find(char *from,const char *needle){char *at=strstr(from,needle);if(!at)fail("original interpreter source anchor differs");return at;}
static void insert(char *at,const char *text,size_t size){struct change *c;if(change_count==sizeof changes/sizeof changes[0])fail("source change bound");c=&changes[change_count++];c->at=at;c->text=text;c->size=size;}
static void literal(char *at,const char *text){insert(at,text,strlen(text));}
static void emit(FILE *out,const char *data,size_t n){if(fwrite(data,1,n,out)!=n)fail("writing source");}
int main(int argc,char **argv){
 size_t n,h,b,i,j;char *include,*at,*cursor;FILE *out;
 if(argc!=6)fail("usage: helper MODE ORIGINAL HEADER BRIDGE OUTPUT");
 n=load(argv[2],source,sizeof source-1);h=load(argv[3],header,sizeof header-1);b=load(argv[4],bridge,sizeof bridge-1);
 include=find(source,"#include \"perl.h\"\n");if(strstr(include+1,"#include \"perl.h\"\n"))fail("multiple interpreter header anchors");include+=strlen("#include \"perl.h\"\n");
 insert(include,header,h);
 if(!strcmp(argv[1],"doio")){
  if(!strstr(source,"Perl_do_exec3(pTHX_ char *cmd, int fd, int do_report)")&&!strstr(source,"Perl_do_exec3(pTHX_ const char *incmd, int fd, int do_report)"))fail("original do_exec3 source ABI differs");
  (void)find(source,"Perl_do_aexec5(pTHX_ SV *really, register SV **mark, register SV **sp,");
  literal(include,"\n#undef PerlProc_execvp\n#define PerlProc_execvp(image, argv) mantle_perl_execvp((image), (char *const *)(argv))\n#undef PerlProc_execl\n#define PerlProc_execl mantle_perl_execl\n");
 }else if(!strcmp(argv[1],"ppsys")){
  at=find(source,"PP(pp_system)\n{");at=find(at,"    PERL_FLUSHALL_FOR_CHILD;\n")+strlen("    PERL_FLUSHALL_FOR_CHILD;\n");
  literal(at,"    if (mantle_perl_transport_active()) {\n        STATUS_NATIVE_SET(mantle_perl_system(aTHX_ (PL_op->op_flags & OPf_STACKED) != 0, MARK, SP));\n        SP = ORIGMARK;\n        PUSHi(STATUS_CURRENT);\n        RETURN;\n    }\n");
 }else if(!strcmp(argv[1],"util")){
  insert(include,bridge,b);
  at=strstr(source,"Perl_my_popen(pTHX_ char *cmd, char *mode)\n{");if(!at)at=find(source,"Perl_my_popen(pTHX_ const char *cmd, const char *mode)\n{");at=find(at,"    if (PerlProc_pipe(p) < 0)\n");
  literal(at,"    if (mantle_perl_transport_active())\n        return mantle_perl_popen(aTHX_ cmd, mode);\n");
  at=find(source,"Perl_my_pclose(pTHX_ PerlIO *ptr)\n{");at=find(at,"    LOCK_FDPID_MUTEX;\n");
  literal(at,"    if (mantle_perl_transport_active() && mantle_perl_pipe_owned(ptr))\n        return mantle_perl_pclose(aTHX_ ptr);\n");
 }else fail("unknown source mode");
 for(i=1;i<change_count;i++){struct change c=changes[i];j=i;while(j&&changes[j-1].at>c.at){changes[j]=changes[j-1];j--;}changes[j]=c;}
 out=fopen(argv[5],"wbx");if(!out)fail("creating source");cursor=source;
 for(i=0;i<change_count;i++){struct change *c=&changes[i];if(c->at<cursor)fail("overlapping source changes");emit(out,cursor,(size_t)(c->at-cursor));emit(out,c->text,c->size);cursor=c->at;}
 emit(out,cursor,n-(size_t)(cursor-source));if(fclose(out))fail("closing source");return 0;
}
