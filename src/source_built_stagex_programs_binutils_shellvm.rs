//! Source-owned shell continuation for the authenticated binutils configure scripts.
//!
//! The parser and evaluator start with the ld generator engine. Commands which
//! are not shell builtins cross an explicit request/result boundary; this helper
//! contains no process launcher. A continuation is a verified replay transcript,
//! not a dump of implementation-specific C stack addresses.
use crate::errors::RunError;
use crate::source_built_stagex_programs::error;

/// Produce the child-free interpreter source. Keep the original ld parser's
/// shared primitives rather than maintaining another shell language convention.
// Pending wiring in the source-only preparation path (Cairn prove-source-built-mantle-fixed-point I3).
#[allow(dead_code)]
pub(crate) fn source() -> Result<String, RunError> {
    let mut source = library_source()?;
    source.push_str("\nint main(int argc,char **argv){int status=mantle_controller_dispatch(argc,argv,mantle_shell_run);return status<0?mantle_shell_run(argc,argv):status;}\n");
    Ok(source)
}

/// One translation unit's shared parser, evaluator, immutable request reader and
/// result writer. Appended source drivers reuse `Request`, `read_request`,
/// `write_result` and `mantle_shell_run`; they do not duplicate the wire format.
pub(crate) fn library_source() -> Result<String, RunError> {
    let mut source =
        format!("{}\n{}", include_str!("source_built_stagex_programs_controller_paths.c"), super::ldscripts::ENGINE);
    replace_section(&mut source, "typedef struct Var Var;", "/* POSIX BRE sed subset", SHELL)?;
    replace_section(&mut source, "static int builtin(Env *e,Args *a)", "/* Read the configured Makefile", EVALUATOR)?;
    replace_section(&mut source, "static Node *source_program(", "static void transfer(", SOURCE_PROGRAM)?;
    replace_section(&mut source, "static int test_command(", "static int external(", TEST)?;
    let main = source
        .find("/* Read the configured Makefile")
        .ok_or_else(|| error("binutils shell engine has no driver boundary"))?;
    source.truncate(main);
    let language = source
        .find("typedef struct Var Var;")
        .ok_or_else(|| error("source shell language boundary disappeared"))?;
    source.insert_str(language, LANGUAGE_IO_BEGIN);
    source.push_str(LANGUAGE_IO_END);
    source.push_str(include_str!("source_built_stagex_programs_workspace_metadata.c"));
    if DRIVER.matches("int main(int argc,char **argv)").count() != 1 {
        return Err(error("shell continuation driver entrypoint drift"));
    }
    source.push_str(&DRIVER.replace("int main(int argc,char **argv)", "int mantle_shell_run(int argc,char **argv)"));
    source.push_str(include_str!("source_built_stagex_programs_controller_protocol.c"));
    source.push('\n');
    source = source.replace(
        "#include <errno.h>",
        "#include <errno.h>\n#include <glob.h>\n#include <limits.h>\n#include <stdint.h>",
    );
    Ok(source)
}

fn replace_section(source: &mut String, start: &str, end: &str, replacement: &str) -> Result<(), RunError> {
    let begin = source.find(start).ok_or_else(|| error(format!("binutils shell engine missing {start}")))?;
    let finish = source[begin..]
        .find(end)
        .map(|n| begin + n)
        .ok_or_else(|| error(format!("binutils shell engine missing {end}")))?;
    source.replace_range(begin..finish, replacement);
    Ok(())
}

const LANGUAGE_IO_BEGIN: &str = r#"
static char *mantle_source_load(const char *path){FILE *f=mantle_paths_fopen(path,"rb");Buf b={0};char q[8192];size_t n;if(!f)die("reading original %s: %s",path,strerror(errno));while((n=fread(q,1,sizeof q,f)))putn(&b,q,n);if(ferror(f)||fclose(f))die("reading %s",path);return done(&b);}
#define load(...) mantle_source_load(__VA_ARGS__)
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
"#;
const LANGUAGE_IO_END: &str = r#"
#undef load
#undef fopen
#undef open
#undef stat
#undef lstat
#undef access
#undef chdir
#undef getcwd
#undef opendir
#undef mkdir
#undef unlink
#undef rmdir
#undef rename
#undef readlink
#undef symlink
#undef link
#undef chmod
#undef utimensat
#undef glob
#undef globfree
"#;

const SHELL: &str = r####"
typedef struct Var Var; struct Var {char *key,*value;int exported,readonly;Var *next;};
typedef struct Node Node;
typedef struct Fun Fun;struct Fun{char *name;Node *body;Fun *next;};
typedef struct {Var *vars;Fun *funs;char **args,*zero,*cwd;int argc,status,flow,depth,suppress,errexit,nounset,noglob,allexport,noclobber,substituted,line;FILE *in,*out,*err,*fd[64];int fd_flags[64],fd_cloexec[64];mode_t creation_mask;int mask_valid;char *traps[32];} Env;
static void mantle_source_ensure_mask(Env *e){if(!e->mask_valid){mode_t actual=umask(0);umask(actual);e->creation_mask=actual;e->mask_valid=1;}if(e->creation_mask>0777)die("source creation mask exceeds permission bits");}
static Var *variable(Env *e,const char *key){for(Var *v=e->vars;v;v=v->next)if(!strcmp(v->key,key))return v;return NULL;}
static const char *get(Env *e,const char *key){Var *v=variable(e,key);return v?v->value:NULL;}
static void set(Env *e,const char *key,const char *value){Var *v=variable(e,key);if(v){if(v->readonly)die("%s: readonly variable",key);free(v->value);v->value=value?dup(value):NULL;return;}v=alloc(sizeof *v);v->key=dup(key);v->value=value?dup(value):NULL;v->exported=e->allexport;v->next=e->vars;e->vars=v;}
static Env clone(Env *e){Env r=*e;Var **tail=&r.vars;r.vars=NULL;r.cwd=dup(e->cwd?e->cwd:".");for(Var *v=e->vars;v;v=v->next){Var *n=alloc(sizeof *n);*n=*v;n->key=dup(v->key);n->value=v->value?dup(v->value):NULL;n->next=NULL;*tail=n;tail=&n->next;}return r;}
static void clearvars(Env *e){Var *v=e->vars;while(v){Var *n=v->next;free(v->key);free(v->value);free(v);v=n;}}

typedef struct Tok Tok;struct Tok{char *s,*doc;int quoted,line;size_t offset;};
typedef struct {Tok **v;int n,c,pos;} Tokens;
static const char *lex_origin,*lex_line_at;
static int lex_line;
static Tok *token(Tokens *t,const char *s,size_t n){Tok *x=alloc(sizeof *x);x->s=slice(s,n);x->offset=(size_t)(s-lex_origin);if(!lex_line_at||s<lex_line_at){lex_line_at=lex_origin;lex_line=1;}while(lex_line_at<s){if(*lex_line_at++=='\n')lex_line++;}x->line=lex_line;if(t->n==t->c){t->c=t->c?t->c*2:128;t->v=realloc(t->v,t->c*sizeof *t->v);if(!t->v)die("token allocation");}t->v[t->n++]=x;return x;}
static const char *balanced(const char *p,char end){
 if(end=='\''){const char *q=strchr(p,'\'');if(!q)die("unterminated single quote");return q+1;}
 int nesting=0;
 while(*p){if(*p=='\\'&&p[1]){p+=2;continue;}if(*p==end){if(end==')'&&nesting){nesting--;p++;continue;}return p+1;}
 if(p[0]=='$'&&p[1]=='{'){p=balanced(p+2,'}');continue;}
 if(p[0]=='$'&&p[1]=='('){p=balanced(p+2,')');continue;}
 if(*p=='`'&&end!='`'){p=balanced(p+1,'`');continue;}
 if(end!='"'&&(*p=='\''||*p=='"')){p=balanced(p+1,*p);continue;}
 if(end==')'&&*p=='(')nesting++;
 p++;}die("unterminated shell expansion");return p;
}
static const char *wordend(const char *p){while(*p&&!strchr(" \t\r\n;|&()<>",*p)){if(*p=='\\'&&p[1])p+=2;else if(*p=='\''||*p=='"')p=balanced(p+1,*p);else if(*p=='`')p=balanced(p+1,'`');else if(p[0]=='$'&&p[1]=='{')p=balanced(p+2,'}');else if(p[0]=='$'&&p[1]=='(')p=balanced(p+2,')');else p++;}return p;}
static char *unquote_delimiter(const char *p,int *quoted){Buf b={0};int q=0;while(*p){if((*p=='\''||*p=='"')&&(!q||q==*p)){*quoted=1;q=q?0:*p++;if(q==0)p++;continue;}if(*p=='\\'&&q!='\''&&p[1]){*quoted=1;p++;}putn(&b,p++,1);}if(q)die("unterminated heredoc delimiter");return done(&b);}
static Tokens lex(const char *p){
 Tokens t={0};Tok **pending=NULL;char **delim=NULL;int *strip=NULL,np=0;lex_origin=lex_line_at=p;lex_line=1;
 while(*p){const char *s;Tok *x;
 if(*p==' '||*p=='\t'||*p=='\r'){p++;continue;}if(p[0]=='\\'&&p[1]=='\n'){p+=2;continue;}
 if(*p=='#'){while(*p&&*p!='\n')p++;continue;}
 if(*p=='\n'){x=token(&t,p,1);free(x->s);x->s=dup(";");p++;
 for(int i=0;i<np;i++){Buf b={0};int found=0;while(*p){s=p;while(*p&&*p!='\n')p++;const char *line=s;if(strip[i])while(*line=='\t')line++;
 if((size_t)(p-line)==strlen(delim[i])&&!memcmp(line,delim[i],p-line)){if(*p)p++;found=1;break;}putn(&b,line,p-line);if(*p){putsb(&b,"\n");p++;}}
 if(!found)die("unterminated heredoc %s",delim[i]);pending[i]->doc=done(&b);free(delim[i]);}np=0;continue;}
 s=p;const char *digits=p;while(isdigit((unsigned char)*digits))digits++;
 if(*digits=='<'||*digits=='>')p=digits;
 if(strchr(";|&()<>",*p)){char op=*p++;if((op=='|'&&*p=='|')||(op=='&'&&*p=='&')||(op==';'&&*p==';')||(op=='>'&&*p=='>')||(op=='<'&&*p=='<')||((op=='<'||op=='>')&&*p=='&')||(op=='<'&&*p=='>')||(op=='>'&&*p=='|'))p++;
 if(op=='<'&&p[-1]=='<'&&*p=='-')p++;x=token(&t,s,p-s);
 if(op=='<'&&strstr(x->s,"<<")){while(*p==' '||*p=='\t')p++;s=p;p=wordend(p);if(p==s)die("missing heredoc delimiter");Tok *d=token(&t,s,p-s);
 pending=realloc(pending,(np+1)*sizeof *pending);delim=realloc(delim,(np+1)*sizeof *delim);strip=realloc(strip,(np+1)*sizeof *strip);if(!pending||!delim||!strip)die("heredoc allocation");pending[np]=x;strip[np]=strstr(x->s,"<<-")!=NULL;delim[np++]=unquote_delimiter(d->s,&x->quoted);}continue;}
 p=wordend(p);if(p==s)die("shell lexer stalled");token(&t,s,p-s);
 }
 if(np)die("missing heredoc newline");token(&t,p,0);free(pending);free(delim);free(strip);return t;
}

