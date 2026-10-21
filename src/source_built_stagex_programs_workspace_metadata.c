/* Source-owned workspace time records shared by shell/Make replay.
 * No interpreter, process launch, ELF loader, or executable selection lives here. */
#define _GNU_SOURCE 1
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>
#include <limits.h>
#include <dirent.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <unistd.h>
#include <time.h>
static void mantle_ws_die(const char *message) { fprintf(stderr,"workspace metadata: %s: %s\n",message,strerror(errno)); exit(125); }
static void *mantle_ws_alloc(size_t size) { void *p=malloc(size?size:1);if(!p)mantle_ws_die("allocation");return p; }
static char *mantle_ws_join(const char *a,const char *b) { size_t n=strlen(a),m=strlen(b);char *p=mantle_ws_alloc(n+m+2);memcpy(p,a,n);if(m){p[n++]='/';memcpy(p+n,b,m);}p[n+m]=0;return p; }
static void mantle_ws_record(FILE *f,const char *p) {size_t n=strlen(p);if(fprintf(f,"%lu:",(unsigned long)n)<0||fwrite(p,1,n,f)!=n||fputc(',',f)==EOF)mantle_ws_die("write record");}
static void mantle_ws_number(FILE *f,long long n) {char s[64];snprintf(s,sizeof s,"%lld",n);mantle_ws_record(f,s);}
static char *mantle_ws_read(FILE *f) {size_t n=0;int c=fgetc(f);if(c<'0'||c>'9')mantle_ws_die("record length");do{if(n>(64UL*1024*1024-(unsigned)(c-'0'))/10)mantle_ws_die("record bound");n=n*10+c-'0';c=fgetc(f);}while(c>='0'&&c<='9');if(c!=':')mantle_ws_die("record delimiter");char *s=mantle_ws_alloc(n+1);if(fread(s,1,n,f)!=n||fgetc(f)!=','||memchr(s,0,n))mantle_ws_die("record bytes");s[n]=0;return s;}
static long long mantle_ws_integer(FILE *f) {char *s=mantle_ws_read(f),*end;errno=0;long long n=strtoll(s,&end,10);if(errno||end==s||*end)mantle_ws_die("record integer");free(s);return n;}
static int mantle_ws_name_order(const void *a,const void *b) {return strcmp(*(const char *const *)a,*(const char *const *)b);}
static void mantle_ws_capture_node(const char *root,const char *relative,FILE *f,unsigned long *count) {
 if(++*count>262144UL)mantle_ws_die("workspace node bound");char *path=mantle_ws_join(root,relative);struct stat st;if(lstat(path,&st))mantle_ws_die("stat workspace");
 if(!S_ISDIR(st.st_mode)&&!S_ISREG(st.st_mode)&&!S_ISLNK(st.st_mode))mantle_ws_die("workspace member kind");
 if(S_ISDIR(st.st_mode)){DIR *d=opendir(path);if(!d)mantle_ws_die("open workspace directory");struct dirent *entry;char **names=NULL;size_t n=0,capacity=0;while((entry=readdir(d))){if(!strcmp(entry->d_name,".")||!strcmp(entry->d_name,".."))continue;if(n==capacity){capacity=capacity?capacity*2:16;char **next=realloc(names,capacity*sizeof *names);if(!next)mantle_ws_die("directory allocation");names=next;}names[n]=strdup(entry->d_name);if(!names[n++])mantle_ws_die("directory name");}if(closedir(d))mantle_ws_die("close directory");qsort(names,n,sizeof *names,mantle_ws_name_order);for(size_t i=0;i<n;i++){char *child=*relative?mantle_ws_join(relative,names[i]):strdup(names[i]);if(!child)mantle_ws_die("child path");mantle_ws_capture_node(root,child,f,count);free(child);free(names[i]);}free(names);}
 mantle_ws_record(f,relative);mantle_ws_number(f,(long long)st.st_mtim.tv_sec);mantle_ws_number(f,(long long)st.st_mtim.tv_nsec);free(path);
}
static void mantle_workspace_capture(const char *root,const char *manifest) {FILE *f=fopen(manifest,"wb");if(!f)mantle_ws_die("create time manifest");mantle_ws_record(f,"MANTLE-WORKSPACE-TIMES-1");unsigned long count=0;mantle_ws_capture_node(root,"",f,&count);if(fclose(f))mantle_ws_die("close time manifest");}
static int mantle_ws_relative(const char *s) {if(*s=='/')return 0;while(*s){const char *p=s;while(*s&&*s!='/')s++;size_t n=s-p;if(!n||(n==1&&p[0]=='.')||(n==2&&p[0]=='.'&&p[1]=='.'))return 0;if(*s)s++;}return 1;}
static void mantle_workspace_restore(const char *root,const char *manifest) {
 FILE *f=fopen(manifest,"rb");if(!f)mantle_ws_die("open time manifest");char *magic=mantle_ws_read(f);if(strcmp(magic,"MANTLE-WORKSPACE-TIMES-1"))mantle_ws_die("time manifest version");free(magic);unsigned long count=0;
 for(;;){int c=fgetc(f);if(c==EOF){if(ferror(f))mantle_ws_die("read time manifest");break;}ungetc(c,f);if(++count>262144UL)mantle_ws_die("time manifest node bound");char *relative=mantle_ws_read(f);long long seconds=mantle_ws_integer(f),nanoseconds=mantle_ws_integer(f);if(!mantle_ws_relative(relative)||nanoseconds<0||nanoseconds>=1000000000LL)mantle_ws_die("time manifest path or value");char *path=mantle_ws_join(root,relative);for(char *p=relative;*p;p++)if(*p=='/'){*p=0;char *parent=mantle_ws_join(root,relative);struct stat st;if(lstat(parent,&st)||!S_ISDIR(st.st_mode))mantle_ws_die("time manifest ancestor");free(parent);*p='/';}
 struct timespec times[2];times[0].tv_sec=0;times[0].tv_nsec=UTIME_OMIT;times[1].tv_sec=(time_t)seconds;times[1].tv_nsec=(long)nanoseconds;if((long long)times[1].tv_sec!=seconds||utimensat(AT_FDCWD,path,times,AT_SYMLINK_NOFOLLOW))mantle_ws_die("restore workspace time");free(path);free(relative);}
 if(fclose(f))mantle_ws_die("close time manifest");
}
