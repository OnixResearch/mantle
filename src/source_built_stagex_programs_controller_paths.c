/* Source-owned logical filesystem for the shell evaluator, not utility objects.
 *
 * Composition: include this file before the evaluator. Rewrite only evaluator
 * file operations to mantle_paths_* (or define MANTLE_PATHS_WRAP_LIBC here and
 * undef the listed macros before controller/metadata/native utility source).
 * libc calls inside this file are deliberately physical and never remapped.
 * Initialize after physical checkpoint setup and before evaluating source.
 *
 * int mantle_paths_init(const char *workspace, const char *manifest,
 *                       const char *initial_logical_cwd);
 * int mantle_paths_init_environment(const char *workspace, const char *cwd);
 * int mantle_paths_logical(const char *physical, char *logical, size_t capacity);
 * void mantle_paths_destroy(void);
 * int mantle_paths_map(const char *path, int writing,
 *                      struct mantle_paths_operand *result);
 *
 * Manifest: consecutive UTF-8/byte netstrings (decimal length ':' bytes ',').
 * First record is MANTLE-CONTROLLER-ROOTS-1. Every subsequent triple is
 *   mutable, LOGICAL-ABSOLUTE-ROOT, WORKSPACE-RELATIVE-DIRECTORY
 * or
 *   runtime, ORIGINAL-RUNTIME-ROOT, IDENTICAL-ORIGINAL-RUNTIME-ROOT
 * or
 *   input, /inputs/NAME, NAME
 * Input roots are exact declared readonly Trees, never executable grants.
 * Runtime roots are /mantle/store/COMPONENT[/subtree...], exact
 * /mantle-rust-runtime, or /lib, /lib64, /usr/lib, /usr/lib64[/subtree...].
 * EOF must fall between triples. No NULs, empty records, duplicate/overlapping
 * logical roots, overlapping mutable targets, '.', '..', repeated separators,
 * backslashes, trailing separators, root '/', or rewritten runtime locations.
 * Limits: 128 roots, PATH_MAX-1 bytes/path, 40 symlinks/resolution.
 * Mutable roots retain actual source existence; only private carrier parents
 * are created, never a deleted source work/output directory during replay.
 * The manifest is explicit Produced Text, not an executable lookup list.
 *
 * Mapping returns WORKSPACE (logical_root + relative), RUNTIME (runtime_root
 * + relative), or INPUT (input_name + relative). Callers classify original
 * option grammar; this module never classifies data/flags/executables.
 * Logical strings, configuration prefixes, symlink contents and source env
 * values are not rewritten. Relative paths use the logical current directory.
 * Engine symlink traversal is resolved component-by-component and cannot
 * escape into undeclared host roots. Unknown dirfds are not ambient authority.
 * Exact /dev/null is an environmental I/O endpoint, not a declared root or
 * artifact operand. Only the actual sandbox null character device is used;
 * other /dev paths are absent and no /dev directory is exposed to source.
 */
#ifndef _GNU_SOURCE
#define _GNU_SOURCE 1
#endif
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdarg.h>
#include <errno.h>
#include <limits.h>
#include <unistd.h>
#include <dirent.h>
#include <fcntl.h>
#include <sys/stat.h>
#include <time.h>
#include <glob.h>
#include <fnmatch.h>
#include <sys/sysmacros.h>

#ifndef PATH_MAX
#define PATH_MAX 4096
#endif
#define MANTLE_PATHS_MAX_ROOTS 128
#define MANTLE_PATHS_WORKSPACE 1
#define MANTLE_PATHS_RUNTIME 2
#define MANTLE_PATHS_INPUT 3
struct mantle_paths_operand {
 int kind;
 const char *logical_root;
 const char *runtime_root; /* Only for a declared runtime mount. */
 const char *input_name; /* Only for a declared readonly /inputs/NAME Tree. */
 char relative[PATH_MAX];
 char workspace_relative[PATH_MAX]; /* empty for readonly paths. */
 char logical[PATH_MAX];
 char physical[PATH_MAX];
};
struct mantle_paths_root {
 int kind, fd;
 char logical[PATH_MAX], target[PATH_MAX], physical[PATH_MAX];
 const char *leaf; /* Mutable roots anchor at their private parent, not the root inode. */
};
static struct mantle_paths_root mantle_paths_roots[MANTLE_PATHS_MAX_ROOTS];
static size_t mantle_paths_count;
static int mantle_paths_ready;
static char mantle_paths_workspace[PATH_MAX], mantle_paths_cwd[PATH_MAX];