enum {CMD=1,SEQ,AND,OR,PIPE,IF,CASE,FOR,FUNC,GROUP,WHILE,UNTIL,NOT};
typedef struct Redir Redir;struct Redir{char *op,*word,*doc;int quoted,fd;Redir *next;};
struct Node{int kind,n,line;size_t offset;char *source;char **words;Node *a,*b,*c,*next;Redir *redirs;};
static char *parsing_source;
typedef struct Frame Frame;struct Frame{Node *node;Env *env;Frame *previous;};
static Frame *frames;
static Node *node(int k){Node *n=alloc(sizeof *n);n->kind=k;n->source=parsing_source;return n;}
static char *peek(Tokens *t){return t->v[t->pos]->s;}
static int is(Tokens *t,const char *s){return !strcmp(peek(t),s);}
static int eat(Tokens *t,const char *s){if(is(t,s)){t->pos++;return 1;}return 0;}
static void expect(Tokens *t,const char *s){if(!eat(t,s))die("expected %s, found %s",s,peek(t));}
static void semis(Tokens *t){while(eat(t,";"));}
static void addword(Node *n,char *s){n->words=realloc(n->words,(n->n+1)*sizeof *n->words);if(!n->words)die("word allocation");n->words[n->n++]=s;}
static int stop(Tokens *t){char *s=peek(t);return !*s||!strcmp(s,"then")||!strcmp(s,"else")||!strcmp(s,"elif")||!strcmp(s,"fi")||!strcmp(s,"do")||!strcmp(s,"done")||!strcmp(s,"esac")||!strcmp(s,";;")||!strcmp(s,"}")||!strcmp(s,")");}
static int isredir(const char *p){while(isdigit((unsigned char)*p))p++;return *p=='>'||*p=='<';}
static Node *list(Tokens *t);
static void redirects(Tokens *t,Node *n){Redir **tail=&n->redirs;while(*tail)tail=&(*tail)->next;while(isredir(peek(t))){Tok *x=t->v[t->pos++];Redir *r=alloc(sizeof *r);char *p=x->s;int explicit_fd=isdigit((unsigned char)*p);r->fd=explicit_fd?strtol(p,&p,10):(*p=='<'?0:1);if(r->fd<0||r->fd>=64)die("descriptor out of range");r->op=p;r->doc=x->doc;r->quoted=x->quoted;if(!*peek(t))die("missing redirect operand");r->word=peek(t);t->pos++;*tail=r;tail=&r->next;}}
static Node *ifnode(Tokens *t){Node *n=node(IF);n->a=list(t);expect(t,"then");n->b=list(t);if(eat(t,"elif"))n->c=ifnode(t);else{if(eat(t,"else"))n->c=list(t);expect(t,"fi");}return n;}
static Node *command(Tokens *t){Node *n;semis(t);size_t offset=t->v[t->pos]->offset;int line=t->v[t->pos]->line;
 if(eat(t,"if"))n=ifnode(t);
 else if(is(t,"while")||is(t,"until")){int k=eat(t,"while")?WHILE:(expect(t,"until"),UNTIL);n=node(k);n->a=list(t);expect(t,"do");n->b=list(t);expect(t,"done");}
 else if(eat(t,"case")){n=node(CASE);addword(n,peek(t));t->pos++;semis(t);expect(t,"in");semis(t);Node **tail=&n->a;while(!is(t,"esac")){if(!*peek(t))die("unterminated case");Node *arm=node(CMD);eat(t,"(");do{addword(arm,peek(t));t->pos++;}while(eat(t,"|"));expect(t,")");arm->a=list(t);*tail=arm;tail=&arm->next;if(!eat(t,";;")&&!is(t,"esac"))die("case arm delimiter %s",peek(t));semis(t);}expect(t,"esac");}
 else if(eat(t,"for")){n=node(FOR);addword(n,peek(t));t->pos++;if(eat(t,"in")){n->c=node(CMD);while(!is(t,";")&&!is(t,"do")){if(!*peek(t))die("unterminated for");addword(n,peek(t));t->pos++;}}semis(t);expect(t,"do");n->a=list(t);expect(t,"done");}
 else if(eat(t,"(")){n=node(GROUP);n->n=1;n->a=list(t);expect(t,")");}
 else if(eat(t,"{")){n=node(GROUP);n->a=list(t);expect(t,"}");}
 else if(t->pos+2<t->n&&*peek(t)&&!strcmp(t->v[t->pos+1]->s,"(")&&!strcmp(t->v[t->pos+2]->s,")")){n=node(FUNC);addword(n,peek(t));t->pos+=3;semis(t);n->a=command(t);}
 else{n=node(CMD);while(*peek(t)&&!strchr(";|&)",*peek(t))&&!stop(t)){if(isredir(peek(t)))redirects(t,n);else{addword(n,peek(t));t->pos++;}}if(!n->n&&!n->redirs)die("empty shell command before %s",peek(t));}
 n->offset=offset;n->line=line;redirects(t,n);return n;
}
static Node *pipeline(Tokens *t){int negate=eat(t,"!");Node *n=command(t);while(eat(t,"|")){semis(t);Node *r=node(PIPE);r->a=n;r->b=command(t);n=r;}if(negate){Node *r=node(NOT);r->a=n;n=r;}return n;}
static Node *andor(Tokens *t){Node *n=pipeline(t);while(is(t,"&&")||is(t,"||")){int k=eat(t,"&&")?AND:(expect(t,"||"),OR);semis(t);Node *r=node(k);r->a=n;r->b=pipeline(t);n=r;}return n;}
static Node *list(Tokens *t){Node *n=node(SEQ),**tail=&n->a;semis(t);while(!stop(t)){*tail=andor(t);tail=&(*tail)->next;if(!eat(t,";")){if(is(t,"&"))die("asynchronous command requires a separate source phase");if(!stop(t))die("missing separator before %s",peek(t));}semis(t);}return n;}
static Node *parse(const char *s){Tokens t=lex(s);Node *n=list(&t);if(*peek(&t))die("unexpected shell token %s",peek(&t));return n;}
static int run(Env *e,Node *n);
static int script(Env *e,const char *s){return run(e,parse(s));}
typedef struct {Buf text,mask;int forced;} Expanded;
static void emit(Expanded *x,const char *s,int protected){size_t n=strlen(s);putn(&x->text,s,n);for(size_t i=0;i<n;i++){char c=protected;putn(&x->mask,&c,1);}}
static Expanded expand(Env *e,const char *s,int mode);
static char *scalar(Env *e,const char *s);
static char *capture(Env *e,const char *s){Env child=clone(e);child.out=tmpfile();if(!child.out)die("capture stream");child.fd[1]=child.out;child.fd_flags[1]=O_WRONLY;child.fd_cloexec[1]=0;script(&child,s);if(fflush(child.out)||fseek(child.out,0,SEEK_SET))die("capture rewind");Buf b={0};char block[8192];size_t n;while((n=fread(block,1,sizeof block,child.out)))putn(&b,block,n);if(ferror(child.out)||fclose(child.out))die("capture read");e->status=child.status;e->substituted=1;clearvars(&child);if(chdir(e->cwd))die("restore command substitution directory");while(b.n&&b.s[b.n-1]=='\n')b.s[--b.n]=0;return done(&b);}
static char *backtick_capture(Env *e,const char *s){Buf b={0};for(const char *p=s;*p;p++){if(*p=='\\'&&p[1]&&strchr("$`\\",p[1]))p++;putn(&b,p,1);}char *text=done(&b),*r=capture(e,text);free(text);return r;}

/* Arithmetic is parsed independently of shell tokenization. The evaluation
   flag preserves short circuit and ternary side-effect semantics. */
typedef struct {const char *p;Env *e;int recursion;} Arithmetic;
static void ar_space(Arithmetic *a){while(isspace((unsigned char)*a->p))a->p++;}
static int ar_take(Arithmetic *a,const char *s){ar_space(a);size_t n=strlen(s);if(strncmp(a->p,s,n))return 0;a->p+=n;return 1;}
static long long ar_expr(Arithmetic *a,int minimum,int evaluate);
static long long arithmetic(Env *e,const char *s,int recursion){if(recursion>64)die("recursive arithmetic variable");Arithmetic a={s,e,recursion};long long v=ar_expr(&a,1,1);ar_space(&a);if(*a.p)die("invalid arithmetic: %s",a.p);return v;}
static long long ar_primary(Arithmetic *a,int evaluate){ar_space(a);int negative=0,logical=0,bitwise=0;const char *p=a->p;
 if(*p=='+'||*p=='-'||*p=='!'||*p=='~'){a->p++;long long v=ar_primary(a,evaluate);if(!evaluate)return 0;return *p=='-'?-v:*p=='!'?!v:*p=='~'?~v:v;}
 if(ar_take(a,"(")){long long v=ar_expr(a,1,evaluate);if(!ar_take(a,")"))die("arithmetic missing )");return v;}
 if(isalpha((unsigned char)*a->p)||*a->p=='_'){p=a->p++;while(isalnum((unsigned char)*a->p)||*a->p=='_')a->p++;char *key=slice(p,a->p-p);const char *s=get(a->e,key);long long v=evaluate&&s&&*s?arithmetic(a->e,s,a->recursion+1):0;free(key);return v;}
 if(!isdigit((unsigned char)*a->p))die("invalid arithmetic operand %s",a->p);char *end;errno=0;unsigned long long v=strtoull(a->p,&end,0);if(end==a->p||errno)die("arithmetic integer range");a->p=end;return evaluate?(long long)v:0;
}
static int ar_operator(Arithmetic *a,const char **op,int *length){static const char *ops[]={",","=","+=","-=","*=","/=","%=","<<=",">>=","&=","^=","|=","?","||","&&","|","^","&","==","!=","<=",">=","<",">","<<",">>","+","-","*","/","%",NULL};static const int precedence[]={1,2,2,2,2,2,2,2,2,2,2,2,3,4,5,6,7,8,9,9,10,10,10,10,11,11,12,12,13,13,13};ar_space(a);int best=-1;size_t n=0;for(int i=0;ops[i];i++){size_t size=strlen(ops[i]);if(size>n&&!strncmp(a->p,ops[i],size)){best=i;n=size;}}if(best<0)return 0;*op=ops[best];*length=n;return precedence[best];}
static long long ar_expr(Arithmetic *a,int minimum,int evaluate){ar_space(a);const char *left=a->p;long long v=ar_primary(a,evaluate);for(;;){const char *op;int n;int precedence=ar_operator(a,&op,&n);if(precedence<minimum)break;const char *operator_start=a->p;a->p+=n;
 if(!strcmp(op,"?")){long long yes=ar_expr(a,1,evaluate&&v);if(!ar_take(a,":"))die("arithmetic ternary missing colon");long long no=ar_expr(a,3,evaluate&&!v);if(evaluate)v=v?yes:no;continue;}
 int assignment=precedence==2;int rhs_evaluate=evaluate&&(!strcmp(op,"&&")?v!=0:!strcmp(op,"||")?v==0:1);long long r=ar_expr(a,assignment?precedence:precedence+1,rhs_evaluate);if(!evaluate)continue;
 if(!strcmp(op,","))v=r;else if(!strcmp(op,"||"))v=v||r;else if(!strcmp(op,"&&"))v=v&&r;
 else{const char *binary=op;char single[4];if(assignment){size_t k=n-1;if(k){memcpy(single,op,k);single[k]=0;binary=single;}else binary="=";}
 if(!strcmp(binary,"="))v=r;else if(!strcmp(binary,"+"))v=(long long)((unsigned long long)v+(unsigned long long)r);else if(!strcmp(binary,"-"))v=(long long)((unsigned long long)v-(unsigned long long)r);else if(!strcmp(binary,"*"))v=(long long)((unsigned long long)v*(unsigned long long)r);else if(!strcmp(binary,"/")||!strcmp(binary,"%")){if(!r)die("arithmetic division by zero");if(v==LLONG_MIN&&r==-1)v=!strcmp(binary,"/")?LLONG_MIN:0;else v=!strcmp(binary,"/")?v/r:v%r;}
 else if(!strcmp(binary,"<<")||!strcmp(binary,">>")){if(r<0||r>=64)die("arithmetic shift out of range");v=!strcmp(binary,"<<")?(long long)((unsigned long long)v<<r):v>>r;}
 else if(!strcmp(binary,"|"))v|=r;else if(!strcmp(binary,"&"))v&=r;else if(!strcmp(binary,"^"))v^=r;else if(!strcmp(binary,"=="))v=v==r;else if(!strcmp(binary,"!="))v=v!=r;else if(!strcmp(binary,"<"))v=v<r;else if(!strcmp(binary,">"))v=v>r;else if(!strcmp(binary,"<="))v=v<=r;else if(!strcmp(binary,">="))v=v>=r;else die("invalid arithmetic operator");
 if(assignment){const char *end=left;while(isalnum((unsigned char)*end)||*end=='_')end++;const char *space=end;while(isspace((unsigned char)*space))space++;if(end==left||space!=operator_start||(!isalpha((unsigned char)*left)&&*left!='_'))die("arithmetic assignment needs a variable");char *key=slice(left,end-left),value[64];snprintf(value,sizeof value,"%lld",v);set(a->e,key,value);free(key);}}
 }return v;}
