/* Child-free minicargo source utility. The authenticated Rust/mrustc sources
 * select the graph; this utility only copies source, frames actual script
 * directives, and inserts an in-process script environment entry boundary. */
#define _DEFAULT_SOURCE
#define _POSIX_C_SOURCE 200809L
#include <sys/stat.h>
#include <sys/types.h>
#include <dirent.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <errno.h>
static void die(const char *s){fprintf(stderr,"minicargo phase: %s (%s)\n",s,strerror(errno));exit(1);}
static void *alloc(size_t n){void*p=malloc(n?n:1);if(!p)die("allocation");return p;}
static char *join(const char*a,const char*b){size_t x=strlen(a),y=strlen(b);char*p=alloc(x+y+2);memcpy(p,a,x);p[x]='/';memcpy(p+x+1,b,y+1);return p;}
static unsigned entries;
static void copy(const char*s,const char*d){struct stat st;if(++entries>1000000||lstat(s,&st))die("source metadata");
 if(S_ISDIR(st.st_mode)){if(mkdir(d,0755)&&errno!=EEXIST)die("source directory");DIR*r=opendir(s);if(!r)die("open source directory");struct dirent*e;while((e=readdir(r))){if(!strcmp(e->d_name,".")||!strcmp(e->d_name,".."))continue;char*a=join(s,e->d_name),*b=join(d,e->d_name);copy(a,b);free(a);free(b);}closedir(r);}
 else if(S_ISREG(st.st_mode)){FILE*i=fopen(s,"rb"),*o=fopen(d,"wb");if(!i||!o)die("copy source");char b[65536];size_t n;while((n=fread(b,1,sizeof b,i)))if(fwrite(b,1,n,o)!=n)die("write source");if(ferror(i)||fclose(i)||fclose(o)||chmod(d,st.st_mode&0777))die("close source");}
 else if(S_ISLNK(st.st_mode)){char b[4096];ssize_t n=readlink(s,b,sizeof b-1);if(n<0||n==(ssize_t)sizeof b-1||b[0]=='/')die("source symlink");b[n]=0;if(symlink(b,d))die("copy source symlink");}else die("non-source member");}
