/* A Bison parser, made by GNU Bison 2.3.  */

/* Skeleton implementation for Bison's Yacc-like parsers in C

   Copyright (C) 1984, 1989, 1990, 2000, 2001, 2002, 2003, 2004, 2005, 2006
   Free Software Foundation, Inc.

   This program is free software; you can redistribute it and/or modify
   it under the terms of the GNU General Public License as published by
   the Free Software Foundation; either version 2, or (at your option)
   any later version.

   This program is distributed in the hope that it will be useful,
   but WITHOUT ANY WARRANTY; without even the implied warranty of
   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
   GNU General Public License for more details.

   You should have received a copy of the GNU General Public License
   along with this program; if not, write to the Free Software
   Foundation, Inc., 51 Franklin Street, Fifth Floor,
   Boston, MA 02110-1301, USA.  */

/* As a special exception, you may create a larger work that contains
   part or all of the Bison parser skeleton and distribute that work
   under terms of your choice, so long as that work isn't itself a
   parser generator using the skeleton or a modified version thereof
   as a parser skeleton.  Alternatively, if you modify or redistribute
   the parser skeleton itself, you may (at your option) remove this
   special exception, which will cause the skeleton and the resulting
   Bison output files to be licensed under the GNU General Public
   License without this special exception.

   This special exception was added by the Free Software Foundation in
   version 2.2 of Bison.  */

/* C LALR(1) parser skeleton written by Richard Stallman, by
   simplifying the original so-called "semantic" parser.  */

/* All symbols defined below should begin with yy or YY, to avoid
   infringing on user name space.  This should be done even for local
   variables, as they might otherwise be expanded by user macros.
   There are some unavoidable exceptions within include files to
   define necessary library symbols; they are noted "INFRINGES ON
   USER NAME SPACE" below.  */

/* Identify Bison output.  */
#define YYBISON 1

/* Bison version.  */
#define YYBISON_VERSION "2.3"

/* Skeleton name.  */
#define YYSKELETON_NAME "yacc.c"

/* Pure parsers.  */
#define YYPURE 0

/* Using locations.  */
#define YYLSP_NEEDED 0



/* Tokens.  */
#ifndef YYTOKENTYPE
# define YYTOKENTYPE
   /* Put the tokens into the symbol table, so that GDB and other debuggers
      know about them.  */
   enum yytokentype {
     INT = 258,
     NAME = 259,
     LNAME = 260,
     OREQ = 261,
     ANDEQ = 262,
     RSHIFTEQ = 263,
     LSHIFTEQ = 264,
     DIVEQ = 265,
     MULTEQ = 266,
     MINUSEQ = 267,
     PLUSEQ = 268,
     OROR = 269,
     ANDAND = 270,
     NE = 271,
     EQ = 272,
     GE = 273,
     LE = 274,
     RSHIFT = 275,
     LSHIFT = 276,
     UNARY = 277,
     END = 278,
     ALIGN_K = 279,
     BLOCK = 280,
     BIND = 281,
     QUAD = 282,
     SQUAD = 283,
     LONG = 284,
     SHORT = 285,
     BYTE = 286,
     SECTIONS = 287,
     PHDRS = 288,
     INSERT_K = 289,
     AFTER = 290,
     BEFORE = 291,
     DATA_SEGMENT_ALIGN = 292,
     DATA_SEGMENT_RELRO_END = 293,
     DATA_SEGMENT_END = 294,
     SORT_BY_NAME = 295,
     SORT_BY_ALIGNMENT = 296,
     SORT_NONE = 297,
     SORT_BY_INIT_PRIORITY = 298,
     SIZEOF_HEADERS = 299,
     OUTPUT_FORMAT = 300,
     FORCE_COMMON_ALLOCATION = 301,
     OUTPUT_ARCH = 302,
     INHIBIT_COMMON_ALLOCATION = 303,
     FORCE_GROUP_ALLOCATION = 304,
     SEGMENT_START = 305,
     INCLUDE = 306,
     MEMORY = 307,
     REGION_ALIAS = 308,
     LD_FEATURE = 309,
     NOLOAD = 310,
     DSECT = 311,
     COPY = 312,
     INFO = 313,
     OVERLAY = 314,
     DEFINED = 315,
     TARGET_K = 316,
     SEARCH_DIR = 317,
     MAP = 318,
     ENTRY = 319,
     NEXT = 320,
     SIZEOF = 321,
     ALIGNOF = 322,
     ADDR = 323,
     LOADADDR = 324,
     MAX_K = 325,
     MIN_K = 326,
     STARTUP = 327,
     HLL = 328,
     SYSLIB = 329,
     FLOAT = 330,
     NOFLOAT = 331,
     NOCROSSREFS = 332,
     NOCROSSREFS_TO = 333,
     ORIGIN = 334,
     FILL = 335,
     LENGTH = 336,
     CREATE_OBJECT_SYMBOLS = 337,
     INPUT = 338,
     GROUP = 339,
     OUTPUT = 340,
     CONSTRUCTORS = 341,
     ALIGNMOD = 342,
     AT = 343,
     SUBALIGN = 344,
     HIDDEN = 345,
     PROVIDE = 346,
     PROVIDE_HIDDEN = 347,
     AS_NEEDED = 348,
     CHIP = 349,
     LIST = 350,
     SECT = 351,
     ABSOLUTE = 352,
     LOAD = 353,
     NEWLINE = 354,
     ENDWORD = 355,
     ORDER = 356,
     NAMEWORD = 357,
     ASSERT_K = 358,
     LOG2CEIL = 359,
     FORMAT = 360,
     PUBLIC = 361,
     DEFSYMEND = 362,
     BASE = 363,
     ALIAS = 364,
     TRUNCATE = 365,
     REL = 366,
     INPUT_SCRIPT = 367,
     INPUT_MRI_SCRIPT = 368,
     INPUT_DEFSYM = 369,
     CASE = 370,
     EXTERN = 371,
     START = 372,
     VERS_TAG = 373,
     VERS_IDENTIFIER = 374,
     GLOBAL = 375,
     LOCAL = 376,
     VERSIONK = 377,
     INPUT_VERSION_SCRIPT = 378,
     KEEP = 379,
     ONLY_IF_RO = 380,
     ONLY_IF_RW = 381,
     SPECIAL = 382,
     INPUT_SECTION_FLAGS = 383,
     ALIGN_WITH_INPUT = 384,
     EXCLUDE_FILE = 385,
     CONSTANT = 386,
     INPUT_DYNAMIC_LIST = 387
   };
#endif
/* Tokens.  */
#define INT 258
#define NAME 259
#define LNAME 260
#define OREQ 261
#define ANDEQ 262
#define RSHIFTEQ 263
#define LSHIFTEQ 264
#define DIVEQ 265
#define MULTEQ 266
#define MINUSEQ 267
#define PLUSEQ 268
#define OROR 269
#define ANDAND 270
#define NE 271
#define EQ 272
#define GE 273
#define LE 274
#define RSHIFT 275
#define LSHIFT 276
#define UNARY 277
#define END 278
#define ALIGN_K 279
#define BLOCK 280
#define BIND 281
#define QUAD 282
#define SQUAD 283
#define LONG 284
#define SHORT 285
#define BYTE 286
#define SECTIONS 287
#define PHDRS 288
#define INSERT_K 289
#define AFTER 290
#define BEFORE 291
#define DATA_SEGMENT_ALIGN 292
#define DATA_SEGMENT_RELRO_END 293
#define DATA_SEGMENT_END 294
#define SORT_BY_NAME 295
#define SORT_BY_ALIGNMENT 296
#define SORT_NONE 297
#define SORT_BY_INIT_PRIORITY 298
#define SIZEOF_HEADERS 299
#define OUTPUT_FORMAT 300
#define FORCE_COMMON_ALLOCATION 301
#define OUTPUT_ARCH 302
#define INHIBIT_COMMON_ALLOCATION 303
#define FORCE_GROUP_ALLOCATION 304
#define SEGMENT_START 305
#define INCLUDE 306
#define MEMORY 307
#define REGION_ALIAS 308
#define LD_FEATURE 309
#define NOLOAD 310
#define DSECT 311
#define COPY 312
#define INFO 313
#define OVERLAY 314
#define DEFINED 315
#define TARGET_K 316
#define SEARCH_DIR 317
#define MAP 318
#define ENTRY 319
#define NEXT 320
#define SIZEOF 321
#define ALIGNOF 322
#define ADDR 323
#define LOADADDR 324
#define MAX_K 325
#define MIN_K 326
#define STARTUP 327
#define HLL 328
#define SYSLIB 329
#define FLOAT 330
#define NOFLOAT 331
#define NOCROSSREFS 332
#define NOCROSSREFS_TO 333
#define ORIGIN 334
#define FILL 335
#define LENGTH 336
#define CREATE_OBJECT_SYMBOLS 337
#define INPUT 338
#define GROUP 339
#define OUTPUT 340
#define CONSTRUCTORS 341
#define ALIGNMOD 342
#define AT 343
#define SUBALIGN 344
#define HIDDEN 345
#define PROVIDE 346
#define PROVIDE_HIDDEN 347
#define AS_NEEDED 348
#define CHIP 349
#define LIST 350
#define SECT 351
#define ABSOLUTE 352
#define LOAD 353
#define NEWLINE 354
#define ENDWORD 355
#define ORDER 356
#define NAMEWORD 357
#define ASSERT_K 358
#define LOG2CEIL 359
#define FORMAT 360
#define PUBLIC 361
#define DEFSYMEND 362
#define BASE 363
#define ALIAS 364
#define TRUNCATE 365
#define REL 366
#define INPUT_SCRIPT 367
#define INPUT_MRI_SCRIPT 368
#define INPUT_DEFSYM 369
#define CASE 370
#define EXTERN 371
#define START 372
#define VERS_TAG 373
#define VERS_IDENTIFIER 374
#define GLOBAL 375
#define LOCAL 376
#define VERSIONK 377
#define INPUT_VERSION_SCRIPT 378
#define KEEP 379
#define ONLY_IF_RO 380
#define ONLY_IF_RW 381
#define SPECIAL 382
#define INPUT_SECTION_FLAGS 383
#define ALIGN_WITH_INPUT 384
#define EXCLUDE_FILE 385
#define CONSTANT 386
#define INPUT_DYNAMIC_LIST 387




/* Copy the first part of user declarations.  */
#line 22 "ldgram.y"

/*

 */

#define DONTDECLARE_MALLOC

#include "sysdep.h"
#include "bfd.h"
#include "bfdlink.h"
#include "ld.h"
#include "ldexp.h"
#include "ldver.h"
#include "ldlang.h"
#include "ldfile.h"
#include "ldemul.h"
#include "ldmisc.h"
#include "ldmain.h"
#include "mri.h"
#include "ldctor.h"
#include "ldlex.h"

#ifndef YYDEBUG
#define YYDEBUG 1
#endif

static enum section_type sectype;
static lang_memory_region_type *region;

static bfd_boolean ldgram_had_keep = FALSE;
static char *ldgram_vers_current_lang = NULL;

#define ERROR_NAME_MAX 20
static char *error_names[ERROR_NAME_MAX];
static int error_index;
#define PUSH_ERROR(x) if (error_index < ERROR_NAME_MAX) error_names[error_index] = x; error_index++;
#define POP_ERROR()   error_index--;


/* Enabling traces.  */
#ifndef YYDEBUG
# define YYDEBUG 0
#endif

/* Enabling verbose error messages.  */
#ifdef YYERROR_VERBOSE
# undef YYERROR_VERBOSE
# define YYERROR_VERBOSE 1
#else
# define YYERROR_VERBOSE 0
#endif

/* Enabling the token table.  */
#ifndef YYTOKEN_TABLE
# define YYTOKEN_TABLE 0
#endif

#if ! defined YYSTYPE && ! defined YYSTYPE_IS_DECLARED
typedef union YYSTYPE
#line 60 "ldgram.y"
{
  bfd_vma integer;
  struct big_int
    {
      bfd_vma integer;
      char *str;
    } bigint;
  fill_type *fill;
  char *name;
  const char *cname;
  struct wildcard_spec wildcard;
  struct wildcard_list *wildcard_list;
  struct name_list *name_list;
  struct flag_info_list *flag_info_list;
  struct flag_info *flag_info;
  int token;
  union etree_union *etree;
  struct phdr_info
    {
      bfd_boolean filehdr;
      bfd_boolean phdrs;
      union etree_union *at;
      union etree_union *flags;
    } phdr;
  struct lang_nocrossref *nocrossref;
  struct lang_output_section_phdr_list *section_phdr;
  struct bfd_elf_version_deps *deflist;
  struct bfd_elf_version_expr *versyms;
  struct bfd_elf_version_tree *versnode;
}
/* Line 193 of yacc.c.  */
#line 430 "ldgram.c"
	YYSTYPE;
# define yystype YYSTYPE /* obsolescent; will be withdrawn */
# define YYSTYPE_IS_DECLARED 1
# define YYSTYPE_IS_TRIVIAL 1
#endif



/* Copy the second part of user declarations.  */


/* Line 216 of yacc.c.  */
#line 443 "ldgram.c"

#ifdef short
# undef short
#endif

#ifdef YYTYPE_UINT8
typedef YYTYPE_UINT8 yytype_uint8;
#else
typedef unsigned char yytype_uint8;
#endif

#ifdef YYTYPE_INT8
typedef YYTYPE_INT8 yytype_int8;
#elif (defined __STDC__ || defined __C99__FUNC__ \
     || defined __cplusplus || defined _MSC_VER)
typedef signed char yytype_int8;
#else
typedef short int yytype_int8;
#endif

#ifdef YYTYPE_UINT16
typedef YYTYPE_UINT16 yytype_uint16;
#else
typedef unsigned short int yytype_uint16;
#endif

#ifdef YYTYPE_INT16
typedef YYTYPE_INT16 yytype_int16;
#else
typedef short int yytype_int16;
#endif

#ifndef YYSIZE_T
# ifdef __SIZE_TYPE__
#  define YYSIZE_T __SIZE_TYPE__
# elif defined size_t
#  define YYSIZE_T size_t
# elif ! defined YYSIZE_T && (defined __STDC__ || defined __C99__FUNC__ \
     || defined __cplusplus || defined _MSC_VER)
#  include <stddef.h> /* INFRINGES ON USER NAME SPACE */
#  define YYSIZE_T size_t
# else
#  define YYSIZE_T unsigned int
# endif
#endif

#define YYSIZE_MAXIMUM ((YYSIZE_T) -1)

#ifndef YY_
# if YYENABLE_NLS
#  if ENABLE_NLS
#   include <libintl.h> /* INFRINGES ON USER NAME SPACE */
#   define YY_(msgid) dgettext ("bison-runtime", msgid)
#  endif
# endif
# ifndef YY_
#  define YY_(msgid) msgid
# endif
#endif

/* Suppress unused-variable warnings by "using" E.  */
#if ! defined lint || defined __GNUC__
# define YYUSE(e) ((void) (e))
#else
# define YYUSE(e) /* empty */
#endif

/* Identity function, used to suppress warnings about constant conditions.  */
#ifndef lint
# define YYID(n) (n)
#else
#if (defined __STDC__ || defined __C99__FUNC__ \
     || defined __cplusplus || defined _MSC_VER)
static int
YYID (int i)
#else
static int
YYID (i)
    int i;
#endif
{
  return i;
}
#endif

#if ! defined yyoverflow || YYERROR_VERBOSE

/* The parser invokes alloca or malloc; define the necessary symbols.  */

# ifdef YYSTACK_USE_ALLOCA
#  if YYSTACK_USE_ALLOCA
#   ifdef __GNUC__
#    define YYSTACK_ALLOC __builtin_alloca
#   elif defined __BUILTIN_VA_ARG_INCR
#    include <alloca.h> /* INFRINGES ON USER NAME SPACE */
#   elif defined _AIX
#    define YYSTACK_ALLOC __alloca
#   elif defined _MSC_VER
#    include <malloc.h> /* INFRINGES ON USER NAME SPACE */
#    define alloca _alloca
#   else
#    define YYSTACK_ALLOC alloca
#    if ! defined _ALLOCA_H && ! defined _STDLIB_H && (defined __STDC__ || defined __C99__FUNC__ \
     || defined __cplusplus || defined _MSC_VER)
#     include <stdlib.h> /* INFRINGES ON USER NAME SPACE */
#     ifndef _STDLIB_H
#      define _STDLIB_H 1
#     endif
#    endif
#   endif
#  endif
# endif

# ifdef YYSTACK_ALLOC
   /* Pacify GCC's `empty if-body' warning.  */
#  define YYSTACK_FREE(Ptr) do { /* empty */; } while (YYID (0))
#  ifndef YYSTACK_ALLOC_MAXIMUM
    /* The OS might guarantee only one guard page at the bottom of the stack,
       and a page size can be as small as 4096 bytes.  So we cannot safely
       invoke alloca (N) if N exceeds 4096.  Use a slightly smaller number
       to allow for a few compiler-allocated temporary stack slots.  */
#   define YYSTACK_ALLOC_MAXIMUM 4032 /* reasonable circa 2006 */
#  endif
# else
#  define YYSTACK_ALLOC YYMALLOC
#  define YYSTACK_FREE YYFREE
#  ifndef YYSTACK_ALLOC_MAXIMUM
#   define YYSTACK_ALLOC_MAXIMUM YYSIZE_MAXIMUM
#  endif
#  if (defined __cplusplus && ! defined _STDLIB_H \
       && ! ((defined YYMALLOC || defined malloc) \
	     && (defined YYFREE || defined free)))
#   include <stdlib.h> /* INFRINGES ON USER NAME SPACE */
#   ifndef _STDLIB_H
#    define _STDLIB_H 1
#   endif
#  endif
#  ifndef YYMALLOC
#   define YYMALLOC malloc
#   if ! defined malloc && ! defined _STDLIB_H && (defined __STDC__ || defined __C99__FUNC__ \
     || defined __cplusplus || defined _MSC_VER)
void *malloc (YYSIZE_T); /* INFRINGES ON USER NAME SPACE */
#   endif
#  endif
#  ifndef YYFREE
#   define YYFREE free
#   if ! defined free && ! defined _STDLIB_H && (defined __STDC__ || defined __C99__FUNC__ \
     || defined __cplusplus || defined _MSC_VER)
void free (void *); /* INFRINGES ON USER NAME SPACE */
#   endif
#  endif
# endif
#endif /* ! defined yyoverflow || YYERROR_VERBOSE */


#if (! defined yyoverflow \
     && (! defined __cplusplus \
	 || (defined YYSTYPE_IS_TRIVIAL && YYSTYPE_IS_TRIVIAL)))

/* A type that is properly aligned for any stack member.  */
union yyalloc
{
  yytype_int16 yyss;
  YYSTYPE yyvs;
  };

/* The size of the maximum gap between one aligned stack and the next.  */
# define YYSTACK_GAP_MAXIMUM (sizeof (union yyalloc) - 1)

/* The size of an array large to enough to hold all stacks, each with
   N elements.  */
# define YYSTACK_BYTES(N) \
     ((N) * (sizeof (yytype_int16) + sizeof (YYSTYPE)) \
      + YYSTACK_GAP_MAXIMUM)

/* Copy COUNT objects from FROM to TO.  The source and destination do
   not overlap.  */
# ifndef YYCOPY
#  if defined __GNUC__ && 1 < __GNUC__
#   define YYCOPY(To, From, Count) \
      __builtin_memcpy (To, From, (Count) * sizeof (*(From)))
#  else
#   define YYCOPY(To, From, Count)		\
      do					\
	{					\
	  YYSIZE_T yyi;				\
	  for (yyi = 0; yyi < (Count); yyi++)	\
	    (To)[yyi] = (From)[yyi];		\
	}					\
      while (YYID (0))
#  endif
# endif

/* Relocate STACK from its old location to the new one.  The
   local variables YYSIZE and YYSTACKSIZE give the old and new number of
   elements in the stack, and YYPTR gives the new location of the
   stack.  Advance YYPTR to a properly aligned location for the next
   stack.  */
# define YYSTACK_RELOCATE(Stack)					\
    do									\
      {									\
	YYSIZE_T yynewbytes;						\
	YYCOPY (&yyptr->Stack, Stack, yysize);				\
	Stack = &yyptr->Stack;						\
	yynewbytes = yystacksize * sizeof (*Stack) + YYSTACK_GAP_MAXIMUM; \
	yyptr += yynewbytes / sizeof (*yyptr);				\
      }									\
    while (YYID (0))

#endif

/* YYFINAL -- State number of the termination state.  */
#define YYFINAL  17
/* YYLAST -- Last index in YYTABLE.  */
#define YYLAST   1925

/* YYNTOKENS -- Number of terminals.  */
#define YYNTOKENS  156
/* YYNNTS -- Number of nonterminals.  */
#define YYNNTS  133
/* YYNRULES -- Number of rules.  */
#define YYNRULES  376
/* YYNRULES -- Number of states.  */
#define YYNSTATES  814

/* YYTRANSLATE(YYLEX) -- Bison symbol number corresponding to YYLEX.  */
#define YYUNDEFTOK  2
#define YYMAXUTOK   387

#define YYTRANSLATE(YYX)						\
  ((unsigned int) (YYX) <= YYMAXUTOK ? yytranslate[YYX] : YYUNDEFTOK)

/* YYTRANSLATE[YYLEX] -- Bison symbol number corresponding to YYLEX.  */
static const yytype_uint8 yytranslate[] =
{
       0,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,   154,     2,     2,     2,    34,    21,     2,
      37,   151,    32,    30,   149,    31,     2,    33,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,    16,   150,
      24,     6,    25,    15,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,   152,     2,   153,    20,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,    58,    19,    59,   155,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     1,     2,     3,     4,
       5,     7,     8,     9,    10,    11,    12,    13,    14,    17,
      18,    22,    23,    26,    27,    28,    29,    35,    36,    38,
      39,    40,    41,    42,    43,    44,    45,    46,    47,    48,
      49,    50,    51,    52,    53,    54,    55,    56,    57,    60,
      61,    62,    63,    64,    65,    66,    67,    68,    69,    70,
      71,    72,    73,    74,    75,    76,    77,    78,    79,    80,
      81,    82,    83,    84,    85,    86,    87,    88,    89,    90,
      91,    92,    93,    94,    95,    96,    97,    98,    99,   100,
     101,   102,   103,   104,   105,   106,   107,   108,   109,   110,
     111,   112,   113,   114,   115,   116,   117,   118,   119,   120,
     121,   122,   123,   124,   125,   126,   127,   128,   129,   130,
     131,   132,   133,   134,   135,   136,   137,   138,   139,   140,
     141,   142,   143,   144,   145,   146,   147,   148
};

#if YYDEBUG
/* YYPRHS[YYN] -- Index of the first RHS symbol of rule number YYN in
   YYRHS.  */