static char *parameter(Env *e,const char *s,int mode){const char *p=s;char number[64];const char *v;int colon=0,length=0;
 if(*p=='#'&&p[1]){length=1;p++;}const char *start=p;if(isdigit((unsigned char)*p)){while(isdigit((unsigned char)*p))p++;}else{while(isalnum((unsigned char)*p)||*p=='_')p++;if(p==start&&strchr("@*#$?-!",*p)&&*p)p++;}
 char *key=slice(start,p-start);if(!strcmp(key,"#")){snprintf(number,sizeof number,"%d",e->argc);v=number;}else if(!strcmp(key,"?")){snprintf(number,sizeof number,"%d",e->status);v=number;}else if(!strcmp(key,"$"))v="1";else if(!strcmp(key,"0"))v=e->zero;
 else if(!strcmp(key,"-")){int n=0;if(e->errexit)number[n++]='e';if(e->nounset)number[n++]='u';if(e->noglob)number[n++]='f';if(e->allexport)number[n++]='a';if(e->noclobber)number[n++]='C';number[n]=0;v=number;}
 else if(!strcmp(key,"!"))v="";
 else if(!strcmp(key,"@")||!strcmp(key,"*")){Buf b={0};const char *ifs=get(e,"IFS");char separator=ifs?*ifs:' ';for(int i=0;i<e->argc;i++){if(i&&separator)putn(&b,&separator,1);putsb(&b,e->args[i]);}v=done(&b);}
 else if(!strcmp(key,"LINENO")){snprintf(number,sizeof number,"%d",e->line);v=number;}else if(isdigit((unsigned char)*key)){int i=atoi(key);v=i>0&&i<=e->argc?e->args[i-1]:NULL;}else v=get(e,key);
 if(length){snprintf(number,sizeof number,"%lu",(unsigned long)(v?strlen(v):0));free(key);return dup(number);}
 if(!*p){if(!v&&e->nounset)die("%s: unset parameter",key);free(key);return dup(v?v:"");}
 if(*p==':'){colon=1;p++;}char op=*p++;int present=v&&(!colon||*v);
 if(op=='-'||op=='+'||op=='='||op=='?'){int use=op=='+'?present:!present;if(use){Expanded a=expand(e,p,mode);char *r=done(&a.text);free(a.mask.s);if(op=='?')die("%s: %s",key,r);if(op=='=')set(e,key,r);free(key);return r;}free(key);return dup(op=='+'?"":v?v:"");}
 if(!colon&&(op=='%'||op=='#')){int longest=*p==op;if(longest)p++;Expanded x=expand(e,p,mode);Buf pat={0};for(size_t i=0;i<x.text.n;i++){if(x.mask.s[i]==1&&strchr("*?[\\",x.text.s[i]))putsb(&pat,"\\");putn(&pat,x.text.s+i,1);}char *pattern=done(&pat);const char *value=v?v:"";size_t n=strlen(value);char *result=NULL;for(size_t k=0;k<=n;k++){size_t cut=longest?n-k:k;char *part=op=='#'?slice(value,cut):dup(value+n-cut);int matched=fnmatch(pattern,part,0)==0;free(part);if(matched){result=op=='#'?dup(value+cut):slice(value,n-cut);break;}}free(pattern);free(x.text.s);free(x.mask.s);free(key);return result?result:dup(value);}
 die("invalid parameter expansion %s",s);return NULL;
}
static void positional(Expanded *x,Env *e){for(int i=0;i<e->argc;i++){if(i){char boundary=0,mask=2;putn(&x->text,&boundary,1);putn(&x->mask,&mask,1);}emit(x,e->args[i],1);}if(e->argc)x->forced=1;}
static void parameter_emit(Expanded *x,Env *e,const char *raw,int mode,int quoted){
 const char *p=raw;while(isalnum((unsigned char)*p)||*p=='_')p++;if(p==raw&&strchr("@*#$?-!",*p)&&*p)p++;
 if(quoted&&!mode&&!strcmp(raw,"@")){positional(x,e);return;}
 int colon=*p==':';const char *op=p+colon;
 if(*op&&strchr("-+=?",*op)){char *key=slice(raw,p-raw);const char *value=get(e,key);if(isdigit((unsigned char)*key)){int n=atoi(key);value=n>0&&n<=e->argc?e->args[n-1]:n==0?e->zero:NULL;}else if(strchr("@*#$?-!",*key)&&key[1]==0)value="set";int present=value&&(!colon||*value);int use=*op=='+'?present:!present;
 if(use){Expanded part=expand(e,op+1,mode);if(*op=='?'||*op=='='){char *v=done(&part.text);if(*op=='?')die("%s: %s",key,v);set(e,key,v);}putn(&x->text,part.text.s?part.text.s:"",part.text.n);for(size_t i=0;i<part.mask.n;i++){char m=part.mask.s[i];if((quoted||mode)&&m!=2)m=1;putn(&x->mask,&m,1);}x->forced|=part.forced;free(part.text.s);free(part.mask.s);free(key);return;}
 free(key);}
 char *value=parameter(e,raw,mode);emit(x,value,quoted||mode);free(value);
}
static Expanded expand(Env *e,const char *s,int mode){Expanded x={0};int q=0;const char *p=s;
 if(!mode&&*p=='~'&&(p[1]=='/'||!p[1])){const char *home=get(e,"HOME");if(home){emit(&x,home,1);p++;}}
 while(*p){char c=*p;if(!mode&&(c=='\''||c=='"')){if(!q){q=c;x.forced=1;p++;continue;}if(q==c){q=0;p++;continue;}}
 if(c=='\\'&&q!='\''&&p[1]){if(p[1]=='\n'){p+=2;continue;}if(q=='"'||mode){const char *special=mode?"$`\\":"$`\\\"";if(!strchr(special,p[1])){char v[2]={c,0};emit(&x,v,1);p++;continue;}}char v[2]={p[1],0};emit(&x,v,1);p+=2;continue;}
 if(q!='\''&&c=='$'){
 if(p[1]=='{'){const char *end=balanced(p+2,'}');char *inner=slice(p+2,end-p-3);parameter_emit(&x,e,inner,mode,q!=0);free(inner);p=end;continue;}
 if(p[1]=='('){const char *end=balanced(p+2,')');char *v;if(p[2]=='('){if(end-p<5||end[-2]!=')')die("arithmetic closing delimiter");char *inner=slice(p+3,end-p-5);long long n=arithmetic(e,inner,0);char number[64];snprintf(number,sizeof number,"%lld",n);v=dup(number);free(inner);}else{char *inner=slice(p+2,end-p-3);v=capture(e,inner);free(inner);}emit(&x,v,q||mode);free(v);p=end;continue;}
 const char *end=p+1;if(strchr("@*#$?-!",*end)&&*end)end++;else if(isdigit((unsigned char)*end))end++;else while(isalnum((unsigned char)*end)||*end=='_')end++;
 if(end>p+1){char *key=slice(p+1,end-p-1);parameter_emit(&x,e,key,mode,q!=0);free(key);p=end;continue;}}
 if(q!='\''&&c=='`'){const char *end=balanced(p+1,'`');char *cmd=slice(p+1,end-p-2);char *v=backtick_capture(e,cmd);emit(&x,v,q||mode);free(cmd);free(v);p=end;continue;}
 char v[2]={c,0};emit(&x,v,q||mode?1:3);p++;}return x;
}
static char *scalar(Env *e,const char *s){Expanded x=expand(e,s,0);free(x.mask.s);return done(&x.text);}
static char *pattern(Env *e,const char *s){Expanded x=expand(e,s,0);Buf b={0};for(size_t i=0;i<x.text.n;i++){if(x.mask.s[i]==1&&strchr("*?[\\",x.text.s[i]))putsb(&b,"\\");putn(&b,x.text.s+i,1);}free(x.text.s);free(x.mask.s);return done(&b);}
typedef struct {char **v;int n;const char *argv0;} Args;
static void arg(Args *a,char *s){a->v=realloc(a->v,(a->n+1)*sizeof *a->v);if(!a->v)die("argument allocation");a->v[a->n++]=s;}
static void field(Env *e,Args *a,Expanded *x,size_t start,size_t n){Buf pat={0};int wildcard=0;for(size_t i=start;i<start+n;i++){if(x->mask.s[i]==1&&strchr("*?[\\",x->text.s[i]))putsb(&pat,"\\");if(x->mask.s[i]!=1&&strchr("*?[",x->text.s[i]))wildcard=1;putn(&pat,x->text.s+i,1);}char *pattern=done(&pat);if(wildcard&&!e->noglob){glob_t paths;memset(&paths,0,sizeof paths);int rc=glob(pattern,0,NULL,&paths);if(!rc){for(size_t i=0;i<paths.gl_pathc;i++)arg(a,dup(paths.gl_pathv[i]));globfree(&paths);free(pattern);return;}globfree(&paths);if(rc!=GLOB_NOMATCH)die("pathname expansion failed");}free(pattern);arg(a,slice(x->text.s+start,n));}
static void fields(Env *e,Args *a,const char *raw){if(!strcmp(raw,"\"$@\"")||!strcmp(raw,"\"${@}\"")){for(int i=0;i<e->argc;i++)arg(a,dup(e->args[i]));return;}
 Expanded x=expand(e,raw,0);const char *ifs=get(e,"IFS");if(!ifs)ifs=" \t\n";size_t i=0;int before=a->n,after_boundary=0;
 while(i<x.text.n){while(i<x.text.n&&!x.mask.s[i]&&strchr(ifs,x.text.s[i])&&strchr(" \t\n",x.text.s[i]))i++;size_t start=i;
 while(i<x.text.n&&x.mask.s[i]!=2&&(x.mask.s[i]||!strchr(ifs,x.text.s[i])))i++;
 if(i>start){field(e,a,&x,start,i-start);after_boundary=0;}
 if(i<x.text.n){if(x.mask.s[i]==2){if(i==start)arg(a,dup(""));i++;after_boundary=1;continue;}int nonwhite=!strchr(" \t\n",x.text.s[i]);if(nonwhite&&i==start)arg(a,dup(""));i++;if(!nonwhite){while(i<x.text.n&&!x.mask.s[i]&&strchr(ifs,x.text.s[i])&&strchr(" \t\n",x.text.s[i]))i++;if(i<x.text.n&&!x.mask.s[i]&&strchr(ifs,x.text.s[i])&&!strchr(" \t\n",x.text.s[i]))i++;}}
 }
 if(after_boundary||(before==a->n&&x.forced))arg(a,dup(""));free(x.text.s);free(x.mask.s);
}
static int assignment(const char *s){const char *p=s;if(!isalpha((unsigned char)*p)&&*p!='_')return 0;while(isalnum((unsigned char)*p)||*p=='_')p++;return *p=='=';}

