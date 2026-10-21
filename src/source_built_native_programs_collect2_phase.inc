/* Included inside the ORIGINAL GCC 10.5.0 collect2 main, immediately after
 * computing early_exit and BEFORE the original first-link call. No original
 * linker argument is regenerated or removed. Normal collect2 is unchanged.
 * The admitted x86_64-musl, non-LTO, non-collecting route has exactly this
 * original ld invocation and an empty post_ld_pass; other modes fail closed.
 */
#define MANTLE_COLLECT2_PHASE_FAIL(message) \
  do { fprintf (stderr, "collect2 source phase: %s\n", message); abort (); } while (0)
{
  const char *mantle_phase_mode = getenv ("MANTLE_COLLECT2_PHASE_QUERY");
  if (mantle_phase_mode)
    {
      const char *mantle_expected_ld = getenv ("MANTLE_COLLECT2_PHASE_LINKER");
      const char *const *mantle_argument;
      if (strcmp (mantle_phase_mode, "1") || !mantle_expected_ld)
        MANTLE_COLLECT2_PHASE_FAIL ("invalid source-owned collect2 phase mode");
#if defined (OBJECT_FORMAT_COFF) || defined (COLLECT_EXPORT_LIST) || defined (DSYMUTIL) || defined (LDD_SUFFIX)
      MANTLE_COLLECT2_PHASE_FAIL ("collect2 target requires additional source-owned phases");
#endif
      if (!early_exit || do_collecting || use_plugin || lto_mode != LTO_MODE_NONE)
        MANTLE_COLLECT2_PHASE_FAIL ("collect2 invocation requires additional source-owned phases");
      if (!ld1_argv[0] || strcmp (ld1_argv[0], mantle_expected_ld))
        MANTLE_COLLECT2_PHASE_FAIL ("collect2 selected a different exact linker image");
      if (fputc (' ', stdout) == EOF)
        MANTLE_COLLECT2_PHASE_FAIL ("cannot write collect2 phase");
      for (mantle_argument = (const char *const *) ld1_argv;
           *mantle_argument; ++mantle_argument)
        {
          const unsigned char *mantle_byte;
          if (fputc ('\"', stdout) == EOF)
            MANTLE_COLLECT2_PHASE_FAIL ("cannot write collect2 phase");
          for (mantle_byte = (const unsigned char *) *mantle_argument;
               *mantle_byte; ++mantle_byte)
            {
              if (*mantle_byte < 32 || *mantle_byte == 127)
                MANTLE_COLLECT2_PHASE_FAIL ("control byte in collect2 phase argument");
              if ((*mantle_byte == '\\' || *mantle_byte == '\"')
                  && fputc ('\\', stdout) == EOF)
                MANTLE_COLLECT2_PHASE_FAIL ("cannot escape collect2 phase argument");
              if (fputc (*mantle_byte, stdout) == EOF)
                MANTLE_COLLECT2_PHASE_FAIL ("cannot write collect2 phase argument");
            }
          if (fputs ("\" ", stdout) == EOF)
            MANTLE_COLLECT2_PHASE_FAIL ("cannot close collect2 phase argument");
        }
      if (fputc ('\n', stdout) == EOF || fflush (stdout))
        MANTLE_COLLECT2_PHASE_FAIL ("cannot finish collect2 phase");
      return 0;
    }
}
#undef MANTLE_COLLECT2_PHASE_FAIL
