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
int main(int argc,char **argv){if(argc==4&&!strcmp(argv[1],"source")){tree(argv[2],argv[3],0,0);return 0;}if(argc==7&&!strcmp(argv[1],"collect")){/* Public source headers first; original generated/installed headers win. */directory(argv[5]);char *include=path(argv[5],"include");tree(argv[3],include,0,0);free(include);tree(argv[2],argv[5],0,1);char *config=path(argv[5],"bin/llvm-config");struct stat s;if(lstat(config,&s)||!S_ISREG(s.st_mode)||!(s.st_mode&0111)||!same(config,argv[4]))fail("llvm-config differs from protected original compiler product");file(argv[4],argv[6],s.st_mode,0);free(config);return 0;}fail("usage: source LLVM_PROJECT OUTPUT | collect ORIGINAL_ROOT SOURCE_INCLUDE PROTECTED_CONFIG ROOT CONFIG");return 2;}
