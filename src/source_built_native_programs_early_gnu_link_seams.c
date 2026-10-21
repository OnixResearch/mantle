/* Source-owned translations of the original GCC 4.0 linker shell seams.
 * This program reads collect2's actual no-child query, transforms its argv by
 * the selected original recipe, and prints another query. It never executes
 * the assembler, linker, a shell, or any other child.
 * CLI: MODE LOG EXPECTED CRT LIBC LIBM LIBGCC RUNTIME COMPAT IMAGE OUTPUT
 * Modes: native, seed-pass3, standard, installed-pass4, generator-base, native-as.
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>

#define LOG_MAX (16u * 1024u * 1024u)
#define WORD_MAX 65536u
#define ARGS_MAX 16384u
static char bytes[LOG_MAX + 1];
static char path[WORD_MAX + 1];
static char *words[ARGS_MAX];
static char *directories[ARGS_MAX];

static void fail(const char *message)
{
    fprintf(stderr, "original GCC4 linker seam: %s\n", message);
    abort();
}

static size_t load(const char *name)
{
    FILE *file = fopen(name, "rb");
    size_t count;
    if (!file) fail("opening actual collect2 query");
    count = fread(bytes, 1, LOG_MAX, file);
    if (ferror(file) || fgetc(file) != EOF || ferror(file)) fail("reading bounded actual collect2 query");
    if (fclose(file)) fail("closing actual collect2 query");
    if (memchr(bytes, 0, count)) fail("NUL in actual collect2 query");
    bytes[count] = 0;
    return count;
}

/* print_argv syntax, not shell evaluation. This matches the original GCC
 * query encoding; command substitution and child execution are not involved. */
static size_t split(void)
{
    char *read = bytes;
    char *write = bytes;
    size_t count = 0;
    while (*read == ' ' || *read == '\t') ++read;
    while (*read && *read != '\n') {
        int quote = 0;
        char *start = write;
        if (count == ARGS_MAX) fail("too many actual linker arguments");
        words[count++] = write;
        while (*read && (quote || (*read != ' ' && *read != '\t' && *read != '\n'))) {
            unsigned char byte = (unsigned char)*read++;
            if (byte < 32 || byte == 127) fail("control byte in actual linker argument");
            if (byte == '\\' && quote != '\'') {
                byte = (unsigned char)*read++;
                if (!byte || byte < 32 || byte == 127) fail("invalid linker argument escape");
                *write++ = (char)byte;
            } else if (byte == '\'' || byte == '"') {
                if (!quote) quote = byte;
                else if (quote == byte) quote = 0;
                else *write++ = (char)byte;
            } else {
                if (!quote && strchr("|;&<>`$()", byte)) fail("shell operator outside query quoting");
                *write++ = (char)byte;
            }
            if ((size_t)(write - start) > WORD_MAX) fail("oversized actual linker argument");
        }
        if (quote) fail("unterminated actual linker argument");
        while (*read == ' ' || *read == '\t') ++read;
        if (*read == '\n') {
            ++read;
            if (*read) fail("unexpected second collect2 phase");
            *write = 0;
            return count;
        }
        *write++ = 0;
    }
    fail("actual collect2 query has no final newline");
    return 0;
}

static void emit(FILE *file, const char *word)
{
    const unsigned char *cursor = (const unsigned char *)word;
    if (fputs(" \"", file) == EOF) fail("writing adapted query");
    while (*cursor) {
        if (*cursor < 32 || *cursor == 127) fail("control byte in adapted argument");
        if ((*cursor == '"' || *cursor == '\\') && fputc('\\', file) == EOF) fail("escaping adapted query");
        if (fputc(*cursor++, file) == EOF) fail("writing adapted argument");
    }
    if (fputc('"', file) == EOF) fail("terminating adapted argument");
}

static const char *basename_of(const char *word)
{
    const char *slash = strrchr(word, '/');
    return slash ? slash + 1 : word;
}

static int crt(const char *word)
{
    return !strcmp(word, "crt1.o") || !strcmp(word, "crti.o") || !strcmp(word, "crtn.o");
}

static int private_crt(const char *word)
{
    const char *name = basename_of(word);
    size_t length = strlen(name);
    return length >= 8 && (!strncmp(name, "crtbegin", 8) || !strncmp(name, "crtend", 6))
        && !strcmp(name + length - 2, ".o");
}