"####;

const EVALUATOR: &str = r####"
static int external(Env *e,Args *a);
static int mantle_shell_external(Env *e,Args *a);
static int (*mantle_source_external_hook)(Env *e,Args *a);
static int builtin(Env *e,Args *a){const char *name=a->v[0];
 if(!strcmp(name,":"))return 0;
 if(!strcmp(name,"true"))return 0;if(!strcmp(name,"false"))return 1;
 if(!strcmp(name,"test")||!strcmp(name,"["))return test_command(a);
 if(!strcmp(name,"unset")){int i=1,functions=0;if(i<a->n&&!strcmp(a->v[i],"-f")){functions=1;i++;}else if(i<a->n&&!strcmp(a->v[i],"-v"))i++;for(;i<a->n;i++){if(functions){Fun **p=&e->funs;while(*p){if(!strcmp((*p)->name,a->v[i])){*p=(*p)->next;break;}p=&(*p)->next;}}else set(e,a->v[i],NULL);}return 0;}
 if(!strcmp(name,"export")||!strcmp(name,"readonly")){for(int i=1;i<a->n;i++){char *key=dup(a->v[i]),*eq=strchr(key,'=');if(eq){*eq++=0;set(e,key,eq);}Var *v=variable(e,key);if(!v){set(e,key,NULL);v=variable(e,key);}if(!strcmp(name,"export"))v->exported=1;else v->readonly=1;free(key);}return 0;}
 if(!strcmp(name,"set")){int i=1;if(i==a->n){for(Var *v=e->vars;v;v=v->next)if(v->value)fprintf(e->out,"%s='%s'\n",v->key,v->value);return 0;}for(;i<a->n;i++){if(!strcmp(a->v[i],"--")){i++;break;}if(!strcmp(a->v[i],"-")){i++;break;}if(a->v[i][0]!='-'&&a->v[i][0]!='+')break;int enabled=a->v[i][0]=='-';for(char *p=a->v[i]+1;*p;p++){if(*p=='e')e->errexit=enabled;else if(*p=='u')e->nounset=enabled;else if(*p=='f')e->noglob=enabled;else if(*p=='a')e->allexport=enabled;else if(*p=='C')e->noclobber=enabled;else if(*p=='x'||*p=='v'){}else if(*p=='o'){if(++i==a->n)die("set -o requires an option");const char *option=a->v[i];if(!strcmp(option,"posix")){}else if(!strcmp(option,"errexit"))e->errexit=enabled;else if(!strcmp(option,"nounset"))e->nounset=enabled;else if(!strcmp(option,"noglob"))e->noglob=enabled;else die("unknown shell option %s",option);break;}else die("unknown shell option %c",*p);}}if(i<a->n||!strcmp(a->v[a->n-1],"--")){e->argc=a->n-i;e->args=alloc((e->argc+1)*sizeof *e->args);for(int j=0;j<e->argc;j++)e->args[j]=dup(a->v[i+j]);}return 0;}
 if(!strcmp(name,"shift")){int n=a->n>1?atoi(a->v[1]):1;if(n<0||n>e->argc)return 1;e->args+=n;e->argc-=n;return 0;}
 if(!strcmp(name,".")){if(a->n<2)return 2;char *path=a->v[1];char **old=e->args;int count=e->argc;if(a->n>2){e->args=a->v+2;e->argc=a->n-2;}int r=run(e,source_program(path));e->args=old;e->argc=count;if(e->flow==1)e->flow=0;return r;}
 if(!strcmp(name,"eval")){Buf b={0};for(int i=1;i<a->n;i++){if(i>1)putsb(&b," ");putsb(&b,a->v[i]);}char *s=done(&b);int r=script(e,s);free(s);return r;}
 if(!strcmp(name,"return")||!strcmp(name,"exit")){e->flow=!strcmp(name,"return")?1:4;return a->n>1?(int)(strtol(a->v[1],NULL,10)&255):e->status;}
 if(!strcmp(name,"break")||!strcmp(name,"continue")){e->flow=!strcmp(name,"break")?2:3;e->depth=a->n>1?atoi(a->v[1]):1;if(e->depth<1)die("invalid loop depth");return 0;}
 if(!strcmp(name,"cd")){const char *path=a->n>1?a->v[1]:get(e,"HOME");if(!path)return 1;if(!strcmp(path,"-"))path=get(e,"OLDPWD");if(!path||chdir(path))return 1;char *cwd=getcwd(NULL,0);if(!cwd)die("getcwd");set(e,"OLDPWD",e->cwd);set(e,"PWD",cwd);e->cwd=cwd;return 0;}
 if(!strcmp(name,"pwd")){fprintf(e->out,"%s\n",e->cwd);return 0;}
 if(!strcmp(name,"trap")){if(a->n<2)return 0;for(int i=2;i<a->n;i++){int signal;if(!strcmp(a->v[i],"EXIT"))signal=0;else if(!strcmp(a->v[i],"HUP"))signal=1;else if(!strcmp(a->v[i],"INT"))signal=2;else if(!strcmp(a->v[i],"PIPE"))signal=13;else if(!strcmp(a->v[i],"TERM"))signal=15;else signal=atoi(a->v[i]);if(signal<0||signal>=32)return 1;e->traps[signal]=!strcmp(a->v[1],"-")?NULL:dup(a->v[1]);}return 0;}
 if(!strcmp(name,"umask")){if(a->n==1){fprintf(e->out,"%04o\n",(unsigned)e->creation_mask);return 0;}char *end;long value=strtol(a->v[1],&end,8);if(*end||value<0||value>0777)return 1;e->creation_mask=value;e->mask_valid=1;umask(e->creation_mask);return 0;}
 if(!strcmp(name,"read")){int raw=0,start=1;if(start<a->n&&!strcmp(a->v[start],"-r")){raw=1;start++;}Buf b={0};int c,found=0;while((c=fgetc(e->in))!=EOF){found=1;if(c=='\n')break;if(!raw&&c=='\\'){c=fgetc(e->in);if(c==EOF)break;if(c=='\n')continue;}char byte=c;putn(&b,&byte,1);}char *line=done(&b);const char *ifs=get(e,"IFS");if(!ifs)ifs=" \t\n";char *p=line;while(*p&&strchr(ifs,*p)&&strchr(" \t\n",*p))p++;if(start==a->n)set(e,"REPLY",line);else for(int i=start;i<a->n;i++){char *s=p;if(i+1<a->n){while(*p&&!strchr(ifs,*p))p++;char *end=p;while(*p&&strchr(ifs,*p))p++;char *value=slice(s,end-s);set(e,a->v[i],value);free(value);}else{char *end=p+strlen(p);while(end>p&&strchr(ifs,end[-1])&&strchr(" \t\n",end[-1]))end--;char *value=slice(s,end-s);set(e,a->v[i],value);free(value);}}free(line);return c==EOF?1:0;}
 if(!strcmp(name,"exec")){if(a->n==1)return 0;Args command={a->v+1,a->n-1};int r=external(e,&command);e->flow=4;return r;}
 if(!strcmp(name,"command")){int i=1;if(i<a->n&&!strcmp(a->v[i],"--"))i++;if(i==a->n)return 0;if(a->v[i][0]=='-')return external(e,a);Args command={a->v+i,a->n-i};return external(e,&command);}
 return external(e,a);
}
static int special(const char *name){return !strcmp(name,":")||!strcmp(name,".")||!strcmp(name,"eval")||!strcmp(name,"exec")||!strcmp(name,"exit")||!strcmp(name,"export")||!strcmp(name,"readonly")||!strcmp(name,"return")||!strcmp(name,"set")||!strcmp(name,"shift")||!strcmp(name,"trap")||!strcmp(name,"unset")||!strcmp(name,"break")||!strcmp(name,"continue");}
static int simple(Env *e,Node *n){Args a={0};int assignments=0;e->substituted=0;while(assignments<n->n&&assignment(n->words[assignments]))assignments++;for(int j=assignments;j<n->n;j++)fields(e,&a,n->words[j]);int temporary=a.n&&assignments&&!special(a.v[0]);Env local;Env *target=e;if(temporary){local=clone(e);target=&local;}for(int j=0;j<assignments;j++){char *raw=n->words[j],*eq=strchr(raw,'=');char *key=slice(raw,eq-raw),*value=scalar(target,eq+1);set(target,key,value);if(a.n)variable(target,key)->exported=1;free(key);free(value);}if(!a.n){free(a.v);return e->substituted?e->status:0;}
 int result;Fun *f;for(f=target->funs;f;f=f->next)if(!strcmp(f->name,a.v[0]))break;
 if(f){char **oldargs=target->args;int oldargc=target->argc;target->args=a.v+1;target->argc=a.n-1;result=run(target,f->body);target->args=oldargs;target->argc=oldargc;if(target->flow==1)target->flow=0;}
 else result=builtin(target,&a);
 if(temporary){e->flow=target->flow;e->depth=target->depth;clearvars(target);if(chdir(e->cwd))die("restore assignment environment");}
 for(int i=0;i<a.n;i++)free(a.v[i]);free(a.v);return result;
}
static int conditional(Env *e,Node *n){e->suppress++;int r=run(e,n);e->suppress--;return r;}
static int loop_control(Env *e){if(e->flow!=2&&e->flow!=3)return e->flow?1:0;if(--e->depth){return 1;}int was=e->flow;e->flow=0;return was==2?1:2;}
static int execute(Env *e,Node *n){int result=0;switch(n->kind){
 case CMD:return simple(e,n);
 case SEQ:for(Node *p=n->a;p&&!e->flow;p=p->next)result=run(e,p);return result;
 case AND:result=conditional(e,n->a);return result||e->flow?result:run(e,n->b);
 case OR:result=conditional(e,n->a);return !result||e->flow?result:run(e,n->b);
 case NOT:return !conditional(e,n->a);
 case IF:result=conditional(e,n->a);if(e->flow)return result;return !result?run(e,n->b):n->c?run(e,n->c):0;
 case FUNC:{Fun *f=alloc(sizeof *f);f->name=n->words[0];f->body=n->a;f->next=e->funs;e->funs=f;return 0;}
 case GROUP:if(n->n){Env child=clone(e);result=run(&child,n->a);clearvars(&child);if(chdir(e->cwd))die("restore subshell directory");return result;}return run(e,n->a);
 case PIPE:{FILE *stream=tmpfile();if(!stream)die("pipeline stream");Env left=clone(e),right=clone(e);left.out=left.fd[1]=stream;left.fd_flags[1]=O_WRONLY;left.fd_cloexec[1]=0;conditional(&left,n->a);if(fflush(stream)||fseek(stream,0,SEEK_SET))die("pipeline rewind");if(chdir(e->cwd))die("restore pipeline directory");right.in=right.fd[0]=stream;right.fd_flags[0]=O_RDONLY;right.fd_cloexec[0]=0;result=run(&right,n->b);clearvars(&left);clearvars(&right);fclose(stream);if(chdir(e->cwd))die("restore pipeline directory");return result;}
 case CASE:{char *value=scalar(e,n->words[0]);for(Node *arm=n->a;arm;arm=arm->next)for(int i=0;i<arm->n;i++){char *p=pattern(e,arm->words[i]);int match=fnmatch(p,value,0)==0;free(p);if(match){free(value);return run(e,arm->a);}}free(value);return 0;}
 case FOR:{Args a={0};if(!n->c){for(int i=0;i<e->argc;i++)arg(&a,dup(e->args[i]));}else for(int i=1;i<n->n;i++)fields(e,&a,n->words[i]);for(int i=0;i<a.n;i++){set(e,n->words[0],a.v[i]);result=run(e,n->a);int control=loop_control(e);if(control==1)break;}for(int i=0;i<a.n;i++)free(a.v[i]);free(a.v);return result;}
 case WHILE:case UNTIL:for(;;){int condition=conditional(e,n->a);if(e->flow||((condition==0)!=(n->kind==WHILE)))break;result=run(e,n->b);int control=loop_control(e);if(control==1)break;}return result;
 default:die("invalid shell node");}return 1;
}
static int run(Env *e,Node *n){if(!n)return 0;mantle_source_ensure_mask(e);mode_t previous_mask=umask(e->creation_mask);Frame frame={n,e,frames};frames=&frame;if(n->line)e->line=n->line;FILE *saved[64];int saved_flags[64],saved_cloexec[64];memcpy(saved,e->fd,sizeof saved);memcpy(saved_flags,e->fd_flags,sizeof saved_flags);memcpy(saved_cloexec,e->fd_cloexec,sizeof saved_cloexec);FILE **opened=NULL;int count=0,redirect_error=0,persistent=n->kind==CMD&&n->n&&!strcmp(n->words[0],"exec");
 for(Redir *r=n->redirs;r;r=r->next){char *path=scalar(e,r->word);FILE *f=NULL;const char *op=r->op;
 if(!strcmp(op,"<&")||!strcmp(op,">&")){if(!strcmp(path,"-")){e->fd[r->fd]=NULL;e->fd_flags[r->fd]=0;}else{char *end;long fd=strtol(path,&end,10);if(*end||fd<0||fd>=64||!e->fd[fd]){redirect_error=1;free(path);break;}e->fd[r->fd]=e->fd[fd];e->fd_flags[r->fd]=e->fd_flags[fd];}e->fd_cloexec[r->fd]=0;free(path);continue;}
 if(!strncmp(op,"<<",2)){char *body;if(r->quoted)body=dup(r->doc);else{Expanded x=expand(e,r->doc,1);body=done(&x.text);free(x.mask.s);}f=tmpfile();if(!f)die("heredoc stream");if(fwrite(body,1,strlen(body),f)!=strlen(body))die("heredoc write");rewind(f);free(body);}
 else if(!strcmp(op,"<"))f=fopen(path,"rb");else if(!strcmp(op,"<>")){f=fopen(path,"r+b");if(!f&&errno==ENOENT)f=fopen(path,"w+b");}
 else if(!strcmp(op,">>")){int fd=open(path,O_RDWR|O_CREAT|O_APPEND,0666);if(fd>=0){f=fdopen(fd,"r+b");if(!f){int error=errno;close(fd);errno=error;}}}else{if(e->noclobber&&!strcmp(op,">")&&!access(path,F_OK)){errno=EEXIST;f=NULL;}else f=fopen(path,"w+b");}
 if(!f){fprintf(e->err?e->err:stderr,"%s: %s\n",path,strerror(errno));redirect_error=1;free(path);break;}
 opened=realloc(opened,(count+1)*sizeof *opened);if(!opened)die("descriptor allocation");opened[count++]=f;e->fd[r->fd]=f;e->fd_flags[r->fd]=!strncmp(op,"<<",2)||!strcmp(op,"<")?O_RDONLY:!strcmp(op,"<>")?O_RDWR:O_WRONLY|(!strcmp(op,">>")?O_APPEND:0);e->fd_cloexec[r->fd]=0;free(path);
 }
 e->in=e->fd[0];e->out=e->fd[1];e->err=e->fd[2];int status=redirect_error?1:execute(e,n);
 if(!persistent||redirect_error){for(int i=0;i<64;i++)if(saved[i])for(int j=0;j<64;j++)if(e->fd[j]==saved[i]){saved_flags[i]=(saved_flags[i]&~(O_APPEND|O_NONBLOCK))|(e->fd_flags[j]&(O_APPEND|O_NONBLOCK));break;}for(int i=count-1;i>=0;i--)if(fclose(opened[i]))die("closing redirection");memcpy(e->fd,saved,sizeof saved);memcpy(e->fd_flags,saved_flags,sizeof saved_flags);memcpy(e->fd_cloexec,saved_cloexec,sizeof saved_cloexec);}free(opened);e->in=e->fd[0];e->out=e->fd[1];e->err=e->fd[2];e->status=status;
 if(status&&e->errexit&&!e->suppress&&!e->flow)e->flow=4;frames=frame.previous;umask(previous_mask);return status;
}

