/* Included immediately before original filter_apply_chain. The original
   filter graph, not a configured guess, supplies the standalone m4 child. */
static void mantle_flex_boundary_fail(const char *message)
{
    fprintf(stderr,"original Flex child boundary: %s\n",message);
    abort();
}
static void mantle_flex_capture_child(struct filter *chain)
{
    const char *path=getenv("MANTLE_FLEX_RESPONSE");
    if(!path)return;
    FILE *out=fopen(path,"wb");if(!out)mantle_flex_boundary_fail("cannot write original Flex child boundary");
    struct filter *child=NULL,*f;int i;
    for(f=chain;f;f=f->next)if(f->argv){
        if(child)mantle_flex_boundary_fail("multiple external original Flex filter boundaries");
        child=f;
    }
    if(!child||child->argc<1||child->argc>16384)mantle_flex_boundary_fail("missing original Flex external filter");
    if(fprintf(out,"%d\n",child->argc)<0)mantle_flex_boundary_fail("writing original Flex child argument count");
    for(i=0;i<child->argc;i++){
        const char *word=child->argv[i];size_t n=strlen(word);
        if(n>65536||fprintf(out,"%lu:",(unsigned long)n)<0||fwrite(word,1,n,out)!=n||fputc(',',out)==EOF)
            mantle_flex_boundary_fail("writing original Flex child argument");
    }
    if(fclose(out))mantle_flex_boundary_fail("closing original Flex child boundary");
}
