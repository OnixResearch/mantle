/* Source-owned GCC -### argument splitter. This program never launches a child.
 * GCC remains the authority for specs expansion. Every emitted image must match
 * the caller's exact expected phase image; in particular collect2 is NOT ld.
 * Usage: helper LOG IMAGE MAP RESPONSE [IMAGE MAP RESPONSE ...]
 * MAP consists of normalized absolute SOURCE<TAB>DESTINATION path prefixes.
 * Each phase has its own map, so an intermediate is an output in its producer
 * phase and an input in its consumer phase. The caller separately binds every
 * actual phase image as an individual Produced executable.
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define LOG_MAX (16u * 1024u * 1024u)
#define MAP_MAX (1024u * 1024u)
#define WORD_MAX 65536u
#define ARGS_MAX 16384u
#define MAPS_MAX 4096u
#define PHASES_MAX 64u
#define ENV_MAX (64u * 1024u * 1024u)
#define ENV_ITEMS_MAX 16384u

static char log_bytes[LOG_MAX + 1];
static char map_bytes[MAP_MAX + 1];
static char relocated[WORD_MAX + 1];
static char *arguments[ARGS_MAX];
struct mapping { char *source; char *destination; char *runtime; size_t length; };
static struct mapping mappings[MAPS_MAX];
static size_t mapping_count;
static const char *relocated_runtime;
static const char *argument_data_name = "gcc-phase-data";
static const char *relocated_workspace;
static char workspace_root_bytes[WORD_MAX + 1];
static const char *workspace_root(const char *path);
static char environment_bytes[ENV_MAX + 1];
static char overlay_bytes[ENV_MAX + 1];
struct json_span { const char *start; const char *end; };
static struct json_span source_arguments;
static struct json_span source_argv0;
static struct json_span source_executable_path;
static struct json_span source_cwd;
static struct json_span source_workspace_mounts;
static struct json_span source_readonly_roots;
static struct json_span source_process_attributes;
static struct json_span source_descriptors;
static struct json_span overlay_cwd;
struct environment_item { struct json_span key; struct json_span value; };
static struct environment_item base_environment[ENV_ITEMS_MAX];
static struct environment_item overlay_environment[ENV_ITEMS_MAX];
static size_t base_environment_count;
static const char *const driver_keys[] = { "\"COLLECT_GCC\"", "\"COLLECT_GCC_OPTIONS\"", "\"COLLECT_LTO_WRAPPER\"", "\"COMPILER_PATH\"", "\"LIBRARY_PATH\"" };
static const char *driver_values[5];

static void capture_driver_metadata(const char *line)
{
    size_t i;
    for (i = 0; i < 5; ++i) {
        size_t n = strlen(driver_keys[i]) - 2;
        if (!strncmp(line, driver_keys[i] + 1, n) && line[n] == '=') driver_values[i] = line + n + 1;
    }
}

static void fail(const char *message)
{
    fprintf(stderr, "native GCC phases: %s\n", message);
    exit(1);
}

static size_t load(const char *path, char *bytes, size_t bound)
{
    FILE *file = fopen(path, "rb");
    size_t count;
    int next;
    if (!file) fail("cannot open source input");
    count = fread(bytes, 1, bound, file);
    if (ferror(file)) fail("cannot read source input");
    next = fgetc(file);
    if (next != EOF || ferror(file)) fail("source input exceeds bound");
    if (fclose(file)) fail("cannot close source input");
    if (memchr(bytes, 0, count)) fail("NUL in source input");
    bytes[count] = 0;
    return count;
}

static int normalized(const char *path, int allow_trailing_slash)
{
    const char *component;
    const char *cursor;
    if (*path != '/' || path[1] == 0) return 0;
    component = path + 1;
    for (cursor = component;; ++cursor) {
        unsigned char byte = (unsigned char)*cursor;
        if (byte && (byte <= 32 || byte == 127 || byte == '\\')) return 0;
        if (!byte || byte == '/') {
            size_t length = (size_t)(cursor - component);
            if (!byte && !length && allow_trailing_slash && cursor > path + 1 && cursor[-1] == '/') return 1;
            if (!length || (length == 1 && component[0] == '.') ||
                (length == 2 && component[0] == '.' && component[1] == '.')) return 0;
            if (!byte) return 1;
            component = cursor + 1;
        }
    }
}

static void read_map(const char *path)
{
    char *cursor = map_bytes;
    load(path, map_bytes, MAP_MAX);
    mapping_count = 0;
    while (*cursor) {
        char *line = cursor;
        char *tab;
        char *runtime;
        char *end = strchr(cursor, '\n');
        size_t i;
        if (!end) fail("unterminated path mapping");
        *end = 0;
        cursor = end + 1;
        tab = strchr(line, '\t');
        if (!tab) fail("invalid path mapping");
        *tab++ = 0;
        runtime = strchr(tab, '\t');
        if (runtime) { *runtime++ = 0; if (strchr(runtime, '\t') || !normalized(runtime, 0)) fail("invalid runtime mapping root"); }
        if (!normalized(line, 0) || !normalized(tab, 0)) fail("non-normalized path mapping");
        if (runtime) {
            size_t n = strlen(runtime);
            if (strncmp(tab, runtime, n) || (tab[n] && tab[n] != '/')) fail("runtime mapping leaves its exact declared root");
        } else if (strncmp(tab, "/inputs/", 8) && strncmp(tab, "/outputs/", 9))
            fail("mapping destination is not an explicit leaf input/output/runtime");
        if (mapping_count == MAPS_MAX) fail("too many path mappings");
        for (i = 0; i < mapping_count; ++i)
            if (!strcmp(line, mappings[i].source)) fail("duplicate path mapping");
        mappings[mapping_count].source = line;
        mappings[mapping_count].destination = tab;
        mappings[mapping_count].runtime = runtime;
        mappings[mapping_count].length = strlen(line);
        ++mapping_count;
    }
}

/* GCC's -### print_argv uses shell quoting, not shell evaluation. Recognize
 * quote/escape syntax only. Operators, substitutions, response-file recursion,
 * unterminated quotes and physical multiline arguments are rejected.
 */