"####;

const SOURCE_PROGRAM: &str = r####"
static Node *source_program(const char *path){char *source=load(path);char *previous=parsing_source;parsing_source=dup(path);Node *n=parse(source);parsing_source=previous;free(source);return n;}
"####;

const TEST: &str = r####"
typedef struct {char **v;int n,pos,error;} Test;
static int test_or(Test *t);
static int test_binary(const char *op){return !strcmp(op,"=")||!strcmp(op,"==")||!strcmp(op,"!=")||!strcmp(op,"-eq")||!strcmp(op,"-ne")||!strcmp(op,"-gt")||!strcmp(op,"-ge")||!strcmp(op,"-lt")||!strcmp(op,"-le")||!strcmp(op,"-nt")||!strcmp(op,"-ot")||!strcmp(op,"-ef");}
static long long test_number(Test *t,const char *s){char *end;errno=0;long long n=strtoll(s,&end,10);while(isspace((unsigned char)*end))end++;if(errno||end==s||*end)t->error=1;return n;}
static int test_pair(Test *t,const char *a,const char *op,const char *b){if(!strcmp(op,"=")||!strcmp(op,"=="))return !strcmp(a,b);if(!strcmp(op,"!="))return strcmp(a,b)!=0;if(!strcmp(op,"-nt")||!strcmp(op,"-ot")||!strcmp(op,"-ef")){struct stat x,y;int have_x=!stat(a,&x),have_y=!stat(b,&y);if(!strcmp(op,"-ef"))return have_x&&have_y&&x.st_dev==y.st_dev&&x.st_ino==y.st_ino;if(!strcmp(op,"-nt"))return have_x&&(!have_y||x.st_mtim.tv_sec>y.st_mtim.tv_sec||(x.st_mtim.tv_sec==y.st_mtim.tv_sec&&x.st_mtim.tv_nsec>y.st_mtim.tv_nsec));return have_y&&(!have_x||x.st_mtim.tv_sec<y.st_mtim.tv_sec||(x.st_mtim.tv_sec==y.st_mtim.tv_sec&&x.st_mtim.tv_nsec<y.st_mtim.tv_nsec));}long long x=test_number(t,a),y=test_number(t,b);if(!strcmp(op,"-eq"))return x==y;if(!strcmp(op,"-ne"))return x!=y;if(!strcmp(op,"-gt"))return x>y;if(!strcmp(op,"-ge"))return x>=y;if(!strcmp(op,"-lt"))return x<y;if(!strcmp(op,"-le"))return x<=y;t->error=1;return 0;}
static int test_unary(Test *t,const char *op,const char *s){if(!strcmp(op,"-n"))return *s!=0;if(!strcmp(op,"-z"))return *s==0;if(!strcmp(op,"-r"))return !access(s,R_OK);if(!strcmp(op,"-w"))return !access(s,W_OK);if(!strcmp(op,"-x"))return !access(s,X_OK);if(!strcmp(op,"-t")){long long fd=test_number(t,s);return fd>=0&&fd<64&&isatty(fd);}struct stat st;if(!strcmp(op,"-h")||!strcmp(op,"-L"))return !lstat(s,&st)&&S_ISLNK(st.st_mode);if(stat(s,&st))return 0;if(!strcmp(op,"-e"))return 1;if(!strcmp(op,"-f"))return S_ISREG(st.st_mode);if(!strcmp(op,"-d"))return S_ISDIR(st.st_mode);if(!strcmp(op,"-b"))return S_ISBLK(st.st_mode);if(!strcmp(op,"-c"))return S_ISCHR(st.st_mode);if(!strcmp(op,"-p"))return S_ISFIFO(st.st_mode);if(!strcmp(op,"-S"))return S_ISSOCK(st.st_mode);if(!strcmp(op,"-s"))return st.st_size>0;if(!strcmp(op,"-u"))return (st.st_mode&S_ISUID)!=0;if(!strcmp(op,"-g"))return (st.st_mode&S_ISGID)!=0;if(!strcmp(op,"-k"))return (st.st_mode&S_ISVTX)!=0;if(!strcmp(op,"-O"))return st.st_uid==geteuid();if(!strcmp(op,"-G"))return st.st_gid==getegid();t->error=1;return 0;}
static int test_primary(Test *t){if(t->pos==t->n){t->error=1;return 0;}const char *word=t->v[t->pos++];if(t->pos+1<t->n&&test_binary(t->v[t->pos])){const char *op=t->v[t->pos++],*rhs=t->v[t->pos++];return test_pair(t,word,op,rhs);}if(!strcmp(word,"!"))return !test_primary(t);if(!strcmp(word,"(")){int v=test_or(t);if(t->pos==t->n||strcmp(t->v[t->pos++],")"))t->error=1;return v;}if(t->pos<t->n&&word[0]=='-'&&word[1]&&word[2]==0){const char *rhs=t->v[t->pos++];return test_unary(t,word,rhs);}return *word!=0;}
static int test_and(Test *t){int v=test_primary(t);while(t->pos<t->n&&!strcmp(t->v[t->pos],"-a")){t->pos++;int rhs=test_primary(t);v=v&&rhs;}return v;}
static int test_or(Test *t){int v=test_and(t);while(t->pos<t->n&&!strcmp(t->v[t->pos],"-o")){t->pos++;int rhs=test_and(t);v=v||rhs;}return v;}
static int test_command(Args *a){Test t={a->v+1,a->n-1,0,0};if(!strcmp(a->v[0],"[")){if(!t.n||strcmp(t.v[t.n-1],"]"))return 2;t.n--;}if(!t.n)return 1;if(t.n==1)return !*t.v[0];if(t.n==2&&!strcmp(t.v[0],"!"))return !!*t.v[1];int v=test_or(&t);return t.error||t.pos!=t.n?2:!v;}
"####;

