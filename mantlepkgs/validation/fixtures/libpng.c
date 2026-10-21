#include <png.h>
#include <stdio.h>
#include <string.h>

int main(void) {
    unsigned char pixels[] = {255, 0, 0, 255, 0, 255, 0, 128};
    unsigned char decoded[sizeof(pixels)], encoded[1024];
    png_alloc_size_t length = sizeof(encoded);
    png_image writer = {0}, reader = {0}, invalid = {0};
    writer.version = PNG_IMAGE_VERSION;
    writer.width = 2;
    writer.height = 1;
    writer.format = PNG_FORMAT_RGBA;
    if (!png_image_write_to_memory(&writer, encoded, &length, 0, pixels, 0, NULL))
        return 1;
    reader.version = PNG_IMAGE_VERSION;
    if (!png_image_begin_read_from_memory(&reader, encoded, length))
        return 2;
    if (reader.width != 2 || reader.height != 1)
        return 3;
    reader.format = PNG_FORMAT_RGBA;
    if (!png_image_finish_read(&reader, NULL, decoded, 0, NULL))
        return 4;
    if (memcmp(pixels, decoded, sizeof(pixels)) != 0)
        return 5;
    png_image_free(&reader);
    encoded[0] ^= 0xff;
    invalid.version = PNG_IMAGE_VERSION;
    if (png_image_begin_read_from_memory(&invalid, encoded, length))
        return 6;
    png_image_free(&invalid);
    puts("libpng: RGBA round-trip and invalid-signature rejection passed");
    return 0;
}