static size_t split(char *line)
{
    char *read = line;
    char *write = line;
    size_t count = 0;
    while (*read) {
        int quote = 0;
        while (*read == ' ' || *read == '\t') ++read;
        if (!*read) break;
        if (count == ARGS_MAX) fail("too many phase arguments");
        arguments[count++] = write;
        while (*read && (quote || (*read != ' ' && *read != '\t'))) {
            unsigned char byte = (unsigned char)*read++;
            if (byte < 32 || byte == 127) fail("control byte in phase argument");
            if (byte == '\\' && quote != '\'') {
                byte = (unsigned char)*read++;
                if (!byte || byte < 32 || byte == 127) fail("invalid phase escape");
                *write++ = (char)byte;
            } else if (byte == '\'' || byte == '"') {
                if (!quote) quote = byte;
                else if (quote == byte) quote = 0;
                else *write++ = (char)byte;
            } else {
                if (!quote && strchr("|;&<>`$()", byte)) fail("shell operator in phase arguments");
                *write++ = (char)byte;
            }
        }
        if (quote) fail("unterminated phase quoting");
        while (*read == ' ' || *read == '\t') ++read;
        *write++ = 0;
        if (strlen(arguments[count - 1]) > WORD_MAX) fail("oversized phase argument");
        if (arguments[count - 1][0] == '@') fail("nested response file in GCC phase");
    }
    return count;
}

static int metadata(const char *line)
{
    static const char *const prefixes[] = {
        "Using built-in specs.", "Reading specs from ", "COLLECT_GCC=",
        "COLLECT_LTO_WRAPPER=", "OFFLOAD_TARGET_NAMES=", "OFFLOAD_TARGET_DEFAULT=",
        "Target: ", "Configured with: ", "Thread model: ",
        "Supported LTO compression algorithms: ", "gcc version ",
        "COLLECT_GCC_OPTIONS=", "COMPILER_PATH=", "LIBRARY_PATH="
    };
    size_t i;
    if (!*line) return 1;
    for (i = 0; i < sizeof(prefixes) / sizeof(prefixes[0]); ++i)
        if (!strncmp(line, prefixes[i], strlen(prefixes[i]))) return 1;
    return 0;
}

static void append(char *output, size_t *used, const char *text, size_t length)
{
    if (length > WORD_MAX - *used) fail("relocated argument exceeds bound");
    memcpy(output + *used, text, length);
    *used += length;
    output[*used] = 0;
}

