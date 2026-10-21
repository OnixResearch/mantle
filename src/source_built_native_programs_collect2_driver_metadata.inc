/* Included in ORIGINAL collect2 main after process_args, before reading any
 * original driver metadata. Rehydrates ONLY the four environment values that
 * the genuine source-built GCC -### driver emitted. These are Produced source
 * generator results, not frozen configured values or an execution envelope.
 */
#define MANTLE_COLLECT2_METADATA_FAIL(message) \
  do { fprintf (stderr, "collect2 source metadata: %s\n", message); abort (); } while (0)
{
  const char *mantle_query_mode = getenv ("MANTLE_COLLECT2_PHASE_QUERY");
  if (mantle_query_mode)
    {
      const char *mantle_driver_log = getenv ("MANTLE_COLLECT2_PHASE_DRIVER_LOG");
      const char *mantle_driver_environment = getenv ("MANTLE_COLLECT2_PHASE_DRIVER_ENV");
      static char mantle_line[1048578];
      static struct {
        const char *prefix;
        char value[1048578];
        int seen;
      } mantle_values[] = {
        { "COLLECT_GCC=", "", 0 },
        { "COLLECT_GCC_OPTIONS=", "", 0 },
        { "COMPILER_PATH=", "", 0 },
        { "LIBRARY_PATH=", "", 0 }
      };
      FILE *mantle_log;
      size_t mantle_index;
      if (mantle_driver_environment)
        {
          /* The source controller copied the actual ordered GCC query values
             into typed canonical environment operands. Do not replace their
             path rebasing with raw logical strings from an earlier leaf. */
          if (strcmp (mantle_query_mode, "1")
              || strcmp (mantle_driver_environment, "1")
              || !getenv ("COLLECT_GCC") || !getenv ("COLLECT_GCC_OPTIONS")
              || !getenv ("COMPILER_PATH") || !getenv ("LIBRARY_PATH"))
            MANTLE_COLLECT2_METADATA_FAIL ("missing actual typed source driver environment");
        }
      else
        {
      if (strcmp (mantle_query_mode, "1") || !mantle_driver_log)
        MANTLE_COLLECT2_METADATA_FAIL ("missing actual source driver phase metadata");
      mantle_log = fopen (mantle_driver_log, "rb");
      if (!mantle_log)
        MANTLE_COLLECT2_METADATA_FAIL ("cannot open actual source driver phase metadata");
      while (fgets (mantle_line, sizeof (mantle_line), mantle_log))
        {
          size_t mantle_length = strlen (mantle_line);
          if (!mantle_length || mantle_line[mantle_length - 1] != '\n')
            MANTLE_COLLECT2_METADATA_FAIL ("oversized or unterminated driver metadata record");
          mantle_line[--mantle_length] = 0;
          for (mantle_index = 0;
               mantle_index < sizeof (mantle_values) / sizeof (mantle_values[0]);
               ++mantle_index)
            {
              size_t mantle_prefix_length = strlen (mantle_values[mantle_index].prefix);
              if (!strncmp (mantle_line, mantle_values[mantle_index].prefix, mantle_prefix_length))
                {
                  if (mantle_values[mantle_index].seen
                      && strcmp (mantle_values[mantle_index].value, mantle_line))
                    MANTLE_COLLECT2_METADATA_FAIL ("source driver metadata changed between phase records");
                  memcpy (mantle_values[mantle_index].value, mantle_line, mantle_length + 1);
                  mantle_values[mantle_index].seen = 1;
                }
            }
        }
      if (ferror (mantle_log) || fclose (mantle_log))
        MANTLE_COLLECT2_METADATA_FAIL ("cannot read actual source driver phase metadata");
      for (mantle_index = 0;
           mantle_index < sizeof (mantle_values) / sizeof (mantle_values[0]);
           ++mantle_index)
        {
          if (!mantle_values[mantle_index].seen)
            MANTLE_COLLECT2_METADATA_FAIL ("source driver omitted required original metadata");
          if (putenv (mantle_values[mantle_index].value))
            MANTLE_COLLECT2_METADATA_FAIL ("cannot restore actual source driver metadata");
        }
        }
    }
}
#undef MANTLE_COLLECT2_METADATA_FAIL