static const char *join(const char *directory, const char *name)
{
    size_t a = strlen(directory), b = strlen(name);
    if (a + b + 1 > WORD_MAX) fail("adapted original library path exceeds argument bound");
    memcpy(path, directory, a);
    path[a] = '/';
    memcpy(path + a + 1, name, b + 1);
    return path;
}

static const char *find_library(const char *name, size_t count)
{
    size_t i;
    size_t n = strlen(name);
    for (i = 0; i < count; ++i) {
        size_t d = strlen(directories[i]);
        struct stat metadata;
        if (d + n + 6 > WORD_MAX) fail("original library search path exceeds argument bound");
        memcpy(path, directories[i], d);
        memcpy(path + d, "/lib", 4);
        memcpy(path + d + 4, name, n);
        memcpy(path + d + 4 + n, ".a", 3);
        if (!stat(path, &metadata) && S_ISREG(metadata.st_mode)) return path;
    }
    fail("relocated generator linker could not resolve original -l argument");
    return NULL;
}

int main(int argc, char **argv)
{
    size_t count, i, directory_count = 0;
    int native, seed, standard, resolve, generator, assembler, expect_directory = 0;
    FILE *result;
    if (argc != 12) fail("usage: MODE LOG EXPECTED CRT LIBC LIBM LIBGCC RUNTIME COMPAT IMAGE OUTPUT");
    native = !strcmp(argv[1], "native");
    seed = !strcmp(argv[1], "seed-pass3");
    standard = !strcmp(argv[1], "standard");
    generator = !strcmp(argv[1], "generator-base");
    resolve = generator || !strcmp(argv[1], "installed-pass4");
    assembler = !strcmp(argv[1], "native-as");
    if (!native && !seed && !standard && !resolve && !assembler) fail("unknown original GCC4 recipe seam");
    load(argv[2]);
    count = split();
    if (!count || strcmp(words[0], argv[3])) fail("actual collect2 image differs from original selected linker");
    result = fopen(argv[11], "wbx");
    if (!result) fail("exclusively creating adapted linker query");
    emit(result, argv[10]);
    for (i = 1; i < count; ++i) {
        const char *word = words[i];
        const char *name = basename_of(word);
        if (assembler) {
            if (strcmp(word, "-Qy")) emit(result, word);
            continue;
        }
        if (resolve && expect_directory) {
            directories[directory_count++] = words[i];
            expect_directory = 0;
            continue;
        }
        if (resolve && !strcmp(word, "-L")) { expect_directory = 1; continue; }
        if (resolve && !strncmp(word, "-L", 2)) { directories[directory_count++] = words[i] + 2; continue; }
        if (native || seed) {
            if (!strcmp(word, "-lgcc_eh") || (seed && (!strcmp(word, "-lgcc") || !strcmp(word, "-lgcc_s")))
                || (strchr(word, '/') && (!strcmp(name, "libgcc.a") || !strcmp(name, "libtcc1.a")))
                || private_crt(word)) continue;
        } else {
            if (!strcmp(word, "-lgcc_s") || (generator && strchr(word, '/') && !strncmp(name, "libgcc_s.so", 11))) continue;
            if (!strcmp(word, "-lgcc") || !strcmp(word, "-lgcc_eh")
                || (generator && strchr(word, '/') && (!strcmp(name, "libgcc.a") || !strcmp(name, "libgcc_eh.a")))) {
                emit(result, argv[7]);
                continue;
            }
        }
        if (crt(word) || (generator && strchr(word, '/') && crt(name))) emit(result, join(argv[4], name));
        else if (!strcmp(word, "-lc")) emit(result, argv[5]);
        else if (!strcmp(word, "-lm")) { if (!native) emit(result, argv[6]); }
        else if (resolve && !strncmp(word, "-l", 2)) emit(result, find_library(word + 2, directory_count));
        else emit(result, word);
    }
    if (expect_directory) fail("relocated generator linker received -L without a directory");
    if (native) emit(result, argv[8]);
    if (native || seed) emit(result, argv[9]);
    if (fputc('\n', result) == EOF || fclose(result)) fail("closing adapted linker query");
    return 0;
}
