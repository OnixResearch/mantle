/* Called only by an authenticated original compiler/link writer after its
 * output stream has successfully closed. Never invoked from an argv guess,
 * workspace walk, executable lookup, or interpreter replay callback. */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>
#include <sys/stat.h>
#include <unistd.h>
#include <limits.h>
static void native_generated_fail(const char *why)
{ fprintf(stderr,"native generated writer: %s: %s\n",why,strerror(errno));abort(); }
static size_t native_generated_record(const char **cursor,char *result,size_t capacity)
{
    const char *p=*cursor;size_t n=0;int digits=0;
    while(*p>='0'&&*p<='9'){if(++digits>1&&(*cursor)[0]=='0')native_generated_fail("record leading zero");if(n>(65536u-(unsigned)(*p-'0'))/10)native_generated_fail("record bound");n=n*10+(unsigned)(*p++-'0');}
    if(!digits||*p++!=':'||n>=capacity||strnlen(p,n+1)!=n+1||p[n]!=',')native_generated_fail("record syntax or bound");
    memcpy(result,p,n);result[n]=0;*cursor=p+n+1;return n;
}
static int native_generated_relative(const char *path)
{
    if(!*path||*path=='/'||strchr(path,'\\'))return 0;
    while(*path){const char *start=path;while(*path&&*path!='/')path++;size_t n=(size_t)(path-start);if(!n||(n==1&&*start=='.')||(n==2&&start[0]=='.'&&start[1]=='.'))return 0;if(*path&&!*++path)return 0;}return 1;
}
static void native_generated_join(char *result,size_t capacity,const char *a,const char *b)
{
    size_t n=strlen(a),m=strlen(b);if(n+1>=capacity||m>=capacity-n-1)native_generated_fail("path bound");memcpy(result,a,n);result[n++]='/';memcpy(result+n,b,m+1);
}
#ifdef __cplusplus
extern "C"
#endif
void mantle_native_generated_created(const char *filename)
{
    const char *root=getenv("MANTLE_NATIVE_GENERATED_ROOT"),*map=getenv("MANTLE_NATIVE_GENERATED_MAP");
    if(!root&&!map)return;
    if(!root||!map)native_generated_fail("incomplete original writer binding");
    char actual[PATH_MAX];if(!realpath(filename,actual))native_generated_fail("actual closed output path");
    struct stat st;if(stat(actual,&st)||!S_ISREG(st.st_mode))native_generated_fail("actual writer output kind");
    /* Relocatable objects and nonexecutables are not executable capabilities. */
    if(!(st.st_mode&0111))return;
    FILE *source=fopen(actual,"rb");unsigned char elf[18];if(!source||fread(elf,1,sizeof elf,source)!=sizeof elf)native_generated_fail("actual executable header");
    if(memcmp(elf,"\177ELF",4)||elf[5]!=1||(elf[4]!=1&&elf[4]!=2)||elf[17])native_generated_fail("actual executable format");
    if(elf[16]==1){if(fclose(source))native_generated_fail("actual relocatable output close");return;}
    if(elf[16]!=2&&elf[16]!=3)native_generated_fail("actual executable type");
    if(fseek(source,0,SEEK_SET))native_generated_fail("actual executable rewind");
    const char *cursor=map;char magic[64];native_generated_record(&cursor,magic,sizeof magic);if(strcmp(magic,"MANTLE-NATIVE-GENERATED-ROOTS-1"))native_generated_fail("writer root map version");
    char relative[PATH_MAX];relative[0]=0;
    while(*cursor){char logical[PATH_MAX],target[PATH_MAX];size_t n=native_generated_record(&cursor,logical,sizeof logical);native_generated_record(&cursor,target,sizeof target);
        if(!*logical||logical[0]!='/'||!native_generated_relative(target))native_generated_fail("writer root map member");
        const char *suffix=NULL;
        if(!strncmp(actual,logical,n)&&(actual[n]=='/'||!actual[n]))suffix=actual+n;
        if(suffix){
            if(*relative)native_generated_fail("overlapping writer roots");
            while(*suffix=='/')suffix++;
            if(*suffix)native_generated_join(relative,sizeof relative,target,suffix);else memcpy(relative,target,strlen(target)+1);
        }
    }
    if(!native_generated_relative(relative))native_generated_fail("actual writer output leaves bound workspace roots");
    char destination[PATH_MAX];native_generated_join(destination,sizeof destination,root,relative);
    char *p;
    for(p=destination+strlen(root)+1;*p;p++)if(*p=='/'){
        *p=0;struct stat parent;if(lstat(destination,&parent)){if(errno!=ENOENT||mkdir(destination,0755))native_generated_fail("generated parent creation");}
        else if(!S_ISDIR(parent.st_mode))native_generated_fail("generated parent is not a directory");*p='/';
    }
    FILE *output=fopen(destination,"wbx");if(!output)native_generated_fail("exclusive genuine output publication");
    unsigned char block[16384];size_t n;while((n=fread(block,1,sizeof block,source)))if(fwrite(block,1,n,output)!=n)native_generated_fail("actual writer byte publication");
    if(ferror(source)||fclose(source)||fclose(output)||chmod(destination,st.st_mode&0777))native_generated_fail("actual writer publication completion");
}