const DRIVER: &str = r####"
#include <sys/sysmacros.h>
/* All textual protocol records are netstrings: decimal byte count, ':',
   exactly that many bytes, ','. Binary streams are separate files. The
   request file is immutable and is copied byte-for-byte into its result. */
/* MANTLE-SHELL-2 header: ordinal,cwd,shell-zero,command-argv0,command argc/argv,
   shell argc/argv,variable count and (key,present,value,exported,readonly), then
   status,errexit,nounset,noglob,allexport,noclobber,actual creation mask,32 traps.
   Each of 64 slots: alias (-1 closed, otherwise canonical first parent alias);
   each primary adds position,access|APPEND|NONBLOCK,null boolean,workspace member
   (empty only for detached/null). Every live slot then adds its own CLOEXEC.
   Non-null primaries have complete descriptor-N backing snapshots. Workspace
   bindings are actual observed same-inode paths, never names guessed from argv.
   Child execution omits CLOEXEC slots; acknowledgment preserves parent slots. */
typedef struct Request {Env env;Args command;char *root,*workspace;unsigned long ordinal;int active;long positions[64];int aliases[64];char *descriptor_workspace[64];} Request;
static char *output_root,*journal_root,*workspace_root;
static unsigned long request_ordinal;
static void (*mantle_source_limit_suspend_hook)(Env *);
static void (*mantle_source_limit_resume_hook)(Env *);
static int (*mantle_source_entry_hook)(Env *,const char *);
static void mkdir_checked(const char *p){if(mkdir(p,0755)&&errno!=EEXIST)die("mkdir %s: %s",p,strerror(errno));}
static void binary_copy(const char *from,const char *to){FILE *a=fopen(from,"rb"),*b;if(!a)die("open %s: %s",from,strerror(errno));b=fopen(to,"wb");if(!b)die("create %s: %s",to,strerror(errno));transfer(a,b);if(fclose(a)||fclose(b))die("copy %s",from);}
static void remove_tree(const char *p){struct stat st;if(lstat(p,&st)){if(errno==ENOENT)return;die("stat %s",p);}if(!S_ISDIR(st.st_mode)){if(unlink(p))die("unlink %s",p);return;}DIR *d=opendir(p);if(!d)die("opendir %s",p);struct dirent *v;while((v=readdir(d))){if(!strcmp(v->d_name,".")||!strcmp(v->d_name,".."))continue;char *q=join(p,v->d_name);remove_tree(q);free(q);}closedir(d);if(rmdir(p))die("rmdir %s",p);}
static void copy_mtime(const char *to,const struct stat *st){struct timespec times[2];times[0].tv_sec=0;times[0].tv_nsec=UTIME_OMIT;times[1]=st->st_mtim;if(utimensat(AT_FDCWD,to,times,AT_SYMLINK_NOFOLLOW))die("copy workspace time %s",to);}
static void copy_tree(const char *from,const char *to){
 struct stat st;if(lstat(from,&st))die("stat %s",from);
 if(S_ISLNK(st.st_mode)){size_t size=st.st_size>0?(size_t)st.st_size+1:4096;char *target=alloc(size+1);ssize_t n=readlink(from,target,size);if(n<0||(size_t)n==size)die("symlink %s",from);target[n]=0;struct stat old;if(!lstat(to,&old))remove_tree(to);if(symlink(target,to))die("symlink %s",to);free(target);copy_mtime(to,&st);return;}
 if(!S_ISDIR(st.st_mode)){if(!S_ISREG(st.st_mode))die("nonregular workspace member %s",from);struct stat old;if(!lstat(to,&old)&&!S_ISREG(old.st_mode))remove_tree(to);binary_copy(from,to);if(chmod(to,st.st_mode&0777))die("chmod %s",to);copy_mtime(to,&st);return;}
 struct stat old;if(!lstat(to,&old)&&!S_ISDIR(old.st_mode))remove_tree(to);mkdir_checked(to);DIR *d=opendir(from);if(!d)die("opendir %s",from);struct dirent *v;while((v=readdir(d))){if(!strcmp(v->d_name,".")||!strcmp(v->d_name,".."))continue;char *a=join(from,v->d_name),*b=join(to,v->d_name);copy_tree(a,b);free(a);free(b);}closedir(d);if(chmod(to,st.st_mode&0777))die("chmod %s",to);copy_mtime(to,&st);
}
static void synchronize_tree(const char *from,const char *to){DIR *d=opendir(to);if(!d)die("workspace %s",to);struct dirent *v;while((v=readdir(d))){if(!strcmp(v->d_name,".")||!strcmp(v->d_name,".."))continue;char *a=join(from,v->d_name),*b=join(to,v->d_name);struct stat st;if(lstat(a,&st)){if(errno!=ENOENT)die("workspace stat");remove_tree(b);}else if(S_ISDIR(st.st_mode)){struct stat dst;if(!lstat(b,&dst)&&S_ISDIR(dst.st_mode))synchronize_tree(a,b);}free(a);free(b);}closedir(d);copy_tree(from,to);}
static void record(FILE *f,const char *s){size_t n=strlen(s);if(fprintf(f,"%lu:",(unsigned long)n)<0||fwrite(s,1,n,f)!=n||fputc(',',f)==EOF)die("checkpoint record write");}
static void integer(FILE *f,long long n){char b[64];snprintf(b,sizeof b,"%lld",n);record(f,b);}
static char *read_record(FILE *f){size_t n=0;int c=fgetc(f);if(c<'0'||c>'9')die("invalid checkpoint length");do{if(n>(64UL*1024*1024-(unsigned)(c-'0'))/10)die("checkpoint record too large");n=n*10+c-'0';c=fgetc(f);}while(c>='0'&&c<='9');if(c!=':')die("invalid checkpoint delimiter");char *s=alloc(n+1);if(fread(s,1,n,f)!=n||fgetc(f)!=','||memchr(s,0,n))die("invalid checkpoint record");return s;}
static long long read_integer(FILE *f){char *s=read_record(f),*end;errno=0;long long n=strtoll(s,&end,10);if(errno||!*s||*end)die("invalid checkpoint integer");free(s);return n;}
static void text_file(const char *root,const char *name,const char *text){char *p=join(root,name);FILE *f=fopen(p,"wb");if(!f||fwrite(text,1,strlen(text),f)!=strlen(text)||fclose(f))die("write %s",p);free(p);}
static int text_is(const char *root,const char *name,const char *value){char *p=join(root,name);char *s=load(p);int same=!strcmp(s,value);free(p);free(s);return same;}
static FILE *open_record(const char *root,const char *name,const char *mode){char *p=join(root,name);FILE *f=fopen(p,mode);if(!f)die("open %s: %s",p,strerror(errno));free(p);return f;}
static int files_equal(const char *a,const char *b){FILE *x=fopen(a,"rb"),*y=fopen(b,"rb");if(!x||!y)die("request comparison open");int c,d,same=1;do{c=fgetc(x);d=fgetc(y);if(c!=d){same=0;break;}}while(c!=EOF);if(ferror(x)||ferror(y))die("request comparison read");fclose(x);fclose(y);return same;}
static int mantle_descriptor_null(FILE *stream){struct stat st;if(fstat(fileno(stream),&st))die("inspect source descriptor");if(S_ISREG(st.st_mode))return 0;if(S_ISCHR(st.st_mode)&&major(st.st_rdev)==1&&minor(st.st_rdev)==3)return 1;die("unsupported original descriptor backing type");return 0;}
static int mantle_descriptor_flags(int fd){int flags=fcntl(fd,F_GETFL);if(flags<0)die("source descriptor status flags");if(flags&~(O_ACCMODE|O_APPEND|O_NONBLOCK|0x8000))die("unsupported original descriptor status flags");flags&=~0x8000;if((flags&O_ACCMODE)==O_ACCMODE)die("unsupported source descriptor access mode");return flags;}
static char *mantle_descriptor_workspace_fd(int fd,const char *workspace) {
 struct stat opened,current;if(fstat(fd,&opened))die("source descriptor inode");
 if(!S_ISREG(opened.st_mode)||!opened.st_nlink)return dup("");
 char proc[64],target[PATH_MAX];snprintf(proc,sizeof proc,"/proc/self/fd/%d",fd);
 ssize_t n=readlink(proc,target,sizeof target-1);if(n<0||n==(ssize_t)sizeof target-1)die("source descriptor path");target[n]=0;
 size_t prefix=strlen(workspace);if(strncmp(target,workspace,prefix)||target[prefix]!='/'||!target[prefix+1])return dup("");
 if(lstat(target,&current)){if(errno==ENOENT)return dup("");die("source descriptor path inode");}
 if(!S_ISREG(current.st_mode)||opened.st_dev!=current.st_dev||opened.st_ino!=current.st_ino)return dup("");
 if(!mantle_paths_clean(target+prefix+1,0))die("source descriptor workspace member");return dup(target+prefix+1);
}
static int mantle_descriptor_open_workspace(const char *workspace,const char *member,int flags) {
 if(!mantle_paths_clean(member,0))die("noncanonical source descriptor workspace member");
 int root=open(workspace,O_RDONLY|O_DIRECTORY|O_NOFOLLOW|O_CLOEXEC);if(root<0)die("source descriptor workspace root");
 char *parent=dup(member),*leaf=strrchr(parent,'/');int directory;
 if(leaf){*leaf++=0;directory=mantle_paths_directory(root,parent,0);}else{leaf=parent;directory=fcntl(root,F_DUPFD_CLOEXEC,3);}
 if(directory<0)die("source descriptor workspace parent");int guard=openat(directory,leaf,O_PATH|O_NOFOLLOW|O_CLOEXEC);if(guard<0)die("source descriptor workspace guard");struct stat st;if(fstat(guard,&st)||!S_ISREG(st.st_mode))die("source descriptor workspace guard is not regular");
 char proc[64];snprintf(proc,sizeof proc,"/proc/self/fd/%d",guard);int fd=open(proc,flags|O_CLOEXEC);int error=errno;
 if(close(guard)||close(directory)||close(root))die("source descriptor workspace parent close");free(parent);errno=error;if(fd<0)die("source descriptor workspace file");return fd;
}
static void stream_file(FILE *stream,const char *path,long *position){if(!stream){*position=-1;return;}if(fflush(stream))die("flush descriptor");long at=ftell(stream);if(at<0)die("descriptor position");char proc[64];snprintf(proc,sizeof proc,"/proc/self/fd/%d",fileno(stream));int fd=open(proc,O_RDONLY|O_CLOEXEC);if(fd<0)die("observe original descriptor backing");FILE *input=fdopen(fd,"rb"),*output=fopen(path,"wb");if(!input||!output)die("descriptor snapshot");transfer(input,output);if(fclose(input)||fclose(output))die("descriptor snapshot close");*position=at;}
static char *relative_cwd(const char *cwd){size_t n=strlen(workspace_root);if(strncmp(cwd,workspace_root,n)||(cwd[n]&&cwd[n]!='/'))return dup(cwd);return dup(cwd[n]?cwd+n+1:"");}
static void mantle_checkpoint_metadata(const char *root,const char *workspace){char *times=join(root,"times");mantle_workspace_capture(workspace,times);free(times);const char *roots=getenv("MANTLE_CONTROLLER_ROOTS");if(roots){char *to=join(root,"controller-roots");binary_copy(roots,to);free(to);}}
static void write_request(const char *root,Env *e,Args *a){mantle_source_ensure_mask(e);for(int i=0;i<64;i++)if(e->fd[i]&&fflush(e->fd[i]))die("source descriptor flush before workspace snapshot");mkdir_checked(root);text_file(root,"state","active\n");char *ws=join(root,"workspace");copy_tree(workspace_root,ws);free(ws);FILE *f=open_record(root,"request","wb");record(f,"MANTLE-SHELL-2");integer(f,request_ordinal);char *cwd=relative_cwd(e->cwd);record(f,cwd);free(cwd);record(f,e->zero?e->zero:"");record(f,a->argv0?a->argv0:a->v[0]);integer(f,a->n);for(int i=0;i<a->n;i++)record(f,a->v[i]);integer(f,e->argc);for(int i=0;i<e->argc;i++)record(f,e->args[i]);int count=0;for(Var *v=e->vars;v;v=v->next)count++;integer(f,count);for(Var *v=e->vars;v;v=v->next){record(f,v->key);integer(f,v->value!=NULL);record(f,v->value?v->value:"");integer(f,v->exported);integer(f,v->readonly);}integer(f,e->status);integer(f,e->errexit);integer(f,e->nounset);integer(f,e->noglob);integer(f,e->allexport);integer(f,e->noclobber);integer(f,e->creation_mask);for(int i=0;i<32;i++)record(f,e->traps[i]?e->traps[i]:"");
 for(int i=0;i<64;i++){int alias=-1;if(e->fd[i]){alias=i;for(int j=0;j<i;j++)if(e->fd[j]==e->fd[i]){alias=j;break;}}integer(f,alias);if(alias==i){int flags=e->fd_flags[i];if((flags&~(O_ACCMODE|O_APPEND|O_NONBLOCK))||(flags&O_ACCMODE)==O_ACCMODE)die("unsupported logical source descriptor flags");int null_device=mantle_descriptor_null(e->fd[i]);long position;if(null_device){if(fflush(e->fd[i])||(position=ftell(e->fd[i]))<0)die("null descriptor position");}else{char name[32];snprintf(name,sizeof name,"descriptor-%d",i);char *path=join(root,name);stream_file(e->fd[i],path,&position);free(path);}integer(f,position);integer(f,flags);integer(f,null_device);char *member=null_device?dup(""):mantle_descriptor_workspace_fd(fileno(e->fd[i]),workspace_root);record(f,member);free(member);}if(alias>=0){if(e->fd_cloexec[i]!=0&&e->fd_cloexec[i]!=1)die("source descriptor close-on-exec value");if(e->fd_flags[i]!=e->fd_flags[alias])die("source aliased descriptor flags differ");integer(f,e->fd_cloexec[i]);}}
 char *times=join(root,"workspace-times");mantle_workspace_capture(workspace_root,times);free(times);
 mantle_checkpoint_metadata(output_root,workspace_root);
 if(fclose(f))die("request close");
}
/* read_request materializes a fresh local workspace and independent descriptor
   streams, retaining aliases and initial offsets. The native producer writes
   those streams directly. No argv is handed to a shell or exec function. */
