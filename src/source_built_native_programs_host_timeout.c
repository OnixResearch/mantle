/* Original timeout control flow, without an opaque BusyBox child process.
 * CLOCK_MONOTONIC entry is measured inside the admitted source evaluator.
 * Exact first-child journal metadata survives every continuation; no deadline
 * or normal exit status is invented by a resumed source helper. */
#include <signal.h>
#include <sys/time.h>
#include <time.h>

#define MANTLE_NATIVE_SCOPE_KEY "MANTLE_NATIVE_PROCESS_SCOPES"
#define MANTLE_NATIVE_SCOPE_MAGIC "MANTLE-NATIVE-PROCESS-SCOPES-1"
static unsigned long mantle_native_timeout_ordinal;
static unsigned long mantle_native_timeout_completed_depth;
static unsigned long mantle_native_timeout_journal_size;
static int mantle_native_timeout_journal_loaded;
static char *mantle_native_timeout_last_table;

static unsigned long long mantle_native_scope_number(const char *text) {
    if(!*text)die("empty native timeout number");
    unsigned long long value=0;
    for(const unsigned char *p=(const unsigned char *)text;*p;p++) {
        if(*p<'0'||*p>'9'||value>(ULLONG_MAX-(*p-'0'))/10)die("invalid native timeout number");
        value=value*10+*p-'0';
    }
    return value;
}
static unsigned long long mantle_native_monotonic_ns(void) {
    struct timespec now;
    if(clock_gettime(CLOCK_MONOTONIC,&now)||now.tv_sec<0||now.tv_nsec<0||now.tv_nsec>=1000000000L)die("native timeout monotonic clock");
    if((unsigned long long)now.tv_sec>ULLONG_MAX/1000000000ULL)die("native timeout clock overflow");
    return (unsigned long long)now.tv_sec*1000000000ULL+(unsigned long long)now.tv_nsec;
}
static FILE *mantle_native_scope_stream(const char *table) {
    FILE *f=fmemopen((void *)table,strlen(table),"rb");
    if(!f)die("native timeout scope stream");
    char *magic=read_record(f);
    if(strcmp(magic,MANTLE_NATIVE_SCOPE_MAGIC))die("native timeout scope version");
    free(magic);return f;
}
static void mantle_native_timeout_arm(const char *table) {
    unsigned long long earliest=ULLONG_MAX;
    if(table&&*table) {
        FILE *f=mantle_native_scope_stream(table);
        int c;
        while((c=fgetc(f))!=EOF) {
            if(ungetc(c,f)==EOF)die("native timeout scope lookahead");
            char *id=read_record(f),*seconds=read_record(f),*deadline=read_record(f);
            if(!*id||!mantle_native_scope_number(seconds))die("invalid native timeout scope");
            unsigned long long value=mantle_native_scope_number(deadline);
            if(value<earliest)earliest=value;
            free(id);free(seconds);free(deadline);
        }
        if(ferror(f)||fclose(f))die("native timeout scope close");
    }
    struct itimerval timer;memset(&timer,0,sizeof timer);
    if(earliest!=ULLONG_MAX) {
        unsigned long long now=mantle_native_monotonic_ns();
        if(earliest<=now)die("original timeout source arena expired");
        unsigned long long remaining=earliest-now;
        timer.it_value.tv_sec=(time_t)(remaining/1000000000ULL);
        timer.it_value.tv_usec=(suseconds_t)((remaining%1000000000ULL+999ULL)/1000ULL);
        if(timer.it_value.tv_usec==1000000){timer.it_value.tv_sec++;timer.it_value.tv_usec=0;}
        if(signal(SIGALRM,SIG_DFL)==SIG_ERR)die("native timeout signal policy");
    }
    if(setitimer(ITIMER_REAL,&timer,NULL))die("native timeout source timer");
}
static void mantle_native_timeout_resume(Env *e) {
    mantle_native_timeout_arm(get(e,MANTLE_NATIVE_SCOPE_KEY));
}
/* Decode only immutable request metadata. read_request_data also restores a
 * mutable workspace and therefore must not be used during source replay. */