static const yytype_uint16 yyprhs[] =
{
       0,     0,     3,     6,     9,    12,    15,    18,    20,    21,
      26,    27,    30,    34,    35,    38,    43,    45,    47,    50,
      52,    57,    62,    66,    69,    74,    78,    83,    88,    93,
      98,   103,   106,   109,   112,   117,   122,   125,   128,   131,
     134,   135,   141,   144,   145,   149,   152,   153,   155,   159,
     161,   165,   166,   168,   172,   173,   176,   178,   181,   185,
     186,   189,   192,   193,   195,   197,   199,   201,   203,   205,
     207,   209,   211,   213,   218,   223,   228,   233,   242,   247,
     249,   251,   253,   258,   259,   265,   270,   271,   277,   282,
     287,   292,   296,   300,   307,   312,   313,   316,   318,   322,
     325,   327,   331,   334,   335,   341,   342,   350,   351,   358,
     363,   366,   369,   370,   375,   378,   379,   387,   389,   391,
     393,   395,   401,   403,   408,   413,   415,   420,   425,   430,
     438,   446,   454,   462,   467,   469,   473,   478,   481,   483,
     487,   489,   491,   494,   498,   503,   508,   514,   516,   517,
     523,   526,   528,   530,   532,   537,   539,   544,   549,   550,
     559,   560,   566,   569,   571,   572,   574,   576,   578,   580,
     582,   584,   586,   589,   590,   592,   594,   596,   598,   600,
     602,   604,   606,   608,   610,   614,   618,   625,   632,   639,
     641,   642,   647,   649,   650,   654,   656,   657,   665,   666,
     672,   676,   680,   681,   685,   687,   690,   692,   695,   700,
     705,   709,   713,   715,   720,   724,   725,   727,   729,   730,
     733,   737,   738,   741,   744,   748,   753,   756,   759,   762,
     766,   770,   774,   778,   782,   786,   790,   794,   798,   802,
     806,   810,   814,   818,   822,   826,   832,   836,   840,   845,
     847,   849,   854,   859,   864,   869,   874,   879,   884,   891,
     898,   905,   910,   917,   922,   924,   931,   938,   945,   950,
     955,   960,   964,   965,   970,   971,   976,   977,   979,   980,
     985,   986,   988,   990,   992,   993,   994,   995,   996,   997,
     998,  1019,  1020,  1021,  1022,  1023,  1024,  1043,  1044,  1045,
    1053,  1054,  1060,  1062,  1064,  1066,  1068,  1070,  1074,  1075,
    1078,  1082,  1085,  1092,  1103,  1106,  1108,  1109,  1111,  1114,
    1115,  1116,  1120,  1121,  1122,  1123,  1124,  1136,  1141,  1142,
    1145,  1146,  1147,  1154,  1156,  1157,  1161,  1167,  1168,  1172,
    1173,  1176,  1178,  1181,  1186,  1189,  1190,  1193,  1194,  1200,
    1202,  1205,  1210,  1216,  1223,  1225,  1228,  1229,  1232,  1237,
    1242,  1251,  1253,  1255,  1259,  1263,  1264,  1274,  1275,  1283,
    1285,  1289,  1291,  1295,  1297,  1301,  1302
};

/* YYRHS -- A `-1'-separated list of the rules' RHS.  */
static const yytype_int16 yyrhs[] =
{
     157,     0,    -1,   128,   173,    -1,   129,   161,    -1,   139,
     277,    -1,   148,   272,    -1,   130,   159,    -1,     4,    -1,
      -1,   160,     4,     6,   232,    -1,    -1,   162,   163,    -1,
     163,   164,   115,    -1,    -1,   110,   232,    -1,   110,   232,
     149,   232,    -1,     4,    -1,   111,    -1,   117,   166,    -1,
     116,    -1,   122,     4,     6,   232,    -1,   122,     4,   149,
     232,    -1,   122,     4,   232,    -1,   121,     4,    -1,   112,
       4,   149,   232,    -1,   112,     4,   232,    -1,   112,     4,
       6,   232,    -1,    38,     4,     6,   232,    -1,    38,     4,
     149,   232,    -1,   103,     4,     6,   232,    -1,   103,     4,
     149,   232,    -1,   113,   168,    -1,   114,   167,    -1,   118,
       4,    -1,   125,     4,   149,     4,    -1,   125,     4,   149,
       3,    -1,   124,   232,    -1,   126,     3,    -1,   131,   169,
      -1,   132,   170,    -1,    -1,    67,   158,   165,   163,    36,
      -1,   133,     4,    -1,    -1,   166,   149,     4,    -1,   166,
       4,    -1,    -1,     4,    -1,   167,   149,     4,    -1,     4,
      -1,   168,   149,     4,    -1,    -1,     4,    -1,   169,   149,
       4,    -1,    -1,   171,   172,    -1,     4,    -1,   172,     4,
      -1,   172,   149,     4,    -1,    -1,   174,   175,    -1,   175,
     176,    -1,    -1,   212,    -1,   185,    -1,   264,    -1,   223,
      -1,   224,    -1,   226,    -1,   228,    -1,   187,    -1,   279,
      -1,   150,    -1,    77,    37,     4,   151,    -1,    78,    37,
     158,   151,    -1,   101,    37,   158,   151,    -1,    61,    37,
       4,   151,    -1,    61,    37,     4,   149,     4,   149,     4,
     151,    -1,    63,    37,     4,   151,    -1,    62,    -1,    65,
      -1,    64,    -1,    99,    37,   179,   151,    -1,    -1,   100,
     177,    37,   179,   151,    -1,    79,    37,   158,   151,    -1,
      -1,    67,   158,   178,   175,    36,    -1,    93,    37,   229,
     151,    -1,    94,    37,   229,   151,    -1,   132,    37,   170,
     151,    -1,    48,    49,     4,    -1,    48,    50,     4,    -1,
      69,    37,     4,   149,     4,   151,    -1,    70,    37,     4,
     151,    -1,    -1,   180,   181,    -1,     4,    -1,   181,   149,
       4,    -1,   181,     4,    -1,     5,    -1,   181,   149,     5,
      -1,   181,     5,    -1,    -1,   109,    37,   182,   181,   151,
      -1,    -1,   181,   149,   109,    37,   183,   181,   151,    -1,
      -1,   181,   109,    37,   184,   181,   151,    -1,    46,    58,
     186,    59,    -1,   186,   239,    -1,   186,   187,    -1,    -1,
      80,    37,     4,   151,    -1,   210,   209,    -1,    -1,   119,
     188,    37,   232,   149,     4,   151,    -1,     4,    -1,    32,
      -1,    15,    -1,   189,    -1,   146,    37,   195,   151,   189,
      -1,   190,    -1,    54,    37,   190,   151,    -1,    56,    37,
     190,   151,    -1,   190,    -1,    54,    37,   190,   151,    -1,
      55,    37,   190,   151,    -1,    56,    37,   190,   151,    -1,
      54,    37,    55,    37,   190,   151,   151,    -1,    54,    37,
      54,    37,   190,   151,   151,    -1,    55,    37,    54,    37,
     190,   151,   151,    -1,    55,    37,    55,    37,   190,   151,
     151,    -1,    57,    37,   190,   151,    -1,     4,    -1,   193,
      21,     4,    -1,   144,    37,   193,   151,    -1,   195,   189,
      -1,   189,    -1,   196,   211,   192,    -1,   192,    -1,     4,
      -1,   194,     4,    -1,   152,   196,   153,    -1,   194,   152,
     196,   153,    -1,   191,    37,   196,   151,    -1,   194,   191,
      37,   196,   151,    -1,   197,    -1,    -1,   140,    37,   199,
     197,   151,    -1,   210,   209,    -1,    98,    -1,   150,    -1,
     102,    -1,    54,    37,   102,   151,    -1,   198,    -1,   205,
      37,   230,   151,    -1,    96,    37,   206,   151,    -1,    -1,
     119,   201,    37,   232,   149,     4,   151,   209,    -1,    -1,
      67,   158,   202,   204,    36,    -1,   203,   200,    -1,   200,
      -1,    -1,   203,    -1,    41,    -1,    42,    -1,    43,    -1,
      44,    -1,    45,    -1,   230,    -1,     6,   206,    -1,    -1,
      14,    -1,    13,    -1,    12,    -1,    11,    -1,    10,    -1,
       9,    -1,     8,    -1,     7,    -1,   150,    -1,   149,    -1,
       4,     6,   230,    -1,     4,   208,   230,    -1,   106,    37,
       4,     6,   230,   151,    -1,   107,    37,     4,     6,   230,
     151,    -1,   108,    37,     4,     6,   230,   151,    -1,   149,
      -1,    -1,    68,    58,   213,    59,    -1,   214,    -1,    -1,
     214,   211,   215,    -1,   215,    -1,    -1,     4,   216,   220,
      16,   218,   211,   219,    -1,    -1,    67,   158,   217,   213,
      36,    -1,    95,     6,   230,    -1,    97,     6,   230,    -1,
      -1,    37,   221,   151,    -1,   222,    -1,   221,   222,    -1,
       4,    -1,   154,     4,    -1,    88,    37,   158,   151,    -1,
      89,    37,   225,   151,    -1,    89,    37,   151,    -1,   225,
     211,   158,    -1,   158,    -1,    90,    37,   227,   151,    -1,
     227,   211,   158,    -1,    -1,    91,    -1,    92,    -1,    -1,
       4,   229,    -1,     4,   149,   229,    -1,    -1,   231,   232,
      -1,    31,   232,    -1,    37,   232,   151,    -1,    81,    37,
     232,   151,    -1,   154,   232,    -1,    30,   232,    -1,   155,
     232,    -1,   232,    32,   232,    -1,   232,    33,   232,    -1,
     232,    34,   232,    -1,   232,    30,   232,    -1,   232,    31,
     232,    -1,   232,    29,   232,    -1,   232,    28,   232,    -1,
     232,    23,   232,    -1,   232,    22,   232,    -1,   232,    27,
     232,    -1,   232,    26,   232,    -1,   232,    24,   232,    -1,
     232,    25,   232,    -1,   232,    21,   232,    -1,   232,    20,
     232,    -1,   232,    19,   232,    -1,   232,    15,   232,    16,
     232,    -1,   232,    18,   232,    -1,   232,    17,   232,    -1,
      76,    37,     4,   151,    -1,     3,    -1,    60,    -1,    83,
      37,     4,   151,    -1,    82,    37,     4,   151,    -1,    84,
      37,     4,   151,    -1,    85,    37,     4,   151,    -1,   147,
      37,     4,   151,    -1,   113,    37,   232,   151,    -1,    38,
      37,   232,   151,    -1,    38,    37,   232,   149,   232,   151,
      -1,    51,    37,   232,   149,   232,   151,    -1,    52,    37,
     232,   149,   232,   151,    -1,    53,    37,   232,   151,    -1,
      66,    37,     4,   149,   232,   151,    -1,    39,    37,   232,
     151,    -1,     4,    -1,    86,    37,   232,   149,   232,   151,
      -1,    87,    37,   232,   149,   232,   151,    -1,   119,    37,
     232,   149,     4,   151,    -1,    95,    37,     4,   151,    -1,
      97,    37,     4,   151,    -1,   120,    37,   232,   151,    -1,
     104,    25,     4,    -1,    -1,   104,    37,   232,   151,    -1,
      -1,    38,    37,   232,   151,    -1,    -1,   145,    -1,    -1,
     105,    37,   232,   151,    -1,    -1,   141,    -1,   142,    -1,
     143,    -1,    -1,    -1,    -1,    -1,    -1,    -1,     4,   240,
     255,   234,   235,   236,   237,   241,   238,    58,   242,   204,
      59,   243,   258,   233,   259,   207,   244,   211,    -1,    -1,
      -1,    -1,    -1,    -1,    75,   245,   256,   257,   234,   237,
     246,    58,   247,   260,    59,   248,   258,   233,   259,   207,
     249,   211,    -1,    -1,    -1,   100,   250,   255,   251,    58,
     186,    59,    -1,    -1,    67,   158,   252,   186,    36,    -1,
      71,    -1,    72,    -1,    73,    -1,    74,    -1,    75,    -1,
      37,   253,   151,    -1,    -1,    37,   151,    -1,   232,   254,
      16,    -1,   254,    16,    -1,    40,    37,   232,   151,   254,
      16,    -1,    40,    37,   232,   151,    39,    37,   232,   151,
     254,    16,    -1,   232,    16,    -1,    16,    -1,    -1,    93,
      -1,    25,     4,    -1,    -1,    -1,   259,    16,     4,    -1,
      -1,    -1,    -1,    -1,   260,     4,   261,    58,   204,    59,
     262,   259,   207,   263,   211,    -1,    47,    58,   265,    59,
      -1,    -1,   265,   266,    -1,    -1,    -1,     4,   267,   269,
     270,   268,   150,    -1,   232,    -1,    -1,     4,   271,   270,
      -1,   104,    37,   232,   151,   270,    -1,    -1,    37,   232,
     151,    -1,    -1,   273,   274,    -1,   275,    -1,   274,   275,
      -1,    58,   276,    59,   150,    -1,   285,   150,    -1,    -1,
     278,   281,    -1,    -1,   280,   138,    58,   281,    59,    -1,
     282,    -1,   281,   282,    -1,    58,   284,    59,   150,    -1,
     134,    58,   284,    59,   150,    -1,   134,    58,   284,    59,
     283,   150,    -1,   134,    -1,   283,   134,    -1,    -1,   285,
     150,    -1,   136,    16,   285,   150,    -1,   137,    16,   285,
     150,    -1,   136,    16,   285,   150,   137,    16,   285,   150,
      -1,   135,    -1,     4,    -1,   285,   150,   135,    -1,   285,
     150,     4,    -1,    -1,   285,   150,   132,     4,    58,   286,
     285,   288,    59,    -1,    -1,   132,     4,    58,   287,   285,
     288,    59,    -1,   136,    -1,   285,   150,   136,    -1,   137,
      -1,   285,   150,   137,    -1,   132,    -1,   285,   150,   132,
      -1,    -1,   150,    -1
};

/* YYRLINE[YYN] -- source line where rule number YYN was defined.  */
static const yytype_uint16 yyrline[] =
{
       0,   166,   166,   167,   168,   169,   170,   174,   178,   178,
     188,   188,   201,   202,   206,   207,   208,   211,   214,   215,
     216,   218,   220,   222,   224,   226,   228,   230,   232,   234,
     236,   238,   239,   240,   242,   244,   246,   248,   250,   251,
     253,   252,   256,   258,   262,   263,   264,   268,   270,   274,
     276,   281,   282,   283,   288,   288,   293,   295,   297,   302,
     302,   308,   309,   314,   315,   316,   317,   318,   319,   320,
     321,   322,   323,   324,   326,   328,   330,   333,   335,   337,
     339,   341,   343,   345,   344,   348,   351,   350,   354,   358,
     362,   363,   365,   367,   369,   374,   374,   379,   382,   385,
     388,   391,   394,   398,   397,   403,   402,   408,   407,   415,
     419,   420,   421,   425,   427,   428,   428,   436,   440,   444,
     451,   458,   468,   469,   474,   482,   483,   488,   493,   498,
     503,   508,   513,   518,   525,   543,   564,   577,   586,   597,
     606,   617,   626,   635,   639,   648,   652,   660,   662,   661,
     668,   669,   673,   674,   679,   684,   685,   690,   694,   694,
     698,   697,   704,   705,   708,   710,   714,   716,   718,   720,
     722,   727,   734,   736,   740,   742,   744,   746,   748,   750,
     752,   754,   759,   759,   764,   768,   776,   780,   784,   792,
     792,   796,   799,   799,   802,   803,   808,   807,   813,   812,
     819,   827,   835,   836,   840,   841,   845,   847,   852,   857,
     858,   863,   865,   871,   873,   875,   879,   881,   887,   890,
     899,   910,   910,   916,   918,   920,   922,   924,   926,   929,
     931,   933,   935,   937,   939,   941,   943,   945,   947,   949,
     951,   953,   955,   957,   959,   961,   963,   965,   967,   969,
     971,   974,   976,   978,   980,   982,   984,   986,   988,   990,
     992,   994,   996,  1005,  1007,  1009,  1011,  1013,  1015,  1017,
    1019,  1025,  1026,  1030,  1031,  1035,  1036,  1040,  1041,  1045,
    1046,  1050,  1051,  1052,  1053,  1056,  1061,  1064,  1070,  1072,
    1056,  1079,  1081,  1083,  1088,  1090,  1078,  1100,  1102,  1100,
    1108,  1107,  1114,  1115,  1116,  1117,  1118,  1122,  1123,  1124,
    1128,  1129,  1134,  1135,  1140,  1141,  1146,  1147,  1152,  1154,
    1159,  1162,  1175,  1179,  1184,  1186,  1177,  1194,  1197,  1199,
    1203,  1204,  1203,  1213,  1258,  1261,  1274,  1283,  1286,  1293,
    1293,  1305,  1306,  1310,  1314,  1323,  1323,  1337,  1337,  1347,
    1348,  1352,  1356,  1360,  1367,  1371,  1379,  1382,  1386,  1390,
    1394,  1401,  1405,  1409,  1413,  1418,  1417,  1431,  1430,  1440,
    1444,  1448,  1452,  1456,  1460,  1466,  1468
};
#endif

#if YYDEBUG || YYERROR_VERBOSE || YYTOKEN_TABLE
/* YYTNAME[SYMBOL-NUM] -- String name of the symbol SYMBOL-NUM.
   First, the terminals, then, starting at YYNTOKENS, nonterminals.  */
static const char *const yytname[] =
{
  "$end", "error", "$undefined", "INT", "NAME", "LNAME", "'='", "OREQ",
  "ANDEQ", "RSHIFTEQ", "LSHIFTEQ", "DIVEQ", "MULTEQ", "MINUSEQ", "PLUSEQ",
  "'?'", "':'", "OROR", "ANDAND", "'|'", "'^'", "'&'", "NE", "EQ", "'<'",
  "'>'", "GE", "LE", "RSHIFT", "LSHIFT", "'+'", "'-'", "'*'", "'/'", "'%'",
  "UNARY", "END", "'('", "ALIGN_K", "BLOCK", "BIND", "QUAD", "SQUAD",
  "LONG", "SHORT", "BYTE", "SECTIONS", "PHDRS", "INSERT_K", "AFTER",
  "BEFORE", "DATA_SEGMENT_ALIGN", "DATA_SEGMENT_RELRO_END",
  "DATA_SEGMENT_END", "SORT_BY_NAME", "SORT_BY_ALIGNMENT", "SORT_NONE",
  "SORT_BY_INIT_PRIORITY", "'{'", "'}'", "SIZEOF_HEADERS", "OUTPUT_FORMAT",
  "FORCE_COMMON_ALLOCATION", "OUTPUT_ARCH", "INHIBIT_COMMON_ALLOCATION",
  "FORCE_GROUP_ALLOCATION", "SEGMENT_START", "INCLUDE", "MEMORY",
  "REGION_ALIAS", "LD_FEATURE", "NOLOAD", "DSECT", "COPY", "INFO",
  "OVERLAY", "DEFINED", "TARGET_K", "SEARCH_DIR", "MAP", "ENTRY", "NEXT",
  "SIZEOF", "ALIGNOF", "ADDR", "LOADADDR", "MAX_K", "MIN_K", "STARTUP",
  "HLL", "SYSLIB", "FLOAT", "NOFLOAT", "NOCROSSREFS", "NOCROSSREFS_TO",
  "ORIGIN", "FILL", "LENGTH", "CREATE_OBJECT_SYMBOLS", "INPUT", "GROUP",
  "OUTPUT", "CONSTRUCTORS", "ALIGNMOD", "AT", "SUBALIGN", "HIDDEN",
  "PROVIDE", "PROVIDE_HIDDEN", "AS_NEEDED", "CHIP", "LIST", "SECT",
  "ABSOLUTE", "LOAD", "NEWLINE", "ENDWORD", "ORDER", "NAMEWORD",
  "ASSERT_K", "LOG2CEIL", "FORMAT", "PUBLIC", "DEFSYMEND", "BASE", "ALIAS",
  "TRUNCATE", "REL", "INPUT_SCRIPT", "INPUT_MRI_SCRIPT", "INPUT_DEFSYM",
  "CASE", "EXTERN", "START", "VERS_TAG", "VERS_IDENTIFIER", "GLOBAL",
  "LOCAL", "VERSIONK", "INPUT_VERSION_SCRIPT", "KEEP", "ONLY_IF_RO",
  "ONLY_IF_RW", "SPECIAL", "INPUT_SECTION_FLAGS", "ALIGN_WITH_INPUT",
  "EXCLUDE_FILE", "CONSTANT", "INPUT_DYNAMIC_LIST", "','", "';'", "')'",
  "'['", "']'", "'!'", "'~'", "$accept", "file", "filename", "defsym_expr",
  "@1", "mri_script_file", "@2", "mri_script_lines", "mri_script_command",
  "@3", "ordernamelist", "mri_load_name_list", "mri_abs_name_list",
  "casesymlist", "extern_name_list", "@4", "extern_name_list_body",
  "script_file", "@5", "ifile_list", "ifile_p1", "@6", "@7", "input_list",
  "@8", "input_list1", "@9", "@10", "@11", "sections", "sec_or_group_p1",
  "statement_anywhere", "@12", "wildcard_name", "wildcard_maybe_exclude",
  "filename_spec", "section_name_spec", "sect_flag_list", "sect_flags",
  "exclude_name_list", "section_name_list", "input_section_spec_no_keep",
  "input_section_spec", "@13", "statement", "@14", "@15", "statement_list",
  "statement_list_opt", "length", "fill_exp", "fill_opt", "assign_op",
  "end", "assignment", "opt_comma", "memory", "memory_spec_list_opt",
  "memory_spec_list", "memory_spec", "@16", "@17", "origin_spec",
  "length_spec", "attributes_opt", "attributes_list", "attributes_string",
  "startup", "high_level_library", "high_level_library_NAME_list",
  "low_level_library", "low_level_library_NAME_list",
  "floating_point_support", "nocrossref_list", "mustbe_exp", "@18", "exp",
  "memspec_at_opt", "opt_at", "opt_align", "opt_align_with_input",
  "opt_subalign", "sect_constraint", "section", "@19", "@20", "@21", "@22",
  "@23", "@24", "@25", "@26", "@27", "@28", "@29", "@30", "@31", "type",
  "atype", "opt_exp_with_type", "opt_exp_without_type", "opt_nocrossrefs",
  "memspec_opt", "phdr_opt", "overlay_section", "@32", "@33", "@34",
  "phdrs", "phdr_list", "phdr", "@35", "@36", "phdr_type",
  "phdr_qualifiers", "phdr_val", "dynamic_list_file", "@37",
  "dynamic_list_nodes", "dynamic_list_node", "dynamic_list_tag",
  "version_script_file", "@38", "version", "@39", "vers_nodes",
  "vers_node", "verdep", "vers_tag", "vers_defns", "@40", "@41",
  "opt_semicolon", 0
};
#endif

# ifdef YYPRINT
/* YYTOKNUM[YYLEX-NUM] -- Internal token number corresponding to
   token YYLEX-NUM.  */
static const yytype_uint16 yytoknum[] =
{
       0,   256,   257,   258,   259,   260,    61,   261,   262,   263,
     264,   265,   266,   267,   268,    63,    58,   269,   270,   124,
      94,    38,   271,   272,    60,    62,   273,   274,   275,   276,
      43,    45,    42,    47,    37,   277,   278,    40,   279,   280,
     281,   282,   283,   284,   285,   286,   287,   288,   289,   290,
     291,   292,   293,   294,   295,   296,   297,   298,   123,   125,
     299,   300,   301,   302,   303,   304,   305,   306,   307,   308,
     309,   310,   311,   312,   313,   314,   315,   316,   317,   318,
     319,   320,   321,   322,   323,   324,   325,   326,   327,   328,
     329,   330,   331,   332,   333,   334,   335,   336,   337,   338,
     339,   340,   341,   342,   343,   344,   345,   346,   347,   348,
     349,   350,   351,   352,   353,   354,   355,   356,   357,   358,
     359,   360,   361,   362,   363,   364,   365,   366,   367,   368,
     369,   370,   371,   372,   373,   374,   375,   376,   377,   378,
     379,   380,   381,   382,   383,   384,   385,   386,   387,    44,
      59,    41,    91,    93,    33,   126
};
# endif