static char *load(const char*p){FILE*f=fopen(p,"rb");if(!f)die("open text");if(fseek(f,0,SEEK_END))die("seek text");long n=ftell(f);if(n<0||n>64*1024*1024||fseek(f,0,SEEK_SET))die("text bound");char*s=alloc(n+1);if(fread(s,1,n,f)!=(size_t)n||fclose(f))die("read text");s[n]=0;return s;}
static void save(const char*p,const char*s){FILE*f=fopen(p,"wb");if(!f||fwrite(s,1,strlen(s),f)!=strlen(s)||fclose(f))die("write text");}
static void u32(FILE*f,size_t n){if(n>65536)die("frame bound");unsigned char b[4]={(unsigned char)n,(unsigned char)(n>>8),(unsigned char)(n>>16),(unsigned char)(n>>24)};if(fwrite(b,1,4,f)!=4)die("write frame");}
static void arg(FILE*f,const char*s){u32(f,strlen(s));if(fwrite(s,1,strlen(s),f)!=strlen(s))die("argument frame");}
static void env(FILE*f,const char*k,const char*v){if(!*k||strchr(k,'=')||strchr(k,'\n'))die("environment key");u32(f,strlen(k));u32(f,strlen(v));if(fwrite(k,1,strlen(k),f)!=strlen(k)||fwrite(v,1,strlen(v),f)!=strlen(v))die("environment frame");}
static void cfg_environment(FILE*out,const char*path){if(!strcmp(path,"-"))return;char*s=load(path),*cursor=s,*line;while((line=strsep(&cursor,"\n"))){if(*line!='>')continue;line++;char*value=strchr(line,'=');if(value)*value++=0;else value="1";size_t n=strlen(line);char*key=alloc(n+11);strcpy(key,"CARGO_CFG_");for(size_t i=0;i<n;i++){char c=line[i];key[10+i]=c>='a'&&c<='z'?c-32:c=='-'?'_':c;}key[10+n]=0;env(out,key,value);free(key);}free(s);}
static void metadata(int argc,char**argv){if(argc<5||(argc-5)%3)die("metadata operands");FILE*a=fopen(argv[2],"wb"),*e=fopen(argv[3],"wb");if(!a||!e)die("metadata outputs");cfg_environment(e,argv[4]);for(int i=5;i<argc;i+=3){const char*links=argv[i],*stdout_path=argv[i+1],*prefix=argv[i+2];char*s=load(stdout_path),*cursor=s,*line;while((line=strsep(&cursor,"\n"))){size_t n=strlen(line);if(n&&line[n-1]=='\r')line[n-1]=0;if(strncmp(line,"cargo:",6))continue;line+=6;if(*line==':')line++;char*equals=strchr(line,'=');if(!equals)continue;*equals++=0;char*v=equals;
 if(!strcmp(links,"-")){if(!strcmp(line,"rustc-cfg")){arg(a,"--cfg");arg(a,v);}else if(!strcmp(line,"rustc-link-search")){arg(a,"-L");char*kind=strchr(v,'=');arg(a,kind?kind+1:v);}else if(!strcmp(line,"rustc-link-lib")){arg(a,"-l");char*kind=strchr(v,'=');arg(a,kind?kind+1:v);}else if(!strcmp(line,"rustc-env")){char*q=strchr(v,'=');if(!q)die("rustc-env directive");*q++=0;env(e,v,q);}else if(!strcmp(line,"rustc-flags")){char*save=0;for(char*t=strtok_r(v," \t",&save);t;t=strtok_r(0," \t",&save)){if(strcmp(t,"-L")&&strcmp(t,"-l"))die("unsupported original rustc-flags");arg(a,t);t=strtok_r(0," \t",&save);if(!t)die("unpaired rustc-flags");arg(a,t);}}}
 else if(strncmp(line,"rustc-",6)&&strncmp(line,"rerun-",6)&&strcmp(line,"warning")&&strcmp(line,"error")){const char*k=line;char*value=v;if(!strcmp(line,"metadata")){char*q=strchr(v,'=');if(!q)die("links metadata");*q++=0;k=v;value=q;}size_t z=strlen(prefix)+strlen(k)+2;char*key=alloc(z);snprintf(key,z,"%s_%s",prefix,k);for(char*p=key;*p;p++){if(*p=='-')*p='_';else if(*p>='a'&&*p<='z')*p-=32;}env(e,key,value);free(key);}}free(s);}if(fclose(a)||fclose(e))die("metadata close");}
