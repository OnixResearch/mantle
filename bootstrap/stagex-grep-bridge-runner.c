#include <assert.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <sys/stat.h>

#define GREP_EXIT_MATCH 0
#define GREP_EXIT_NO_MATCH 1
#define GREP_EXIT_USAGE 2
#define GREP_ARGUMENT_COUNT_MAX 256
#define GREP_PATTERN_BYTES_MAX 4096
#define GREP_LINE_BYTES_MAX 65536
#define GREP_INPUT_BYTES_MAX 16777216

typedef struct {
    int quiet;
    int invert;
    const char *pattern;
    int file_index;
} grep_options;

static size_t bounded_length(const char *value, size_t bytes_max) {
    size_t index = 0;
    assert(value != NULL);
    assert(bytes_max > 0);
    while (index <= bytes_max && value[index] != '\0') {
        index++;
    }
    return index;
}

static int prefix_matches(const char *line, size_t line_len, const char *pattern, size_t pattern_len) {
    if (pattern_len > line_len) {
        return 0;
    }
    assert(line != NULL);
    assert(pattern != NULL);
    return memcmp(line, pattern, pattern_len) == 0;
}

static int suffix_matches(const char *line, size_t line_len, const char *pattern, size_t pattern_len) {
    if (pattern_len > line_len) {
        return 0;
    }
    assert(line != NULL);
    assert(pattern != NULL);
    return memcmp(line + line_len - pattern_len, pattern, pattern_len) == 0;
}

static int contains_pattern(const char *line, size_t line_len, const char *pattern, size_t pattern_len) {
    size_t offset = 0;
    if (pattern_len == 0) {
        return 1;
    }
    if (pattern_len > line_len) {
        return 0;
    }
    assert(line != NULL);
    assert(pattern != NULL);
    while (offset <= line_len - pattern_len) {
        if (memcmp(line + offset, pattern, pattern_len) == 0) {
            return 1;
        }
        offset++;
    }
    return 0;
}

static int line_matches(const char *line, size_t line_len, const char *pattern) {
    size_t pattern_len = bounded_length(pattern, GREP_PATTERN_BYTES_MAX);
    assert(line != NULL);
    assert(pattern != NULL);
    if (pattern_len > GREP_PATTERN_BYTES_MAX) {
        return -1;
    }
    if (pattern_len > 0 && pattern[0] == '^') {
        return prefix_matches(line, line_len, pattern + 1, pattern_len - 1);
    }
    if (pattern_len > 0 && pattern[pattern_len - 1] == '$') {
        return suffix_matches(line, line_len, pattern, pattern_len - 1);
    }
    return contains_pattern(line, line_len, pattern, pattern_len);
}

static int emit_matching_line(const char *line, size_t line_len, const grep_options *options, int *found) {
    int matched = line_matches(line, line_len, options->pattern);
    if (matched < 0) {
        return GREP_EXIT_USAGE;
    }
    if (options->invert) {
        matched = !matched;
    }
    if (!matched) {
        return GREP_EXIT_NO_MATCH;
    }
    *found = 1;
    if (options->quiet) {
        return GREP_EXIT_MATCH;
    }
    if (fwrite(line, 1, line_len, stdout) != line_len || fputc('\n', stdout) == EOF) {
        return GREP_EXIT_USAGE;
    }
    assert(*found == 1);
    assert(!options->quiet);
    return GREP_EXIT_NO_MATCH;
}

static int scan_stream(FILE *stream, const grep_options *options) {
    char line[GREP_LINE_BYTES_MAX];
    size_t line_len = 0;
    uint64_t input_bytes = 0;
    int found = 0;
    int byte = 0;
    assert(stream != NULL);
    assert(options != NULL);
    while ((byte = fgetc(stream)) != EOF) {
        if (input_bytes >= GREP_INPUT_BYTES_MAX) {
            return GREP_EXIT_USAGE;
        }
        input_bytes++;
        if (byte == '\n') {
            int result = emit_matching_line(line, line_len, options, &found);
            if (result == GREP_EXIT_USAGE || (result == GREP_EXIT_MATCH && options->quiet)) {
                return result;
            }
            line_len = 0;
        } else {
            if (line_len >= GREP_LINE_BYTES_MAX) {
                return GREP_EXIT_USAGE;
            }
            line[line_len++] = (char)byte;
        }
    }
    if (ferror(stream)) {
        return GREP_EXIT_USAGE;
    }
    assert(input_bytes <= GREP_INPUT_BYTES_MAX);
    assert(line_len <= GREP_LINE_BYTES_MAX);
    return found ? GREP_EXIT_MATCH : GREP_EXIT_NO_MATCH;
}