/* YYR1[YYN] -- Symbol number of symbol that rule YYN derives.  */
static const yytype_uint16 yyr1[] =
{
       0,   156,   157,   157,   157,   157,   157,   158,   160,   159,
     162,   161,   163,   163,   164,   164,   164,   164,   164,   164,
     164,   164,   164,   164,   164,   164,   164,   164,   164,   164,
     164,   164,   164,   164,   164,   164,   164,   164,   164,   164,
     165,   164,   164,   164,   166,   166,   166,   167,   167,   168,
     168,   169,   169,   169,   171,   170,   172,   172,   172,   174,
     173,   175,   175,   176,   176,   176,   176,   176,   176,   176,
     176,   176,   176,   176,   176,   176,   176,   176,   176,   176,
     176,   176,   176,   177,   176,   176,   178,   176,   176,   176,
     176,   176,   176,   176,   176,   180,   179,   181,   181,   181,
     181,   181,   181,   182,   181,   183,   181,   184,   181,   185,
     186,   186,   186,   187,   187,   188,   187,   189,   189,   189,
     190,   190,   191,   191,   191,   192,   192,   192,   192,   192,
     192,   192,   192,   192,   193,   193,   194,   195,   195,   196,
     196,   197,   197,   197,   197,   197,   197,   198,   199,   198,
     200,   200,   200,   200,   200,   200,   200,   200,   201,   200,
     202,   200,   203,   203,   204,   204,   205,   205,   205,   205,
     205,   206,   207,   207,   208,   208,   208,   208,   208,   208,
     208,   208,   209,   209,   210,   210,   210,   210,   210,   211,
     211,   212,   213,   213,   214,   214,   216,   215,   217,   215,
     218,   219,   220,   220,   221,   221,   222,   222,   223,   224,
     224,   225,   225,   226,   227,   227,   228,   228,   229,   229,
     229,   231,   230,   232,   232,   232,   232,   232,   232,   232,
     232,   232,   232,   232,   232,   232,   232,   232,   232,   232,
     232,   232,   232,   232,   232,   232,   232,   232,   232,   232,
     232,   232,   232,   232,   232,   232,   232,   232,   232,   232,
     232,   232,   232,   232,   232,   232,   232,   232,   232,   232,
     232,   233,   233,   234,   234,   235,   235,   236,   236,   237,
     237,   238,   238,   238,   238,   240,   241,   242,   243,   244,
     239,   245,   246,   247,   248,   249,   239,   250,   251,   239,
     252,   239,   253,   253,   253,   253,   253,   254,   254,   254,
     255,   255,   255,   255,   256,   256,   257,   257,   258,   258,
     259,   259,   260,   261,   262,   263,   260,   264,   265,   265,
     267,   268,   266,   269,   270,   270,   270,   271,   271,   273,
     272,   274,   274,   275,   276,   278,   277,   280,   279,   281,
     281,   282,   282,   282,   283,   283,   284,   284,   284,   284,
     284,   285,   285,   285,   285,   286,   285,   287,   285,   285,
     285,   285,   285,   285,   285,   288,   288
};

/* YYR2[YYN] -- Number of symbols composing right hand side of rule YYN.  */
static const yytype_uint8 yyr2[] =
{
       0,     2,     2,     2,     2,     2,     2,     1,     0,     4,
       0,     2,     3,     0,     2,     4,     1,     1,     2,     1,
       4,     4,     3,     2,     4,     3,     4,     4,     4,     4,
       4,     2,     2,     2,     4,     4,     2,     2,     2,     2,
       0,     5,     2,     0,     3,     2,     0,     1,     3,     1,
       3,     0,     1,     3,     0,     2,     1,     2,     3,     0,
       2,     2,     0,     1,     1,     1,     1,     1,     1,     1,
       1,     1,     1,     4,     4,     4,     4,     8,     4,     1,
       1,     1,     4,     0,     5,     4,     0,     5,     4,     4,
       4,     3,     3,     6,     4,     0,     2,     1,     3,     2,
       1,     3,     2,     0,     5,     0,     7,     0,     6,     4,
       2,     2,     0,     4,     2,     0,     7,     1,     1,     1,
       1,     5,     1,     4,     4,     1,     4,     4,     4,     7,
       7,     7,     7,     4,     1,     3,     4,     2,     1,     3,
       1,     1,     2,     3,     4,     4,     5,     1,     0,     5,
       2,     1,     1,     1,     4,     1,     4,     4,     0,     8,
       0,     5,     2,     1,     0,     1,     1,     1,     1,     1,
       1,     1,     2,     0,     1,     1,     1,     1,     1,     1,
       1,     1,     1,     1,     3,     3,     6,     6,     6,     1,
       0,     4,     1,     0,     3,     1,     0,     7,     0,     5,
       3,     3,     0,     3,     1,     2,     1,     2,     4,     4,
       3,     3,     1,     4,     3,     0,     1,     1,     0,     2,
       3,     0,     2,     2,     3,     4,     2,     2,     2,     3,
       3,     3,     3,     3,     3,     3,     3,     3,     3,     3,
       3,     3,     3,     3,     3,     5,     3,     3,     4,     1,
       1,     4,     4,     4,     4,     4,     4,     4,     6,     6,
       6,     4,     6,     4,     1,     6,     6,     6,     4,     4,
       4,     3,     0,     4,     0,     4,     0,     1,     0,     4,
       0,     1,     1,     1,     0,     0,     0,     0,     0,     0,
      20,     0,     0,     0,     0,     0,    18,     0,     0,     7,
       0,     5,     1,     1,     1,     1,     1,     3,     0,     2,
       3,     2,     6,    10,     2,     1,     0,     1,     2,     0,
       0,     3,     0,     0,     0,     0,    11,     4,     0,     2,
       0,     0,     6,     1,     0,     3,     5,     0,     3,     0,
       2,     1,     2,     4,     2,     0,     2,     0,     5,     1,
       2,     4,     5,     6,     1,     2,     0,     2,     4,     4,
       8,     1,     1,     3,     3,     0,     9,     0,     7,     1,
       3,     1,     3,     1,     3,     0,     1
};

/* YYDEFACT[STATE-NAME] -- Default rule to reduce with in state
   STATE-NUM when YYTABLE doesn't specify something else to do.  Zero
   means the default is an error.  */
static const yytype_uint16 yydefact[] =
{
       0,    59,    10,     8,   345,   339,     0,     2,    62,     3,
      13,     6,     0,     4,     0,     5,     0,     1,    60,    11,
       0,   356,     0,   346,   349,     0,   340,   341,     0,     0,
       0,     0,     0,    79,     0,    81,    80,     0,     0,     0,
       0,     0,     0,     0,     0,     0,     0,     0,   216,   217,
       0,     0,     0,    83,     0,     0,     0,     0,   115,     0,
      72,    61,    64,    70,     0,    63,    66,    67,    68,    69,
      65,    71,     0,    16,     0,     0,     0,     0,    17,     0,
       0,     0,    19,    46,     0,     0,     0,     0,     0,     0,
      51,    54,     0,     0,     0,   362,   373,   361,   369,   371,
       0,     0,   356,   350,   369,   371,     0,     0,   342,   221,
     181,   180,   179,   178,   177,   176,   175,   174,   221,   112,
     328,     0,     0,     0,     0,     7,    86,   193,     0,     0,
       0,     0,     0,     0,     0,     0,   215,   218,   218,    95,
       0,     0,     0,     0,     0,     0,    54,   183,   182,   114,
       0,     0,    40,     0,   249,   264,     0,     0,     0,     0,
       0,     0,     0,     0,   250,     0,     0,     0,     0,     0,
       0,     0,     0,     0,     0,     0,     0,     0,     0,     0,
       0,     0,    14,     0,    49,    31,    47,    32,    18,    33,
      23,     0,    36,     0,    37,    52,    38,    39,     0,    42,
      12,     9,     0,     0,     0,     0,   357,     0,     0,   344,
     184,     0,   185,     0,     0,    91,    92,     0,     0,    62,
     196,     0,     0,   190,   195,     0,     0,     0,     0,     0,
       0,     0,   210,   212,   190,   190,   218,     0,     0,     0,
       0,    95,     0,     0,     0,     0,     0,     0,     0,     0,
       0,    13,     0,     0,   227,   223,     0,     0,     0,     0,
       0,     0,     0,     0,     0,     0,     0,     0,     0,     0,
       0,     0,     0,     0,     0,     0,     0,   226,   228,     0,
       0,     0,     0,     0,     0,     0,     0,     0,     0,     0,
       0,     0,     0,     0,     0,     0,     0,     0,     0,     0,
       0,    25,     0,     0,    45,     0,     0,     0,    22,     0,
       0,    56,    55,   367,     0,     0,   351,   364,   374,   363,
     370,   372,     0,   343,   222,   285,   109,     0,   291,   297,
     111,   110,   330,   327,   329,     0,    76,    78,   347,   202,
     198,   191,   189,     0,     0,    94,    73,    74,    85,   113,
     208,   209,     0,   213,     0,   218,   219,    88,    89,    82,
      97,   100,     0,    96,     0,    75,   221,   221,   221,     0,
      90,     0,    27,    28,    43,    29,    30,   224,     0,     0,
       0,     0,     0,     0,     0,     0,     0,     0,     0,     0,
       0,     0,     0,     0,     0,     0,     0,     0,     0,   247,
     246,   244,   243,   242,   237,   236,   240,   241,   239,   238,
     235,   234,   232,   233,   229,   230,   231,    15,    26,    24,
      50,    48,    44,    20,    21,    35,    34,    53,    57,     0,
       0,   358,   359,     0,   354,   352,     0,   308,   300,     0,
     308,     0,     0,    87,     0,     0,   193,   194,     0,   211,
     214,   220,   103,    99,   102,     0,     0,    84,     0,     0,
       0,     0,   348,    41,     0,   257,   263,     0,     0,   261,
       0,   248,   225,   252,   251,   253,   254,     0,     0,   268,
     269,   256,     0,   270,   255,     0,    58,   375,   372,   365,
     355,   353,     0,     0,   308,     0,   274,   112,   315,     0,
     316,   298,   333,   334,     0,   206,     0,     0,   204,     0,
       0,    93,     0,   107,    98,   101,     0,   186,   187,   188,
       0,     0,     0,     0,     0,     0,     0,     0,   245,   376,
       0,     0,     0,   302,   303,   304,   305,   306,   309,     0,
       0,     0,     0,   311,     0,   276,     0,   314,   317,   274,
       0,   337,     0,   331,     0,   207,   203,   205,     0,   190,
     199,     0,     0,   105,   116,   258,   259,   260,   262,   265,
     266,   267,   368,     0,   375,   307,     0,   310,     0,     0,
     278,   301,   280,   112,     0,   334,     0,     0,    77,   221,
       0,   104,     0,     0,   360,     0,   308,     0,     0,   277,
     280,     0,   292,     0,     0,   335,     0,   332,   200,     0,
     197,   108,     0,   366,     0,     0,   273,     0,   286,     0,
       0,   299,   338,   334,   221,   106,     0,   312,   275,   284,
       0,   293,   336,   201,     0,   281,   282,   283,     0,   279,
     322,   308,   287,     0,     0,   164,   323,   294,   313,   141,
     119,   118,   166,   167,   168,   169,   170,     0,     0,     0,
       0,   151,   153,   158,     0,     0,     0,   152,     0,   120,
     122,     0,     0,   147,   155,   163,   165,     0,     0,     0,
       0,   319,     0,     0,   160,   221,     0,   148,     0,     0,
     117,     0,     0,     0,     0,   125,   140,   190,     0,   142,
       0,     0,     0,   162,   288,   221,   150,   164,     0,   272,
       0,     0,     0,   164,     0,   171,     0,     0,   134,     0,
     138,     0,     0,     0,     0,     0,   143,     0,   190,     0,
     190,     0,   319,     0,     0,   318,     0,   320,   154,   123,
     124,     0,   157,     0,   117,     0,     0,   136,     0,   137,
       0,     0,     0,     0,     0,     0,     0,     0,   139,   145,
     144,   190,   272,   156,   324,     0,   173,   161,     0,   149,
     135,   121,     0,     0,   126,     0,     0,   127,   128,   133,
     146,   320,   320,   271,   221,     0,   295,     0,     0,     0,
       0,     0,   173,   173,   172,   321,   190,     0,     0,     0,
       0,     0,   289,   325,   296,   159,   130,   129,   131,   132,
     190,   190,   290,   326
};

/* YYDEFGOTO[NTERM-NUM].  */
static const yytype_int16 yydefgoto[] =
{
      -1,     6,   126,    11,    12,     9,    10,    19,    93,   251,
     188,   187,   185,   196,   197,   198,   312,     7,     8,    18,
      61,   140,   219,   239,   240,   363,   512,   593,   562,    62,
     213,   330,   145,   669,   670,   671,   696,   719,   672,   721,
     697,   673,   674,   717,   675,   686,   713,   676,   677,   678,
     714,   786,   118,   149,    64,   727,    65,   222,   223,   224,
     339,   446,   559,   610,   445,   507,   508,    66,    67,   234,
      68,   235,    69,   237,   715,   211,   256,   737,   545,   580,
     600,   602,   638,   331,   437,   629,   645,   732,   810,   439,
     620,   640,   681,   796,   440,   550,   497,   539,   495,   496,
     500,   549,   709,   766,   643,   680,   782,   811,    70,   214,
     334,   441,   587,   503,   553,   585,    15,    16,    26,    27,
     106,    13,    14,    71,    72,    23,    24,   436,   100,   101,
     532,   430,   530
};

/* YYPACT[STATE-NUM] -- Index in YYTABLE of the portion describing
   STATE-NUM.  */
#define YYPACT_NINF -650
static const yytype_int16 yypact[] =
{
     -53,  -650,  -650,  -650,  -650,  -650,    66,  -650,  -650,  -650,
    -650,  -650,    74,  -650,   -11,  -650,    47,  -650,   891,   335,
     106,   104,    61,   -11,  -650,   111,    47,  -650,   899,    80,
      92,   201,   122,  -650,   125,  -650,  -650,   140,   130,   191,
     198,   205,   208,   220,   226,   238,   247,   260,  -650,  -650,
     264,   291,   297,  -650,   298,   299,   301,   303,  -650,   304,
    -650,  -650,  -650,  -650,   173,  -650,  -650,  -650,  -650,  -650,
    -650,  -650,   204,  -650,   339,   140,   340,   736,  -650,   341,
     346,   351,  -650,  -650,   352,   353,   354,   736,   355,   358,
     368,  -650,   370,   262,   736,  -650,   374,  -650,   363,   364,
     327,   242,   104,  -650,  -650,  -650,   336,   244,  -650,  -650,
    -650,  -650,  -650,  -650,  -650,  -650,  -650,  -650,  -650,  -650,
    -650,   395,   399,   400,   402,  -650,  -650,    31,   403,   405,
     406,   140,   140,   409,   140,    27,  -650,   410,   410,  -650,
     378,   140,   412,   413,   414,   382,  -650,  -650,  -650,  -650,
     365,    22,  -650,    42,  -650,  -650,   736,   736,   736,   385,
     387,   388,   396,   397,  -650,   398,   404,   407,   417,   418,
     425,   432,   433,   434,   435,   436,   437,   439,   440,   441,
     736,   736,   847,   345,  -650,   287,  -650,   288,    15,  -650,
    -650,   481,  1874,   290,  -650,  -650,   294,  -650,   475,  -650,
    -650,  1874,   422,   111,   111,   331,   117,   427,   338,   117,
    -650,   736,  -650,   416,    50,  -650,  -650,   107,   342,  -650,
    -650,   140,   430,    -2,  -650,   348,   344,   347,   357,   359,
     375,   376,  -650,  -650,   132,   147,    40,   377,   379,   380,
      34,  -650,   386,   484,   496,   497,   736,   389,   -11,   736,
     736,  -650,   736,   736,  -650,  -650,  1103,   736,   736,   736,
     736,   736,   500,   501,   736,   502,   511,   521,   525,   736,
     736,   532,   534,   736,   736,   736,   535,  -650,  -650,   736,
     736,   736,   736,   736,   736,   736,   736,   736,   736,   736,
     736,   736,   736,   736,   736,   736,   736,   736,   736,   736,
     736,  1874,   539,   540,  -650,   541,   736,   736,  1874,   127,
     542,  -650,    41,  -650,   401,   408,  -650,  -650,   544,  -650,
    -650,  -650,   -78,  -650,  1874,   899,  -650,   140,  -650,  -650,
    -650,  -650,  -650,  -650,  -650,   546,  -650,  -650,   969,   515,
    -650,  -650,  -650,    31,   549,  -650,  -650,  -650,  -650,  -650,
    -650,  -650,   140,  -650,   140,   410,  -650,  -650,  -650,  -650,
    -650,  -650,   517,    45,   420,  -650,  -650,  -650,  -650,  1528,
    -650,    -7,  1874,  1874,  1715,  1874,  1874,  -650,   901,  1123,
    1548,  1568,  1143,   411,   421,  1163,   424,   426,   428,   429,
    1588,  1608,   431,   442,  1183,  1661,  1203,   448,  1834,  1891,
    1083,   729,  1217,  1351,   752,   752,   337,   337,   337,   337,
     258,   258,   167,   167,  -650,  -650,  -650,  1874,  1874,  1874,
    -650,  -650,  -650,  1874,  1874,  -650,  -650,  -650,  -650,   551,
     111,   142,   117,   498,  -650,  -650,   -60,   566,  -650,   651,
     566,   736,   443,  -650,     4,   543,    31,  -650,   451,  -650,
    -650,  -650,  -650,  -650,  -650,   524,    36,  -650,   465,   469,
     470,   569,  -650,  -650,   736,  -650,  -650,   736,   736,  -650,
     736,  -650,  -650,  -650,  -650,  -650,  -650,   736,   736,  -650,
    -650,  -650,   577,  -650,  -650,   736,  -650,   472,   567,  -650,
    -650,  -650,   234,   547,  1766,   570,   486,  -650,  -650,  1854,
     492,  -650,  1874,    12,   587,  -650,   619,     2,  -650,   529,
     589,  -650,    34,  -650,  -650,  -650,   592,  -650,  -650,  -650,
     480,  1238,  1258,  1278,  1298,  1318,  1338,   482,  1874,   117,
     575,   111,   111,  -650,  -650,  -650,  -650,  -650,  -650,   489,
     736,   195,   621,  -650,   604,   605,   591,  -650,  -650,   486,
     586,   608,   609,  -650,   505,  -650,  -650,  -650,   653,   508,
    -650,    16,    34,  -650,  -650,  -650,  -650,  -650,  -650,  -650,
    -650,  -650,  -650,   510,   472,  -650,  1373,  -650,   736,   625,
     519,  -650,   560,  -650,   736,    12,   736,   518,  -650,  -650,
     572,  -650,    19,    34,   117,   611,   265,  1393,   736,  -650,
     560,   635,  -650,  1659,  1413,  -650,  1433,  -650,  -650,   667,
    -650,  -650,    28,  -650,   637,   659,  -650,  1453,  -650,   736,
     618,  -650,  -650,    12,  -650,  -650,   736,  -650,  -650,    91,
    1473,  -650,  -650,  -650,  1508,  -650,  -650,  -650,   620,  -650,
    -650,   640,  -650,    67,   664,   796,  -650,  -650,  -650,   601,
    -650,  -650,  -650,  -650,  -650,  -650,  -650,   646,   647,   140,
     650,  -650,  -650,  -650,   655,   656,   657,  -650,    85,  -650,
    -650,   658,    14,  -650,  -650,  -650,   796,   641,   668,   173,
     638,   676,    49,    79,  -650,  -650,   669,  -650,   703,    69,
    -650,   671,   672,   675,   677,  -650,  -650,   -29,    85,   678,
     679,    85,   681,  -650,  -650,  -650,  -650,   796,   715,   622,
     571,   573,   574,   796,   578,  -650,   736,    11,  -650,     1,
    -650,    10,    78,    81,    79,    79,  -650,    85,   162,    79,
     146,    85,   676,   579,   682,  -650,   698,  -650,  -650,  -650,
    -650,   692,  -650,  1681,   580,   593,   738,  -650,    69,  -650,
     706,   708,   596,   728,   731,   639,   642,   643,  -650,  -650,
    -650,   163,   622,  -650,  -650,   765,    86,  -650,   768,  -650,
    -650,  -650,    79,    79,  -650,    79,    79,  -650,  -650,  -650,
    -650,  -650,  -650,  -650,  -650,   787,  -650,   644,   648,   652,
     662,   663,    86,    86,  -650,  -650,   508,   173,   665,   673,
     674,   683,  -650,  -650,  -650,  -650,  -650,  -650,  -650,  -650,
     508,   508,  -650,  -650
};

/* YYPGOTO[NTERM-NUM].  */
static const yytype_int16 yypgoto[] =
{
    -650,  -650,   -72,  -650,  -650,  -650,  -650,   550,  -650,  -650,
    -650,  -650,  -650,  -650,   661,  -650,  -650,  -650,  -650,   585,
    -650,  -650,  -650,   556,  -650,  -475,  -650,  -650,  -650,  -650,
    -461,   -13,  -650,  -445,  1048,   120,    82,  -650,  -650,  -650,
    -643,    93,  -650,  -650,   139,  -650,  -650,  -650,  -616,  -650,
      24,  -466,  -650,  -649,  -383,  -222,  -650,   381,  -650,   483,
    -650,  -650,  -650,  -650,  -650,  -650,   322,  -650,  -650,  -650,
    -650,  -650,  -650,  -129,  -107,  -650,   -77,    68,   283,  -650,
    -650,   235,  -650,  -650,  -650,  -650,  -650,  -650,  -650,  -650,
    -650,  -650,  -650,  -650,  -650,  -650,  -650,  -650,  -467,   419,
    -650,  -650,   110,  -449,  -650,  -650,  -650,  -650,  -650,  -650,
    -650,  -650,  -650,  -650,  -524,  -650,  -650,  -650,  -650,   810,
    -650,  -650,  -650,  -650,  -650,   595,   -19,  -650,   742,   -18,
    -650,  -650,   271
};

/* YYTABLE[YYPACT[STATE-NUM]].  What to do in state STATE-NUM.  If
   positive, shift that token.  If negative, reduce the rule which
   number is the opposite.  If zero, do what YYDEFACT says.
   If YYTABLE_NINF, syntax error.  */