static const char script_entry[]="\nfn __mantle_minicargo_env() {\n fn n(b: &[u8], p: &mut usize) -> usize { assert!(*p+4<=b.len()); let v=(b[*p] as usize)|((b[*p+1] as usize)<<8)|((b[*p+2] as usize)<<16)|((b[*p+3] as usize)<<24); *p+=4; v }\n if let Ok(path)=std::env::var(\"MRUSTC_CAIRN_SCRIPT_ENV\") { let b=std::fs::read(path).unwrap(); let mut p=0; while p<b.len(){let k=n(&b,&mut p);let v=n(&b,&mut p);assert!(k<=65536&&v<=65536&&p+k+v<=b.len());let key=std::str::from_utf8(&b[p..p+k]).unwrap();p+=k;let val=std::str::from_utf8(&b[p..p+v]).unwrap();p+=v;unsafe {std::env::set_var(key,val);} } }\n if let Some(manifest)=std::env::var_os(\"MRUSTC_CAIRN_MANIFEST_DIR\"){unsafe {std::env::set_var(\"CARGO_MANIFEST_DIR\",manifest);}}\n if let Ok(out)=std::env::var(\"OUT_DIR\"){std::fs::create_dir_all(out).unwrap();}\n std::env::set_current_dir(std::env::var_os(\"CARGO_MANIFEST_DIR\").unwrap()).unwrap();\n}\n";
static void script(const char*source,const char*dest){char*s=load(source),*p=strstr(s,"fn main(");if(!p)die("original script main entry");p=strchr(p,'{');if(!p)die("original script main body");p++;const char*call=" __mantle_minicargo_env(); ";size_t before=p-s,n=strlen(s)+strlen(call)+strlen(script_entry)+1;char*r=alloc(n);memcpy(r,s,before);strcpy(r+before,call);strcat(r,p);strcat(r,script_entry);save(dest,r);free(s);free(r);}
static const char *field_names[]={"authors","description","homepage","license","license-file","repository","readme","rust-version","version"};
static const char *space(const char *p){for(;;){while(*p==' '||*p=='\t'||*p=='\r'||*p=='\n')p++;if(*p!='#')return p;while(*p&&*p!='\n')p++;}}
static void utf8(char **out,unsigned v){if(!v||(v>=0xd800&&v<=0xdfff))die("invalid TOML Unicode scalar");char*p=*out;if(v<=0x7f)*p++=(char)v;else if(v<=0x7ff){*p++=(char)(0xc0|(v>>6));*p++=(char)(0x80|(v&63));}else if(v<=0xffff){*p++=(char)(0xe0|(v>>12));*p++=(char)(0x80|((v>>6)&63));*p++=(char)(0x80|(v&63));}else if(v<=0x10ffff){*p++=(char)(0xf0|(v>>18));*p++=(char)(0x80|((v>>12)&63));*p++=(char)(0x80|((v>>6)&63));*p++=(char)(0x80|(v&63));}else die("invalid TOML Unicode scalar");*out=p;}
static char *toml_string(const char **cursor){const char*p=*cursor;char quote=*p++;if(quote!='\''&&quote!='"')die("expected TOML string");int multi=p[0]==quote&&p[1]==quote;if(multi){p+=2;if(*p=='\r'&&p[1]=='\n')p+=2;else if(*p=='\n')p++;}char*result=alloc(strlen(p)+1),*o=result;
 while(*p){if(*p==quote&&(!multi||(p[1]==quote&&p[2]==quote))){p+=multi?3:1;*o=0;*cursor=p;return result;}if(!multi&&(*p=='\n'||*p=='\r'))die("newline in TOML string");if(quote=='"'&&*p=='\\'){p++;char c=*p++;if(multi&&(c=='\n'||c=='\r'||c==' '||c=='\t')){while(*p==' '||*p=='\t'||*p=='\r'||*p=='\n')p++;continue;}switch(c){case 'b':*o++='\b';break;case 't':*o++='\t';break;case 'n':*o++='\n';break;case 'f':*o++='\f';break;case 'r':*o++='\r';break;case '"':case '\\':*o++=c;break;case 'u':case 'U':{unsigned v=0;int count=c=='u'?4:8;for(int i=0;i<count;i++){unsigned x=(unsigned char)*p++;if(x>='0'&&x<='9')x-='0';else if(x>='a'&&x<='f')x=x-'a'+10;else if(x>='A'&&x<='F')x=x-'A'+10;else die("invalid TOML Unicode escape");v=(v<<4)|x;}utf8(&o,v);break;}default:die("invalid TOML string escape");}}else{*o++=*p++;}}
 die("unterminated TOML string");return 0;
}
static const char *value_end(const char*p){unsigned arrays=0,tables=0;for(;*p;p++){if(*p=='\''||*p=='"'){char*ignored=toml_string(&p);free(ignored);p--;continue;}if(*p=='[')arrays++;else if(*p==']'){if(!arrays)die("unbalanced TOML array");arrays--;}else if(*p=='{')tables++;else if(*p=='}'){if(!tables)die("unbalanced TOML inline table");tables--;}else if(*p=='#'){while(*p&&*p!='\n')p++;if(!*p)return p;}if(*p=='\n'&&!arrays&&!tables)return p+1;}return p;}
static char *field_value(const char *p,int *inherit){p=space(p);*inherit=0;if(*p=='\''||*p=='"')return toml_string(&p);if(!strncmp(p,"false",5)){char*empty=alloc(1);*empty=0;return empty;}if(*p=='['){p++;char*result=alloc(1);result[0]=0;size_t len=0,count=0;for(;;){p=space(p);if(*p==']')return result;char*part=toml_string(&p);size_t n=strlen(part);char*next=alloc(len+n+2);memcpy(next,result,len);if(count++)next[len++]=':';memcpy(next+len,part,n+1);len+=n;free(part);free(result);result=next;p=space(p);if(*p==',')p++;else if(*p!=']')die("invalid original authors array");}}
 if(*p=='{'){p=space(p+1);if(strncmp(p,"workspace",9))die("unsupported original inherited metadata");p=space(p+9);if(*p++!='=')die("inherited metadata key");p=space(p);if(strncmp(p,"true",4))die("inherited metadata flag");p=space(p+4);if(*p!='}')die("inherited metadata table");*inherit=1;return 0;}die("non-string original package metadata");return 0;}