static const char *relocate(const char *word)
{
    const char *path = NULL;
    const char *cursor;
    size_t prefix = 0;
    size_t best = MAPS_MAX;
    size_t used = 0;
    size_t i;
    struct mapping workspace_mapping;
    const struct mapping *selected;
    const char *root;
    relocated_runtime = NULL;
    relocated_workspace = NULL;
    /* Absolute operands and attached GCC/binutils path options are the only
     * path-bearing forms accepted. Unknown forms fail instead of retaining a
     * preparation machine or a previous phase's mutable output directory.
     */
    if (word[0] == '/') path = word;
    else if (word[0] == '@' && word[1] == '/') path = word + 1;
    else if (word[0] == '-' && (word[1] == 'I' || word[1] == 'L' || word[1] == 'B') && word[2] == '/')
        path = word + 2;
    else if ((cursor = strchr(word, '=')) && cursor[1] == '/') path = cursor + 1;
    if (!path) {
        if (strchr(word, '/')) fail("unmapped relative or compound path operand");
        return word;
    }
    for (cursor = path; *cursor; ++cursor)
        if ((unsigned char)*cursor <= 32 || (unsigned char)*cursor == 127 || *cursor == '\\') fail("invalid phase path byte");
    prefix = (size_t)(path - word);
    for (i = 0; i < mapping_count; ++i) {
        size_t length = mappings[i].length;
        if (!strncmp(path, mappings[i].source, length) && (path[length] == 0 || path[length] == '/')) {
            if (best == MAPS_MAX || length > mappings[best].length) best = i;
        }
    }
    root = workspace_root(path);
    if (root) {
        workspace_mapping.source = workspace_mapping.destination = (char *)root;
        workspace_mapping.runtime = NULL;
        workspace_mapping.length = strlen(root);
        selected = &workspace_mapping;
        relocated_workspace = root;
    } else {
        if (best == MAPS_MAX) fail("phase operand lacks an explicit path mapping");
        selected = &mappings[best];
    }
    relocated_runtime = selected->runtime;
    append(relocated, &used, word, prefix);
    append(relocated, &used, selected->destination, strlen(selected->destination));
    {
        size_t boundary = used;
        cursor = path + selected->length;
        while (*cursor) {
            const char *component;
            size_t n;
            while (*cursor == '/') ++cursor;
            component = cursor;
            while (*cursor && *cursor != '/') ++cursor;
            n = (size_t)(cursor - component);
            if (!n || (n == 1 && component[0] == '.')) continue;
            if (n == 2 && component[0] == '.' && component[1] == '.') {
                if (used == boundary) fail("phase path escapes its declared mapping");
                while (used > boundary && relocated[used - 1] != '/') --used;
                if (used > boundary) --used;
                relocated[used] = 0;
            } else { append(relocated, &used, "/", 1); append(relocated, &used, component, n); }
        }
        if (path[strlen(path) - 1] == '/') append(relocated, &used, "/", 1);
    }
    return relocated;
}

static void emit_response(const char *path, size_t count)
{
    size_t i;
    FILE *file = fopen(path, "wbx");
    if (!file) fail("cannot exclusively create phase response");
    for (i = 1; i < count; ++i) {
        const char *word = relocate(arguments[i]);
        const char *cursor;
        if (fputc('"', file) == EOF) fail("cannot write phase response");
        for (cursor = word; *cursor; ++cursor) {
            if ((*cursor == '"' || *cursor == '\\') && fputc('\\', file) == EOF)
                fail("cannot escape phase response");
            if (fputc((unsigned char)*cursor, file) == EOF) fail("cannot write phase response");
        }
        if (fputs("\"\n", file) == EOF) fail("cannot terminate phase response argument");
    }
    if (fclose(file)) fail("cannot close phase response");
}

/* These frames have already crossed the canonical RecipeInvocation boundary.
 * Still parse bounded structure instead of searching for a quoted key inside a
 * program string. Values are copied byte-for-byte; no environment is frozen. */
