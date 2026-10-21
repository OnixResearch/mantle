/* Exact source transform for the GCC 10.5.0 collect2 first-link phase seam.
 * Input is the authenticated ORIGINAL collect2.c, never an observed generated
 * compiler file. Phase and metadata sources are embedded before the original
 * source compiler runs; no virtual /inputs include leaks into original Make.
 * Usage: helper ORIGINAL PHASE_SOURCE METADATA_SOURCE OUTPUT
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define SOURCE_MAX (4u * 1024u * 1024u)
static char source[SOURCE_MAX + 1];
#define PHASE_MAX (128u * 1024u)
static char phase_source[PHASE_MAX + 1], metadata_source[PHASE_MAX + 1];
static const char anchor[] =
  "    bool early_exit\n"
  "      = rflag || (! DO_COLLECT_EXPORT_LIST && ! do_collecting);\n";
static const char metadata_anchor[] = "  process_args (&argc, argv);\n";
static void fail(const char *message)
{
  fprintf(stderr, "native collect2 source phase: %s\n", message);
  exit(1);
}
static size_t read_phase(const char *path,char *bytes)
{
  FILE *f=fopen(path,"rb");size_t n;
  if(!f)fail("cannot open authenticated phase source");
  n=fread(bytes,1,PHASE_MAX,f);
  if(ferror(f)||fgetc(f)!=EOF||ferror(f)||fclose(f)||!n||memchr(bytes,0,n))
    fail("invalid authenticated phase source");
  return n;
}
int main(int argc, char **argv)
{
  FILE *input;
  FILE *output;
  size_t length;
  size_t prefix;
  size_t metadata_prefix;
  char *metadata_match;
  char *match;
  if (argc != 5) fail("usage: helper ORIGINAL PHASE_SOURCE METADATA_SOURCE OUTPUT");
  size_t phase_length=read_phase(argv[2],phase_source);
  size_t metadata_length=read_phase(argv[3],metadata_source);
  input = fopen(argv[1], "rb");
  if (!input) fail("cannot open original collect2 source");
  length = fread(source, 1, SOURCE_MAX, input);
  if (ferror(input) || fgetc(input) != EOF || ferror(input)) fail("source exceeds bound or cannot be read");
  if (fclose(input)) fail("cannot close original source");
  if (memchr(source, 0, length)) fail("NUL in original source");
  source[length] = 0;
  if (strstr(source, "MANTLE_COLLECT2_PHASE_QUERY") || strstr(source, "collect2-phase-header"))
    fail("original source already contains a phase transform");
  match = strstr(source, anchor);
  if (!match || strstr(match + strlen(anchor), anchor)) fail("original GCC 10 collect2 source seam differs");
  prefix = (size_t)(match - source) + strlen(anchor);
  metadata_match = strstr(source, metadata_anchor);
  if (!metadata_match || strstr(metadata_match + strlen(metadata_anchor), metadata_anchor))
    fail("original GCC 10 collect2 metadata seam differs");
  metadata_prefix = (size_t)(metadata_match - source) + strlen(metadata_anchor);
  if (metadata_prefix >= prefix) fail("original collect2 source phase order differs");
  output = fopen(argv[4], "wbx");
  if (!output) fail("cannot exclusively create transformed source");
  if (fwrite(source, 1, metadata_prefix, output) != metadata_prefix ||
      fputc('\n',output) == EOF ||
      fwrite(metadata_source, 1, metadata_length, output) != metadata_length ||
      fputc('\n',output) == EOF ||
      fwrite(source + metadata_prefix, 1, prefix - metadata_prefix, output) != prefix - metadata_prefix ||
      fputc('\n',output) == EOF ||
      fwrite(phase_source, 1, phase_length, output) != phase_length ||
      fputc('\n',output) == EOF ||
      fwrite(source + prefix, 1, length - prefix, output) != length - prefix)
    fail("cannot write transformed source");
  if (fclose(output)) fail("cannot close transformed source");
  return 0;
}
