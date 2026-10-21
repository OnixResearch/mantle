//! Source-owned, childless evaluation of the original ld script generators.
//!
//! The phase program interprets the authenticated shell sources; it does not
//! contain precomputed linker scripts or emulation C.  Pipelines use temporary
//! streams, and every utility used by the selected generators runs in-process.
#![allow(dead_code)] // Pending wiring in the source-only preparation path (Cairn prove-source-built-mantle-fixed-point I3).
use std::collections::BTreeMap;

use crate::errors::RunError;
use crate::source_built_recipe::ArtifactKind;
use crate::source_built_recipe::ArtifactRef;
use crate::source_built_recipe::RecipeExitPolicy;
use crate::source_built_recipe::RecipeOperation;
use crate::source_built_recipe::RecipeOutput;
use crate::source_built_recipe::RecipeStep;
use crate::source_built_stagex_programs::TransitionBuilder;
use crate::source_built_stagex_programs::error;
use crate::source_built_stagex_programs::input;
use crate::source_built_stagex_programs::literal;
use crate::source_built_stagex_programs::output;
use crate::stagex_transition::BINUTILS_GENERATOR_BUILD_STAGE_ID as TOOL;

const SOURCE: &str = "binutils-2.30-source";
const EMULATIONS: &[&str] = &[
    "elf_x86_64",
    "elf32_x86_64",
    "elf_i386",
    "elf_iamcu",
    "i386linux",
    "elf_l1om",
    "elf_k1om",
];

fn spec(kind: ArtifactKind) -> RecipeOutput {
    RecipeOutput {
        kind,
        bytes_max: if kind == ArtifactKind::Tree {
            256 * 1024 * 1024
        } else {
            64 * 1024 * 1024
        },
    }
}

pub(super) fn generate(b: &mut TransitionBuilder) -> Result<ArtifactRef, RunError> {
    let helper = b.authored("helpers/binutils-ldscripts.c", ENGINE.into())?;
    let cc = b.artifact("tcc-musl-v2")?;
    let object = b.add(TOOL, RecipeStep {
        id: "binutils-ldscripts-engine-compile".into(),
        inputs: BTreeMap::from([
            ("executable".into(), cc.clone()),
            ("source".into(), helper),
            ("headers".into(), b.artifact("musl-native-headers")?),
        ]),
        outputs: BTreeMap::from([
            ("object".into(), spec(ArtifactKind::Object)),
            ("stdout".into(), spec(ArtifactKind::Text)),
            ("stderr".into(), spec(ArtifactKind::Text)),
            ("exit-status".into(), RecipeOutput {
                kind: ArtifactKind::Text,
                bytes_max: 4,
            }),
        ]),
        operation: RecipeOperation::Exec {
            invocation_input: None,
            command_state: None,
            merge_stderr: false,
            executable_input: "executable".into(),
            arguments: vec![
                literal("-c"),
                literal("-I"),
                input("headers", ""),
                input("source", ""),
                literal("-o"),
                output("object", ""),
            ],
            environment: BTreeMap::new(),
            runtime_mounts: BTreeMap::new(),
            output_seeds: BTreeMap::new(),
            guard: None,
            cwd_input: None,
            cwd_output: None,
            stdin_input: None,
            exit_policy: RecipeExitPolicy::Exact { code: 0 },
        },
    })?["object"]
        .clone();
    let executable = b.add(TOOL, RecipeStep {
        id: "binutils-ldscripts-engine-link".into(),
        inputs: BTreeMap::from([
            ("executable".into(), cc),
            ("object".into(), object),
            ("crt1".into(), b.artifact("musl-native-crt1")?),
            ("libc".into(), b.artifact("musl-native-libc")?),
            ("runtime".into(), b.artifact("binutils-runtime-object")?),
            ("assembly".into(), b.artifact("binutils-runtime-assembly-object")?),
        ]),
        outputs: BTreeMap::from([
            ("helper".into(), spec(ArtifactKind::Executable)),
            ("stdout".into(), spec(ArtifactKind::Text)),
            ("stderr".into(), spec(ArtifactKind::Text)),
            ("exit-status".into(), RecipeOutput {
                kind: ArtifactKind::Text,
                bytes_max: 4,
            }),
        ]),
        operation: RecipeOperation::Exec {
            invocation_input: None,
            command_state: None,
            merge_stderr: false,
            executable_input: "executable".into(),
            arguments: vec![
                literal("-nostdlib"),
                literal("-static"),
                input("crt1", ""),
                input("object", ""),
                input("runtime", ""),
                input("assembly", ""),
                input("libc", ""),
                literal("-o"),
                output("helper", ""),
            ],
            environment: BTreeMap::new(),
            runtime_mounts: BTreeMap::new(),
            output_seeds: BTreeMap::new(),
            guard: None,
            cwd_input: None,
            cwd_output: None,
            stdin_input: None,
            exit_policy: RecipeExitPolicy::Exact { code: 0 },
        },
    })?["helper"]
        .clone();
    let mut originals = BTreeMap::new();
    let mut entries = BTreeMap::new();
    for path in b.members(SOURCE)? {
        if path == "ld/genscripts.sh"
            || path.starts_with("ld/emulparams/")
            || path.starts_with("ld/emultempl/")
            || path.starts_with("ld/scripttempl/")
        {
            let key = format!("original-{}", originals.len());
            originals.insert(key.clone(), b.member(SOURCE, &path)?);
            entries.insert(path.trim_start_matches("ld/").into(), key);
        }
    }
    let source = b.add(TOOL, RecipeStep {
        id: "binutils-ld-original-generators".into(),
        inputs: originals,
        outputs: BTreeMap::from([("source".into(), spec(ArtifactKind::Tree))]),
        operation: RecipeOperation::AssembleTree {
            output: "source".into(),
            entries,
        },
    })?["source"]
        .clone();
    let policy = b.artifact("binutils-ld-configured-makefile")?;
    let mut generated = BTreeMap::from([("executable".into(), executable.clone())]);
    for emulation in EMULATIONS {
        let result = b.add(TOOL, RecipeStep {
            id: format!("binutils-ld-genscripts-{emulation}"),
            inputs: BTreeMap::from([
                ("executable".into(), executable.clone()),
                ("source".into(), source.clone()),
                ("policy".into(), policy.clone()),
            ]),
            outputs: BTreeMap::from([
                ("generated".into(), spec(ArtifactKind::Tree)),
                ("stdout".into(), spec(ArtifactKind::Text)),
                ("stderr".into(), spec(ArtifactKind::Text)),
                ("exit-status".into(), RecipeOutput {
                    kind: ArtifactKind::Text,
                    bytes_max: 4,
                }),
            ]),
            operation: RecipeOperation::Exec {
                invocation_input: None,
                command_state: None,
                merge_stderr: false,
                executable_input: "executable".into(),
                arguments: vec![
                    literal("generate"),
                    input("source", ""),
                    input("policy", ""),
                    literal(emulation),
                ],
                environment: BTreeMap::new(),
                runtime_mounts: BTreeMap::new(),
                output_seeds: BTreeMap::new(),
                guard: None,
                cwd_input: None,
                cwd_output: Some("generated".into()),
                stdin_input: None,
                exit_policy: RecipeExitPolicy::Exact { code: 0 },
            },
        })?["generated"]
            .clone();
        generated.insert((*emulation).into(), result);
    }
    let mut args = vec![literal("merge")];
    args.extend(EMULATIONS.iter().map(|name| input(name, "")));
    let tree = b.add(TOOL, RecipeStep {
        id: "binutils-ld-generated-tree".into(),
        inputs: generated,
        outputs: BTreeMap::from([
            ("generated".into(), spec(ArtifactKind::Tree)),
            ("stdout".into(), spec(ArtifactKind::Text)),
            ("stderr".into(), spec(ArtifactKind::Text)),
            ("exit-status".into(), RecipeOutput {
                kind: ArtifactKind::Text,
                bytes_max: 4,
            }),
        ]),
        operation: RecipeOperation::Exec {
            invocation_input: None,
            command_state: None,
            merge_stderr: false,
            executable_input: "executable".into(),
            arguments: args,
            environment: BTreeMap::new(),
            runtime_mounts: BTreeMap::new(),
            output_seeds: BTreeMap::new(),
            guard: None,
            cwd_input: None,
            cwd_output: Some("generated".into()),
            stdin_input: None,
            exit_policy: RecipeExitPolicy::Exact { code: 0 },
        },
    })?["generated"]
        .clone();
    if b.artifacts.insert("publication-binutils-ldscripts".into(), tree.clone()).is_some() {
        return Err(error("duplicate ld script publication"));
    }
    Ok(tree)
}

