/* Included only after original perl.h. Native default execution is unchanged.
 * The transport is linked as source into this same original interpreter. */
extern int mantle_perl_transport_active(void);
extern int mantle_perl_transport(const char *, const char *, char *const *, int, char **);
extern I32 mantle_perl_system(pTHX_ int, SV **, SV **);
extern PerlIO *mantle_perl_popen(pTHX_ const char *, const char *);
extern int mantle_perl_pipe_owned(PerlIO *);
extern I32 mantle_perl_pclose(pTHX_ PerlIO *);
extern int mantle_perl_execvp(const char *, char *const *);
extern int mantle_perl_execl(const char *, const char *, ...);
