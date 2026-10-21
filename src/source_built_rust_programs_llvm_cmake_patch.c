/* Exact original CMake 3.31.8 source transformation. No configuration,
 * compiler invocation, or process creation occurs in this program. */
#define _POSIX_C_SOURCE 200809L
#include <dirent.h>
#include <errno.h>
#include <fcntl.h>
#include <limits.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <unistd.h>

#define LIMIT (64u * 1024u * 1024u)
static void die(const char *message) { fprintf(stderr, "CMake phase source transform: %s\n", message); exit(1); }
static void *allocate(size_t n) { void *p = malloc(n ? n : 1); if (!p) die("allocation failed"); return p; }
static char *join(const char *a, const char *b) {
    size_t x = strlen(a), y = strlen(b);
    if (x > LIMIT || y > LIMIT || x + y + 2 > LIMIT) die("path bound exceeded");
    char *p = allocate(x+y+2); memcpy(p,a,x); p[x]='/'; memcpy(p+x+1,b,y+1); return p;
}
static char *load(const char *path, size_t *length) {
    FILE *f=fopen(path,"rb"); if(!f)die("cannot open source");
    size_t cap=16384,n=0; char *p=allocate(cap+1);
    for (;;) {
        if(n==cap) { if(cap==LIMIT)die("source bound exceeded"); cap*=2; if(cap>LIMIT)cap=LIMIT; char *q=realloc(p,cap+1); if(!q)die("allocation failed"); p=q; }
        size_t k=fread(p+n,1,cap-n,f); n+=k;
        if(!k) {if(ferror(f))die("cannot read source"); break;}
    }
    if(fclose(f))die("cannot close source"); p[n]=0; *length=n; return p;
}
static void save(const char *path,const char *p,size_t n,mode_t mode) {
    int fd=open(path,O_WRONLY|O_CREAT|O_EXCL|O_NOFOLLOW,mode); if(fd<0)die("cannot create transformed source");
    size_t at=0; while(at<n) {ssize_t k=write(fd,p+at,n-at); if(k<0&&errno==EINTR)continue; if(k<=0)die("cannot write transformed source"); at+=(size_t)k;}
    if(close(fd))die("cannot close transformed source");
}
static void replace(char **text,size_t *length,const char *old,const char *replacement) {
    size_t a=strlen(old),b=strlen(replacement); char *at=strstr(*text,old);
    if(!at||strstr(at+a,old))die("original source anchor is absent or ambiguous");
    if(*length<a||*length-a>LIMIT-b)die("transformed source bound exceeded");
    size_t prefix=(size_t)(at-*text), n=*length-a+b; char *p=allocate(n+1);
    memcpy(p,*text,prefix); memcpy(p+prefix,replacement,b); memcpy(p+prefix+b,at+a,*length-prefix-a+1);
    free(*text);*text=p;*length=n;
}
static const char chain_branch[] =
"  if (!mantle_cmake_phase::active())\n"
"    mantle_cmake_phase::fail(\"phase evaluator requires declared continuation inputs\");\n"
"  {\n"
"    std::vector<std::vector<std::string>> commands;\n"
"    for (auto const& process : this->Processes) commands.push_back(process.Arguments);\n"
"    std::string kind = \"process\";\n"
"    for (auto const& stream : this->Stdio) kind += \":\" + std::to_string(static_cast<int>(stream.Type));\n"
"    kind += this->MergedBuiltinStreams ? \":merged\" : \":separate\";\n"
"    auto const& in = this->Stdio[Stream_INPUT];\n"
"    auto stdinBytes = mantle_cmake_phase::input(in.Type == External ? in.FileDescriptor : -1);\n"
"    auto result = mantle_cmake_phase::exchange(\n"
"      mantle_cmake_phase::request(kind, commands, this->WorkingDirectory, stdinBytes), commands.size());\n"
"    mantle_cmake_phase::consume_input(in.Type == External ? in.FileDescriptor : -1,\n"
"      result.inputConsumed, stdinBytes.size());\n"
"    if (this->MergedBuiltinStreams && !result.err.empty())\n"
"      mantle_cmake_phase::fail(\"merged original stream requires a genuine merged native observation\");\n"
"    auto const& out = this->Stdio[Stream_OUTPUT];\n"
"    auto const& err = this->Stdio[Stream_ERROR];\n"
"    if (out.Type != None) mantle_cmake_phase::stream(\n"
"      this->MergedBuiltinStreams ? chain.Data->TempErrorPipe : chain.Data->TempOutputPipe,\n"
"      out.Type == External ? out.FileDescriptor : -1, out.Type == Builtin, std::move(result.out));\n"
"    if (!this->MergedBuiltinStreams && err.Type != None) mantle_cmake_phase::stream(\n"
"      chain.Data->TempErrorPipe, err.Type == External ? err.FileDescriptor : -1,\n"
"      err.Type == Builtin, std::move(result.err));\n"
"    for (std::size_t i = 0; i < result.codes.size(); ++i) {\n"
"      auto& process = *chain.Data->Processes[i];\n"
"      process.ProcessStatus.SpawnResult = 0;\n"
"      process.ProcessStatus.ExitStatus = result.codes[i];\n"
"      process.ProcessStatus.TermSignal = 0;\n"
"      process.InputPipe.reset(); process.OutputPipe.reset(); process.Finish();\n"
"    }\n"
"    chain.Data->Finish();\n"
"    return chain;\n"
"  }\n"
"  for (std::size_t i = 0; i < this->Processes.size(); i++) {";
static void transform(const char *member,char **text,size_t *length) {
    if(!strcmp(member,"Source/cmUVProcessChain.cxx")) {
        replace(text,length,"#include \"cmUVProcessChain.h\"","#include \"cmUVProcessChain.h\"\n#include \"cmMantleSourcePhase.h\"");
        replace(text,length,"  for (std::size_t i = 0; i < this->Processes.size(); i++) {",chain_branch);
    } else if(!strcmp(member,"Source/cmSystemTools.cxx")) {
        replace(text,length,"#include \"cmSystemTools.h\"","#include \"cmSystemTools.h\"\n#include \"cmMantleSourcePhase.h\"");
        replace(text,length,"unsigned int cmSystemTools::RandomSeed()\n{",
          "unsigned int cmSystemTools::RandomSeed()\n{\n  auto original = []() -> unsigned int {");
        replace(text,length,
          "  return tv_sec ^ (tv_usec << 21) ^ pid;\n#endif\n}",
          "  return tv_sec ^ (tv_usec << 21) ^ pid;\n#endif\n  };\n  return mantle_cmake_phase::random_seed(original);\n}");
        replace(text,length,"cmsys::Status cmSystemTools::MakeTempDirectory(char* path, const mode_t* mode)\n{",
          "cmsys::Status cmSystemTools::MakeTempDirectory(char* path, const mode_t* mode)\n{\n  auto original = [&]() -> cmsys::Status {");
        replace(text,length,
          "  return cmsys::Status::Success();\n#endif\n}\n\n#ifdef _WIN32\nnamespace {\nbool cmMoveFile",
          "  return cmsys::Status::Success();\n#endif\n  };\n  return mantle_cmake_phase::temporary_directory(path, mode, original);\n}\n\n#ifdef _WIN32\nnamespace {\nbool cmMoveFile");
    } else if(!strcmp(member,"Source/cmUVHandlePtr.cxx")) {
        replace(text,length,"#include \"cmUVHandlePtr.h\"","#include \"cmUVHandlePtr.h\"\n#include \"cmMantleSourcePhase.h\"");
        replace(text,length,"  return uv_spawn(&loop, *this, &options);","  mantle_cmake_phase::fail(\"unlowered original libuv child creation\");");
    } else if(!strcmp(member,"Source/cmExecProgramCommand.cxx")) {
        replace(text,length,"#include \"cmExecProgramCommand.h\"","#include \"cmExecProgramCommand.h\"\n#include \"cmMantleSourcePhase.h\"");
        replace(text,length,"  cmsysProcess_Execute(cp);","  mantle_cmake_phase::fail(\"legacy exec_program requires source phase lowering\");");
    }
}
static unsigned patched;
static void copy(const char *source,const char *destination,const char *member,unsigned depth) {
    if(depth>128)die("source tree depth exceeded"); struct stat st;
    if(lstat(source,&st))die("cannot inspect original source tree");
    if(S_ISDIR(st.st_mode)) {
        if(mkdir(destination,0700) && !(depth==0 && errno==EEXIST))die("cannot create transformed source directory");
        DIR *d=opendir(source);if(!d)die("cannot open original source directory");struct dirent *entry;
        errno=0;while((entry=readdir(d))) {
            if(!strcmp(entry->d_name,".")||!strcmp(entry->d_name,".."))continue;
            char *s=join(source,entry->d_name),*o=join(destination,entry->d_name);
            char *m=*member?join(member,entry->d_name):strdup(entry->d_name);if(!m)die("allocation failed");
            copy(s,o,m,depth+1);free(s);free(o);free(m);errno=0;
        }
        if(errno)die("cannot enumerate original source directory");if(closedir(d))die("cannot close original source directory");
    } else if(S_ISREG(st.st_mode)) {
        size_t n;char *text=load(source,&n);
        if(!strcmp(member,"Source/cmUVProcessChain.cxx")||!strcmp(member,"Source/cmSystemTools.cxx")||
           !strcmp(member,"Source/cmUVHandlePtr.cxx")||!strcmp(member,"Source/cmExecProgramCommand.cxx")) {
            if(memchr(text,0,n))die("NUL in original CMake translation unit");transform(member,&text,&n);++patched;
        }
        save(destination,text,n,(st.st_mode&0111)?0700:0600);free(text);
    } else if(S_ISLNK(st.st_mode)) {
        char target[PATH_MAX+1];ssize_t n=readlink(source,target,PATH_MAX);if(n<0||n==PATH_MAX)die("invalid source symlink");
        target[n]=0;if(target[0]=='/')die("absolute original source symlink");
        /* Authenticated original relative links are retained verbatim. The
         * source executor independently rejects tree links escaping its root. */
        if(symlink(target,destination))die("cannot copy original source symlink");
    } else die("non-source member in original CMake tree");
}
int main(int argc,char **argv) {
    if(argc!=4)die("expected ORIGINAL_CMAKE_SOURCE PHASE_HEADER OUTPUT_TREE");
    struct stat st;
    if(lstat(argv[3],&st)==0) {
        if(!S_ISDIR(st.st_mode))die("output is not a fresh tree");
        DIR *d=opendir(argv[3]);if(!d)die("cannot inspect fresh output tree");struct dirent *entry;
        while((entry=readdir(d)))if(strcmp(entry->d_name,".")&&strcmp(entry->d_name,".."))die("output tree is not empty");
        if(closedir(d))die("cannot close fresh output tree");
    } else if(errno!=ENOENT)die("cannot inspect fresh output tree");
    copy(argv[1],argv[3],"",0);if(patched!=4)die("original CMake source seam inventory changed");
    size_t n;char *header=load(argv[2],&n);char *path=join(argv[3],"Source/cmMantleSourcePhase.h");
    save(path,header,n,0600);free(path);free(header);return 0;
}