static int mantle_paths_error(int error) { errno=error; return -1; }
static int mantle_paths_copy(char *to,const char *from) {
 size_t n=strlen(from); if(n>=PATH_MAX)return mantle_paths_error(ENAMETOOLONG);
 memcpy(to,from,n+1); return 0;
}
static int mantle_paths_join(char *out,const char *a,const char *b) {
 size_t n=strlen(a),m=strlen(b),slash=n&&m&&a[n-1]!='/';
 if(n+m+slash>=PATH_MAX)return mantle_paths_error(ENAMETOOLONG);
 memmove(out,a,n); if(slash)out[n++]='/'; memmove(out+n,b,m+1); return 0;
}
static int mantle_paths_contains(const char *parent,const char *child) {
 size_t n=strlen(parent); return !strncmp(parent,child,n)&&(!child[n]||child[n]=='/');
}
static int mantle_paths_clean(const char *path,int absolute) {
 if(!*path||strchr(path,'\\')||((*path=='/')!=absolute))return 0;
 const char *p=path+absolute;
 if(!*p)return 0;
 while(*p){const char *start=p;while(*p&&*p!='/')p++;size_t n=(size_t)(p-start);
  if(!n||(n==1&&start[0]=='.')||(n==2&&start[0]=='.'&&start[1]=='.'))return 0;
  if(*p&&! *++p)return 0;
 }
 return 1;
}
static int mantle_paths_runtime_root(const char *path) {
 if(!mantle_paths_clean(path,1))return 0;
 const char *prefix="/mantle/store/";size_t n=strlen(prefix);
 if(!strncmp(path,prefix,n)&&path[n])return 1;
 if(!strcmp(path,"/mantle-rust-runtime"))return 1;
 static const char *const system_roots[]={"/lib","/lib64","/usr/lib","/usr/lib64"};
 for(size_t i=0;i<sizeof system_roots/sizeof *system_roots;i++)if(mantle_paths_contains(system_roots[i],path))return 1;
 return 0;
}
static struct mantle_paths_root *mantle_paths_find(const char *logical) {
 for(size_t i=0;i<mantle_paths_count;i++)if(mantle_paths_contains(mantle_paths_roots[i].logical,logical))return &mantle_paths_roots[i];
 return NULL;
}
/* Records are bounded before reading and forbid embedded NUL bytes. */
static int mantle_paths_record(FILE *f,char *out) {
 size_t n=0;int c=fgetc(f);if(c==EOF)return ferror(f)?mantle_paths_error(EIO):0;
 if(c<'0'||c>'9')return mantle_paths_error(EINVAL);
 int leading_zero=c=='0';size_t digits=0;
 do{if(++digits>1&&leading_zero)return mantle_paths_error(EINVAL);
  if(n>((PATH_MAX-1)-(unsigned)(c-'0'))/10)return mantle_paths_error(ENAMETOOLONG);
  n=n*10+(unsigned)(c-'0');c=fgetc(f);
 }while(c>='0'&&c<='9');
 if(c!=':'||!n)return mantle_paths_error(EINVAL);
 if(fread(out,1,n,f)!=n||fgetc(f)!=','||memchr(out,0,n))return mantle_paths_error(EINVAL);
 out[n]=0;return 1;
}
void mantle_paths_destroy(void) {
 for(size_t i=0;i<mantle_paths_count;i++)if(mantle_paths_roots[i].fd>=0)close(mantle_paths_roots[i].fd);
 mantle_paths_count=0;mantle_paths_ready=0;mantle_paths_cwd[0]=0;mantle_paths_workspace[0]=0;
}
/* Traverse only directory components beneath an already trusted descriptor. */
static int mantle_paths_directory(int root,const char *relative,int create) {
 int fd=fcntl(root,F_DUPFD_CLOEXEC,3);if(fd<0)return -1;
 const char *p=relative;char name[PATH_MAX];
 while(*p){const char *start=p;while(*p&&*p!='/')p++;size_t n=(size_t)(p-start);memcpy(name,start,n);name[n]=0;
  if(create&&mkdirat(fd,name,0755)&&errno!=EEXIST){int e=errno;close(fd);return mantle_paths_error(e);}
  int next=openat(fd,name,O_RDONLY|O_DIRECTORY|O_NOFOLLOW|O_CLOEXEC);int e=errno;close(fd);if(next<0)return mantle_paths_error(e);fd=next;if(*p)p++;
 }
 return fd;
}
static int mantle_paths_root_directory(struct mantle_paths_root *root,const char *relative) {
 if(root->kind!=MANTLE_PATHS_WORKSPACE)return mantle_paths_directory(root->fd,relative,0);
 int fd=openat(root->fd,root->leaf,O_RDONLY|O_DIRECTORY|O_NOFOLLOW|O_CLOEXEC);if(fd<0)return -1;
 if(!*relative)return fd;
 int child=mantle_paths_directory(fd,relative,0),error=errno;close(fd);errno=error;return child;
}
/* Anchor the raw path before normalizing '..'. Once inside a declared root,
 * '..' may never cross it, even if later components would reenter that root. */
static int mantle_paths_start(const char *path,struct mantle_paths_root **root,char *pending) {
 if(!path||!*path)return mantle_paths_error(ENOENT);
 if(strchr(path,'\\'))return mantle_paths_error(EINVAL);
 if(mantle_paths_contains("/dev",path))return mantle_paths_error(ENOENT);
 if(*path!='/'){
  *root=mantle_paths_find(mantle_paths_cwd);if(!*root)return mantle_paths_error(EACCES);
  const char *relative=mantle_paths_cwd+strlen((*root)->logical);if(*relative=='/')relative++;
  return mantle_paths_join(pending,relative,path);
 }
 char prefix[PATH_MAX]="";const char *p=path;
 while(*p){while(*p=='/')p++;if(!*p)break;const char *start=p;while(*p&&*p!='/')p++;size_t n=(size_t)(p-start);
  if(n==1&&start[0]=='.')continue;
  if(n==2&&start[0]=='.'&&start[1]=='.'){char *last=strrchr(prefix,'/');if(!last)return mantle_paths_error(EACCES);*last=0;continue;}
  size_t used=strlen(prefix);if(used+n+1>=PATH_MAX)return mantle_paths_error(ENAMETOOLONG);prefix[used++]='/';memcpy(prefix+used,start,n);prefix[used+n]=0;
  *root=mantle_paths_find(prefix);if(*root){while(*p=='/')p++;return mantle_paths_copy(pending,p);}
 }
 return mantle_paths_error(EACCES);
}
static int mantle_paths_pop(char *relative) {
 if(!*relative)return mantle_paths_error(EACCES);
 char *slash=strrchr(relative,'/');if(slash)*slash=0;else relative[0]=0;return 0;
}
static int mantle_paths_component(char *pending,char *component) {
 char *p=pending;while(*p=='/')p++;if(!*p){pending[0]=0;component[0]=0;return 0;}
 char *end=p;while(*end&&*end!='/')end++;size_t n=(size_t)(end-p);memcpy(component,p,n);component[n]=0;
 while(*end=='/')end++;memmove(pending,end,strlen(end)+1);return 1;
}
static int mantle_paths_operand_fill(struct mantle_paths_root *root,const char *relative,struct mantle_paths_operand *out) {
 out->kind=root->kind;out->logical_root=root->logical;out->runtime_root=root->kind==MANTLE_PATHS_RUNTIME?root->logical:NULL;out->input_name=root->kind==MANTLE_PATHS_INPUT?root->target:NULL;
 if(mantle_paths_copy(out->relative,relative)||mantle_paths_join(out->logical,root->logical,relative)||mantle_paths_join(out->physical,root->physical,relative))return -1;
 out->workspace_relative[0]=0;
 return root->kind==MANTLE_PATHS_WORKSPACE?mantle_paths_join(out->workspace_relative,root->target,relative):0;
}
int mantle_paths_map(const char *path,int writing,struct mantle_paths_operand *out) {
 if(!mantle_paths_ready)return mantle_paths_error(ENXIO);
 if(!out)return mantle_paths_error(EINVAL);
 struct mantle_paths_root *root;char pending[PATH_MAX],relative[PATH_MAX]="",name[PATH_MAX];
 if(mantle_paths_start(path,&root,pending))return -1;
 while(mantle_paths_component(pending,name)){
  if(!strcmp(name,"."))continue;
  if(!strcmp(name,"..")){if(mantle_paths_pop(relative))return -1;}
  else if(mantle_paths_join(relative,relative,name))return -1;
 }
 if(writing&&root->kind!=MANTLE_PATHS_WORKSPACE)return mantle_paths_error(EROFS);
 return mantle_paths_operand_fill(root,relative,out);
}
/* Reverse only helper-owned physical workspace paths through the exact source
 * root manifest. This never rewrites source variables or grants another root. */
