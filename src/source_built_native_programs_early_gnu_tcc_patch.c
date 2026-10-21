/* Transform only authenticated TinyCC source. The original recipe still
 * compiles/links this tree and performs its original runtime probes. */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#define SOURCE_MAX (16u*1024u*1024u)
static void fail(const char *why){fprintf(stderr,"native TinyCC source boundary: %s\n",why);exit(1);}
static char *load(const char *path,size_t *size){FILE *f=fopen(path,"rb");char *p=malloc(SOURCE_MAX+1);if(!f||!p)fail("opening source");*size=fread(p,1,SOURCE_MAX+1,f);if(*size>SOURCE_MAX||ferror(f)||fclose(f))fail("reading bounded source");p[*size]=0;if(memchr(p,0,*size))fail("embedded source NUL");return p;}
static void emit(FILE *f,const char *p,size_t n){if(fwrite(p,1,n,f)!=n)fail("writing source");}
static void text(FILE *f,const char *p){emit(f,p,strlen(p));}
static char *required(char *source,const char *needle){char *at=strstr(source,needle);if(!at||strstr(at+strlen(needle),needle))fail("original source anchor differs or repeats");return at;}
int main(int argc,char **argv)
{
    char *source,*publisher=NULL,*at,*closed,*end;size_t n,m;FILE *out;
    if(argc!=5)fail("usage: helper MODE ORIGINAL PUBLISHER OUTPUT");
    source=load(argv[2],&n);
    if(strstr(source,"mantle_native_generated_created"))fail("source is already adapted");
    out=fopen(argv[4],"wbx");if(!out)fail("creating source");
    if(!strcmp(argv[1],"writer")){
        const char *declaration="static int tcc_write_elf_file(TCCState *s1, const char *filename, int phnum,";
        const char *close="    fclose(f);\n\n    return 0;\n}";
        publisher=load(argv[3],&m);
        at=required(source,declaration);end=strstr(at,"\n}\n");closed=strstr(at,close);
        if(!end||!closed||closed>end)fail("original ELF writer close differs");
        emit(out,source,(size_t)(at-source));text(out,"extern void mantle_native_generated_created(const char *filename);\n");
        emit(out,at,(size_t)(closed-at));
        text(out,"    if (fclose(f) == 0 && file_type == TCC_OUTPUT_EXE && s1->output_format == TCC_OUTPUT_FORMAT_ELF)\n        mantle_native_generated_created(filename);\n\n    return 0;\n}");
        closed+=strlen(close);emit(out,closed,n-(size_t)(closed-source));
        text(out,"\nextern char *realpath(const char *, char *);\nextern size_t strnlen(const char *, size_t);\n");emit(out,publisher,m);text(out,"\n");
    }else fail("unknown source mode");
    if(fclose(out))fail("closing source");free(source);free(publisher);return 0;
}