static void read_request_data_at(const char *tree,Request *r,const char *workspace){memset(r,0,sizeof *r);r->root=realpath(tree,NULL);if(!r->root)die("request root");if(text_is(r->root,"state","not-active\n")){r->active=0;return;}if(!text_is(r->root,"state","active\n"))die("unknown request state");r->active=1;char *from=join(r->root,"workspace");if(workspace)r->workspace=dup(workspace);else{char *cwd=getcwd(NULL,0);r->workspace=join(cwd,"workspace");free(cwd);}mkdir_checked(r->workspace);synchronize_tree(from,r->workspace);free(from);FILE *f=open_record(r->root,"request","rb");char *magic=read_record(f);if(strcmp(magic,"MANTLE-SHELL-2"))die("request version");free(magic);long long ordinal=read_integer(f);if(ordinal<0)die("request ordinal");r->ordinal=ordinal;char *relative=read_record(f);r->env.cwd=*relative=='/'?dup(relative):join(r->workspace,relative);free(relative);r->env.zero=read_record(f);r->command.argv0=read_record(f);long long count=read_integer(f);if(count<1||count>1048576)die("request argc");for(long long i=0;i<count;i++)arg(&r->command,read_record(f));count=read_integer(f);if(count<0||count>1048576)die("shell argc");r->env.argc=count;r->env.args=alloc((count+1)*sizeof *r->env.args);for(int i=0;i<count;i++)r->env.args[i]=read_record(f);count=read_integer(f);if(count<0||count>1048576)die("variable count");Var **tail=&r->env.vars;for(long long i=0;i<count;i++){Var *v=alloc(sizeof *v);v->key=read_record(f);int present=read_integer(f);char *value=read_record(f);v->value=present?value:NULL;v->exported=read_integer(f);v->readonly=read_integer(f);*tail=v;tail=&v->next;}r->env.status=read_integer(f);r->env.errexit=read_integer(f);r->env.nounset=read_integer(f);r->env.noglob=read_integer(f);r->env.allexport=read_integer(f);r->env.noclobber=read_integer(f);long long creation_mask=read_integer(f);if(creation_mask<0||creation_mask>0777)die("source request creation mask");r->env.creation_mask=creation_mask;r->env.mask_valid=1;for(int i=0;i<32;i++){char *s=read_record(f);r->env.traps[i]=*s?s:NULL;}
 for(int i=0;i<64;i++) {
  long long alias=read_integer(f);if(alias < -1||alias>i)die("descriptor alias");r->aliases[i]=alias;
  if(alias<0){r->positions[i]=-1;continue;}
  if(alias<i){if(r->aliases[alias]!=alias||!r->env.fd[alias])die("descriptor alias has no primary");r->env.fd[i]=r->env.fd[alias];r->positions[i]=r->positions[alias];r->env.fd_flags[i]=r->env.fd_flags[alias];r->descriptor_workspace[i]=r->descriptor_workspace[alias];}
  else {
   long long at=read_integer(f),flags=read_integer(f),null_device=read_integer(f);char *member=read_record(f);
   if(at<0||at>LONG_MAX)die("descriptor offset");if(flags<0||(flags&~(O_ACCMODE|O_APPEND|O_NONBLOCK))||(flags&O_ACCMODE)==O_ACCMODE)die("descriptor flags");
   if((null_device!=0&&null_device!=1)||(null_device&&*member)||(*member&&!mantle_paths_clean(member,0)))die("descriptor backing kind or workspace member");
   r->positions[i]=at;r->env.fd_flags[i]=flags;r->descriptor_workspace[i]=member;FILE *stream;const char *mode=(flags&O_ACCMODE)==O_RDONLY?"r":(flags&O_ACCMODE)==O_WRONLY?"w":"r+";
   if(null_device){int fd=open("/dev/null",flags|O_CLOEXEC);if(fd<0)die("native null descriptor");stream=fdopen(fd,mode);if(!stream||!mantle_descriptor_null(stream))die("native null descriptor identity");}
   else if(*member){int fd=mantle_descriptor_open_workspace(r->workspace,member,flags);stream=fdopen(fd,mode);if(!stream||mantle_descriptor_null(stream))die("workspace descriptor is not regular");char name[32],proc[64];snprintf(name,sizeof name,"descriptor-%d",i);snprintf(proc,sizeof proc,"/proc/self/fd/%d",fd);char *snapshot=join(r->root,name);if(!files_equal(proc,snapshot))die("source descriptor workspace binding bytes differ");free(snapshot);}
   else {char name[32];snprintf(name,sizeof name,"descriptor-%d",i);FILE *input=open_record(r->root,name,"rb");stream=tmpfile();if(!stream)die("native descriptor");transfer(input,stream);if(fclose(input))die("native descriptor snapshot close");int actual=fcntl(fileno(stream),F_GETFL);if(actual<0||fcntl(fileno(stream),F_SETFL,(actual&~(O_APPEND|O_NONBLOCK))|(flags&(O_APPEND|O_NONBLOCK))))die("native descriptor flags");}
   if(fseek(stream,at,SEEK_SET))die("native descriptor seek");r->env.fd[i]=stream;
  }
  long long cloexec=read_integer(f);if(cloexec!=0&&cloexec!=1)die("descriptor close-on-exec");r->env.fd_cloexec[i]=cloexec;
 }
 char *times=join(r->root,"workspace-times");mantle_workspace_restore(r->workspace,times);free(times);
 if(fgetc(f)!=EOF)die("trailing request bytes");fclose(f);r->env.in=r->env.fd[0];r->env.out=r->env.fd[1];r->env.err=r->env.fd[2];
}
static void read_request_data(const char *tree,Request *r){read_request_data_at(tree,r,NULL);}
static void read_request(const char *tree,Request *r){read_request_data(tree,r);if(r->active&&(mantle_paths_ready?mantle_paths_chdir(r->env.cwd):chdir(r->env.cwd)))die("native request cwd");}
/* stdout_stream/stderr_stream may be NULL when the producer writes env.fd
   directly. Otherwise they contain the real producer's diagnostic bytes. */