int mantle_paths_logical(const char *physical,char *logical,size_t capacity) {
 if(!mantle_paths_ready)return mantle_paths_error(ENXIO);
 if(!physical||!logical||!capacity)return mantle_paths_error(EINVAL);
 if(strnlen(physical,PATH_MAX)>=PATH_MAX)return mantle_paths_error(ENAMETOOLONG);
 if(!mantle_paths_clean(physical,1))return mantle_paths_error(EINVAL);
 for(size_t i=0;i<mantle_paths_count;i++){
  struct mantle_paths_root *root=&mantle_paths_roots[i];
  if(!mantle_paths_contains(root->physical,physical))continue;
  const char *relative=physical+strlen(root->physical);if(*relative=='/')relative++;
  char path[PATH_MAX];if(mantle_paths_join(path,root->logical,relative))return -1;
  size_t size=strlen(path)+1;if(size>capacity)return mantle_paths_error(ENAMETOOLONG);
  memcpy(logical,path,size);return 0;
 }
 return mantle_paths_error(ENOENT);
}
struct mantle_paths_resolved {
 struct mantle_paths_root *root;
 int parent;
 char name[PATH_MAX],relative[PATH_MAX];
};
/* Return an anchored parent fd; the last component is never followed by libc.
 * Existing symlinks are read as source logical paths, including absolute ones.
 * A missing last component is permitted for creation and ordinary ENOENT calls.
 */
