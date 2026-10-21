/* Original Perl 5.6.2 child selectors are called, not reimplemented. The only
 * intercepted operations are their final execvp/execl and pipe wait boundary.
 * This source is included in util.c after perl.h and the bridge declarations. */
#include <setjmp.h>
#include <stdarg.h>
#include <fcntl.h>
#include <unistd.h>
#include <errno.h>
static jmp_buf mantle_perl_boundary;
static int mantle_perl_selecting;
static int mantle_perl_child_status;
static int mantle_perl_child_stdin = -1;
static const char *mantle_perl_operation;
static char *mantle_perl_child_stdout;
struct mantle_perl_pipe {
    PerlIO *stream;
    char *command;
    int writing;
    I32 status;
    struct mantle_perl_pipe *next;
};
static struct mantle_perl_pipe *mantle_perl_pipes;
static void mantle_perl_fail(const char *why)
{
    dTHX;
    Perl_croak(aTHX_ "original Perl process continuation: %s", why);
}
static void mantle_perl_selected(const char *image, char *const *argv)
{
    int status = mantle_perl_transport(mantle_perl_selecting ? mantle_perl_operation : "exec",
        image, argv, mantle_perl_selecting ? mantle_perl_child_stdin : -1,
        &mantle_perl_child_stdout);
    if (status < 0 || status > 65280 || (status & 255))
        mantle_perl_fail("transport did not return an authenticated normal wait status");
    if (!mantle_perl_selecting)
        _exit(status >> 8); /* Successful original exec never runs Perl END. */
    mantle_perl_child_status = status;
    longjmp(mantle_perl_boundary, 1);
}
int mantle_perl_execvp(const char *image, char *const *argv)
{
    dTHX;
    if (mantle_perl_transport_active())
        mantle_perl_selected(image, argv);
    return PerlProc_execvp(image, argv);
}
int mantle_perl_execl(const char *image, const char *first, ...)
{
    dTHX;
    char *argv[256];
    size_t count = 0;
    const char *argument = first;
    va_list ap;
    va_start(ap, first);
    while (argument) {
        if (count == sizeof(argv) / sizeof(argv[0]) - 1)
            mantle_perl_fail("original execl argument bound");
        argv[count++] = (char *)argument;
        argument = va_arg(ap, const char *);
    }
    va_end(ap);
    argv[count] = NULL;
    if (mantle_perl_transport_active())
        mantle_perl_selected(image, argv);
    return PerlProc_execv(image, argv);
}
static void mantle_perl_select_begin(const char *operation, int input)
{
    if (mantle_perl_selecting)
        mantle_perl_fail("nested original child selector");
    mantle_perl_selecting = 1;
    mantle_perl_operation = operation;
    mantle_perl_child_stdin = input;
    mantle_perl_child_stdout = NULL;
}
static int mantle_perl_save_stderr(void)
{
    int fd = fcntl(2, F_DUPFD, 64);
    if (fd < 0 && errno != EBADF)
        mantle_perl_fail("saving original parent stderr");
    if (fd >= 0 && fcntl(fd, F_SETFD, FD_CLOEXEC))
        mantle_perl_fail("protecting selector-only stderr descriptor");
    return fd;
}
static void mantle_perl_select_end(pTHX_ int saved_stderr)
{
    /* Original do_exec3 may perform its 2>&1 optimization in its CHILD.
     * Selection runs without that fork, so restore the unchanged parent fd. */
    if (saved_stderr < 0) {
        if (close(2) && errno != EBADF)
            mantle_perl_fail("restoring closed original stderr");
    } else {
        if (dup2(saved_stderr, 2) < 0 || close(saved_stderr))
            mantle_perl_fail("restoring original parent stderr");
    }
    mantle_perl_selecting = 0;
    do_execfree();
}
I32 mantle_perl_system(pTHX_ int stacked, SV **mark, SV **sp)
{
    int saved_stderr = mantle_perl_save_stderr();
    STRLEN length;
    mantle_perl_select_begin("system", -1);
    if (!setjmp(mantle_perl_boundary)) {
        if (stacked) {
            SV *really = *++mark;
            (void)do_aexec5(really, mark, sp, 0, 0);
        } else if (sp - mark != 1) {
            (void)do_aexec5(Nullsv, mark, sp, 0, 0);
        } else {
            (void)do_exec3(SvPVx(sv_mortalcopy(*sp), length), 0, 0);
        }
        mantle_perl_fail("original system selector chose no executable");
    }
    mantle_perl_select_end(aTHX_ saved_stderr);
    return (I32)mantle_perl_child_status;
}
static I32 mantle_perl_pipe_select(pTHX_ const char *command, const char *operation, int input)
{
    int saved_stderr = mantle_perl_save_stderr();
    char *copy = savepv(command);
    mantle_perl_select_begin(operation, input);
    if (!setjmp(mantle_perl_boundary)) {
        (void)do_exec3(copy, 0, 0);
        mantle_perl_fail("original pipe selector chose no executable");
    }
    mantle_perl_select_end(aTHX_ saved_stderr);
    Safefree(copy);
    return (I32)mantle_perl_child_status;
}
PerlIO *mantle_perl_popen(pTHX_ const char *command, const char *mode)
{
    struct mantle_perl_pipe *pipe;
    PerlIO *stream;
    I32 status = 0;
    int writing = mode[0] == 'w';
    if ((!writing && mode[0] != 'r') || !strcmp(command, "-"))
        mantle_perl_fail("selected pipe requires a real original child image");
    if (writing)
        stream = PerlIO_tmpfile();
    else {
        status = mantle_perl_pipe_select(aTHX_ command, "readpipe", -1);
        if (!mantle_perl_child_stdout || !*mantle_perl_child_stdout)
            mantle_perl_fail("actual read-pipe output is absent");
        stream = PerlIO_open(mantle_perl_child_stdout, mode);
        free(mantle_perl_child_stdout);
        mantle_perl_child_stdout = NULL;
    }
    if (!stream)
        return Nullfp;
    pipe = (struct mantle_perl_pipe *)malloc(sizeof(*pipe));
    if (!pipe)
        mantle_perl_fail("pipe continuation allocation");
    pipe->stream = stream;
    pipe->command = writing ? savepv(command) : NULL;
    pipe->writing = writing;
    pipe->status = status;
    pipe->next = mantle_perl_pipes;
    mantle_perl_pipes = pipe;
    return stream;
}
int mantle_perl_pipe_owned(PerlIO *stream)
{
    struct mantle_perl_pipe *pipe;
    for (pipe = mantle_perl_pipes; pipe; pipe = pipe->next)
        if (pipe->stream == stream)
            return 1;
    return 0;
}
I32 mantle_perl_pclose(pTHX_ PerlIO *stream)
{
    struct mantle_perl_pipe **link = &mantle_perl_pipes;
    struct mantle_perl_pipe *pipe;
    I32 status;
    while (*link && (*link)->stream != stream)
        link = &(*link)->next;
    pipe = *link;
    if (!pipe)
        mantle_perl_fail("unknown original pipe continuation");
    PERL_FLUSHALL_FOR_CHILD;
    if (pipe->writing) {
        if (PerlIO_flush(stream) || PerlIO_seek(stream, 0, SEEK_SET))
            mantle_perl_fail("rewinding actual original write-pipe payload");
        pipe->status = mantle_perl_pipe_select(aTHX_ pipe->command, "writepipe", PerlIO_fileno(stream));
    }
    status = pipe->status;
    if (PerlIO_close(stream) == EOF)
        status = -1;
    *link = pipe->next;
    if (pipe->command)
        Safefree(pipe->command);
    free(pipe);
    return status;
}
