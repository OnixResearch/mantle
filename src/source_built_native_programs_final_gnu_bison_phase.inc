/* Included before original Bison 3.4.1 src/output.c output_skeleton. Its
   optional source-phase branch calls this function; default behavior retains
   the original body. Capture terminates before any child; replay reruns the
   original grammar/tables and consumes the matching completed m4 leaf. */
static void
mantle_bison_boundary_error (int status, int code, char const *message)
{
  (void) status;
  fprintf (stderr, "original Bison child boundary: %s: %s\n", message,
           code ? strerror (code) : "invalid source continuation");
  abort ();
}

static FILE *
mantle_bison_boundary_open (char const *path, char const *mode)
{
  FILE *stream = fopen (path, mode);
  if (!stream)
    mantle_bison_boundary_error (EXIT_FAILURE, errno, "opening actual phase artifact");
  return stream;
}

static void
mantle_bison_boundary_close (FILE *stream)
{
  if (ferror (stream) || fclose (stream))
    mantle_bison_boundary_error (EXIT_FAILURE, errno, "closing actual phase artifact");
}

static void
mantle_bison_response_byte (FILE *stream, bool replay, unsigned char byte)
{
  if (replay)
    {
      if (fgetc (stream) != byte)
        mantle_bison_boundary_error (EXIT_FAILURE, 0, "original Bison m4 request changed on replay");
    }
  else if (fputc (byte, stream) == EOF)
    mantle_bison_boundary_error (EXIT_FAILURE, errno, "writing original Bison m4 request");
}

static void
mantle_bison_response (char const *const *argv, bool replay)
{
  char const *path = getenv ("MANTLE_BISON_RESPONSE");
  if (!path || !*path)
    mantle_bison_boundary_error (EXIT_FAILURE, 0, "missing original Bison m4 response boundary");
  FILE *stream = mantle_bison_boundary_open (path, replay ? "r" : "w");
  char const *image_path = getenv ("MANTLE_BISON_IMAGE");
  if (image_path && *image_path)
    {
      FILE *image = mantle_bison_boundary_open (image_path, replay ? "r" : "w");
      for (unsigned char const *p = (unsigned char const *) argv[0]; *p; ++p)
        {
          if (*p < 32 || *p == 127)
            mantle_bison_boundary_error (EXIT_FAILURE, 0, "unrepresentable original Bison m4 image");
          if (replay ? fgetc (image) != *p : fputc (*p, image) == EOF)
            mantle_bison_boundary_error (EXIT_FAILURE, 0, "original Bison m4 image changed on replay");
        }
      if (replay ? (fgetc (image) != '\n' || fgetc (image) != EOF) : fputc ('\n', image) == EOF)
        mantle_bison_boundary_error (EXIT_FAILURE, 0, "invalid original Bison m4 image boundary");
      if (ferror (image))
        mantle_bison_boundary_error (EXIT_FAILURE, errno, "original Bison m4 image boundary");
      mantle_bison_boundary_close (image);
    }
  for (size_t i = 1; argv[i]; ++i)
    {
      mantle_bison_response_byte (stream, replay, '"');
      size_t size = 0;
      for (unsigned char const *p = (unsigned char const *) argv[i]; *p; ++p)
        {
          if (++size > 65536 || *p < 32 || *p == 127)
            mantle_bison_boundary_error (EXIT_FAILURE, 0, "unrepresentable original Bison m4 argument");
          if (*p == '\\' || *p == '"')
            mantle_bison_response_byte (stream, replay, '\\');
          mantle_bison_response_byte (stream, replay, *p);
        }
      mantle_bison_response_byte (stream, replay, '"');
      mantle_bison_response_byte (stream, replay, '\n');
    }
  if (replay && fgetc (stream) != EOF)
    mantle_bison_boundary_error (EXIT_FAILURE, 0, "trailing original Bison m4 replay argument");
  if (ferror (stream))
    mantle_bison_boundary_error (EXIT_FAILURE, errno, "reading original Bison m4 request");
  mantle_bison_boundary_close (stream);
}

