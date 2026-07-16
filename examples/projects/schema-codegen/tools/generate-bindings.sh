#!/bin/sh
set -eu

ARGUMENT_COUNT_REQUIRED=2
SUPPORTED_SCHEMA_VERSION=1
EXIT_USAGE=64
EXIT_SCHEMA=65

if [ "$#" -ne "$ARGUMENT_COUNT_REQUIRED" ]; then
    echo 'usage: generate-bindings.sh <schema> <output-directory>' >&2
    exit "$EXIT_USAGE"
fi

schema_path="$1"
output_directory="$2"

read_field() {
    field_name="$1"
    field_value=$(grep -m 1 "^${field_name}=" "$schema_path" | cut -d= -f2- || true)
    printf '%s' "$field_value"
}

schema_version=$(read_field schema_version)
type_name=$(read_field type)
prefix=$(read_field prefix)
default_name=$(read_field default_name)

if [ "$schema_version" != "$SUPPORTED_SCHEMA_VERSION" ]; then
    echo "unsupported schema_version: $schema_version" >&2
    exit "$EXIT_SCHEMA"
fi
if [ "$type_name" != 'Greeting' ]; then
    echo "unsupported schema type: $type_name" >&2
    exit "$EXIT_SCHEMA"
fi
if [ -z "$prefix" ] || [ -z "$default_name" ]; then
    echo 'schema requires non-empty prefix and default_name fields' >&2
    exit "$EXIT_SCHEMA"
fi
if printf '%s%s' "$prefix" "$default_name" | grep '[^A-Za-z0-9 _-]' >/dev/null; then
    echo 'schema text fields contain unsupported characters' >&2
    exit "$EXIT_SCHEMA"
fi

mkdir -p "$output_directory/c" "$output_directory/rust"

cat > "$output_directory/c/greeting.h" << 'HEADER'
#ifndef GENERATED_GREETING_H
#define GENERATED_GREETING_H

#include <stddef.h>

typedef enum GeneratedGreetingStatus {
    GENERATED_GREETING_STATUS_OK = 0,
    GENERATED_GREETING_STATUS_INVALID_ARGUMENT = 1,
    GENERATED_GREETING_STATUS_BUFFER_TOO_SMALL = 2,
} GeneratedGreetingStatus;

GeneratedGreetingStatus generated_greeting_format(const char *name, char *output, size_t output_capacity);

#endif
HEADER

cat > "$output_directory/c/greeting.c" << SOURCE
#include "greeting.h"

#include <stdio.h>

static const char *const GENERATED_PREFIX = "$prefix";
static const char *const GENERATED_DEFAULT_NAME = "$default_name";

GeneratedGreetingStatus generated_greeting_format(const char *name, char *output, size_t output_capacity) {
    if (output == NULL || output_capacity == 0u) {
        return GENERATED_GREETING_STATUS_INVALID_ARGUMENT;
    }
    const char *selected_name = name;
    if (selected_name == NULL || selected_name[0] == '\\0') {
        selected_name = GENERATED_DEFAULT_NAME;
    }
    const int written = snprintf(output, output_capacity, "%s, %s!", GENERATED_PREFIX, selected_name);
    if (written < 0 || (size_t)written >= output_capacity) {
        output[0] = '\\0';
        return GENERATED_GREETING_STATUS_BUFFER_TOO_SMALL;
    }
    return GENERATED_GREETING_STATUS_OK;
}
SOURCE

cat > "$output_directory/rust/greeting.rs" << SOURCE
pub const PREFIX: &str = "$prefix";
pub const DEFAULT_NAME: &str = "$default_name";

pub fn render(name: Option<&str>) -> Result<String, &'static str> {
    let selected = name.filter(|value| !value.is_empty()).unwrap_or(DEFAULT_NAME);
    if selected.contains('\\n') || selected.contains('\\r') {
        return Err("name contains a line break");
    }
    Ok(format!("{}, {}!", PREFIX, selected))
}

#[cfg(test)]
mod tests {
    use super::render;

    #[test]
    fn renders_default_and_explicit_names() {
        assert_eq!(render(None).unwrap(), "Hello, World!");
        assert_eq!(render(Some("Mantle")).unwrap(), "Hello, Mantle!");
    }

    #[test]
    fn rejects_line_breaks() {
        assert_eq!(render(Some("bad\\nname")).unwrap_err(), "name contains a line break");
    }
}
SOURCE

printf '%s\n' "schema_version=$schema_version" "type=$type_name" "prefix=$prefix" "default_name=$default_name" > "$output_directory/bindings.manifest"
