/* Childless source-tree primitives used by source-authored StageX transforms.
 * Operands always name declared source/Produced inputs and declared outputs.
 * This program has no interpreter, process-launch or ambient tool facility. */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>
#include <sys/stat.h>
#include <unistd.h>
#include <dirent.h>
#include <fcntl.h>
#include <limits.h>
static void die(const char *s) { perror(s); exit(1); }
static void require(int ok,const char *s) { if(!ok) { fprintf(stderr,"%s\n",s); exit(1); } }
static void new_directory(const char *path,mode_t mode) {
  struct stat st;DIR *dir;struct dirent *entry;
  if(!mkdir(path,mode))return;
  if(errno!=EEXIST)die(path);
  if(lstat(path,&st))die(path);
  require(S_ISDIR(st.st_mode),"tree output is not a directory");
  dir=opendir(path);if(!dir)die(path);
  while((entry=readdir(dir)))require(!strcmp(entry->d_name,".")||!strcmp(entry->d_name,".."),"tree output must be empty");
  if(closedir(dir)||chmod(path,mode))die(path);
}
static void copy_file(const char *src,const char *dst) {
  char buf[16384]; size_t n; struct stat st;
  FILE *in=fopen(src,"rb"),*out;
  if(!in)die(src);
  if(fstat(fileno(in),&st))die(src);
  require(S_ISREG(st.st_mode),"copy requires regular input");
  { int fd=open(dst,O_WRONLY|O_CREAT|O_EXCL,st.st_mode&0777); if(fd<0)die(dst); out=fdopen(fd,"wb"); if(!out)die(dst); }
  while((n=fread(buf,1,sizeof buf,in))) if(fwrite(buf,1,n,out)!=n)die(dst);
  if(ferror(in)||fclose(in)||fclose(out))die("copy");
}
static void join(char *out,const char *root,const char *name) {
  int n=snprintf(out,PATH_MAX,"%s/%s",root,name);
  require(n>0&&n<PATH_MAX,"source member path exceeds bound");
}
static void equal_files(const char *a,const char *b) {
  unsigned char x[16384],y[16384];size_t nx,ny;FILE *fa=fopen(a,"rb"),*fb=fopen(b,"rb");
  if(!fa)die(a);if(!fb)die(b);
  do {nx=fread(x,1,sizeof x,fa);ny=fread(y,1,sizeof y,fb);require(nx==ny&&!memcmp(x,y,nx),"transition product ancestry differs");}while(nx);
  if(ferror(fa)||ferror(fb)||fclose(fa)||fclose(fb))die("compare transition ancestry");
}
static void tree(const char *src,const char *dst) {
  struct stat st; char a[PATH_MAX],b[PATH_MAX]; DIR *dir; struct dirent *entry;
  if(lstat(src,&st))die(src);
  if(S_ISREG(st.st_mode)) {copy_file(src,dst); return;}
  if(S_ISLNK(st.st_mode)) {
    ssize_t n=readlink(src,a,sizeof(a)-1); if(n<0)die(src); a[n]=0;
    require(a[0]!='/'&&strcmp(a,"..")&&strncmp(a,"../",3)&&!strstr(a,"/../")&&!(n>=3&&!strcmp(a+n-3,"/..")),"source symlink escapes tree");
    if(symlink(a,dst))die(dst); return;
  }
  require(S_ISDIR(st.st_mode),"unsupported source tree member");
  new_directory(dst,st.st_mode&0777);
  dir=opendir(src); if(!dir)die(src);
  while((entry=readdir(dir))) {
    if(!strcmp(entry->d_name,".")||!strcmp(entry->d_name,".."))continue;
    join(a,src,entry->d_name); join(b,dst,entry->d_name); tree(a,b);
  }
  if(closedir(dir))die(src);
}
static char *load(const char *path,size_t *len) {
  FILE *f=fopen(path,"rb"); long n; char *text;
  if(!f)die(path); if(fseek(f,0,SEEK_END))die(path); n=ftell(f);
  require(n>=0&&n<=64*1024*1024,"source text exceeds bound");
  if(fseek(f,0,SEEK_SET))die(path);
  text=malloc((size_t)n+1); if(!text)die("malloc");
  require(fread(text,1,(size_t)n,f)==(size_t)n,"short original text read");
  if(fclose(f))die(path); text[n]=0; require(!memchr(text,0,(size_t)n),"text contains NUL"); *len=(size_t)n; return text;
}
static FILE *new_file(const char *path) {int fd=open(path,O_WRONLY|O_CREAT|O_EXCL,0644); FILE *f;if(fd<0)die(path);f=fdopen(fd,"wb");if(!f)die(path);return f;}
static void put(FILE *f,const char *p,size_t n) {if(fwrite(p,1,n,f)!=n)die("write");}
int main(int argc,char **argv) {
  require(argc>=3,"missing source transform operation");
  if(!strcmp(argv[1],"copy")) {require(argc==4,"copy SRC DEST");copy_file(argv[2],argv[3]);}
  else if(!strcmp(argv[1],"copy-checked")) {require(argc==5,"copy-checked SRC DEST ANCESTRY_FILE");equal_files(argv[2],argv[4]);copy_file(argv[2],argv[3]);}
  else if(!strcmp(argv[1],"tree")) {require(argc==4,"tree SRC DEST");tree(argv[2],argv[3]);}
  else if(!strcmp(argv[1],"overlay")) {char path[PATH_MAX];require(argc==6,"overlay SRC_TREE OUT_TREE MEMBER FILE");require(argv[4][0]!='/'&&!strstr(argv[4],".."),"invalid overlay member");tree(argv[2],argv[3]);join(path,argv[3],argv[4]);copy_file(argv[5],path);}
  else if(!strcmp(argv[1],"mkdir")) {require(argc==3,"mkdir DEST");new_directory(argv[2],0755);}
  else if(!strcmp(argv[1],"concat")) {
    char buf[16384];size_t n;int i;FILE *out=new_file(argv[2]);require(argc>=4,"concat DEST INPUT...");
    for(i=3;i<argc;i++) {FILE *in=fopen(argv[i],"rb");if(!in)die(argv[i]);while((n=fread(buf,1,sizeof buf,in)))put(out,buf,n);if(ferror(in)||fclose(in))die(argv[i]);}if(fclose(out))die(argv[2]);
  } else if(!strcmp(argv[1],"replace")||!strcmp(argv[1],"replace-absent")||!strcmp(argv[1],"replace-files-absent")) {
    size_t n,old_len,new_len,count=0;char *text,*p,*found,*end,*old_file=NULL,*new_file_text=NULL;const char *old,*replacement;unsigned long expected;FILE *out;
    require(argc==7,"replace SRC DEST OLD NEW COUNT");
    if(!strcmp(argv[1],"replace-files-absent")){old=old_file=load(argv[4],&old_len);replacement=new_file_text=load(argv[5],&new_len);}else{old=argv[4];replacement=argv[5];old_len=strlen(old);new_len=strlen(replacement);}
    require(old_len!=0,"empty replacement pattern");
    errno=0;expected=strtoul(argv[6],&end,10);require(!errno&&*argv[6]&&!*end,"invalid replacement count");
    text=load(argv[2],&n);for(p=text;(found=strstr(p,old));p=found+old_len)count++;
    if(strcmp(argv[1],"replace"))require(new_len&&!strstr(text,replacement),"replacement definition already exists");
    require(count==expected,"original replacement occurrence count changed");out=new_file(argv[3]);
    for(p=text;(found=strstr(p,old));p=found+old_len) {put(out,p,(size_t)(found-p));put(out,replacement,new_len);}put(out,p,n-(size_t)(p-text));if(fclose(out))die(argv[3]);free(text);free(old_file);free(new_file_text);
  } else if(!strcmp(argv[1],"remove-lines")) {
    size_t n,prefix;char *text,*p,*end,*stop;FILE *out;
    require(argc==5,"remove-lines SRC DEST PREFIX");text=load(argv[2],&n);prefix=strlen(argv[4]);require(prefix!=0,"empty line prefix");stop=text+n;out=new_file(argv[3]);
    for(p=text;p<stop;p=end) {end=memchr(p,'\n',(size_t)(stop-p));end=end?end+1:stop;if((size_t)(end-p)<prefix||memcmp(p,argv[4],prefix))put(out,p,(size_t)(end-p));}if(fclose(out))die(argv[3]);free(text);
  } else {require(0,"unknown source transform operation");}
  return 0;
}