static void
mantle_bison_output_skeleton (void)
{
  char const *mode = getenv ("MANTLE_BISON_MODE");
  bool replay = mode && STREQ (mode, "replay");
  if (!mode || (!replay && !STREQ (mode, "capture")))
    mantle_bison_boundary_error (EXIT_FAILURE, 0, "missing original Bison source phase mode");

  /* The following path and argument decisions are original output_skeleton. */
  char const *m4 = (m4 = getenv ("M4")) ? m4 : M4;
  char const *datadir = pkgdatadir ();
  char *skeldir = xpath_join (datadir, "skeletons");
  char *m4sugar = xpath_join (datadir, "m4sugar/m4sugar.m4");
  char *m4bison = xpath_join (skeldir, "bison.m4");
  char *skel = (IS_PATH_WITH_DIR (skeleton)
                ? xstrdup (skeleton)
                : xpath_join (skeldir, skeleton));
  xfclose (xfopen (m4sugar, "r"));
  if (trace_flag & trace_tools)
    fprintf (stderr, "running: %s %s - %s %s\n",
             m4, m4sugar, m4bison, skel);

  char const *argv[10];
  int i = 0;
  argv[i++] = m4;
  if (*M4_GNU_OPTION)
    argv[i++] = M4_GNU_OPTION;
  argv[i++] = "-I";
  argv[i++] = datadir;
  if (trace_flag & trace_m4)
    argv[i++] = "-dV";
  argv[i++] = m4sugar;
  argv[i++] = "-";
  argv[i++] = m4bison;
  argv[i++] = skel;
  argv[i++] = NULL;
  aver (i <= ARRAY_CARDINALITY (argv));
  mantle_bison_response (argv, replay);

  free (skeldir);
  free (m4sugar);
  free (m4bison);
  free (skel);
  if (trace_flag & trace_muscles)
    muscles_output (stderr);
  if (!replay)
    {
      muscles_output (stdout);
      if (fflush (stdout) || ferror (stdout))
        mantle_bison_boundary_error (EXIT_FAILURE, errno, "writing original Bison muscles");
      exit (86);
    }

  char const *result = getenv ("MANTLE_BISON_RESULT");
  char const *status = getenv ("MANTLE_BISON_STATUS");
  char const *diagnostic = getenv ("MANTLE_BISON_STDERR");
  if (!result || !*result || !status || !*status || !diagnostic || !*diagnostic)
    mantle_bison_boundary_error (EXIT_FAILURE, 0, "missing actual Bison m4 result/status/diagnostic");
  FILE *errors = mantle_bison_boundary_open (diagnostic, "r");
  int byte;
  while ((byte = fgetc (errors)) != EOF)
    if (fputc (byte, stderr) == EOF)
      mantle_bison_boundary_error (EXIT_FAILURE, errno, "writing actual Bison m4 diagnostic");
  if (ferror (errors))
    mantle_bison_boundary_error (EXIT_FAILURE, errno, "reading actual Bison m4 diagnostic");
  mantle_bison_boundary_close (errors);
  timevar_push (tv_m4);
  FILE *in = mantle_bison_boundary_open (result, "r");
  scan_skel (in);
  aver (feof (in));
  mantle_bison_boundary_close (in);
  FILE *completion = mantle_bison_boundary_open (status, "r");
  unsigned int code = 0;
  int byte_status = fgetc (completion);
  if (byte_status < '0' || byte_status > '9')
    mantle_bison_boundary_error (EXIT_FAILURE, 0, "missing actual normal m4 status");
  do
    {
      code = code * 10 + byte_status - '0';
      if (code > 255)
        mantle_bison_boundary_error (EXIT_FAILURE, 0, "invalid actual normal m4 status");
      byte_status = fgetc (completion);
    }
  while (byte_status >= '0' && byte_status <= '9');
  if (byte_status == '\n')
    byte_status = fgetc (completion);
  if (byte_status != EOF)
    mantle_bison_boundary_error (EXIT_FAILURE, 0, "trailing actual normal m4 status");
  mantle_bison_boundary_close (completion);
  if (code)
    error (EXIT_FAILURE, 0, "actual Bison m4 child did not complete successfully");
  timevar_pop (tv_m4);
}