static const char *json_string_end(const char *p)
{
    if (*p++ != '"') fail("expected JSON string");
    while (*p && *p != '"') {
        unsigned char c = (unsigned char)*p++;
        if (c < 32) fail("control byte in JSON string");
        if (c == '\\') {
            c = (unsigned char)*p++;
            if (!c || !strchr("\"\\/bfnrtu", c)) fail("invalid JSON escape");
            if (c == 'u') { unsigned i; for (i = 0; i < 4; ++i) {
                c = (unsigned char)*p++;
                if (!c || !((c >= '0' && c <= '9') || (c >= 'a' && c <= 'f') || (c >= 'A' && c <= 'F'))) fail("invalid JSON unicode escape");
            } }
        }
    }
    if (*p != '"') fail("unterminated JSON string");
    return p + 1;
}
static const char *json_value_end(const char *p, unsigned depth)
{
    char close;
    if (depth > 64) fail("JSON frame nesting exceeds bound");
    if (*p == '"') return json_string_end(p);
    if (*p == '[' || *p == '{') {
        close = *p++ == '[' ? ']' : '}';
        if (*p == close) return p + 1;
        for (;;) {
            if (close == '}') { p = json_string_end(p); if (*p++ != ':') fail("JSON object separator"); }
            p = json_value_end(p, depth + 1);
            if (*p == close) return p + 1;
            if (*p++ != ',') fail("JSON collection separator");
        }
    }
    { const char *start = p;
      while ((*p >= '0' && *p <= '9') || *p == '-' || *p == '+' || *p == '.' || *p == 'e' || *p == 'E' || (*p >= 'a' && *p <= 'z')) ++p;
      if (p == start) fail("invalid JSON scalar");
      return p;
    }
}
static size_t read_environment(const char *path, char *bytes, struct environment_item *items)
{
    const char *p;
    size_t count = 0;
    if (bytes == environment_bytes) { source_cwd.start = NULL; source_workspace_mounts.start = NULL; source_process_attributes.start = NULL; source_executable_path.start = NULL; source_descriptors.start = NULL; source_argv0.start = NULL; source_readonly_roots.start = NULL; }
    else overlay_cwd.start = NULL;
    load(path, bytes, ENV_MAX);
    p = bytes;
    if (strncmp(p, "{\"arguments\":", 13)) fail("environment source is not a canonical invocation");
    if (bytes == environment_bytes) source_arguments.start = p + 13;
    p = json_value_end(p + 13, 0);
    if (bytes == environment_bytes) source_arguments.end = p;
    if (strncmp(p, ",\"environment\":{", 16)) fail("invocation environment field is missing");
    p += 16;
    if (*p != '}') for (;;) {
        if (count == ENV_ITEMS_MAX) fail("too many original environment entries");
        items[count].key.start = p; p = json_string_end(p); items[count].key.end = p;
        if (*p++ != ':') fail("environment separator");
        items[count].value.start = p; p = json_value_end(p, 0); items[count].value.end = p;
        ++count;
        if (*p == '}') break;
        if (*p++ != ',') fail("environment entry separator");
    }
    if (*p++ != '}') fail("missing invocation environment close");
    while (*p == ',') {
        struct json_span *field;
        size_t prefix;
        if (!strncmp(p, ",\"cwd\":", 7)) {
            field = bytes == environment_bytes ? &source_cwd : &overlay_cwd;
            prefix = 7;
        } else if (!strncmp(p, ",\"argv0\":", 9)) {
            if (bytes != environment_bytes) fail("phase overlay cannot override actual source argv0");
            field = &source_argv0; prefix = 9;
        } else if (!strncmp(p, ",\"executable_path\":", 19)) {
            /* A selected cc1/as/ld phase must not inherit the driver's image. */
            if (bytes != environment_bytes) fail("phase overlay cannot select a driver executable path");
            field = &source_executable_path; prefix = 19;
        } else if (!strncmp(p, ",\"workspace_mounts\":", 20)) {
            if (bytes != environment_bytes) fail("phase overlay cannot override workspace mounts");
            field = &source_workspace_mounts; prefix = 20;
        } else if (!strncmp(p, ",\"readonly_input_roots\":", 24)) {
            if (bytes != environment_bytes) fail("phase overlay cannot override authenticated readonly input roots");
            field = &source_readonly_roots; prefix = 24;
        } else if (!strncmp(p, ",\"process_attributes\":", 22)) {
            if (bytes != environment_bytes) fail("phase overlay cannot override original process attributes");
            field = &source_process_attributes; prefix = 22;
        } else if (!strncmp(p, ",\"descriptors\":", 15)) {
            if (bytes != environment_bytes) fail("phase overlay cannot override original descriptor scope");
            field = &source_descriptors; prefix = 15;
        } else fail("unknown original invocation field");
        if (field->start) fail("duplicate original invocation field");
        p += prefix; field->start = p; p = json_value_end(p, 0); field->end = p;
    }
    if (*p++ != '}' || *p) fail("trailing invocation frame bytes");
    return count;
}
static void json_string(FILE *f, const char *p, size_t length)
{
    size_t i;
    fputc('"', f);
    for (i = 0; i < length; ++i) {
        unsigned char c = (unsigned char)p[i];
        switch (c) {
        case '"': fputs("\\\"", f); break;
        case '\\': fputs("\\\\", f); break;
        case '\b': fputs("\\b", f); break;
        case '\f': fputs("\\f", f); break;
        case '\n': fputs("\\n", f); break;
        case '\r': fputs("\\r", f); break;
        case '\t': fputs("\\t", f); break;
        default: if (c < 32) fprintf(f, "\\u%04x", (unsigned)c); else fputc(c, f);
        }
    }
    fputc('"', f);
}
static void data_part(FILE *json, FILE *data, const char *bytes, size_t length, unsigned long long *offset)
{
    if (length && fwrite(bytes, 1, length, data) != length) fail("cannot write original phase data");
    fprintf(json, "{\"kind\":\"text_slice\",\"name\":\"%s\",\"offset_bytes\":%llu,\"length_bytes\":%llu}", argument_data_name, *offset, (unsigned long long)length);
    *offset += length;
}
static void invocation_parts(FILE *json, FILE *data, const char *original, unsigned long long *offset, int macro_data)
{
    const char *word;
    const char *path = NULL, *member, *slash;
    size_t prefix = 0, label_length;
    int output_path = 0;
    if (!macro_data && (!strncmp(original, "-Wl,", 4) || !strncmp(original, "-Wa,", 4) || !strncmp(original, "-Wp,", 4))) {
        const char *part = original + 4;
        data_part(json, data, original, 4, offset);
        for (;;) {
            const char *end = strchr(part, ',');
            size_t n = end ? (size_t)(end - part) : strlen(part);
            char component[WORD_MAX + 1];
            if (n > WORD_MAX) fail("source compound argument exceeds bound");
            memcpy(component, part, n); component[n] = 0;
            fputc(',', json);
            invocation_parts(json, data, component, offset, !strncmp(component, "-D", 2) || !strncmp(component, "-U", 2));
            if (!end) break;
            fputs(",{\"kind\":\"literal\",\"text\":\",\"}", json); part = end + 1;
        }
        return;
    }
    if (!macro_data) {
        const char *absolute = original[0] == '/' ? original : strchr(original, '=');
        if (absolute && absolute != original) absolute = absolute[1] == '/' ? absolute + 1 : NULL;
        if (absolute && strchr(absolute, ':')) {
            const char *part = absolute;
            if (absolute != original) { data_part(json, data, original, (size_t)(absolute - original), offset); fputc(',', json); }
            for (;;) {
                const char *end = strchr(part, ':');
                size_t n = end ? (size_t)(end - part) : strlen(part);
                char component[WORD_MAX + 1];
                if (n > WORD_MAX) fail("source path-list argument exceeds bound");
                memcpy(component, part, n); component[n] = 0;
                invocation_parts(json, data, component, offset, 0);
                if (!end) break;
                fputs(",{\"kind\":\"literal\",\"text\":\":\"},", json); part = end + 1;
            }
            return;
        }
    }
    word = macro_data ? original : relocate(original);
    if (!macro_data) {
        if (word[0] == '/') path = word;
        else if (word[0] == '@' && word[1] == '/') path = word + 1;
        else if (word[0] == '-' && (word[1] == 'I' || word[1] == 'L' || word[1] == 'B') && word[2] == '/') path = word + 2;
        else { const char *equal = strchr(word, '='); if (equal && equal[1] == '/') path = equal + 1; }
    }
    if (!path) data_part(json, data, word, strlen(word), offset);
    else {
        prefix = (size_t)(path - word);
        if (relocated_runtime || relocated_workspace) {
            const char *root = relocated_workspace ? relocated_workspace : relocated_runtime;
            size_t root_length = strlen(root);
            const char *relative = path + root_length;
            if (prefix) { data_part(json, data, word, prefix, offset); fputc(',', json); }
            fputs(relocated_workspace ? "{\"kind\":\"workspace\",\"mount\":" : "{\"kind\":\"runtime\",\"mount\":", json);
            json_string(json, root, root_length); fputs(",\"path\":", json);
            if (*relative == '/') ++relative;
            { size_t n = strlen(relative); int directory = path[strlen(path) - 1] == '/';
              json_string(json, relative, n && directory ? n - 1 : n); fputc('}', json);
              if (directory) fputs(",{\"kind\":\"directory_separator\"}", json);
            }
            return;
        }
        if (!strncmp(path, "/inputs/", 8)) member = path + 8;
        else if (!strncmp(path, "/outputs/", 9)) { member = path + 9; output_path = 1; }
        else fail("typed invocation operand leaves declared input/output roots");
        slash = strchr(member, '/'); label_length = slash ? (size_t)(slash - member) : strlen(member);
        if (!label_length) fail("empty typed invocation label");
        if (prefix) { data_part(json, data, word, prefix, offset); fputc(',', json); }
        fputs(output_path ? "{\"kind\":\"output\",\"name\":" : "{\"kind\":\"input\",\"name\":", json);
        json_string(json, member, label_length); fputs(",\"path\":", json);
        { const char *relative = slash ? slash + 1 : ""; size_t n = strlen(relative);
          int directory = path[strlen(path) - 1] == '/';
          json_string(json, relative, n && directory ? n - 1 : n); fputc('}', json);
          if (directory) fputs(",{\"kind\":\"directory_separator\"}", json);
        }
    }
}
static void invocation_word(FILE *json, FILE *data, const char *original, unsigned long long *offset, int macro_data)
{
    fputs("{\"parts\":[", json);
    invocation_parts(json, data, original, offset, macro_data);
    fputs("]}", json);
}
static void driver_environment_word(FILE *json, FILE *data, size_t index, unsigned long long *offset)
{
    const char *value = driver_values[index];
    fputs("{\"parts\":[", json);
    if (index < 3) invocation_parts(json, data, value, offset, index == 1);
    else for (;;) {
        const char *end = strchr(value, ':');
        size_t n = end ? (size_t)(end - value) : strlen(value);
        if (n > WORD_MAX) fail("oversized original driver path list member");
        if (n) { char part[WORD_MAX + 1]; memcpy(part, value, n); part[n] = 0; invocation_parts(json, data, part, offset, 0); }
        else data_part(json, data, "", 0, offset);
        if (!end) break;
        fputs(",{\"kind\":\"literal\",\"text\":\":\"},", json);
        value = end + 1;
    }
    fputs("]}", json);
}
static const char *json_path_string(const char *p, char *out, size_t capacity, size_t *used)
{
    if (*p++ != '"') fail("typed environment path is not a JSON string");
    while (*p && *p != '"') {
        unsigned char c = (unsigned char)*p++;
        if (c == '\\') {
            c = (unsigned char)*p++;
            switch (c) {
            case '"': case '\\': break;
            case 'b': c = '\b'; break; case 'f': c = '\f'; break;
            case 'n': c = '\n'; break; case 'r': c = '\r'; break; case 't': c = '\t'; break;
            default: fail("noncanonical escaped typed environment path");
            }
        }
        if (*used >= capacity - 1) fail("typed environment path exceeds word bound");
        out[(*used)++] = (char)c;
    }
    if (*p++ != '"') fail("unterminated typed environment path");
    out[*used] = 0; return p;
}
/* Preserve the declared original mutable root, rather than inventing an input
 * or rewriting it into a compiler output path. The consumer validates and
 * materializes the same typed workspace declaration before executing a leaf. */
