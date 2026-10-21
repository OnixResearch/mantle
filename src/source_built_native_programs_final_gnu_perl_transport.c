/* Optional original-program child boundary, embedded in genuine Perl/M4 source
 * and the native controller. No child is launched and no language is replaced.
 * Full rows preserve closed slots, OFD aliases, status flags, offsets, CLOEXEC,
 * genuine null and exact workspace bindings. */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>
#include <fcntl.h>
#include <unistd.h>
#include <dirent.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <limits.h>
#include <stdint.h>
#include <sys/sysmacros.h>

extern char **environ;
#define MP_LIMIT (64UL * 1024UL * 1024UL)
#define MP_TREE_LIMIT (UINT64_C(4) * 1024 * 1024 * 1024)
static unsigned long mp_ordinal;
static uint64_t mp_copied;
static unsigned long mp_entries;
static void mp_fail(const char *why)
{ fprintf(stderr,"original native child boundary: %s: %s\n",why,strerror(errno));abort(); }
static char *mp_join(const char *a,const char *b)
{
    size_t n,m;char *p;n=strlen(a);m=strlen(b);
    if(n>65536||m>65536||n+m+2>131074)mp_fail("path bound");
    p=(char *)malloc(n+m+2);if(!p)mp_fail("path allocation");
    memcpy(p,a,n);p[n++]='/';memcpy(p+n,b,m+1);return p;
}
static void mp_close(FILE *f)
{ if(ferror(f)||fclose(f))mp_fail("stream close"); }
static void mp_record(FILE *f,const char *value)
{
    size_t n;n=strlen(value);if(n>MP_LIMIT||fprintf(f,"%lu:",(unsigned long)n)<0||fwrite(value,1,n,f)!=n||fputc(',',f)==EOF)mp_fail("request record");
}
static void mp_number(FILE *f,long value)
{ char text[64];sprintf(text,"%ld",value);mp_record(f,text); }
static void mp_transfer(FILE *in,FILE *out)
{
    unsigned char block[16384];size_t n;
    while((n=fread(block,1,sizeof block,in))!=0){mp_copied+=n;if(mp_copied>MP_TREE_LIMIT||fwrite(block,1,n,out)!=n)mp_fail("bounded byte copy");}
    if(ferror(in))mp_fail("byte copy read");
}
static int mp_inside(const char *path,const char *root)
{ size_t n;n=strlen(root);return !strncmp(path,root,n)&&(!path[n]||path[n]=='/'); }
static void mp_remove(const char *path,const char *excluded)
{
    struct stat st;DIR *d;struct dirent *entry;char *child;
    if(excluded&&mp_inside(path,excluded))return;
    if(lstat(path,&st)){if(errno==ENOENT)return;mp_fail("state remove stat");}
    if(S_ISDIR(st.st_mode)){
        d=opendir(path);if(!d)mp_fail("state remove directory");
        while((entry=readdir(d))!=NULL){if(!strcmp(entry->d_name,".")||!strcmp(entry->d_name,".."))continue;child=mp_join(path,entry->d_name);mp_remove(child,excluded);free(child);}
        if(closedir(d))mp_fail("state remove close");
        if(excluded&&mp_inside(excluded,path))return;
        if(rmdir(path))mp_fail("state remove directory completion");
    }else if(unlink(path))mp_fail("state remove file");
}
static void mp_tree(const char *from,const char *to,const char *excluded,int synchronize)
{
    struct stat source,target;int present;DIR *d;struct dirent *entry;char *a,*b;FILE *in,*out;
    struct timespec times[2];char link[65537];ssize_t length;
    if(excluded&&mp_inside(synchronize?to:from,excluded))return;
    if(++mp_entries>1000000)mp_fail("state entry bound");
    if(lstat(from,&source))mp_fail("state source stat");
    present=lstat(to,&target)==0;if(!present&&errno!=ENOENT)mp_fail("state target stat");
    if(present&&((source.st_mode&S_IFMT)!=(target.st_mode&S_IFMT))){mp_remove(to,excluded);present=0;}
    if(S_ISDIR(source.st_mode)){
        if(!present&&mkdir(to,0700))mp_fail("state directory create");
        if(synchronize){
            d=opendir(to);if(!d)mp_fail("state target directory");
            while((entry=readdir(d))!=NULL){
                if(!strcmp(entry->d_name,".")||!strcmp(entry->d_name,".."))continue;
                a=mp_join(from,entry->d_name);b=mp_join(to,entry->d_name);
                if(lstat(a,&target)){if(errno!=ENOENT)mp_fail("state compare member");mp_remove(b,excluded);}free(a);free(b);
            }
            if(closedir(d))mp_fail("state target directory close");
        }
        d=opendir(from);if(!d)mp_fail("state source directory");
        while((entry=readdir(d))!=NULL){if(!strcmp(entry->d_name,".")||!strcmp(entry->d_name,".."))continue;a=mp_join(from,entry->d_name);b=mp_join(to,entry->d_name);mp_tree(a,b,excluded,synchronize);free(a);free(b);}
        if(closedir(d)||chmod(to,source.st_mode&0777))mp_fail("state directory completion");
    }else if(S_ISREG(source.st_mode)){
        /* Updating an existing inode is essential: original Perl still owns
         * open handles to files changed by the completed child. */
        in=fopen(from,"rb");out=fopen(to,"wb");if(!in||!out)mp_fail("state regular file");mp_transfer(in,out);mp_close(in);mp_close(out);if(chmod(to,source.st_mode&0777))mp_fail("state file mode");
    }else if(S_ISLNK(source.st_mode)){
        length=readlink(from,link,sizeof link-1);if(length<0||(size_t)length>=sizeof link-1)mp_fail("state symlink");link[length]=0;
        if(present&&unlink(to))mp_fail("state old symlink");if(symlink(link,to))mp_fail("state symlink create");
    }else mp_fail("unsupported state file kind");
    times[0]=source.st_atim;times[1]=source.st_mtim;
    if(utimensat(AT_FDCWD,to,times,AT_SYMLINK_NOFOLLOW))mp_fail("state original timestamps");
}
static void mp_copy_workspace(const char *from,const char *to,const char *excluded,int synchronize)
{ mp_copied=0;mp_entries=0;mp_tree(from,to,excluded,synchronize); }
#define MP_DESCRIPTORS 64
typedef struct {
    int aliases[MP_DESCRIPTORS],flags[MP_DESCRIPTORS],cloexec[MP_DESCRIPTORS],nulls[MP_DESCRIPTORS];
    long positions[MP_DESCRIPTORS];
    char *members[MP_DESCRIPTORS];
} mp_descriptors;
static int mp_null(int fd)
{
    struct stat st;if(fstat(fd,&st))mp_fail("actual descriptor backing");
    if(S_ISREG(st.st_mode))return 0;
    if(S_ISCHR(st.st_mode)&&major(st.st_rdev)==1&&minor(st.st_rdev)==3)return 1;
    mp_fail("unsupported original descriptor backing type");return 0;
}
static int mp_flags(int fd)
{
    int flags;flags=fcntl(fd,F_GETFL);if(flags<0)mp_fail("actual descriptor status flags");
    if((flags&~(O_ACCMODE|O_APPEND|O_NONBLOCK|0x8000))||(flags&O_ACCMODE)==O_ACCMODE)mp_fail("unsupported original descriptor status flags");
    return flags&~0x8000;
}
static int mp_same_description(int a,int b)
{
    struct stat x,y;int before,after,flags;
    if(fstat(a,&x)||fstat(b,&y))mp_fail("actual descriptor identity");
    if(x.st_dev!=y.st_dev||x.st_ino!=y.st_ino||mp_flags(a)!=mp_flags(b))return 0;
    before=fcntl(a,F_GETFL);flags=fcntl(b,F_GETFL);if(before<0||flags<0)mp_fail("descriptor alias flags");
    if(fcntl(a,F_SETFL,before^O_NONBLOCK))mp_fail("descriptor alias probe");
    after=fcntl(b,F_GETFL);if(fcntl(a,F_SETFL,before)||after<0)mp_fail("descriptor alias restore");
    return ((after^flags)&O_NONBLOCK)!=0;
}
static int mp_clean_member(const char *member)
{
    const char *p,*end;if(!*member||*member=='/')return 0;
    for(p=member;*p;p=end+1){end=strchr(p,'/');if(!end)end=p+strlen(p);
        if(end==p||(end-p==1&&p[0]=='.')||(end-p==2&&p[0]=='.'&&p[1]=='.'))return 0;
        if(!*end)return 1;if(!end[1])return 0;
    }
    return 0;
}
static char *mp_workspace_member(int fd,const char *workspace)
{
    struct stat opened,current;char proc[64],target[PATH_MAX],*member;size_t prefix;ssize_t n;
    if(fstat(fd,&opened))mp_fail("source descriptor inode");
    if(!S_ISREG(opened.st_mode)||!opened.st_nlink){member=strdup("");if(!member)mp_fail("descriptor member allocation");return member;}
    sprintf(proc,"/proc/self/fd/%d",fd);n=readlink(proc,target,sizeof target-1);if(n<0||n==(ssize_t)sizeof target-1)mp_fail("source descriptor path");target[n]=0;
    prefix=strlen(workspace);member=NULL;
    if(!strncmp(target,workspace,prefix)&&target[prefix]=='/'&&target[prefix+1]){
        if(lstat(target,&current)){if(errno!=ENOENT)mp_fail("source descriptor path inode");}
        else if(S_ISREG(current.st_mode)&&opened.st_dev==current.st_dev&&opened.st_ino==current.st_ino){
            if(!mp_clean_member(target+prefix+1))mp_fail("source descriptor workspace member");member=target+prefix+1;
        }
    }
    member=strdup(member?member:"");if(!member)mp_fail("descriptor member allocation");return member;
}
static void mp_inventory(mp_descriptors *table,int payload,const char *workspace)
{
    DIR *directory;struct dirent *entry;char *end;long fd;int own,i,j,flags;off_t position;
    directory=opendir("/proc/self/fd");if(!directory)mp_fail("actual descriptor inventory");own=dirfd(directory);
    while((entry=readdir(directory))!=NULL){
        errno=0;fd=strtol(entry->d_name,&end,10);if(errno||*end||fd<MP_DESCRIPTORS||fd==own||fd==payload)continue;
        flags=fcntl((int)fd,F_GETFD);if(flags<0)mp_fail("actual descriptor inventory flags");
        if(!(flags&FD_CLOEXEC))mp_fail("inherited descriptor exceeds source wire slot bound");
    }
    if(closedir(directory))mp_fail("actual descriptor inventory close");memset(table,0,sizeof *table);
    for(i=0;i<MP_DESCRIPTORS;i++){
        table->aliases[i]=-1;if(i==payload)continue;
        flags=fcntl(i,F_GETFD);if(flags<0){if(errno==EBADF)continue;mp_fail("actual descriptor close-on-exec flags");}
        table->cloexec[i]=(flags&FD_CLOEXEC)!=0;table->flags[i]=mp_flags(i);table->nulls[i]=mp_null(i);
        position=lseek(i,0,SEEK_CUR);if(position<0||position>LONG_MAX)mp_fail("actual descriptor offset");table->positions[i]=(long)position;
        table->aliases[i]=i;for(j=0;j<i;j++)if(table->aliases[j]==j&&mp_same_description(j,i)){table->aliases[i]=j;break;}
        table->members[i]=table->aliases[i]==i?mp_workspace_member(i,workspace):table->members[table->aliases[i]];
    }
}
static FILE *mp_descriptor_reader(int fd)
{
    char path[64];int reader;FILE *stream;sprintf(path,"/proc/self/fd/%d",fd);
    reader=open(path,O_RDONLY|O_CLOEXEC);if(reader<0)mp_fail("independent descriptor snapshot");
    stream=fdopen(reader,"rb");if(!stream)mp_fail("independent descriptor stream");return stream;
}
static char *mp_descriptor_path(const char *root,const char *prefix,int fd)
{ char name[80];sprintf(name,"%sdescriptor-%d",prefix,fd);return mp_join(root,name); }
static void mp_snapshot(int fd,const char *path)
{
    FILE *input,*output;input=mp_descriptor_reader(fd);output=fopen(path,"wb");if(!output)mp_fail("descriptor snapshot output");
    mp_transfer(input,output);mp_close(input);mp_close(output);
}
static void mp_descriptor_rows(FILE *f,const mp_descriptors *table)
{
    int i;for(i=0;i<MP_DESCRIPTORS;i++){
        mp_number(f,table->aliases[i]);if(table->aliases[i]==i){mp_number(f,table->positions[i]);mp_number(f,table->flags[i]);mp_number(f,table->nulls[i]);mp_record(f,table->members[i]);}
        if(table->aliases[i]>=0)mp_number(f,table->cloexec[i]);
    }
}
static char *mp_read_member(FILE *f)
{
    size_t n;int c;char *text;n=0;c=fgetc(f);if(c<'0'||c>'9')mp_fail("descriptor member length");
    do{if(n>(PATH_MAX-1-(unsigned)(c-'0'))/10)mp_fail("descriptor member length bound");n=n*10+c-'0';c=fgetc(f);}while(c>='0'&&c<='9');
    text=(char *)malloc(n+1);if(!text)mp_fail("descriptor member allocation");
    if(c!=':'||fread(text,1,n,f)!=n||fgetc(f)!=','||memchr(text,0,n))mp_fail("descriptor member record");text[n]=0;
    if(n&&!mp_clean_member(text))mp_fail("descriptor member syntax");return text;
}
static long mp_read_number(FILE *f,long minimum,long maximum)
{
    char text[80],*end;size_t n;int c;long value;
    n=0;c=fgetc(f);if(c<'0'||c>'9')mp_fail("descriptor scalar record length");
    do{if(n>(sizeof text-2-(unsigned)(c-'0'))/10)mp_fail("descriptor scalar record bound");n=n*10+c-'0';c=fgetc(f);}while(c>='0'&&c<='9');
    if(c!=':'||!n||fread(text,1,n,f)!=n||fgetc(f)!=','||memchr(text,0,n))mp_fail("descriptor scalar record");text[n]=0;
    errno=0;value=strtol(text,&end,10);if(errno||end==text||*end||value<minimum||value>maximum)mp_fail("descriptor scalar value");return value;
}
static int mp_equal(FILE *a,FILE *b);
static void mp_restore_descriptor(int fd,int flags,long position,int null_device,const char *path)
{
    FILE *current,*saved;char proc[64];unsigned char bytes[16384];size_t n,at;ssize_t written;off_t offset;int writer;
    if(mp_null(fd)!=null_device)mp_fail("child changed original descriptor backing");
    if(!null_device){
        current=mp_descriptor_reader(fd);saved=fopen(path,"rb");if(!saved)mp_fail("actual descriptor reply");
        if(!mp_equal(current,saved)){
            sprintf(proc,"/proc/self/fd/%d",fd);writer=open(proc,O_WRONLY|O_CLOEXEC);if(writer<0)mp_fail("restore actual descriptor bytes");
            if(fseek(saved,0,SEEK_SET))mp_fail("actual descriptor reply rewind");offset=0;
            while((n=fread(bytes,1,sizeof bytes,saved))!=0){at=0;while(at<n){written=pwrite(writer,bytes+at,n-at,offset+at);if(written<0&&errno==EINTR)continue;if(written<=0)mp_fail("actual descriptor reply write");at+=(size_t)written;}offset+=(off_t)n;}
            if(ferror(saved)||ftruncate(writer,offset)||close(writer))mp_fail("actual descriptor reply completion");
        }
        mp_close(current);mp_close(saved);
    }
    if(fcntl(fd,F_SETFL,flags)||lseek(fd,position,SEEK_SET)!=position)mp_fail("restore actual descriptor flags and offset");
}
static int mp_private_environment(const char *entry)
{
    return !strncmp(entry,"MANTLE_COMMAND_",15)||!strncmp(entry,"MANTLE_PERL_PHASE_ROOT=",23)||!strncmp(entry,"MANTLE_PERL_PHASE_REPLIES=",26)||!strncmp(entry,"MANTLE_PERL_PHASE_WORKSPACE=",28);
}
static int mp_environment_order(const void *a,const void *b)
{ return strcmp(*(const char *const *)a,*(const char *const *)b); }
static int mp_equal(FILE *a,FILE *b)
{
    int x,y;if(fseek(a,0,SEEK_SET)||fseek(b,0,SEEK_SET))mp_fail("request compare rewind");
    do{x=fgetc(a);y=fgetc(b);if(x!=y)return 0;}while(x!=EOF);if(ferror(a)||ferror(b))mp_fail("request compare read");return 1;
}
static long mp_decimal_file(const char *path,long maximum)
{
    FILE *f;char text[80],*end;size_t n;long value;f=fopen(path,"rb");if(!f)mp_fail("actual reply scalar");
    n=fread(text,1,sizeof text-1,f);if(fgetc(f)!=EOF||ferror(f))mp_fail("reply scalar bound");text[n]=0;mp_close(f);
    errno=0;value=strtol(text,&end,10);if(errno||end==text||value<0||value>maximum||(*end&&strcmp(end,"\n")))mp_fail("actual reply scalar syntax");return value;
}
static void mp_restore_reply(const char *reply,const mp_descriptors *table)
{
    char *path,*member;FILE *positions,*current,*saved;int i,alias,flags,null_device,cloexec;long position;
    for(i=0;i<MP_DESCRIPTORS;i++)if(table->aliases[i]==i&&!table->nulls[i]){
        path=mp_descriptor_path(reply,"input-",i);current=mp_descriptor_reader(i);saved=fopen(path,"rb");
        if(!saved||!mp_equal(current,saved))mp_fail("original inherited descriptor changed on replay");mp_close(current);mp_close(saved);free(path);
    }
    path=mp_join(reply,"descriptors");positions=fopen(path,"rb");if(!positions)mp_fail("actual descriptor reply table");free(path);
    for(i=0;i<MP_DESCRIPTORS;i++){
        alias=(int)mp_read_number(positions,-1,i);if(alias!=table->aliases[i])mp_fail("actual child reply changed parent descriptor aliases");
        if(alias==i){
            position=mp_read_number(positions,0,LONG_MAX);flags=(int)mp_read_number(positions,0,O_ACCMODE|O_APPEND|O_NONBLOCK);
            null_device=(int)mp_read_number(positions,0,1);if((flags&~(O_ACCMODE|O_APPEND|O_NONBLOCK))||(flags&O_ACCMODE)==O_ACCMODE)mp_fail("actual descriptor reply flags");
            if((flags&O_ACCMODE)!=(table->flags[i]&O_ACCMODE))mp_fail("child changed original descriptor access");
            member=mp_read_member(positions);if(strcmp(member,table->members[i])||(null_device&&*member))mp_fail("actual descriptor reply binding");free(member);
            path=mp_descriptor_path(reply,"",i);mp_restore_descriptor(i,flags,position,null_device,path);free(path);
        }
        if(alias>=0){cloexec=(int)mp_read_number(positions,0,1);if(cloexec!=table->cloexec[i])mp_fail("child reply changed parent close-on-exec flags");}
    }
    if(fgetc(positions)!=EOF)mp_fail("trailing actual descriptor reply table");mp_close(positions);
}
int mantle_perl_transport_active(void)
{ return getenv("MANTLE_PERL_PHASE_ROOT")!=NULL; }
int mantle_perl_transport(const char *operation,const char *image,char *const *argv,int stdin_fd,char **stdout_path)
{
    const char *root,*replies,*workspace;char ordinal[64],*reply,*path,*cwd,*canonical_root;
    FILE *request,*saved;char **environment;size_t count,i;int fd,status;struct stat st;long arguments;mp_descriptors descriptors;
    mode_t creation_mask;
    root=getenv("MANTLE_PERL_PHASE_ROOT");replies=getenv("MANTLE_PERL_PHASE_REPLIES");workspace=getenv("MANTLE_PERL_PHASE_WORKSPACE");
    if(!root||!replies||!workspace||!image||!argv||!stdout_path)mp_fail("incomplete original child phase binding");
    if(strcmp(operation,"system")&&strcmp(operation,"readpipe")&&strcmp(operation,"writepipe")&&strcmp(operation,"exec"))mp_fail("original child operation");
    creation_mask=umask(0);umask(creation_mask);
    if(creation_mask>0777)mp_fail("actual child creation mask exceeds permission bits");
    *stdout_path=NULL;mp_inventory(&descriptors,stdin_fd,workspace);request=tmpfile();if(!request)mp_fail("request stream");
    /* CHILD-2 records the actual creation mask after cwd, before environment. */
    mp_record(request,"MANTLE-NATIVE-PERL-CHILD-2");mp_record(request,operation);mp_record(request,image);
    for(arguments=0;argv[arguments];arguments++)if(arguments>=16384)mp_fail("child argument count");mp_number(request,arguments);
    for(i=0;i<(size_t)arguments;i++){if(strlen(argv[i])>65536)mp_fail("child word bound");mp_record(request,argv[i]);}
    cwd=getcwd(NULL,0);if(!cwd)mp_fail("actual child cwd");mp_record(request,cwd);free(cwd);
    mp_number(request,(long)creation_mask);
    count=0;for(i=0;environ[i];i++)if(!mp_private_environment(environ[i]))count++;
    environment=(char **)malloc((count?count:1)*sizeof *environment);if(!environment)mp_fail("actual child environment allocation");
    count=0;for(i=0;environ[i];i++)if(!mp_private_environment(environ[i]))environment[count++]=environ[i];
    qsort(environment,count,sizeof *environment,mp_environment_order);mp_number(request,(long)count);
    for(i=0;i<count;i++)mp_record(request,environment[i]);free(environment);
    mp_descriptor_rows(request,&descriptors);mp_number(request,stdin_fd>=0?1:0);
    sprintf(ordinal,"%lu",mp_ordinal++);reply=mp_join(replies,ordinal);path=mp_join(reply,"request");
    if(!stat(path,&st)){
        saved=fopen(path,"rb");if(!saved||!mp_equal(request,saved))mp_fail("original child request changed on replay");mp_close(saved);free(path);mp_close(request);
        mp_restore_reply(reply,&descriptors);
        path=mp_join(reply,"workspace");canonical_root=realpath(root,NULL);if(!canonical_root)mp_fail("actual private phase root");
        cwd=strrchr(canonical_root,'/');if(!cwd||cwd==canonical_root)mp_fail("actual private continuation namespace");*cwd=0;
        mp_copy_workspace(path,workspace,canonical_root,1);free(path);free(canonical_root);
        path=mp_join(reply,"status");status=(int)mp_decimal_file(path,255);free(path);
        if(!strcmp(operation,"readpipe"))*stdout_path=mp_join(reply,"pipe-stdout");free(reply);
        for(fd=0;fd<MP_DESCRIPTORS;fd++)if(descriptors.aliases[fd]==fd)free(descriptors.members[fd]);return status<<8;
    }
    if(errno!=ENOENT)mp_fail("actual reply lookup");free(path);free(reply);
    path=mp_join(root,"request");saved=fopen(path,"wb");if(!saved||fseek(request,0,SEEK_SET))mp_fail("new original child request");mp_copied=0;mp_transfer(request,saved);mp_close(request);mp_close(saved);free(path);
    for(fd=0;fd<MP_DESCRIPTORS;fd++)if(descriptors.aliases[fd]==fd&&!descriptors.nulls[fd]){path=mp_descriptor_path(root,"",fd);mp_snapshot(fd,path);free(path);}
    if(stdin_fd>=0){path=mp_join(root,"stdin");mp_snapshot(stdin_fd,path);free(path);}
    _exit(86);return -1;
}