static int mantle_paths_resolve(const char *path,int follow,int writing,struct mantle_paths_resolved *out) {
 if(!mantle_paths_ready)return mantle_paths_error(ENXIO);
 struct mantle_paths_root *root;char pending[PATH_MAX],relative[PATH_MAX]="",name[PATH_MAX];
 if(mantle_paths_start(path,&root,pending))return -1;
 int require_directory=path[strlen(path)-1]=='/';
 int fd=-1,root_self=0;unsigned links=0;
restart_root:
 relative[0]=0;root_self=0;
 if(root->kind==MANTLE_PATHS_WORKSPACE&&!*pending){fd=mantle_paths_directory(root->fd,"",0);if(fd<0)return -1;strcpy(name,root->leaf);root_self=1;goto resolved;}
 fd=mantle_paths_root_directory(root,"");if(fd<0)return -1;
 for(;;){
  if(!mantle_paths_component(pending,name)){strcpy(name,".");goto resolved;}
  if(!strcmp(name,"."))continue;
  if(!strcmp(name,"..")){
   if(mantle_paths_pop(relative))goto fail;
   int next=mantle_paths_root_directory(root,relative);if(next<0)goto fail;close(fd);fd=next;continue;
  }
  struct stat st;
  if(fstatat(fd,name,&st,AT_SYMLINK_NOFOLLOW)){
   if(errno==ENOENT&&!*pending)goto resolved;
   goto fail;
  }
  if(S_ISLNK(st.st_mode)&&(*pending||follow||require_directory)){
   if(++links>40){errno=ELOOP;goto fail;}
   char target[PATH_MAX],combined[PATH_MAX];ssize_t n=readlinkat(fd,name,target,sizeof target-1);
   if(n<0)goto fail;if((size_t)n==sizeof target-1){errno=ENAMETOOLONG;goto fail;}target[n]=0;
   if(!n){errno=ENOENT;goto fail;}
   if(mantle_paths_join(combined,target,pending))goto fail;
   if(*target=='/'){
    if(mantle_paths_start(combined,&root,pending))goto fail;
    close(fd);goto restart_root;
   }else if(mantle_paths_copy(pending,combined))goto fail;
   continue;
  }
  if(!*pending)goto resolved;
  int next=openat(fd,name,O_RDONLY|O_DIRECTORY|O_NOFOLLOW|O_CLOEXEC);if(next<0)goto fail;
  if(mantle_paths_join(relative,relative,name)){close(next);goto fail;}close(fd);fd=next;
 }
resolved:
 if(require_directory){struct stat directory;if(fstatat(fd,name,&directory,AT_SYMLINK_NOFOLLOW))goto fail;if(!S_ISDIR(directory.st_mode)){errno=ENOTDIR;goto fail;}}
 if(writing&&root->kind!=MANTLE_PATHS_WORKSPACE){errno=EROFS;goto fail;}
 out->root=root;out->parent=fd;strcpy(out->name,name);
 if(root_self)out->relative[0]=0;
 else if(mantle_paths_join(out->relative,relative,!strcmp(name,".")?"":name)){close(fd);return -1;}
 return 0;
fail:{int e=errno;close(fd);return mantle_paths_error(e);}
}
int mantle_paths_init(const char *workspace,const char *manifest,const char *cwd) {
 mantle_paths_destroy();if(!workspace||!manifest||!cwd)return mantle_paths_error(EINVAL);
 char *actual=realpath(workspace,NULL);if(!actual)return -1;
 if(!mantle_paths_clean(actual,1)||mantle_paths_copy(mantle_paths_workspace,actual)){free(actual);return mantle_paths_error(EINVAL);}free(actual);
 int ws=open(mantle_paths_workspace,O_RDONLY|O_DIRECTORY|O_NOFOLLOW|O_CLOEXEC);if(ws<0)return -1;
 FILE *f=fopen(manifest,"rb");if(!f){int e=errno;close(ws);return mantle_paths_error(e);}
 char kind[PATH_MAX],logical[PATH_MAX],target[PATH_MAX];int rc=mantle_paths_record(f,kind);
 if(rc!=1||strcmp(kind,"MANTLE-CONTROLLER-ROOTS-1")){errno=EINVAL;goto fail;}
 for(;;){
  rc=mantle_paths_record(f,kind);if(!rc)break;if(rc<0)goto fail;
  if(mantle_paths_count==MANTLE_PATHS_MAX_ROOTS){errno=E2BIG;goto fail;}
  if(mantle_paths_record(f,logical)!=1||mantle_paths_record(f,target)!=1){errno=EINVAL;goto fail;}
  int mutable=!strcmp(kind,"mutable"),input=!strcmp(kind,"input");if(!mutable&&!input&&strcmp(kind,"runtime")){errno=EINVAL;goto fail;}
  if(!mantle_paths_clean(logical,1)||!mantle_paths_clean(target,!mutable&&!input)){errno=EINVAL;goto fail;}
  if(input){char expected[PATH_MAX];if(strchr(target,'/')||mantle_paths_join(expected,"/inputs",target)||strcmp(logical,expected)){errno=EINVAL;goto fail;}}
  else if(!mutable&&(!mantle_paths_runtime_root(logical)||strcmp(logical,target))){errno=EINVAL;goto fail;}
  if(mantle_paths_contains("/dev",logical)){errno=EINVAL;goto fail;}
  for(size_t i=0;i<mantle_paths_count;i++){
   struct mantle_paths_root *old=&mantle_paths_roots[i];
   if(mantle_paths_contains(old->logical,logical)||mantle_paths_contains(logical,old->logical)||(mutable&&old->kind==MANTLE_PATHS_WORKSPACE&&(mantle_paths_contains(old->target,target)||mantle_paths_contains(target,old->target)))){errno=EINVAL;goto fail;}
  }
  struct mantle_paths_root *root=&mantle_paths_roots[mantle_paths_count];root->fd=-1;root->kind=mutable?MANTLE_PATHS_WORKSPACE:input?MANTLE_PATHS_INPUT:MANTLE_PATHS_RUNTIME;
  strcpy(root->logical,logical);strcpy(root->target,target);
  if(mutable){
   if(mantle_paths_join(root->physical,mantle_paths_workspace,target))goto fail;
   char parent[PATH_MAX];strcpy(parent,target);char *slash=strrchr(parent,'/');
   root->leaf=strrchr(root->target,'/');root->leaf=root->leaf?root->leaf+1:root->target;
   if(slash){*slash=0;root->fd=mantle_paths_directory(ws,parent,1);}else root->fd=mantle_paths_directory(ws,"",0);
  }
  else {if(input){if(mantle_paths_join(root->physical,"/inputs",target))goto fail;}else strcpy(root->physical,target);if(mantle_paths_contains(root->physical,mantle_paths_workspace)||mantle_paths_contains(mantle_paths_workspace,root->physical)){errno=EINVAL;goto fail;}root->fd=open(root->physical,O_RDONLY|O_DIRECTORY|O_NOFOLLOW|O_CLOEXEC);}
  if(root->fd<0)goto fail;mantle_paths_count++;
 }
 if(!mantle_paths_count){errno=EINVAL;goto fail;}
 if(fclose(f)){f=NULL;goto fail;}f=NULL;close(ws);ws=-1;
 if(!mantle_paths_clean(cwd,1)||!mantle_paths_find(cwd)||mantle_paths_copy(mantle_paths_cwd,cwd)){errno=EINVAL;goto fail;}
 mantle_paths_ready=1;
 /* Establish a real cwd as well, but return only logical strings to source. */
 struct mantle_paths_resolved resolved;if(mantle_paths_resolve(cwd,1,0,&resolved))goto fail;
 int fd=openat(resolved.parent,resolved.name,O_RDONLY|O_DIRECTORY|O_NOFOLLOW|O_CLOEXEC);int e=errno;close(resolved.parent);
 if(fd<0){errno=e;goto fail;}
 rc=fchdir(fd);e=errno;close(fd);if(rc){errno=e;goto fail;}
 if(mantle_paths_join(mantle_paths_cwd,resolved.root->logical,resolved.relative))goto fail;
 return 0;
fail:{int e=errno;if(f)fclose(f);if(ws>=0)close(ws);mantle_paths_destroy();return mantle_paths_error(e);}
}
int mantle_paths_init_environment(const char *workspace,const char *cwd) {
 const char *manifest=getenv("MANTLE_CONTROLLER_ROOTS");if(!manifest||!*manifest)return mantle_paths_error(EINVAL);
 return mantle_paths_init(workspace,manifest,cwd);
}
/* Preserve errno across parent-fd cleanup on both success and failure. */
static int mantle_paths_finish(int parent,int result) {int e=errno;close(parent);errno=e;return result;}
static int mantle_paths_null_fd(int flags,mode_t mode) {
 if(!mantle_paths_ready)return mantle_paths_error(ENXIO);
 struct stat st;if(lstat("/dev/null",&st))return -1;
 if(!S_ISCHR(st.st_mode)||major(st.st_rdev)!=1||minor(st.st_rdev)!=3)return mantle_paths_error(ENODEV);
 if((flags&(O_CREAT|O_EXCL))==(O_CREAT|O_EXCL))return mantle_paths_error(EEXIST);
 /* A sink/source may never create or truncate a regular file in /dev. */
 int fd=open("/dev/null",(flags&~(O_CREAT|O_EXCL|O_TRUNC))|O_NOFOLLOW,mode);if(fd<0)return -1;
 int rc=fstat(fd,&st),e=errno;
 if(rc){close(fd);return mantle_paths_error(e);}
 if(!S_ISCHR(st.st_mode)||major(st.st_rdev)!=1||minor(st.st_rdev)!=3){close(fd);return mantle_paths_error(ENODEV);}
 return fd;
}
static int mantle_paths_null_stat(struct stat *st) {
 int fd=mantle_paths_null_fd(O_RDONLY|O_NONBLOCK|O_CLOEXEC,0);if(fd<0)return -1;
 return mantle_paths_finish(fd,fstat(fd,st));
}
int mantle_paths_open(const char *path,int flags,...) {
 mode_t mode=0;if(flags&O_CREAT){va_list ap;va_start(ap,flags);mode=(mode_t)va_arg(ap,int);va_end(ap);}
#ifdef O_TMPFILE
 if((flags&O_TMPFILE)==O_TMPFILE)return mantle_paths_error(EOPNOTSUPP);
#endif
 if(path&&!strcmp(path,"/dev/null"))return mantle_paths_null_fd(flags,mode);
 int writing=(flags&(O_WRONLY|O_RDWR|O_CREAT|O_TRUNC|O_APPEND))!=0;
 struct mantle_paths_resolved r;int follow=!(flags&O_NOFOLLOW)&&!((flags&O_CREAT)&&(flags&O_EXCL));
 if(mantle_paths_resolve(path,follow,writing,&r))return -1;
 int fd=openat(r.parent,r.name,flags|O_NOFOLLOW,mode);return mantle_paths_finish(r.parent,fd);
}
FILE *mantle_paths_fopen(const char *path,const char *mode) {
 if(!mode||!*mode){errno=EINVAL;return NULL;}int flags;
 if(*mode=='r')flags=O_RDONLY;else if(*mode=='w')flags=O_WRONLY|O_CREAT|O_TRUNC;else if(*mode=='a')flags=O_WRONLY|O_CREAT|O_APPEND;else{errno=EINVAL;return NULL;}
 for(const char *p=mode+1;*p;p++){if(*p=='+')flags=(flags&~O_ACCMODE)|O_RDWR;else if(*p=='x')flags|=O_EXCL;else if(*p=='e')flags|=O_CLOEXEC;else if(*p!='b'){errno=EINVAL;return NULL;}}
 int fd=mantle_paths_open(path,flags,0666);if(fd<0)return NULL;FILE *f=fdopen(fd,mode);if(!f){int e=errno;close(fd);errno=e;}return f;
}
int mantle_paths_stat(const char *path,struct stat *st) {
 if(path&&!strcmp(path,"/dev/null"))return mantle_paths_null_stat(st);
 struct mantle_paths_resolved r;if(mantle_paths_resolve(path,1,0,&r))return -1;
 return mantle_paths_finish(r.parent,fstatat(r.parent,r.name,st,AT_SYMLINK_NOFOLLOW));
}
int mantle_paths_lstat(const char *path,struct stat *st) {
 if(path&&!strcmp(path,"/dev/null"))return mantle_paths_null_stat(st);
 struct mantle_paths_resolved r;if(mantle_paths_resolve(path,0,0,&r))return -1;
 return mantle_paths_finish(r.parent,fstatat(r.parent,r.name,st,AT_SYMLINK_NOFOLLOW));
}
int mantle_paths_access(const char *path,int mode) {
 if(path&&!strcmp(path,"/dev/null")){
  if(mode&~(R_OK|W_OK|X_OK))return mantle_paths_error(EINVAL);
  struct stat st;if(mantle_paths_null_stat(&st))return -1;
  return faccessat(AT_FDCWD,"/dev/null",mode,AT_SYMLINK_NOFOLLOW);
 }
 struct mantle_paths_resolved r;if(mantle_paths_resolve(path,1,(mode&W_OK)!=0,&r))return -1;
 return mantle_paths_finish(r.parent,faccessat(r.parent,r.name,mode,AT_SYMLINK_NOFOLLOW));
}
int mantle_paths_chdir(const char *path) {
 struct mantle_paths_resolved r;if(mantle_paths_resolve(path,1,0,&r))return -1;
 int fd=openat(r.parent,r.name,O_RDONLY|O_DIRECTORY|O_NOFOLLOW|O_CLOEXEC);int e=errno;close(r.parent);if(fd<0)return mantle_paths_error(e);
 char logical[PATH_MAX];if(mantle_paths_join(logical,r.root->logical,r.relative)){e=errno;close(fd);return mantle_paths_error(e);}
 int rc=fchdir(fd);e=errno;close(fd);if(rc)return mantle_paths_error(e);strcpy(mantle_paths_cwd,logical);return 0;
}
char *mantle_paths_getcwd(char *buffer,size_t size) {
 if(!mantle_paths_ready){errno=ENXIO;return NULL;}size_t n=strlen(mantle_paths_cwd)+1;
 if(buffer){if(!size){errno=EINVAL;return NULL;}if(size<n){errno=ERANGE;return NULL;}}
 else {if(size&&size<n){errno=ERANGE;return NULL;}buffer=malloc(size?size:n);if(!buffer)return NULL;}
 memcpy(buffer,mantle_paths_cwd,n);return buffer;
}
DIR *mantle_paths_opendir(const char *path) {
 struct mantle_paths_resolved r;if(mantle_paths_resolve(path,1,0,&r))return NULL;
 int fd=openat(r.parent,r.name,O_RDONLY|O_DIRECTORY|O_NOFOLLOW|O_CLOEXEC);int e=errno;close(r.parent);if(fd<0){errno=e;return NULL;}
 DIR *d=fdopendir(fd);if(!d){e=errno;close(fd);errno=e;}return d;
}
static int mantle_paths_not_root(struct mantle_paths_resolved *r) {
 if(!strcmp(r->name,".")){errno=EBUSY;return -1;}return 0;
}
int mantle_paths_mkdir(const char *path,mode_t mode) {
 /* mkdir permits a missing last directory even with a trailing separator. */
 char clean[PATH_MAX];if(!path)return mantle_paths_error(EINVAL);
 if(mantle_paths_copy(clean,path))return -1;size_t n=strlen(clean);while(n>1&&clean[n-1]=='/')clean[--n]=0;
 struct mantle_paths_resolved r;if(mantle_paths_resolve(clean,0,1,&r))return -1;
 return mantle_paths_finish(r.parent,mkdirat(r.parent,r.name,mode));
}
int mantle_paths_unlink(const char *path) {
 struct mantle_paths_resolved r;if(mantle_paths_resolve(path,0,1,&r))return -1;
 int rc=mantle_paths_not_root(&r);if(!rc)rc=unlinkat(r.parent,r.name,0);return mantle_paths_finish(r.parent,rc);
}
int mantle_paths_rmdir(const char *path) {
 struct mantle_paths_resolved r;if(mantle_paths_resolve(path,0,1,&r))return -1;
 int rc=mantle_paths_not_root(&r);if(!rc)rc=unlinkat(r.parent,r.name,AT_REMOVEDIR);return mantle_paths_finish(r.parent,rc);
}
int mantle_paths_rename(const char *from,const char *to) {
 struct mantle_paths_resolved a,b;if(mantle_paths_resolve(from,0,1,&a))return -1;
 if(mantle_paths_resolve(to,0,1,&b))return mantle_paths_finish(a.parent,-1);
 int rc=mantle_paths_not_root(&a);if(!rc)rc=mantle_paths_not_root(&b);
 if(!rc)rc=renameat(a.parent,a.name,b.parent,b.name);rc=mantle_paths_finish(a.parent,rc);return mantle_paths_finish(b.parent,rc);
}
ssize_t mantle_paths_readlink(const char *path,char *buffer,size_t size) {
 struct mantle_paths_resolved r;if(mantle_paths_resolve(path,0,0,&r))return -1;
 ssize_t rc=readlinkat(r.parent,r.name,buffer,size);int e=errno;close(r.parent);errno=e;return rc;
}
int mantle_paths_symlink(const char *target,const char *path) {
 if(!target||!*target)return mantle_paths_error(ENOENT);
 /* Contents remain original logical bytes, even for dangling links. Traversal
  * is checked when followed, not guessed when the link is created. */
 struct mantle_paths_resolved r;if(mantle_paths_resolve(path,0,1,&r))return -1;
 return mantle_paths_finish(r.parent,symlinkat(target,r.parent,r.name));
}
int mantle_paths_link(const char *from,const char *to) {
 /* Both ends must be mutable: linking a runtime inode would expose mutations. */
 struct mantle_paths_resolved a,b;if(mantle_paths_resolve(from,0,1,&a))return -1;
 if(mantle_paths_resolve(to,0,1,&b))return mantle_paths_finish(a.parent,-1);
 int rc=linkat(a.parent,a.name,b.parent,b.name,0);rc=mantle_paths_finish(a.parent,rc);return mantle_paths_finish(b.parent,rc);
}
int mantle_paths_chmod(const char *path,mode_t mode) {
 struct mantle_paths_resolved r;if(mantle_paths_resolve(path,1,1,&r))return -1;
 /* chmod requires ownership, not read access; opening O_RDONLY here would
  * incorrectly prevent an owner from restoring permissions on mode-000 files. */
 return mantle_paths_finish(r.parent,fchmodat(r.parent,r.name,mode,AT_SYMLINK_NOFOLLOW));
}
int mantle_paths_utimensat(int dirfd,const char *path,const struct timespec times[2],int flags) {
 if(dirfd!=AT_FDCWD)return mantle_paths_error(EBADF);
 if(flags&~AT_SYMLINK_NOFOLLOW)return mantle_paths_error(EINVAL);
 struct mantle_paths_resolved r;if(mantle_paths_resolve(path,!(flags&AT_SYMLINK_NOFOLLOW),1,&r))return -1;
 return mantle_paths_finish(r.parent,utimensat(r.parent,r.name,times,AT_SYMLINK_NOFOLLOW));
}