#define YYTABLE_NINF -348
static const yytype_int16 yytable[] =
{
     182,   343,   210,   152,   103,    63,   505,   107,   505,   238,
     192,   212,   352,   354,   690,   744,   551,   201,   699,   304,
     453,   454,   746,   453,   454,   650,   650,   542,   249,   650,
     706,   125,   453,   454,  -192,   220,   546,   561,   360,   361,
     514,   515,   651,   651,   236,   428,   651,    21,   252,   453,
     454,    21,   462,   690,   332,   728,   434,  -192,   730,   228,
     229,   605,   231,   233,   650,   700,    17,   658,   700,   242,
     658,   646,   435,   690,   490,     1,     2,     3,    20,   254,
     255,   651,   690,   690,   650,   690,     4,   592,   761,   690,
     491,   734,   784,   650,   650,     5,   650,   741,   221,   632,
     650,   651,   785,   277,   278,    25,   301,   356,    95,   333,
     651,   651,    94,   651,   308,    95,   552,   651,   612,   102,
     342,   317,   603,    22,   726,   455,   647,    22,   455,   615,
     425,   426,   750,   751,   324,   753,   754,   455,   119,   691,
     692,   693,   694,   362,   125,   516,   317,   342,   805,   340,
     120,   710,   747,   556,   455,   665,   506,   666,   506,   123,
     666,   748,   124,   668,   305,   456,   701,   591,   456,   369,
     611,   250,   372,   373,   644,   375,   376,   456,   232,   625,
     378,   379,   380,   381,   382,   314,   315,   385,   127,   355,
     429,   253,   390,   391,   456,   666,   394,   395,   396,   295,
     296,   297,   398,   399,   400,   401,   402,   403,   404,   405,
     406,   407,   408,   409,   410,   411,   412,   413,   414,   415,
     416,   417,   418,   419,   666,   666,   451,   666,   128,   423,
     424,   666,   635,   636,   637,   129,    96,   154,   155,    97,
      98,    99,   130,    96,   720,   131,    97,   104,   105,   318,
     121,   122,   319,   320,   321,   438,   335,   132,   336,   458,
     459,   460,   679,   133,   156,   157,   533,   534,   535,   536,
     537,   158,   159,   160,   318,   134,   749,   319,   320,   488,
     449,   342,   450,   351,   135,   161,   162,   163,   293,   294,
     295,   296,   297,   679,   164,   342,   342,   136,   353,   760,
     165,   137,   541,   771,   614,   533,   534,   535,   536,   537,
     166,   342,   342,   759,   780,   167,   168,   169,   170,   171,
     172,   173,   147,   148,   679,    63,   802,   803,   138,   174,
     679,   175,   792,   793,   139,   141,   142,   590,   143,    73,
     144,   146,   150,   151,   153,   183,   538,   176,   154,   155,
     184,   299,   103,   177,   178,   186,   189,   190,   191,   193,
     494,   194,   499,   494,   502,   291,   292,   293,   294,   295,
     296,   297,   195,    74,   199,   156,   157,   200,   202,   203,
     204,   179,   158,   159,   160,   538,   205,   521,   180,   181,
     522,   523,   206,   524,   209,   208,   161,   162,   163,   215,
     525,   526,    75,   216,   217,   164,   218,   225,   528,   226,
     227,   165,   487,   230,   236,   241,   243,   244,   245,   246,
     325,   166,   257,   248,   258,   259,   167,   168,   169,   170,
     171,   172,   173,   260,   261,   262,   302,   303,    76,   309,
     174,   263,   175,   310,   264,    77,    78,    79,    80,    81,
     -43,    82,    83,    84,   265,   266,    85,    86,   176,    87,
      88,    89,   267,   576,   177,   178,    90,    91,    92,   268,
     269,   270,   271,   272,   273,   326,   274,   275,   276,   311,
     313,   316,   608,   327,   154,   155,   322,   306,   323,   341,
     366,   328,   179,   337,   300,   345,    44,   344,   346,   180,
     181,   597,   367,   368,   383,   384,   386,   604,   347,   606,
     348,   156,   157,   573,   574,   387,   329,   633,   158,   159,
     160,   617,    55,    56,    57,   388,   349,   350,   357,   389,
     358,   359,   161,   162,   163,    58,   392,   365,   393,   397,
     370,   164,   630,   420,   421,   422,   427,   165,   433,   634,
     442,   431,   444,   448,   452,   486,   489,   166,   432,   509,
     470,   513,   167,   168,   169,   170,   171,   172,   173,   154,
     155,   457,   471,   520,   804,   473,   174,   474,   175,   475,
     476,   527,   479,   531,   540,   548,   543,   684,   812,   813,
     544,   554,   504,   480,   176,   325,   156,   157,   733,   484,
     177,   178,   511,   492,   159,   160,   493,   109,   110,   111,
     112,   113,   114,   115,   116,   117,   517,   161,   162,   163,
     518,   519,   529,   555,   558,   560,   164,   581,   179,   563,
     307,   564,   165,   571,   572,   180,   181,   577,  -117,   743,
     575,   578,   166,   579,   583,   584,   586,   167,   168,   169,
     170,   171,   172,   173,   154,   155,   588,   342,   327,   589,
     594,   174,   598,   175,   599,   601,   328,   498,   607,   609,
     613,    44,   619,   624,   626,   627,   631,   541,   642,   176,
     648,   156,   157,   682,   683,   177,   178,   685,   158,   159,
     160,   329,   687,   688,   689,   698,   707,    55,    56,    57,
     704,   708,   161,   162,   163,   705,   716,   718,   722,   723,
      58,   164,   724,   179,   725,  -117,   729,   165,   731,   735,
     180,   181,   738,   765,   739,   740,   736,   166,   767,   742,
     763,  -141,   167,   168,   169,   170,   171,   172,   173,   154,
     155,   764,   770,   772,   769,   773,   174,   774,   175,   283,
     284,   285,   286,   287,   288,   289,   290,   291,   292,   293,
     294,   295,   296,   297,   176,   775,   156,   157,   776,   783,
     177,   178,   787,   158,   159,   160,   287,   288,   289,   290,
     291,   292,   293,   294,   295,   296,   297,   161,   162,   163,
     777,   795,   702,   778,   779,   797,   164,   364,   179,   798,
     649,   374,   165,   799,   338,   180,   181,   247,   794,   758,
     745,   650,   166,   800,   801,   703,   806,   167,   168,   169,
     170,   171,   172,   173,   807,   808,   447,   510,   651,   557,
     781,   174,   582,   175,   809,   618,   108,   652,   653,   654,
     655,   656,   762,   371,   207,   595,     0,     0,     0,   176,
     657,     0,   658,     0,     0,   177,   178,     0,     0,   501,
       0,     0,   279,   659,   280,   281,   282,   283,   284,   285,
     286,   287,   288,   289,   290,   291,   292,   293,   294,   295,
     296,   297,     0,   179,     0,     0,     0,     0,     0,     0,
     180,   181,   660,     0,   661,    28,     0,     0,   662,     0,
       0,     0,    55,    56,    57,   109,   110,   111,   112,   113,
     114,   115,   116,   117,     0,   663,   279,     0,   280,   281,
     282,   283,   284,   285,   286,   287,   288,   289,   290,   291,
     292,   293,   294,   295,   296,   297,   664,    29,    30,    31,
     665,     0,   666,     0,     0,     0,   667,     0,   668,     0,
       0,     0,    32,    33,    34,    35,    36,     0,    37,    38,
      39,    40,     0,     0,     0,     0,     0,     0,    41,    42,
      43,    44,     0,    28,     0,     0,     0,     0,     0,    45,
      46,    47,    48,    49,    50,    51,     0,     0,     0,     0,
      52,    53,    54,     0,     0,     0,   298,    55,    56,    57,
       0,     0,     0,     0,     0,   443,     0,     0,     0,     0,
      58,     0,     0,     0,     0,    29,    30,    31,     0,     0,
       0,     0,     0,    59,     0,     0,     0,     0,     0,  -347,
      32,    33,    34,    35,    36,     0,    37,    38,    39,    40,
       0,    60,     0,     0,     0,     0,    41,    42,    43,    44,
     464,     0,   465,     0,     0,     0,     0,    45,    46,    47,
      48,    49,    50,    51,     0,     0,     0,     0,    52,    53,
      54,     0,     0,     0,     0,    55,    56,    57,     0,     0,
       0,     0,     0,     0,     0,     0,     0,     0,    58,     0,
       0,     0,     0,     0,     0,     0,     0,     0,     0,     0,
       0,    59,   282,   283,   284,   285,   286,   287,   288,   289,
     290,   291,   292,   293,   294,   295,   296,   297,   279,    60,
     280,   281,   282,   283,   284,   285,   286,   287,   288,   289,
     290,   291,   292,   293,   294,   295,   296,   297,   279,     0,
     280,   281,   282,   283,   284,   285,   286,   287,   288,   289,
     290,   291,   292,   293,   294,   295,   296,   297,   279,     0,
     280,   281,   282,   283,   284,   285,   286,   287,   288,   289,
     290,   291,   292,   293,   294,   295,   296,   297,   279,     0,
     280,   281,   282,   283,   284,   285,   286,   287,   288,   289,
     290,   291,   292,   293,   294,   295,   296,   297,   279,     0,
     280,   281,   282,   283,   284,   285,   286,   287,   288,   289,
     290,   291,   292,   293,   294,   295,   296,   297,   279,     0,
     280,   281,   282,   283,   284,   285,   286,   287,   288,   289,
     290,   291,   292,   293,   294,   295,   296,   297,   284,   285,
     286,   287,   288,   289,   290,   291,   292,   293,   294,   295,
     296,   297,     0,   279,   377,   280,   281,   282,   283,   284,
     285,   286,   287,   288,   289,   290,   291,   292,   293,   294,
     295,   296,   297,   279,   466,   280,   281,   282,   283,   284,
     285,   286,   287,   288,   289,   290,   291,   292,   293,   294,
     295,   296,   297,   279,   469,   280,   281,   282,   283,   284,
     285,   286,   287,   288,   289,   290,   291,   292,   293,   294,
     295,   296,   297,   279,   472,   280,   281,   282,   283,   284,
     285,   286,   287,   288,   289,   290,   291,   292,   293,   294,
     295,   296,   297,   279,   481,   280,   281,   282,   283,   284,
     285,   286,   287,   288,   289,   290,   291,   292,   293,   294,
     295,   296,   297,   279,   483,   280,   281,   282,   283,   284,
     285,   286,   287,   288,   289,   290,   291,   292,   293,   294,
     295,   296,   297,   285,   286,   287,   288,   289,   290,   291,
     292,   293,   294,   295,   296,   297,     0,     0,   279,   565,
     280,   281,   282,   283,   284,   285,   286,   287,   288,   289,
     290,   291,   292,   293,   294,   295,   296,   297,   279,   566,
     280,   281,   282,   283,   284,   285,   286,   287,   288,   289,
     290,   291,   292,   293,   294,   295,   296,   297,   279,   567,
     280,   281,   282,   283,   284,   285,   286,   287,   288,   289,
     290,   291,   292,   293,   294,   295,   296,   297,   279,   568,
     280,   281,   282,   283,   284,   285,   286,   287,   288,   289,
     290,   291,   292,   293,   294,   295,   296,   297,   279,   569,
     280,   281,   282,   283,   284,   285,   286,   287,   288,   289,
     290,   291,   292,   293,   294,   295,   296,   297,   279,   570,
     280,   281,   282,   283,   284,   285,   286,   287,   288,   289,
     290,   291,   292,   293,   294,   295,   296,   297,     0,     0,
       0,     0,     0,     0,     0,     0,     0,     0,     0,     0,
       0,     0,     0,   279,   596,   280,   281,   282,   283,   284,
     285,   286,   287,   288,   289,   290,   291,   292,   293,   294,
     295,   296,   297,   279,   616,   280,   281,   282,   283,   284,
     285,   286,   287,   288,   289,   290,   291,   292,   293,   294,
     295,   296,   297,   279,   622,   280,   281,   282,   283,   284,
     285,   286,   287,   288,   289,   290,   291,   292,   293,   294,
     295,   296,   297,   279,   623,   280,   281,   282,   283,   284,
     285,   286,   287,   288,   289,   290,   291,   292,   293,   294,
     295,   296,   297,   279,   628,   280,   281,   282,   283,   284,
     285,   286,   287,   288,   289,   290,   291,   292,   293,   294,
     295,   296,   297,   279,   639,   280,   281,   282,   283,   284,
     285,   286,   287,   288,   289,   290,   291,   292,   293,   294,
     295,   296,   297,     0,     0,     0,     0,     0,     0,     0,
       0,     0,     0,     0,     0,     0,     0,     0,     0,   641,
       0,     0,     0,   325,     0,     0,     0,     0,     0,     0,
       0,     0,     0,     0,     0,     0,   279,   461,   280,   281,
     282,   283,   284,   285,   286,   287,   288,   289,   290,   291,
     292,   293,   294,   295,   296,   297,   279,   467,   280,   281,
     282,   283,   284,   285,   286,   287,   288,   289,   290,   291,
     292,   293,   294,   295,   296,   297,   695,   468,   621,    73,
       0,     0,     0,     0,     0,     0,   327,     0,     0,     0,
     711,   712,     0,     0,   328,     0,     0,   477,     0,    44,
       0,     0,     0,     0,     0,     0,   695,     0,     0,   695,
       0,   463,     0,    74,     0,     0,     0,   478,     0,   329,
       0,     0,     0,     0,     0,    55,    56,    57,     0,     0,
     752,   755,   756,   757,     0,   695,     0,   711,    58,   695,
       0,   279,    75,   280,   281,   282,   283,   284,   285,   286,
     287,   288,   289,   290,   291,   292,   293,   294,   295,   296,
     297,     0,     0,   541,     0,     0,     0,     0,     0,     0,
     482,     0,     0,     0,     0,     0,     0,     0,    76,     0,
     788,   789,     0,   790,   791,    77,    78,    79,    80,    81,
     768,    82,    83,    84,     0,     0,    85,    86,     0,    87,
      88,    89,     0,     0,     0,     0,    90,    91,    92,   279,
     485,   280,   281,   282,   283,   284,   285,   286,   287,   288,
     289,   290,   291,   292,   293,   294,   295,   296,   297,   279,
     547,   280,   281,   282,   283,   284,   285,   286,   287,   288,
     289,   290,   291,   292,   293,   294,   295,   296,   297,   279,
       0,   280,   281,   282,   283,   284,   285,   286,   287,   288,
     289,   290,   291,   292,   293,   294,   295,   296,   297,   281,
     282,   283,   284,   285,   286,   287,   288,   289,   290,   291,
     292,   293,   294,   295,   296,   297
};

static const yytype_int16 yycheck[] =
{
      77,   223,   109,    75,    23,    18,     4,    25,     4,   138,
      87,   118,   234,   235,     4,     4,     4,    94,     4,     4,
       4,     5,    21,     4,     5,    15,    15,   494,     6,    15,
     679,     4,     4,     5,    36,     4,   497,   512,     4,     5,
       4,     5,    32,    32,     4,     4,    32,    58,     6,     4,
       5,    58,    59,     4,     4,   698,   134,    59,   701,   131,
     132,   585,   134,   135,    15,    54,     0,    56,    54,   141,
      56,     4,   150,     4,   134,   128,   129,   130,     4,   156,
     157,    32,     4,     4,    15,     4,   139,   562,   731,     4,
     150,   707,     6,    15,    15,   148,    15,   713,    67,   623,
      15,    32,    16,   180,   181,    58,   183,   236,     4,    59,
      32,    32,     6,    32,   191,     4,   104,    32,   593,    58,
     149,     4,   583,   134,   153,   109,    59,   134,   109,   596,
       3,     4,    54,    55,   211,    54,    55,   109,    58,    54,
      55,    56,    57,   109,     4,   109,     4,   149,   797,   221,
      58,   102,   151,   151,   109,   144,   154,   146,   154,    37,
     146,   151,    37,   152,   149,   149,   152,   151,   149,   246,
     151,   149,   249,   250,   641,   252,   253,   149,   151,   151,
     257,   258,   259,   260,   261,   203,   204,   264,    58,   149,
     149,   149,   269,   270,   149,   146,   273,   274,   275,    32,
      33,    34,   279,   280,   281,   282,   283,   284,   285,   286,
     287,   288,   289,   290,   291,   292,   293,   294,   295,   296,
     297,   298,   299,   300,   146,   146,   355,   146,    37,   306,
     307,   146,   141,   142,   143,    37,   132,     3,     4,   135,
     136,   137,    37,   132,   689,    37,   135,   136,   137,   132,
      49,    50,   135,   136,   137,   327,   149,    37,   151,   366,
     367,   368,   645,    37,    30,    31,    71,    72,    73,    74,
      75,    37,    38,    39,   132,    37,   721,   135,   136,   137,
     352,   149,   354,   151,    37,    51,    52,    53,    30,    31,
      32,    33,    34,   676,    60,   149,   149,    37,   151,   153,
      66,    37,    37,   748,    39,    71,    72,    73,    74,    75,
      76,   149,   149,   151,   151,    81,    82,    83,    84,    85,
      86,    87,   149,   150,   707,   338,   792,   793,    37,    95,
     713,    97,   781,   782,    37,    37,    37,   559,    37,     4,
      37,    37,   138,     4,     4,     4,   151,   113,     3,     4,
       4,     6,   371,   119,   120,     4,     4,     4,     4,     4,
     437,     3,   439,   440,   441,    28,    29,    30,    31,    32,
      33,    34,     4,    38,     4,    30,    31,   115,     4,    16,
      16,   147,    37,    38,    39,   151,    59,   464,   154,   155,
     467,   468,   150,   470,   150,    59,    51,    52,    53,     4,
     477,   478,    67,     4,     4,    60,     4,     4,   485,     4,
       4,    66,   430,     4,     4,    37,     4,     4,     4,    37,
       4,    76,    37,    58,    37,    37,    81,    82,    83,    84,
      85,    86,    87,    37,    37,    37,   149,   149,   103,   149,
      95,    37,    97,   149,    37,   110,   111,   112,   113,   114,
     115,   116,   117,   118,    37,    37,   121,   122,   113,   124,
     125,   126,    37,   540,   119,   120,   131,   132,   133,    37,
      37,    37,    37,    37,    37,    59,    37,    37,    37,     4,
      58,   150,   589,    67,     3,     4,    59,     6,   150,    59,
       6,    75,   147,   151,   149,   151,    80,   149,   151,   154,
     155,   578,     6,     6,     4,     4,     4,   584,   151,   586,
     151,    30,    31,   531,   532,     4,   100,   624,    37,    38,
      39,   598,   106,   107,   108,     4,   151,   151,   151,     4,
     151,   151,    51,    52,    53,   119,     4,   151,     4,     4,
     151,    60,   619,     4,     4,     4,     4,    66,     4,   626,
       4,   150,    37,     4,    37,     4,    58,    76,   150,    16,
     149,    37,    81,    82,    83,    84,    85,    86,    87,     3,
       4,   151,   151,     4,   796,   151,    95,   151,    97,   151,
     151,     4,   151,    16,    37,    93,    16,   659,   810,   811,
     104,     4,   149,   151,   113,     4,    30,    31,   705,   151,
     119,   120,   151,    37,    38,    39,    40,     6,     7,     8,
       9,    10,    11,    12,    13,    14,   151,    51,    52,    53,
     151,   151,   150,     4,    95,    36,    60,    36,   147,    37,
     149,   151,    66,   151,    59,   154,   155,    16,    37,   716,
     151,    37,    76,    38,    58,    37,    37,    81,    82,    83,
      84,    85,    86,    87,     3,     4,   151,   149,    67,     6,
     150,    95,    37,    97,   145,   105,    75,    16,   150,    97,
      59,    80,    37,     6,    37,    16,    58,    37,    58,   113,
      16,    30,    31,    37,    37,   119,   120,    37,    37,    38,
      39,   100,    37,    37,    37,    37,    58,   106,   107,   108,
      59,    25,    51,    52,    53,    37,    37,     4,    37,    37,
     119,    60,    37,   147,    37,    37,    37,    66,    37,     4,
     154,   155,   151,    25,   151,   151,   104,    76,    36,   151,
     151,   151,    81,    82,    83,    84,    85,    86,    87,     3,
       4,    59,     4,    37,   151,    37,    95,   151,    97,    20,
      21,    22,    23,    24,    25,    26,    27,    28,    29,    30,
      31,    32,    33,    34,   113,    37,    30,    31,    37,     4,
     119,   120,     4,    37,    38,    39,    24,    25,    26,    27,
      28,    29,    30,    31,    32,    33,    34,    51,    52,    53,
     151,     4,   672,   151,   151,   151,    60,   241,   147,   151,
       4,   251,    66,   151,   219,   154,   155,   146,   784,   727,
     717,    15,    76,   151,   151,   676,   151,    81,    82,    83,
      84,    85,    86,    87,   151,   151,   343,   446,    32,   507,
     762,    95,   549,    97,   151,   600,    26,    41,    42,    43,
      44,    45,   732,   248,   102,   574,    -1,    -1,    -1,   113,
      54,    -1,    56,    -1,    -1,   119,   120,    -1,    -1,   440,
      -1,    -1,    15,    67,    17,    18,    19,    20,    21,    22,
      23,    24,    25,    26,    27,    28,    29,    30,    31,    32,
      33,    34,    -1,   147,    -1,    -1,    -1,    -1,    -1,    -1,
     154,   155,    96,    -1,    98,     4,    -1,    -1,   102,    -1,
      -1,    -1,   106,   107,   108,     6,     7,     8,     9,    10,
      11,    12,    13,    14,    -1,   119,    15,    -1,    17,    18,
      19,    20,    21,    22,    23,    24,    25,    26,    27,    28,
      29,    30,    31,    32,    33,    34,   140,    46,    47,    48,
     144,    -1,   146,    -1,    -1,    -1,   150,    -1,   152,    -1,
      -1,    -1,    61,    62,    63,    64,    65,    -1,    67,    68,
      69,    70,    -1,    -1,    -1,    -1,    -1,    -1,    77,    78,
      79,    80,    -1,     4,    -1,    -1,    -1,    -1,    -1,    88,
      89,    90,    91,    92,    93,    94,    -1,    -1,    -1,    -1,
      99,   100,   101,    -1,    -1,    -1,   149,   106,   107,   108,
      -1,    -1,    -1,    -1,    -1,    36,    -1,    -1,    -1,    -1,
     119,    -1,    -1,    -1,    -1,    46,    47,    48,    -1,    -1,
      -1,    -1,    -1,   132,    -1,    -1,    -1,    -1,    -1,   138,
      61,    62,    63,    64,    65,    -1,    67,    68,    69,    70,
      -1,   150,    -1,    -1,    -1,    -1,    77,    78,    79,    80,
     149,    -1,   151,    -1,    -1,    -1,    -1,    88,    89,    90,
      91,    92,    93,    94,    -1,    -1,    -1,    -1,    99,   100,
     101,    -1,    -1,    -1,    -1,   106,   107,   108,    -1,    -1,
      -1,    -1,    -1,    -1,    -1,    -1,    -1,    -1,   119,    -1,
      -1,    -1,    -1,    -1,    -1,    -1,    -1,    -1,    -1,    -1,
      -1,   132,    19,    20,    21,    22,    23,    24,    25,    26,
      27,    28,    29,    30,    31,    32,    33,    34,    15,   150,
      17,    18,    19,    20,    21,    22,    23,    24,    25,    26,
      27,    28,    29,    30,    31,    32,    33,    34,    15,    -1,
      17,    18,    19,    20,    21,    22,    23,    24,    25,    26,
      27,    28,    29,    30,    31,    32,    33,    34,    15,    -1,
      17,    18,    19,    20,    21,    22,    23,    24,    25,    26,
      27,    28,    29,    30,    31,    32,    33,    34,    15,    -1,
      17,    18,    19,    20,    21,    22,    23,    24,    25,    26,
      27,    28,    29,    30,    31,    32,    33,    34,    15,    -1,
      17,    18,    19,    20,    21,    22,    23,    24,    25,    26,
      27,    28,    29,    30,    31,    32,    33,    34,    15,    -1,
      17,    18,    19,    20,    21,    22,    23,    24,    25,    26,
      27,    28,    29,    30,    31,    32,    33,    34,    21,    22,
      23,    24,    25,    26,    27,    28,    29,    30,    31,    32,
      33,    34,    -1,    15,   151,    17,    18,    19,    20,    21,
      22,    23,    24,    25,    26,    27,    28,    29,    30,    31,
      32,    33,    34,    15,   151,    17,    18,    19,    20,    21,
      22,    23,    24,    25,    26,    27,    28,    29,    30,    31,
      32,    33,    34,    15,   151,    17,    18,    19,    20,    21,
      22,    23,    24,    25,    26,    27,    28,    29,    30,    31,
      32,    33,    34,    15,   151,    17,    18,    19,    20,    21,
      22,    23,    24,    25,    26,    27,    28,    29,    30,    31,
      32,    33,    34,    15,   151,    17,    18,    19,    20,    21,
      22,    23,    24,    25,    26,    27,    28,    29,    30,    31,
      32,    33,    34,    15,   151,    17,    18,    19,    20,    21,
      22,    23,    24,    25,    26,    27,    28,    29,    30,    31,
      32,    33,    34,    22,    23,    24,    25,    26,    27,    28,
      29,    30,    31,    32,    33,    34,    -1,    -1,    15,   151,
      17,    18,    19,    20,    21,    22,    23,    24,    25,    26,
      27,    28,    29,    30,    31,    32,    33,    34,    15,   151,
      17,    18,    19,    20,    21,    22,    23,    24,    25,    26,
      27,    28,    29,    30,    31,    32,    33,    34,    15,   151,
      17,    18,    19,    20,    21,    22,    23,    24,    25,    26,
      27,    28,    29,    30,    31,    32,    33,    34,    15,   151,
      17,    18,    19,    20,    21,    22,    23,    24,    25,    26,
      27,    28,    29,    30,    31,    32,    33,    34,    15,   151,
      17,    18,    19,    20,    21,    22,    23,    24,    25,    26,
      27,    28,    29,    30,    31,    32,    33,    34,    15,   151,
      17,    18,    19,    20,    21,    22,    23,    24,    25,    26,
      27,    28,    29,    30,    31,    32,    33,    34,    -1,    -1,
      -1,    -1,    -1,    -1,    -1,    -1,    -1,    -1,    -1,    -1,
      -1,    -1,    -1,    15,   151,    17,    18,    19,    20,    21,
      22,    23,    24,    25,    26,    27,    28,    29,    30,    31,
      32,    33,    34,    15,   151,    17,    18,    19,    20,    21,
      22,    23,    24,    25,    26,    27,    28,    29,    30,    31,
      32,    33,    34,    15,   151,    17,    18,    19,    20,    21,
      22,    23,    24,    25,    26,    27,    28,    29,    30,    31,
      32,    33,    34,    15,   151,    17,    18,    19,    20,    21,
      22,    23,    24,    25,    26,    27,    28,    29,    30,    31,
      32,    33,    34,    15,   151,    17,    18,    19,    20,    21,
      22,    23,    24,    25,    26,    27,    28,    29,    30,    31,
      32,    33,    34,    15,   151,    17,    18,    19,    20,    21,
      22,    23,    24,    25,    26,    27,    28,    29,    30,    31,
      32,    33,    34,    -1,    -1,    -1,    -1,    -1,    -1,    -1,
      -1,    -1,    -1,    -1,    -1,    -1,    -1,    -1,    -1,   151,
      -1,    -1,    -1,     4,    -1,    -1,    -1,    -1,    -1,    -1,
      -1,    -1,    -1,    -1,    -1,    -1,    15,   149,    17,    18,
      19,    20,    21,    22,    23,    24,    25,    26,    27,    28,
      29,    30,    31,    32,    33,    34,    15,   149,    17,    18,
      19,    20,    21,    22,    23,    24,    25,    26,    27,    28,
      29,    30,    31,    32,    33,    34,   668,   149,    59,     4,
      -1,    -1,    -1,    -1,    -1,    -1,    67,    -1,    -1,    -1,
     682,   683,    -1,    -1,    75,    -1,    -1,   149,    -1,    80,
      -1,    -1,    -1,    -1,    -1,    -1,   698,    -1,    -1,   701,
      -1,    36,    -1,    38,    -1,    -1,    -1,   149,    -1,   100,
      -1,    -1,    -1,    -1,    -1,   106,   107,   108,    -1,    -1,
     722,   723,   724,   725,    -1,   727,    -1,   729,   119,   731,
      -1,    15,    67,    17,    18,    19,    20,    21,    22,    23,
      24,    25,    26,    27,    28,    29,    30,    31,    32,    33,
      34,    -1,    -1,    37,    -1,    -1,    -1,    -1,    -1,    -1,
     149,    -1,    -1,    -1,    -1,    -1,    -1,    -1,   103,    -1,
     772,   773,    -1,   775,   776,   110,   111,   112,   113,   114,
     149,   116,   117,   118,    -1,    -1,   121,   122,    -1,   124,
     125,   126,    -1,    -1,    -1,    -1,   131,   132,   133,    15,
      16,    17,    18,    19,    20,    21,    22,    23,    24,    25,
      26,    27,    28,    29,    30,    31,    32,    33,    34,    15,
      16,    17,    18,    19,    20,    21,    22,    23,    24,    25,
      26,    27,    28,    29,    30,    31,    32,    33,    34,    15,
      -1,    17,    18,    19,    20,    21,    22,    23,    24,    25,
      26,    27,    28,    29,    30,    31,    32,    33,    34,    18,
      19,    20,    21,    22,    23,    24,    25,    26,    27,    28,
      29,    30,    31,    32,    33,    34
};