static const char *workspace_root(const char *path)
{
    const char *p;
    if (!source_workspace_mounts.start) return NULL;
    p = source_workspace_mounts.start;
    if (*p++ != '{') fail("source workspace map is not an object");
    while (*p != '}') {
        size_t used = 0;
        p = json_path_string(p, workspace_root_bytes, sizeof(workspace_root_bytes), &used);
        if (*p++ != ':' || !normalized(workspace_root_bytes, 0)) fail("invalid source workspace root");
        if (!strncmp(path, workspace_root_bytes, used) && (!path[used] || path[used] == '/')) return workspace_root_bytes;
        p = json_value_end(p, 0);
        if (*p == '}') break;
        if (*p++ != ',') fail("source workspace map separator");
    }
    return NULL;
}
static void environment_word(FILE *json, FILE *data, struct json_span word, unsigned long long *offset)
{
    const char *p = word.start;
    int first = 1;
    if (strncmp(p, "{\"parts\":[", 10)) fail("original environment word structure");
    p += 10; fputs("{\"parts\":[", json);
    while (*p != ']') {
        const char *end = json_value_end(p, 0), *value = NULL, *prefix = NULL;
        char path[WORD_MAX + 1]; size_t used = 0, i; int mapped = 0;
        if (!strncmp(p, "{\"kind\":\"input\",\"name\":", 23)) { value = p + 23; prefix = "/inputs/"; }
        else if (!strncmp(p, "{\"kind\":\"output\",\"name\":", 24)) { value = p + 24; prefix = "/outputs/"; }
        else if (!strncmp(p, "{\"kind\":\"runtime\",\"mount\":", 26)) { value = p + 26; prefix = ""; }
        if (value) {
            used = strlen(prefix); memcpy(path, prefix, used);
            value = json_path_string(value, path, sizeof path, &used);
            if (strncmp(value, ",\"path\":", 8)) fail("typed environment member field");
            value += 8;
            if (value[0] != '"') fail("typed environment member string");
            if (value[1] != '"') { if (used >= WORD_MAX) fail("typed environment member bound"); path[used++] = '/'; }
            value = json_path_string(value, path, sizeof path, &used);
            if (*value != '}' || value + 1 != end) fail("typed environment member trailing bytes");
            for (i = 0; i < mapping_count; ++i) {
                size_t n = mappings[i].length;
                if (!strncmp(path, mappings[i].source, n) && (!path[n] || path[n] == '/')) { mapped = 1; break; }
            }
        }
        if (!first) fputc(',', json); first = 0;
        if (mapped) invocation_parts(json, data, path, offset, 0);
        else fwrite(p, 1, (size_t)(end - p), json);
        p = end;
        if (*p == ',') ++p; else if (*p != ']') fail("original environment part separator");
    }
    if (p + 2 != word.end || p[1] != '}') fail("original environment word trailing bytes");
    fputs("]}", json);
}
static int key_compare(struct json_span a, struct json_span b)
{
    size_t an = (size_t)(a.end - a.start), bn = (size_t)(b.end - b.start);
    int comparison = memcmp(a.start, b.start, an < bn ? an : bn);
    return comparison ? comparison : (an > bn) - (an < bn);
}
static void emit_invocation(const char *path, const char *data_path, const char *overlay_path, size_t count)
{
    size_t i, base = 0, overlay = 0, derived = 0, overlays = read_environment(overlay_path, overlay_bytes, overlay_environment);
    unsigned long long offset = 0;
    int first = 1;
    FILE *file = fopen(path, "wbx"), *data = fopen(data_path, "wbx");
    if (!file || !data) fail("cannot exclusively create typed phase invocation");
    fputs("{\"arguments\":[", file);
    for (i = 1; i < count; ++i) {
        int macro_data = !strncmp(arguments[i], "-D", 2) || !strncmp(arguments[i], "-U", 2) || !strncmp(arguments[i], "-fmacro-prefix-map=", 19) || !strncmp(arguments[i], "-fdebug-prefix-map=", 19) || !strncmp(arguments[i], "-ffile-prefix-map=", 18) || (i > 1 && (!strcmp(arguments[i - 1], "-D") || !strcmp(arguments[i - 1], "-U")));
        if (i != 1) fputc(',', file);
        invocation_word(file, data, arguments[i], &offset, macro_data);
    }
    fputs("],\"environment\":{", file);
    while (base < base_environment_count || overlay < overlays || derived < 5) {
        struct environment_item item;
        int comparison;
        while (derived < 5 && !driver_values[derived]) ++derived;
        if (derived < 5) {
            struct json_span key = { driver_keys[derived], driver_keys[derived] + strlen(driver_keys[derived]) };
            int bc = base == base_environment_count ? -1 : key_compare(key, base_environment[base].key);
            int oc = overlay == overlays ? -1 : key_compare(key, overlay_environment[overlay].key);
            if (bc <= 0 && oc <= 0) {
                if (!oc) fail("phase overlay shadows actual driver metadata");
                if (!bc) ++base;
                if (!first) fputc(',', file); first = 0;
                fputs(driver_keys[derived], file); fputc(':', file);
                driver_environment_word(file, data, derived++, &offset);
                continue;
            }
        }
        if (base == base_environment_count && overlay == overlays) break;
        comparison = base == base_environment_count ? 1 : overlay == overlays ? -1 : key_compare(base_environment[base].key, overlay_environment[overlay].key);
        if (!comparison) fail("phase environment shadows original source policy");
        item = comparison < 0 ? base_environment[base++] : overlay_environment[overlay++];
        if (!first) fputc(',', file); first = 0;
        fwrite(item.key.start, 1, (size_t)(item.key.end - item.key.start), file); fputc(':', file);
        environment_word(file, data, item.value, &offset);
    }
    fputc('}', file);
    if (source_cwd.start || overlay_cwd.start) {
        fputs(",\"cwd\":", file);
        environment_word(file, data, source_cwd.start ? source_cwd : overlay_cwd, &offset);
    }
    /* GCC's actual captured argument zero belongs to THIS selected phase,
     * not the driver frame and not a fabricated executable basename. */
    fputs(",\"argv0\":", file);
    invocation_word(file, data, arguments[0], &offset, 0);
    if (source_workspace_mounts.start) {
        fputs(",\"workspace_mounts\":", file);
        fwrite(source_workspace_mounts.start, 1, (size_t)(source_workspace_mounts.end - source_workspace_mounts.start), file);
    }
    if (source_readonly_roots.start) {
        fputs(",\"readonly_input_roots\":", file);
        fwrite(source_readonly_roots.start, 1, (size_t)(source_readonly_roots.end - source_readonly_roots.start), file);
    }
    if (source_process_attributes.start) {
        fputs(",\"process_attributes\":", file);
        fwrite(source_process_attributes.start, 1, (size_t)(source_process_attributes.end - source_process_attributes.start), file);
    }
    if (source_descriptors.start) {
        fputs(",\"descriptors\":", file);
        fwrite(source_descriptors.start, 1, (size_t)(source_descriptors.end - source_descriptors.start), file);
    }
    fputc('}', file);
    if (ferror(file) || ferror(data) || fclose(file) || fclose(data)) fail("cannot finish typed phase invocation");
}