struct mantle_paths_glob_part {char *text;size_t slashes;};
struct mantle_paths_glob_state {
 char pattern[PATH_MAX],prefix[PATH_MAX];
 struct mantle_paths_glob_part parts[PATH_MAX/2+1];
 size_t count,capacity,offset;
 int flags;
 int (*error)(const char *,int);
 glob_t *result;
};
static int mantle_paths_glob_add(struct mantle_paths_glob_state *s,const char *path) {
 glob_t *g=s->result;size_t n=g->gl_pathc;
 if(n>=262144){errno=E2BIG;return GLOB_NOSPACE;}
 if(s->offset+n+2>s->capacity){
  size_t capacity=s->capacity?s->capacity*2:s->offset+16;
  if(capacity<s->offset+n+2)capacity=s->offset+n+2;
  char **next=realloc(g->gl_pathv,capacity*sizeof *next);if(!next)return GLOB_NOSPACE;
  g->gl_pathv=next;s->capacity=capacity;
 }
 char *copy=strdup(path);if(!copy)return GLOB_NOSPACE;
 if(!n)for(size_t i=0;i<s->offset;i++)g->gl_pathv[i]=NULL;
 g->gl_pathv[s->offset+n]=copy;g->gl_pathv[s->offset+n+1]=NULL;g->gl_pathc++;return 0;
}
static int mantle_paths_glob_append(struct mantle_paths_glob_state *s,const char *text,size_t slashes) {
 size_t n=strlen(s->prefix),m=strlen(text);
 if(n+m+slashes>=PATH_MAX)return mantle_paths_error(ENAMETOOLONG);
 memcpy(s->prefix+n,text,m);n+=m;while(slashes--)s->prefix[n++]='/';s->prefix[n]=0;return 0;
}
static int mantle_paths_root_present(const struct mantle_paths_root *root) {
 struct stat st;return root->kind!=MANTLE_PATHS_WORKSPACE?!fstat(root->fd,&st):!fstatat(root->fd,root->leaf,&st,AT_SYMLINK_NOFOLLOW);
}
/* Parent directories of distinct declared roots are a finite namespace union,
 * not permission to list their ambient physical parent (notably /mantle/store). */