/* YYSTOS[STATE-NUM] -- The (internal number of the) accessing
   symbol of state STATE-NUM.  */
static const yytype_uint16 yystos[] =
{
       0,   128,   129,   130,   139,   148,   157,   173,   174,   161,
     162,   159,   160,   277,   278,   272,   273,     0,   175,   163,
       4,    58,   134,   281,   282,    58,   274,   275,     4,    46,
      47,    48,    61,    62,    63,    64,    65,    67,    68,    69,
      70,    77,    78,    79,    80,    88,    89,    90,    91,    92,
      93,    94,    99,   100,   101,   106,   107,   108,   119,   132,
     150,   176,   185,   187,   210,   212,   223,   224,   226,   228,
     264,   279,   280,     4,    38,    67,   103,   110,   111,   112,
     113,   114,   116,   117,   118,   121,   122,   124,   125,   126,
     131,   132,   133,   164,     6,     4,   132,   135,   136,   137,
     284,   285,    58,   282,   136,   137,   276,   285,   275,     6,
       7,     8,     9,    10,    11,    12,    13,    14,   208,    58,
      58,    49,    50,    37,    37,     4,   158,    58,    37,    37,
      37,    37,    37,    37,    37,    37,    37,    37,    37,    37,
     177,    37,    37,    37,    37,   188,    37,   149,   150,   209,
     138,     4,   158,     4,     3,     4,    30,    31,    37,    38,
      39,    51,    52,    53,    60,    66,    76,    81,    82,    83,
      84,    85,    86,    87,    95,    97,   113,   119,   120,   147,
     154,   155,   232,     4,     4,   168,     4,   167,   166,     4,
       4,     4,   232,     4,     3,     4,   169,   170,   171,     4,
     115,   232,     4,    16,    16,    59,   150,   284,    59,   150,
     230,   231,   230,   186,   265,     4,     4,     4,     4,   178,
       4,    67,   213,   214,   215,     4,     4,     4,   158,   158,
       4,   158,   151,   158,   225,   227,     4,   229,   229,   179,
     180,    37,   158,     4,     4,     4,    37,   170,    58,     6,
     149,   165,     6,   149,   232,   232,   232,    37,    37,    37,
      37,    37,    37,    37,    37,    37,    37,    37,    37,    37,
      37,    37,    37,    37,    37,    37,    37,   232,   232,    15,
      17,    18,    19,    20,    21,    22,    23,    24,    25,    26,
      27,    28,    29,    30,    31,    32,    33,    34,   149,     6,
     149,   232,   149,   149,     4,   149,     6,   149,   232,   149,
     149,     4,   172,    58,   285,   285,   150,     4,   132,   135,
     136,   137,    59,   150,   232,     4,    59,    67,    75,   100,
     187,   239,     4,    59,   266,   149,   151,   151,   175,   216,
     158,    59,   149,   211,   149,   151,   151,   151,   151,   151,
     151,   151,   211,   151,   211,   149,   229,   151,   151,   151,
       4,     5,   109,   181,   179,   151,     6,     6,     6,   232,
     151,   281,   232,   232,   163,   232,   232,   151,   232,   232,
     232,   232,   232,     4,     4,   232,     4,     4,     4,     4,
     232,   232,     4,     4,   232,   232,   232,     4,   232,   232,
     232,   232,   232,   232,   232,   232,   232,   232,   232,   232,
     232,   232,   232,   232,   232,   232,   232,   232,   232,   232,
       4,     4,     4,   232,   232,     3,     4,     4,     4,   149,
     287,   150,   150,     4,   134,   150,   283,   240,   158,   245,
     250,   267,     4,    36,    37,   220,   217,   215,     4,   158,
     158,   229,    37,     4,     5,   109,   149,   151,   230,   230,
     230,   149,    59,    36,   149,   151,   151,   149,   149,   151,
     149,   151,   151,   151,   151,   151,   151,   149,   149,   151,
     151,   151,   149,   151,   151,    16,     4,   285,   137,    58,
     134,   150,    37,    40,   232,   254,   255,   252,    16,   232,
     256,   255,   232,   269,   149,     4,   154,   221,   222,    16,
     213,   151,   182,    37,     4,     5,   109,   151,   151,   151,
       4,   232,   232,   232,   232,   232,   232,     4,   232,   150,
     288,    16,   286,    71,    72,    73,    74,    75,   151,   253,
      37,    37,   254,    16,   104,   234,   186,    16,    93,   257,
     251,     4,   104,   270,     4,     4,   151,   222,    95,   218,
      36,   181,   184,    37,   151,   151,   151,   151,   151,   151,
     151,   151,    59,   285,   285,   151,   232,    16,    37,    38,
     235,    36,   234,    58,    37,   271,    37,   268,   151,     6,
     211,   151,   181,   183,   150,   288,   151,   232,    37,   145,
     236,   105,   237,   186,   232,   270,   232,   150,   230,    97,
     219,   151,   181,    59,    39,   254,   151,   232,   237,    37,
     246,    59,   151,   151,     6,   151,    37,    16,   151,   241,
     232,    58,   270,   230,   232,   141,   142,   143,   238,   151,
     247,   151,    58,   260,   254,   242,     4,    59,    16,     4,
      15,    32,    41,    42,    43,    44,    45,    54,    56,    67,
      96,    98,   102,   119,   140,   144,   146,   150,   152,   189,
     190,   191,   194,   197,   198,   200,   203,   204,   205,   210,
     261,   248,    37,    37,   158,    37,   201,    37,    37,    37,
       4,    54,    55,    56,    57,   190,   192,   196,    37,     4,
      54,   152,   191,   200,    59,    37,   209,    58,    25,   258,
     102,   190,   190,   202,   206,   230,    37,   199,     4,   193,
     189,   195,    37,    37,    37,    37,   153,   211,   196,    37,
     196,    37,   243,   230,   204,     4,   104,   233,   151,   151,
     151,   204,   151,   232,     4,   197,    21,   151,   151,   189,
      54,    55,   190,    54,    55,   190,   190,   190,   192,   151,
     153,   196,   258,   151,    59,    25,   259,    36,   149,   151,
       4,   189,    37,    37,   151,    37,    37,   151,   151,   151,
     151,   233,   262,     4,     6,    16,   207,     4,   190,   190,
     190,   190,   259,   259,   206,     4,   249,   151,   151,   151,
     151,   151,   207,   207,   211,   209,   151,   151,   151,   151,
     244,   263,   211,   211
};

#define yyerrok		(yyerrstatus = 0)
#define yyclearin	(yychar = YYEMPTY)
#define YYEMPTY		(-2)
#define YYEOF		0

#define YYACCEPT	goto yyacceptlab
#define YYABORT		goto yyabortlab
#define YYERROR		goto yyerrorlab


/* Like YYERROR except do call yyerror.  This remains here temporarily
   to ease the transition to the new meaning of YYERROR, for GCC.
   Once GCC version 2 has supplanted version 1, this can go.  */

#define YYFAIL		goto yyerrlab

#define YYRECOVERING()  (!!yyerrstatus)

#define YYBACKUP(Token, Value)					\
do								\
  if (yychar == YYEMPTY && yylen == 1)				\
    {								\
      yychar = (Token);						\
      yylval = (Value);						\
      yytoken = YYTRANSLATE (yychar);				\
      YYPOPSTACK (1);						\
      goto yybackup;						\
    }								\
  else								\
    {								\
      yyerror (YY_("syntax error: cannot back up")); \
      YYERROR;							\
    }								\
while (YYID (0))


#define YYTERROR	1
#define YYERRCODE	256


/* YYLLOC_DEFAULT -- Set CURRENT to span from RHS[1] to RHS[N].
   If N is 0, then set CURRENT to the empty location which ends
   the previous symbol: RHS[0] (always defined).  */

#define YYRHSLOC(Rhs, K) ((Rhs)[K])
#ifndef YYLLOC_DEFAULT
# define YYLLOC_DEFAULT(Current, Rhs, N)				\
    do									\
      if (YYID (N))                                                    \
	{								\
	  (Current).first_line   = YYRHSLOC (Rhs, 1).first_line;	\
	  (Current).first_column = YYRHSLOC (Rhs, 1).first_column;	\
	  (Current).last_line    = YYRHSLOC (Rhs, N).last_line;		\
	  (Current).last_column  = YYRHSLOC (Rhs, N).last_column;	\
	}								\
      else								\
	{								\
	  (Current).first_line   = (Current).last_line   =		\
	    YYRHSLOC (Rhs, 0).last_line;				\
	  (Current).first_column = (Current).last_column =		\
	    YYRHSLOC (Rhs, 0).last_column;				\
	}								\
    while (YYID (0))
#endif


/* YY_LOCATION_PRINT -- Print the location on the stream.
   This macro was not mandated originally: define only if we know
   we won't break user code: when these are the locations we know.  */

#ifndef YY_LOCATION_PRINT
# if YYLTYPE_IS_TRIVIAL
#  define YY_LOCATION_PRINT(File, Loc)			\
     fprintf (File, "%d.%d-%d.%d",			\
	      (Loc).first_line, (Loc).first_column,	\
	      (Loc).last_line,  (Loc).last_column)
# else
#  define YY_LOCATION_PRINT(File, Loc) ((void) 0)
# endif
#endif


/* YYLEX -- calling `yylex' with the right arguments.  */

#ifdef YYLEX_PARAM
# define YYLEX yylex (YYLEX_PARAM)
#else
# define YYLEX yylex ()
#endif

/* Enable debugging if requested.  */
#if YYDEBUG

# ifndef YYFPRINTF
#  include <stdio.h> /* INFRINGES ON USER NAME SPACE */
#  define YYFPRINTF fprintf
# endif

# define YYDPRINTF(Args)			\
do {						\
  if (yydebug)					\
    YYFPRINTF Args;				\
} while (YYID (0))

# define YY_SYMBOL_PRINT(Title, Type, Value, Location)			  \
do {									  \
  if (yydebug)								  \
    {									  \
      YYFPRINTF (stderr, "%s ", Title);					  \
      yy_symbol_print (stderr,						  \
		  Type, Value); \
      YYFPRINTF (stderr, "\n");						  \
    }									  \
} while (YYID (0))


/*--------------------------------.
| Print this symbol on YYOUTPUT.  |
`--------------------------------*/

/*ARGSUSED*/
#if (defined __STDC__ || defined __C99__FUNC__ \
     || defined __cplusplus || defined _MSC_VER)
static void
yy_symbol_value_print (FILE *yyoutput, int yytype, YYSTYPE const * const yyvaluep)
#else
static void
yy_symbol_value_print (yyoutput, yytype, yyvaluep)
    FILE *yyoutput;
    int yytype;
    YYSTYPE const * const yyvaluep;
#endif
{
  if (!yyvaluep)
    return;
# ifdef YYPRINT
  if (yytype < YYNTOKENS)
    YYPRINT (yyoutput, yytoknum[yytype], *yyvaluep);
# else
  YYUSE (yyoutput);
# endif
  switch (yytype)
    {
      default:
	break;
    }
}


/*--------------------------------.
| Print this symbol on YYOUTPUT.  |
`--------------------------------*/

#if (defined __STDC__ || defined __C99__FUNC__ \
     || defined __cplusplus || defined _MSC_VER)
static void
yy_symbol_print (FILE *yyoutput, int yytype, YYSTYPE const * const yyvaluep)
#else
static void
yy_symbol_print (yyoutput, yytype, yyvaluep)
    FILE *yyoutput;
    int yytype;
    YYSTYPE const * const yyvaluep;
#endif
{
  if (yytype < YYNTOKENS)
    YYFPRINTF (yyoutput, "token %s (", yytname[yytype]);
  else
    YYFPRINTF (yyoutput, "nterm %s (", yytname[yytype]);

  yy_symbol_value_print (yyoutput, yytype, yyvaluep);
  YYFPRINTF (yyoutput, ")");
}

/*------------------------------------------------------------------.
| yy_stack_print -- Print the state stack from its BOTTOM up to its |
| TOP (included).                                                   |
`------------------------------------------------------------------*/

#if (defined __STDC__ || defined __C99__FUNC__ \
     || defined __cplusplus || defined _MSC_VER)
static void
yy_stack_print (yytype_int16 *bottom, yytype_int16 *top)
#else
static void
yy_stack_print (bottom, top)
    yytype_int16 *bottom;
    yytype_int16 *top;
#endif
{
  YYFPRINTF (stderr, "Stack now");
  for (; bottom <= top; ++bottom)
    YYFPRINTF (stderr, " %d", *bottom);
  YYFPRINTF (stderr, "\n");
}

# define YY_STACK_PRINT(Bottom, Top)				\
do {								\
  if (yydebug)							\
    yy_stack_print ((Bottom), (Top));				\
} while (YYID (0))


/*------------------------------------------------.
| Report that the YYRULE is going to be reduced.  |
`------------------------------------------------*/

#if (defined __STDC__ || defined __C99__FUNC__ \
     || defined __cplusplus || defined _MSC_VER)
static void
yy_reduce_print (YYSTYPE *yyvsp, int yyrule)
#else
static void
yy_reduce_print (yyvsp, yyrule)
    YYSTYPE *yyvsp;
    int yyrule;
#endif
{
  int yynrhs = yyr2[yyrule];
  int yyi;
  unsigned long int yylno = yyrline[yyrule];
  YYFPRINTF (stderr, "Reducing stack by rule %d (line %lu):\n",
	     yyrule - 1, yylno);
  /* The symbols being reduced.  */
  for (yyi = 0; yyi < yynrhs; yyi++)
    {
      fprintf (stderr, "   $%d = ", yyi + 1);
      yy_symbol_print (stderr, yyrhs[yyprhs[yyrule] + yyi],
		       &(yyvsp[(yyi + 1) - (yynrhs)])
		       		       );
      fprintf (stderr, "\n");
    }
}

# define YY_REDUCE_PRINT(Rule)		\
do {					\
  if (yydebug)				\
    yy_reduce_print (yyvsp, Rule); \
} while (YYID (0))

/* Nonzero means print parse trace.  It is left uninitialized so that
   multiple parsers can coexist.  */
int yydebug;
#else /* !YYDEBUG */
# define YYDPRINTF(Args)
# define YY_SYMBOL_PRINT(Title, Type, Value, Location)
# define YY_STACK_PRINT(Bottom, Top)
# define YY_REDUCE_PRINT(Rule)
#endif /* !YYDEBUG */


/* YYINITDEPTH -- initial size of the parser's stacks.  */
#ifndef	YYINITDEPTH
# define YYINITDEPTH 200
#endif

/* YYMAXDEPTH -- maximum size the stacks can grow to (effective only
   if the built-in stack extension method is used).

   Do not make this value too large; the results are undefined if
   YYSTACK_ALLOC_MAXIMUM < YYSTACK_BYTES (YYMAXDEPTH)
   evaluated with infinite-precision integer arithmetic.  */

#ifndef YYMAXDEPTH
# define YYMAXDEPTH 10000
#endif



#if YYERROR_VERBOSE

# ifndef yystrlen
#  if defined __GLIBC__ && defined _STRING_H
#   define yystrlen strlen
#  else
/* Return the length of YYSTR.  */
#if (defined __STDC__ || defined __C99__FUNC__ \
     || defined __cplusplus || defined _MSC_VER)
static YYSIZE_T
yystrlen (const char *yystr)
#else
static YYSIZE_T
yystrlen (yystr)
    const char *yystr;
#endif
{
  YYSIZE_T yylen;
  for (yylen = 0; yystr[yylen]; yylen++)
    continue;
  return yylen;
}
#  endif
# endif

# ifndef yystpcpy
#  if defined __GLIBC__ && defined _STRING_H && defined _GNU_SOURCE
#   define yystpcpy stpcpy
#  else
/* Copy YYSRC to YYDEST, returning the address of the terminating '\0' in
   YYDEST.  */
#if (defined __STDC__ || defined __C99__FUNC__ \
     || defined __cplusplus || defined _MSC_VER)
static char *
yystpcpy (char *yydest, const char *yysrc)
#else
static char *
yystpcpy (yydest, yysrc)
    char *yydest;
    const char *yysrc;
#endif
{
  char *yyd = yydest;
  const char *yys = yysrc;

  while ((*yyd++ = *yys++) != '\0')
    continue;

  return yyd - 1;
}
#  endif
# endif

# ifndef yytnamerr
/* Copy to YYRES the contents of YYSTR after stripping away unnecessary
   quotes and backslashes, so that it's suitable for yyerror.  The
   heuristic is that double-quoting is unnecessary unless the string
   contains an apostrophe, a comma, or backslash (other than
   backslash-backslash).  YYSTR is taken from yytname.  If YYRES is
   null, do not copy; instead, return the length of what the result
   would have been.  */
static YYSIZE_T
yytnamerr (char *yyres, const char *yystr)
{
  if (*yystr == '"')
    {
      YYSIZE_T yyn = 0;
      char const *yyp = yystr;

      for (;;)
	switch (*++yyp)
	  {
	  case '\'':
	  case ',':
	    goto do_not_strip_quotes;

	  case '\\':
	    if (*++yyp != '\\')
	      goto do_not_strip_quotes;
	    /* Fall through.  */
	  default:
	    if (yyres)
	      yyres[yyn] = *yyp;
	    yyn++;
	    break;

	  case '"':
	    if (yyres)
	      yyres[yyn] = '\0';
	    return yyn;
	  }
    do_not_strip_quotes: ;
    }

  if (! yyres)
    return yystrlen (yystr);

  return yystpcpy (yyres, yystr) - yyres;
}
# endif

/* Copy into YYRESULT an error message about the unexpected token
   YYCHAR while in state YYSTATE.  Return the number of bytes copied,
   including the terminating null byte.  If YYRESULT is null, do not
   copy anything; just return the number of bytes that would be
   copied.  As a special case, return 0 if an ordinary "syntax error"
   message will do.  Return YYSIZE_MAXIMUM if overflow occurs during
   size calculation.  */
static YYSIZE_T
yysyntax_error (char *yyresult, int yystate, int yychar)
{
  int yyn = yypact[yystate];

  if (! (YYPACT_NINF < yyn && yyn <= YYLAST))
    return 0;
  else
    {
      int yytype = YYTRANSLATE (yychar);
      YYSIZE_T yysize0 = yytnamerr (0, yytname[yytype]);
      YYSIZE_T yysize = yysize0;
      YYSIZE_T yysize1;
      int yysize_overflow = 0;
      enum { YYERROR_VERBOSE_ARGS_MAXIMUM = 5 };
      char const *yyarg[YYERROR_VERBOSE_ARGS_MAXIMUM];
      int yyx;

# if 0
      /* This is so xgettext sees the translatable formats that are
	 constructed on the fly.  */
      YY_("syntax error, unexpected %s");
      YY_("syntax error, unexpected %s, expecting %s");
      YY_("syntax error, unexpected %s, expecting %s or %s");
      YY_("syntax error, unexpected %s, expecting %s or %s or %s");
      YY_("syntax error, unexpected %s, expecting %s or %s or %s or %s");
# endif
      char *yyfmt;
      char const *yyf;
      static char const yyunexpected[] = "syntax error, unexpected %s";
      static char const yyexpecting[] = ", expecting %s";
      static char const yyor[] = " or %s";
      char yyformat[sizeof yyunexpected
		    + sizeof yyexpecting - 1
		    + ((YYERROR_VERBOSE_ARGS_MAXIMUM - 2)
		       * (sizeof yyor - 1))];
      char const *yyprefix = yyexpecting;

      /* Start YYX at -YYN if negative to avoid negative indexes in
	 YYCHECK.  */
      int yyxbegin = yyn < 0 ? -yyn : 0;

      /* Stay within bounds of both yycheck and yytname.  */
      int yychecklim = YYLAST - yyn + 1;
      int yyxend = yychecklim < YYNTOKENS ? yychecklim : YYNTOKENS;
      int yycount = 1;

      yyarg[0] = yytname[yytype];
      yyfmt = yystpcpy (yyformat, yyunexpected);

      for (yyx = yyxbegin; yyx < yyxend; ++yyx)
	if (yycheck[yyx + yyn] == yyx && yyx != YYTERROR)
	  {
	    if (yycount == YYERROR_VERBOSE_ARGS_MAXIMUM)
	      {
		yycount = 1;
		yysize = yysize0;
		yyformat[sizeof yyunexpected - 1] = '\0';
		break;
	      }
	    yyarg[yycount++] = yytname[yyx];
	    yysize1 = yysize + yytnamerr (0, yytname[yyx]);
	    yysize_overflow |= (yysize1 < yysize);
	    yysize = yysize1;
	    yyfmt = yystpcpy (yyfmt, yyprefix);
	    yyprefix = yyor;
	  }

      yyf = YY_(yyformat);
      yysize1 = yysize + yystrlen (yyf);
      yysize_overflow |= (yysize1 < yysize);
      yysize = yysize1;

      if (yysize_overflow)
	return YYSIZE_MAXIMUM;

      if (yyresult)
	{
	  /* Avoid sprintf, as that infringes on the user's name space.
	     Don't have undefined behavior even if the translation
	     produced a string with the wrong number of "%s"s.  */
	  char *yyp = yyresult;
	  int yyi = 0;
	  while ((*yyp = *yyf) != '\0')
	    {
	      if (*yyp == '%' && yyf[1] == 's' && yyi < yycount)
		{
		  yyp += yytnamerr (yyp, yyarg[yyi++]);
		  yyf += 2;
		}
	      else
		{
		  yyp++;
		  yyf++;
		}
	    }
	}
      return yysize;
    }
}
#endif /* YYERROR_VERBOSE */


