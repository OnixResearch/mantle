/* Source boundary for authenticated Bison 2.3 src/output.c. The original
 * argument selection, muscle serialization and skeleton scanner stay in place.
 * Capture stops before create_subpipe; replay consumes one real m4 leaf. */
static void
mantle_bison23_byte (FILE *stream, int replay, unsigned char byte)
{
  if (replay)
    {
      if (fgetc (stream) != byte)
        error (EXIT_FAILURE, 0, "original Bison 2.3 request changed on replay");
    }
  else if (fputc (byte, stream) == EOF)
    error (EXIT_FAILURE, get_errno (), "writing original Bison 2.3 request");
}

static void
mantle_bison23_response (char const *const *argv, int replay)
{
  char const *path = getenv ("MANTLE_BISON_RESPONSE");
  FILE *stream;
  int i;
  if (!path || !*path)
    error (EXIT_FAILURE, 0, "missing original Bison 2.3 request boundary");
  {
    char const *image_path = getenv ("MANTLE_BISON_IMAGE");
    if (image_path)
      {
        FILE *image;
        unsigned char const *p;
        size_t size = 0;
        if (!*image_path || !argv[0] || !*argv[0])
          error (EXIT_FAILURE, 0, "missing original Bison 2.3 m4 image");
        image = xfopen (image_path, replay ? "r" : "w");
        for (p = (unsigned char const *) argv[0]; *p; ++p)
          {
            if (++size > 65536 || *p < 32 || *p == 127)
              error (EXIT_FAILURE, 0, "unrepresentable Bison 2.3 m4 image");
            mantle_bison23_byte (image, replay, *p);
          }
        mantle_bison23_byte (image, replay, '\n');
        if ((replay && fgetc (image) != EOF) || ferror (image))
          error (EXIT_FAILURE, 0, "invalid Bison 2.3 replay image");
        xfclose (image);
      }
  }
  stream = xfopen (path, replay ? "r" : "w");
  for (i = 1; argv[i]; ++i)
    {
      unsigned char const *p;
      size_t size = 0;
      mantle_bison23_byte (stream, replay, '"');
      for (p = (unsigned char const *) argv[i]; *p; ++p)
        {
          if (++size > 65536 || *p < 32 || *p == 127)
            error (EXIT_FAILURE, 0, "unrepresentable Bison 2.3 m4 argument");
          if (*p == '\\' || *p == '"')
            mantle_bison23_byte (stream, replay, '\\');
          mantle_bison23_byte (stream, replay, *p);
        }
      mantle_bison23_byte (stream, replay, '"');
      mantle_bison23_byte (stream, replay, '\n');
    }
  if ((replay && fgetc (stream) != EOF) || ferror (stream))
    error (EXIT_FAILURE, get_errno (), "invalid Bison 2.3 replay request");
  xfclose (stream);
}

static FILE *
mantle_bison23_result (void)
{
  char const *result = getenv ("MANTLE_BISON_RESULT");
  char const *status = getenv ("MANTLE_BISON_STATUS");
  char const *diagnostic = getenv ("MANTLE_BISON_STDERR");
  FILE *errors;
  FILE *completion;
  int byte;
  int first;
  int next;
  if (!result || !*result || !status || !*status || !diagnostic || !*diagnostic)
    error (EXIT_FAILURE, 0, "missing actual Bison 2.3 m4 result/status/diagnostic");
  errors = xfopen (diagnostic, "r");
  while ((byte = fgetc (errors)) != EOF)
    if (fputc (byte, stderr) == EOF)
      error (EXIT_FAILURE, get_errno (), "writing actual Bison 2.3 m4 diagnostic");
  if (ferror (errors))
    error (EXIT_FAILURE, get_errno (), "reading actual Bison 2.3 m4 diagnostic");
  xfclose (errors);
  completion = xfopen (status, "r");
  first = fgetc (completion);
  next = fgetc (completion);
  if (first != '0' || (next != EOF && (next != '\n' || fgetc (completion) != EOF))
      || ferror (completion))
    error (EXIT_FAILURE, 0, "actual Bison 2.3 m4 child failed");
  xfclose (completion);
  return xfopen (result, "r");
}