pub(super) const ENGINE: &str = r####"#define _GNU_SOURCE
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ctype.h>
#include <stdarg.h>
#include <sys/stat.h>
#include <dirent.h>
#include <unistd.h>
#include <fnmatch.h>
#include <regex.h>
#include <errno.h>

/* This is a phase interpreter, not a process launcher.  In particular there is
   deliberately no external-command path.  Sources, streams and filesystem
   effects are evaluated in this address space. */
#define dup ld_string_duplicate
static void die(const char *fmt, ...) { va_list a; va_start(a,fmt); vfprintf(stderr,fmt,a); va_end(a); fputc('\n',stderr); exit(1); }
static void *alloc(size_t n) { void *p=calloc(1,n?n:1); if(!p)die("ld generator: out of memory"); return p; }
static char *slice(const char *p,size_t n) { char *r=alloc(n+1); memcpy(r,p,n);return r; }
static char *dup(const char *p) { return slice(p,strlen(p)); }
typedef struct {char *s;size_t n,c;} Buf;
static void putn(Buf *b,const char *p,size_t n) { if(b->n+n+1>b->c){b->c=(b->n+n+1)*2;b->s=realloc(b->s,b->c);if(!b->s)die("out of memory");}memcpy(b->s+b->n,p,n);b->n+=n;b->s[b->n]=0; }
static void putsb(Buf *b,const char *s){putn(b,s,strlen(s));}
static char *done(Buf *b){return b->s?b->s:dup("");}
static char *load(const char *path){FILE *f=fopen(path,"rb");Buf b={0};char q[8192];size_t n;if(!f)die("reading original %s: %s",path,strerror(errno));while((n=fread(q,1,sizeof q,f)))putn(&b,q,n);if(ferror(f))die("reading %s",path);fclose(f);return done(&b);}
static void copyfile(const char *from,const char *to){char *p=load(from);FILE *f=fopen(to,"wb");if(!f)die("writing %s",to);if(fwrite(p,1,strlen(p),f)!=strlen(p)||fclose(f))die("writing %s",to);free(p);}
static char *join(const char *a,const char *b){Buf r={0};putsb(&r,a);putsb(&r,"/");putsb(&r,b);return done(&r);}

