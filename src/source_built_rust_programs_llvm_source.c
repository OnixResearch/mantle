/* Original LLVM source/image namespace boundary and product projection.
 * No configuration, compiler, child process, or generated inventory executes here. */
#define _POSIX_C_SOURCE 200809L
#include <dirent.h>
#include <errno.h>
#include <fcntl.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <unistd.h>
static void fail(const char *s){fprintf(stderr,"LLVM source boundary: %s: %s\n",s,strerror(errno));abort();}
static void *mem(size_t n){void *p=malloc(n?n:1);if(!p)fail("allocation");return p;}
static char *path(const char *a,const char *b){size_t n=strlen(a),m=strlen(b);if(n>1048576||m>1048576)fail("path bound");char *p=mem(n+m+2);memcpy(p,a,n);p[n++]='/';memcpy(p+n,b,m+1);return p;}
static void directory(const char *p){if(mkdir(p,0700)&&errno!=EEXIST)fail("directory");struct stat s;if(lstat(p,&s)||!S_ISDIR(s.st_mode))fail("directory kind");}
static void file(const char *from,const char *to,mode_t mode,int replace){int in=open(from,O_RDONLY|O_NOFOLLOW);if(in<0)fail("source file");int out=open(to,O_WRONLY|O_CREAT|O_NOFOLLOW|(replace?O_TRUNC:O_EXCL),mode&0777);if(out<0)fail("output file");char bytes[65536];for(;;){ssize_t n=read(in,bytes,sizeof bytes);if(n<0&&errno==EINTR)continue;if(n<0)fail("read");if(!n)break;for(ssize_t at=0;at<n;){ssize_t k=write(out,bytes+at,(size_t)(n-at));if(k<0&&errno==EINTR)continue;if(k<=0)fail("write");at+=k;}}if(close(in)||close(out))fail("close");}
static unsigned long entries;
static void tree(const char *from,const char *to,unsigned depth,int merge){if(depth>128||++entries>1000000)fail("source tree bound");struct stat s;if(lstat(from,&s))fail("source stat");if(S_ISDIR(s.st_mode)){directory(to);DIR *d=opendir(from);if(!d)fail("source directory");struct dirent *e;errno=0;while((e=readdir(d))){if(!strcmp(e->d_name,".")||!strcmp(e->d_name,".."))continue;char *a=path(from,e->d_name),*b=path(to,e->d_name);tree(a,b,depth+1,merge);free(a);free(b);errno=0;}if(errno||closedir(d))fail("source enumeration");}else if(S_ISREG(s.st_mode))file(from,to,s.st_mode,merge);else if(S_ISLNK(s.st_mode)){size_t n=s.st_size>0?(size_t)s.st_size+1:4096;char *b=mem(n+1);ssize_t k=readlink(from,b,n);if(k<0||(size_t)k==n)fail("source link");b[k]=0;if(merge&&unlink(to)&&errno!=ENOENT)fail("replace link");if(symlink(b,to))fail("copy link");free(b);}else fail("non-source tree member");if(!S_ISLNK(s.st_mode)&&chmod(to,s.st_mode&0777))fail("source mode");struct timespec times[2]={{0,UTIME_OMIT},s.st_mtim};if(utimensat(AT_FDCWD,to,times,AT_SYMLINK_NOFOLLOW))fail("source time");}
static int same(const char *a,const char *b){FILE *x=fopen(a,"rb"),*y=fopen(b,"rb");if(!x||!y)fail("protected LLVM image");int equal=1;char p[65536],q[65536];for(;;){size_t n=fread(p,1,sizeof p,x),m=fread(q,1,sizeof q,y);if(n!=m||memcmp(p,q,n)){equal=0;break;}if(!n)break;}if(ferror(x)||ferror(y)||fclose(x)||fclose(y))fail("protected LLVM image read");return equal;}
static void empty(const char *root,const char *name){char *p=path(root,name);int fd=open(p,O_WRONLY|O_CREAT|O_EXCL|O_NOFOLLOW,0600);free(p);if(fd<0||close(fd))fail("empty source entry");}
static void native_seed(const char *request,const char *tmp,const char *out){
 directory(out);char *target=path(out,"request");tree(request,target,0,0);
 char *workspace=path(target,"workspace");directory(workspace);char *temporary=path(workspace,"tmp");tree(tmp,temporary,0,0);free(temporary);free(workspace);free(target);
 for(int i=0;i<2;i++){const char *name=i?"zero":"cwd";char *from=path(request,name),*to=path(out,name);file(from,to,0600,0);free(from);free(to);}
 /* The shared request reader installs the full workspace. Startup needs only
  * the authentic cwd directories, not a second copy of every /tmp member. */
 char *initial=path(out,"initial");directory(initial);char *start=path(initial,"tmp");directory(start);free(start);
 char *cwd_file=path(request,"cwd");FILE *cwd_input=fopen(cwd_file,"rb");free(cwd_file);if(!cwd_input)fail("native cwd metadata");char cwd[65536];size_t size=fread(cwd,1,sizeof cwd-1,cwd_input);if(ferror(cwd_input)||!feof(cwd_input)||fclose(cwd_input))fail("native cwd metadata bound");cwd[size]=0;if(size&&cwd[size-1]=='\n')cwd[--size]=0;
 if(!strncmp(cwd,"/tmp/",5)){char *member=cwd+1;for(char *end=member;;end++){if(*end&&*end!='/')continue;char saved=*end;*end=0;char *part=strrchr(member,'/');part=part?part+1:member;if(!*part||!strcmp(part,".")||!strcmp(part,".."))fail("native cwd is not canonical");char *target=path(initial,member);directory(target);free(target);*end=saved;if(!saved)break;}}
 free(initial);
 empty(out,"native-command");empty(out,"environment");
}
static void diagnostic(const char *root,const char *name,FILE *to){char *p=path(root,name);FILE *in=fopen(p,"rb");free(p);if(!in){if(errno==ENOENT)return;fail("native diagnostic");}char buffer[65536];size_t count;while((count=fread(buffer,1,sizeof buffer,in)))if(fwrite(buffer,1,count,to)!=count)fail("native diagnostic write");if(ferror(in)||fclose(in))fail("native diagnostic read");}
static int native_result(const char *checkpoint,const char *tmp,const char *result){
 char *original=path(checkpoint,"native-result");tree(original,result,0,0);char *temporary=path(original,"workspace/tmp");tree(temporary,tmp,0,0);free(temporary);
 char *status=path(original,"exit-status");FILE *in=fopen(status,"rb");if(!in)fail("native result status");char number[16],*end;if(!fgets(number,sizeof number,in)||fgetc(in)!=EOF||fclose(in))fail("native result status bytes");errno=0;long code=strtol(number,&end,10);if(errno||code<0||code>255||strcmp(end,"\n"))fail("native result normal status");free(status);free(original);
 diagnostic(checkpoint,"stdout",stdout);diagnostic(checkpoint,"stderr",stderr);return (int)code;
}
int main(int argc,char **argv){if(argc==5&&!strcmp(argv[1],"native-seed")){native_seed(argv[2],argv[3],argv[4]);return 0;}if(argc==5&&!strcmp(argv[1],"native-result"))return native_result(argv[2],argv[3],argv[4]);if(argc==4&&!strcmp(argv[1],"source")){tree(argv[2],argv[3],0,0);return 0;}if(argc==7&&!strcmp(argv[1],"collect")){/* Public source headers first; original generated/installed headers win. */directory(argv[5]);char *include=path(argv[5],"include");tree(argv[3],include,0,0);free(include);tree(argv[2],argv[5],0,1);char *config=path(argv[5],"bin/llvm-config");struct stat s;if(lstat(config,&s)||!S_ISREG(s.st_mode)||!(s.st_mode&0111)||!same(config,argv[4]))fail("llvm-config differs from protected original compiler product");file(argv[4],argv[6],s.st_mode,0);free(config);return 0;}fail("usage: source LLVM_PROJECT OUTPUT | collect ORIGINAL_ROOT SOURCE_INCLUDE PROTECTED_CONFIG ROOT CONFIG | native-seed REQUEST TMP ENTRY | native-result CHECKPOINT TMP RESULT");return 2;}