static char *find_field(const char*text,const char*section,const char*field,int*inherit){const char*p=text;int selected=0;*inherit=0;while(*(p=space(p))){if(*p=='['){const char*start=++p;while(*p&&*p!=']'&&*p!='\n')p++;if(*p!=']')die("original TOML section");size_t n=(size_t)(p-start);selected=n==strlen(section)&&!memcmp(start,section,n);while(*p&&*p!='\n')p++;continue;}char*key=0;if(*p=='\''||*p=='"'){key=toml_string(&p);p=space(p);}else{const char*start=p;while(*p&&*p!='='&&*p!=' '&&*p!='\t'&&*p!='\n')p++;size_t n=(size_t)(p-start);key=alloc(n+1);memcpy(key,start,n);key[n]=0;while(*p==' '||*p=='\t')p++;}if(*p++!='='){free(key);die("original TOML assignment");}if(selected&&!strcmp(key,field)){free(key);return field_value(p,inherit);}size_t n=strlen(field);if(selected&&!strncmp(key,field,n)&&!strcmp(key+n,".workspace")){free(key);p=space(p);if(strncmp(p,"true",4))die("original dotted workspace inheritance");*inherit=1;return 0;}free(key);p=value_end(p);}return 0;}
static void package_fields(int argc,char**argv){if(argc!=13)die("package fields operands");char*manifest=load(argv[2]),*workspace=strcmp(argv[3],"-")?load(argv[3]):0;for(size_t i=0;i<sizeof field_names/sizeof*field_names;i++){int inherit=0;char*value=find_field(manifest,"package",field_names[i],&inherit);if(inherit){if(!workspace)die("missing original metadata workspace");value=find_field(workspace,"workspace.package",field_names[i],&inherit);if(!value||inherit)die("missing inherited original metadata field");}save(argv[4+i],value?value:"");free(value);}free(manifest);free(workspace);}
static char *patch_field(char **cursor){char*p=*cursor,*start=p;size_t n=0;if(*p<'0'||*p>'9')die("source patch netstring length");while(*p>='0'&&*p<='9'){if(n>(64u*1024u*1024u-(unsigned)(*p-'0'))/10u)die("source patch field bound");n=n*10u+(unsigned)(*p++-'0');}if((p-start>1&&*start=='0')||*p++!=':'||n>strlen(p)||p[n]!=',')die("source patch netstring frame");char*field=p;p[n]=0;*cursor=p+n+1;return field;}
static void patch_relative(const char*p){if(!*p||*p=='/')die("source patch relative path");for(;;){const char*slash=strchr(p,'/');size_t n=slash?(size_t)(slash-p):strlen(p);if(!n||(n==1&&p[0]=='.')||(n==2&&p[0]=='.'&&p[1]=='.')||memchr(p,'\\',n))die("source patch path component");if(!slash)break;p=slash+1;}}
static char *patch_replace(char*source,const char*before,const char*after){size_t a=strlen(before),b=strlen(after),n=strlen(source);char*found;if(!a||!(found=strstr(source,before))||strstr(found+a,before))die("authenticated std process patch anchor mismatch");if(n-a>64u*1024u*1024u-b)die("patched source text bound");size_t head=(size_t)(found-source);char*out=alloc(n-a+b+1);memcpy(out,source,head);memcpy(out+head,after,b);memcpy(out+head+b,found+a,n-head-a+1);free(source);return out;}
static char *rust_top(const char*root){DIR*d=opendir(root);if(!d)die("open original Rust source root");struct dirent*e;char*selected=0;while((e=readdir(d))){size_t n=strlen(e->d_name);if(n<10||strncmp(e->d_name,"rustc-",6)||strcmp(e->d_name+n-4,"-src"))continue;char*path=join(root,e->d_name);struct stat st;if(lstat(path,&st)||!S_ISDIR(st.st_mode))die("original Rust archive top directory");free(path);if(selected)die("ambiguous original Rust source archive top");selected=alloc(n+1);memcpy(selected,e->d_name,n+1);}if(closedir(d)||!selected)die("missing original Rust source archive top");return selected;}
static void std_process_overlay(const char *source, const char *dest,
                                const char *descriptor, const char *module_source)
{
    char *bytes = load(descriptor), *cursor = bytes;
    if (strcmp(patch_field(&cursor), "MANTLE-STD-PROCESS-PATCH-1"))
        die("standard process patch descriptor version");
    char *target = patch_field(&cursor), *module_path = patch_field(&cursor);
    patch_relative(target);
    patch_relative(module_path);
    if (!strcmp(target, module_path)) die("standard process module shadows original process source");
    char *count_text = patch_field(&cursor), *end;
    errno = 0;
    unsigned long count = strtoul(count_text, &end, 10);
    if (errno || !*count_text || *end || count > 32 || (count_text[0] == '0' && count_text[1]))
        die("standard process patch count");
    char *top = rust_top(source);
    copy(source, dest);
    char *root = join(dest, top), *process_path = join(root, target);
    char *module_dest = join(root, module_path);
    struct stat st;
    if (lstat(process_path, &st) || !S_ISREG(st.st_mode)) die("original standard process source kind");
    char *text = load(process_path);
    for (unsigned long i = 0; i < count; i++) {
        char *before = patch_field(&cursor), *after = patch_field(&cursor);
        text = patch_replace(text, before, after);
    }
    char *append = patch_field(&cursor);
    if (*cursor) die("trailing standard process patch descriptor bytes");
    size_t n = strlen(text), a = strlen(append);
    if (n > 64u * 1024u * 1024u - a) die("standard process appended source bound");
    char *combined = alloc(n + a + 1);
    memcpy(combined, text, n);
    memcpy(combined + n, append, a + 1);
    if (chmod(process_path, (st.st_mode & 0777) | 0200)) die("writable copied standard process source");
    save(process_path, combined);
    if (chmod(process_path, st.st_mode & 0777)) die("restore original standard process source mode");
    if (!lstat(module_dest, &st) || errno != ENOENT) die("standard process module already exists");
    char *module_text = load(module_source);
    save(module_dest, module_text);
    free(module_text);
    free(combined);
    free(text);
    free(module_dest);
    free(process_path);
    free(root);
    free(top);
    free(bytes);
}
/* Exact native_helper::quote_response translation, usable before Rust std. */
static void native_sdk(int argc,char**argv){if(argc<3)die("native SDK operands");FILE*f=fopen(argv[2],"wb");if(!f)die("native SDK response");for(int i=3;i<argc;i++){fputc('"',f);for(const unsigned char*p=(const unsigned char*)argv[i];*p;p++){if(*p=='"'||*p=='\\')fputc('\\',f);fputc(*p,f);}fputs("\"\n",f);}if(ferror(f)||fclose(f))die("native SDK response write");}
int main(int argc,char**argv){if(argc<2)die("mode");if(!strcmp(argv[1],"--native-sdk")){native_sdk(argc,argv);return 0;}if(!strcmp(argv[1],"std-process-overlay")&&argc==6){std_process_overlay(argv[2],argv[3],argv[4],argv[5]);return 0;}if(!strcmp(argv[1],"select")&&argc==4){copy(argv[2],argv[3]);return 0;}if(!strcmp(argv[1],"merge")&&argc>=3){if(mkdir(argv[2],0755)&&errno!=EEXIST)die("merge directory");for(int i=3;i<argc;i++)copy(argv[i],argv[2]);return 0;}if(!strcmp(argv[1],"metadata")){metadata(argc,argv);return 0;}if(!strcmp(argv[1],"package-fields")){package_fields(argc,argv);return 0;}if(!strcmp(argv[1],"scripts")&&argc>=4){copy(argv[2],argv[3]);size_t n=strlen(argv[2]);for(int i=4;i<argc;i++){if(strncmp(argv[i],argv[2],n)||argv[i][n]!='/')die("script outside original tree");char*p=join(argv[3],argv[i]+n+1);script(p,p);free(p);}return 0;}die("unknown mode");return 1;