typedef struct Var Var; struct Var {char *key,*value;Var *next;};
typedef struct Node Node;
typedef struct Fun Fun;struct Fun{char *name;Node *body;Fun *next;};
typedef struct {Var *vars;Fun *funs;char **args;int argc,status;FILE *in,*out;} Env;
static const char *get(Env *e,const char *key){Var *v;for(v=e->vars;v;v=v->next)if(!strcmp(v->key,key))return v->value;return NULL;}
static void set(Env *e,const char *key,const char *value){Var *v;for(v=e->vars;v;v=v->next)if(!strcmp(v->key,key)){free(v->value);v->value=value?dup(value):NULL;return;}v=alloc(sizeof *v);v->key=dup(key);v->value=value?dup(value):NULL;v->next=e->vars;e->vars=v;}
static Env clone(Env *e){Env r=*e;Var *v;r.vars=NULL;for(v=e->vars;v;v=v->next)set(&r,v->key,v->value);return r;}
static void clearvars(Env *e){Var *v=e->vars;while(v){Var *n=v->next;free(v->key);free(v->value);free(v);v=n;}}

typedef struct Tok Tok;struct Tok{char *s,*doc;int quoted;};
typedef struct {Tok **v;int n,c,pos;} Tokens;
static Tok *token(Tokens *t,const char *s,size_t n){Tok *x=alloc(sizeof *x);x->s=slice(s,n);if(t->n==t->c){t->c=t->c?t->c*2:128;t->v=realloc(t->v,t->c*sizeof *t->v);}t->v[t->n++]=x;return x;}
static const char *balanced(const char *p,char end){if(end=='\''){const char *q=strchr(p,'\'');if(!q)die("unterminated single quote");return q+1;}while(*p){if(*p=='\\'&&p[1]){p+=2;continue;}if(*p==end)return p+1;if(p[0]=='$'&&p[1]=='{'){p=balanced(p+2,'}');continue;}if(*p=='`'){p=balanced(p+1,'`');continue;}if(end!='"'&&(*p=='\''||*p=='"')){p=balanced(p+1,*p);continue;}p++;}die("unterminated original shell expansion");return p;}
static const char *wordend(const char *p){while(*p&&!strchr(" \t\r\n;|&()<>",*p)){if(*p=='\\'&&p[1])p+=2;else if(*p=='\''||*p=='"')p=balanced(p+1,*p);else if(*p=='`')p=balanced(p+1,'`');else if(p[0]=='$'&&p[1]=='{')p=balanced(p+2,'}');else if(p[0]=='$'&&p[1]=='(')p=balanced(p+2,')');else p++;}return p;}
static Tokens lex(const char *p){Tokens t={0};Tok *pending[32];char *delim[32];int strip[32],np=0;while(*p){const char *s;Tok *x;if(*p==' '||*p=='\t'||*p=='\r'){p++;continue;}if(p[0]=='\\'&&p[1]=='\n'){p+=2;continue;}if(*p=='#'){while(*p&&*p!='\n')p++;continue;}if(*p=='\n'){token(&t,";",1);p++;for(int i=0;i<np;i++){Buf b={0};int found=0;while(*p){s=p;while(*p&&*p!='\n')p++;const char *line=s;if(strip[i])while(*line=='\t')line++;if((size_t)(p-line)==strlen(delim[i])&&!memcmp(line,delim[i],p-line)){if(*p)p++;found=1;break;}putn(&b,line,p-line);if(*p){putsb(&b,"\n");p++;}}if(!found)die("unterminated heredoc %s",delim[i]);pending[i]->doc=done(&b);}np=0;continue;}s=p;if(strchr(";|&()<>",*p)){p++;if((*s=='|'&&*p=='|')||(*s=='&'&&*p=='&')||(*s==';'&&*p==';')||(*s=='>'&&*p=='>')||(*s=='<'&&*p=='<'))p++;if(*s=='<'&&p-s==2&&*p=='-')p++;x=token(&t,s,p-s);if(!strncmp(x->s,"<<",2)){while(*p==' '||*p=='\t')p++;s=p;p=wordend(p);Tok *d=token(&t,s,p-s);char *a=d->s;size_t n=strlen(a);if(np==32)die("too many heredocs");pending[np]=x;strip[np]=!strcmp(x->s,"<<-");x->quoted=(*a=='\''||*a=='"');delim[np++]=x->quoted?slice(a+1,n-2):dup(a);}continue;}p=wordend(p);if(p==s)die("shell lexer stalled");token(&t,s,p-s);}if(np)die("missing heredoc newline");token(&t,"",0);return t;}

