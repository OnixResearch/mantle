/* Child-free source transformations for the original host recipes.
 * No command interpreter, compiler, or build-system process is launched here.
 * Header substitutions are read from the original headers_install.sh itself.
 */
#define _GNU_SOURCE 1
#include <ctype.h>
#include <dirent.h>
#include <errno.h>
#include <limits.h>
#include <regex.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <stdint.h>
#include <unistd.h>

static void die(const char *s) { fprintf(stderr, "native host source: %s: %s\n", s, strerror(errno)); exit(1); }
static void require(int ok, const char *s) { if (!ok) { errno = EINVAL; die(s); } }
static char *load(const char *path) {
    FILE *f = fopen(path, "rb"); if (!f) die(path);
    require(!fseek(f, 0, SEEK_END), "seek"); long n = ftell(f);
    require(n >= 0 && n <= 64 * 1024 * 1024, "source size"); rewind(f);
    char *s = malloc((size_t)n + 1); if (!s) die("allocate");
    require(fread(s, 1, (size_t)n, f) == (size_t)n && !ferror(f), "read");
    s[n] = 0; require(strlen(s) == (size_t)n, "source NUL"); fclose(f); return s;
}
static void save(const char *path, const char *s) {
    FILE *f = fopen(path, "wb"); if (!f) die(path);
    require(fwrite(s, 1, strlen(s), f) == strlen(s) && !fclose(f), "write");
}
static char *joined(const char *a, const char *b) {
    char *p; if (asprintf(&p, "%s/%s", a, b) < 0) die("path"); return p;
}
static void copy_tree(const char *from, const char *to) {
    struct stat st; if (lstat(from, &st)) die(from);
    if (S_ISDIR(st.st_mode)) {
        if (mkdir(to, 0755) && errno != EEXIST) die(to);
        DIR *d = opendir(from); if (!d) die(from); struct dirent *e;
        while ((e = readdir(d))) {
            if (!strcmp(e->d_name, ".") || !strcmp(e->d_name, "..")) continue;
            char *a = joined(from, e->d_name), *b = joined(to, e->d_name);
            copy_tree(a, b); free(a); free(b);
        }
        closedir(d);
    } else if (S_ISLNK(st.st_mode)) {
        char target[PATH_MAX + 1]; ssize_t n = readlink(from, target, PATH_MAX);
        require(n > 0 && n < PATH_MAX, "source symlink"); target[n] = 0;
        if (symlink(target, to)) die(to);
    } else {
        require(S_ISREG(st.st_mode), "non-file source"); FILE *in = fopen(from, "rb"), *out = fopen(to, "wb");
        if (!in || !out) die(from); char bytes[32768]; size_t n;
        while ((n = fread(bytes, 1, sizeof(bytes), in))) require(fwrite(bytes, 1, n, out) == n, "copy");
        require(!ferror(in) && !fclose(out), "copy close"); fclose(in);
        if (chmod(to, (st.st_mode & 0111) ? 0755 : 0644)) die(to);
    }
}
static char *block(const char *s, const char *start, const char *end) {
    const char *a = strstr(s, start); require(a && !strstr(a + strlen(start), start), "unique source block");
    a += strlen(start); const char *b = strstr(a, end); require(b != NULL, "source block terminator");
    char *r = strndup(a, (size_t)(b-a)); if (!r) die("source block"); return r;
}
static char *replace(char *s, const char *find, const char *with) {
    size_t count = 0, a = strlen(find), b = strlen(with); require(a > 0, "empty replacement");
    for (const char *p = s; (p = strstr(p, find)); p += a) ++count;
    size_t n = strlen(s); require(b <= a || count <= (SIZE_MAX - n - 1) / (b-a), "replacement size");
    char *r = malloc(n - count*a + count*b + 1); if (!r) die("replacement"); char *o = r, *p = s, *q;
    while ((q = strstr(p, find))) { memcpy(o,p,(size_t)(q-p)); o += q-p; memcpy(o,with,b); o += b; p=q+a; }
    strcpy(o,p); free(s); return r;
}
static void patch_source(const char *root,const char *member,const char *find,const char *with) {
    char *path=joined(root,member),*text=load(path),*at=strstr(text,find);
    require(at&&!strstr(at+strlen(find),find),"unique original Make continuation seam");
    text=replace(text,find,with);save(path,text);free(text);free(path);
}
static void make_source(const char *source, const char *recipe, const char *control, const char *unit, const char *out) {
    copy_tree(source,out); char *r = load(recipe);
    char *config = block(r,"<<'CFGEOF'\n","\nCFGEOF");
    char *config_nl; require(asprintf(&config_nl,"%s\n",config) >= 0,"config newline");
    char *path = joined(out,"src/config.h"); save(path,config_nl); free(path); free(config_nl); free(config);
    path = joined(out,"src/dir.c"); char *dir = load(path);
    const char *changes[] = {"gl->gl_opendir = open_dirstream;", "gl->gl_readdir = read_dirstream;", "gl->gl_closedir = free;", "gl->gl_lstat = local_lstat;", "gl->gl_stat = local_stat;"};
    for (size_t i=0;i<sizeof(changes)/sizeof(changes[0]);++i) { require(strstr(r,changes[i]) != NULL,"original make substitution"); dir = replace(dir,changes[i],""); }
    save(path,dir); free(path); free(dir); free(r);
    path=joined(out,"src/mantle_make_control.h");copy_tree(control,path);free(path);
    path=joined(out,"src/mantle_make_control.c");copy_tree(unit,path);free(path);
    path=joined(out,"src/makeint.h");char *makeint=load(path),*with_control;
    require(asprintf(&with_control,"%s\n#include \"mantle_make_control.h\"\n",makeint)>=0,"Make evaluator header");
    save(path,with_control);free(path);free(makeint);free(with_control);
    patch_source(out,"src/main.c","#include \"makeint.h\"","#include \"makeint.h\"\n#include \"mantle_make_control.h\"\n#define main mantle_original_make_main");
    patch_source(out,"src/job.c","#include \"makeint.h\"","#include \"makeint.h\"\n#include \"mantle_make_control.h\"");
    patch_source(out,"src/function.c","#include \"makeint.h\"","#include \"makeint.h\"\n#include \"mantle_make_control.h\"");
    patch_source(out,"src/commands.c","#include \"makeint.h\"","#include \"makeint.h\"\n#include \"mantle_make_control.h\"");
    patch_source(out,"src/main.c","  exit (status);","  mantle_make_exit (status);");
    patch_source(out,"src/job.c","pid = WAIT_NOHANG (&status);","pid = mantle_make_wait (&status, 1);");
    patch_source(out,"src/job.c","EINTRLOOP (pid, wait (&status));","EINTRLOOP (pid, mantle_make_wait (&status, 0));");
    patch_source(out,"src/job.c","#if !defined(USE_POSIX_SPAWN)","if (mantle_make_active ())\n    return mantle_make_child (argv, child->environment, fdin, fdout, fderr);\n\n#if !defined(USE_POSIX_SPAWN)");
    patch_source(out,"src/job.c","exec_command (char **argv, char **envp)\n{","exec_command (char **argv, char **envp)\n{\n  if (mantle_make_active ()) mantle_make_replace (argv, envp);");
    patch_source(out,"src/function.c","if (pipe (pipedes) < 0)","if (mantle_make_pipe (pipedes) < 0)");
    patch_source(out,"src/commands.c","kill (c->pid, SIGTERM)","mantle_make_cancel (c->pid, SIGTERM)");
}
static void emit_block(const char *recipe, const char *start, const char *end, const char *out) {
    char *r=load(recipe), *s=block(r,start,end), *nl; require(asprintf(&nl,"%s\n",s)>=0,"block newline");
    save(out,nl); free(nl); free(s); free(r);
}
/* The seven extended-regexp substitutions are not re-authored here. This
 * small substitution evaluator reads their patterns/replacements from the
 * authenticated original shell producer and applies its per-line semantics. */
