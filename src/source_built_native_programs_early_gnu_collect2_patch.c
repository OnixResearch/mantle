/* Source-only transform of authenticated original GCC 4.0.4 collect2.c.
 * The paired phase include is an independent manifest source input. No
 * command-line translation, installed compiler image or measured data enters
 * this source transform. Usage: helper ORIGINAL PHASE_SOURCE OUTPUT. */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define SOURCE_MAX (4u * 1024u * 1024u)
static char source[SOURCE_MAX + 1];
#define PHASE_MAX (64u * 1024u)
static char phase[PHASE_MAX + 1];
static const char startup_anchor[] =
  "  no_demangle = !! getenv (\"COLLECT_NO_DEMANGLE\");\n";
static const char phase_anchor[] =
  "  *c_ptr = *ld1 = *object = (char *) 0;\n";
static const char startup[] =
  "\n#define mantle_collect2_phase_fail(message) \\\n"
  "  do { fputs (\"source-owned collect2 phase: \", stderr); fputs ((message), stderr); fputc ('\\n', stderr); abort (); } while (0)\n"
  "\n  if (getenv (\"MANTLE_COLLECT2_PHASE_QUERY\"))\n"
  "    {\n"
  "      const char *mantle_options = getenv (\"COLLECT_GCC_OPTIONS\");\n"
  "      const char *mantle_query = getenv (\"MANTLE_COLLECT2_PHASE_QUERY\");\n"
  "      if (strcmp (mantle_query, \"1\"))\n"
  "        mantle_collect2_phase_fail (\"invalid source-owned collect2 phase mode\");\n"
  "#if defined (OBJECT_FORMAT_COFF) || defined (COLLECT_EXPORT_LIST) || defined (LDD_SUFFIX)\n"
  "      mantle_collect2_phase_fail (\"collect2 target requires additional source-owned phases\");\n"
  "#endif\n"
  "      if (do_collecting)\n"
  "        mantle_collect2_phase_fail (\"collect2 target requires constructor collection phases\");\n"
  "      if (mantle_options && strstr (mantle_options, \"-frepo\"))\n"
  "        mantle_collect2_phase_fail (\"collect2 invocation requires repository phases\");\n"
  "    }\n";

static void fail(const char *message)
{
    fprintf(stderr, "native GCC4 collect2 source phase: %s\n", message);
    abort();
}

int main(int argc, char **argv)
{
    FILE *input, *output;
    size_t length, phase_length, first, second;
    char *a, *b;
    if (argc != 4) fail("usage: helper ORIGINAL PHASE_SOURCE OUTPUT");
    input = fopen(argv[1], "rb");
    if (!input) fail("opening original collect2 source");
    length = fread(source, 1, SOURCE_MAX, input);
    if (ferror(input) || fgetc(input) != EOF || ferror(input)) fail("source exceeds bound or cannot be read");
    if (fclose(input)) fail("closing original collect2 source");
    if (memchr(source, 0, length)) fail("NUL in original source");
    source[length] = 0;
    if (strstr(source, "MANTLE_COLLECT2_PHASE_QUERY") || strstr(source, "collect2-phase-header"))
        fail("original source already contains a phase transform");
    a = strstr(source, startup_anchor);
    b = strstr(source, phase_anchor);
    if (!a || !b || b <= a || strstr(a + strlen(startup_anchor), startup_anchor)
        || strstr(b + strlen(phase_anchor), phase_anchor))
        fail("original GCC 4.0.4 collect2 source seams differ");
    first = (size_t)(a - source);
    second = (size_t)(b - source) + strlen(phase_anchor);
    input = fopen(argv[2], "rb");
    if (!input) fail("opening authenticated collect2 phase source");
    phase_length = fread(phase, 1, PHASE_MAX, input);
    if (ferror(input) || fgetc(input) != EOF || ferror(input)) fail("phase source exceeds bound or cannot be read");
    if (fclose(input) || !phase_length || memchr(phase, 0, phase_length)) fail("invalid collect2 phase source");
    output = fopen(argv[3], "wbx");
    if (!output) fail("exclusively creating transformed source");
    if (fwrite(source, 1, first, output) != first
        || fwrite(startup, 1, sizeof(startup) - 1, output) != sizeof(startup) - 1
        || fwrite(source + first, 1, second - first, output) != second - first
        || fputc('\n', output) == EOF
        || fwrite(phase, 1, phase_length, output) != phase_length
        || fputc('\n', output) == EOF
        || fwrite(source + second, 1, length - second, output) != length - second)
        fail("writing transformed collect2 source");
    if (fclose(output)) fail("closing transformed collect2 source");
    return 0;
}