enum {CMD=1,SEQ,AND,OR,PIPE,IF,CASE,FOR,FUNC,GROUP};
typedef struct Redir Redir;struct Redir{char *op,*word,*doc;int quoted;Redir *next;};
struct Node{int kind,n;char **words;Node *a,*b,*c,*next;Redir *redirs;};
static Node *node(int k){Node *n=alloc(sizeof *n);n->kind=k;return n;}
static char *peek(Tokens *t){return t->v[t->pos]->s;}
static int is(Tokens *t,const char *s){return !strcmp(peek(t),s);}
static int eat(Tokens *t,const char *s){if(is(t,s)){t->pos++;return 1;}return 0;}
static void expect(Tokens *t,const char *s){if(!eat(t,s))die("expected %s, found %s",s,peek(t));}
static void semis(Tokens *t){while(eat(t,";"));}
static void addword(Node *n,char *s){n->words=realloc(n->words,(n->n+1)*sizeof *n->words);n->words[n->n++]=s;}
static int stop(Tokens *t){char *s=peek(t);return !*s||!strcmp(s,"then")||!strcmp(s,"else")||!strcmp(s,"elif")||!strcmp(s,"fi")||!strcmp(s,"do")||!strcmp(s,"done")||!strcmp(s,"esac")||!strcmp(s,";;")||!strcmp(s,"}")||!strcmp(s,")");}
static Node *list(Tokens *t);
static void redirects(Tokens *t,Node *n){Redir **tail=&n->redirs;while(*tail)tail=&(*tail)->next;while(*peek(t)=='>'||*peek(t)=='<'){Tok *x=t->v[t->pos++];Redir *r=alloc(sizeof *r);r->op=x->s;r->doc=x->doc;r->quoted=x->quoted;r->word=peek(t);t->pos++;*tail=r;tail=&r->next;}}
static Node *ifnode(Tokens *t){Node *n=node(IF);n->a=list(t);expect(t,"then");n->b=list(t);if(eat(t,"elif"))n->c=ifnode(t);else{if(eat(t,"else"))n->c=list(t);expect(t,"fi");}return n;}
static Node *command(Tokens *t){Node *n;semis(t);if(eat(t,"if"))n=ifnode(t);else if(eat(t,"case")){n=node(CASE);addword(n,peek(t));t->pos++;expect(t,"in");semis(t);Node **tail=&n->a;while(!is(t,"esac")){Node *arm=node(CMD);eat(t,"(");do{addword(arm,peek(t));t->pos++;}while(eat(t,"|"));expect(t,")");arm->a=list(t);*tail=arm;tail=&arm->next;if(!eat(t,";;")&&!is(t,"esac"))die("case arm delimiter %s",peek(t));semis(t);}expect(t,"esac");}else if(eat(t,"for")){n=node(FOR);addword(n,peek(t));t->pos++;if(eat(t,"in")){while(!is(t,";")&&!is(t,"do")){addword(n,peek(t));t->pos++;}}semis(t);expect(t,"do");n->a=list(t);expect(t,"done");}else if(eat(t,"(")){n=node(GROUP);n->n=1;n->a=list(t);expect(t,")");}else if(eat(t,"{")){n=node(GROUP);n->a=list(t);expect(t,"}");}else if(*peek(t)&&!strcmp(t->v[t->pos+1]->s,"(")&&!strcmp(t->v[t->pos+2]->s,")")){n=node(FUNC);addword(n,peek(t));t->pos+=3;semis(t);expect(t,"{");n->a=list(t);expect(t,"}");}else{n=node(CMD);while(*peek(t)&&!strchr(";|&)",*peek(t))&&!stop(t)){if(*peek(t)=='>'||*peek(t)=='<')redirects(t,n);else{addword(n,peek(t));t->pos++;}}if(!n->n&&!n->redirs)die("empty shell command before %s",peek(t));}redirects(t,n);return n;}
static Node *pipeline(Tokens *t){Node *n=command(t);while(eat(t,"|")){semis(t);Node *r=node(PIPE);r->a=n;r->b=command(t);n=r;}return n;}
static Node *andor(Tokens *t){Node *n=pipeline(t);while(is(t,"&&")||is(t,"||")){int k=eat(t,"&&")?AND:(expect(t,"||"),OR);semis(t);Node *r=node(k);r->a=n;r->b=pipeline(t);n=r;}return n;}
static Node *list(Tokens *t){Node *n=node(SEQ),**tail=&n->a;semis(t);while(!stop(t)){*tail=andor(t);tail=&(*tail)->next;if(!eat(t,";")){if(!stop(t))die("missing command separator before %s",peek(t));}semis(t);}return n;}
static Node *parse(const char *s){Tokens t=lex(s);Node *n=list(&t);if(*peek(&t))die("unexpected shell token %s",peek(&t));return n;}
static int run(Env *e,Node *n);
static int script(Env *e,const char *s){return run(e,parse(s));}
typedef struct {Buf text,mask;int forced;} Expanded;
static void emit(Expanded *x,const char *s,int protected){size_t n=strlen(s);putn(&x->text,s,n);for(size_t i=0;i<n;i++){char c=protected?1:0;putn(&x->mask,&c,1);}}
static Expanded expand(Env *e,const char *s,int mode);
static char *capture(Env *e,const char *s){Env child=clone(e);char *text=NULL;size_t size=0;Buf command={0};for(const char *p=s;*p;p++){if(*p=='\\'&&p[1]&&strchr("$`\\",p[1]))p++;putn(&command,p,1);}char *source=done(&command);child.out=open_memstream(&text,&size);if(!child.out)die("capture stream");script(&child,source);fclose(child.out);free(source);clearvars(&child);while(size&&text[size-1]=='\n')text[--size]=0;return text;}
static char *parameter(Env *e,const char *s,int mode){const char *p=s;char number[64];const char *v;int colon=0;while(isalnum((unsigned char)*p)||*p=='_')p++;if(p==s&&strchr("@*#$?",*p)&&*p)p++;char *key=slice(s,p-s);if(!strcmp(key,"#")){snprintf(number,sizeof number,"%d",e->argc);v=number;}else if(!strcmp(key,"?")){snprintf(number,sizeof number,"%d",e->status);v=number;}else if(!strcmp(key,"$"))v="1";else if(!strcmp(key,"@")||!strcmp(key,"*")){Buf b={0};for(int i=0;i<e->argc;i++){if(i)putsb(&b," ");putsb(&b,e->args[i]);}v=done(&b);}else if(isdigit((unsigned char)*key)){int i=atoi(key);v=i>0&&i<=e->argc?e->args[i-1]:NULL;}else v=get(e,key);if(!*p){free(key);return dup(v?v:"");}if(*p==':'){colon=1;p++;}char op=*p++;int present=v&&(!colon||*v);if(op=='-'||op=='+'||op=='='||op=='?'){int use=op=='+'?present:!present;if(use){Expanded a=expand(e,p,mode);char *r=done(&a.text);free(a.mask.s);if(op=='?')die("%s: %s",key,r);if(op=='=')set(e,key,r);free(key);return r;}free(key);return dup(op=='+'?"":v?v:"");}die("unsupported original parameter expansion %s",s);return NULL;}
static Expanded expand(Env *e,const char *s,int mode){Expanded x={0};int q=0;const char *p=s;while(*p){char c=*p;if(!mode&&(c=='\''||c=='"')){if(!q){q=c;x.forced=1;p++;continue;}if(q==c){q=0;p++;continue;}}if(c=='\\'&&q!='\''&&p[1]){if(p[1]=='\n'){p+=2;continue;}if(q=='"'||mode){const char *special=mode?"$`\\":"$`\\\"";if(!strchr(special,p[1])){char v[2]={c,0};emit(&x,v,1);p++;continue;}}char v[2]={p[1],0};emit(&x,v,1);p+=2;continue;}if(q!='\''&&c=='$'){if(p[1]=='{'){const char *end=balanced(p+2,'}');char *inner=slice(p+2,end-p-3);char *v=parameter(e,inner,mode);emit(&x,v,q||mode);free(inner);free(v);p=end;continue;}if(p[1]=='(')die("unexpected command substitution form in original ld sources");const char *end=p+1;if(strchr("@*#$?",*end)&&*end)end++;else if(isdigit((unsigned char)*end))end++;else while(isalnum((unsigned char)*end)||*end=='_')end++;if(end>p+1){char *key=slice(p+1,end-p-1);char *v=parameter(e,key,mode);emit(&x,v,q||mode);free(v);free(key);p=end;continue;}}if(q!='\''&&c=='`'){const char *end=balanced(p+1,'`');char *cmd=slice(p+1,end-p-2);char *v=capture(e,cmd);emit(&x,v,q||mode);free(cmd);free(v);p=end;continue;}char v[2]={c,0};emit(&x,v,q||mode);p++;}return x;}
static char *scalar(Env *e,const char *s){Expanded x=expand(e,s,0);free(x.mask.s);return done(&x.text);}
static char *pattern(Env *e,const char *s){Expanded x=expand(e,s,0);Buf b={0};for(size_t i=0;i<x.text.n;i++){if(x.mask.s[i]&&strchr("*?[\\",x.text.s[i]))putsb(&b,"\\");putn(&b,x.text.s+i,1);}free(x.text.s);free(x.mask.s);return done(&b);}
typedef struct {char **v;int n;} Args;
static void arg(Args *a,char *s){a->v=realloc(a->v,(a->n+1)*sizeof *a->v);a->v[a->n++]=s;}
static void fields(Env *e,Args *a,const char *raw){if(!strcmp(raw,"\"$@\"")){for(int i=0;i<e->argc;i++)arg(a,dup(e->args[i]));return;}Expanded x=expand(e,raw,0);size_t i=0;int before=a->n;while(i<x.text.n){while(i<x.text.n&&!x.mask.s[i]&&strchr(" \t\n",x.text.s[i]))i++;size_t start=i;while(i<x.text.n&&(x.mask.s[i]||!strchr(" \t\n",x.text.s[i])))i++;if(i>start)arg(a,slice(x.text.s+start,i-start));}if(before==a->n&&x.forced)arg(a,dup(""));free(x.text.s);free(x.mask.s);}
static int assignment(const char *s){const char *p=s;if(!isalpha((unsigned char)*p)&&*p!='_')return 0;while(isalnum((unsigned char)*p)||*p=='_')p++;return *p=='=';}