static char *substitute(char *line, const char *pattern, const char *replacement, int global) {
    regex_t re; require(regcomp(&re,pattern,REG_EXTENDED)==0,"original sed regexp");
    size_t cap=strlen(line)+1024, used=0; char *result=malloc(cap); if(!result)die("sed buffer");
    const char *p=line; regmatch_t m[10]; int flags=0;
    while(regexec(&re,p,10,m,flags)==0) {
        size_t need=(size_t)m[0].rm_so+strlen(replacement)+strlen(p)+1024;
        if(cap-used<need){cap=used+need;result=realloc(result,cap);if(!result)die("sed growth");}
        memcpy(result+used,p,(size_t)m[0].rm_so);used+=(size_t)m[0].rm_so;
        for(const char *r=replacement;*r;++r){
            int group=-1; if(*r=='&')group=0;else if(*r=='\\'&&isdigit((unsigned char)r[1]))group=*++r-'0';
            if(group>=0){if(m[group].rm_so>=0){size_t n=(size_t)(m[group].rm_eo-m[group].rm_so);memcpy(result+used,p+m[group].rm_so,n);used+=n;}}
            else {if(*r=='\\'&&r[1])++r;result[used++]=*r;}
        }
        require(m[0].rm_eo>0,"zero-width source substitution");p+=m[0].rm_eo;flags=REG_NOTBOL;
        if(!global)break;
    }
    strcpy(result+used,p); regfree(&re);free(line);return result;
}
static void header_prepare(const char *source,const char *script,const char *out) {
    char *s=load(script), *program=block(s,"sed -E -e '\n","\n' $INFILE");
    char *raw=load(source),*copy=strdup(raw);if(!copy)die("header copy");
    char *line_save=NULL,*line=strtok_r(copy,"\n",&line_save);
    while(line){if(strstr(line,"SPDX-License-Identifier:")&&strstr(line,"GPL-")&&!strstr(line,"WITH Linux-syscall-note"))require(0,"missing Linux syscall license note");line=strtok_r(NULL,"\n",&line_save);}
    free(copy); FILE *f=fopen(out,"wb");if(!f)die(out);char *p=raw;
    while(*p){char *end=strchr(p,'\n');size_t n=end?(size_t)(end-p):strlen(p);char *value=strndup(p,n);if(!value)die("header line");
        char *rules=strdup(program),*rule_save=NULL,*rule=strtok_r(rules,"\n",&rule_save);if(!rules)die("sed rules");
        while(rule){while(isspace((unsigned char)*rule))++rule;require(rule[0]=='s'&&rule[1],"original header substitution command");char delim=rule[1];char *a=rule+2,*b=strchr(a,delim);require(b!=NULL,"sed pattern delimiter");*b++=0;char *c=strchr(b,delim);require(c!=NULL,"sed replacement delimiter");*c++=0;require(!*c||!strcmp(c,"g"),"sed substitution flag");value=substitute(value,a,b,*c=='g');rule=strtok_r(NULL,"\n",&rule_save);}
        fputs(value,f);if(end)fputc('\n',f);free(value);free(rules);if(!end)break;p=end+1;
    }
    require(!ferror(f)&&!fclose(f),"processed header write");free(program);free(raw);free(s);
}
static void header_check(const char *source,const char *script,const char *original,const char *out) {
    char *s=load(source),*r=load(script),*ignores=block(r,"config_leak_ignores=\"\n","\n\"");int comment=0;
    for(char *p=s;*p;){if(!comment&&p[0]=='/'&&p[1]=='*'){comment=1;p+=2;continue;}if(comment&&p[0]=='*'&&p[1]=='/'){comment=0;p+=2;continue;}if(comment){++p;continue;}
        if((p==s||!(isalnum((unsigned char)p[-1])||p[-1]=='_'))&&!strncmp(p,"CONFIG_",7)){char *e=p+7;while(isalnum((unsigned char)*e)||*e=='_')++e;char *key;require(asprintf(&key,"%s:%.*s",original,(int)(e-p),p)>=0,"CONFIG leak key");int found=0;const char *a=ignores;while(*a){const char *b=strchr(a,'\n');size_t n=b?(size_t)(b-a):strlen(a);if(strlen(key)==n&&!memcmp(a,key,n))found=1;if(!b)break;a=b+1;}free(key);require(found,"CONFIG leaked to user space");p=e;}else ++p;
    }
    save(out,s);free(ignores);free(r);free(s);
}
static int abi_selected(const char *list,const char *abi){const char *p=list;while(*p){const char *e=strchr(p,',');size_t n=e?(size_t)(e-p):strlen(p);if(strlen(abi)==n&&!memcmp(p,abi,n))return 1;if(!e)break;p=e+1;}return 0;}
static void syscall_header(const char *table,const char *script,const char *abis,const char *offset,const char *prefix,const char *name,const char *out){
    char *r=load(script);require(strstr(r,"#define __NR_$prefix$name $nr")&&strstr(r,"#define __NR_${prefix}syscalls"),"original syscall producer seam");free(r);
    char guard[256]="_UAPI_ASM_";size_t used=strlen(guard);for(const char *p=name;*p;++p){unsigned char c=(unsigned char)toupper((unsigned char)*p);if(!(isalnum(c)||c=='_'))c='_';if(c=='_'&&used&&guard[used-1]=='_')continue;require(used+1<sizeof(guard),"syscall guard");guard[used++]=(char)c;}guard[used]=0;
    FILE *in=fopen(table,"rb"),*f=fopen(out,"wb");if(!in||!f)die("syscall files");fprintf(f,"#ifndef %s\n#define %s\n\n",guard,guard);char *line=NULL;size_t cap=0;unsigned long max=0;
    while(getline(&line,&cap,in)>=0){char nr[64],abi[64],symbol[256];if(sscanf(line,"%63s %63s %255s",nr,abi,symbol)!=3||!isxdigit((unsigned char)nr[0])||!abi_selected(abis,abi))continue;char *end;max=strtoul(nr,&end,0);require(!*end,"syscall number");if(*offset)fprintf(f,"#define __NR_%s%s (%s + %s)\n",prefix,symbol,offset,nr);else fprintf(f,"#define __NR_%s%s %s\n",prefix,symbol,nr);}
    fprintf(f,"\n#ifdef __KERNEL__\n#define __NR_%ssyscalls %lu\n#endif\n\n#endif /* %s */\n",prefix,max+1,guard);free(line);require(!ferror(in)&&!ferror(f)&&!fclose(f),"syscall output");fclose(in);
}
static unsigned long make_number(const char *source,const char *key){
    const char *p=source;size_t n=strlen(key);while(*p){if(!strncmp(p,key,n)&&p[n]==' '){const char *e=strchr(p,'=');require(e!=NULL,"Make numeric assignment");char *end;unsigned long value=strtoul(e+1,&end,10);require(end!=e+1&&(*end=='\n'||!*end),"Make numeric value");return value;}p=strchr(p,'\n');if(!p)break;++p;}require(0,"missing Make numeric source");return 0;
}
static void version_header(const char *makefile,const char *version,const char *out){
    char *r=load(makefile),*v=load(version);require(strstr(v,"GNU Make 4.4.1")!=NULL,"original Make version requirement");free(v);
    require(strstr(r,"define filechk_version.h")&&strstr(r,"LINUX_VERSION_SUBLEVEL $(SUBLEVEL)"),"original version generator");
    unsigned long a=make_number(r,"VERSION"),b=make_number(r,"PATCHLEVEL"),c=make_number(r,"SUBLEVEL");FILE *f=fopen(out,"wb");if(!f)die(out);
    fprintf(f,"#define LINUX_VERSION_CODE %lu\n#define KERNEL_VERSION(a,b,c) (((a) << 16) + ((b) << 8) + ((c) > 255 ? 255 : (c)))\n#define LINUX_VERSION_MAJOR %lu\n#define LINUX_VERSION_PATCHLEVEL %lu\n#define LINUX_VERSION_SUBLEVEL %lu\n",a*65536+b*256+(c>255?255:c),a,b,c);
    require(!ferror(f)&&!fclose(f),"version output");free(r);
}
static void wrapper_header(const char *makefile,const char *name,const char *out){
    char *r=load(makefile);require(strstr(r,"#include <asm-generic/$*.h>")!=NULL,"original asm-generic wrapper rule");require(!strchr(name,'/')&&!strstr(name,".."),"wrapper member");FILE *f=fopen(out,"wb");if(!f)die(out);fprintf(f,"#include <asm-generic/%s>\n",name);require(!ferror(f)&&!fclose(f),"wrapper output");free(r);
}
static void make_version_output(const char *root,const char *out){
    char *path=joined(root,"state"),*value=load(path);require(!strcmp(value,"done\n"),"original Make version completed");free(path);free(value);
    path=joined(root,"exit-status");value=load(path);require(!strcmp(value,"0\n"),"original Make version status");free(path);free(value);
    path=joined(root,"source-result/descriptor-1");copy_tree(path,out);free(path);
}
int main(int argc,char **argv){
    require(argc>=2,"mode");
    if(!strcmp(argv[1],"make-source")){require(argc==7,"make source arguments");make_source(argv[2],argv[3],argv[4],argv[5],argv[6]);}
    else if(!strcmp(argv[1],"block")){require(argc==6,"block arguments");emit_block(argv[2],argv[3],argv[4],argv[5]);}
    else if(!strcmp(argv[1],"make-version-output")){require(argc==4,"Make version output arguments");make_version_output(argv[2],argv[3]);}
    else if(!strcmp(argv[1],"copy")){require(argc==4,"copy arguments");copy_tree(argv[2],argv[3]);}
    else if(!strcmp(argv[1],"header-prepare")){require(argc==5,"header prepare arguments");header_prepare(argv[2],argv[3],argv[4]);}
    else if(!strcmp(argv[1],"header-check")){require(argc==6,"header check arguments");header_check(argv[2],argv[3],argv[4],argv[5]);}
    else if(!strcmp(argv[1],"syscall")){require(argc==9,"syscall arguments");syscall_header(argv[2],argv[3],argv[4],argv[5],argv[6],argv[7],argv[8]);}
    else if(!strcmp(argv[1],"version")){require(argc==5,"version arguments");version_header(argv[2],argv[3],argv[4]);}
    else if(!strcmp(argv[1],"wrapper")){require(argc==5,"wrapper arguments");wrapper_header(argv[2],argv[3],argv[4]);}
    else require(0,"unknown mode");return 0;
}