static int mantle_paths_glob_ancestor(const char *path,char *directory) {
 if(*path=='/'){if(mantle_paths_copy(directory,path))return 0;}
 else if(mantle_paths_join(directory,mantle_paths_cwd,*path?path:"."))return 0;
 size_t n=strlen(directory);while(n>1&&directory[n-1]=='/')directory[--n]=0;
 if(strcmp(directory,"/")&&!mantle_paths_clean(directory,1))return 0;
 for(size_t i=0;i<mantle_paths_count;i++){
  const struct mantle_paths_root *root=&mantle_paths_roots[i];
  if(!mantle_paths_root_present(root))continue;
  if(!strcmp(directory,"/")||(!strncmp(root->logical,directory,n)&&root->logical[n]=='/'))return 1;
 }
 return 0;
}
static int mantle_paths_glob_walk(struct mantle_paths_glob_state *s,size_t index) {
 size_t before=strlen(s->prefix);
 if(index==s->count){
  struct stat st;char directory[PATH_MAX];int virtual_directory=0;
  if(mantle_paths_lstat(s->prefix,&st)){if(!mantle_paths_glob_ancestor(s->prefix,directory))return 0;virtual_directory=1;}
  if((s->flags&GLOB_MARK)&&before&&s->prefix[before-1]!='/'&&(virtual_directory||(!mantle_paths_stat(s->prefix,&st)&&S_ISDIR(st.st_mode)))){
   if(mantle_paths_glob_append(s,"",1))return GLOB_NOSPACE;
  }
  int rc=mantle_paths_glob_add(s,s->prefix);s->prefix[before]=0;return rc;
 }
 struct mantle_paths_glob_part *part=&s->parts[index];
 int magic=0;for(const char *p=part->text;*p;p++){
  if(*p=='\\'&&!(s->flags&GLOB_NOESCAPE)&&p[1]){p++;continue;}
  if(strchr("*?[",*p)){magic=1;break;}
 }
 if(!magic){
  /* Decode escapes directly into the shared output buffer. No per-component
   * allocation or PATH_MAX-sized recursive stack frame is necessary. */
  size_t at=before;for(const char *p=part->text;*p;p++){
   if(*p=='\\'&&!(s->flags&GLOB_NOESCAPE)&&p[1])p++;
   if(at+1>=PATH_MAX){errno=ENAMETOOLONG;s->prefix[before]=0;return GLOB_NOSPACE;}
   s->prefix[at++]=*p;
  }
  s->prefix[at]=0;
  if(mantle_paths_glob_append(s,"",part->slashes)){s->prefix[before]=0;return GLOB_NOSPACE;}
  int rc=mantle_paths_glob_walk(s,index+1);s->prefix[before]=0;return rc;
 }
 const char *directory=before?s->prefix:".";DIR *d=mantle_paths_opendir(directory);
 if(!d){
  int error=errno;char logical[PATH_MAX];
  if(mantle_paths_glob_ancestor(directory,logical)){
   size_t prefix=strlen(logical);int root_directory=!strcmp(logical,"/");
   const char *seen[MANTLE_PATHS_MAX_ROOTS];size_t lengths[MANTLE_PATHS_MAX_ROOTS],count=0;
   for(size_t i=0;i<mantle_paths_count;i++){
    const struct mantle_paths_root *root=&mantle_paths_roots[i];if(!mantle_paths_root_present(root))continue;
    if(!root_directory&&(strncmp(root->logical,logical,prefix)||root->logical[prefix]!='/'))continue;
    const char *child=root->logical+(root_directory?1:prefix+1);size_t length=strcspn(child,"/");if(!length)continue;
    size_t previous;for(previous=0;previous<count;previous++)if(lengths[previous]==length&&!memcmp(seen[previous],child,length))break;
    if(previous<count)continue;seen[count]=child;lengths[count++]=length;
    char name[PATH_MAX];memcpy(name,child,length);name[length]=0;
    int flags=FNM_PERIOD|((s->flags&GLOB_NOESCAPE)?FNM_NOESCAPE:0);if(fnmatch(part->text,name,flags))continue;
    if(mantle_paths_glob_append(s,name,part->slashes))return GLOB_NOSPACE;
    int rc=mantle_paths_glob_walk(s,index+1);s->prefix[before]=0;if(rc)return rc;
   }
   return 0;
  }
  if((s->error&&s->error(directory,error))||(s->flags&GLOB_ERR))return GLOB_ABORTED;return 0;
 }
 int rc=0;struct dirent *entry;
 for(;;){
  errno=0;entry=readdir(d);if(!entry){int e=errno;if(e&&((s->error&&s->error(directory,e))||(s->flags&GLOB_ERR)))rc=GLOB_ABORTED;break;}
  int flags=FNM_PERIOD|((s->flags&GLOB_NOESCAPE)?FNM_NOESCAPE:0);
  if(fnmatch(part->text,entry->d_name,flags))continue;
  if(mantle_paths_glob_append(s,entry->d_name,part->slashes)){rc=GLOB_NOSPACE;break;}
  rc=mantle_paths_glob_walk(s,index+1);s->prefix[before]=0;if(rc)break;
 }
 s->prefix[before]=0;int e=errno;closedir(d);errno=e;return rc;
}
static int mantle_paths_glob_order(const void *a,const void *b) {return strcoll(*(const char *const *)a,*(const char *const *)b);}
int mantle_paths_glob(const char *pattern,int flags,int (*error)(const char *,int),glob_t *result) {
 if(!mantle_paths_ready){errno=ENXIO;return GLOB_ABORTED;}
 int allowed=GLOB_ERR|GLOB_MARK|GLOB_NOSORT|GLOB_DOOFFS|GLOB_NOCHECK|GLOB_APPEND|GLOB_NOESCAPE;
 if(!pattern||!result||(flags&~allowed)){errno=EINVAL;return GLOB_ABORTED;}
 struct mantle_paths_glob_state s;memset(&s,0,sizeof s);s.flags=flags;s.error=error;s.result=result;
 if(mantle_paths_copy(s.pattern,pattern))return GLOB_NOSPACE;
 s.offset=(flags&GLOB_DOOFFS)?result->gl_offs:0;
 if(s.offset>262144||((flags&GLOB_APPEND)&&result->gl_pathc>262144)){errno=E2BIG;return GLOB_NOSPACE;}
 if(!(flags&GLOB_APPEND)){result->gl_pathc=0;result->gl_pathv=NULL;}
 else if(result->gl_pathv)s.capacity=s.offset+result->gl_pathc+1;
 if((flags&GLOB_APPEND)&&result->gl_pathc&&!result->gl_pathv){errno=EINVAL;return GLOB_ABORTED;}
 result->gl_offs=s.offset;size_t first=result->gl_pathc;
 char *p=s.pattern;size_t leading=0;while(*p=='/'){p++;leading++;}
 if(mantle_paths_glob_append(&s,"",leading))return GLOB_NOSPACE;
 while(*p){
  struct mantle_paths_glob_part *part=&s.parts[s.count++];part->text=p;
  while(*p&&*p!='/')p++;
  if(*p){*p++=0;part->slashes=1;while(*p=='/'){part->slashes++;p++;}}
 }
 int rc=*pattern?mantle_paths_glob_walk(&s,0):0;
 if(!rc&&result->gl_pathc==first){if(flags&GLOB_NOCHECK)rc=mantle_paths_glob_add(&s,pattern);else rc=GLOB_NOMATCH;}
 if(!rc&&!(flags&GLOB_NOSORT))qsort(result->gl_pathv+s.offset+first,result->gl_pathc-first,sizeof *result->gl_pathv,mantle_paths_glob_order);
 return rc;
}
void mantle_paths_globfree(glob_t *result) {
 if(!result)return;
 for(size_t i=0;i<result->gl_pathc;i++)free(result->gl_pathv[result->gl_offs+i]);
 free(result->gl_pathv);result->gl_pathv=NULL;result->gl_pathc=0;
}

