#ifndef MANTLE_NATIVE_MAKE_CONTROL_H
#define MANTLE_NATIVE_MAKE_CONTROL_H
#include <sys/types.h>
#include <stdio.h>
#include <dirent.h>
#include <fcntl.h>
#include <glob.h>
#include <sys/stat.h>
#include <unistd.h>
#include <time.h>
int mantle_make_active(void);
pid_t mantle_make_child(char **argv, char **environment, int input, int output, int error);
pid_t mantle_make_wait(int *status, int nonblocking);
void mantle_make_replace(char **argv, char **environment) __attribute__((noreturn));
int mantle_make_cancel(pid_t token, int signal);
int mantle_make_pipe(int descriptors[2]);
void mantle_make_exit(int status) __attribute__((noreturn));
int mantle_make_paths_active(void);
FILE *mantle_paths_fopen(const char *,const char *);
int mantle_paths_open(const char *,int,...);
int mantle_paths_stat(const char *,struct stat *);
int mantle_paths_lstat(const char *,struct stat *);
int mantle_paths_access(const char *,int);
int mantle_paths_chdir(const char *);
char *mantle_paths_getcwd(char *,size_t);
DIR *mantle_paths_opendir(const char *);
int mantle_paths_mkdir(const char *,mode_t);
int mantle_paths_unlink(const char *);
int mantle_paths_rmdir(const char *);
int mantle_paths_rename(const char *,const char *);
ssize_t mantle_paths_readlink(const char *,char *,size_t);
int mantle_paths_symlink(const char *,const char *);
int mantle_paths_link(const char *,const char *);
int mantle_paths_chmod(const char *,mode_t);
int mantle_paths_utimensat(int,const char *,const struct timespec[2],int);
int mantle_paths_glob(const char *,int,int (*)(const char *,int),glob_t *);
void mantle_paths_globfree(glob_t *);
/* One bridge TU owns the logical path state. Only original Make evaluator
 * calls are redirected; the checkpoint/protocol and native utility code are
 * deliberately physical. Ordinary installed Make preserves libc behavior. */
#ifndef MANTLE_NATIVE_MAKE_PHYSICAL
#define fopen(...) (mantle_make_paths_active()?mantle_paths_fopen(__VA_ARGS__):fopen(__VA_ARGS__))
#define open(...) (mantle_make_paths_active()?mantle_paths_open(__VA_ARGS__):open(__VA_ARGS__))
#define stat(...) (mantle_make_paths_active()?mantle_paths_stat(__VA_ARGS__):stat(__VA_ARGS__))
#define lstat(...) (mantle_make_paths_active()?mantle_paths_lstat(__VA_ARGS__):lstat(__VA_ARGS__))
#define access(...) (mantle_make_paths_active()?mantle_paths_access(__VA_ARGS__):access(__VA_ARGS__))
#define chdir(...) (mantle_make_paths_active()?mantle_paths_chdir(__VA_ARGS__):chdir(__VA_ARGS__))
#define getcwd(...) (mantle_make_paths_active()?mantle_paths_getcwd(__VA_ARGS__):getcwd(__VA_ARGS__))
#define opendir(...) (mantle_make_paths_active()?mantle_paths_opendir(__VA_ARGS__):opendir(__VA_ARGS__))
#define mkdir(...) (mantle_make_paths_active()?mantle_paths_mkdir(__VA_ARGS__):mkdir(__VA_ARGS__))
#define unlink(...) (mantle_make_paths_active()?mantle_paths_unlink(__VA_ARGS__):unlink(__VA_ARGS__))
#define rmdir(...) (mantle_make_paths_active()?mantle_paths_rmdir(__VA_ARGS__):rmdir(__VA_ARGS__))
#define rename(...) (mantle_make_paths_active()?mantle_paths_rename(__VA_ARGS__):rename(__VA_ARGS__))
#define readlink(...) (mantle_make_paths_active()?mantle_paths_readlink(__VA_ARGS__):readlink(__VA_ARGS__))
#define symlink(...) (mantle_make_paths_active()?mantle_paths_symlink(__VA_ARGS__):symlink(__VA_ARGS__))
#define link(...) (mantle_make_paths_active()?mantle_paths_link(__VA_ARGS__):link(__VA_ARGS__))
#define chmod(...) (mantle_make_paths_active()?mantle_paths_chmod(__VA_ARGS__):chmod(__VA_ARGS__))
#define utimensat(...) (mantle_make_paths_active()?mantle_paths_utimensat(__VA_ARGS__):utimensat(__VA_ARGS__))
#define glob(...) (mantle_make_paths_active()?mantle_paths_glob(__VA_ARGS__):glob(__VA_ARGS__))
#define globfree(...) (mantle_make_paths_active()?mantle_paths_globfree(__VA_ARGS__):globfree(__VA_ARGS__))
#endif
#endif
