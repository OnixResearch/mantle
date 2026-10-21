/* Source-only adaptation of authenticated original Flex releases. No child
 * launch or replacement scanner/filter engine is present. The original tee
 * pipe and default main remain selected outside explicit source-phase modes. */
#define _GNU_SOURCE 1
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <limits.h>
#include <errno.h>
#include <dirent.h>
#include <sys/stat.h>
#include <unistd.h>

#define TEXT_MAX (16U * 1024U * 1024U)
#define TREE_MAX (UINT64_C(4) * 1024U * 1024U * 1024U)
#define ENTRY_MAX 100000U
static uint64_t copied_bytes;
static unsigned copied_entries;
static void fail(const char *message) { fprintf(stderr,"native Flex source adaptation: %s: %s\n",message,strerror(errno)); exit(1); }
static void require(int value,const char *message) { if(!value) { errno=0; fail(message); } }
static void join(char path[PATH_MAX],const char *root,const char *name) {
    int n=snprintf(path,PATH_MAX,"%s/%s",root,name); require(n>=0&&(size_t)n<PATH_MAX,"source path bound");
}
static void copy_file(const char *from,const char *to,mode_t mode) {
    FILE *in=fopen(from,"rb"),*out; if(!in)fail(from);
    out=fopen(to,"wb");if(!out)fail(to);
    unsigned char block[65536];size_t n;
    while((n=fread(block,1,sizeof block,in))) { copied_bytes+=n;require(copied_bytes<=TREE_MAX,"source tree byte bound");if(fwrite(block,1,n,out)!=n)fail(to); }
    if(ferror(in)||fclose(in)||fclose(out)||chmod(to,(mode&0777)|0600))fail("source copy");
}
static void copy_tree(const char *from,const char *to) {
    struct stat st;require(++copied_entries<=ENTRY_MAX,"source tree entry bound");if(lstat(from,&st))fail(from);
    if(S_ISDIR(st.st_mode)) {
        if(mkdir(to,0755)&&errno!=EEXIST)fail(to);
        struct stat target;if(lstat(to,&target))fail(to);require(S_ISDIR(target.st_mode),"source directory destination");
        DIR *dir=opendir(from);if(!dir)fail(from);struct dirent *entry;
        while((entry=readdir(dir))) { if(!strcmp(entry->d_name,".")||!strcmp(entry->d_name,".."))continue;
            char a[PATH_MAX],b[PATH_MAX];join(a,from,entry->d_name);join(b,to,entry->d_name);copy_tree(a,b);
        }
        if(closedir(dir))fail(from);
    } else if(S_ISREG(st.st_mode)) copy_file(from,to,st.st_mode);
    else if(S_ISLNK(st.st_mode)) { char target[PATH_MAX];ssize_t n=readlink(from,target,sizeof target-1);if(n<0)fail(from);require((size_t)n<sizeof target-1,"symlink bound");target[n]=0;if(symlink(target,to))fail(to); }
    else require(0,"special original source member");
}
static char *load(const char *path) {
    struct stat st;if(lstat(path,&st))fail(path);require(S_ISREG(st.st_mode)&&st.st_size>=0&&(uint64_t)st.st_size<=TEXT_MAX,"source text bound");
    size_t n=(size_t)st.st_size;char *text=malloc(n+1);if(!text)fail("source text allocation");FILE *f=fopen(path,"rb");if(!f)fail(path);
    require(fread(text,1,n,f)==n&&fgetc(f)==EOF&&!ferror(f),"source text read");if(fclose(f))fail(path);require(!memchr(text,0,n),"source text NUL");text[n]=0;return text;
}
static void save(const char *path,const char *text) {
    size_t n=strlen(text);require(n<=TEXT_MAX,"adapted source text bound");FILE *f=fopen(path,"wb");if(!f)fail(path);
    if(fwrite(text,1,n,f)!=n||fclose(f))fail(path);
}
static char *unique(char *text,const char *needle) {
    require(*needle!=0,"empty original source seam");char *at=strstr(text,needle);require(at&&!strstr(at+strlen(needle),needle),"unique original source seam");return at;
}
static void replace(char **text,const char *needle,const char *value) {
    size_t offset=(size_t)(unique(*text,needle)-*text),old=strlen(needle),added=strlen(value),n=strlen(*text);
    require(n-old<=TEXT_MAX&&added<=TEXT_MAX-(n-old),"adapted source size");char *next=realloc(*text,n-old+added+1);if(!next)fail("adapted source allocation");
    memmove(next+offset+added,next+offset+old,n-offset-old+1);memcpy(next+offset,value,added);*text=next;
}
static char *concatenate(const char *a,const char *b,const char *c) {
    size_t na=strlen(a),nb=strlen(b),nc=strlen(c);require(na<=TEXT_MAX&&nb<=TEXT_MAX-na&&nc<=TEXT_MAX-na-nb,"phase source concatenation bound");
    char *text=malloc(na+nb+nc+1);if(!text)fail("phase source allocation");memcpy(text,a,na);memcpy(text+na,b,nb);memcpy(text+na+nb,c,nc+1);return text;
}
static char *netstring(char **cursor) {
    char *p=*cursor;size_t n=0;require(*p>='0'&&*p<='9',"phase seam length");
    if(*p=='0')require(p[1]==':',"canonical phase seam length");
    while(*p>='0'&&*p<='9') { require(n<=(TEXT_MAX-(unsigned)(*p-'0'))/10,"phase seam size");n=n*10+(unsigned)(*p++-'0'); }
    require(*p++==':'&&strlen(p)>n&&p[n]==',',"phase seam delimiter");char *value=p;p[n]=0;*cursor=p+n+1;return value;
}
int main(int argc,char **argv) {
    if(argc==5&&!strcmp(argv[1],"gcc-source")) {
        copy_tree(argv[2],argv[3]);char path[PATH_MAX];join(path,argv[3],"gcc/collect2.c");char *text=load(argv[4]);save(path,text);free(text);
        return 0;
    }
    if(argc==6&&!strcmp(argv[1],"ld-source")) {
        copy_tree(argv[2],argv[3]);
        char directory[PATH_MAX],path[PATH_MAX];join(directory,argv[3],argv[5]);join(path,directory,"ld/ldmain.c");
        char *text=load(path),*publisher=load(argv[4]);
        const char *old_close="      if (!bfd_close (link_info.output_bfd))\n\teinfo (_(\"%F%B: final close failed: %E\\n\"), link_info.output_bfd);\n";
        const char *new_close="      if (!bfd_close (obfd))\n\teinfo (_(\"%F%P: %s: final close failed: %E\\n\"), output_filename);\n";
        const char *seam=strstr(text,old_close)?old_close:new_close;
        char *closed=concatenate(seam,"      mantle_native_generated_created (output_filename);\n","");
        replace(&text,seam,closed);free(closed);
        char *declared=concatenate("extern void mantle_native_generated_created (const char *);\n",text,"\n");free(text);
        text=concatenate(declared,publisher,"\n");save(path,text);free(declared);free(publisher);free(text);return 0;
    }
    if(argc==6&&!strcmp(argv[1],"bison-source")) {
        copy_tree(argv[2],argv[3]);
        char directory[PATH_MAX],path[PATH_MAX];join(directory,argv[3],argv[5]);join(path,directory,"src/output.c");
        char *text=load(path),*phase=load(argv[4]);
        const char *seam="static void\noutput_skeleton (void)\n{";
        replace(&text,seam,"#include \"mantle-bison-phase.inc\"\n\nstatic void\noutput_skeleton (void)\n{\n  if (getenv (\"MANTLE_BISON_MODE\"))\n    {\n      mantle_bison_output_skeleton ();\n      return;\n    }");
        save(path,text);free(text);join(path,directory,"src/mantle-bison-phase.inc");save(path,phase);free(phase);
        join(path,directory,"src/main.c");text=load(path);
        replace(&text,"main (int argc, char *argv[])\n{","main (int argc, char *argv[])\n{\n  if (getenv (\"MANTLE_BISON_MODE\"))\n    {\n      char const *cwd = getenv (\"MANTLE_PHASE_CWD\");\n      size_t cwd_length; int i;\n      if (!cwd || chdir (cwd))\n        error (EXIT_FAILURE, errno, \"original Bison source cwd\");\n      cwd_length = strlen (cwd);\n      for (i = 1; i < argc; ++i)\n        if (strncmp (argv[i], cwd, cwd_length) == 0 && argv[i][cwd_length] == '/')\n          argv[i] += cwd_length + 1;\n    }");
        save(path,text);free(text);return 0;
    }
    if(argc==4&&!strcmp(argv[1],"copy")) { copy_tree(argv[2],argv[3]);return 0; }
    require(argc==8,"usage: source-patch SOURCE DESTINATION CAPTURE TEE MAIN SEAMS CHILD");
    copy_tree(argv[1],argv[2]);
    char path[PATH_MAX];join(path,argv[2],"src/main.c");struct stat st;
    const char *prefix=lstat(path,&st)==0?"src/":"";
    char member[64];snprintf(member,sizeof member,"%sfilter.c",prefix);join(path,argv[2],member);char *filter=load(path);
    char *capture=load(argv[3]),*tee=load(argv[4]),*main_phase=load(argv[5]),*seams=load(argv[6]),*cursor=seams;
    char *start=netstring(&cursor),*end=netstring(&cursor),*wait=netstring(&cursor);require(!*cursor,"trailing phase source seam");
    const char *signature="bool filter_apply_chain (struct filter * chain)\n{";
    char *capture_body=concatenate("#include \"mantle-flex-child.inc\"\n\n",signature,"\nif (getenv(\"MANTLE_FLEX_STATE\")) mantle_flex_capture_child(chain);\n");
    char *insertion=concatenate(capture_body,capture,"");replace(&filter,signature,insertion);free(insertion);free(capture_body);free(capture);
    char *first=unique(filter,start),*last=unique(filter,end);require(last>first,"original tee source region order");
    size_t length=(size_t)(last-first);char *pipes=malloc(length+1);if(!pipes)fail("original tee source allocation");memcpy(pipes,first,length);pipes[length]=0;
    char *branch=concatenate("if (getenv(\"MANTLE_FLEX_TEE\")) {\n",tee,"\n} else {\n"),*body=concatenate(branch,pipes,"\n}\n");free(branch);free(tee);
    replace(&filter,pipes,body);free(pipes);free(body);
    if(!*prefix)wait="\twhile (wait (0) > 0) ;\n\n\texit (0);\n\treturn 0;";
    insertion=concatenate("if (getenv(\"MANTLE_FLEX_TEE\")) return 0;\n",wait,"");replace(&filter,wait,insertion);free(insertion);free(seams);save(path,filter);free(filter);
    snprintf(member,sizeof member,"%smantle-flex-child.inc",prefix);join(path,argv[2],member);char *child=load(argv[7]);save(path,child);free(child);
    snprintf(member,sizeof member,"%smain.c",prefix);join(path,argv[2],member);char *main_source=load(path);
    if(*prefix)replace(&main_source,"int main (int argc, char *argv[])\n{","int mantle_flex_original_main (int argc, char *argv[])\n{");
    else replace(&main_source,"int main (argc, argv)\n     int argc;\n     char   *argv[];\n{","int mantle_flex_original_main (argc, argv)\n     int argc;\n     char   *argv[];\n{");
    char *adapted=concatenate(main_source,"\n",main_phase);save(path,adapted);free(adapted);free(main_source);free(main_phase);return 0;
}