/*-----------------------------------------------.
| Release the memory associated to this symbol.  |
`-----------------------------------------------*/

/*ARGSUSED*/
#if (defined __STDC__ || defined __C99__FUNC__ \
     || defined __cplusplus || defined _MSC_VER)
static void
yydestruct (const char *yymsg, int yytype, YYSTYPE *yyvaluep)
#else
static void
yydestruct (yymsg, yytype, yyvaluep)
    const char *yymsg;
    int yytype;
    YYSTYPE *yyvaluep;
#endif
{
  YYUSE (yyvaluep);

  if (!yymsg)
    yymsg = "Deleting";
  YY_SYMBOL_PRINT (yymsg, yytype, yyvaluep, yylocationp);

  switch (yytype)
    {

      default:
	break;
    }
}


/* Prevent warnings from -Wmissing-prototypes.  */

#ifdef YYPARSE_PARAM
#if defined __STDC__ || defined __cplusplus
int yyparse (void *YYPARSE_PARAM);
#else
int yyparse ();
#endif
#else /* ! YYPARSE_PARAM */
#if defined __STDC__ || defined __cplusplus
int yyparse (void);
#else
int yyparse ();
#endif
#endif /* ! YYPARSE_PARAM */



/* The look-ahead symbol.  */
int yychar;

/* The semantic value of the look-ahead symbol.  */
YYSTYPE yylval;

/* Number of syntax errors so far.  */
int yynerrs;



/*----------.
| yyparse.  |
`----------*/

#ifdef YYPARSE_PARAM
#if (defined __STDC__ || defined __C99__FUNC__ \
     || defined __cplusplus || defined _MSC_VER)
int
yyparse (void *YYPARSE_PARAM)
#else
int
yyparse (YYPARSE_PARAM)
    void *YYPARSE_PARAM;
#endif
#else /* ! YYPARSE_PARAM */
#if (defined __STDC__ || defined __C99__FUNC__ \
     || defined __cplusplus || defined _MSC_VER)
int
yyparse (void)
#else
int
yyparse ()