static void write_result(const char *tree,Request *r,int status,FILE *stdout_stream,FILE *stderr_stream){mkdir_checked(tree);if(!r->active){text_file(tree,"state","not-active\n");return;}if(status<0||status>255)die("normal status out of range");if(stdout_stream){rewind(stdout_stream);transfer(stdout_stream,r->env.fd[1]);}if(stderr_stream){rewind(stderr_stream);transfer(stderr_stream,r->env.fd[2]);}text_file(tree,"state","active\n");char number[16];snprintf(number,sizeof number,"%d\n",status);text_file(tree,"exit-status",number);char *a=join(r->root,"request"),*b=join(tree,"request");binary_copy(a,b);free(a);free(b);FILE *positions=open_record(tree,"positions","wb");for(int i=0;i<64;i++){integer(positions,r->aliases[i]);if(r->aliases[i]==i){long at;if(mantle_descriptor_null(r->env.fd[i])){if(fflush(r->env.fd[i])||(at=ftell(r->env.fd[i]))<0)die("result null descriptor position");}else{char name[32];snprintf(name,sizeof name,"descriptor-%d",i);char *p=join(tree,name);stream_file(r->env.fd[i],p,&at);free(p);}integer(positions,at);integer(positions,r->env.fd_flags[i]);}}if(fclose(positions))die("result positions close");char *ws=join(tree,"workspace");copy_tree(r->workspace,ws);free(ws);char *times=join(tree,"workspace-times");mantle_workspace_capture(r->workspace,times);free(times);}
static int restore_result(const char *request,const char *result,Env *e){if(!text_is(result,"state","active\n"))die("active command bound to inactive result");char *a=join(request,"request"),*b=join(result,"request");if(!files_equal(a,b))die("result does not acknowledge immutable request");free(a);free(b);for(int i=0;i<64;i++)if(e->fd[i]&&fflush(e->fd[i]))die("descriptor flush");char *ws=join(result,"workspace");synchronize_tree(ws,workspace_root);free(ws);char *times=join(result,"workspace-times");mantle_workspace_restore(workspace_root,times);free(times);FILE *positions=open_record(result,"positions","rb");for(int i=0;i<64;i++){int alias=read_integer(positions);if(alias < -1||alias>i)die("result descriptor alias");if(alias==i){long long at=read_integer(positions),flags=read_integer(positions);FILE *destination=e->fd[i];if(!destination)die("restore descriptor");if(flags<0||(flags&~(O_ACCMODE|O_APPEND|O_NONBLOCK))||(flags&O_ACCMODE)!=(e->fd_flags[i]&O_ACCMODE))die("restore descriptor flags");if(!mantle_descriptor_null(destination)){char name[32];snprintf(name,sizeof name,"descriptor-%d",i);char *before=join(request,name),*after=join(result,name);if(!files_equal(before,after)){char proc[64];snprintf(proc,sizeof proc,"/proc/self/fd/%d",fileno(destination));FILE *input=fopen(after,"rb"),*output=fopen(proc,"wb");if(!input||!output)die("restore original descriptor backing");transfer(input,output);if(fclose(input)||fclose(output))die("restore original descriptor backing close");}free(before);free(after);}int actual=fcntl(fileno(destination),F_GETFL);if(actual<0||fcntl(fileno(destination),F_SETFL,(actual&~(O_APPEND|O_NONBLOCK))|(flags&(O_APPEND|O_NONBLOCK))))die("restore actual descriptor flags");for(int j=0;j<64;j++)if(e->fd[j]==destination)e->fd_flags[j]=flags;if(at<0||at>LONG_MAX||fseek(destination,(long)at,SEEK_SET))die("restore descriptor position");}else if((alias<0)!=(e->fd[i]==NULL)||(alias>=0&&e->fd[i]!=e->fd[alias]))die("descriptor identity changed");}if(fgetc(positions)!=EOF||ferror(positions)||fclose(positions))die("trailing result positions");char *status_path=join(result,"exit-status"),*s=load(status_path),*end;errno=0;long status=strtol(s,&end,10);if(errno||status<0||status>255||strcmp(end,"\n"))die("invalid normal exit status");free(status_path);free(s);return status;}
static int mantle_shell_external(Env *e,Args *a){char name[64];snprintf(name,sizeof name,"%lu",request_ordinal);char *prior=join(journal_root,name);struct stat st;char *request=join(output_root,"request");if(mantle_source_limit_suspend_hook)mantle_source_limit_suspend_hook(e);remove_tree(request);write_request(request,e,a);if(!stat(prior,&st)){int status=restore_result(request,prior,e);request_ordinal++;free(prior);free(request);if(mantle_source_limit_resume_hook)mantle_source_limit_resume_hook(e);return status;}if(errno!=ENOENT)die("journal lookup");/* A normal helper exit is a pause, never a pretend command success. */text_file(output_root,"state","paused\n");char count[64];snprintf(count,sizeof count,"%lu\n",request_ordinal);text_file(output_root,"ordinal",count);exit(0);}
static int external(Env *e,Args *a){return mantle_source_external_hook?mantle_source_external_hook(e,a):mantle_shell_external(e,a);}
static void initial_environment(Env *e,const char *path){FILE *f=fopen(path,"rb");if(!f)die("initial environment");while(1){int c=fgetc(f);if(c==EOF)break;ungetc(c,f);char *key=read_record(f),*value=read_record(f);if(!*key||strchr(key,'='))die("initial environment key");set(e,key,value);variable(e,key)->exported=1;free(key);free(value);}if(ferror(f)||fclose(f))die("initial environment read");}
static unsigned long journal_count(const char *root){unsigned long n=0;for(;;n++){char name[64];snprintf(name,sizeof name,"%lu",n);char *p=join(root,name);struct stat st;int rc=stat(p,&st);free(p);if(rc){if(errno!=ENOENT)die("journal stat");return n;}if(!S_ISDIR(st.st_mode))die("journal result is not a tree");}}
static char *source_metadata(const char *key){const char *path=getenv(key);if(!path||!*path)die("source controller requires %s",key);char *value=load(path);size_t n=strlen(value);if(n&&value[n-1]=='\n')value[--n]=0;if(!n||strchr(value,'\n')||strchr(value,'\r'))die("invalid source controller metadata %s",key);return value;}
int main(int argc,char **argv){if(argc<9||strcmp(argv[1],"run"))die("usage: shell-vm run SOURCE SCRIPT INITIAL-ENV PREVIOUS-CHECKPOINT RESULT INITIAL-WORKSPACE OUTPUT [ARGS...]");char *source_root=realpath(argv[2],NULL),*environment=realpath(argv[4],NULL);if(!source_root||!environment)die("source or environment path");mkdir_checked(argv[8]);output_root=realpath(argv[8],NULL);workspace_root=join(output_root,"workspace");journal_root=join(output_root,"journal");mkdir_checked(journal_root);if(strcmp(argv[5],"-")){char *previous=realpath(argv[5],NULL);if(!previous)die("previous checkpoint");char *old=join(previous,"journal");copy_tree(old,journal_root);free(old);unsigned long count=journal_count(journal_root);if(strcmp(argv[6],"-")){char *result=realpath(argv[6],NULL);if(!result)die("result path");char name[64];snprintf(name,sizeof name,"%lu",count);char *to=join(journal_root,name);copy_tree(result,to);free(to);free(result);}free(previous);}else if(strcmp(argv[6],"-"))die("result without previous checkpoint");remove_tree(workspace_root);mkdir_checked(workspace_root);
 /* argv[7] is the immutable initial workspace tree, or '-' for an empty one.
    Replaying starts from that tree, never from the last mutable workspace. */
 if(strcmp(argv[7],"-")){char *initial=realpath(argv[7],NULL);if(!initial)die("initial workspace");copy_tree(initial,workspace_root);free(initial);}
 const char *initial_times=getenv("MANTLE_CONTROLLER_INITIAL_TIMES");if(initial_times&&*initial_times)mantle_workspace_restore(workspace_root,initial_times);
 char *initial_cwd=source_metadata("MANTLE_CONTROLLER_INITIAL_CWD");
 if(mantle_paths_init_environment(workspace_root,initial_cwd))die("source controller initial logical workspace: %s",strerror(errno));free(initial_cwd);
 Env e={0};e.cwd=mantle_paths_getcwd(NULL,0);if(!e.cwd)die("source controller initial cwd");
 e.zero=source_metadata("MANTLE_SHELL_ORIGINAL_ZERO");e.argc=argc-9;e.args=argv+9;initial_environment(&e,environment);if(!get(&e,"PWD"))set(&e,"PWD",e.cwd);
 for(int i=0;i<3;i++){e.fd[i]=tmpfile();if(!e.fd[i])die("initial descriptor");e.fd_flags[i]=i?O_WRONLY:O_RDONLY;}e.in=e.fd[0];e.out=e.fd[1];e.err=e.fd[2];
 char *script_path=join(source_root,argv[3]),*program=load(script_path);free(script_path);parsing_source=e.zero;int status=mantle_source_entry_hook?mantle_source_entry_hook(&e,program):-1;if(status < -1||status>255)die("invalid source entry status");if(status==-1)status=script(&e,program);free(program);
 if(e.traps[0]){char *trap=e.traps[0];e.traps[0]=NULL;e.flow=0;int saved=status;script(&e,trap);status=e.flow==4?e.status:saved;}
 if(request_ordinal!=journal_count(journal_root))die("unused acknowledged command results");
 mantle_checkpoint_metadata(output_root,workspace_root);text_file(output_root,"state","done\n");
 char *inactive=join(output_root,"request");remove_tree(inactive);mkdir_checked(inactive);text_file(inactive,"state","not-active\n");free(inactive);
 char value[16];snprintf(value,sizeof value,"%d\n",status);text_file(output_root,"exit-status",value);
 FILE *vars=open_record(output_root,"environment","wb");for(Var *v=e.vars;v;v=v->next)if(v->value){record(vars,v->key);record(vars,v->value);}fclose(vars);
 for(int i=1;i<3;i++){char *p=join(output_root,i==1?"stdout":"stderr");long at;stream_file(e.fd[i],p,&at);free(p);}return 1;}
"####;