/* POSIX BRE sed subset used by the original generators and stringify.sed.
   Commands are parsed, not recognized by a precomputed source-text match. */
typedef struct Sed Sed;struct Sed{int address,end,active,global;char kind;regex_t re,re2;int has_re,has_re2;char *replacement;Sed *next;};
static char *delimited(const char **pp,char delimiter){const char *p=*pp;Buf b={0};while(*p&&*p!=delimiter){if(*p=='\\'&&p[1]){if(p[1]==delimiter){putn(&b,p+1,1);p+=2;continue;}putn(&b,p,2);p+=2;}else putn(&b,p++,1);}if(*p!=delimiter)die("unterminated sed expression");*pp=p+1;return done(&b);}
static void regex_compile(regex_t *r,const char *s){int rc=regcomp(r,s,0);if(rc){char b[256];regerror(rc,r,b,sizeof b);die("original sed regex %s: %s",s,b);}}
static void sed_parse(Sed ***tail,const char *p){while(*p){while(isspace((unsigned char)*p)||*p==';')p++;if(!*p)break;Sed *s=alloc(sizeof *s);if(*p=='/'){p++;char *r=delimited(&p,'/');regex_compile(&s->re,r);free(r);s->has_re=1;if(*p==','){p++;if(*p!='/')die("invalid sed range");p++;r=delimited(&p,'/');regex_compile(&s->re2,r);free(r);s->has_re2=1;}}else if(isdigit((unsigned char)*p)){s->address=strtol(p,(char**)&p,10);}else if(*p=='$'){s->address=-1;p++;}while(*p==' '||*p=='\t')p++;s->kind=*p++;if(s->kind=='s'){char delimiter=*p++;char *r=delimited(&p,delimiter);regex_compile(&s->re2,r);free(r);s->replacement=delimited(&p,delimiter);if(*p=='g'){s->global=1;p++;}}else if(s->kind!='d')die("unsupported sed command %c",s->kind);**tail=s;*tail=&s->next;}}
static char *sed_replace(Sed *s,const char *text){Buf b={0};size_t offset=0;regmatch_t m[10];int matched=0;while(!regexec(&s->re2,text+offset,10,m,offset?REG_NOTBOL:0)){size_t start=offset+m[0].rm_so,end=offset+m[0].rm_eo;putn(&b,text+offset,start-offset);const char *p=s->replacement;while(*p){if(*p=='&'){putn(&b,text+start,end-start);p++;}else if(*p=='\\'&&p[1]){p++;if(isdigit((unsigned char)*p)){int k=*p++-'0';if(m[k].rm_so>=0)putn(&b,text+offset+m[k].rm_so,m[k].rm_eo-m[k].rm_so);}else putn(&b,p++,1);}else putn(&b,p++,1);}matched=1;offset=end;if(!s->global)break;if(start==end){if(!text[offset])break;putn(&b,text+offset,1);offset++;}}if(!matched)return dup(text);putsb(&b,text+offset);return done(&b);}
static void sed_stream(Sed *program,FILE *in,FILE *out){char *line=NULL;size_t cap=0;ssize_t count;long number=0;while((count=getline(&line,&cap,in))>=0){number++;int newline=count&&line[count-1]=='\n';if(newline)line[--count]=0;int last=0;int c=fgetc(in);if(c==EOF)last=1;else ungetc(c,in);char *text=dup(line);int deleted=0;for(Sed *s=program;s;s=s->next){int selected=1;if(s->address)selected=s->address<0?last:number==s->address;if(s->has_re){if(s->has_re2){if(!s->active&&!regexec(&s->re,text,0,NULL,0))s->active=1;selected=s->active;if(s->active&&!regexec(&s->re2,text,0,NULL,0))s->active=0;}else selected=!regexec(&s->re,text,0,NULL,0);}if(!selected)continue;if(s->kind=='d'){deleted=1;break;}char *next=sed_replace(s,text);free(text);text=next;}if(!deleted){fputs(text,out);if(newline)fputc('\n',out);}free(text);}free(line);}
static int sed_command(Env *e,Args *a){Sed *program=NULL,**tail=&program;int i=1,expressions=0;while(i<a->n&&a->v[i][0]=='-'){if(!strcmp(a->v[i],"-e")&&i+1<a->n){sed_parse(&tail,a->v[i+1]);i+=2;expressions++;}else if(!strcmp(a->v[i],"-f")&&i+1<a->n){char *text=load(a->v[i+1]);sed_parse(&tail,text);free(text);i+=2;expressions++;}else die("unsupported sed option %s",a->v[i]);}if(!expressions){if(i==a->n)die("missing sed expression");sed_parse(&tail,a->v[i++]);}if(i==a->n)sed_stream(program,e->in,e->out);else for(;i<a->n;i++){FILE *f=fopen(a->v[i],"r");if(!f)die("sed input %s: %s",a->v[i],strerror(errno));sed_stream(program,f,e->out);fclose(f);}while(program){Sed *next=program->next;if(program->has_re)regfree(&program->re);if(program->kind=='s'||program->has_re2)regfree(&program->re2);free(program->replacement);free(program);program=next;}return 0;}
typedef struct Cache Cache;struct Cache{char *path;Node *program;Cache *next;};
static Cache *cache;
static Node *source_program(const char *path){for(Cache *c=cache;c;c=c->next)if(!strcmp(c->path,path))return c->program;Cache *c=alloc(sizeof *c);c->path=dup(path);char *s=load(path);c->program=parse(s);free(s);c->next=cache;cache=c;return c->program;}
static void transfer(FILE *in,FILE *out){char b[8192];size_t n;while((n=fread(b,1,sizeof b,in)))if(fwrite(b,1,n,out)!=n)die("generator stream write");if(ferror(in))die("generator stream read");}
static int test_command(Args *a){int n=a->n-1;char **v=a->v+1;if(!strcmp(a->v[0],"[")){if(n<1||strcmp(v[n-1],"]"))die("missing ]");n--;}if(!n)return 1;if(n==1)return !*v[0];if(n==2){if(!strcmp(v[0],"!"))return !(!*v[1]);if(!strcmp(v[0],"-n"))return !*v[1];if(!strcmp(v[0],"-z"))return !!*v[1];struct stat st;if(!strcmp(v[0],"-d"))return stat(v[1],&st)||!S_ISDIR(st.st_mode);if(!strcmp(v[0],"-f"))return stat(v[1],&st)||!S_ISREG(st.st_mode);if(!strcmp(v[0],"-s"))return stat(v[1],&st)||!st.st_size;if(!strcmp(v[0],"-r")||!strcmp(v[0],"-e"))return access(v[1],R_OK)!=0;}if(n==3){if(!strcmp(v[1],"="))return strcmp(v[0],v[2])!=0;if(!strcmp(v[1],"!="))return strcmp(v[0],v[2])==0;if(!strcmp(v[1],"-eq"))return strtol(v[0],NULL,10)!=strtol(v[2],NULL,10);if(!strcmp(v[1],"-ne"))return strtol(v[0],NULL,10)==strtol(v[2],NULL,10);}die("unsupported original test expression");return 1;}
static int builtin(Env *e,Args *a){const char *name=a->v[0];
  if(!strcmp(name,":")||!strcmp(name,"true"))return 0;
  if(!strcmp(name,"false"))return 1;
  if(!strcmp(name,"test")||!strcmp(name,"["))return test_command(a);
  if(!strcmp(name,"echo")){int start=1,newline=1;if(a->n>1&&!strcmp(a->v[1],"-n")){start++;newline=0;}for(int i=start;i<a->n;i++){if(i>start)fputc(' ',e->out);fputs(a->v[i],e->out);}if(newline)fputc('\n',e->out);return 0;}
  if(!strcmp(name,"cat")){if(a->n==1)transfer(e->in,e->out);else for(int i=1;i<a->n;i++){FILE *f=fopen(a->v[i],"rb");if(!f)die("cat %s: %s",a->v[i],strerror(errno));transfer(f,e->out);fclose(f);}return 0;}
  if(!strcmp(name,"sed"))return sed_command(e,a);
  if(!strcmp(name,"cmp")){int i=1;if(i<a->n&&!strcmp(a->v[i],"-s"))i++;if(a->n-i!=2)die("original cmp arguments");FILE *f=fopen(a->v[i],"rb"),*g=fopen(a->v[i+1],"rb");if(!f||!g)die("original cmp input");int c,d,result=0;do{c=fgetc(f);d=fgetc(g);if(c!=d){result=1;break;}}while(c!=EOF);if(ferror(f)||ferror(g))die("original cmp read");fclose(f);fclose(g);return result;}
  if(!strcmp(name,"mkdir")){for(int i=1;i<a->n;i++)if(mkdir(a->v[i],0755)&&errno!=EEXIST)die("mkdir %s",a->v[i]);return 0;}
  if(!strcmp(name,"rm")){for(int i=1;i<a->n;i++)if(strcmp(a->v[i],"-f")&&unlink(a->v[i])&&errno!=ENOENT)die("unlink %s",a->v[i]);return 0;}
  if(!strcmp(name,"unset")){for(int i=1;i<a->n;i++)set(e,a->v[i],NULL);return 0;}
  if(!strcmp(name,"set")){e->argc=a->n-1;e->args=alloc((e->argc+1)*sizeof *e->args);for(int i=0;i<e->argc;i++)e->args[i]=dup(a->v[i+1]);return 0;}
  if(!strcmp(name,"shift")){int n=a->n>1?atoi(a->v[1]):1;if(n<0||n>e->argc)return 1;e->args+=n;e->argc-=n;return 0;}
  if(!strcmp(name,".")){if(a->n!=2)die("original source arguments");return run(e,source_program(a->v[1]));}
  if(!strcmp(name,"eval")){Buf b={0};for(int i=1;i<a->n;i++){if(i>1)putsb(&b," ");putsb(&b,a->v[i]);}char *s=done(&b);int r=script(e,s);free(s);return r;}
  die("unsupported original ld command %s",name);return 1;
}
static int simple(Env *e,Node *n){Args a={0};int i=0;while(i<n->n&&assignment(n->words[i])){char *raw=n->words[i++],*eq=strchr(raw,'=');char *key=slice(raw,eq-raw),*value=scalar(e,eq+1);set(e,key,value);free(key);free(value);}for(;i<n->n;i++)fields(e,&a,n->words[i]);if(!a.n){free(a.v);return 0;}int result;Fun *f;for(f=e->funs;f;f=f->next)if(!strcmp(f->name,a.v[0]))break;if(f){char **oldargs=e->args;int oldargc=e->argc;e->args=a.v+1;e->argc=a.n-1;result=run(e,f->body);e->args=oldargs;e->argc=oldargc;}else result=builtin(e,&a);for(i=0;i<a.n;i++)free(a.v[i]);free(a.v);return result;}
static int execute(Env *e,Node *n){int result=0;switch(n->kind){
  case CMD:return simple(e,n);
  case SEQ:for(Node *p=n->a;p;p=p->next)result=run(e,p);return result;
  case AND:result=run(e,n->a);return result?result:run(e,n->b);
  case OR:result=run(e,n->a);return result?run(e,n->b):result;
  case IF:return run(e,n->a)==0?run(e,n->b):n->c?run(e,n->c):0;
  case FUNC:{Fun *f=alloc(sizeof *f);f->name=n->words[0];f->body=n->a;f->next=e->funs;e->funs=f;return 0;}
  case GROUP:if(n->n){Env child=clone(e);result=run(&child,n->a);clearvars(&child);return result;}return run(e,n->a);
  case PIPE:{FILE *stream=tmpfile();if(!stream)die("pipeline stream");Env left=clone(e),right=clone(e);left.out=stream;run(&left,n->a);if(fflush(stream)||fseek(stream,0,SEEK_SET))die("pipeline rewind");right.in=stream;result=run(&right,n->b);clearvars(&left);clearvars(&right);fclose(stream);return result;}
  case CASE:{char *value=scalar(e,n->words[0]);for(Node *arm=n->a;arm;arm=arm->next)for(int i=0;i<arm->n;i++){char *p=pattern(e,arm->words[i]);int match=fnmatch(p,value,0)==0;free(p);if(match){free(value);return run(e,arm->a);}}free(value);return 0;}
  case FOR:{Args a={0};if(n->n==1){for(int i=0;i<e->argc;i++)arg(&a,dup(e->args[i]));}else for(int i=1;i<n->n;i++)fields(e,&a,n->words[i]);for(int i=0;i<a.n;i++){set(e,n->words[0],a.v[i]);result=run(e,n->a);free(a.v[i]);}free(a.v);return result;}
  default:die("invalid original shell node");}return 1;
}
static int run(Env *e,Node *n){if(!n)return 0;FILE *oldin=e->in,*oldout=e->out,*opened[32];int count=0;for(Redir *r=n->redirs;r;r=r->next){if(count==32)die("too many redirections");char *path=scalar(e,r->word);FILE *f;if(!strncmp(r->op,"<<",2)){char *body;if(r->quoted)body=dup(r->doc);else{Expanded x=expand(e,r->doc,1);body=done(&x.text);free(x.mask.s);}f=tmpfile();if(!f)die("heredoc stream");fwrite(body,1,strlen(body),f);rewind(f);free(body);e->in=f;}else if(!strcmp(r->op,"<")){f=fopen(path,"rb");if(!f)die("redirect input %s",path);e->in=f;}else{f=fopen(path,!strcmp(r->op,">>")?"ab":"wb");if(!f)die("redirect output %s: %s",path,strerror(errno));e->out=f;}opened[count++]=f;free(path);}int status=execute(e,n);for(int i=count-1;i>=0;i--)if(fclose(opened[i]))die("closing generator redirect");e->in=oldin;e->out=oldout;e->status=status;return status;}