static char *mantle_native_timeout_journal_table(unsigned long index) {
    char ordinal[64];snprintf(ordinal,sizeof ordinal,"%lu",index);
    char *prior=join(journal_root,ordinal);struct stat st;
    if(stat(prior,&st)){int absent=errno==ENOENT;free(prior);if(absent)return NULL;die("native timeout first-child journal");}
    FILE *f=open_record(prior,"request","rb");free(prior);
    char *magic=read_record(f);
    if(strcmp(magic,"MANTLE-SHELL-2")||read_integer(f)!=(long long)index)die("native timeout immutable request identity");
    free(magic);
    for(int i=0;i<3;i++){char *value=read_record(f);free(value);}
    for(int group=0;group<2;group++) {
        long long count=read_integer(f);if(count<0)die("native timeout original argument count");
        for(long long i=0;i<count;i++){char *value=read_record(f);free(value);}
    }
    long long count=read_integer(f);if(count<0)die("native timeout original environment count");
    char *table=NULL;
    for(long long i=0;i<count;i++) {
        char *key=read_record(f);long long present=read_integer(f);char *value=read_record(f);
        (void)read_integer(f);(void)read_integer(f);
        if(!strcmp(key,MANTLE_NATIVE_SCOPE_KEY)) {
            if(table||present!=1)die("native timeout duplicate or absent journal scope table");
            table=value;value=NULL;
        }
        free(key);free(value);
    }
    if(fclose(f))die("native timeout immutable request close");
    return table;
}
static unsigned long long mantle_native_timeout_new_deadline(unsigned long long seconds) {
    unsigned long long now=mantle_native_monotonic_ns();
    if(seconds>(ULLONG_MAX-now)/1000000000ULL)die("original timeout duration overflow");
    return now+seconds*1000000000ULL;
}
static unsigned long long mantle_native_timeout_lookup(const char *table,const char *id,unsigned long long seconds,int *found) {
    *found=0;if(!table)return 0;
    FILE *f=mantle_native_scope_stream(table);unsigned long long result=0;int c;
    while((c=fgetc(f))!=EOF) {
        if(ungetc(c,f)==EOF)die("native timeout journal scope lookahead");
        char *key=read_record(f),*duration=read_record(f),*deadline=read_record(f);
        if(!strcmp(key,id)) {
            if(*found||mantle_native_scope_number(duration)!=seconds)die("original timeout scope duration changed on continuation");
            result=mantle_native_scope_number(deadline);*found=1;
        }
        free(key);free(duration);free(deadline);
    }
    if(ferror(f)||fclose(f))die("native timeout journal scope close");
    if(*found&&!result)die("original timeout continuation lost its stable measured deadline");
    return result;
}
static unsigned long long mantle_native_timeout_deadline(const char *id,unsigned long long seconds,int *completed) {
    char *table=mantle_native_timeout_journal_table(request_ordinal);int found;
    unsigned long long deadline=mantle_native_timeout_lookup(table,id,seconds,&found);free(table);
    /* Pure source children have no command journal entry. A later command's
     * request cannot turn that completed pure child into an active old scope. */
    if(!found)return mantle_native_timeout_new_deadline(seconds);
    if(!mantle_native_timeout_journal_loaded) {
        mantle_native_timeout_journal_size=journal_count(journal_root);
        if(mantle_native_timeout_journal_size)mantle_native_timeout_last_table=mantle_native_timeout_journal_table(mantle_native_timeout_journal_size-1);
        mantle_native_timeout_journal_loaded=1;
    }
    (void)mantle_native_timeout_lookup(mantle_native_timeout_last_table,id,seconds,&found);
    /* A subsequent real request outside this ID authenticates the original
     * return. Replay preserves its request bytes, not an expired live arena. */
    if(!found)*completed=1;
    return deadline;
}
static int mantle_native_timeout(Env *parent,Args *a) {
    if(!a->n)return -1;
    int first=1;const char *name=mantle_native_basename(a->v[0]);
    if(!strcmp(name,"busybox")&&a->n>1&&!strcmp(a->v[1],"timeout"))first=2;
    else if(strcmp(name,"timeout"))return -1;
    if(first<a->n&&!strcmp(a->v[first],"--"))first++;
    if(first+1>=a->n)die("original native timeout requires duration and command");
    /* Every selected original host recipe supplies its actual positive decimal
     * TOOL_PROBE_TIMEOUT_SECONDS. Other grammars are not silently approximated. */
    unsigned long long seconds=mantle_native_scope_number(a->v[first++]);
    if(!seconds)die("original native timeout requires a positive duration");
    const char *root=getenv("MANTLE_NATIVE_SCOPE_ROOT");
    if(!root||!*root)die("original native timeout requires immutable controller scope identity");
    char *id=NULL;
    if(asprintf(&id,"%s.timeout.%lu",root,mantle_native_timeout_ordinal++)<0)die("native timeout stable identity");
    int completed=mantle_native_timeout_completed_depth!=0;
    unsigned long long deadline=mantle_native_timeout_deadline(id,seconds,&completed);
    const char *outer=get(parent,MANTLE_NATIVE_SCOPE_KEY);char *table=NULL;size_t size=0;
    FILE *f=open_memstream(&table,&size);if(!f)die("native timeout captured scope table");
    if(outer&&*outer){if(fwrite(outer,1,strlen(outer),f)!=strlen(outer))die("native timeout outer scope copy");}
    else record(f,MANTLE_NATIVE_SCOPE_MAGIC);
    record(f,id);char value[64];snprintf(value,sizeof value,"%llu",seconds);record(f,value);snprintf(value,sizeof value,"%llu",deadline);record(f,value);
    if(fclose(f))die("native timeout captured scope close");free(id);
    Env child=mantle_native_shell_environment(parent);
    set(&child,MANTLE_NATIVE_SCOPE_KEY,table);variable(&child,MANTLE_NATIVE_SCOPE_KEY)->exported=1;
    if(completed)mantle_native_timeout_completed_depth++;
    if(!mantle_native_timeout_completed_depth)mantle_native_timeout_arm(table);free(table);
    Args command={.v=a->v+first,.n=a->n-first};int status=external(&child,&command);
    if(!mantle_native_timeout_completed_depth&&mantle_native_monotonic_ns()>=deadline)die("original timeout source arena expired before child return");
    if(completed)mantle_native_timeout_completed_depth--;
    if(!mantle_native_timeout_completed_depth)mantle_native_timeout_arm(outer);
    mantle_native_env_free(&child,0);
    if(mantle_paths_chdir(parent->cwd))die("restore original timeout parent cwd");
    return status;
}
static void mantle_native_timeout_attributes(mantle_controller_builder *b,Buf *json,int *field_count) {
    const char *table=get(&b->request->env,MANTLE_NATIVE_SCOPE_KEY);
    if(!table||!*table)return;
    FILE *f=mantle_native_scope_stream(table);
    if((*field_count)++)putsb(json,",");
    putsb(json,"\"timeout_scopes\":[");int count=0,c;
    while((c=fgetc(f))!=EOF) {
        if(ungetc(c,f)==EOF)die("native timeout attribute lookahead");
        char *id=read_record(f),*duration=read_record(f),*deadline=read_record(f);
        unsigned long long seconds=mantle_native_scope_number(duration),absolute=mantle_native_scope_number(deadline);
        if(!*id||!seconds||!absolute)die("invalid measured native timeout attributes");
        if(count++)putsb(json,",");
        putsb(json,"{\"id\":");mantle_controller_json(json,id);
        char value[160];snprintf(value,sizeof value,",\"seconds_max\":%llu,\"monotonic_deadline_ns\":%llu}",seconds,absolute);putsb(json,value);
        free(id);free(duration);free(deadline);
    }
    if(ferror(f)||fclose(f))die("native timeout attribute close");
    putsb(json,"]");
}
