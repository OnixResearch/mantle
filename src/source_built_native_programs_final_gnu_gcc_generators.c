/* Argument roles from original GCC 10.5 gcc/read-md.c:1209, genattrtab.c:5227,
 * genopinit.c:80, gengtype.c:4960, genhooks.c:340 and genmatch.c:5068. The
 * protected generated-image matcher, not any basename below, grants execution.
 * Original programs retain option validation, diagnostics and all generation. */
static int final_gnu_gengtype_prepare(Args *a,mantle_controller_builder *b)
{
    int options=1;
    for(int i=1;i<a->n;i++) {
        const char *s=a->v[i];
        if(options&&!strcmp(s,"--")){mantle_controller_argument_literal(b,s);options=0;continue;}
        if(options&&mantle_controller_oneof(s,"-P --plugin -S --srcdir -B --backupdir -I --inputs -r --read-state -w --write-state")) {
            int writing=mantle_controller_oneof(s,"-P --plugin -B --backupdir -w --write-state");mantle_controller_argument_literal(b,s);
            if(++i<a->n)mantle_controller_argument_path(b,a->v[i],writing);continue;
        }
        if(options&&s[0]=='-'&&s[1]&&strchr("PSBIrw",s[1])&&s[2]) {
            char flag[3]={'-',s[1],0};mantle_controller_option_path(b,s,flag,strchr("PBw",s[1])!=NULL);continue;
        }
        if(options&&s[0]=='-'&&s[1]) {
            static const char *const paths[]={"--plugin=","--srcdir=","--backupdir=","--inputs=","--read-state=","--write-state="};
            int handled=0;for(size_t p=0;p<sizeof paths/sizeof paths[0];p++)if(!strncmp(s,paths[p],strlen(paths[p]))){mantle_controller_option_path(b,s,paths[p],p==0||p==2||p==5);handled=1;break;}
            if(!handled)mantle_controller_argument_data(b,s);continue;
        }
        mantle_controller_argument_path(b,s,0);
    }
    return 1;
}
static int final_gnu_generated_prepare(Args *a,mantle_controller_builder *b)
{
    if(!a->n)return 0;const char *name=mantle_controller_basename(a->v[0]);
    if(!strcmp(name,"gengtype"))return final_gnu_gengtype_prepare(a,b);
    if(mantle_controller_oneof(name,"genmodes gencfn-macros gencheck gencondmd gengenrtl")) {
        for(int i=1;i<a->n;i++)mantle_controller_argument_data(b,a->v[i]);return 1;
    }
    if(!strcmp(name,"genchecksum")) {
        for(int i=1;i<a->n;i++)mantle_controller_argument_path(b,a->v[i],0);return 1;
    }
    if(!strcmp(name,"genhooks")) {
        for(int i=1;i<a->n;i++){if(a->n>=3&&i==2)mantle_controller_argument_path(b,a->v[i],0);else mantle_controller_argument_data(b,a->v[i]);}return 1;
    }
    if(!strcmp(name,"genmatch")) {
        for(int i=1;i<a->n;i++){if(i==a->n-1)mantle_controller_argument_path(b,a->v[i],0);else mantle_controller_argument_data(b,a->v[i]);}return 1;
    }
    if(!mantle_controller_oneof(name,"genattr genattr-common genattrtab genautomata gencodes genconditions genconfig genconstants genemit genenums genextract genflags genmddeps genmddump genopinit genoutput genpeep genpreds genrecog gentarget-def"))return 0;
    int options=1;
    for(int i=1;i<a->n;i++) {
        const char *s=a->v[i];
        if(options&&!strcmp(s,"--")){mantle_controller_argument_literal(b,s);options=0;continue;}
        if(options&&!strcmp(s,"-I")){mantle_controller_argument_literal(b,s);if(++i<a->n)mantle_controller_argument_path(b,a->v[i],0);continue;}
        if(options&&!strncmp(s,"-I",2)&&s[2]){mantle_controller_option_path(b,s,"-I",0);continue;}
        if(options&&s[0]=='-'&&s[1]) {
            if(s[2]&&((!strcmp(name,"genattrtab")&&strchr("ADL",s[1]))||(!strcmp(name,"genopinit")&&strchr("hc",s[1])))) {
                char flag[3]={'-',s[1],0};mantle_controller_option_path(b,s,flag,1);
            }else mantle_controller_argument_data(b,s);
            continue;
        }
        if(options&&!strcmp(s,"-"))mantle_controller_argument_literal(b,s);else mantle_controller_argument_path(b,s,0);
    }
    return 1;
}