/* GCC 10.5 libiberty/argv.c buildargv/expandargv grammar: whitespace outside
 * quotes separates words; backslash quotes the next byte inside either quote
 * style too. Empty quoted arguments survive; an all-whitespace file is empty.
 * This consumes real source-produced response bytes, never a replacement flag
 * table. Source templates supply the explicit protected driver's -### query. */
static int response_space(unsigned char byte)
{
    return byte == ' ' || byte == '\t' || byte == '\n' || byte == '\r' || byte == '\f' || byte == '\v';
}
static size_t source_response_arguments(char *bytes)
{
    char *read = bytes, *write = bytes;
    size_t count = 0;
    while (*read && response_space((unsigned char)*read)) ++read;
    while (*read) {
        int single = 0, double_quote = 0, escape = 0;
        char *argument = write;
        if (count == ARGS_MAX) fail("too many original source response arguments");
        arguments[count++] = argument;
        while (*read) {
            unsigned char byte = (unsigned char)*read;
            if (response_space(byte) && !single && !double_quote && !escape) break;
            ++read;
            if (escape) { escape = 0; *write++ = (char)byte; }
            else if (byte == '\\') escape = 1;
            else if (single) { if (byte == '\'') single = 0; else *write++ = (char)byte; }
            else if (double_quote) { if (byte == '"') double_quote = 0; else *write++ = (char)byte; }
            else if (byte == '\'') single = 1;
            else if (byte == '"') double_quote = 1;
            else *write++ = (char)byte;
        }
        while (*read && response_space((unsigned char)*read)) ++read;
        *write++ = 0;
        if ((size_t)(write - argument) > WORD_MAX + 1) fail("original response argument exceeds bound");
        if (*argument == '@') fail("nested source response requires its own admitted source expansion");
    }
    return count;
}
static int source_response(int argc, char **argv)
{
    FILE *file, *data;
    size_t count, i;
    unsigned long long offset = 0;
    int first;
    if (argc != 7) fail("usage: helper --source-response TEMPLATE RESPONSE MAP INVOCATION DATA");
    argument_data_name = "gcc-specs-data";
    base_environment_count = read_environment(argv[2], environment_bytes, base_environment);
    read_map(argv[4]);
    load(argv[3], log_bytes, LOG_MAX);
    count = source_response_arguments(log_bytes);
    if (!source_arguments.start || *source_arguments.start != '[' || source_arguments.end[-1] != ']') fail("source response template argument array");
    file = fopen(argv[5], "wbx"); data = fopen(argv[6], "wbx");
    if (!file || !data) fail("cannot exclusively create source response invocation");
    fputs("{\"arguments\":[", file);
    first = source_arguments.end - source_arguments.start == 2;
    fwrite(source_arguments.start + 1, 1, (size_t)(source_arguments.end - source_arguments.start - 2), file);
    for (i = 0; i < count; ++i) {
        int macro_data = !strncmp(arguments[i], "-D", 2) || !strncmp(arguments[i], "-U", 2) || !strncmp(arguments[i], "-fmacro-prefix-map=", 19) || !strncmp(arguments[i], "-fdebug-prefix-map=", 19) || !strncmp(arguments[i], "-ffile-prefix-map=", 18) || (i && (!strcmp(arguments[i - 1], "-D") || !strcmp(arguments[i - 1], "-U")));
        if (!first) fputc(',', file); first = 0;
        invocation_word(file, data, arguments[i], &offset, macro_data);
    }
    fputs("],\"environment\":{", file);
    for (i = 0; i < base_environment_count; ++i) {
        if (i) fputc(',', file);
        fwrite(base_environment[i].key.start, 1, (size_t)(base_environment[i].key.end - base_environment[i].key.start), file);
        fputc(':', file); environment_word(file, data, base_environment[i].value, &offset);
    }
    fputc('}', file);
    if (source_cwd.start) { fputs(",\"cwd\":", file); environment_word(file, data, source_cwd, &offset); }
    if (source_argv0.start) { fputs(",\"argv0\":", file); environment_word(file, data, source_argv0, &offset); }
    if (source_executable_path.start) { fputs(",\"executable_path\":", file); environment_word(file, data, source_executable_path, &offset); }
    if (source_workspace_mounts.start) { fputs(",\"workspace_mounts\":", file); fwrite(source_workspace_mounts.start, 1, (size_t)(source_workspace_mounts.end - source_workspace_mounts.start), file); }
    if (source_readonly_roots.start) { fputs(",\"readonly_input_roots\":", file); fwrite(source_readonly_roots.start, 1, (size_t)(source_readonly_roots.end - source_readonly_roots.start), file); }
    if (source_process_attributes.start) { fputs(",\"process_attributes\":", file); fwrite(source_process_attributes.start, 1, (size_t)(source_process_attributes.end - source_process_attributes.start), file); }
    if (source_descriptors.start) { fputs(",\"descriptors\":", file); fwrite(source_descriptors.start, 1, (size_t)(source_descriptors.end - source_descriptors.start), file); }
    fputc('}', file);
    if (ferror(file) || ferror(data) || fclose(file) || fclose(data)) fail("cannot finish source response invocation");
    return 0;
}