/* Read the configured Makefile, retaining recursive variable semantics rather
   than installing configured output text as an authored input. */
static Env makevars(const char *path){Env e={0};char *text=load(path);Buf logical={0};for(char *p=text;*p;p++){if(p[0]=='\\'&&p[1]=='\n'){putsb(&logical," ");p++;}else putn(&logical,p,1);}char *all=done(&logical),*save=NULL;for(char *line=strtok_r(all,"\n",&save);line;line=strtok_r(NULL,"\n",&save)){if(*line=='\t'||*line=='#')continue;char *eq=strchr(line,'=');if(!eq)continue;char *end=eq;while(end>line&&isspace((unsigned char)end[-1]))end--;int append=end>line&&end[-1]=='+';if(append)end--;while(end>line&&isspace((unsigned char)end[-1]))end--;char *start=line;while(isspace((unsigned char)*start))start++;if(start==end)continue;int valid=1;for(char *p=start;p<end;p++)if(!isalnum((unsigned char)*p)&&*p!='_')valid=0;if(!valid)continue;char *key=slice(start,end-start),*value=eq+1;while(*value==' '||*value=='\t')value++;char *tail=value+strlen(value);while(tail>value&&isspace((unsigned char)tail[-1]))*--tail=0;if(append){Buf b={0};const char *old=get(&e,key);if(old){putsb(&b,old);putsb(&b," ");}putsb(&b,value);char *s=done(&b);set(&e,key,s);free(s);}else set(&e,key,value);free(key);}free(all);free(text);return e;}
static char *makevalue(Env *e,const char *key,int depth){if(depth>32)die("recursive Makefile variable %s",key);const char *raw=get(e,key);if(!raw)die("missing configured ld Makefile variable %s",key);Buf b={0};for(const char *p=raw;*p;){if(p[0]=='$'&&(p[1]=='{'||p[1]=='(')){char close=p[1]=='{'?'}':')';const char *end=strchr(p+2,close);if(!end)die("Makefile expansion %s",key);char *name=slice(p+2,end-p-2);char *value=makevalue(e,name,depth+1);putsb(&b,value);free(name);free(value);p=end+1;}else if(p[0]=='$'&&p[1]=='$'){putsb(&b,"$");p+=2;}else putn(&b,p++,1);}return done(&b);}
static void merge(const char *root){DIR *d=opendir(root);if(!d)die("generated tree %s",root);struct dirent *entry;while((entry=readdir(d))){if(!strcmp(entry->d_name,".")||!strcmp(entry->d_name,".."))continue;char *path=join(root,entry->d_name);struct stat st;if(lstat(path,&st))die("generated member %s",path);if(S_ISDIR(st.st_mode)){if(strcmp(entry->d_name,"ldscripts")){free(path);continue;}if(mkdir("ldscripts",0755)&&errno!=EEXIST)die("merge ldscripts");DIR *scripts=opendir(path);if(!scripts)die("open ldscripts");struct dirent *s;while((s=readdir(scripts))){if(s->d_name[0]=='.')continue;char *from=join(path,s->d_name),*to=join("ldscripts",s->d_name);if(!access(to,F_OK))die("duplicate generated member %s",to);copyfile(from,to);free(from);free(to);}closedir(scripts);}else if(S_ISREG(st.st_mode)&&entry->d_name[0]=='e'&&strstr(entry->d_name,".c")){if(!access(entry->d_name,F_OK))die("duplicate emulation %s",entry->d_name);copyfile(path,entry->d_name);}free(path);}closedir(d);}
int main(int argc,char **argv){if(argc>=3&&!strcmp(argv[1],"merge")){for(int i=2;i<argc;i++)merge(argv[i]);return 0;}if(argc!=5||strcmp(argv[1],"generate"))die("usage: ld-phase generate SOURCE CONFIGURED-MAKEFILE EMULATION | merge TREES...");const char *emulation=argv[4];Env policy=makevars(argv[3]),e={0};e.in=stdin;e.out=stdout;const char *names[]={"libdir","prefix","exec_prefix","host","target","target_alias","EMULATION_LIBPATH","NATIVE_LIB_DIRS","use_sysroot","enable_initfini_array"};Args args={0};arg(&args,dup(argv[2]));for(size_t i=0;i<sizeof names/sizeof names[0];i++)arg(&args,makevalue(&policy,names[i],0));arg(&args,dup(emulation));char *tdir=join("tdir",emulation);tdir[4]='_';const char *tool=get(&policy,tdir);arg(&args,tool?makevalue(&policy,tdir,0):dup(""));free(tdir);e.args=args.v;e.argc=args.n;char *libpath=makevalue(&policy,"LIB_PATH",0);set(&e,"LIB_PATH",libpath);free(libpath);char *ofiles=makevalue(&policy,"EMULATION_OFILES",0);Buf expected={0};putsb(&expected,"e");putsb(&expected,emulation);putsb(&expected,".o");int selected=0;char *save=NULL;for(char *p=strtok_r(ofiles," \t",&save);p;p=strtok_r(NULL," \t",&save))if(!strcmp(p,expected.s))selected++;if(selected!=1)die("emulation %s absent or duplicated in configured original target",emulation);free(ofiles);free(expected.s);char *stringify=makevalue(&policy,"STRINGIFY",0),*templates=join(argv[2],"emultempl"),*stringifier=join(templates,stringify);copyfile(stringifier,"stringify.sed");free(stringify);free(templates);free(stringifier);char *entry=join(argv[2],"genscripts.sh");int result=run(&e,source_program(entry));if(result)die("original genscripts %s returned %d",emulation,result);if(unlink("stringify.sed"))die("remove temporary stringifier");Buf generated={0};putsb(&generated,"e");putsb(&generated,emulation);putsb(&generated,".c");struct stat st;if(stat(generated.s,&st)||!S_ISREG(st.st_mode))die("original generator did not produce %s",generated.s);return 0;}
"####;