#endif
#endif
{
  
  int yystate;
  int yyn;
  int yyresult;
  /* Number of tokens to shift before error messages enabled.  */
  int yyerrstatus;
  /* Look-ahead token as an internal (translated) token number.  */
  int yytoken = 0;
#if YYERROR_VERBOSE
  /* Buffer for error messages, and its allocated size.  */
  char yymsgbuf[128];
  char *yymsg = yymsgbuf;
  YYSIZE_T yymsg_alloc = sizeof yymsgbuf;
#endif

  /* Three stacks and their tools:
     `yyss': related to states,
     `yyvs': related to semantic values,
     `yyls': related to locations.

     Refer to the stacks thru separate pointers, to allow yyoverflow
     to reallocate them elsewhere.  */

  /* The state stack.  */
  yytype_int16 yyssa[YYINITDEPTH];
  yytype_int16 *yyss = yyssa;
  yytype_int16 *yyssp;

  /* The semantic value stack.  */
  YYSTYPE yyvsa[YYINITDEPTH];
  YYSTYPE *yyvs = yyvsa;
  YYSTYPE *yyvsp;



#define YYPOPSTACK(N)   (yyvsp -= (N), yyssp -= (N))

  YYSIZE_T yystacksize = YYINITDEPTH;

  /* The variables used to return semantic value and location from the
     action routines.  */
  YYSTYPE yyval;


  /* The number of symbols on the RHS of the reduced rule.
     Keep to zero when no symbol should be popped.  */
  int yylen = 0;

  YYDPRINTF ((stderr, "Starting parse\n"));

  yystate = 0;
  yyerrstatus = 0;
  yynerrs = 0;
  yychar = YYEMPTY;		/* Cause a token to be read.  */

  /* Initialize stack pointers.
     Waste one element of value and location stack
     so that they stay on the same level as the state stack.
     The wasted elements are never initialized.  */

  yyssp = yyss;
  yyvsp = yyvs;

  goto yysetstate;

/*------------------------------------------------------------.
| yynewstate -- Push a new state, which is found in yystate.  |
`------------------------------------------------------------*/
 yynewstate:
  /* In all cases, when you get here, the value and location stacks
     have just been pushed.  So pushing a state here evens the stacks.  */
  yyssp++;

 yysetstate:
  *yyssp = yystate;

  if (yyss + yystacksize - 1 <= yyssp)
    {
      /* Get the current used size of the three stacks, in elements.  */
      YYSIZE_T yysize = yyssp - yyss + 1;

#ifdef yyoverflow
      {
	/* Give user a chance to reallocate the stack.  Use copies of
	   these so that the &'s don't force the real ones into
	   memory.  */
	YYSTYPE *yyvs1 = yyvs;
	yytype_int16 *yyss1 = yyss;


	/* Each stack pointer address is followed by the size of the
	   data in use in that stack, in bytes.  This used to be a
	   conditional around just the two extra args, but that might
	   be undefined if yyoverflow is a macro.  */
	yyoverflow (YY_("memory exhausted"),
		    &yyss1, yysize * sizeof (*yyssp),
		    &yyvs1, yysize * sizeof (*yyvsp),

		    &yystacksize);

	yyss = yyss1;
	yyvs = yyvs1;
      }
#else /* no yyoverflow */
# ifndef YYSTACK_RELOCATE
      goto yyexhaustedlab;
# else
      /* Extend the stack our own way.  */
      if (YYMAXDEPTH <= yystacksize)
	goto yyexhaustedlab;
      yystacksize *= 2;
      if (YYMAXDEPTH < yystacksize)
	yystacksize = YYMAXDEPTH;

      {
	yytype_int16 *yyss1 = yyss;
	union yyalloc *yyptr =
	  (union yyalloc *) YYSTACK_ALLOC (YYSTACK_BYTES (yystacksize));
	if (! yyptr)
	  goto yyexhaustedlab;
	YYSTACK_RELOCATE (yyss);
	YYSTACK_RELOCATE (yyvs);

#  undef YYSTACK_RELOCATE
	if (yyss1 != yyssa)
	  YYSTACK_FREE (yyss1);
      }
# endif
#endif /* no yyoverflow */

      yyssp = yyss + yysize - 1;
      yyvsp = yyvs + yysize - 1;


      YYDPRINTF ((stderr, "Stack size increased to %lu\n",
		  (unsigned long int) yystacksize));

      if (yyss + yystacksize - 1 <= yyssp)
	YYABORT;
    }

  YYDPRINTF ((stderr, "Entering state %d\n", yystate));

  goto yybackup;

/*-----------.
| yybackup.  |
`-----------*/
yybackup:

  /* Do appropriate processing given the current state.  Read a
     look-ahead token if we need one and don't already have one.  */

  /* First try to decide what to do without reference to look-ahead token.  */
  yyn = yypact[yystate];
  if (yyn == YYPACT_NINF)
    goto yydefault;

  /* Not known => get a look-ahead token if don't already have one.  */

  /* YYCHAR is either YYEMPTY or YYEOF or a valid look-ahead symbol.  */
  if (yychar == YYEMPTY)
    {
      YYDPRINTF ((stderr, "Reading a token: "));
      yychar = YYLEX;
    }

  if (yychar <= YYEOF)
    {
      yychar = yytoken = YYEOF;
      YYDPRINTF ((stderr, "Now at end of input.\n"));
    }
  else
    {
      yytoken = YYTRANSLATE (yychar);
      YY_SYMBOL_PRINT ("Next token is", yytoken, &yylval, &yylloc);
    }

  /* If the proper action on seeing token YYTOKEN is to reduce or to
     detect an error, take that action.  */
  yyn += yytoken;
  if (yyn < 0 || YYLAST < yyn || yycheck[yyn] != yytoken)
    goto yydefault;
  yyn = yytable[yyn];
  if (yyn <= 0)
    {
      if (yyn == 0 || yyn == YYTABLE_NINF)
	goto yyerrlab;
      yyn = -yyn;
      goto yyreduce;
    }

  if (yyn == YYFINAL)
    YYACCEPT;

  /* Count tokens shifted since error; after three, turn off error
     status.  */
  if (yyerrstatus)
    yyerrstatus--;

  /* Shift the look-ahead token.  */
  YY_SYMBOL_PRINT ("Shifting", yytoken, &yylval, &yylloc);

  /* Discard the shifted token unless it is eof.  */
  if (yychar != YYEOF)
    yychar = YYEMPTY;

  yystate = yyn;
  *++yyvsp = yylval;

  goto yynewstate;


/*-----------------------------------------------------------.
| yydefault -- do the default action for the current state.  |
`-----------------------------------------------------------*/
yydefault:
  yyn = yydefact[yystate];
  if (yyn == 0)
    goto yyerrlab;
  goto yyreduce;


/*-----------------------------.
| yyreduce -- Do a reduction.  |
`-----------------------------*/
yyreduce:
  /* yyn is the number of a rule to reduce with.  */
  yylen = yyr2[yyn];

  /* If YYLEN is nonzero, implement the default value of the action:
     `$$ = $1'.

     Otherwise, the following line sets YYVAL to garbage.
     This behavior is undocumented and Bison
     users should not rely upon it.  Assigning to YYVAL
     unconditionally makes the parser a bit smaller, and it avoids a
     GCC warning that YYVAL may be used uninitialized.  */
  yyval = yyvsp[1-yylen];


  YY_REDUCE_PRINT (yyn);
  switch (yyn)
    {
        case 8:
#line 178 "ldgram.y"
    { ldlex_defsym(); }
    break;

  case 9:
#line 180 "ldgram.y"
    {
		  ldlex_popstate();
		  lang_add_assignment (exp_defsym ((yyvsp[(2) - (4)].name), (yyvsp[(4) - (4)].etree)));
		}
    break;

  case 10:
#line 188 "ldgram.y"
    {
		  ldlex_mri_script ();
		  PUSH_ERROR (_("MRI style script"));
		}
    break;

  case 11:
#line 193 "ldgram.y"
    {
		  ldlex_popstate ();
		  mri_draw_tree ();
		  POP_ERROR ();
		}
    break;

  case 16:
#line 208 "ldgram.y"
    {
			einfo(_("%P%F: unrecognised keyword in MRI style script '%s'\n"),(yyvsp[(1) - (1)].name));
			}
    break;

  case 17:
#line 211 "ldgram.y"
    {
			config.map_filename = "-";
			}
    break;

  case 20:
#line 217 "ldgram.y"
    { mri_public((yyvsp[(2) - (4)].name), (yyvsp[(4) - (4)].etree)); }
    break;

  case 21:
#line 219 "ldgram.y"
    { mri_public((yyvsp[(2) - (4)].name), (yyvsp[(4) - (4)].etree)); }
    break;

  case 22:
#line 221 "ldgram.y"
    { mri_public((yyvsp[(2) - (3)].name), (yyvsp[(3) - (3)].etree)); }
    break;

  case 23:
#line 223 "ldgram.y"
    { mri_format((yyvsp[(2) - (2)].name)); }
    break;

  case 24:
#line 225 "ldgram.y"
    { mri_output_section((yyvsp[(2) - (4)].name), (yyvsp[(4) - (4)].etree));}
    break;

  case 25:
#line 227 "ldgram.y"
    { mri_output_section((yyvsp[(2) - (3)].name), (yyvsp[(3) - (3)].etree));}
    break;

  case 26:
#line 229 "ldgram.y"
    { mri_output_section((yyvsp[(2) - (4)].name), (yyvsp[(4) - (4)].etree));}
    break;

  case 27:
#line 231 "ldgram.y"
    { mri_align((yyvsp[(2) - (4)].name),(yyvsp[(4) - (4)].etree)); }
    break;

  case 28:
#line 233 "ldgram.y"
    { mri_align((yyvsp[(2) - (4)].name),(yyvsp[(4) - (4)].etree)); }
    break;

  case 29:
#line 235 "ldgram.y"
    { mri_alignmod((yyvsp[(2) - (4)].name),(yyvsp[(4) - (4)].etree)); }
    break;

  case 30:
#line 237 "ldgram.y"
    { mri_alignmod((yyvsp[(2) - (4)].name),(yyvsp[(4) - (4)].etree)); }
    break;

  case 33:
#line 241 "ldgram.y"
    { mri_name((yyvsp[(2) - (2)].name)); }
    break;

  case 34:
#line 243 "ldgram.y"
    { mri_alias((yyvsp[(2) - (4)].name),(yyvsp[(4) - (4)].name),0);}
    break;

  case 35:
#line 245 "ldgram.y"
    { mri_alias ((yyvsp[(2) - (4)].name), 0, (int) (yyvsp[(4) - (4)].bigint).integer); }
    break;

  case 36:
#line 247 "ldgram.y"
    { mri_base((yyvsp[(2) - (2)].etree)); }
    break;

  case 37:
#line 249 "ldgram.y"
    { mri_truncate ((unsigned int) (yyvsp[(2) - (2)].bigint).integer); }
    break;

  case 40:
#line 253 "ldgram.y"
    { ldlex_script (); ldfile_open_command_file((yyvsp[(2) - (2)].name)); }
    break;

  case 41:
#line 255 "ldgram.y"
    { ldlex_popstate (); }
    break;

  case 42:
#line 257 "ldgram.y"
    { lang_add_entry ((yyvsp[(2) - (2)].name), FALSE); }
    break;

  case 44:
#line 262 "ldgram.y"
    { mri_order((yyvsp[(3) - (3)].name)); }
    break;

  case 45:
#line 263 "ldgram.y"
    { mri_order((yyvsp[(2) - (2)].name)); }
    break;

  case 47:
#line 269 "ldgram.y"
    { mri_load((yyvsp[(1) - (1)].name)); }
    break;

  case 48:
#line 270 "ldgram.y"
    { mri_load((yyvsp[(3) - (3)].name)); }
    break;

  case 49:
#line 275 "ldgram.y"
    { mri_only_load((yyvsp[(1) - (1)].name)); }
    break;

  case 50:
#line 277 "ldgram.y"
    { mri_only_load((yyvsp[(3) - (3)].name)); }
    break;

  case 51:
#line 281 "ldgram.y"
    { (yyval.name) = NULL; }
    break;

  case 54:
#line 288 "ldgram.y"
    { ldlex_expression (); }
    break;

  case 55:
#line 290 "ldgram.y"
    { ldlex_popstate (); }
    break;

  case 56:
#line 294 "ldgram.y"
    { ldlang_add_undef ((yyvsp[(1) - (1)].name), FALSE); }
    break;

  case 57:
#line 296 "ldgram.y"
    { ldlang_add_undef ((yyvsp[(2) - (2)].name), FALSE); }
    break;

  case 58:
#line 298 "ldgram.y"
    { ldlang_add_undef ((yyvsp[(3) - (3)].name), FALSE); }
    break;

  case 59:
#line 302 "ldgram.y"
    { ldlex_both(); }
    break;

  case 60:
#line 304 "ldgram.y"
    { ldlex_popstate(); }
    break;

  case 73:
#line 325 "ldgram.y"
    { lang_add_target((yyvsp[(3) - (4)].name)); }
    break;

  case 74:
#line 327 "ldgram.y"
    { ldfile_add_library_path ((yyvsp[(3) - (4)].name), FALSE); }
    break;

  case 75:
#line 329 "ldgram.y"
    { lang_add_output((yyvsp[(3) - (4)].name), 1); }
    break;

  case 76:
#line 331 "ldgram.y"
    { lang_add_output_format ((yyvsp[(3) - (4)].name), (char *) NULL,
					    (char *) NULL, 1); }
    break;

  case 77:
#line 334 "ldgram.y"
    { lang_add_output_format ((yyvsp[(3) - (8)].name), (yyvsp[(5) - (8)].name), (yyvsp[(7) - (8)].name), 1); }
    break;

  case 78:
#line 336 "ldgram.y"
    { ldfile_set_output_arch ((yyvsp[(3) - (4)].name), bfd_arch_unknown); }
    break;

  case 79:
#line 338 "ldgram.y"
    { command_line.force_common_definition = TRUE ; }
    break;

  case 80:
#line 340 "ldgram.y"
    { command_line.force_group_allocation = TRUE ; }
    break;

  case 81:
#line 342 "ldgram.y"
    { link_info.inhibit_common_definition = TRUE ; }
    break;

  case 83:
#line 345 "ldgram.y"
    { lang_enter_group (); }
    break;

  case 84:
#line 347 "ldgram.y"
    { lang_leave_group (); }
    break;

  case 85:
#line 349 "ldgram.y"
    { lang_add_map((yyvsp[(3) - (4)].name)); }
    break;

  case 86:
#line 351 "ldgram.y"
    { ldlex_script (); ldfile_open_command_file((yyvsp[(2) - (2)].name)); }
    break;

  case 87:
#line 353 "ldgram.y"
    { ldlex_popstate (); }
    break;

  case 88:
#line 355 "ldgram.y"
    {
		  lang_add_nocrossref ((yyvsp[(3) - (4)].nocrossref));
		}
    break;

  case 89:
#line 359 "ldgram.y"
    {
		  lang_add_nocrossref_to ((yyvsp[(3) - (4)].nocrossref));
		}
    break;

  case 91:
#line 364 "ldgram.y"
    { lang_add_insert ((yyvsp[(3) - (3)].name), 0); }
    break;

  case 92:
#line 366 "ldgram.y"
    { lang_add_insert ((yyvsp[(3) - (3)].name), 1); }
    break;

  case 93:
#line 368 "ldgram.y"
    { lang_memory_region_alias ((yyvsp[(3) - (6)].name), (yyvsp[(5) - (6)].name)); }
    break;

  case 94:
#line 370 "ldgram.y"
    { lang_ld_feature ((yyvsp[(3) - (4)].name)); }
    break;

  case 95:
#line 374 "ldgram.y"
    { ldlex_inputlist(); }
    break;

  case 96:
#line 376 "ldgram.y"
    { ldlex_popstate(); }
    break;

  case 97:
#line 380 "ldgram.y"
    { lang_add_input_file((yyvsp[(1) - (1)].name),lang_input_file_is_search_file_enum,
				 (char *)NULL); }
    break;

  case 98:
#line 383 "ldgram.y"
    { lang_add_input_file((yyvsp[(3) - (3)].name),lang_input_file_is_search_file_enum,
				 (char *)NULL); }
    break;

  case 99:
#line 386 "ldgram.y"
    { lang_add_input_file((yyvsp[(2) - (2)].name),lang_input_file_is_search_file_enum,
				 (char *)NULL); }
    break;

  case 100:
#line 389 "ldgram.y"
    { lang_add_input_file((yyvsp[(1) - (1)].name),lang_input_file_is_l_enum,
				 (char *)NULL); }
    break;

  case 101:
#line 392 "ldgram.y"
    { lang_add_input_file((yyvsp[(3) - (3)].name),lang_input_file_is_l_enum,
				 (char *)NULL); }
    break;

  case 102:
#line 395 "ldgram.y"
    { lang_add_input_file((yyvsp[(2) - (2)].name),lang_input_file_is_l_enum,
				 (char *)NULL); }
    break;

  case 103:
#line 398 "ldgram.y"
    { (yyval.integer) = input_flags.add_DT_NEEDED_for_regular;
		    input_flags.add_DT_NEEDED_for_regular = TRUE; }
    break;

  case 104:
#line 401 "ldgram.y"
    { input_flags.add_DT_NEEDED_for_regular = (yyvsp[(3) - (5)].integer); }
    break;

  case 105:
#line 403 "ldgram.y"
    { (yyval.integer) = input_flags.add_DT_NEEDED_for_regular;
		    input_flags.add_DT_NEEDED_for_regular = TRUE; }
    break;

  case 106:
#line 406 "ldgram.y"
    { input_flags.add_DT_NEEDED_for_regular = (yyvsp[(5) - (7)].integer); }
    break;

  case 107:
#line 408 "ldgram.y"
    { (yyval.integer) = input_flags.add_DT_NEEDED_for_regular;
		    input_flags.add_DT_NEEDED_for_regular = TRUE; }
    break;

  case 108:
#line 411 "ldgram.y"
    { input_flags.add_DT_NEEDED_for_regular = (yyvsp[(4) - (6)].integer); }
    break;

  case 113:
#line 426 "ldgram.y"
    { lang_add_entry ((yyvsp[(3) - (4)].name), FALSE); }
    break;

  case 115:
#line 428 "ldgram.y"
    {ldlex_expression ();}
    break;

  case 116:
#line 429 "ldgram.y"
    { ldlex_popstate ();
		  lang_add_assignment (exp_assert ((yyvsp[(4) - (7)].etree), (yyvsp[(6) - (7)].name))); }
    break;

  case 117:
#line 437 "ldgram.y"
    {
			  (yyval.cname) = (yyvsp[(1) - (1)].name);
			}
    break;

  case 118:
#line 441 "ldgram.y"
    {
			  (yyval.cname) = "*";
			}
    break;

  case 119:
#line 445 "ldgram.y"
    {
			  (yyval.cname) = "?";
			}
    break;

  case 120:
#line 452 "ldgram.y"
    {
			  (yyval.wildcard).name = (yyvsp[(1) - (1)].cname);
			  (yyval.wildcard).sorted = none;
			  (yyval.wildcard).exclude_name_list = NULL;
			  (yyval.wildcard).section_flag_list = NULL;
			}
    break;

  case 121:
#line 459 "ldgram.y"
    {
			  (yyval.wildcard).name = (yyvsp[(5) - (5)].cname);
			  (yyval.wildcard).sorted = none;
			  (yyval.wildcard).exclude_name_list = (yyvsp[(3) - (5)].name_list);
			  (yyval.wildcard).section_flag_list = NULL;
			}
    break;

  case 123:
#line 470 "ldgram.y"
    {
			  (yyval.wildcard) = (yyvsp[(3) - (4)].wildcard);
			  (yyval.wildcard).sorted = by_name;
			}
    break;

  case 124:
#line 475 "ldgram.y"
    {
			  (yyval.wildcard) = (yyvsp[(3) - (4)].wildcard);
			  (yyval.wildcard).sorted = by_none;
			}
    break;

  case 126:
#line 484 "ldgram.y"
    {
			  (yyval.wildcard) = (yyvsp[(3) - (4)].wildcard);
			  (yyval.wildcard).sorted = by_name;
			}
    break;

  case 127:
#line 489 "ldgram.y"
    {
			  (yyval.wildcard) = (yyvsp[(3) - (4)].wildcard);
			  (yyval.wildcard).sorted = by_alignment;
			}
    break;

  case 128:
#line 494 "ldgram.y"
    {
			  (yyval.wildcard) = (yyvsp[(3) - (4)].wildcard);
			  (yyval.wildcard).sorted = by_none;
			}
    break;

  case 129:
#line 499 "ldgram.y"
    {
			  (yyval.wildcard) = (yyvsp[(5) - (7)].wildcard);
			  (yyval.wildcard).sorted = by_name_alignment;
			}
    break;

  case 130:
#line 504 "ldgram.y"
    {
			  (yyval.wildcard) = (yyvsp[(5) - (7)].wildcard);
			  (yyval.wildcard).sorted = by_name;
			}
    break;

  case 131:
#line 509 "ldgram.y"
    {
			  (yyval.wildcard) = (yyvsp[(5) - (7)].wildcard);
			  (yyval.wildcard).sorted = by_alignment_name;
			}
    break;

  case 132:
#line 514 "ldgram.y"
    {
			  (yyval.wildcard) = (yyvsp[(5) - (7)].wildcard);
			  (yyval.wildcard).sorted = by_alignment;
			}
    break;

  case 133:
#line 519 "ldgram.y"
    {
			  (yyval.wildcard) = (yyvsp[(3) - (4)].wildcard);
			  (yyval.wildcard).sorted = by_init_priority;
			}
    break;

  case 134:
#line 526 "ldgram.y"
    {
			  struct flag_info_list *n;
			  n = ((struct flag_info_list *) xmalloc (sizeof *n));
			  if ((yyvsp[(1) - (1)].name)[0] == '!')
			    {
			      n->with = without_flags;
			      n->name = &(yyvsp[(1) - (1)].name)[1];
			    }
			  else
			    {
			      n->with = with_flags;
			      n->name = (yyvsp[(1) - (1)].name);
			    }
			  n->valid = FALSE;
			  n->next = NULL;
			  (yyval.flag_info_list) = n;
			}
    break;

  case 135:
#line 544 "ldgram.y"
    {
			  struct flag_info_list *n;
			  n = ((struct flag_info_list *) xmalloc (sizeof *n));
			  if ((yyvsp[(3) - (3)].name)[0] == '!')
			    {
			      n->with = without_flags;
			      n->name = &(yyvsp[(3) - (3)].name)[1];
			    }
			  else
			    {
			      n->with = with_flags;
			      n->name = (yyvsp[(3) - (3)].name);
			    }
			  n->valid = FALSE;
			  n->next = (yyvsp[(1) - (3)].flag_info_list);
			  (yyval.flag_info_list) = n;
			}
    break;

  case 136:
#line 565 "ldgram.y"
    {
			  struct flag_info *n;
			  n = ((struct flag_info *) xmalloc (sizeof *n));
			  n->flag_list = (yyvsp[(3) - (4)].flag_info_list);
			  n->flags_initialized = FALSE;
			  n->not_with_flags = 0;
			  n->only_with_flags = 0;
			  (yyval.flag_info) = n;
			}
    break;

  case 137:
#line 578 "ldgram.y"
    {
			  struct name_list *tmp;
			  tmp = (struct name_list *) xmalloc (sizeof *tmp);
			  tmp->name = (yyvsp[(2) - (2)].cname);
			  tmp->next = (yyvsp[(1) - (2)].name_list);
			  (yyval.name_list) = tmp;
			}
    break;

  case 138:
#line 587 "ldgram.y"
    {
			  struct name_list *tmp;
			  tmp = (struct name_list *) xmalloc (sizeof *tmp);
			  tmp->name = (yyvsp[(1) - (1)].cname);
			  tmp->next = NULL;
			  (yyval.name_list) = tmp;
			}
    break;

  case 139:
#line 598 "ldgram.y"
    {
			  struct wildcard_list *tmp;
			  tmp = (struct wildcard_list *) xmalloc (sizeof *tmp);
			  tmp->next = (yyvsp[(1) - (3)].wildcard_list);
			  tmp->spec = (yyvsp[(3) - (3)].wildcard);
			  (yyval.wildcard_list) = tmp;
			}
    break;

  case 140:
#line 607 "ldgram.y"
    {
			  struct wildcard_list *tmp;
			  tmp = (struct wildcard_list *) xmalloc (sizeof *tmp);
			  tmp->next = NULL;
			  tmp->spec = (yyvsp[(1) - (1)].wildcard);
			  (yyval.wildcard_list) = tmp;
			}
    break;

  case 141:
#line 618 "ldgram.y"
    {
			  struct wildcard_spec tmp;
			  tmp.name = (yyvsp[(1) - (1)].name);
			  tmp.exclude_name_list = NULL;
			  tmp.sorted = none;
			  tmp.section_flag_list = NULL;
			  lang_add_wild (&tmp, NULL, ldgram_had_keep);
			}
    break;

  case 142:
#line 627 "ldgram.y"
    {
			  struct wildcard_spec tmp;
			  tmp.name = (yyvsp[(2) - (2)].name);
			  tmp.exclude_name_list = NULL;
			  tmp.sorted = none;
			  tmp.section_flag_list = (yyvsp[(1) - (2)].flag_info);
			  lang_add_wild (&tmp, NULL, ldgram_had_keep);
			}
    break;

  case 143:
#line 636 "ldgram.y"
    {
			  lang_add_wild (NULL, (yyvsp[(2) - (3)].wildcard_list), ldgram_had_keep);
			}
    break;

  case 144:
#line 640 "ldgram.y"
    {
			  struct wildcard_spec tmp;
			  tmp.name = NULL;
			  tmp.exclude_name_list = NULL;
			  tmp.sorted = none;
			  tmp.section_flag_list = (yyvsp[(1) - (4)].flag_info);
			  lang_add_wild (&tmp, (yyvsp[(3) - (4)].wildcard_list), ldgram_had_keep);
			}
    break;

  case 145:
#line 649 "ldgram.y"
    {
			  lang_add_wild (&(yyvsp[(1) - (4)].wildcard), (yyvsp[(3) - (4)].wildcard_list), ldgram_had_keep);
			}
    break;

  case 146:
#line 653 "ldgram.y"
    {
			  (yyvsp[(2) - (5)].wildcard).section_flag_list = (yyvsp[(1) - (5)].flag_info);
			  lang_add_wild (&(yyvsp[(2) - (5)].wildcard), (yyvsp[(4) - (5)].wildcard_list), ldgram_had_keep);
			}
    break;

  case 148:
#line 662 "ldgram.y"
    { ldgram_had_keep = TRUE; }
    break;

  case 149:
#line 664 "ldgram.y"
    { ldgram_had_keep = FALSE; }
    break;

  case 151:
#line 670 "ldgram.y"
    {
		lang_add_attribute(lang_object_symbols_statement_enum);
		}
    break;

  case 153:
#line 675 "ldgram.y"
    {

		  lang_add_attribute(lang_constructors_statement_enum);
		}
    break;

  case 154:
#line 680 "ldgram.y"
    {
		  constructors_sorted = TRUE;
		  lang_add_attribute (lang_constructors_statement_enum);
		}
    break;

  case 156:
#line 686 "ldgram.y"
    {
		  lang_add_data ((int) (yyvsp[(1) - (4)].integer), (yyvsp[(3) - (4)].etree));
		}
    break;

  case 157:
#line 691 "ldgram.y"
    {
		  lang_add_fill ((yyvsp[(3) - (4)].fill));
		}
    break;

  case 158:
#line 694 "ldgram.y"
    {ldlex_expression ();}
    break;

  case 159:
#line 695 "ldgram.y"
    { ldlex_popstate ();
			  lang_add_assignment (exp_assert ((yyvsp[(4) - (8)].etree), (yyvsp[(6) - (8)].name))); }
    break;

  case 160:
#line 698 "ldgram.y"
    { ldlex_script (); ldfile_open_command_file((yyvsp[(2) - (2)].name)); }
    break;

  case 161:
#line 700 "ldgram.y"
    { ldlex_popstate (); }
    break;

  case 166:
#line 715 "ldgram.y"
    { (yyval.integer) = (yyvsp[(1) - (1)].token); }
    break;

  case 167:
#line 717 "ldgram.y"
    { (yyval.integer) = (yyvsp[(1) - (1)].token); }
    break;

  case 168:
#line 719 "ldgram.y"
    { (yyval.integer) = (yyvsp[(1) - (1)].token); }
    break;

  case 169:
#line 721 "ldgram.y"
    { (yyval.integer) = (yyvsp[(1) - (1)].token); }
    break;

  case 170:
#line 723 "ldgram.y"
    { (yyval.integer) = (yyvsp[(1) - (1)].token); }
    break;

  case 171:
#line 728 "ldgram.y"
    {
		  (yyval.fill) = exp_get_fill ((yyvsp[(1) - (1)].etree), 0, "fill value");
		}
    break;

  case 172:
#line 735 "ldgram.y"
    { (yyval.fill) = (yyvsp[(2) - (2)].fill); }
    break;

  case 173:
#line 736 "ldgram.y"
    { (yyval.fill) = (fill_type *) 0; }
    break;

  case 174:
#line 741 "ldgram.y"
    { (yyval.token) = '+'; }
    break;

  case 175:
#line 743 "ldgram.y"
    { (yyval.token) = '-'; }
    break;

  case 176:
#line 745 "ldgram.y"
    { (yyval.token) = '*'; }
    break;

  case 177:
#line 747 "ldgram.y"
    { (yyval.token) = '/'; }
    break;

  case 178:
#line 749 "ldgram.y"
    { (yyval.token) = LSHIFT; }
    break;

  case 179:
#line 751 "ldgram.y"
    { (yyval.token) = RSHIFT; }
    break;

  case 180:
#line 753 "ldgram.y"
    { (yyval.token) = '&'; }
    break;

  case 181:
#line 755 "ldgram.y"
    { (yyval.token) = '|'; }
    break;

  case 184:
#line 765 "ldgram.y"
    {
		  lang_add_assignment (exp_assign ((yyvsp[(1) - (3)].name), (yyvsp[(3) - (3)].etree), FALSE));
		}
    break;

  case 185:
#line 769 "ldgram.y"
    {
		  lang_add_assignment (exp_assign ((yyvsp[(1) - (3)].name),
						   exp_binop ((yyvsp[(2) - (3)].token),
							      exp_nameop (NAME,
									  (yyvsp[(1) - (3)].name)),
							      (yyvsp[(3) - (3)].etree)), FALSE));
		}
    break;

  case 186:
#line 777 "ldgram.y"
    {
		  lang_add_assignment (exp_assign ((yyvsp[(3) - (6)].name), (yyvsp[(5) - (6)].etree), TRUE));
		}
    break;

  case 187:
#line 781 "ldgram.y"
    {
		  lang_add_assignment (exp_provide ((yyvsp[(3) - (6)].name), (yyvsp[(5) - (6)].etree), FALSE));
		}
    break;

  case 188:
#line 785 "ldgram.y"
    {
		  lang_add_assignment (exp_provide ((yyvsp[(3) - (6)].name), (yyvsp[(5) - (6)].etree), TRUE));
		}
    break;

  case 196:
#line 808 "ldgram.y"
    { region = lang_memory_region_lookup ((yyvsp[(1) - (1)].name), TRUE); }
    break;

  case 197:
#line 811 "ldgram.y"
    {}
    break;

  case 198:
#line 813 "ldgram.y"
    { ldlex_script (); ldfile_open_command_file((yyvsp[(2) - (2)].name)); }
    break;

  case 199:
#line 815 "ldgram.y"
    { ldlex_popstate (); }
    break;

  case 200:
#line 820 "ldgram.y"
    {
		  region->origin_exp = (yyvsp[(3) - (3)].etree);
		  region->current = region->origin;
		}
    break;

  case 201:
#line 828 "ldgram.y"
    {
		  region->length_exp = (yyvsp[(3) - (3)].etree);
		}
    break;

  case 202:
#line 835 "ldgram.y"
    { /* dummy action to avoid bison 1.25 error message */ }
    break;

  case 206:
#line 846 "ldgram.y"
    { lang_set_flags (region, (yyvsp[(1) - (1)].name), 0); }
    break;

  case 207:
#line 848 "ldgram.y"
    { lang_set_flags (region, (yyvsp[(2) - (2)].name), 1); }
    break;

  case 208:
#line 853 "ldgram.y"
    { lang_startup((yyvsp[(3) - (4)].name)); }
    break;

  case 210:
#line 859 "ldgram.y"
    { ldemul_hll((char *)NULL); }
    break;

  case 211:
#line 864 "ldgram.y"
    { ldemul_hll((yyvsp[(3) - (3)].name)); }
    break;

  case 212:
#line 866 "ldgram.y"
    { ldemul_hll((yyvsp[(1) - (1)].name)); }
    break;

  case 214:
#line 874 "ldgram.y"
    { ldemul_syslib((yyvsp[(3) - (3)].name)); }
    break;

  case 216:
#line 880 "ldgram.y"
    { lang_float(TRUE); }
    break;

  case 217:
#line 882 "ldgram.y"
    { lang_float(FALSE); }
    break;

  case 218:
#line 887 "ldgram.y"
    {
		  (yyval.nocrossref) = NULL;
		}
    break;

  case 219:
#line 891 "ldgram.y"
    {
		  struct lang_nocrossref *n;

		  n = (struct lang_nocrossref *) xmalloc (sizeof *n);
		  n->name = (yyvsp[(1) - (2)].name);
		  n->next = (yyvsp[(2) - (2)].nocrossref);
		  (yyval.nocrossref) = n;
		}
    break;

  case 220:
#line 900 "ldgram.y"
    {
		  struct lang_nocrossref *n;

		  n = (struct lang_nocrossref *) xmalloc (sizeof *n);
		  n->name = (yyvsp[(1) - (3)].name);
		  n->next = (yyvsp[(3) - (3)].nocrossref);
		  (yyval.nocrossref) = n;
		}
    break;

  case 221:
#line 910 "ldgram.y"
    { ldlex_expression (); }
    break;

  case 222:
#line 912 "ldgram.y"
    { ldlex_popstate (); (yyval.etree)=(yyvsp[(2) - (2)].etree);}
    break;

  case 223:
#line 917 "ldgram.y"
    { (yyval.etree) = exp_unop ('-', (yyvsp[(2) - (2)].etree)); }
    break;

  case 224:
#line 919 "ldgram.y"
    { (yyval.etree) = (yyvsp[(2) - (3)].etree); }
    break;

  case 225:
#line 921 "ldgram.y"
    { (yyval.etree) = exp_unop ((int) (yyvsp[(1) - (4)].integer),(yyvsp[(3) - (4)].etree)); }
    break;

  case 226:
#line 923 "ldgram.y"
    { (yyval.etree) = exp_unop ('!', (yyvsp[(2) - (2)].etree)); }
    break;

  case 227:
#line 925 "ldgram.y"
    { (yyval.etree) = (yyvsp[(2) - (2)].etree); }
    break;

  case 228:
#line 927 "ldgram.y"
    { (yyval.etree) = exp_unop ('~', (yyvsp[(2) - (2)].etree));}
    break;

  case 229:
#line 930 "ldgram.y"
    { (yyval.etree) = exp_binop ('*', (yyvsp[(1) - (3)].etree), (yyvsp[(3) - (3)].etree)); }
    break;

  case 230:
#line 932 "ldgram.y"
    { (yyval.etree) = exp_binop ('/', (yyvsp[(1) - (3)].etree), (yyvsp[(3) - (3)].etree)); }
    break;

  case 231:
#line 934 "ldgram.y"
    { (yyval.etree) = exp_binop ('%', (yyvsp[(1) - (3)].etree), (yyvsp[(3) - (3)].etree)); }
    break;

  case 232:
#line 936 "ldgram.y"
    { (yyval.etree) = exp_binop ('+', (yyvsp[(1) - (3)].etree), (yyvsp[(3) - (3)].etree)); }
    break;

  case 233:
#line 938 "ldgram.y"
    { (yyval.etree) = exp_binop ('-' , (yyvsp[(1) - (3)].etree), (yyvsp[(3) - (3)].etree)); }
    break;

  case 234:
#line 940 "ldgram.y"
    { (yyval.etree) = exp_binop (LSHIFT , (yyvsp[(1) - (3)].etree), (yyvsp[(3) - (3)].etree)); }
    break;

  case 235:
#line 942 "ldgram.y"
    { (yyval.etree) = exp_binop (RSHIFT , (yyvsp[(1) - (3)].etree), (yyvsp[(3) - (3)].etree)); }
    break;

  case 236:
#line 944 "ldgram.y"
    { (yyval.etree) = exp_binop (EQ , (yyvsp[(1) - (3)].etree), (yyvsp[(3) - (3)].etree)); }
    break;

  case 237:
#line 946 "ldgram.y"
    { (yyval.etree) = exp_binop (NE , (yyvsp[(1) - (3)].etree), (yyvsp[(3) - (3)].etree)); }
    break;

  case 238:
#line 948 "ldgram.y"
    { (yyval.etree) = exp_binop (LE , (yyvsp[(1) - (3)].etree), (yyvsp[(3) - (3)].etree)); }
    break;

  case 239:
#line 950 "ldgram.y"
    { (yyval.etree) = exp_binop (GE , (yyvsp[(1) - (3)].etree), (yyvsp[(3) - (3)].etree)); }
    break;

  case 240:
#line 952 "ldgram.y"
    { (yyval.etree) = exp_binop ('<' , (yyvsp[(1) - (3)].etree), (yyvsp[(3) - (3)].etree)); }
    break;

  case 241:
#line 954 "ldgram.y"
    { (yyval.etree) = exp_binop ('>' , (yyvsp[(1) - (3)].etree), (yyvsp[(3) - (3)].etree)); }
    break;

  case 242:
#line 956 "ldgram.y"
    { (yyval.etree) = exp_binop ('&' , (yyvsp[(1) - (3)].etree), (yyvsp[(3) - (3)].etree)); }
    break;

  case 243:
#line 958 "ldgram.y"
    { (yyval.etree) = exp_binop ('^' , (yyvsp[(1) - (3)].etree), (yyvsp[(3) - (3)].etree)); }
    break;

  case 244:
#line 960 "ldgram.y"
    { (yyval.etree) = exp_binop ('|' , (yyvsp[(1) - (3)].etree), (yyvsp[(3) - (3)].etree)); }
    break;

  case 245:
#line 962 "ldgram.y"
    { (yyval.etree) = exp_trinop ('?' , (yyvsp[(1) - (5)].etree), (yyvsp[(3) - (5)].etree), (yyvsp[(5) - (5)].etree)); }
    break;

  case 246:
#line 964 "ldgram.y"
    { (yyval.etree) = exp_binop (ANDAND , (yyvsp[(1) - (3)].etree), (yyvsp[(3) - (3)].etree)); }
    break;

  case 247:
#line 966 "ldgram.y"
    { (yyval.etree) = exp_binop (OROR , (yyvsp[(1) - (3)].etree), (yyvsp[(3) - (3)].etree)); }
    break;

  case 248:
#line 968 "ldgram.y"
    { (yyval.etree) = exp_nameop (DEFINED, (yyvsp[(3) - (4)].name)); }
    break;

  case 249:
#line 970 "ldgram.y"
    { (yyval.etree) = exp_bigintop ((yyvsp[(1) - (1)].bigint).integer, (yyvsp[(1) - (1)].bigint).str); }
    break;

  case 250:
#line 972 "ldgram.y"
    { (yyval.etree) = exp_nameop (SIZEOF_HEADERS,0); }
    break;

  case 251:
#line 975 "ldgram.y"
    { (yyval.etree) = exp_nameop (ALIGNOF,(yyvsp[(3) - (4)].name)); }
    break;

  case 252:
#line 977 "ldgram.y"
    { (yyval.etree) = exp_nameop (SIZEOF,(yyvsp[(3) - (4)].name)); }
    break;

  case 253:
#line 979 "ldgram.y"
    { (yyval.etree) = exp_nameop (ADDR,(yyvsp[(3) - (4)].name)); }
    break;

  case 254:
#line 981 "ldgram.y"
    { (yyval.etree) = exp_nameop (LOADADDR,(yyvsp[(3) - (4)].name)); }
    break;

  case 255:
#line 983 "ldgram.y"
    { (yyval.etree) = exp_nameop (CONSTANT,(yyvsp[(3) - (4)].name)); }
    break;

  case 256:
#line 985 "ldgram.y"
    { (yyval.etree) = exp_unop (ABSOLUTE, (yyvsp[(3) - (4)].etree)); }
    break;

  case 257:
#line 987 "ldgram.y"
    { (yyval.etree) = exp_unop (ALIGN_K,(yyvsp[(3) - (4)].etree)); }
    break;

  case 258:
#line 989 "ldgram.y"
    { (yyval.etree) = exp_binop (ALIGN_K,(yyvsp[(3) - (6)].etree),(yyvsp[(5) - (6)].etree)); }
    break;

  case 259:
#line 991 "ldgram.y"
    { (yyval.etree) = exp_binop (DATA_SEGMENT_ALIGN, (yyvsp[(3) - (6)].etree), (yyvsp[(5) - (6)].etree)); }
    break;

  case 260:
#line 993 "ldgram.y"
    { (yyval.etree) = exp_binop (DATA_SEGMENT_RELRO_END, (yyvsp[(5) - (6)].etree), (yyvsp[(3) - (6)].etree)); }
    break;

  case 261:
#line 995 "ldgram.y"
    { (yyval.etree) = exp_unop (DATA_SEGMENT_END, (yyvsp[(3) - (4)].etree)); }
    break;

  case 262:
#line 997 "ldgram.y"
    { /* The operands to the expression node are
			     placed in the opposite order from the way
			     in which they appear in the script as
			     that allows us to reuse more code in
			     fold_binary.  */
			  (yyval.etree) = exp_binop (SEGMENT_START,
					  (yyvsp[(5) - (6)].etree),
					  exp_nameop (NAME, (yyvsp[(3) - (6)].name))); }
    break;

  case 263:
#line 1006 "ldgram.y"
    { (yyval.etree) = exp_unop (ALIGN_K,(yyvsp[(3) - (4)].etree)); }
    break;

  case 264:
#line 1008 "ldgram.y"
    { (yyval.etree) = exp_nameop (NAME,(yyvsp[(1) - (1)].name)); }
    break;

  case 265:
#line 1010 "ldgram.y"
    { (yyval.etree) = exp_binop (MAX_K, (yyvsp[(3) - (6)].etree), (yyvsp[(5) - (6)].etree) ); }
    break;

  case 266:
#line 1012 "ldgram.y"
    { (yyval.etree) = exp_binop (MIN_K, (yyvsp[(3) - (6)].etree), (yyvsp[(5) - (6)].etree) ); }
    break;

  case 267:
#line 1014 "ldgram.y"
    { (yyval.etree) = exp_assert ((yyvsp[(3) - (6)].etree), (yyvsp[(5) - (6)].name)); }
    break;

  case 268:
#line 1016 "ldgram.y"
    { (yyval.etree) = exp_nameop (ORIGIN, (yyvsp[(3) - (4)].name)); }
    break;

  case 269:
#line 1018 "ldgram.y"
    { (yyval.etree) = exp_nameop (LENGTH, (yyvsp[(3) - (4)].name)); }
    break;

  case 270:
#line 1020 "ldgram.y"
    { (yyval.etree) = exp_unop (LOG2CEIL, (yyvsp[(3) - (4)].etree)); }
    break;

  case 271:
#line 1025 "ldgram.y"
    { (yyval.name) = (yyvsp[(3) - (3)].name); }
    break;

  case 272:
#line 1026 "ldgram.y"
    { (yyval.name) = 0; }
    break;

  case 273:
#line 1030 "ldgram.y"
    { (yyval.etree) = (yyvsp[(3) - (4)].etree); }
    break;

  case 274:
#line 1031 "ldgram.y"
    { (yyval.etree) = 0; }
    break;

  case 275:
#line 1035 "ldgram.y"
    { (yyval.etree) = (yyvsp[(3) - (4)].etree); }
    break;

  case 276:
#line 1036 "ldgram.y"
    { (yyval.etree) = 0; }
    break;

  case 277:
#line 1040 "ldgram.y"
    { (yyval.token) = ALIGN_WITH_INPUT; }
    break;

  case 278:
#line 1041 "ldgram.y"
    { (yyval.token) = 0; }
    break;

  case 279:
#line 1045 "ldgram.y"
    { (yyval.etree) = (yyvsp[(3) - (4)].etree); }
    break;

  case 280:
#line 1046 "ldgram.y"
    { (yyval.etree) = 0; }
    break;

  case 281:
#line 1050 "ldgram.y"
    { (yyval.token) = ONLY_IF_RO; }
    break;

  case 282:
#line 1051 "ldgram.y"
    { (yyval.token) = ONLY_IF_RW; }
    break;

  case 283:
#line 1052 "ldgram.y"
    { (yyval.token) = SPECIAL; }
    break;

  case 284:
#line 1053 "ldgram.y"
    { (yyval.token) = 0; }
    break;

  case 285:
#line 1056 "ldgram.y"
    { ldlex_expression(); }
    break;

  case 286:
#line 1061 "ldgram.y"
    { ldlex_popstate (); ldlex_script (); }
    break;

  case 287:
#line 1064 "ldgram.y"
    {
			  lang_enter_output_section_statement((yyvsp[(1) - (10)].name), (yyvsp[(3) - (10)].etree),
							      sectype,
							      (yyvsp[(5) - (10)].etree), (yyvsp[(7) - (10)].etree), (yyvsp[(4) - (10)].etree), (yyvsp[(9) - (10)].token), (yyvsp[(6) - (10)].token));
			}
    break;

  case 288:
#line 1070 "ldgram.y"
    { ldlex_popstate (); ldlex_expression (); }
    break;

  case 289:
#line 1072 "ldgram.y"
    {
		  ldlex_popstate ();
		  lang_leave_output_section_statement ((yyvsp[(18) - (18)].fill), (yyvsp[(15) - (18)].name), (yyvsp[(17) - (18)].section_phdr), (yyvsp[(16) - (18)].name));
		}
    break;

  case 290:
#line 1077 "ldgram.y"
    {}
    break;

  case 291:
#line 1079 "ldgram.y"
    { ldlex_expression (); }
    break;

  case 292:
#line 1081 "ldgram.y"
    { ldlex_popstate (); ldlex_script (); }
    break;

  case 293:
#line 1083 "ldgram.y"
    {
			  lang_enter_overlay ((yyvsp[(3) - (8)].etree), (yyvsp[(6) - (8)].etree));
			}
    break;

  case 294:
#line 1088 "ldgram.y"
    { ldlex_popstate (); ldlex_expression (); }
    break;

  case 295:
#line 1090 "ldgram.y"
    {
			  ldlex_popstate ();
			  lang_leave_overlay ((yyvsp[(5) - (16)].etree), (int) (yyvsp[(4) - (16)].integer),
					      (yyvsp[(16) - (16)].fill), (yyvsp[(13) - (16)].name), (yyvsp[(15) - (16)].section_phdr), (yyvsp[(14) - (16)].name));
			}
    break;

  case 297:
#line 1100 "ldgram.y"
    { ldlex_expression (); }
    break;

  case 298:
#line 1102 "ldgram.y"
    {
		  ldlex_popstate ();
		  lang_add_assignment (exp_assign (".", (yyvsp[(3) - (3)].etree), FALSE));
		}
    break;

  case 300:
#line 1108 "ldgram.y"
    { ldlex_script (); ldfile_open_command_file((yyvsp[(2) - (2)].name)); }
    break;

  case 301:
#line 1110 "ldgram.y"
    { ldlex_popstate (); }
    break;

  case 302:
#line 1114 "ldgram.y"
    { sectype = noload_section; }
    break;

  case 303:
#line 1115 "ldgram.y"
    { sectype = noalloc_section; }
    break;

  case 304:
#line 1116 "ldgram.y"
    { sectype = noalloc_section; }
    break;

  case 305:
#line 1117 "ldgram.y"
    { sectype = noalloc_section; }
    break;

  case 306:
#line 1118 "ldgram.y"
    { sectype = noalloc_section; }
    break;

  case 308:
#line 1123 "ldgram.y"
    { sectype = normal_section; }
    break;

  case 309:
#line 1124 "ldgram.y"
    { sectype = normal_section; }
    break;

  case 310:
#line 1128 "ldgram.y"
    { (yyval.etree) = (yyvsp[(1) - (3)].etree); }
    break;

  case 311:
#line 1129 "ldgram.y"
    { (yyval.etree) = (etree_type *)NULL;  }
    break;

  case 312:
#line 1134 "ldgram.y"
    { (yyval.etree) = (yyvsp[(3) - (6)].etree); }
    break;

  case 313:
#line 1136 "ldgram.y"
    { (yyval.etree) = (yyvsp[(3) - (10)].etree); }
    break;

  case 314:
#line 1140 "ldgram.y"
    { (yyval.etree) = (yyvsp[(1) - (2)].etree); }
    break;

  case 315:
#line 1141 "ldgram.y"
    { (yyval.etree) = (etree_type *) NULL;  }
    break;

  case 316:
#line 1146 "ldgram.y"
    { (yyval.integer) = 0; }
    break;

  case 317:
#line 1148 "ldgram.y"
    { (yyval.integer) = 1; }
    break;

  case 318:
#line 1153 "ldgram.y"
    { (yyval.name) = (yyvsp[(2) - (2)].name); }
    break;

  case 319:
#line 1154 "ldgram.y"
    { (yyval.name) = DEFAULT_MEMORY_REGION; }
    break;

  case 320:
#line 1159 "ldgram.y"
    {
		  (yyval.section_phdr) = NULL;
		}
    break;

  case 321:
#line 1163 "ldgram.y"
    {
		  struct lang_output_section_phdr_list *n;

		  n = ((struct lang_output_section_phdr_list *)
		       xmalloc (sizeof *n));
		  n->name = (yyvsp[(3) - (3)].name);
		  n->used = FALSE;
		  n->next = (yyvsp[(1) - (3)].section_phdr);
		  (yyval.section_phdr) = n;
		}
    break;

  case 323:
#line 1179 "ldgram.y"
    {
			  ldlex_script ();
			  lang_enter_overlay_section ((yyvsp[(2) - (2)].name));
			}
    break;

  case 324:
#line 1184 "ldgram.y"
    { ldlex_popstate (); ldlex_expression (); }
    break;

  case 325:
#line 1186 "ldgram.y"
    {
			  ldlex_popstate ();
			  lang_leave_overlay_section ((yyvsp[(9) - (9)].fill), (yyvsp[(8) - (9)].section_phdr));
			}
    break;

  case 330:
#line 1203 "ldgram.y"
    { ldlex_expression (); }
    break;

  case 331:
#line 1204 "ldgram.y"
    { ldlex_popstate (); }
    break;

  case 332:
#line 1206 "ldgram.y"
    {
		  lang_new_phdr ((yyvsp[(1) - (6)].name), (yyvsp[(3) - (6)].etree), (yyvsp[(4) - (6)].phdr).filehdr, (yyvsp[(4) - (6)].phdr).phdrs, (yyvsp[(4) - (6)].phdr).at,
				 (yyvsp[(4) - (6)].phdr).flags);
		}
    break;

  case 333:
#line 1214 "ldgram.y"
    {
		  (yyval.etree) = (yyvsp[(1) - (1)].etree);

		  if ((yyvsp[(1) - (1)].etree)->type.node_class == etree_name
		      && (yyvsp[(1) - (1)].etree)->type.node_code == NAME)
		    {
		      const char *s;
		      unsigned int i;
		      static const char * const phdr_types[] =
			{
			  "PT_NULL", "PT_LOAD", "PT_DYNAMIC",
			  "PT_INTERP", "PT_NOTE", "PT_SHLIB",
			  "PT_PHDR", "PT_TLS"
			};

		      s = (yyvsp[(1) - (1)].etree)->name.name;
		      for (i = 0;
			   i < sizeof phdr_types / sizeof phdr_types[0];
			   i++)
			if (strcmp (s, phdr_types[i]) == 0)
			  {
			    (yyval.etree) = exp_intop (i);
			    break;
			  }
		      if (i == sizeof phdr_types / sizeof phdr_types[0])
			{
			  if (strcmp (s, "PT_GNU_EH_FRAME") == 0)
			    (yyval.etree) = exp_intop (0x6474e550);
			  else if (strcmp (s, "PT_GNU_STACK") == 0)
			    (yyval.etree) = exp_intop (0x6474e551);
			  else
			    {
			      einfo (_("\
%X%P:%S: unknown phdr type `%s' (try integer literal)\n"),
				     NULL, s);
			      (yyval.etree) = exp_intop (0);
			    }
			}
		    }
		}
    break;

  case 334:
#line 1258 "ldgram.y"
    {
		  memset (&(yyval.phdr), 0, sizeof (struct phdr_info));
		}
    break;

  case 335:
#line 1262 "ldgram.y"
    {
		  (yyval.phdr) = (yyvsp[(3) - (3)].phdr);
		  if (strcmp ((yyvsp[(1) - (3)].name), "FILEHDR") == 0 && (yyvsp[(2) - (3)].etree) == NULL)
		    (yyval.phdr).filehdr = TRUE;
		  else if (strcmp ((yyvsp[(1) - (3)].name), "PHDRS") == 0 && (yyvsp[(2) - (3)].etree) == NULL)
		    (yyval.phdr).phdrs = TRUE;
		  else if (strcmp ((yyvsp[(1) - (3)].name), "FLAGS") == 0 && (yyvsp[(2) - (3)].etree) != NULL)
		    (yyval.phdr).flags = (yyvsp[(2) - (3)].etree);
		  else
		    einfo (_("%X%P:%S: PHDRS syntax error at `%s'\n"),
			   NULL, (yyvsp[(1) - (3)].name));
		}
    break;

  case 336:
#line 1275 "ldgram.y"
    {
		  (yyval.phdr) = (yyvsp[(5) - (5)].phdr);
		  (yyval.phdr).at = (yyvsp[(3) - (5)].etree);
		}
    break;

  case 337:
#line 1283 "ldgram.y"
    {
		  (yyval.etree) = NULL;
		}
    break;

  case 338:
#line 1287 "ldgram.y"
    {
		  (yyval.etree) = (yyvsp[(2) - (3)].etree);
		}
    break;

  case 339:
#line 1293 "ldgram.y"
    {
		  ldlex_version_file ();
		  PUSH_ERROR (_("dynamic list"));
		}
    break;

  case 340:
#line 1298 "ldgram.y"
    {
		  ldlex_popstate ();
		  POP_ERROR ();
		}
    break;

  case 344:
#line 1315 "ldgram.y"
    {
		  lang_append_dynamic_list ((yyvsp[(1) - (2)].versyms));
		}
    break;

  case 345:
#line 1323 "ldgram.y"
    {
		  ldlex_version_file ();
		  PUSH_ERROR (_("VERSION script"));
		}
    break;

  case 346:
#line 1328 "ldgram.y"
    {
		  ldlex_popstate ();
		  POP_ERROR ();
		}
    break;

  case 347:
#line 1337 "ldgram.y"
    {
		  ldlex_version_script ();
		}
    break;

  case 348:
#line 1341 "ldgram.y"
    {
		  ldlex_popstate ();
		}
    break;

  case 351:
#line 1353 "ldgram.y"
    {
		  lang_register_vers_node (NULL, (yyvsp[(2) - (4)].versnode), NULL);
		}
    break;

  case 352:
#line 1357 "ldgram.y"
    {
		  lang_register_vers_node ((yyvsp[(1) - (5)].name), (yyvsp[(3) - (5)].versnode), NULL);
		}
    break;

  case 353:
#line 1361 "ldgram.y"
    {
		  lang_register_vers_node ((yyvsp[(1) - (6)].name), (yyvsp[(3) - (6)].versnode), (yyvsp[(5) - (6)].deflist));
		}
    break;

  case 354:
#line 1368 "ldgram.y"
    {
		  (yyval.deflist) = lang_add_vers_depend (NULL, (yyvsp[(1) - (1)].name));
		}
    break;

  case 355:
#line 1372 "ldgram.y"
    {
		  (yyval.deflist) = lang_add_vers_depend ((yyvsp[(1) - (2)].deflist), (yyvsp[(2) - (2)].name));
		}
    break;

  case 356:
#line 1379 "ldgram.y"
    {
		  (yyval.versnode) = lang_new_vers_node (NULL, NULL);
		}
    break;

  case 357:
#line 1383 "ldgram.y"
    {
		  (yyval.versnode) = lang_new_vers_node ((yyvsp[(1) - (2)].versyms), NULL);
		}
    break;

  case 358:
#line 1387 "ldgram.y"
    {
		  (yyval.versnode) = lang_new_vers_node ((yyvsp[(3) - (4)].versyms), NULL);
		}
    break;

  case 359:
#line 1391 "ldgram.y"
    {
		  (yyval.versnode) = lang_new_vers_node (NULL, (yyvsp[(3) - (4)].versyms));
		}
    break;

  case 360:
#line 1395 "ldgram.y"
    {
		  (yyval.versnode) = lang_new_vers_node ((yyvsp[(3) - (8)].versyms), (yyvsp[(7) - (8)].versyms));
		}
    break;

  case 361:
#line 1402 "ldgram.y"
    {
		  (yyval.versyms) = lang_new_vers_pattern (NULL, (yyvsp[(1) - (1)].name), ldgram_vers_current_lang, FALSE);
		}
    break;

  case 362:
#line 1406 "ldgram.y"
    {
		  (yyval.versyms) = lang_new_vers_pattern (NULL, (yyvsp[(1) - (1)].name), ldgram_vers_current_lang, TRUE);
		}
    break;

  case 363:
#line 1410 "ldgram.y"
    {
		  (yyval.versyms) = lang_new_vers_pattern ((yyvsp[(1) - (3)].versyms), (yyvsp[(3) - (3)].name), ldgram_vers_current_lang, FALSE);
		}
    break;

  case 364:
#line 1414 "ldgram.y"
    {
		  (yyval.versyms) = lang_new_vers_pattern ((yyvsp[(1) - (3)].versyms), (yyvsp[(3) - (3)].name), ldgram_vers_current_lang, TRUE);
		}
    break;

  case 365:
#line 1418 "ldgram.y"
    {
			  (yyval.name) = ldgram_vers_current_lang;
			  ldgram_vers_current_lang = (yyvsp[(4) - (5)].name);
			}
    break;

  case 366:
#line 1423 "ldgram.y"
    {
			  struct bfd_elf_version_expr *pat;
			  for (pat = (yyvsp[(7) - (9)].versyms); pat->next != NULL; pat = pat->next);
			  pat->next = (yyvsp[(1) - (9)].versyms);
			  (yyval.versyms) = (yyvsp[(7) - (9)].versyms);
			  ldgram_vers_current_lang = (yyvsp[(6) - (9)].name);
			}
    break;

  case 367:
#line 1431 "ldgram.y"
    {
			  (yyval.name) = ldgram_vers_current_lang;
			  ldgram_vers_current_lang = (yyvsp[(2) - (3)].name);
			}
    break;

  case 368:
#line 1436 "ldgram.y"
    {
			  (yyval.versyms) = (yyvsp[(5) - (7)].versyms);
			  ldgram_vers_current_lang = (yyvsp[(4) - (7)].name);
			}
    break;

  case 369:
#line 1441 "ldgram.y"
    {
		  (yyval.versyms) = lang_new_vers_pattern (NULL, "global", ldgram_vers_current_lang, FALSE);
		}
    break;

  case 370:
#line 1445 "ldgram.y"
    {
		  (yyval.versyms) = lang_new_vers_pattern ((yyvsp[(1) - (3)].versyms), "global", ldgram_vers_current_lang, FALSE);
		}
    break;

  case 371:
#line 1449 "ldgram.y"
    {
		  (yyval.versyms) = lang_new_vers_pattern (NULL, "local", ldgram_vers_current_lang, FALSE);
		}
    break;

  case 372:
#line 1453 "ldgram.y"
    {
		  (yyval.versyms) = lang_new_vers_pattern ((yyvsp[(1) - (3)].versyms), "local", ldgram_vers_current_lang, FALSE);
		}
    break;

  case 373:
#line 1457 "ldgram.y"
    {
		  (yyval.versyms) = lang_new_vers_pattern (NULL, "extern", ldgram_vers_current_lang, FALSE);
		}
    break;

  case 374:
#line 1461 "ldgram.y"
    {
		  (yyval.versyms) = lang_new_vers_pattern ((yyvsp[(1) - (3)].versyms), "extern", ldgram_vers_current_lang, FALSE);
		}
    break;


/* Line 1267 of yacc.c.  */
#line 4504 "ldgram.c"
      default: break;
    }
  YY_SYMBOL_PRINT ("-> $$ =", yyr1[yyn], &yyval, &yyloc);

  YYPOPSTACK (yylen);
  yylen = 0;
  YY_STACK_PRINT (yyss, yyssp);

  *++yyvsp = yyval;


  /* Now `shift' the result of the reduction.  Determine what state
     that goes to, based on the state we popped back to and the rule
     number reduced by.  */

  yyn = yyr1[yyn];

  yystate = yypgoto[yyn - YYNTOKENS] + *yyssp;
  if (0 <= yystate && yystate <= YYLAST && yycheck[yystate] == *yyssp)
    yystate = yytable[yystate];
  else
    yystate = yydefgoto[yyn - YYNTOKENS];

  goto yynewstate;


/*------------------------------------.
| yyerrlab -- here on detecting error |
`------------------------------------*/
yyerrlab:
  /* If not already recovering from an error, report this error.  */
  if (!yyerrstatus)
    {
      ++yynerrs;
#if ! YYERROR_VERBOSE
      yyerror (YY_("syntax error"));
#else
      {
	YYSIZE_T yysize = yysyntax_error (0, yystate, yychar);
	if (yymsg_alloc < yysize && yymsg_alloc < YYSTACK_ALLOC_MAXIMUM)
	  {
	    YYSIZE_T yyalloc = 2 * yysize;
	    if (! (yysize <= yyalloc && yyalloc <= YYSTACK_ALLOC_MAXIMUM))
	      yyalloc = YYSTACK_ALLOC_MAXIMUM;
	    if (yymsg != yymsgbuf)
	      YYSTACK_FREE (yymsg);
	    yymsg = (char *) YYSTACK_ALLOC (yyalloc);
	    if (yymsg)
	      yymsg_alloc = yyalloc;
	    else
	      {
		yymsg = yymsgbuf;
		yymsg_alloc = sizeof yymsgbuf;
	      }
	  }

	if (0 < yysize && yysize <= yymsg_alloc)
	  {
	    (void) yysyntax_error (yymsg, yystate, yychar);
	    yyerror (yymsg);
	  }
	else
	  {
	    yyerror (YY_("syntax error"));
	    if (yysize != 0)
	      goto yyexhaustedlab;
	  }
      }
#endif
    }



  if (yyerrstatus == 3)
    {
      /* If just tried and failed to reuse look-ahead token after an
	 error, discard it.  */

      if (yychar <= YYEOF)
	{
	  /* Return failure if at end of input.  */
	  if (yychar == YYEOF)
	    YYABORT;
	}
      else
	{
	  yydestruct ("Error: discarding",
		      yytoken, &yylval);
	  yychar = YYEMPTY;
	}
    }

  /* Else will try to reuse look-ahead token after shifting the error
     token.  */
  goto yyerrlab1;


/*---------------------------------------------------.
| yyerrorlab -- error raised explicitly by YYERROR.  |
`---------------------------------------------------*/
yyerrorlab:

  /* Pacify compilers like GCC when the user code never invokes
     YYERROR and the label yyerrorlab therefore never appears in user
     code.  */
  if (/*CONSTCOND*/ 0)
     goto yyerrorlab;

  /* Do not reclaim the symbols of the rule which action triggered
     this YYERROR.  */
  YYPOPSTACK (yylen);
  yylen = 0;
  YY_STACK_PRINT (yyss, yyssp);
  yystate = *yyssp;
  goto yyerrlab1;


/*-------------------------------------------------------------.
| yyerrlab1 -- common code for both syntax error and YYERROR.  |
`-------------------------------------------------------------*/
yyerrlab1:
  yyerrstatus = 3;	/* Each real token shifted decrements this.  */

  for (;;)
    {
      yyn = yypact[yystate];
      if (yyn != YYPACT_NINF)
	{
	  yyn += YYTERROR;
	  if (0 <= yyn && yyn <= YYLAST && yycheck[yyn] == YYTERROR)
	    {
	      yyn = yytable[yyn];
	      if (0 < yyn)
		break;
	    }
	}

      /* Pop the current state because it cannot handle the error token.  */
      if (yyssp == yyss)
	YYABORT;


      yydestruct ("Error: popping",
		  yystos[yystate], yyvsp);
      YYPOPSTACK (1);
      yystate = *yyssp;
      YY_STACK_PRINT (yyss, yyssp);
    }

  if (yyn == YYFINAL)
    YYACCEPT;

  *++yyvsp = yylval;


  /* Shift the error token.  */
  YY_SYMBOL_PRINT ("Shifting", yystos[yyn], yyvsp, yylsp);

  yystate = yyn;
  goto yynewstate;


/*-------------------------------------.
| yyacceptlab -- YYACCEPT comes here.  |
`-------------------------------------*/
yyacceptlab:
  yyresult = 0;
  goto yyreturn;

/*-----------------------------------.
| yyabortlab -- YYABORT comes here.  |
`-----------------------------------*/
yyabortlab:
  yyresult = 1;
  goto yyreturn;

#ifndef yyoverflow
/*-------------------------------------------------.
| yyexhaustedlab -- memory exhaustion comes here.  |
`-------------------------------------------------*/
yyexhaustedlab:
  yyerror (YY_("memory exhausted"));
  yyresult = 2;
  /* Fall through.  */
#endif

yyreturn:
  if (yychar != YYEOF && yychar != YYEMPTY)
     yydestruct ("Cleanup: discarding lookahead",
		 yytoken, &yylval);
  /* Do not reclaim the symbols of the rule which action triggered
     this YYABORT or YYACCEPT.  */
  YYPOPSTACK (yylen);
  YY_STACK_PRINT (yyss, yyssp);
  while (yyssp != yyss)
    {
      yydestruct ("Cleanup: popping",
		  yystos[*yyssp], yyvsp);
      YYPOPSTACK (1);
    }
#ifndef yyoverflow
  if (yyss != yyssa)
    YYSTACK_FREE (yyss);
#endif
#if YYERROR_VERBOSE
  if (yymsg != yymsgbuf)
    YYSTACK_FREE (yymsg);
#endif
  /* Make sure YYID is used.  */
  return YYID (yyresult);
}


#line 1471 "ldgram.y"

void
yyerror(arg)
     const char *arg;
{
  if (ldfile_assumed_script)
    einfo (_("%P:%s: file format not recognized; treating as linker script\n"),
	   ldlex_filename ());
  if (error_index > 0 && error_index < ERROR_NAME_MAX)
    einfo ("%P%F:%S: %s in %s\n", NULL, arg, error_names[error_index - 1]);
  else
    einfo ("%P%F:%S: %s\n", NULL, arg);
}