int main(int argc, char **argv)
{
    char *cursor = log_bytes;
    size_t phase = 0;
    size_t expected;
    int typed = argc > 2 && !strcmp(argv[2], "--invocations");
    size_t stride = typed ? 5 : 3, start = typed ? 4 : 2;
    if (argc > 1 && !strcmp(argv[1], "--source-response")) return source_response(argc, argv);
    if ((size_t)argc < start + stride || ((size_t)argc - start) % stride)
        fail("usage: helper LOG [--invocations ENVIRONMENT] IMAGE MAP RESPONSE [DATA OVERLAY] ...");
    expected = ((size_t)argc - start) / stride;
    if (expected > PHASES_MAX) fail("too many expected phases");
    load(argv[1], log_bytes, LOG_MAX);
    if (typed) base_environment_count = read_environment(argv[3], environment_bytes, base_environment);
    while (*cursor) {
        char *line = cursor;
        char *end = strchr(cursor, '\n');
        size_t count;
        if (!end) fail("unterminated GCC log line");
        *end = 0;
        cursor = end + 1;
        if (metadata(line)) { capture_driver_metadata(line); continue; }
        if (*line != ' ' && *line != '\t') fail("unknown GCC diagnostic or phase record");
        count = split(line);
        if (!count || phase == expected) fail("unexpected or empty GCC phase");
        if (strcmp(arguments[0], argv[start + phase * stride])) fail("GCC phase image differs from exact declared image");
        {
            const char *image = strrchr(arguments[0], '/');
            image = image ? image + 1 : arguments[0];
            if (strcmp(image, "cc1") && strcmp(image, "cc1plus") &&
                strcmp(image, "as") && strcmp(image, "ld") && strcmp(image, "collect2"))
                fail("phase image is not a declared compiler/assembler/linker/collect2-query leaf");
        }
        read_map(argv[start + 1 + phase * stride]);
        if (typed) emit_invocation(argv[start + 2 + phase * stride], argv[start + 3 + phase * stride], argv[start + 4 + phase * stride], count);
        else emit_response(argv[start + 2 + phase * stride], count);
        ++phase;
    }
    if (phase != expected) fail("GCC phase inventory is incomplete");
    return 0;
}