/* Optional evaluator-only composition hooks. Never leave these enabled around
 * checkpoint/manifest I/O, shared metadata code, or original utility objects. */
#ifdef MANTLE_PATHS_WRAP_LIBC
#define fopen(...) mantle_paths_fopen(__VA_ARGS__)
#define open(...) mantle_paths_open(__VA_ARGS__)
#define stat(...) mantle_paths_stat(__VA_ARGS__)
#define lstat(...) mantle_paths_lstat(__VA_ARGS__)
#define access(...) mantle_paths_access(__VA_ARGS__)
#define chdir(...) mantle_paths_chdir(__VA_ARGS__)
#define getcwd(...) mantle_paths_getcwd(__VA_ARGS__)
#define opendir(...) mantle_paths_opendir(__VA_ARGS__)
#define mkdir(...) mantle_paths_mkdir(__VA_ARGS__)
#define unlink(...) mantle_paths_unlink(__VA_ARGS__)
#define rmdir(...) mantle_paths_rmdir(__VA_ARGS__)
#define rename(...) mantle_paths_rename(__VA_ARGS__)
#define readlink(...) mantle_paths_readlink(__VA_ARGS__)
#define symlink(...) mantle_paths_symlink(__VA_ARGS__)
#define link(...) mantle_paths_link(__VA_ARGS__)
#define chmod(...) mantle_paths_chmod(__VA_ARGS__)
#define utimensat(...) mantle_paths_utimensat(__VA_ARGS__)
#define glob(...) mantle_paths_glob(__VA_ARGS__)
#define globfree(...) mantle_paths_globfree(__VA_ARGS__)
#endif
