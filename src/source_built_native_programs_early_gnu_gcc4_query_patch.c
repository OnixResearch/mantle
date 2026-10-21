/* Add query-only metadata observation to original GCC 4.0.4.
 * Original argv0 handling, specs and build_search_list remain authorities.
 * CLI: helper ORIGINAL_GCC_C OUTPUT_GCC_C. */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define SOURCE_MAX (4u * 1024u * 1024u)
static char source[SOURCE_MAX + 1];
static const char prefix_anchor[] =
    "putenv_from_prefixes (struct path_prefix *paths, const char *env_var)\n"
    "{\n"
    "  putenv (build_search_list (paths, env_var, 1));\n"
    "}";
static const char prefix_replacement[] =
    "putenv_from_prefixes (struct path_prefix *paths, const char *env_var)\n"
    "{\n"
    "  char *mantle_query_environment = build_search_list (paths, env_var, 1);\n"
    "  putenv (mantle_query_environment);\n"
    "  if (verbose_only_flag)\n"
    "    fprintf (stderr, \"%s\\n\", mantle_query_environment);\n"
    "}";
static const char options_anchor[] =
    "  obstack_grow (&collect_obstack, \"\\0\", 1);\n"
    "  putenv (obstack_finish (&collect_obstack));\n"
    "}";
static const char options_replacement[] =
    "  obstack_grow (&collect_obstack, \"\\0\", 1);\n"
    "  {\n"
    "    char *mantle_query_environment = obstack_finish (&collect_obstack);\n"
    "    putenv (mantle_query_environment);\n"
    "    if (verbose_only_flag)\n"
    "      fprintf (stderr, \"%s\\n\", mantle_query_environment);\n"
    "  }\n"
    "}";
static const char compiler_anchor[] =
    "  process_command (argc, argv);\n";
static const char compiler_replacement[] =
    "  process_command (argc, argv);\n"
    "  if (verbose_only_flag)\n"
    "    fprintf (stderr, \"COLLECT_GCC=%s\\n\", getenv (\"COLLECT_GCC\"));\n";

static void fail(const char *message)
{
    fprintf(stderr, "original GCC4 query environment source: %s\n", message);
    abort();
}

int main(int argc, char **argv)
{
    const char *anchors[3] = { prefix_anchor, options_anchor, compiler_anchor };
    const char *replacements[3] = { prefix_replacement, options_replacement, compiler_replacement };
    char *positions[3];
    const char *cursor;
    size_t length, i;
    FILE *input, *output;
    if (argc != 3) fail("usage: helper ORIGINAL_GCC_C OUTPUT_GCC_C");
    input = fopen(argv[1], "rb");
    if (!input) fail("opening actual original driver source");
    length = fread(source, 1, SOURCE_MAX, input);
    if (ferror(input) || fgetc(input) != EOF || ferror(input)) fail("reading bounded original driver source");
    if (fclose(input)) fail("closing actual original driver source");
    if (memchr(source, 0, length)) fail("NUL in original driver source");
    source[length] = 0;
    if (strstr(source, "mantle_query_environment")) fail("driver source already has the query observation seam");
    for (i = 0; i < 3; ++i) {
        positions[i] = strstr(source, anchors[i]);
        if (!positions[i] || strstr(positions[i] + strlen(anchors[i]), anchors[i]))
            fail("original GCC4 driver environment seam differs");
        if (i && positions[i] <= positions[i - 1]) fail("original driver environment seam order differs");
    }
    output = fopen(argv[2], "wbx");
    if (!output) fail("exclusively creating observed original driver source");
    cursor = source;
    for (i = 0; i < 3; ++i) {
        size_t bytes = (size_t)(positions[i] - cursor);
        size_t replacement_length = strlen(replacements[i]);
        if (fwrite(cursor, 1, bytes, output) != bytes
            || fwrite(replacements[i], 1, replacement_length, output) != replacement_length)
            fail("writing observed driver source");
        cursor = positions[i] + strlen(anchors[i]);
    }
    length -= (size_t)(cursor - source);
    if (fwrite(cursor, 1, length, output) != length || fclose(output)) fail("closing observed driver source");
    return 0;
}
