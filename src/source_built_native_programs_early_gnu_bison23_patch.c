/* Source-only transform of authenticated Bison 2.3 src/output.c. No installed
 * executable, observed parser output, or command-line flag table enters it. */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#define SOURCE_MAX (4u * 1024u * 1024u)
static char source[SOURCE_MAX + 1];
static char phase[64u * 1024u + 1];
static void fail(const char *message) { fprintf(stderr,"original Bison 2.3 source seam: %s\n",message); exit(1); }
struct replacement { const char *before; const char *after; char *at; };
static struct replacement replacements[] = {
 {"  pid_t pid;\n", "  pid_t pid;\n  char const *mantle_mode = getenv (\"MANTLE_BISON_MODE\");\n  int mantle_replay = mantle_mode && !strcmp (mantle_mode, \"replay\");\n", NULL},
 {"  init_subpipe ();\n  pid = create_subpipe (argv, filter_fd);\n", "  if (mantle_mode)\n    {\n      if (!mantle_replay && strcmp (mantle_mode, \"capture\"))\n        error (EXIT_FAILURE, 0, \"invalid Bison 2.3 phase mode\");\n      mantle_bison23_response (argv, mantle_replay);\n    }\n  else\n    {\n      init_subpipe ();\n      pid = create_subpipe (argv, filter_fd);\n    }\n", NULL},
 {"  out = fdopen (filter_fd[0], \"w\");", "  out = mantle_mode ? tmpfile () : fdopen (filter_fd[0], \"w\");", NULL},
 {"  fputs (\"m4_divert_push(0)dnl\\n\", out);\n  xfclose (out);", "  fputs (\"m4_divert_push(0)dnl\\n\", out);\n  if (mantle_mode && !mantle_replay)\n    {\n      int mantle_byte;\n      if (fflush (out) || fseek (out, 0, SEEK_SET))\n        error (EXIT_FAILURE, get_errno (), \"rewinding original Bison muscles\");\n      while ((mantle_byte = fgetc (out)) != EOF)\n        if (fputc (mantle_byte, stdout) == EOF)\n          error (EXIT_FAILURE, get_errno (), \"writing original Bison muscles\");\n      if (ferror (out) || fflush (stdout))\n        error (EXIT_FAILURE, get_errno (), \"finishing original Bison muscles\");\n    }\n  xfclose (out);\n  if (mantle_mode && !mantle_replay)\n    exit (86);", NULL},
 {"  end_of_output_subpipe (pid, filter_fd);\n  in = fdopen (filter_fd[1], \"r\");", "  if (mantle_mode)\n    in = mantle_bison23_result ();\n  else\n    {\n      end_of_output_subpipe (pid, filter_fd);\n      in = fdopen (filter_fd[1], \"r\");\n    }", NULL},
 {"  reap_subpipe (pid, m4);", "  if (!mantle_mode)\n    reap_subpipe (pid, m4);", NULL}
};
static size_t load(const char *path, char *buffer, size_t cap) {
 FILE *f=fopen(path,"rb"); size_t n;
 if(!f)fail("opening authenticated source"); n=fread(buffer,1,cap,f);
 if(ferror(f)||n==cap||fclose(f))fail("reading bounded authenticated source"); buffer[n]=0;
 if(memchr(buffer,0,n))fail("NUL in source"); return n;
}
static void emit(FILE *f, const char *bytes, size_t size) { if(fwrite(bytes,1,size,f)!=size)fail("writing source adaptation"); }
int main(int argc,char **argv) {
 size_t size,phase_size,index; char *opener,*cursor; FILE *out;
 if(argc!=4)fail("usage: helper ORIGINAL PHASE-SOURCE OUTPUT");
 size=load(argv[1],source,sizeof source-1); phase_size=load(argv[2],phase,sizeof phase-1);
 opener=strstr(source,"static void\noutput_skeleton (void)\n{");
 if(!opener||strstr(opener+1,"static void\noutput_skeleton (void)\n{"))fail("original output_skeleton source seam differs");
 cursor=opener;
 for(index=0;index<sizeof replacements/sizeof replacements[0];++index) {
  struct replacement *r=&replacements[index]; r->at=strstr(cursor,r->before);
  if(!r->at||strstr(r->at+1,r->before))fail("original Bison 2.3 phase anchor differs");
  cursor=r->at+strlen(r->before);
 }
 out=fopen(argv[3],"wbx");if(!out)fail("creating source adaptation");
 emit(out,source,(size_t)(opener-source));emit(out,phase,phase_size);emit(out,"\n",1);cursor=opener;
 for(index=0;index<sizeof replacements/sizeof replacements[0];++index) {
  struct replacement *r=&replacements[index];emit(out,cursor,(size_t)(r->at-cursor));emit(out,r->after,strlen(r->after));cursor=r->at+strlen(r->before);
 }
 emit(out,cursor,size-(size_t)(cursor-source));if(fclose(out))fail("closing source adaptation");return 0;
}
