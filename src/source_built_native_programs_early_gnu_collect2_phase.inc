/* Included in original GCC 4.0.4 collect2 main after ld1_argv is complete.
 * All argument selection remains original source code. The ordinary entry
 * point is unchanged; query mode exits before tlink can launch the linker.
 * Startup guards inserted by the paired original-source transform run before
 * any target-specific scanning can launch an executable. */
{
  const char *mantle_phase_mode = getenv ("MANTLE_COLLECT2_PHASE_QUERY");
  if (mantle_phase_mode)
    {
      const char *mantle_expected_ld = getenv ("MANTLE_COLLECT2_PHASE_LINKER");
      const char *const *mantle_argument;
      if (strcmp (mantle_phase_mode, "1") || !mantle_expected_ld)
        mantle_collect2_phase_fail ("invalid source-owned collect2 phase mode");
      if (do_collecting)
        mantle_collect2_phase_fail ("collect2 target requires constructor collection phases");
      if (!ld1_argv[0] || strcmp (ld1_argv[0], mantle_expected_ld))
        mantle_collect2_phase_fail ("collect2 selected a different exact linker image");
      if (fputc (' ', stdout) == EOF)
        mantle_collect2_phase_fail ("cannot write collect2 phase");
      for (mantle_argument = (const char *const *) ld1_argv;
           *mantle_argument; ++mantle_argument)
        {
          const unsigned char *mantle_byte;
          if (strstr (*mantle_argument, "-plugin")
              || strstr (*mantle_argument, "-flto"))
            mantle_collect2_phase_fail ("collect2 invocation requires plugin phases");
          if (fputc ('\"', stdout) == EOF)
            mantle_collect2_phase_fail ("cannot write collect2 phase");
          for (mantle_byte = (const unsigned char *) *mantle_argument;
               *mantle_byte; ++mantle_byte)
            {
              if (*mantle_byte < 32 || *mantle_byte == 127)
                mantle_collect2_phase_fail ("control byte in collect2 phase argument");
              if ((*mantle_byte == '\\' || *mantle_byte == '\"')
                  && fputc ('\\', stdout) == EOF)
                mantle_collect2_phase_fail ("cannot escape collect2 phase argument");
              if (fputc (*mantle_byte, stdout) == EOF)
                mantle_collect2_phase_fail ("cannot write collect2 phase argument");
            }
          if (fputs ("\" ", stdout) == EOF)
            mantle_collect2_phase_fail ("cannot close collect2 phase argument");
        }
      if (fputc ('\n', stdout) == EOF || fflush (stdout))
        mantle_collect2_phase_fail ("cannot finish collect2 phase");
      return 0;
    }
}
