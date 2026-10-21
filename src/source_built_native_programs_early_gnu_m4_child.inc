/* Included after the genuine original M4 builtin implementation and shared
 * native process transport. Original argument validation, flushes, obstack
 * byte collection and sysval decoding remain at their original callsites. */
static FILE *mantle_m4_child_stream;
static int mantle_m4_child_status;
static char *mantle_m4_child_path;
static void mantle_m4_child_fail(const char *why)
{
    fprintf(stderr,"original M4 child boundary: %s\n",why);
    abort();
}
static int mantle_m4_child_request(const char *operation,const char *command,char **path)
{
    /* musl 1.1.24 system.c and popen.c both select this exact image/argv. */
    char *argv[4];int status;
    argv[0]=(char *)"sh";argv[1]=(char *)"-c";argv[2]=(char *)command;argv[3]=NULL;
    status=mantle_perl_transport(operation,"/bin/sh",argv,-1,path);
    if(status<0||status>65280||(status&255))
        mantle_m4_child_fail("non-normal authenticated child status");
    return status;
}
static int mantle_m4_child_system(const char *command)
{
    char *path=NULL;int status;
    if(!mantle_perl_transport_active())return system(command);
    status=mantle_m4_child_request("system",command,&path);
    if(path)mantle_m4_child_fail("system unexpectedly returned a pipe path");
    return status;
}
static FILE *mantle_m4_child_popen(const char *command,const char *mode)
{
    if(!mantle_perl_transport_active())return popen(command,mode);
    if(strcmp(mode,"r")||mantle_m4_child_stream)
        mantle_m4_child_fail("original esyscmd pipe invariant differs");
    mantle_m4_child_status=mantle_m4_child_request("readpipe",command,&mantle_m4_child_path);
    if(!mantle_m4_child_path)mantle_m4_child_fail("missing authenticated pipe output");
    mantle_m4_child_stream=fopen(mantle_m4_child_path,"rb");
    if(!mantle_m4_child_stream)mantle_m4_child_fail("opening authenticated pipe output");
    return mantle_m4_child_stream;
}
static int mantle_m4_child_pclose(FILE *stream)
{
    int closed,status,saved;
    if(stream!=mantle_m4_child_stream)return pclose(stream);
    status=mantle_m4_child_status;closed=fclose(stream);saved=errno;
    free(mantle_m4_child_path);mantle_m4_child_path=NULL;mantle_m4_child_stream=NULL;
    if(closed){errno=saved;return -1;}
    return status;
}
