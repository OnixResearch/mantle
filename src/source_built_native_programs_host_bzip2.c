/* Original BusyBox bzip2 algorithm, with only the usage-generator stdio entry.
 * This executable is not advertised as a full bzip2 or BusyBox implementation.
 */
#include "libbb.h"
#include "libbb/crc32.c"
#include "archival/libarchive/bz/bzlib.h"
#include "archival/libarchive/bz/bzlib_private.h"
#include "archival/libarchive/bz/blocksort.c"
#include "archival/libarchive/bz/bzlib.c"
#include "archival/libarchive/bz/compress.c"
#include "archival/libarchive/bz/huffman.c"
int main(int argc,char **argv){
    bz_stream stream;unsigned char input[8192],output[8192];size_t count;int result;
    if(argc!=2||strcmp(argv[1],"-1")){fputs("original BusyBox usage compressor expects -1\n",stderr);return 2;}
    memset(&stream,0,sizeof stream);BZ2_bzCompressInit(&stream,1);
    for(;;){
        count=fread(input,1,sizeof input,stdin);if(ferror(stdin)){perror("BusyBox bzip2 input");BZ2_bzCompressEnd(&stream);return 1;}
        stream.next_in=(char *)input;stream.avail_in=(unsigned)count;
        do{
            stream.next_out=(char *)output;stream.avail_out=sizeof output;
            result=BZ2_bzCompress(&stream,count?BZ_RUN:BZ_FINISH);
            if(result!=BZ_RUN_OK&&result!=BZ_FINISH_OK&&result!=BZ_STREAM_END){fputs("BusyBox bzip2 internal error\n",stderr);BZ2_bzCompressEnd(&stream);return 1;}
            size_t written=sizeof output-stream.avail_out;if(fwrite(output,1,written,stdout)!=written){perror("BusyBox bzip2 output");BZ2_bzCompressEnd(&stream);return 1;}
        }while(result!=BZ_STREAM_END&&(!count||stream.avail_in));
        if(!count)break;
    }
    BZ2_bzCompressEnd(&stream);if(fflush(stdout)){perror("BusyBox bzip2 output close");return 1;}return 0;
}