static int parse_options(int argc, char **argv, grep_options *options) {
    int index = 1;
    size_t pattern_len;
    if (argc <= 1 || argc > GREP_ARGUMENT_COUNT_MAX) {
        return GREP_EXIT_USAGE;
    }
    options->quiet = 0;
    options->invert = 0;
    options->pattern = NULL;
    while (index < argc) {
        const char *argument = argv[index];
        if (strcmp(argument, "-q") == 0) {
            options->quiet = 1;
            index++;
        } else if (strcmp(argument, "-v") == 0) {
            options->invert = 1;
            index++;
        } else if (strcmp(argument, "-i") == 0 || strcmp(argument, "-n") == 0 || strcmp(argument, "-E") == 0 ||
                   strcmp(argument, "-F") == 0) {
            index++;
        } else if (strcmp(argument, "-qi") == 0 || strcmp(argument, "-iq") == 0) {
            options->quiet = 1;
            index++;
        } else if (strcmp(argument, "-e") == 0) {
            index++;
            if (index >= argc) {
                return GREP_EXIT_USAGE;
            }
            options->pattern = argv[index++];
        } else {
            break;
        }
    }
    if (options->pattern == NULL) {
        if (index >= argc) {
            return GREP_EXIT_USAGE;
        }
        options->pattern = argv[index++];
    }
    pattern_len = bounded_length(options->pattern, GREP_PATTERN_BYTES_MAX);
    if (pattern_len == 0 || pattern_len > GREP_PATTERN_BYTES_MAX) {
        return GREP_EXIT_USAGE;
    }
    options->file_index = index;
    assert(options->pattern != NULL);
    assert(options->file_index <= argc);
    return GREP_EXIT_MATCH;
}

static int scan_files(int argc, char **argv, const grep_options *options) {
    int status = GREP_EXIT_NO_MATCH;
    int index = options->file_index;
    assert(index < argc);
    assert(options->pattern != NULL);
    while (index < argc) {
        struct stat metadata;
        FILE *stream;
        int result;
        if (stat(argv[index], &metadata) != 0 || !S_ISREG(metadata.st_mode)) {
            index++;
            continue;
        }
        stream = fopen(argv[index], "r");
        if (stream == NULL) {
            index++;
            continue;
        }
        result = scan_stream(stream, options);
        if (fclose(stream) != 0 || result == GREP_EXIT_USAGE) {
            return GREP_EXIT_USAGE;
        }
        if (result == GREP_EXIT_MATCH) {
            status = GREP_EXIT_MATCH;
            if (options->quiet) {
                return status;
            }
        }
        index++;
    }
    return status;
}

int main(int argc, char **argv) {
    grep_options options;
    int parsed;
    assert(argv != NULL);
    assert(argc >= 0);
    if (argc >= 2 && strcmp(argv[1], "--version") == 0) {
        fputs("grep (GNU grep) 2.4\n", stdout);
        return GREP_EXIT_MATCH;
    }
    if (argc >= 2 && strcmp(argv[1], "--help") == 0) {
        fputs("usage: grep OPTIONS PATTERN FILES\n", stdout);
        return GREP_EXIT_MATCH;
    }
    parsed = parse_options(argc, argv, &options);
    if (parsed != GREP_EXIT_MATCH) {
        return parsed;
    }
    if (options.file_index == argc) {
        return scan_stream(stdin, &options);
    }
    return scan_files(argc, argv, &options);
}
