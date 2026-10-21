/* Source-owned response entry shim for original programs without @response.
 * Compile ORIGINAL main translation unit with -Dmain=mantle_phase_original_main;
 * compile this translation unit without that define, in the SAME language as
 * original main, and link the original program's real objects/libraries.
 * The original main signature must be int main(int,char **). No child launch,
 * flag interpretation, filesystem copying or generator emulation occurs here.
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#define RESPONSE_MAX (16u * 1024u * 1024u)
#define ARGUMENTS_MAX 16384u
#define WORD_MAX 65536u
static char response[RESPONSE_MAX + 1];
static char *arguments[ARGUMENTS_MAX + 2];
extern int mantle_phase_original_main(int, char **);

static void fail(const char *message)
{
    fprintf(stderr, "native source phase argv: %s\n", message);
    exit(1);
}
static int space(unsigned char byte)
{
    return byte == ' ' || byte == '\t' || byte == '\n' || byte == '\r';
}
static void phase_cwd(void)
{
    const char *path = getenv("MANTLE_PHASE_CWD");
    const char *component;
    const char *cursor;
    if (!path) return;
    if (strncmp(path, "/inputs/", 8) && strncmp(path, "/outputs/", 9))
        fail("phase cwd is not a declared input/output path");
    component = path + 1;
    for (cursor = component;; ++cursor) {
        unsigned char byte = (unsigned char)*cursor;
        if (byte && (byte <= 32 || byte == 127 || byte == '\\')) fail("invalid phase cwd byte");
        if (!byte || byte == '/') {
            size_t length = (size_t)(cursor - component);
            if (!length || (length == 1 && component[0] == '.') ||
                (length == 2 && component[0] == '.' && component[1] == '.'))
                fail("non-normalized phase cwd");
            if (!byte) break;
            component = cursor + 1;
        }
    }
    if (chdir(path)) fail("cannot enter original source generator cwd");
}
int main(int argc, char **argv)
{
    FILE *file;
    size_t length;
    size_t count = 1;
    char *read;
    char *write;
    if (argc != 2 || argv[1][0] != '@' || argv[1][1] == 0)
        fail("expected one explicit Produced @response input");
    file = fopen(argv[1] + 1, "rb");
    if (!file) fail("cannot open response input");
    length = fread(response, 1, RESPONSE_MAX, file);
    if (ferror(file) || fgetc(file) != EOF || ferror(file)) fail("response unreadable or oversized");
    if (fclose(file)) fail("cannot close response input");
    if (memchr(response, 0, length)) fail("NUL in response input");
    response[length] = 0;
    read = response;
    write = response;
    arguments[0] = argv[0];
    while (*read) {
        int quote = 0;
        char *start;
        while (space((unsigned char)*read)) ++read;
        if (!*read) break;
        if (count > ARGUMENTS_MAX) fail("too many response arguments");
        arguments[count++] = start = write;
        while (*read && (quote || !space((unsigned char)*read))) {
            unsigned char byte = (unsigned char)*read++;
            if (byte == '\\') {
                byte = (unsigned char)*read++;
                if (!byte || byte < 32 || byte == 127) fail("invalid response escape");
                *write++ = (char)byte;
            } else if (byte == '\'' || byte == '"') {
                if (!quote) quote = byte;
                else if (quote == byte) quote = 0;
                else *write++ = (char)byte;
            } else {
                if (byte < 32 || byte == 127) fail("control byte in response argument");
                *write++ = (char)byte;
            }
            if ((size_t)(write - start) > WORD_MAX) fail("response argument oversized");
        }
        if (quote) fail("unterminated response quote");
        while (space((unsigned char)*read)) ++read;
        *write++ = 0;
        if (*start == '@') fail("nested response expansion is not a source phase");
    }
    arguments[count] = NULL;
    phase_cwd();
    return mantle_phase_original_main((int)count, arguments);
}
